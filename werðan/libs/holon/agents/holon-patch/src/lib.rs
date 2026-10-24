//! Surgical code repair agent for the ARIADNE cluster.
//! Exposes one Rig agent with the repair preamble wired, plus the domain
//! call that prompts it and extracts the repaired module source.

/// System prompt steering the model toward repaired module code in the
/// configured target language.
pub fn patch_system_prompt(target_language: &::core::primitive::str) -> ::std::string::String {
    ::std::format!(
        "You are a {target_language} code repair engine for a legacy modernization pipeline. \
Output only the repaired complete {target_language} source for one module with no markdown and no commentary. \
Keep every public signature from the candidate unchanged unless a contract requires otherwise. \
The repaired code must compile and must satisfy every rule card. Return only repaired source."
    )
}

/// Builder assembling the surgical repair Rig agent with role defaults wired.
#[derive(::core::fmt::Debug)]
pub struct HolonPatchAgentBuilder<M: ::rig::completion::CompletionModel> {
    model: ::core::option::Option<M>,
    temperature: ::core::option::Option<::core::primitive::f64>,
    max_tokens_override: ::core::option::Option<::core::primitive::u64>,
    target_language: ::std::string::String,
}

impl<M: ::rig::completion::CompletionModel> self::HolonPatchAgentBuilder<M> {
    /// Wire the completion model backing the agent.
    pub fn model(mut self, wired_model: M) -> Self {
        self.model = ::core::option::Option::Some(wired_model);
        self
    }

    /// Override the sampling temperature.
    pub fn temperature(mut self, temperature_value: ::core::primitive::f64) -> Self {
        self.temperature = ::core::option::Option::Some(temperature_value);
        self
    }

    /// Override the completion token budget.
    pub fn max_tokens(mut self, token_budget: ::core::primitive::u64) -> Self {
        self.max_tokens_override = ::core::option::Option::Some(token_budget);
        self
    }

    /// Set the target language the repair preamble advertises.
    pub fn target_language(mut self, language: ::std::string::String) -> Self {
        self.target_language = language;
        self
    }

    /// Build the Rig agent with the repair preamble and token budget wired.
    pub fn build(self) -> ::rig::agent::Agent<M> {
        let wired_model = match self.model {
            ::core::option::Option::Some(found_model) => found_model,
            ::core::option::Option::None => ::core::panic!("patch model must be wired before build"),
        };
        let preamble_text = self::patch_system_prompt(&self.target_language);
        let mut agent_builder = ::rig::agent::AgentBuilder::new(wired_model)
            .preamble(&preamble_text)
            .max_tokens(16384);
        if let ::core::option::Option::Some(temperature_value) = self.temperature {
            agent_builder = agent_builder.temperature(temperature_value);
        }
        if let ::core::option::Option::Some(token_budget) = self.max_tokens_override {
            agent_builder = agent_builder.max_tokens(token_budget);
        }
        agent_builder.build()
    }
}

/// Namespace for constructing the surgical repair agent and invoking it.
pub struct HolonPatchAgent;

impl self::HolonPatchAgent {
    /// Start building the surgical repair Rig agent.
    pub fn builder<M: ::rig::completion::CompletionModel>() -> self::HolonPatchAgentBuilder<M> {
        self::HolonPatchAgentBuilder {
            model: ::core::option::Option::None,
            temperature: ::core::option::Option::None,
            max_tokens_override: ::core::option::Option::None,
            target_language: "rust".to_string(),
        }
    }

    /// Prompt the wired agent and extract the repaired module source.
    pub fn repair_code<M: ::rig::completion::CompletionModel + 'static>(
        agent: &::std::sync::Arc<::rig::agent::Agent<M>>,
        candidate_code: &::core::primitive::str,
        fault: &::holon_blackboard::FaultRecord,
        problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::std::string::String {
        // A candidate that is already a fault sentinel carries no repairable
        // code. Re-prompting would nest sentinels and churn the repair loop,
        // so the existing marker propagates unchanged.
        let candidate_is_sentinel = candidate_code.contains("SYNTAX_ERROR:")
            || candidate_code.contains("compile_error!");
        if candidate_is_sentinel {
            return candidate_code.to_string();
        }
        let user_prompt = self::render_repair_prompt(candidate_code, fault, problem_model);
        let response_text = self::prompt_blocking(agent, user_prompt);
        match self::fault_sentinel_of(&response_text) {
            ::core::option::Option::Some(sentinel_text) => sentinel_text,
            ::core::option::Option::None => self::extract_code(&response_text),
        }
    }
}

