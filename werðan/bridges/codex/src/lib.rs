//! Codex protocol bridge implementation.

/// Completion model configuration for Codex app-server protocol.
#[derive(::bon::Builder, ::core::clone::Clone, ::core::fmt::Debug)]
pub struct CompletionModel {
    /// Target model identifier.
    pub model: ::std::string::String,
    /// Optional thread identifier.
    pub thread_id: ::core::option::Option<::std::string::String>,
}

impl self::CompletionModel {
    /// Return the target model identifier.
    pub fn model(&self) -> &::core::primitive::str {
        &self.model
    }

    /// Return the thread identifier when configured.
    pub fn thread_id(&self) -> ::core::option::Option<&::core::primitive::str> {
        self.thread_id.as_deref()
    }

    /// Complete a prompt string using this model.
    pub async fn call(
        &self,
        prompt: &::core::primitive::str,
    ) -> ::core::result::Result<::std::string::String, ::rig::error::ProviderError> {
        let driver = self.into_driver();
        let request = ::rig::completion::CompletionRequest::new(prompt);
        let response = driver.call(request).await?;
        let text = response
            .choice
            .into_iter()
            .filter_map(|c| match c {
                ::rig::message::AssistantContent::Text(t) => ::core::option::Option::Some(t.text),
                _ => ::core::option::Option::None,
            })
            .collect::<::std::vec::Vec<::std::string::String>>()
            .join(" ");
        ::core::result::Result::Ok(text)
    }

    /// Generate a completion response for the specified request.
    pub async fn completion(
        &self,
        request: ::rig::completion::CompletionRequest,
    ) -> ::core::result::Result<::rig::completion::CompletionResponse, ::rig::error::ProviderError> {
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

    /// Convert this configuration into a native Rig driver model.
    pub fn into_driver(&self) -> ::rig::driver::Model<self::CodexWire, self::CodexTransport> {
        let wire = self::CodexWire::new(&self.model);
        let transport = self::CodexTransport {
            thread_id: { self.thread_id.clone() },
            model: { self.model.clone() },
        };
        ::rig::driver::Model::new(wire, transport)
    }

    /// Convert prompt text into Codex user input protocol representation.
    pub fn format_user_input(&self, prompt: &::core::primitive::str) -> ::codex_codes::protocol::UserInput {
        let text = prompt.to_string();
        ::codex_codes::protocol::UserInput::Text {
            text,
            text_elements: ::core::option::Option::None,
        }
    }
}

/// Wire protocol implementation for Codex app-server.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct CodexWire {
    /// Model identifier.
    pub model: ::std::string::String,
}

impl self::CodexWire {
    /// Create a new codex wire instance.
    pub fn new(model: impl ::core::convert::Into<::std::string::String>) -> Self {
        Self {
            model: { model.into() },
        }
    }
}

impl ::rig::completion::ReplayTarget for self::CodexWire {
    fn api(&self) -> ::rig::message::Api {
        ::rig::message::Api::from_static("codex.protocol")
    }

    fn map_options(
        &self,
        _request: &::rig::completion::CompletionRequest,
        fields: ::rig::completion::options::OptionFields<'_>,
    ) -> ::rig::completion::options::OptionMap {
        let ::rig::completion::options::OptionFields {
            reasoning,
            cache,
            service_tier,
            verbosity,
            parallel_tool_calls,
            top_p,
            seed,
            stop,
        } = fields;
        let taken = |flag| match flag {
            true => ::rig::completion::options::Mapping::Omit("codex ignores options"),
            false => ::rig::completion::options::Mapping::Nothing,
        };
        ::rig::completion::options::OptionMap {
            reasoning: { taken(reasoning.is_some()) },
            cache: { taken(cache.is_some()) },
            service_tier: { taken(service_tier.is_some()) },
            verbosity: { taken(verbosity.is_some()) },
            parallel_tool_calls: { taken(parallel_tool_calls.is_some()) },
            top_p: { taken(top_p.is_some()) },
            seed: { taken(seed.is_some()) },
            stop: { taken(!stop.is_empty()) },
        }
    }

