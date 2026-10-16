//! Codex protocol bridge implementation.

/// Default timeout for completion execution.
pub const DEFAULT_TIMEOUT: ::std::time::Duration = ::std::time::Duration::from_secs(60);

/// Default base URL for model inference loaded at compile time from .env.
pub const DEFAULT_BASE_URL: &::core::primitive::str = ::core::env!("BASE_URL");

/// Default model identifier loaded at compile time from .env.
pub const DEFAULT_MODEL_ID: &::core::primitive::str = ::core::env!("MODEL_ID");

/// Completion model configuration for Codex app-server protocol.
#[derive(::bon::Builder, ::core::clone::Clone, ::core::fmt::Debug)]
pub struct CodexCompletionModel {
    /// Target model identifier.
    #[builder(into, default = self::DEFAULT_MODEL_ID.to_string())]
    pub model: ::std::string::String,

    /// Base URL for the OpenAI-compatible inference endpoint.
    #[builder(into, default = self::DEFAULT_BASE_URL.to_string())]
    pub base_url: ::std::string::String,

    /// Optional thread identifier.
    pub thread_id: ::core::option::Option<::std::string::String>,

    /// Execution timeout duration.
    #[builder(default = self::DEFAULT_TIMEOUT)]
    pub timeout: ::std::time::Duration,

    /// Sampling temperature configured for deterministic high-quality reasoning.
    #[builder(default = 0.2)]
    pub temperature: ::core::primitive::f64,
}

impl self::CodexCompletionModel {
    /// Return the target model identifier.
    pub fn model(&self) -> &::core::primitive::str {
        &self.model
    }

    /// Return the thread identifier when configured.
    pub fn thread_id(&self) -> ::core::option::Option<&::core::primitive::str> {
        self.thread_id.as_deref()
    }

    /// Return the endpoint base URL.
    pub fn base_url(&self) -> &::core::primitive::str {
        &self.base_url
    }

    /// Return the execution timeout duration.
    pub fn timeout(&self) -> ::std::time::Duration {
        self.timeout
    }

