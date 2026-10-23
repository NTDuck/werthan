//! Rig completion-model layer for honest benchmark runs.
//! The transport targets any OpenAI-compatible chat-completions endpoint
//! through the stock rig OpenAI provider. Reasoning-heavy backends bury the
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

/// Completion model over the stock rig OpenAI provider.
///
/// The rig client exposes no timeout knob, so the adapter re-imposes the
/// configured inference timeout around every inner exchange. The provider
/// never serializes `max_tokens`, so the adapter injects it through
/// `additional_params`.
#[derive(::core::clone::Clone)]
pub struct OpenAiCompatModel {
    /// Stock rig completion model that reaches the inference endpoint.
    inner: ::rig::providers::openai::CompletionModel,
    /// Whole-request timeout in seconds.
    timeout_secs: ::core::primitive::u64,
    /// Sampling and budget defaults reused for every exchange.
    defaults: self::CompletionDefaults,
}

impl ::core::fmt::Debug for self::OpenAiCompatModel {
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        formatter
            .debug_struct("OpenAiCompatModel")
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
        let api_key = ::core::env!("INFERENCE_API_KEY");
        let client = ::rig::providers::openai::Client::from_url(api_key, &base_url_value);
        let inner = client.completion_model(&model_value);
        ::core::result::Result::Ok(Self {
            timeout_secs: { inference.inference_timeout_secs },
            defaults: self::CompletionDefaults::from_inference(inference),
            inner: { inner },
        })
    }
}

impl ::rig::completion::CompletionModel for self::OpenAiCompatModel {
    type Response = ::rig::providers::openai::CompletionResponse;

    /// Run one completion exchange with the configured defaults and timeout.
    ///
    /// The inner provider future runs async-native, so the adapter needs no
    /// thread bridge. The timeout wrap requires a tokio reactor with the
    /// time driver enabled at the call site.
    fn completion(
        &self,
        request: ::rig::completion::CompletionRequest,
    ) -> impl ::core::future::Future<
        Output = ::core::result::Result<
            ::rig::completion::CompletionResponse<Self::Response>,
            ::rig::completion::CompletionError,
        >,
    > + ::core::marker::Send {
        let mut effective_request = request;
        let effective_temperature = effective_request
            .temperature
            .unwrap_or(self.defaults.temperature);
        effective_request.temperature = ::core::option::Option::Some(effective_temperature);
        let effective_max_tokens = effective_request
            .max_tokens
            .unwrap_or(self.defaults.max_tokens as ::core::primitive::u64);
        let existing_params = effective_request.additional_params.take();
        let merged_params = self::merged_additional_params(
            effective_max_tokens,
            existing_params,
        );
        effective_request.additional_params =
            ::core::option::Option::Some(merged_params);
        let timeout_secs = self.timeout_secs;
        let inner_future = self.inner.completion(effective_request);
        let exchange_future = async move {
            let request_timeout = ::std::time::Duration::from_secs(timeout_secs);
            let timeout_result =
                ::tokio::time::timeout(request_timeout, inner_future).await;
            let inner_result = match timeout_result {
                ::core::result::Result::Ok(inner_result) => inner_result,
                ::core::result::Result::Err(_elapsed) => {
                    return ::core::result::Result::Err(
                        ::rig::completion::CompletionError::ProviderError(::std::format!(
                            "inference timeout after {timeout_secs} seconds"
                        )),
                    );
                },
            };
            let inner_response = match inner_result {
                ::core::result::Result::Ok(response) => response,
                ::core::result::Result::Err(completion_error) => {
                    return ::core::result::Result::Err(completion_error);
                },
            };
            let raw_text = self::assistant_text_of(&inner_response.choice);
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
                raw_response: { inner_response.raw_response },
            })
        };
        exchange_future
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

/// Build the injected additional parameters for one completion request.
///
/// Existing provider parameters win over the injected token budget.
fn merged_additional_params(
    max_tokens: ::core::primitive::u64,
    existing_params: ::core::option::Option<::serde_json::Value>,
) -> ::serde_json::Value {
    let mut params = ::serde_json::Map::new();
    let max_tokens_json = ::serde_json::Value::from(max_tokens);
    params.insert("max_tokens".to_string(), max_tokens_json);
    if let ::core::option::Option::Some(::serde_json::Value::Object(existing)) = existing_params {
        for (key, value) in existing {
            params.insert(key, value);
        }
    }
    ::serde_json::Value::Object(params)
}

/// Extract the text content from one completion choice.
///
/// The helper skips tool-call content. An empty result means the response
/// carried no text channel at all.
fn assistant_text_of(
    choice: &::rig::OneOrMany<::rig::message::AssistantContent>,
) -> ::std::string::String {
    choice
        .iter()
        .find_map(|item| match item {
            ::rig::message::AssistantContent::Text(text) => {
                ::core::option::Option::Some(text.text.clone())
            },
            _ => ::core::option::Option::None,
        })
        .unwrap_or_default()
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
