//! Rig completion-model layer for honest benchmark runs.
//! The transport targets any OpenAI-compatible chat-completions endpoint
//! through a daemon-owned reqwest client. Reasoning-heavy backends bury the
//! answer behind reasoning traces. The adapter reduces every response to
//! its most usable answer text and keeps the error when nothing usable
//! remains.

/// Sampling and budget defaults for calls that the caller leaves unset.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct CompletionDefaults {
    /// Completion token budget per request when the caller sets none.
    pub max_tokens: ::core::primitive::u32,
    /// Sampling temperature applied when the caller sets none.
    pub temperature: ::core::primitive::f64,
}

impl self::CompletionDefaults {
    /// Collect the sampling defaults from the bench inference settings.
    pub fn from_inference(inference: &::configschema::InferenceSection) -> Self {
        Self {
            max_tokens: { inference.inference_max_tokens },
            temperature: { inference.inference_temperature },
        }
    }
}

/// Completion model over a daemon-owned OpenAI-compatible HTTP transport.
///
/// The stock rig OpenAI client builds its own reqwest client with no timeout
/// knob, which let stalled body reads hang past every wall-clock budget. This
/// model owns the HTTP call directly: the reqwest client enforces the whole
/// request deadline (connect, headers, body) inside hyper, independent of any
/// future wrapping. Responses reduce to the most usable answer text before
/// the rig prompt layer ever sees them.
#[derive(::core::clone::Clone)]
pub struct OpenAiCompatModel {
    /// Base URL of the chat-completions endpoint, without the /chat path.
    base_url: ::std::string::String,
    /// Target model identifier sent in every request body.
    model_id: ::std::string::String,
    /// Compiled API key. The local server ignores the Bearer header.
    api_key: ::std::string::String,
    /// Shared HTTP client carrying the hard whole-request deadline.
    http_client: ::reqwest::Client,
    /// Whole-request timeout in seconds.
    timeout_secs: ::core::primitive::u64,
    /// Sampling and budget defaults reused for every exchange.
    defaults: self::CompletionDefaults,
}

impl ::core::fmt::Debug for self::OpenAiCompatModel {
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        formatter
            .debug_struct("OpenAiCompatModel")
            .field("base_url", &self.base_url)
            .field("model_id", &self.model_id)
            .field("timeout_secs", &self.timeout_secs)
            .field("defaults", &self.defaults)
            .finish()
    }
}

impl self::OpenAiCompatModel {
    /// Construct the model from the bench inference settings.
    ///
    /// The local inference server ignores the Bearer header, so the compiled
    /// empty API key stays correct. An empty base URL with no environment
    /// default fails the construction honestly (VAL-DAEMON-02): the contract
    /// forbids a silent fallback backend.
    pub fn from_inference(inference: &::configschema::InferenceSection) -> ::core::result::Result<Self, ::std::string::String> {
        use super::bench_config::InferenceExt;
        let environment_url = super::bench_config::default_base_url();
        let base_url_res = super::bench_smoke::resolve_inference_url(&inference.base_url, &environment_url);
        let base_url_value = match base_url_res {
            ::core::result::Result::Ok(value) => value,
            ::core::result::Result::Err(failure) => {
                return ::core::result::Result::Err(failure);
            },
        };
        let model_value = inference
            .effective_model()
            .unwrap_or_default();
        let request_timeout = ::std::time::Duration::from_secs(inference.inference_timeout_secs);
        let client_build_result = ::reqwest::Client::builder()
            .timeout(request_timeout)
            .connect_timeout(request_timeout)
            .build();
        let http_client = match client_build_result {
            ::core::result::Result::Ok(value) => value,
            ::core::result::Result::Err(build_error) => {
                return ::core::result::Result::Err(::std::format!(
                    "inference HTTP client build failed: {build_error}"
                ));
            },
        };
        ::core::result::Result::Ok(Self {
            base_url: { base_url_value },
            model_id: { model_value },
            api_key: { ::core::env!("INFERENCE_API_KEY") }.to_string(),
            http_client: { http_client },
            timeout_secs: { inference.inference_timeout_secs },
            defaults: self::CompletionDefaults::from_inference(inference),
        })
    }
}

impl ::rig::completion::CompletionModel for self::OpenAiCompatModel {
    type Response = ::serde_json::Value;

