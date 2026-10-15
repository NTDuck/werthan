//! omp RPC agent implementation.

use std::sync::Arc;
use std::time::Duration;

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
use agent::UserContent;
use futures_util::stream;
use omp_rpc::Client;
use omp_rpc::HostTool;
use omp_rpc::HostToolContext;
use omp_rpc::HostToolOutput;
use omp_rpc::PromptCommand;
use omp_rpc::SetModelCommand;

/// omp RPC agent implementing the unified `agent::Agent` trait.
pub struct OmpAgent {
    model: OpenAiModel,
    tools: Vec<LocalTool>,
    client: Option<Arc<Client>>,
}

impl OmpAgent {
    /// Create a new `OmpAgent` configured with the specified model.
    pub fn new(model: OpenAiModel) -> Self {
        Self {
            model,
            tools: Vec::new(),
            client: None,
        }
    }

    /// Attach an `omp_rpc::Client` to this agent.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = Some(Arc::new(client));
        self
    }

    /// Access currently registered tools.
    pub fn tools(&self) -> &[LocalTool] {
        &self.tools
    }

    /// Access current model configuration.
    pub fn model(&self) -> &OpenAiModel {
        &self.model
    }

    /// Access the client if attached.
    pub fn client(&self) -> Option<&Arc<Client>> {
        self.client.as_ref()
    }
}

fn convert_tool(tool: &LocalTool) -> HostTool {
    let params_map = match &tool.parameters {
        serde_json::Value::Object(map) => map.clone(),
        _ => serde_json::Map::new(),
    };
    let handler = Arc::clone(&tool.handler);

    HostTool::new(tool.name.clone(), tool.description.clone(), params_map, move |args, _ctx: &HostToolContext| {
        let args_val = serde_json::Value::Object(args);
        let handler = Arc::clone(&handler);
        let res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { handler(args_val).await })
        });
        match res {
            Ok(val) => {
                let text = match val {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                };
                Ok(HostToolOutput::Text(text))
            },
            Err(err) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))),
        }
    })
}

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
impl Agent for OmpAgent {
    async fn set_model(&mut self, model: OpenAiModel) -> Result<(), AgentError> {
        self.model = model;
        if let Some(client) = &self.client {
            let cmd = SetModelCommand {
                provider: "openai".to_string(),
                model_id: self.model.model.clone(),
            };
            client.call(&cmd).map_err(|e| AgentError::Rpc(format!("Failed to set model: {e}")))?;
        }
        Ok(())
    }

    async fn register_tool(&mut self, tool: LocalTool) -> Result<(), AgentError> {
        self.tools.push(tool);
        if let Some(client) = &self.client {
            let host_tools: Vec<HostTool> = self.tools.iter().map(convert_tool).collect();
            client
                .set_custom_tools(host_tools)
                .map_err(|e| AgentError::Tool(format!("Failed to register custom tools: {e}")))?;
        }
        Ok(())
    }

    async fn send(&self, input: AgentInput) -> Result<AgentOutputStream<'_>, AgentError> {
        let prompt_str = extract_prompt(&input.message);

        if let Some(client) = &self.client {
            let client = Arc::clone(client);
            let cmd = PromptCommand {
                message: prompt_str.clone(),
                images: None,
                streaming_behavior: None,
            };

            let turn = tokio::task::spawn_blocking(move || client.prompt_and_wait(&cmd, Duration::from_secs(60)))
                .await
                .map_err(|e| AgentError::Rpc(format!("Prompt task panicked or cancelled: {e}")))?
                .map_err(|e| AgentError::Rpc(format!("Prompt failed: {e}")))?;

            let text_response = turn.assistant_text.unwrap_or_else(|| format!("Echo: {prompt_str}"));
            let outputs = vec![
                Ok(AgentOutput::Text(text_response.clone())),
                Ok(AgentOutput::Done(AssistantMessage::new(vec![AssistantContent::text(text_response)]))),
            ];
            Ok(Box::pin(stream::iter(outputs)))
        } else {
            let response_text = format!("Echo: {prompt_str}");
            let outputs = vec![
                Ok(AgentOutput::Text(response_text.clone())),
                Ok(AgentOutput::Done(AssistantMessage::new(vec![AssistantContent::text(response_text)]))),
            ];
            Ok(Box::pin(stream::iter(outputs)))
        }
    }
}

#[cfg(test)]
mod tests {
    use futures_util::StreamExt;
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_omp_agent_creation() {
        let model = OpenAiModel::new("gpt-4o");
        let agent = OmpAgent::new(model);
        assert_eq!(agent.model().model, "gpt-4o");
        assert!(agent.tools().is_empty());
        assert!(agent.client().is_none());
    }

    #[tokio::test]
    async fn test_omp_agent_set_model() {
        let model = OpenAiModel::new("gpt-4o");
        let mut agent = OmpAgent::new(model);

        let new_model = OpenAiModel::new("claude-3-5-sonnet");
        agent.set_model(new_model.clone()).await.unwrap();
        assert_eq!(agent.model().model, "claude-3-5-sonnet");
    }

    #[tokio::test]
    async fn test_omp_agent_register_tool() {
        let model = OpenAiModel::new("gpt-4o");
        let mut agent = OmpAgent::new(model);

        let tool = LocalTool::new(
            "test_tool",
            "A test tool",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            }),
            |args| async move { Ok(json!({ "result": format!("got {:?}", args) })) },
        );

        agent.register_tool(tool).await.unwrap();
        assert_eq!(agent.tools().len(), 1);
        assert_eq!(agent.tools()[0].name, "test_tool");
    }

    #[tokio::test]
    async fn test_omp_agent_stream_output_offline() {
        let model = OpenAiModel::new("gpt-4o");
        let agent = OmpAgent::new(model);

        let mut stream = agent.prompt("Hello world").await.unwrap();

        let first = stream.next().await.unwrap().unwrap();
        match first {
            AgentOutput::Text(t) => assert_eq!(t, "Echo: Hello world"),
            other => panic!("Expected AgentOutput::Text, got {:?}", other),
        }

        let second = stream.next().await.unwrap().unwrap();
        match second {
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
}
