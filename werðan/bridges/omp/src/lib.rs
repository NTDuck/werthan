//! Omp RPC bridge implementation.

/// Completion model configuration for the omp RPC protocol.
#[derive(::bon::Builder, ::core::clone::Clone)]
pub struct CompletionModel {
    /// Target model identifier.
    pub model: ::std::string::String,
    /// Attached omp RPC client.
    pub client: ::core::option::Option<::std::sync::Arc<::omp_rpc::Client>>,
}

impl self::CompletionModel {
    /// Return the target model identifier.
    pub fn model(&self) -> &::core::primitive::str {
        &self.model
    }

    /// Return the attached client when configured.
    pub fn client(&self) -> ::core::option::Option<&::omp_rpc::Client> {
        self.client.as_deref()
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
            client: { self.client.clone() },
            model: { self.model.clone() },
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
#[derive(::core::clone::Clone)]
pub struct OmpTransport {
    /// Attached omp client.
    pub client: ::core::option::Option<::std::sync::Arc<::omp_rpc::Client>>,
    /// Model identifier.
    pub model: ::std::string::String,
}

impl ::rig::driver::Transport<self::OmpWire> for self::OmpTransport {
    fn send(
        &self,
        request: ::rig::completion::CompletionRequest,
        _exchange: ::rig::driver::Exchange,
    ) -> ::rig::driver::Opening<self::OmpFrame> {
        let client_opt = self.client.clone();
        let model = self.model.clone();
        let prompt = extract_prompt(&request);
        ::rig::driver::Opening::new(async move {
            let text = execute_prompt(&client_opt, &prompt).await?;
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

async fn execute_prompt(
    client_opt: &::core::option::Option<::std::sync::Arc<::omp_rpc::Client>>,
    prompt: &::core::primitive::str,
) -> ::core::result::Result<::std::string::String, ::rig::error::ProviderError> {
    if let ::core::option::Option::Some(client) = client_opt {
        return execute_client(client, prompt).await;
    }
    let echo = execute_offline(prompt);
    ::core::result::Result::Ok(echo)
}

async fn execute_client(
    client: &::std::sync::Arc<::omp_rpc::Client>,
    prompt: &::core::primitive::str,
) -> ::core::result::Result<::std::string::String, ::rig::error::ProviderError> {
    let client_clone = ::std::sync::Arc::clone(client);
    let message = prompt.to_string();
    let cmd = ::omp_rpc::PromptCommand {
        message,
        images: ::core::option::Option::None,
        streaming_behavior: ::core::option::Option::None,
    };
    let timeout = ::std::time::Duration::from_secs(60);
    let task_result = ::tokio::task::spawn_blocking(move || client_clone.prompt_and_wait(&cmd, timeout)).await;
    let turn_result = task_result.map_err(|e| ::rig::error::ProviderError::Provider(e.to_string()))?;
    let turn = turn_result.map_err(|e| ::rig::error::ProviderError::Provider(e.to_string()))?;
    let default_echo = ::std::format!("Echo: {prompt}");
    let text = turn.assistant_text.unwrap_or(default_echo);
    ::core::result::Result::Ok(text)
}

fn execute_offline(prompt: &::core::primitive::str) -> ::std::string::String {
    ::std::format!("Echo: {prompt}")
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
    let api = ::rig::message::Api::from_static("omp.rpc");
    let origin = ::rig::message::Origin::new(api, "omp", model);
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
        let model = self::CompletionModel::builder().model("omp-default".to_string()).build();
        ::std::assert_eq!(model.model(), "omp-default");
        ::std::assert!(model.client().is_none());
    }

    #[::tokio::test]
    async fn offline_call_echos_prompt() {
        let model = self::CompletionModel::builder().model("omp-default".to_string()).build();
        let res = model.call("hello").await.unwrap();
        ::std::assert_eq!(res, "Echo: hello");
    }

    #[::tokio::test]
    async fn completion_returns_response() {
        let model = self::CompletionModel::builder().model("omp-default".to_string()).build();
        let req = ::rig::completion::CompletionRequest::new("hello");
        let res = model.completion(req).await.unwrap();
        ::std::assert_eq!(res.choice.len(), 1);
        ::std::assert_eq!(res.provider(), "omp");
    }

    #[::tokio::test]
    async fn dyn_model_conversion() {
        let model = self::CompletionModel::builder().model("omp-default".to_string()).build();
        let dyn_model = ::rig::driver::DynModel::from(&model);
        let res = dyn_model.call("ping").await.unwrap();
        ::std::assert_eq!(res.choice.len(), 1);
    }
}