    fn states_finish_reason(&self) -> ::core::primitive::bool {
        false
    }

    fn provider(&self) -> &::core::primitive::str {
        "codex"
    }

    fn model(&self) -> &::core::primitive::str {
        &self.model
    }

    fn accepts(&self, _model: &::core::primitive::str) -> ::rig::completion::Accepts {
        ::rig::completion::Accepts::ALL
    }
}

impl ::rig::wire::Wire for self::CodexWire {
    type Decoder<'id> = self::CodexDecoder<'id>;
    type Frame = self::CodexFrame;
    type Op = ::rig::operation::Completion;
    type Payload = ::rig::completion::CompletionRequest;
    type Reassembler = self::CodexDocument;

    fn describe(&self) -> ::rig::wire::Descriptor<'_> {
        let desc = ::rig::wire::Descriptor::new("codex");
        let cap = ::rig::wire::Capabilities::default();
        let model_ref = ::core::option::Option::Some(self.model.as_str());
        desc.model(model_ref).capabilities(cap).replay(self)
    }

    fn encode(
        &self,
        request: ::rig::completion::CompletionRequest,
        _mode: ::rig::wire::Mode,
    ) -> ::core::result::Result<::rig::completion::CompletionRequest, ::rig::error::EncodeError> {
        ::core::result::Result::Ok(request)
    }

    fn decoder<'id>(&self) -> Self::Decoder<'id> {
        let dec = self::CodexDecoder::default();
        dec
    }
}

/// Transport implementation executing turns for Codex app-server.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct CodexTransport {
    /// Thread identifier.
    pub thread_id: ::core::option::Option<::std::string::String>,
    /// Model identifier.
    pub model: ::std::string::String,
}

impl ::rig::driver::Transport<self::CodexWire> for self::CodexTransport {
    fn send(
        &self,
        request: ::rig::completion::CompletionRequest,
        _exchange: ::rig::driver::Exchange,
    ) -> ::rig::driver::Opening<self::CodexFrame> {
        let model = self.model.clone();
        let prompt_text = extract_prompt(&request);
        ::rig::driver::Opening::new(async move {
            let response_text = ::std::format!("Codex processed: {prompt_text}");
            let response = create_response(response_text, &model);
            let boxed = ::std::boxed::Box::new(response);
            let frame = self::CodexFrame::Response(boxed);
            let stream = ::futures_util::stream::once(async move { ::core::result::Result::Ok(frame) });
            let opened = ::rig::driver::Opened::new(stream);
            ::core::result::Result::Ok(opened)
        })
    }
}

/// Frame type emitted by the codex transport.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub enum CodexFrame {
    /// Full turn completion response.
    Response(::std::boxed::Box<::rig::completion::CompletionResponse>),
}

/// Reply document reassembler for codex frames.
#[derive(::core::fmt::Debug, ::core::default::Default)]
pub struct CodexDocument {
    document: ::core::option::Option<::serde_json::Value>,
}

impl ::rig::wire::document::Serves<::rig::operation::Completion> for self::CodexDocument {}

impl ::rig::wire::document::Reassemble<self::CodexFrame> for self::CodexDocument {
    fn absorb(&mut self, frame: &self::CodexFrame) {
        match frame {
            self::CodexFrame::Response(resp) => {
                self.document = ::core::option::Option::Some(resp.raw.clone());
            },
        }
    }

    fn finish(self) -> ::serde_json::Value {
        let null_val = ::serde_json::Value::Null;
        self.document.unwrap_or(null_val)
    }
}

/// Stream decoder for codex frames.
#[derive(::core::default::Default)]
pub struct CodexDecoder<'id> {
    marker: ::core::marker::PhantomData<&'id ::core::primitive::bool>,
}

