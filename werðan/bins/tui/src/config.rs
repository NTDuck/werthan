//! Configuration for the underlying inference model backend.

/// Configuration holding endpoints, authentication, and token constraints.
#[derive(::core::fmt::Debug, ::core::clone::Clone)]
#[allow(dead_code)]
pub struct ModelConfig {
    /// Base URL for the OpenAI-compatible inference endpoint.
    pub base_url: ::std::string::String,
    /// API format dialect.
    pub api: ::std::string::String,
    /// Authentication token or none.
    pub auth: ::std::string::String,
    /// Target model identifier.
    pub model_id: ::std::string::String,
    /// Human-readable model display name.
    pub model_name: ::std::string::String,
    /// Flag that enables reasoning mode.
    pub reasoning: ::core::primitive::bool,
    /// Maximum context window size in tokens.
    pub context_window: ::core::primitive::usize,
    /// Maximum generated tokens per completion turn.
    pub max_tokens: ::core::primitive::u64,
}

impl ::core::default::Default for self::ModelConfig {
    fn default() -> Self {
        Self {
            base_url: "http://lenovo-legion-16iah7h-pf3xj8sp:8092/v1".to_string(),
            api: "openai-completions".to_string(),
            auth: "none".to_string(),
            model_id: "qwen3.8-27b".to_string(),
            model_name: "Qwen 3.8 27B".to_string(),
            reasoning: true,
            context_window: 262144,
            max_tokens: 16384,
        }
    }
}

#[allow(dead_code)]
impl self::ModelConfig {
    /// Create default Ashalliants model configuration.
    pub fn ashalliants() -> Self {
        Self::default()
    }

    /// Instantiate a Rig erased dynamic completion model from this
    /// configuration.
    pub fn build_completion_model(&self) -> ::rig::driver::DynModel<::rig::operation::Completion> {
        let rig_config = self.build_rig_config();
        let client = rig_config.client();
        let model = client.completion(&self.model_id);
        model.erase()
    }

    /// Build a Rig OpenAI client configuration from this model specification.
    pub fn build_rig_config(&self) -> ::rig::providers::openai::OpenAIConfig {
        let key = if self.auth == "none" {
            "none"
        } else {
            self.auth.as_str()
        };
        ::rig::providers::openai::OpenAIConfig::new(key)
            .with_base_url(&self.base_url)
            .with_route(::rig::providers::openai::Route::Chat)
    }
}