    /// Run one completion exchange through the daemon-owned HTTP transport.
    ///
    /// The reqwest client carries the whole-request deadline at the hyper
    /// layer, so a stalled body read returns an error instead of parking
    /// forever. Requires a tokio reactor with the time driver at the call
    /// site.
    fn completion(
        &self,
        request: ::rig::completion::CompletionRequest,
    ) -> impl ::core::future::Future<
        Output = ::core::result::Result<
            ::rig::completion::CompletionResponse<Self::Response>,
            ::rig::completion::CompletionError,
        >,
    > + ::core::marker::Send {
        let temperature = request
            .temperature
            .unwrap_or(self.defaults.temperature);
        let max_tokens = request
            .max_tokens
            .unwrap_or(self.defaults.max_tokens as ::core::primitive::u64);
        let exchange_future = async move {
            let request_payload_build =
                self::request_payload_of(&request, &self.model_id, temperature, max_tokens);
            let request_payload = match request_payload_build {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(build_error) => {
                    return ::core::result::Result::Err(build_error);
                },
            };
        let chat_url = ::std::format!("{}/chat/completions", self.base_url);
            let send_result = self
                .http_client
                .post(&chat_url)
                .bearer_auth(&self.api_key)
                .json(&request_payload)
                .send()
                .await;
            let response = match send_result {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(send_error) => {
                    if send_error.is_timeout() {
                        return ::core::result::Result::Err(
                            ::rig::completion::CompletionError::ProviderError(::std::format!(
                                "inference timeout after {} seconds: {send_error}",
                                self.timeout_secs
                            )),
                        );
                    }
                    return ::core::result::Result::Err(
                        ::rig::completion::CompletionError::ProviderError(::std::format!(
                            "inference transport error: {send_error}"
                        )),
                    );
                },
            };
            let status = response.status();
            let body_result = response.text().await;
            let body_text = match body_result {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(body_error) => {
                    return ::core::result::Result::Err(
                        ::rig::completion::CompletionError::ProviderError(::std::format!(
                            "inference body read failed: {body_error}"
                        )),
                    );
                },
            };
            if !status.is_success() {
                let failure_snippet = self::error_snippet_of(&body_text);
                return ::core::result::Result::Err(
                    ::rig::completion::CompletionError::ProviderError(::std::format!(
                        "inference endpoint status {status}: {failure_snippet}"
                    )),
                );
            }
            let parse_result = ::serde_json::from_str::<self::ChatCompletionsBody>(&body_text);
            let parsed_body = match parse_result {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(parse_error) => {
                    return ::core::result::Result::Err(
                        ::rig::completion::CompletionError::ProviderError(::std::format!(
                            "inference response parse failed: {parse_error}"
                        )),
                    );
                },
            };
            let raw_text = self::response_text_of(&parsed_body);
            let answer_text = self::usable_answer(&raw_text);
            let answer_present = !answer_text.is_empty();
            if !answer_present {
                return ::core::result::Result::Err(
                    ::rig::completion::CompletionError::ProviderError(
                        "no usable answer in response".to_string(),
                    ),
                );
            }
            let answer_content = ::rig::message::AssistantContent::Text(::rig::message::Text {
                text: { answer_text },
            });
            let choice = ::rig::OneOrMany::one(answer_content);
            ::core::result::Result::Ok(::rig::completion::CompletionResponse {
                choice: { choice },
                raw_response: { parsed_body.raw },
            })
        };
        ::std::boxed::Box::pin(exchange_future)
    }
}

/// Completion model wrapper that counts real backend calls for honesty auditing.
#[derive(::core::clone::Clone)]
pub struct CountingCompletionModel<M: ::rig::completion::CompletionModel> {
    /// Wrapped completion model that reaches the real backend.
    inner: M,
    /// Number of completion invocations issued to the backend.
    request_counter: ::std::sync::Arc<::std::sync::atomic::AtomicU64>,
}

impl<M: ::rig::completion::CompletionModel> ::core::fmt::Debug
    for self::CountingCompletionModel<M>
{
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        formatter
            .debug_struct("CountingCompletionModel")
            .field("request_counter", &self.request_counter)
            .finish()
    }
}

impl<M: ::rig::completion::CompletionModel> self::CountingCompletionModel<M> {
    /// Wrap one completion model with a shared request counter.
    pub fn new(
        inner: M,
        request_counter: ::std::sync::Arc<::std::sync::atomic::AtomicU64>,
    ) -> Self {
        Self {
            inner: { inner },
            request_counter,
        }
    }
}

impl<M: ::rig::completion::CompletionModel> ::rig::completion::CompletionModel
    for self::CountingCompletionModel<M>
{
    type Response = <M as ::rig::completion::CompletionModel>::Response;

    fn completion(
        &self,
        request: ::rig::completion::CompletionRequest,
    ) -> impl ::core::future::Future<
        Output = ::core::result::Result<
            ::rig::completion::CompletionResponse<Self::Response>,
            ::rig::completion::CompletionError,
        >,
    > + ::core::marker::Send {
        let inner_future = self.inner.completion(request);
        let counting_future = async move {
            self.request_counter
                .fetch_add(1, ::std::sync::atomic::Ordering::SeqCst);
            inner_future.await
        };
        counting_future
    }
}

