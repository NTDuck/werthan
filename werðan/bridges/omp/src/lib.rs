//! Omp RPC bridge implementation.

/// Default timeout for completion execution.
pub const DEFAULT_TIMEOUT: ::std::time::Duration = ::std::time::Duration::from_secs(60);

/// Completion model configuration for the omp RPC protocol.
#[derive(::bon::Builder, ::core::clone::Clone, ::core::fmt::Debug)]
pub struct OmpCompletionModel {
    /// Target model identifier.
    pub model: ::std::string::String,
    /// Execution timeout duration.
    #[builder(default = self::DEFAULT_TIMEOUT)]
    pub timeout: ::std::time::Duration,
}

/// Alias for backwards compatibility.
pub type CompletionModel = self::OmpCompletionModel;

impl self::OmpCompletionModel {
    /// Return the target model identifier.
    pub fn model(&self) -> &::core::primitive::str {
        &self.model
    }

    /// Return the execution timeout duration.
    pub fn timeout(&self) -> ::std::time::Duration {
        self.timeout
    }

    /// Convert this completion model into a native Rig agent.
    pub fn into_agent(self, preamble: &::core::primitive::str) -> ::rig::agent::Agent {
        let dyn_model = self.into_dyn_model();
        ::rig::agent::AgentBuilder::new(dyn_model).preamble(preamble).build()
    }

    /// Convert this configuration into an erased dynamic completion model.
    pub fn into_dyn_model(&self) -> ::rig::driver::DynModel<::rig::operation::Completion> {
        let driver = self.into_driver();
        driver.erase()
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
    pub fn into_driver(&self) -> ::rig::driver::Model<self::OmpWire, self::OmpTransport> {
        let wire = self::OmpWire::new(&self.model);
        let transport = self::OmpTransport {
            model: { self.model.clone() },
            timeout: { self.timeout },
        };
        ::rig::driver::Model::new(wire, transport)
    }
}

/// Wire protocol implementation for omp RPC.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct OmpWire {
    /// Model identifier.
    pub model: ::std::string::String,
}

impl self::OmpWire {
    /// Create a new omp wire instance.
    pub fn new(model: impl ::core::convert::Into<::std::string::String>) -> Self {
        Self {
            model: { model.into() },
        }
    }
}

impl ::rig::completion::ReplayTarget for self::OmpWire {
    fn api(&self) -> ::rig::message::Api {
        ::rig::message::Api::from_static("omp.rpc")
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
            true => ::rig::completion::options::Mapping::Omit("omp ignores options"),
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
        "omp"
    }

    fn model(&self) -> &::core::primitive::str {
        &self.model
    }

    fn accepts(&self, _model: &::core::primitive::str) -> ::rig::completion::Accepts {
        ::rig::completion::Accepts::ALL
    }
}

impl ::rig::wire::Wire for self::OmpWire {
    type Decoder<'id> = self::OmpDecoder<'id>;
    type Frame = self::OmpFrame;
    type Op = ::rig::operation::Completion;
    type Payload = ::rig::completion::CompletionRequest;
    type Reassembler = self::OmpDocument;

    fn describe(&self) -> ::rig::wire::Descriptor<'_> {
        let desc = ::rig::wire::Descriptor::new("omp");
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
        let dec = self::OmpDecoder::default();
        dec
    }
}

/// Transport implementation executing prompt commands over omp RPC.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct OmpTransport {
    /// Model identifier.
    pub model: ::std::string::String,
    /// Timeout duration.
    pub timeout: ::std::time::Duration,
}

impl ::rig::driver::Transport<self::OmpWire> for self::OmpTransport {
    fn send(
        &self,
        request: ::rig::completion::CompletionRequest,
        _exchange: ::rig::driver::Exchange,
    ) -> ::rig::driver::Opening<self::OmpFrame> {
        let model = self.model.clone();
        let prompt_text = extract_prompt(&request);
        ::rig::driver::Opening::new(async move {
            let text = ::std::format!("[omp-offline: {prompt_text}]");
            let response = create_response(text, &model);
            let boxed = ::std::boxed::Box::new(response);
            let frame = self::OmpFrame::Response(boxed);
            let stream = ::futures_util::stream::once(async move { ::core::result::Result::Ok(frame) });
            let opened = ::rig::driver::Opened::new(stream);
            ::core::result::Result::Ok(opened)
        })
    }
}

/// Frame type emitted by the omp transport.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub enum OmpFrame {
    /// Full turn completion response.
    Response(::std::boxed::Box<::rig::completion::CompletionResponse>),
}

/// Reply document reassembler for omp frames.
#[derive(::core::fmt::Debug, ::core::default::Default)]
pub struct OmpDocument {
    document: ::core::option::Option<::serde_json::Value>,
}

impl ::rig::wire::document::Serves<::rig::operation::Completion> for self::OmpDocument {}

impl ::rig::wire::document::Reassemble<self::OmpFrame> for self::OmpDocument {
    fn absorb(&mut self, frame: &self::OmpFrame) {
        match frame {
            self::OmpFrame::Response(resp) => {
                self.document = ::core::option::Option::Some(resp.raw.clone());
            },
        }
    }

    fn finish(self) -> ::serde_json::Value {
        let null_val = ::serde_json::Value::Null;
        self.document.unwrap_or(null_val)
    }
}

/// Stream decoder for omp frames.
#[derive(::core::default::Default)]
pub struct OmpDecoder<'id> {
    marker: ::core::marker::PhantomData<&'id ::core::primitive::bool>,
}

impl<'id> ::rig::wire::Decoder<'id, ::rig::operation::Completion, self::OmpFrame> for self::OmpDecoder<'id> {
    type Event = self::OmpFrame;

    fn classify(&self, frame: self::OmpFrame) -> ::rig::wire::WireEvent<self::OmpFrame> {
        ::rig::wire::WireEvent::Known(frame)
    }

    fn decode(
        &mut self,
        frame: self::OmpFrame,
        mut out: ::rig::wire::Out<'id, ::rig::operation::Completion>,
    ) -> ::core::result::Result<::rig::wire::Flow, ::rig::error::ProviderError> {
        match frame {
            self::OmpFrame::Response(resp) => {
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
        if let ::core::option::Option::Some(text) = msg.rag_text() {
            parts.push(text);
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
    let api = ::rig::message::Api::from_static("omp.rpc");
    let origin = ::rig::message::Origin::new(api, "omp", model);
    let raw = ::serde_json::Value::Null;
    ::rig::completion::CompletionResponse::new(choice, usage, origin, raw)
}

impl ::core::convert::From<&self::OmpCompletionModel> for ::rig::driver::DynModel<::rig::operation::Completion> {
    fn from(model: &self::OmpCompletionModel) -> Self {
        model.into_driver().erase()
    }
}

impl ::core::convert::From<self::OmpCompletionModel> for ::rig::driver::DynModel<::rig::operation::Completion> {
    fn from(model: self::OmpCompletionModel) -> Self {
        model.into_driver().erase()
    }
}
