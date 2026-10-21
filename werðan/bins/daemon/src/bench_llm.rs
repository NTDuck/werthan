//! Rig completion-model layer for honest benchmark runs.
//! The 27B reasoning backend spends its token budget in `reasoning_content`
//! and may return an empty `content` field. The Qwen model unwraps the
//! reasoning channel, extracts the answer block, and keeps the error when
//! nothing usable remains.

/// Configuration for the Qwen chat-completions backend.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct QwenConfig {
    /// Base URL of the OpenAI-compatible inference server.
    pub base_url: ::std::string::String,
    /// Model identifier requested on every completion call.
    pub model: ::std::string::String,
    /// Whole-request timeout in seconds.
    pub timeout_secs: ::core::primitive::u64,
    /// Completion token budget per request when the caller sets none.
    pub max_tokens: ::core::primitive::u32,
    /// Sampling temperature added when the caller sets none.
    pub temperature: ::core::primitive::f64,
}

impl self::QwenConfig {
    /// Collect the backend configuration from the bench inference settings.
    pub fn from_inference(inference: &super::bench_config::InferenceConfig) -> Self {
        let base_url_value = inference.effective_base_url();
        let model_value = inference
            .effective_model()
            .unwrap_or_default();
        Self {
            base_url: { base_url_value },
            model: { model_value },
            timeout_secs: { inference.inference_timeout_secs },
            max_tokens: { inference.inference_max_tokens },
            temperature: { inference.inference_temperature },
        }
    }
}

/// OpenAI-compatible completion model backed by blocking reqwest HTTP.
#[derive(::core::clone::Clone)]
pub struct QwenReasoningModel {
    /// Backend configuration reused for every exchange.
    config: self::QwenConfig,
    /// Blocking HTTP client that runs each chat-completions exchange.
    http: ::reqwest::blocking::Client,
}

impl ::core::fmt::Debug for self::QwenReasoningModel {
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        formatter
            .debug_struct("QwenReasoningModel")
            .field("config", &self.config)
            .finish()
    }
}

impl self::QwenReasoningModel {
    /// Construct the model from explicit configuration.
    pub fn new(config: self::QwenConfig) -> Self {
        let request_timeout = ::std::time::Duration::from_secs(config.timeout_secs);
        let built = ::reqwest::blocking::Client::builder()
            .timeout(request_timeout)
            .build();
        let http = built.expect("blocking http client construction succeeds");
        Self { config, http }
    }
}

impl ::rig::completion::CompletionModel for self::QwenReasoningModel {
    type Response = ::serde_json::Value;

    /// Run one chat-completions exchange for the request.
    ///
    /// The HTTP call inside this future is blocking reqwest.
    /// We keep it in the async body on purpose.
    /// The daemon drives futures on a dedicated runtime per call.
    /// Blocking here matches the previous always-blocking client.
    /// The body never spawns threads of its own.
    fn completion(
        &self,
        request: ::rig::completion::CompletionRequest,
    ) -> impl ::core::future::Future<
        Output = ::core::result::Result<
            ::rig::completion::CompletionResponse<::serde_json::Value>,
            ::rig::completion::CompletionError,
        >,
    > + ::core::marker::Send {
        let prompt_text = self::prompt_text_of(&request);
        let preamble_clone = request.preamble.clone();
        let preamble_text = preamble_clone.unwrap_or_default();
        let request_temperature = request.temperature.unwrap_or(self.config.temperature);
        let request_max_tokens = request
            .max_tokens
            .unwrap_or(self.config.max_tokens as ::core::primitive::u64);
        let max_tokens = request_max_tokens as ::core::primitive::u32;
        let http = self.http.clone();
        let config = self.config.clone();
        let exchange_future = async move {
            let (sender, receiver) = ::std::sync::mpsc::channel();
            let handle_res = ::std::thread::Builder::new()
                .name("qwen-exchange".to_string())
                .spawn(move || {
                    let exchange_res = self::blocking_exchange(
                        &http,
                        &config,
                        &preamble_text,
                        &prompt_text,
                        max_tokens,
                        request_temperature,
                    );
                    let _ = sender.send(exchange_res);
                });
            if let ::core::result::Result::Err(spawn_err) = handle_res {
                return ::core::result::Result::Err(
                    ::rig::completion::CompletionError::ResponseError(spawn_err.to_string()),
                );
            }
            let response_json_res = match receiver.recv() {
                ::core::result::Result::Ok(res) => res,
                ::core::result::Result::Err(recv_err) => {
                    return ::core::result::Result::Err(
                        ::rig::completion::CompletionError::ResponseError(recv_err.to_string()),
                    );
                }
            };
            let response_json = match response_json_res {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(exchange_error) => {
                    return ::core::result::Result::Err(exchange_error);
                },
            };
            let answer_text = self::usable_answer_of(&response_json);
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
                raw_response: { response_json },
            })
        };
        exchange_future
    }
}

/// Extract the user prompt text from one completion request.
fn prompt_text_of(
    request: &::rig::completion::CompletionRequest,
) -> ::std::string::String {
    match &request.prompt {
        ::rig::message::Message::User { content } => content
            .iter()
            .find_map(|item| match item {
                ::rig::message::UserContent::Text(text) => {
                    ::core::option::Option::Some(text.text.clone())
                },
                _ => ::core::option::Option::None,
            })
            .unwrap_or_default(),
        _ => ::std::string::String::new(),
    }
}

/// Post one blocking chat-completions exchange and return the raw JSON body.
fn blocking_exchange(
    http: &::reqwest::blocking::Client,
    config: &self::QwenConfig,
    system_prompt: &::core::primitive::str,
    user_prompt: &::core::primitive::str,
    max_tokens: ::core::primitive::u32,
    temperature: ::core::primitive::f64,
) -> ::core::result::Result<
    ::serde_json::Value,
    ::rig::completion::CompletionError,
