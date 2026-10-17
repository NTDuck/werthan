//! Configuration for the underlying inference model backend.

/// Configuration holding endpoints, authentication, and model identifier.
#[derive(::bon::Builder, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct ModelConfig {
    /// Base URL for the OpenAI-compatible inference endpoint loaded at compile time from .env.
    #[builder(into, default = ::core::env!("BASE_URL").to_string())]
    pub base_url: ::std::string::String,

    /// Authentication token or none loaded at compile time from .env.
    #[builder(into, default = ::core::env!("AUTH").to_string())]
    pub auth: ::std::string::String,

    /// Target model identifier loaded at compile time from .env.
    #[builder(into, default = ::core::env!("MODEL_ID").to_string())]
    pub model_id: ::std::string::String,

    /// Human-readable model display name loaded at compile time from .env.
    #[builder(into, default = ::core::env!("MODEL_NAME").to_string())]
    pub model_name: ::std::string::String,
}

impl ::core::default::Default for self::ModelConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl self::ModelConfig {
    /// Instantiate a Rig erased dynamic completion model from this configuration.
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