/// Render the repair prompt from candidate code, fault record, and problem model.
pub fn render_repair_prompt(
    candidate_code: &::core::primitive::str,
    fault: &::holon_blackboard::FaultRecord,
    problem_model: &::holon_blackboard::ProblemModelSlice,
) -> ::std::string::String {
    let context_block = problem_model.render_prompt_context();
    let fault_summary = &fault.summary;
    let diagnostic_trace = &fault.diagnostic_trace;
    let target_language = &problem_model.target_language;
    ::std::format!(
        "[PROBLEM CONTEXT]\n{context_block}\n\n[CANDIDATE CODE]\n{candidate_code}\n\n[FAULT SUMMARY]\n{fault_summary}\n\n[DIAGNOSTIC TRACE]\n{diagnostic_trace}\n\nReturn the repaired complete {target_language} module now.",
    )
}

/// Run one blocking prompt exchange on a dedicated single-thread runtime.
/// The rig prompt API is async while the ARIADNE controller path is synchronous,
/// so the future is driven to completion on its own worker thread.
fn prompt_blocking<M: ::rig::completion::CompletionModel + 'static>(
    agent: &::std::sync::Arc<::rig::agent::Agent<M>>,
    user_prompt: ::std::string::String,
) -> ::std::string::String {
    let agent_for_call = ::std::sync::Arc::clone(agent);
    let (sender, receiver) = ::std::sync::mpsc::channel::<
        ::core::result::Result<::std::string::String, ::std::string::String>,
    >();
    let handle_result = ::std::thread::Builder::new()
        .name("ariadne-patch-call".to_string())
        .spawn(move || {
            let runtime = ::tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime builds");
            let outcome = runtime.block_on(async {
                ::rig::completion::Prompt::prompt(&*agent_for_call, user_prompt)
                    .await
                    .map_err(|prompt_error| prompt_error.to_string())
            });
            let _ = sender.send(outcome);
        });
    match handle_result {
        ::core::result::Result::Ok(handle) => {
            let _ = handle.join();
        }
        ::core::result::Result::Err(spawn_error) => {
            let failure_message = ::std::format!("repair inference unavailable: {spawn_error}");
            let sentinel = self::fault_candidate(&failure_message);
            return sentinel;
        }
    }
    match receiver.recv() {
        ::core::result::Result::Ok(::core::result::Result::Ok(response_text)) => response_text,
        ::core::result::Result::Ok(::core::result::Result::Err(prompt_error)) => {
            let failure_message = ::std::format!("repair inference failed: {prompt_error}");
            let sentinel = self::fault_candidate(&failure_message);
            sentinel
        }
        ::core::result::Result::Err(_) => {
            let sentinel = self::fault_candidate("repair thread panicked");
            sentinel
        }
    }
}

fn fault_sentinel_of(response_text: &::core::primitive::str) -> ::core::option::Option<::std::string::String> {
    let is_fault = response_text.contains("repair inference unavailable")
        || response_text.contains("repair inference failed")
        || response_text.contains("repair thread panicked");
    if !is_fault {
        return ::core::option::Option::None;
    }
    let sentinel_text = self::fault_candidate(response_text);
    ::core::option::Option::Some(sentinel_text)
}

/// Encode an inference failure as a language-neutral SYNTAX_ERROR sentinel candidate.
pub fn fault_candidate(failure_reason: &::core::primitive::str) -> ::std::string::String {
    // A reason that already embeds a sentinel marker would nest markers on
    // every retry cycle. Collapse to the existing sentinel text instead.
    let reason_is_sentinel = failure_reason.contains("SYNTAX_ERROR:")
        || failure_reason.contains("compile_error!");
    if reason_is_sentinel {
        return failure_reason.to_string();
    }
    let sanitized_reason = failure_reason
        .replace('"', "'")
        .replace('\n', " ")
        .replace(';', "");
    ::std::format!("SYNTAX_ERROR: ariadne llm repair fault: {sanitized_reason}")
}