/// Parsed OpenAI-compatible chat-completions response body.
#[derive(::serde::Deserialize)]
struct ChatCompletionsBody {
    /// Raw body kept for honesty auditing and logging.
    #[serde(flatten)]
    raw: ::serde_json::Value,
    /// Choice list. The daemon reads only the first text channel.
    #[serde(default)]
    choices: ::std::vec::Vec<self::ChatCompletionsChoice>,
}

/// One choice in a chat-completions response.
#[derive(::serde::Deserialize)]
struct ChatCompletionsChoice {
    /// Assistant message carrying the answer channels.
    #[serde(default)]
    message: ::core::option::Option<self::ChatCompletionsMessage>,
}

/// Assistant message with every text channel the local backend may fill.
#[derive(::serde::Deserialize)]
struct ChatCompletionsMessage {
    /// Primary answer channel.
    #[serde(default)]
    content: ::core::option::Option<::std::string::String>,
    /// Reasoning trace channel some OpenAI-compatible backends fill instead.
    #[serde(default, rename = "reasoning_content")]
    reasoning_content: ::core::option::Option<::std::string::String>,
    /// Refusal channel.
    #[serde(default)]
    refusal: ::core::option::Option<::std::string::String>,
}

/// Serialize one completion request into the OpenAI wire format.
///
/// Documents ride along as a leading user turn. Chat history and the prompt
/// convert through the stock rig OpenAI message format.
fn request_payload_of(
    request: &::rig::completion::CompletionRequest,
    model_id: &::core::primitive::str,
    temperature: ::core::primitive::f64,
    max_tokens: ::core::primitive::u64,
) -> ::core::result::Result<::serde_json::Value, ::rig::completion::CompletionError> {
    let mut messages = ::std::vec::Vec::<::serde_json::Value>::new();
    if let ::core::option::Option::Some(preamble_text) = &request.preamble {
        let preamble_message = self::system_message_value(preamble_text);
        messages.push(preamble_message);
    }
    for document in &request.documents {
        let document_message = self::user_message_value(&document.text);
        messages.push(document_message);
    }
    for history_message in &request.chat_history {
        let wire_messages = ::std::convert::TryInto::<
            ::std::vec::Vec<::rig::providers::openai::Message>,
        >::try_into(history_message.clone())
        .map_err(|conversion_error| {
            ::rig::completion::CompletionError::ProviderError(::std::format!(
                "chat history conversion failed: {conversion_error}"
            ))
        })?;
        for wire_message in wire_messages {
            let serialized = ::serde_json::to_value(&wire_message).map_err(|serialization_error| {
                ::rig::completion::CompletionError::ProviderError(::std::format!(
                    "chat history serialization failed: {serialization_error}"
                ))
            })?;
            messages.push(serialized);
        }
    }
    let prompt_wire_messages =
        ::std::convert::TryInto::<::std::vec::Vec<::rig::providers::openai::Message>>::try_into(
            request.prompt.clone(),
        )
        .map_err(|conversion_error| {
            ::rig::completion::CompletionError::ProviderError(::std::format!(
                "prompt conversion failed: {conversion_error}"
            ))
        })?;
    for wire_message in prompt_wire_messages {
        let serialized = ::serde_json::to_value(&wire_message).map_err(|serialization_error| {
            ::rig::completion::CompletionError::ProviderError(::std::format!(
                "prompt serialization failed: {serialization_error}"
            ))
        })?;
        messages.push(serialized);
    }
    let payload = self::chat_payload_value(model_id, messages, temperature, max_tokens);
    ::core::result::Result::Ok(payload)
}

/// Build one wire system message value.
fn system_message_value(
    text: &::core::primitive::str,
) -> ::serde_json::Value {
    let mut message = ::serde_json::Map::new();
    let role_value = ::serde_json::Value::from("system");
    message.insert("role".to_string(), role_value);
    let content_value = ::serde_json::Value::from(text);
    message.insert("content".to_string(), content_value);
    ::serde_json::Value::Object(message)
}

/// Build one wire user message value.
fn user_message_value(
    text: &::core::primitive::str,
) -> ::serde_json::Value {
    let mut message = ::serde_json::Map::new();
    let role_value = ::serde_json::Value::from("user");
    message.insert("role".to_string(), role_value);
    let content_value = ::serde_json::Value::from(text);
    message.insert("content".to_string(), content_value);
    ::serde_json::Value::Object(message)
}

