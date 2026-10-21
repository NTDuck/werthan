//! Code generation agent for the ARIADNE cluster.
//! Exposes one Rig agent with the synthesis preamble wired, plus the domain
//! call that prompts it and extracts the candidate module source.

/// System prompt steering the model toward compilable module code.
pub const CODEGEN_SYSTEM_PROMPT: &::core::primitive::str =
    "You are a Rust code synthesis engine for a legacy modernization pipeline. \
Output only Rust source code for one module with no markdown and no commentary. \
The code must define every item the contracts and rule cards require. \
The code must compile on its own and must not reference crates outside ::core and ::std.";

/// Builder assembling the code generation Rig agent with role defaults wired.
#[derive(::core::fmt::Debug)]
pub struct HolonCodeGenAgentBuilder<M: ::rig::completion::CompletionModel> {
    model: ::core::option::Option<M>,
    temperature: ::core::option::Option<::core::primitive::f64>,
    max_tokens_override: ::core::option::Option<::core::primitive::u64>,
}

impl<M: ::rig::completion::CompletionModel> self::HolonCodeGenAgentBuilder<M> {
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

    /// Build the Rig agent with the synthesis preamble and token budget wired.
    pub fn build(self) -> ::rig::agent::Agent<M> {
        let wired_model = match self.model {
            ::core::option::Option::Some(found_model) => found_model,
            ::core::option::Option::None => ::core::panic!("codegen model must be wired before build"),
        };
        let mut agent_builder = ::rig::agent::AgentBuilder::new(wired_model)
            .preamble(self::CODEGEN_SYSTEM_PROMPT)
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

/// Namespace for constructing the code generation agent and invoking it.
pub struct HolonCodeGenAgent;

impl self::HolonCodeGenAgent {
    /// Start building the code generation Rig agent.
    pub fn builder<M: ::rig::completion::CompletionModel>() -> self::HolonCodeGenAgentBuilder<M> {
        self::HolonCodeGenAgentBuilder {
            model: ::core::option::Option::None,
            temperature: ::core::option::Option::None,
            max_tokens_override: ::core::option::Option::None,
        }
    }

    /// Prompt the wired agent and extract the candidate module source.
    pub fn generate_candidate<M: ::rig::completion::CompletionModel + 'static>(
        agent: &::std::sync::Arc<::rig::agent::Agent<M>>,
        problem_model: &::holon_blackboard::ProblemModelSlice,
        hypothesis: &::holon_blackboard::StrategyHypothesis,
    ) -> ::std::string::String {
        let user_prompt = self::render_codegen_prompt(problem_model, hypothesis);
        let response_text = self::prompt_blocking(agent, user_prompt);
        match self::fault_sentinel_of(&response_text) {
            ::core::option::Option::Some(sentinel_text) => sentinel_text,
            ::core::option::Option::None => self::extract_code(&response_text),
        }
    }
}

/// Render the user prompt from the problem model and hypothesis.
fn render_codegen_prompt(
    problem_model: &::holon_blackboard::ProblemModelSlice,
    hypothesis: &::holon_blackboard::StrategyHypothesis,
) -> ::std::string::String {
    let context_block = problem_model.render_prompt_context();
    ::std::format!(
        "[PROBLEM CONTEXT]\n{context_block}\n\n[STRATEGY]\n{} - {}\n\nWrite the complete Rust implementation of the target module now.",
        hypothesis.strategy_id,
        hypothesis.description,
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
        .name("ariadne-codegen-call".to_string())
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
            let failure_message = ::std::format!("codegen inference unavailable: {spawn_error}");
            let sentinel = self::fault_candidate(&failure_message);
            return sentinel;
        }
    }
    match receiver.recv() {
        ::core::result::Result::Ok(::core::result::Result::Ok(response_text)) => response_text,
        ::core::result::Result::Ok(::core::result::Result::Err(prompt_error)) => {
            let failure_message = ::std::format!("codegen inference failed: {prompt_error}");
            let sentinel = self::fault_candidate(&failure_message);
            sentinel
        }
        ::core::result::Result::Err(_) => {
            let sentinel = self::fault_candidate("codegen thread panicked");
            sentinel
        }
    }
}
/// Detect an inference failure carried by the transport error text and
/// encode it as a compile_error sentinel candidate. The sentinel keeps
/// failure propagation honest through the evaluator.
fn fault_sentinel_of(response_text: &::core::primitive::str) -> ::core::option::Option<::std::string::String> {
    let fault_marker = "strategy inference";
    let repair_marker = "repair inference";
    let is_fault = response_text.contains("inference unavailable")
        || response_text.contains("inference failed")
        || response_text.contains("thread panicked")
        || response_text.contains(fault_marker)
        || response_text.contains(repair_marker);
    if !is_fault {
        return ::core::option::Option::None;
    }
    let sentinel_text = self::fault_candidate(response_text);
    ::core::option::Option::Some(sentinel_text)
}

/// Encode an inference failure as a compile_error sentinel candidate.
pub fn fault_candidate(failure_reason: &::core::primitive::str) -> ::std::string::String {
    let sanitized_reason = failure_reason
        .replace('"', "'")
        .replace('\n', " ")
        .replace(';', "");
    ::std::format!(
        "::core::compile_error!(\"ariadne llm synthesis fault: {sanitized_reason}\")"
    )
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
        let sample = "Here is the code:\n```rust\npub fn demo() -> u32 {\n    7\n}\n```\nDone.";
        let extracted = self::extract_code(sample);
        ::std::assert!(extracted.starts_with("pub fn demo()"));
        ::std::assert!(extracted.ends_with('}'));
        ::std::assert!(!extracted.contains("```"));
    }

    #[test]
    fn test_extract_code_bare_response_passthrough() {
        let sample = "pub fn demo() -> u32 {\n    7\n}";
        let extracted = self::extract_code(sample);
        ::std::assert_eq!(extracted, sample);
    }

    #[test]
    fn test_fault_candidate_sanitizes_and_compiles_as_sentinel() {
        let sentinel = self::fault_candidate("boom\nwith \"quotes\"");
        ::std::assert!(sentinel.contains("compile_error!"));
        let token_outcome = sentinel.parse::<::proc_macro2::TokenStream>();
        ::std::assert!(token_outcome.is_ok());
    }

    #[test]
    fn test_fault_sentinel_detects_transport_errors() {
        let failure_text = "codegen inference failed: timeout";
        let sentinel = self::fault_sentinel_of(failure_text);
        ::std::assert!(sentinel.is_some());
        let normal_text = "pub fn demo() -> u32 { 7 }";
        let no_sentinel = self::fault_sentinel_of(normal_text);
        ::std::assert!(no_sentinel.is_none());
    }
}
