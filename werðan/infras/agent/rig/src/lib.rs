//! Rig agent implementation adapting the unified [`agent::Agent`] trait.

use agent::Agent;
use agent::AgentError;
use agent::AgentInput;
use agent::AgentOutput;
use agent::AgentOutputStream;
use agent::AssistantContent;
use agent::AssistantMessage;
use agent::LocalTool;
use agent::Message;
use agent::OpenAiModel;
use agent::StreamEvent;
use agent::UserContent;
use futures_util::StreamExt;

/// Rig agent adapter implementing the unified [`Agent`] trait.
#[derive(Clone)]
pub struct RigAgent {
    /// Active OpenAI-compatible model configuration.
    model: OpenAiModel,
    /// Host tools registered with this agent.
    tools: Vec<LocalTool>,
}

impl RigAgent {
    /// Create a new [`RigAgent`] configured with the specified model.
    pub fn new(model: OpenAiModel) -> Self {
        Self {
            model,
            tools: Vec::new(),
        }
    }

    /// Access currently registered tools.
    pub fn tools(&self) -> &[LocalTool] {
        &self.tools
    }

    /// Access current model configuration.
    pub fn model(&self) -> &OpenAiModel {
        &self.model
    }

    /// Determine whether the agent should operate in offline/mock mode.
    fn is_offline(&self) -> bool {
        if std::env::var("WERTHAN_OFFLINE").map(|v| v == "1" || v == "true").unwrap_or(false) {
            return true;
        }
        match &self.model.api_key {
            None => true,
            Some(key) =>
                key.is_empty()
                    || key == "none"
                    || key == "test"
                    || key.starts_with("test-")
                    || key.starts_with("sk-test"),
        }
    }
}