/// Build the chat-completions request payload.
fn chat_payload_value(
    model_id: &::core::primitive::str,
    messages: ::std::vec::Vec<::serde_json::Value>,
    temperature: ::core::primitive::f64,
    max_tokens: ::core::primitive::u64,
) -> ::serde_json::Value {
    let mut payload = ::serde_json::Map::new();
    let model_value = ::serde_json::Value::from(model_id);
    payload.insert("model".to_string(), model_value);
    let messages_value = ::serde_json::Value::from(messages);
    payload.insert("messages".to_string(), messages_value);
    let temperature_value = ::serde_json::Value::from(temperature);
    payload.insert("temperature".to_string(), temperature_value);
    let max_tokens_value = ::serde_json::Value::from(max_tokens);
    payload.insert("max_tokens".to_string(), max_tokens_value);
    ::serde_json::Value::Object(payload)
}

/// Parse the raw chat-completions JSON into its raw value and answer text.
///
/// The text channel falls back through: assistant text content, the
/// `reasoning_content` field many OpenAI-compatible backends fill instead of
/// `content`, and the refusal channel.
fn response_text_of(body: &self::ChatCompletionsBody) -> ::std::string::String {
    let mut collected = ::std::string::String::new();
    for choice in &body.choices {
        if let ::core::option::Option::Some(message) = &choice.message {
            if let ::core::option::Option::Some(content_text) = &message.content {
                collected.push_str(content_text);
            }
            if let ::core::option::Option::Some(reasoning_text) = &message.reasoning_content {
                collected.push_str(reasoning_text);
            }
            if let ::core::option::Option::Some(refusal_text) = &message.refusal {
                collected.push_str(refusal_text);
            }
        }
    }
    collected
}

/// Cut one long failed body down to a bounded snippet for error reporting.
fn error_snippet_of(body_text: &::core::primitive::str) -> ::std::string::String {
    const SNIPPET_LIMIT: ::core::primitive::usize = 240;
    let collapsed = body_text.split_whitespace().collect::<::std::vec::Vec<_>>().join(" ");
    if collapsed.len() > SNIPPET_LIMIT {
        collapsed[..SNIPPET_LIMIT].to_string()
    } else {
        collapsed
    }
}

/// Reduce a reasoning-dominant response to its most usable answer text.
///
/// The heuristic order: a fenced code block if present, then the text after
/// the last reasoning closing tag. A truncated reasoning trace with no
/// conclusion is prose, never code. It returns empty so the caller keeps
/// the primary error instead of corrupting the target workspace.
fn usable_answer(response_text: &::core::primitive::str) -> ::std::string::String {
    let fenced = self::extract_code(response_text);
    let fence_present = fenced.contains("fn ") || fenced.contains("pub ");
    if fence_present {
        return fenced;
    }
    let think_closed = response_text.rfind("</think");
    if let ::core::option::Option::Some(close_index) = think_closed {
        let after_close = &response_text[close_index..];
        let tag_end = after_close.find('>');
        if let ::core::option::Option::Some(tag_end_index) = tag_end {
            let tail = after_close[tag_end_index + 1..].trim();
            let tail_present = !tail.is_empty();
            if tail_present {
                return tail.to_string();
            }
        }
    }
    ::std::string::String::new()
}

/// Extract the first fenced Rust code block from a model response.
/// Unfenced responses come back trimmed so bare code still works.
fn extract_code(response_text: &::core::primitive::str) -> ::std::string::String {
    let fence_token = "```";
    let fence_start = match response_text.find(fence_token) {
        ::core::option::Option::Some(found_index) => found_index + fence_token.len(),
        ::core::option::Option::None => return response_text.trim().to_string(),
    };
    let after_language_line = match response_text[fence_start..].find('\n') {
        ::core::option::Option::Some(line_break) => fence_start + line_break + 1,
        ::core::option::Option::None => return response_text.trim().to_string(),
    };
    let remainder = &response_text[after_language_line..];
    let code_body = match remainder.find(fence_token) {
        ::core::option::Option::Some(close_index) => &remainder[..close_index],
        ::core::option::Option::None => remainder,
    };
    code_body.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usable_answer_prefers_fenced_code() {
        let response = "thinking about it\n```rust\npub fn one() -> u8 { 1 }\n```";
        let answer = self::usable_answer(response);
        ::std::assert!(answer.contains("pub fn one"));
    }

    #[test]
    fn usable_answer_uses_text_after_think_close() {
        let response = "long reasoning here </think\n> The answer is 4";
        let answer = self::usable_answer(response);
        ::std::assert!(answer.contains("The answer is 4"));
    }

    #[test]
    fn usable_answer_rejects_truncated_prose_without_think_close() {
        // The corrupted lib.rs failure mode: the reasoning budget ran out
        // mid-thought and no conclusion was ever written.
        let response = "We need answer user's request. Need write lib.rs with vec functions. But wait";
        let answer = self::usable_answer(response);
        ::std::assert!(answer.is_empty(), "prose must be rejected, got: {answer}");
    }
}
