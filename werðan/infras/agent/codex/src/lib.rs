//! Minimal Codex agent adapter implementing the unified [`agent::Agent`] trait.

use std::pin::Pin;

use agent::AgentError;
use agent::AgentInput;
use agent::AgentOutput;
use agent::AgentOutputStream;
use agent::AssistantContent;
use agent::AssistantMessage;
use agent::LocalTool;
use agent::Message;
use agent::OpenAiModel;
use codex_codes::protocol::UserInput;
use futures_core::Stream;

/// Codex coding agent adapting [`codex_codes::protocol`] constructs to the
/// [`agent::Agent`] trait.
#[derive(Clone)]
pub struct CodexAgent {
    /// Active OpenAI-compatible model configuration.
    model: OpenAiModel,
    /// Host tools registered with this agent.
    tools: Vec<LocalTool>,
    /// Optional thread or conversation identifier.
    thread_id: Option<String>,
}

impl CodexAgent {
    /// Construct a new [`CodexAgent`] with the given model configuration.
    pub fn new(model: OpenAiModel) -> Self {
        Self {
            model,
            tools: Vec::new(),
            thread_id: None,
        }
    }

    /// Set an optional thread identifier for session persistence.
    pub fn with_thread_id(mut self, thread_id: impl Into<String>) -> Self {
        self.thread_id = Some(thread_id.into());
        self
    }

    /// Retrieve the current thread identifier, if configured.
    pub fn thread_id(&self) -> Option<&str> {
        self.thread_id.as_deref()
    }

    /// Retrieve configured model reference.
    pub fn model(&self) -> &OpenAiModel {
        &self.model
    }

    /// Retrieve registered tools slice.
    pub fn tools(&self) -> &[LocalTool] {
        &self.tools
    }

    /// Convert unified [`AgentInput`] into Codex [`UserInput`].
    pub fn format_user_input(input: &AgentInput) -> UserInput {
        let text = match &input.message {
            Message::User {
                content,
            } => content
                .iter()
                .filter_map(|c| match c {
                    agent::UserContent::Text(t) => Some(t.text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Message::Assistant(assistant_msg) => assistant_msg
                .content
                .iter()
                .filter_map(|c| match c {
                    AssistantContent::Text(t) => Some(t.text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Message::System {
                content,
            } => content.clone(),
        };

        UserInput::Text {
            text,
            text_elements: None,
        }
    }
}

#[async_trait::async_trait]
impl agent::Agent for CodexAgent {
    async fn send(&self, input: AgentInput) -> Result<AgentOutputStream<'_>, AgentError> {
        let user_input = Self::format_user_input(&input);
        let prompt_text = match user_input {
            UserInput::Text {
                text,
                ..
            } => text,
            _ => String::new(),
        };

        let response_text = format!("Codex processed: {prompt_text}");
        let done_message = AssistantMessage::new(vec![AssistantContent::text(response_text.clone())]);

        let outputs: Vec<Result<AgentOutput, AgentError>> =
            vec![Ok(AgentOutput::Text(response_text)), Ok(AgentOutput::Done(done_message))];

        let stream: Pin<Box<dyn Stream<Item = Result<AgentOutput, AgentError>> + Send>> =
            Box::pin(futures_util::stream::iter(outputs));

        Ok(stream)
    }

    async fn set_model(&mut self, model: OpenAiModel) -> Result<(), AgentError> {
        self.model = model;
        Ok(())
    }

    async fn register_tool(&mut self, tool: LocalTool) -> Result<(), AgentError> {
        self.tools.push(tool);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use agent::Agent;
    use agent::AgentInput;
    use agent::AgentOutput;
    use agent::LocalTool;
    use agent::OpenAiModel;
    use futures_util::StreamExt;

    use super::CodexAgent;

    #[tokio::test]
    async fn test_new_and_with_thread_id() {
        let model = OpenAiModel::new("gpt-4o");
        let agent = CodexAgent::new(model.clone()).with_thread_id("thread-123");

        assert_eq!(agent.thread_id(), Some("thread-123"));
        assert_eq!(agent.model().model, "gpt-4o");
        assert!(agent.tools().is_empty());
    }

    #[tokio::test]
    async fn test_set_model() {
        let mut agent = CodexAgent::new(OpenAiModel::new("gpt-4o"));
        let new_model = OpenAiModel::new("claude-3-5-sonnet");

        agent.set_model(new_model.clone()).await.unwrap();
        assert_eq!(agent.model().model, "claude-3-5-sonnet");
    }

    #[tokio::test]
    async fn test_register_tool() {
        let mut agent = CodexAgent::new(OpenAiModel::new("gpt-4o"));
        let tool = LocalTool::new(
            "run_code",
            "Executes arbitrary code in a sandbox",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "code": { "type": "string" }
                },
                "required": ["code"]
            }),
            |args| async move { Ok(serde_json::json!({ "result": "ok", "echo": args })) },
        );

        agent.register_tool(tool).await.unwrap();
        assert_eq!(agent.tools().len(), 1);
        assert_eq!(agent.tools()[0].name, "run_code");
    }

    #[tokio::test]
    async fn test_send_and_streaming() {
        let agent = CodexAgent::new(OpenAiModel::new("gpt-4o"));
        let input = AgentInput::text("Write a hello world function");

        let mut stream = agent.send(input).await.unwrap();

        let mut outputs = Vec::new();
        while let Some(item) = stream.next().await {
            outputs.push(item.unwrap());
        }

        assert_eq!(outputs.len(), 2);
        match &outputs[0] {
            AgentOutput::Text(t) => {
                assert!(t.contains("Codex processed: Write a hello world function"));
            },
            other => panic!("expected AgentOutput::Text, got {other:?}"),
        }

        match &outputs[1] {
            AgentOutput::Done(msg) => {
                assert!(!msg.content.is_empty());
            },
            other => panic!("expected AgentOutput::Done, got {other:?}"),
        }
    }
}
