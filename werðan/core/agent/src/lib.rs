//! Minimal core agent interface reusing Rig messaging and streaming
//! infrastructure.
//!
//! Provides the primary [`Agent`] trait along with unified representations for:
//! - [`AgentInput`]: Asynchronous input handling, reusing
//!   [`rig::completion::Message`].
//! - [`AgentOutput`]: Streamed chunk/event outputs, integrating
//!   [`rig::streaming::StreamEvent`].
//! - [`OpenAiModel`]: OpenAI-compatible provider/model configuration.
//! - [`LocalTool`]: Host-registered executable tools with JSON Schema
//!   parameters.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use futures_core::Stream;
pub use rig::completion::message::AssistantContent;
pub use rig::completion::message::AssistantMessage;
pub use rig::completion::message::ToolCall;
pub use rig::completion::message::UserContent;
pub use rig::completion::Message;
pub use rig::streaming::StreamEvent;

/// Pinned asynchronous stream of agent output events or errors.
pub type AgentOutputStream<'a> = Pin<Box<dyn Stream<Item = Result<AgentOutput, AgentError>> + Send + 'a>>;

/// Boxed asynchronous future for tool execution.
pub type ToolBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Asynchronous tool execution handler function.
pub type ToolHandler =
    Arc<dyn Fn(serde_json::Value) -> ToolBoxFuture<'static, Result<serde_json::Value, AgentError>> + Send + Sync>;

/// Primary agent interface for interactive and automated LLM interactions.
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    /// Send input asynchronously and receive an output stream of events.
    async fn send(&self, input: AgentInput) -> Result<AgentOutputStream<'_>, AgentError>;

    /// Set or change the active OpenAI-compatible model configuration.
    async fn set_model(&mut self, model: OpenAiModel) -> Result<(), AgentError>;

    /// Register a local host-side tool callable by the agent.
    async fn register_tool(&mut self, tool: LocalTool) -> Result<(), AgentError>;

    /// Convenience shortcut to send a prompt string directly.
    async fn prompt(&self, text: impl Into<String> + Send) -> Result<AgentOutputStream<'_>, AgentError> {
        self.send(AgentInput::text(text)).await
    }
}

/// Unified input sent to an agent.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentInput {
    /// Canonical chat message payload.
    pub message: Message,
}

impl AgentInput {
    /// Construct a user prompt input from string content.
    pub fn text(prompt: impl Into<String>) -> Self {
        Self {
            message: Message::user(prompt),
        }
    }

    /// Construct input from an existing Rig [`Message`].
    pub fn message(message: Message) -> Self {
        Self {
            message,
        }
    }
}

impl From<&str> for AgentInput {
    fn from(s: &str) -> Self {
        Self::text(s)
    }
}

impl From<String> for AgentInput {
    fn from(s: String) -> Self {
        Self::text(s)
    }
}

impl From<Message> for AgentInput {
    fn from(message: Message) -> Self {
        Self::message(message)
    }
}

/// Streamed output chunk or event emitted during agent response generation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AgentOutput {
    /// Incremental text fragment produced by the model.
    Text(String),
    /// Incremental reasoning or thinking fragment.
    Reasoning(String),
    /// Request by the agent to execute a tool.
    ToolCall {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    /// Result payload from an executed tool.
    ToolResult {
        id: String,
        name: String,
        content: serde_json::Value,
    },
    /// Native structured streaming event from Rig.
    Event(StreamEvent),
    /// Finalized complete assistant message turn.
    Done(AssistantMessage),
}

impl AgentOutput {
    /// Return the text fragment if this output variant is
    /// [`AgentOutput::Text`].
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text.as_str()),
            _ => None,
        }
    }

    /// Check if this event marks turn completion.
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done(_))
    }
}

impl From<String> for AgentOutput {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for AgentOutput {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

impl From<StreamEvent> for AgentOutput {
    fn from(event: StreamEvent) -> Self {
        Self::Event(event)
    }
}

/// OpenAI-compatible model configuration.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OpenAiModel {
    /// Model identifier (e.g., `gpt-4o`, `deepseek-chat`, `claude-3-7-sonnet`).
    pub model: String,
    /// Base endpoint URL for custom/compatible providers (e.g., Ollama, vLLM,
    /// OpenRouter).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Authentication token or API key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Optional sampling temperature parameter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
}

impl OpenAiModel {
    /// Create a new model configuration with defaults.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            base_url: None,
            api_key: None,
            temperature: None,
        }
    }

    /// Set custom API base URL for OpenAI-compatible endpoints.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set authentication token or API key.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set sampling temperature.
    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }
}

impl From<&OpenAiModel> for rig::providers::openai::OpenAIConfig {
    fn from(model: &OpenAiModel) -> Self {
        let key = model.api_key.clone().unwrap_or_else(|| "none".to_string());
        let mut config = Self::new(key);
        if let Some(base_url) = &model.base_url {
            config = config.with_base_url(base_url.clone());
        }
        config
    }
}