> {
    let switched_prompt = ::std::format!("{user_prompt}\n/no_think");
    let base_trimmed = config.base_url.trim_end_matches('/');
    let chat_url = ::std::format!("{}/chat/completions", base_trimmed);
    let request_payload = self::request_payload_of(
        config,
        system_prompt,
        &switched_prompt,
        max_tokens,
        temperature,
    );
    ::tracing::info!("blocking_exchange: sending POST to {}", chat_url);
    let http_response_res = http
        .post(chat_url)
        .json(&request_payload)
        .send();
    let http_response = match http_response_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(send_error) => {
            return ::core::result::Result::Err(
                ::rig::completion::CompletionError::ProviderError(::std::format!(
                    "transport failure: {send_error}"
                )),
            );
        },
    };
    let response_status = http_response.status();
    let status_ok = response_status.is_success();
    ::tracing::info!("blocking_exchange: received status {}", response_status);
    if !status_ok {
        let body_excerpt = http_response.text().unwrap_or_default();
        return ::core::result::Result::Err(
            ::rig::completion::CompletionError::ProviderError(::std::format!(
                "backend status {}: {body_excerpt}",
                response_status.as_u16()
            )),
        );
    }
    let parse_res = http_response.json::<::serde_json::Value>();
    let response_json = match parse_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(parse_error) => {
            return ::core::result::Result::Err(
                ::rig::completion::CompletionError::ProviderError(::std::format!(
                    "malformed response: {parse_error}"
                )),
            );
        },
    };
    ::core::result::Result::Ok(response_json)
}

/// Assemble the chat-completions JSON request body.
fn request_payload_of(
    config: &self::QwenConfig,
    system_prompt: &::core::primitive::str,
    user_prompt: &::core::primitive::str,
    max_tokens: ::core::primitive::u32,
    temperature: ::core::primitive::f64,
) -> ::serde_json::Value {
    let system_message = self::message_object("system", system_prompt);
    let user_message = self::message_object("user", user_prompt);
    let pair_messages = ::std::vec![
        ::serde_json::Value::Object(system_message),
        ::serde_json::Value::Object(user_message),
    ];
    let model_json = ::serde_json::Value::String(config.model.clone());
    let messages_json = ::serde_json::Value::Array(pair_messages);
    let max_tokens_number = ::serde_json::Number::from(max_tokens);
    let zero_number = ::serde_json::Number::from(0);
    let temperature_number =
        ::serde_json::Number::from_f64(temperature).unwrap_or(zero_number);
    let max_tokens_json = ::serde_json::Value::Number(max_tokens_number);
    let temperature_json = ::serde_json::Value::Number(temperature_number);
    let mut request_payload = ::serde_json::Map::new();
    request_payload.insert("model".to_string(), model_json);
    request_payload.insert("messages".to_string(), messages_json);
    request_payload.insert("max_tokens".to_string(), max_tokens_json);
    request_payload.insert("temperature".to_string(), temperature_json);
    ::serde_json::Value::Object(request_payload)
}

/// Build one chat message JSON object.
fn message_object(
    role: &::core::primitive::str,
    content: &::core::primitive::str,
) -> ::serde_json::Map<::std::string::String, ::serde_json::Value> {
    let role_json = ::serde_json::Value::String(role.to_string());
    let content_json = ::serde_json::Value::String(content.to_string());
    let mut message = ::serde_json::Map::new();
    message.insert("role".to_string(), role_json);
    message.insert("content".to_string(), content_json);
    message
}

/// Reduce one chat-completions response to its most usable answer text.
///
/// Empty output means nothing usable remained. The caller keeps the error
/// instead of corrupting the target workspace.
fn usable_answer_of(response_json: &::serde_json::Value) -> ::std::string::String {
    let assistant_text = self::assistant_text_of(response_json);
    let reduced = self::usable_answer(&assistant_text);
    reduced
}

/// Extract the assistant text from one chat-completions JSON response.
///
/// The content channel answers first. An empty content channel switches to
/// the reasoning channel, because the 27B reasoning backend may spend the
/// whole token budget inside `reasoning_content`.
fn assistant_text_of(response_json: &::serde_json::Value) -> ::std::string::String {
    let message_json = response_json
        .get("choices")
        .and_then(|choices_value| choices_value.get(0))
        .and_then(|first_choice| first_choice.get("message"));
    let message_present = message_json.is_some();
    if !message_present {
        return ::std::string::String::new();
    }
    let message_value = message_json.expect("assistant message checked above");
    let content_value = message_value
        .get("content")
        .and_then(|content_field| content_field.as_str())
        .unwrap_or("");
    let content_present = !content_value.trim().is_empty();
    if content_present {
        return content_value.to_string();
    }
    message_value
        .get("reasoning_content")
        .and_then(|reasoning_field| reasoning_field.as_str())
        .unwrap_or("")
        .to_string()
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

/// Completion model wrapper that counts real backend calls for honesty auditing.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct CountingCompletionModel<M: ::rig::completion::CompletionModel> {
    /// Wrapped completion model that reaches the real backend.
    inner: M,
    /// Number of completion invocations issued to the backend.
    request_counter: ::std::sync::Arc<::std::sync::atomic::AtomicU64>,
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

    #[allow(dead_code)]
    /// Read the current request count for the honesty gate.
    pub fn request_count(&self) -> ::core::primitive::u64 {
        self.request_counter
            .load(::std::sync::atomic::Ordering::SeqCst)
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
