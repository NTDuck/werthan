//! Rig bridge implementation.

/// Completion model configuration for Rig.
#[derive(::bon::Builder, ::core::clone::Clone, ::core::fmt::Debug)]
pub struct CompletionModel {
    /// Target model identifier.
    pub model: ::std::string::String,
    /// Authentication token or API key.
    pub api_key: ::core::option::Option<::std::string::String>,
    /// Endpoint base URL for custom providers.
    pub base_url: ::core::option::Option<::std::string::String>,
    /// Optional sampling temperature parameter.
    pub temperature: ::core::option::Option<::core::primitive::f64>,
}

impl self::CompletionModel {
    /// Return the target model identifier.
    pub fn model(&self) -> &::core::primitive::str {
        &self.model
    }

    /// Return the API key when configured.
    pub fn api_key(&self) -> ::core::option::Option<&::core::primitive::str> {
        self.api_key.as_deref()
    }

    /// Return the base URL when configured.
    pub fn base_url(&self) -> ::core::option::Option<&::core::primitive::str> {
        self.base_url.as_deref()
    }

    /// Return the sampling temperature when configured.
    pub fn temperature(&self) -> ::core::option::Option<::core::primitive::f64> {
        self.temperature
    }

    /// Complete a prompt string using this model.
    pub async fn call(
        &self,
        prompt: &::core::primitive::str,
    ) -> ::core::result::Result<::std::string::String, ::rig::error::ProviderError> {
        let request = ::rig::completion::CompletionRequest::new(prompt);
        let response = self.completion(request).await?;
        let text = extract_text(&response);
        ::core::result::Result::Ok(text)
    }

    /// Generate a completion response for the specified request.
    pub async fn completion(
        &self,
        request: ::rig::completion::CompletionRequest,
    ) -> ::core::result::Result<::rig::completion::CompletionResponse, ::rig::error::ProviderError> {
        if self.is_offline() {
            let choice = ::std::vec![::rig::message::AssistantContent::text("Echo response")];
            let usage = ::rig::completion::Usage::default();
            let api = ::rig::message::Api::from_static("openai.chat");
            let origin = ::rig::message::Origin::new(api, "openai", &self.model);
            let raw = ::serde_json::Value::Null;
            let response = ::rig::completion::CompletionResponse::new(choice, usage, origin, raw);
            return ::core::result::Result::Ok(response);
        }
        let driver = self.into_driver();
        driver.call(request).await
    }

    /// Stream a completion response for the specified request.
    #[allow(clippy::result_large_err)]
    pub fn stream(
        &self,
        request: ::rig::completion::CompletionRequest,
    ) -> ::core::result::Result<::rig::streaming::Streamed<::rig::operation::Completion>, ::rig::error::ProviderError>
    {
        let driver = self.into_driver();
        driver.stream(request)
    }

    fn is_offline(&self) -> ::core::primitive::bool {
        ::core::matches!(
            self.api_key.as_deref(),
            ::core::option::Option::Some("none") | ::core::option::Option::Some("mock") | ::core::option::Option::None
        )
    }

    /// Convert this model configuration into a native Rig driver model.
    pub fn into_driver(&self) -> ::rig::driver::Model<::rig::providers::openai::wire::OpenAiWire> {
        let fallback = "none".to_string();
        let key = self.api_key.clone().unwrap_or(fallback);
        let mut config = ::rig::providers::openai::OpenAIConfig::new(key);
        if let ::core::option::Option::Some(url) = &self.base_url {
            config = config.with_base_url(url.clone());
        }
        let client = config.client();
        client.completion(&self.model)
    }
}

fn extract_text(response: &::rig::completion::CompletionResponse) -> ::std::string::String {
    response
        .choice
        .iter()
        .filter_map(|c| match c {
            ::rig::message::AssistantContent::Text(t) => ::core::option::Option::Some(t.text.clone()),
            _ => ::core::option::Option::None,
        })
        .collect::<::std::vec::Vec<::std::string::String>>()
        .join(" ")
}

impl ::core::convert::From<&self::CompletionModel> for ::rig::driver::DynModel<::rig::operation::Completion> {
    fn from(model: &self::CompletionModel) -> Self {
        model.into_driver().erase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[::tokio::test]
    async fn builder_creates_model() {
        let model = self::CompletionModel::builder().model("gpt-4o".to_string()).build();
        ::std::assert_eq!(model.model(), "gpt-4o");
        ::std::assert_eq!(model.api_key(), ::core::option::Option::None);
    }

    #[::tokio::test]
    async fn offline_call_echos_prompt() {
        let model = self::CompletionModel::builder().model("gpt-4o".to_string()).build();
        let res = model.call("hello").await.unwrap();
        ::std::assert_eq!(res, "Echo response");
    }

    #[::tokio::test]
    async fn completion_returns_response() {
        let model = self::CompletionModel::builder().model("gpt-4o".to_string()).build();
        let req = ::rig::completion::CompletionRequest::new("hello");
        let res = model.completion(req).await.unwrap();
        ::std::assert_eq!(res.choice.len(), 1);
        ::std::assert_eq!(res.provider(), "openai");
    }

    #[::tokio::test]
    async fn dyn_model_conversion() {
        let model = self::CompletionModel::builder().model("gpt-4o".to_string()).build();
        let dyn_model = ::rig::driver::DynModel::from(&model);
        ::std::assert_eq!(dyn_model.name(), "openai");
    }
}