/// Host-registered tool callable by an agent.
#[derive(Clone)]
pub struct LocalTool {
    /// Unique identifier and name of the tool.
    pub name: String,
    /// Human- and model-readable description of functionality.
    pub description: String,
    /// JSON Schema describing expected parameters.
    pub parameters: serde_json::Value,
    /// Asynchronous execution callback.
    pub handler: ToolHandler,
}

impl LocalTool {
    /// Create a new local tool from an asynchronous callback.
    pub fn new<F, Fut>(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
        handler: F,
    ) -> Self
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<serde_json::Value, AgentError>> + Send + 'static,
    {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
            handler: Arc::new(move |args| Box::pin(handler(args))),
        }
    }

    /// Execute the tool directly with given arguments.
    pub async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value, AgentError> {
        (self.handler)(args).await
    }
}

impl TryFrom<LocalTool> for rig::tool::DynamicTool {
    type Error = AgentError;

    fn try_from(tool: LocalTool) -> Result<Self, Self::Error> {
        let tool_name =
            rig::message::ToolName::new(&tool.name).map_err(|e| AgentError::Tool(format!("Invalid tool name: {e}")))?;
        let handler = tool.handler.clone();
        Ok(Self::new_with_context(tool_name, tool.description, tool.parameters, move |_ctx, args| {
            let handler = handler.clone();
            Box::pin(async move {
                let res = handler(args).await.map_err(|e| rig::tool::ToolExecutionError::other(e.to_string()))?;
                Ok(rig::tool::ToolOutput::json(res))
            })
        }))
    }
}

/// Error kinds encountered during agent execution or configuration.
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    /// Failure communicated from provider or model endpoint.
    #[error("Provider error: {0}")]
    Provider(String),

    /// Failure executing or validating a tool.
    #[error("Tool error: {0}")]
    Tool(String),

    /// Failure in RPC communication or transport.
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Missing or invalid configuration.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Standard I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use futures_util::stream;

    use super::*;

    struct MockAgent {
        model: OpenAiModel,
        tools: Vec<LocalTool>,
    }

    impl MockAgent {
        fn new() -> Self {
            Self {
                model: OpenAiModel::new("test-model"),
                tools: Vec::new(),
            }
        }
    }

    #[async_trait::async_trait]
    impl Agent for MockAgent {
        async fn send(&self, input: AgentInput) -> Result<AgentOutputStream<'_>, AgentError> {
            let prompt = match &input.message {
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
                _ => String::new(),
            };

            let out_items = vec![
                Ok(AgentOutput::Text(format!("Echo: {prompt}"))),
                Ok(AgentOutput::Done(AssistantMessage::new(vec![AssistantContent::text(format!("Echo: {prompt}"))]))),
            ];

            Ok(Box::pin(stream::iter(out_items)))
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

    #[tokio::test]
    async fn test_agent_input_creation() {
        let input_str = AgentInput::from("hello world");
        assert!(matches!(input_str.message, Message::User { .. }));

        let input_msg = AgentInput::message(Message::system("system instruction"));
        assert!(matches!(input_msg.message, Message::System { .. }));
    }

    #[tokio::test]
    async fn test_openai_model_config() {
        let model = OpenAiModel::new("gpt-4o")
            .with_base_url("https://custom.api/v1")
            .with_api_key("sk-test")
            .with_temperature(0.7);

        assert_eq!(model.model, "gpt-4o");
        assert_eq!(model.base_url.as_deref(), Some("https://custom.api/v1"));
        assert_eq!(model.api_key.as_deref(), Some("sk-test"));
        assert_eq!(model.temperature, Some(0.7));

        let config: rig::providers::openai::OpenAIConfig = (&model).into();
        assert_eq!(config.base_url, "https://custom.api/v1");
    }

    #[tokio::test]
    async fn test_local_tool_execution_and_conversion() {
        let tool = LocalTool::new(
            "calculator",
            "adds two numbers",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "a": { "type": "number" },
                    "b": { "type": "number" }
                }
            }),
            |args| async move {
                let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                Ok(serde_json::json!({ "result": a + b }))
            },
        );

        let res =
            tool.execute(serde_json::json!({ "a": 10.0, "b": 25.0 })).await.expect("tool execution should succeed");
        assert_eq!(res.get("result").and_then(|v| v.as_f64()), Some(35.0));

        let dynamic_tool: Result<rig::tool::DynamicTool, _> = tool.try_into();
        assert!(dynamic_tool.is_ok());
    }

    #[tokio::test]
    async fn test_mock_agent_send_and_prompt() {
        use futures_util::StreamExt;

        let mut agent = MockAgent::new();
        agent.set_model(OpenAiModel::new("gpt-4o-mini")).await.expect("set model should succeed");

        let mut stream = agent.prompt("testing 1 2 3").await.expect("prompt should succeed");
        let first = stream.next().await.expect("first chunk").expect("ok");
        assert_eq!(first.as_text(), Some("Echo: testing 1 2 3"));

        let second = stream.next().await.expect("second chunk").expect("ok");
        assert!(second.is_done());
    }
}