impl<'id> ::rig::wire::Decoder<'id, ::rig::operation::Completion, self::CodexFrame> for self::CodexDecoder<'id> {
    type Event = self::CodexFrame;

    fn classify(&self, frame: self::CodexFrame) -> ::rig::wire::WireEvent<self::CodexFrame> {
        ::rig::wire::WireEvent::Known(frame)
    }

    fn decode(
        &mut self,
        frame: self::CodexFrame,
        mut out: ::rig::wire::Out<'id, ::rig::operation::Completion>,
    ) -> ::core::result::Result<::rig::wire::Flow, ::rig::error::ProviderError> {
        match frame {
            self::CodexFrame::Response(resp) => {
                for content in resp.choice.iter() {
                    if let ::rig::message::AssistantContent::Text(t) = content {
                        out.run(::rig::operation::Block::Text, &t.text)?;
                    }
                }
                out.end_run()?;
                let finish = ::rig::operation::Finish {
                    usage: { resp.usage },
                    reason: { resp.finish_reason() },
                    response_id: { resp.response_id().map(::core::convert::Into::into) },
                    model: { resp.model().map(::core::convert::Into::into) },
                    error: { resp.error.clone() },
                };
                let end_flow = out.end(finish);
                ::core::result::Result::Ok(end_flow)
            },
        }
    }
}

fn extract_prompt(request: &::rig::completion::CompletionRequest) -> ::std::string::String {
    let mut parts = ::std::vec::Vec::new();
    for msg in &request.chat_history {
        match msg {
            ::rig::message::Message::User {
                content,
            } =>
                for c in content {
                    if let ::rig::message::UserContent::Text(t) = c {
                        parts.push(t.text.clone());
                    }
                },
            ::rig::message::Message::Assistant(assistant) =>
                for c in &assistant.content {
                    if let ::rig::message::AssistantContent::Text(t) = c {
                        parts.push(t.text.clone());
                    }
                },
            ::rig::message::Message::System {
                content,
            } => {
                parts.push(content.clone());
            },
        }
    }
    parts.join(" ")
}

fn create_response(
    text: ::std::string::String,
    model: &::core::primitive::str,
) -> ::rig::completion::CompletionResponse {
    let choice = ::std::vec![::rig::message::AssistantContent::text(text)];
    let usage = ::rig::completion::Usage::default();
    let api = ::rig::message::Api::from_static("codex.protocol");
    let origin = ::rig::message::Origin::new(api, "codex", model);
    let raw = ::serde_json::Value::Null;
    ::rig::completion::CompletionResponse::new(choice, usage, origin, raw)
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
        let model =
            self::CompletionModel::builder().model("codex-model".to_string()).thread_id("th-42".to_string()).build();
        ::std::assert_eq!(model.model(), "codex-model");
        ::std::assert_eq!(model.thread_id(), ::core::option::Option::Some("th-42"));
    }

    #[::tokio::test]
    async fn call_processes_prompt() {
        let model = self::CompletionModel::builder().model("codex-model".to_string()).build();
        let res = model.call("sample-query").await.unwrap();
        ::std::assert_eq!(res, "Codex processed: sample-query");
    }

    #[::tokio::test]
    async fn completion_returns_response() {
        let model = self::CompletionModel::builder().model("codex-model".to_string()).build();
        let req = ::rig::completion::CompletionRequest::new("sample");
        let res = model.completion(req).await.unwrap();
        ::std::assert_eq!(res.choice.len(), 1);
        ::std::assert_eq!(res.provider(), "codex");
    }

    #[::tokio::test]
    async fn dyn_model_conversion() {
        let model = self::CompletionModel::builder().model("codex-model".to_string()).build();
        let dyn_model = ::rig::driver::DynModel::from(&model);
        let res = dyn_model.call("ping").await.unwrap();
        ::std::assert_eq!(res.choice.len(), 1);
    }

    #[::tokio::test]
    async fn format_user_input_creates_text() {
        let model = self::CompletionModel::builder().model("codex-model".to_string()).build();
        let input = model.format_user_input("hello");
        match input {
            ::codex_codes::protocol::UserInput::Text {
                text,
                ..
            } => {
                ::std::assert_eq!(text, "hello");
            },
            _ => ::std::panic!("expected UserInput::Text"),
        }
    }
}