    /// Return the sampling temperature.
    pub fn temperature(&self) -> ::core::primitive::f64 {
        self.temperature
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

    /// Convert this configuration into a native Rig driver model.
    pub fn into_driver(&self) -> ::rig::driver::Model<self::CodexWire, self::CodexTransport> {
        let wire = self::CodexWire::new(&self.model);
        let transport = self::CodexTransport::builder()
            .maybe_thread_id(self.thread_id.clone())
            .model(self.model.clone())
            .base_url(self.base_url.clone())
            .timeout(self.timeout)
            .temperature(self.temperature)
            .build();
        ::rig::driver::Model::new(wire, transport)
    }
}

/// Wire protocol implementation for Codex app-server.
#[derive(::bon::Builder, ::core::clone::Clone, ::core::fmt::Debug)]
pub struct CodexWire {
    /// Model identifier.
    #[builder(into)]
    pub model: ::std::string::String,
}

impl self::CodexWire {
    /// Create a new codex wire instance.
    pub fn new(model: impl ::core::convert::Into<::std::string::String>) -> Self {
        Self::builder().model(model.into()).build()
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
#[derive(::bon::Builder, ::core::clone::Clone, ::core::fmt::Debug)]
pub struct CodexTransport {
    /// Thread identifier.
    pub thread_id: ::core::option::Option<::std::string::String>,

    /// Model identifier.
    #[builder(into)]
    pub model: ::std::string::String,

    /// Base URL for inference endpoint.
    #[builder(into)]
    pub base_url: ::std::string::String,

    /// Timeout duration.
    pub timeout: ::std::time::Duration,

    /// Sampling temperature.
    pub temperature: ::core::primitive::f64,
}

impl ::rig::driver::Transport<self::CodexWire> for self::CodexTransport {
    fn send(
        &self,
        request: ::rig::completion::CompletionRequest,
        _exchange: ::rig::driver::Exchange,
    ) -> ::rig::driver::Opening<self::CodexFrame> {
        let model = self.model.clone();
        let base_url = self.base_url.clone();
        let timeout = self.timeout;
        let temperature = self.temperature;
        let prompt_text = extract_prompt(&request);
        ::rig::driver::Opening::new(async move {
            let client = ::reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .map_err(|e| ::rig::error::ProviderError::Provider(e.to_string()))?;
            let url = ::std::format!("{base_url}/chat/completions");
            let message_item = self::ChatMessagePayload::builder().role("user").content(prompt_text.as_str()).build();
            let body = self::ChatCompletionPayload::builder()
                .model(model.as_str())
                .messages([message_item])
                .stream(true)
                .max_tokens(1024)
                .temperature(temperature)
                .build();
            ::tracing::info!("codex bridge sending prompt request to inference endpoint");
            let res = client
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| ::rig::error::ProviderError::Provider(e.to_string()))?;
            let (tx, rx) = ::tokio::sync::mpsc::channel(64);
            ::tokio::spawn(async move {
                use ::futures_util::StreamExt;
                let mut byte_stream = res.bytes_stream();
                let mut buf = ::std::string::String::new();
                let mut accumulated = ::std::string::String::new();
                while let ::core::option::Option::Some(chunk_res) = byte_stream.next().await {
                    let chunk = match chunk_res {
                        ::core::result::Result::Ok(c) => c,
                        ::core::result::Result::Err(_) => break,
                    };
                    let chunk_str = ::std::string::String::from_utf8_lossy(&chunk);
                    buf.push_str(&chunk_str);
                    while let ::core::option::Option::Some(pos) = buf.find('\n') {
                        let line = buf[..pos].trim().to_string();
                        buf = buf[pos + 1..].to_string();
                        if let ::core::option::Option::Some(data) = line.strip_prefix("data: ") {
                            let trimmed = data.trim();
                            if trimmed == "[DONE]" {
                                break;
                            }
                            if let ::core::result::Result::Ok(json) =
                                ::serde_json::from_str::<::serde_json::Value>(trimmed)
                            {
                                let content = json["choices"][0]["delta"]["content"].as_str().unwrap_or("");
                                let reasoning = json["choices"][0]["delta"]["reasoning_content"].as_str().unwrap_or("");
                                let token = if !content.is_empty() {
                                    content
                                } else {
                                    reasoning
                                };
                                if !token.is_empty() {
                                    accumulated.push_str(token);
                                    let frame = self::CodexFrame::Chunk(token.to_string());
                                    if tx.send(::core::result::Result::Ok(frame)).await.is_err() {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
                let response = create_response(accumulated, &model);
                let boxed = ::std::boxed::Box::new(response);
                let final_frame = self::CodexFrame::Response(boxed);
                let _ = tx.send(::core::result::Result::Ok(final_frame)).await;
            });

            let rx_stream = ::tokio_stream::wrappers::ReceiverStream::new(rx);
            let opened = ::rig::driver::Opened::new(rx_stream);
            ::core::result::Result::Ok(opened)
        })
    }
}

/// Frame type emitted by the codex transport.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub enum CodexFrame {
    /// Incremental text chunk.
    Chunk(::std::string::String),
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
            self::CodexFrame::Chunk(_) => {},
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
            self::CodexFrame::Chunk(text) => {
                out.run(::rig::operation::Block::Text, &text)?;
                ::core::result::Result::Ok(::rig::wire::Flow::More)
            },
            self::CodexFrame::Response(resp) => {
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
    request
        .chat_history
        .iter()
        .filter_map(::rig::message::Message::rag_text)
        .collect::<::std::vec::Vec<_>>()
        .join(" ")
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

#[derive(::bon::Builder, ::serde::Serialize)]
struct ChatMessagePayload<'a> {
    role: &'static ::core::primitive::str,
    content: &'a ::core::primitive::str,
}

#[derive(::bon::Builder, ::serde::Serialize)]
struct ChatCompletionPayload<'a> {
    model: &'a ::core::primitive::str,
    messages: [ChatMessagePayload<'a>; 1],
    stream: ::core::primitive::bool,
    max_tokens: ::core::primitive::u64,
    temperature: ::core::primitive::f64,
}