/// Extract user or assistant text from a message payload.
fn extract_prompt(message: &Message) -> String {
    match message {
        Message::User {
            content,
        } => content
            .iter()
            .filter_map(|c| match c {
                UserContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" "),
        Message::Assistant(assistant_msg) => assistant_msg
            .content
            .iter()
            .filter_map(|c| match c {
                AssistantContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" "),
        Message::System {
            content,
        } => content.clone(),
    }
}

#[async_trait::async_trait]
impl Agent for RigAgent {
    async fn set_model(&mut self, model: OpenAiModel) -> Result<(), AgentError> {
        self.model = model;
        Ok(())
    }

    async fn register_tool(&mut self, tool: LocalTool) -> Result<(), AgentError> {
        self.tools.push(tool);
        Ok(())
    }

    async fn send(&self, input: AgentInput) -> Result<AgentOutputStream<'_>, AgentError> {
        // Convert &self.model into rig::providers::openai::OpenAIConfig
        let config: rig::providers::openai::OpenAIConfig = (&self.model).into();

        // Create dynamic tools via
        // rig::tool::DynamicTool::try_from(tool.clone())
        let dynamic_tools =
            self.tools.iter().cloned().map(rig::tool::DynamicTool::try_from).collect::<Result<Vec<_>, AgentError>>()?;

        let prompt_str = extract_prompt(&input.message);

        if self.is_offline() {
            let response_text = format!("Echo: {prompt_str}");
            let stream_event = serde_json::from_value::<StreamEvent>(serde_json::json!({
                "event": "text",
                "part": 0,
                "text": response_text.clone(),
            }))
            .map_err(AgentError::Serialization)?;

            let outputs: Vec<Result<AgentOutput, AgentError>> = vec![
                Ok(AgentOutput::Text(response_text.clone())),
                Ok(AgentOutput::Event(stream_event)),
                Ok(AgentOutput::Done(AssistantMessage::new(vec![AssistantContent::text(response_text)]))),
            ];

            return Ok(Box::pin(futures_util::stream::iter(outputs)));
        }

        // Online streaming completion path
        let client = config.client();
        let mut completion_request = rig::completion::CompletionRequest::new(input.message.clone());
        if let Some(temperature) = self.model.temperature {
            completion_request = completion_request.temperature(temperature);
        }
        if !dynamic_tools.is_empty() {
            let tool_defs = dynamic_tools.into_iter().map(|t| t.definition()).collect();
            completion_request = completion_request.tools(tool_defs);
        }

        let model = client.completion(&self.model.model);
        let stream = model.stream(completion_request).map_err(|e| AgentError::Provider(e.to_string()))?;

        struct StreamState {
            stream: rig::streaming::Streamed<rig::operation::Completion>,
            accumulated_text: String,
            pending: Vec<Result<AgentOutput, AgentError>>,
            finished: bool,
        }

        let initial_state = StreamState {
            stream,
            accumulated_text: String::new(),
            pending: Vec::new(),
            finished: false,
        };

        let output_stream = futures_util::stream::unfold(initial_state, |mut state| async move {
            loop {
                if !state.pending.is_empty() {
                    let next_item = state.pending.remove(0);
                    return Some((next_item, state));
                }

                if state.finished {
                    return None;
                }

                match state.stream.next().await {
                    Some(Ok(rig::streaming::Item::Event(event))) => {
                        if let StreamEvent::Text {
                            ref text,
                            ..
                        } = event
                        {
                            state.accumulated_text.push_str(text);
                            state.pending.push(Ok(AgentOutput::Text(text.clone())));
                        }
                        state.pending.push(Ok(AgentOutput::Event(event)));
                    },
                    Some(Ok(rig::streaming::Item::Unknown(_))) => {},
                    Some(Err(err)) => {
                        state.finished = true;
                        return Some((Err(AgentError::Provider(err.to_string())), state));
                    },
                    None => {
                        state.finished = true;
                        let done_message =
                            AssistantMessage::new(vec![AssistantContent::text(state.accumulated_text.clone())]);
                        return Some((Ok(AgentOutput::Done(done_message)), state));
                    },
                }
            }
        });

        Ok(Box::pin(output_stream))
    }
}

#[cfg(test)]
mod tests {
    use futures_util::StreamExt;
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_rig_agent_creation() {
        let model = OpenAiModel::new("gpt-4o");
        let agent = RigAgent::new(model);

        assert_eq!(agent.model().model, "gpt-4o");
        assert!(agent.tools().is_empty());
    }

    #[tokio::test]
    async fn test_rig_agent_set_model() {
        let model = OpenAiModel::new("gpt-4o");
        let mut agent = RigAgent::new(model);

        let new_model = OpenAiModel::new("claude-3-5-sonnet");
        agent.set_model(new_model.clone()).await.unwrap();
        assert_eq!(agent.model().model, "claude-3-5-sonnet");
    }

    #[tokio::test]
    async fn test_rig_agent_register_tool() {
        let model = OpenAiModel::new("gpt-4o");
        let mut agent = RigAgent::new(model);

        let tool = LocalTool::new(
            "calculator",
            "adds two numbers",
            json!({
                "type": "object",
                "properties": {
                    "a": { "type": "number" },
                    "b": { "type": "number" }
                }
            }),
            |args| async move {
                let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                Ok(json!({ "result": a + b }))
            },
        );

        agent.register_tool(tool).await.unwrap();
        assert_eq!(agent.tools().len(), 1);
        assert_eq!(agent.tools()[0].name, "calculator");
    }

    #[tokio::test]
    async fn test_rig_agent_stream_output_offline() {
        let model = OpenAiModel::new("gpt-4o");
        let agent = RigAgent::new(model);

        let mut stream = agent.prompt("Hello world").await.unwrap();

        let first = stream.next().await.unwrap().unwrap();
        match first {
            AgentOutput::Text(t) => assert_eq!(t, "Echo: Hello world"),
            other => panic!("Expected AgentOutput::Text, got {:?}", other),
        }

        let second = stream.next().await.unwrap().unwrap();
        match second {
            AgentOutput::Event(event) => match event {
                StreamEvent::Text {
                    text,
                    ..
                } => assert_eq!(text, "Echo: Hello world"),
                other => panic!("Expected StreamEvent::Text, got {:?}", other),
            },
            other => panic!("Expected AgentOutput::Event, got {:?}", other),
        }

        let third = stream.next().await.unwrap().unwrap();
        match third {
            AgentOutput::Done(msg) => {
                let text = msg
                    .content
                    .iter()
                    .filter_map(|c| match c {
                        AssistantContent::Text(t) => Some(t.text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                assert_eq!(text, "Echo: Hello world");
            },
            other => panic!("Expected AgentOutput::Done, got {:?}", other),
        }

        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_rig_agent_send_with_agent_input() {
        let model = OpenAiModel::new("gpt-4o").with_api_key("sk-test");
        let agent = RigAgent::new(model);

        let input = AgentInput::text("input testing");
        let mut stream = agent.send(input).await.unwrap();

        let first = stream.next().await.unwrap().unwrap();
        assert_eq!(first.as_text(), Some("Echo: input testing"));

        let second = stream.next().await.unwrap().unwrap();
        assert!(matches!(second, AgentOutput::Event(_)));

        let third = stream.next().await.unwrap().unwrap();
        assert!(third.is_done());

        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_rig_agent_invalid_tool_fails_send() {
        let model = OpenAiModel::new("gpt-4o");
        let mut agent = RigAgent::new(model);

        // Tool with empty name is rejected by rig's ToolName
        let invalid_tool = LocalTool::new("", "empty name tool", json!({}), |_| async move { Ok(json!({})) });

        agent.register_tool(invalid_tool).await.unwrap();

        let result = agent.prompt("test").await;
        assert!(result.is_err());
        match result {
            Err(AgentError::Tool(msg)) => {
                assert!(msg.contains("Invalid tool name"));
            },
            Err(other) => panic!("Expected AgentError::Tool, got {other:?}"),
            Ok(_) => panic!("Expected Err, got Ok"),
        }
    }

    #[tokio::test]
    async fn test_rig_agent_is_offline_conditions() {
        let agent_none = RigAgent::new(OpenAiModel::new("gpt-4o"));
        assert!(agent_none.is_offline());

        let agent_empty = RigAgent::new(OpenAiModel::new("gpt-4o").with_api_key(""));
        assert!(agent_empty.is_offline());

        let agent_default_none = RigAgent::new(OpenAiModel::new("gpt-4o").with_api_key("none"));
        assert!(agent_default_none.is_offline());

        let agent_sk_test = RigAgent::new(OpenAiModel::new("gpt-4o").with_api_key("sk-test"));
        assert!(agent_sk_test.is_offline());
    }
}