/// Extract the first fenced Rust code block from a model response.
/// Unfenced responses come back trimmed so bare code still works.
pub fn extract_code(response_text: &::core::primitive::str) -> ::std::string::String {
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
    fn test_extract_code_strips_fences() {
        let sample = "```rust\npub fn repaired_probe() -> u8 { 1 }\n```";
        let extracted = self::extract_code(sample);
        ::std::assert!(extracted.starts_with("pub fn repaired_probe()"));
        ::std::assert!(!extracted.contains("```"));
    }

    #[test]
    fn test_extract_code_bare_response_passthrough() {
        let sample = "pub fn repaired_probe() -> u8 { 1 }";
        let extracted = self::extract_code(sample);
        ::std::assert_eq!(extracted, sample);
    }

    #[test]
    fn test_fault_candidate_sanitizes_and_encodes_marker() {
        let sentinel = self::fault_candidate("boom\nnext \"line\"");
        ::std::assert!(sentinel.contains("SYNTAX_ERROR"));
        ::std::assert!(sentinel.contains("ariadne llm repair fault"));
        ::std::assert!(!sentinel.contains('\n'));
        ::std::assert!(!sentinel.contains('"'));
    }

    #[test]
    fn test_fault_sentinel_detects_transport_errors() {
        let failure_text = "repair inference failed: timeout";
        let sentinel = self::fault_sentinel_of(failure_text);
        ::std::assert!(sentinel.is_some());
        let normal_text = "pub fn repaired_probe() -> u8 { 1 }";
        let no_sentinel = self::fault_sentinel_of(normal_text);
        ::std::assert!(no_sentinel.is_none());
    }

    /// Counting completion model: records every call and answers with a
    /// marker that would corrupt the candidate if a call ever happened.
    #[derive(::core::clone::Clone)]
    struct CountingFakeModel {
        call_count: ::std::sync::Arc<::std::sync::atomic::AtomicU64>,
    }

    impl ::rig::completion::CompletionModel for self::CountingFakeModel {
        type Response = ::std::string::String;

        fn completion(
            &self,
            _request: ::rig::completion::CompletionRequest,
        ) -> impl ::core::future::Future<
            Output = ::core::result::Result<
                ::rig::completion::CompletionResponse<Self::Response>,
                ::rig::completion::CompletionError,
            >,
        > + ::core::marker::Send {
            self.call_count
                .fetch_add(1, ::std::sync::atomic::Ordering::SeqCst);
            let content_item = ::rig::message::AssistantContent::text(
                "SYNTAX_ERROR: ariadne llm repair fault: nested marker would prove inference ran",
            );
            let outcome = ::rig::completion::CompletionResponse {
                choice: ::rig::OneOrMany::one(content_item),
                raw_response: ::std::string::String::from("counting-fake"),
            };
            ::core::future::ready(::core::result::Result::Ok(outcome))
        }
    }

    #[test]
    fn test_repair_code_on_sentinel_candidate_makes_zero_calls() {
        // A candidate that is already a fault marker carries no repairable
        // code. Re-prompting would nest sentinels and churn the repair
        // loop, so repair_code must propagate it without any inference.
        let zero_calls = ::std::sync::atomic::AtomicU64::new(0);
        let call_count = ::std::sync::Arc::new(zero_calls);
        let patch_agent = self::HolonPatchAgent::builder::<self::CountingFakeModel>()
            .model(self::CountingFakeModel {
                call_count: ::std::sync::Arc::clone(&call_count),
            })
            .build();
        let agent = ::std::sync::Arc::new(patch_agent);
        let sentinel_candidate = self::fault_candidate("prior synthesis failure");
        let fault = ::holon_blackboard::FaultRecord {
            summary: ::std::string::String::from("prior failure"),
            diagnostic_trace: ::std::string::String::new(),
            cause: ::core::option::Option::None,
        };
        let problem_model = ::holon_blackboard::ProblemModelSlice::new();
        let outcome = self::HolonPatchAgent::repair_code(
            &agent,
            &sentinel_candidate,
            &fault,
            &problem_model,
        );
        ::std::assert_eq!(outcome, sentinel_candidate);
        let observed = call_count.load(::std::sync::atomic::Ordering::SeqCst);
        ::std::assert_eq!(observed, 0, "sentinel candidate must not trigger inference");
    }
}
