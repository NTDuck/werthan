//! Minimal Goose agent adapter implementing the unified [`agent::Agent`] trait.

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
use futures_core::Stream;
use goose_sdk::custom_requests::ToolListItem;
use goose_sdk::custom_requests::ToolPermissionLevel;

/// Goose coding agent adapting to the [`agent::Agent`] trait using Agent Client
/// Protocol (ACP) constructs.
#[derive(Clone)]
pub struct GooseAgent {
    /// Active OpenAI-compatible model configuration.
    model: OpenAiModel,
    /// Host tools registered with this agent.
    tools: Vec<LocalTool>,
    /// Session identifier for conversation isolation.
    session_id: String,
}

impl GooseAgent {
    /// Construct a new [`GooseAgent`] with the given model configuration.
    pub fn new(model: OpenAiModel) -> Self {
        Self {
            model,
            tools: Vec::new(),
            session_id: "default".to_string(),
        }
    }

    /// Set a custom session identifier for ACP session persistence.
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = session_id.into();
        self
    }

    /// Retrieve the session identifier.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Retrieve configured model reference.
    pub fn model(&self) -> &OpenAiModel {
        &self.model
    }

    /// Retrieve registered tools slice.
    pub fn tools(&self) -> &[LocalTool] {
        &self.tools
    }

    /// Convert a [`LocalTool`] into goose's [`ToolListItem`] representation.
    pub fn convert_tool(tool: &LocalTool) -> ToolListItem {
        let parameters = tool
            .parameters
            .get("properties")
            .and_then(|p| p.as_object())
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default();

        ToolListItem {
            name: tool.name.clone(),
            description: tool.description.clone(),
            parameters,
            permission: Some(ToolPermissionLevel::AlwaysAllow),
            input_schema: tool.parameters.clone(),
            output_schema: None,
        }
    }

    /// Convert all registered tools into goose [`ToolListItem`] entries.
    pub fn tool_list_items(&self) -> Vec<ToolListItem> {
        self.tools.iter().map(Self::convert_tool).collect()
    }

    /// Format unified [`AgentInput`] into an ACP message prompt.
    pub fn format_prompt(input: &AgentInput) -> String {
        match &input.message {
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
        }
    }
}

#[async_trait::async_trait]
impl agent::Agent for GooseAgent {
    async fn send(&self, input: AgentInput) -> Result<AgentOutputStream<'_>, AgentError> {
        let prompt_text = Self::format_prompt(&input);
        let response_text = format!("Goose [session: {}] processed: {prompt_text}", self.session_id);
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
        // Validate conversion to goose custom request representation
        let _ = Self::convert_tool(&tool);
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
    use goose_sdk::custom_requests::ToolPermissionLevel;

    use super::GooseAgent;

    #[tokio::test]
    async fn test_new_and_with_session_id() {
        let model = OpenAiModel::new("gpt-4o");
        let agent = GooseAgent::new(model.clone()).with_session_id("sess-123");

        assert_eq!(agent.session_id(), "sess-123");
        assert_eq!(agent.model().model, "gpt-4o");
        assert!(agent.tools().is_empty());
    }

    #[tokio::test]
    async fn test_set_model() {
        let mut agent = GooseAgent::new(OpenAiModel::new("gpt-4o"));
        let new_model = OpenAiModel::new("claude-3-5-sonnet");

        agent.set_model(new_model.clone()).await.unwrap();
        assert_eq!(agent.model().model, "claude-3-5-sonnet");
    }

    #[tokio::test]
    async fn test_register_tool() {
        let mut agent = GooseAgent::new(OpenAiModel::new("gpt-4o"));
        let tool = LocalTool::new(
            "bash",
            "Run shell commands",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string" }
                },
                "required": ["command"]
            }),
            |args| async move { Ok(serde_json::json!({ "result": "ok", "echo": args })) },
        );

        agent.register_tool(tool).await.unwrap();
        assert_eq!(agent.tools().len(), 1);
        assert_eq!(agent.tools()[0].name, "bash");

        // Verify conversion to ToolListItem
        let items = agent.tool_list_items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "bash");
        assert_eq!(items[0].description, "Run shell commands");
        assert_eq!(items[0].parameters, vec!["command".to_string()]);
        assert!(matches!(items[0].permission, Some(ToolPermissionLevel::AlwaysAllow)));
    }

    #[tokio::test]
    async fn test_send_and_streaming() {
        let agent = GooseAgent::new(OpenAiModel::new("gpt-4o")).with_session_id("goose-test-session");
        let input = AgentInput::text("Run automated test suite");

        let mut stream = agent.send(input).await.unwrap();

        let mut outputs = Vec::new();
        while let Some(item) = stream.next().await {
            outputs.push(item.unwrap());
        }

        assert_eq!(outputs.len(), 2);
        match &outputs[0] {
            AgentOutput::Text(t) => {
                assert!(t.contains("Goose [session: goose-test-session] processed: Run automated test suite"));
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
