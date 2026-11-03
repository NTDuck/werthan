//! Code generation agent for the ARIADNE cluster.
//! Exposes one Rig agent with the synthesis preamble wired, plus the domain
//! call that prompts it and extracts the candidate module source.

/// Typed output emitted by code synthesis.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
)]
pub struct CodeSynthesisOutput {
    /// Generated module source code.
    pub code: ::std::string::String,
}

/// System prompt steering the model toward compilable module code in the
/// configured target language.
pub fn codegen_system_prompt(target_language: &::core::primitive::str) -> ::std::string::String {
    let separator = ';';
    ::std::format!(
        "You are a {target_language} code synthesis engine for a legacy modernization pipeline. \
Output only a valid JSON object matching {{\"code\": \"<module source code>\"}} without prose or commentary outside the JSON. \
The code field must contain the complete {target_language} source code for one module with no markdown fences. \
The code must define every item the contracts and rule cards require. \
Strictly adhere to the exact function signatures, argument counts, and parameter types defined in [EXPORT OBLIGATIONS]. If an exported function is \
specified with N arguments, you must define it with exactly those N arguments so that integration tests compile. \
All translated functions, types, and constants must be declared pub so they are accessible by external test suites. \
Do NOT wrap the output in 'pub mod <name> {{ ... }}'. Output only top-level items (functions, structs, constants) for this single module - the module is already declared by the crate root. Do NOT output code for other modules. \
In Rust, always annotate public structs and enums with #[derive(Clone, Debug, PartialEq)] (and also add Copy for small attribute, option, and argument structs whose fields are primitives, enums, or function pointers) so callers and test suites can copy, clone, inspect, and assert equivalence without move errors. \
The code must compile on its own and must not reference crates outside ::core and ::std. \
Do not use architecture-specific SIMD intrinsics (such as std::arch){separator} write standard, portable, bounds-safe loops."
    )
}

/// Builder assembling the code generation Rig agent with role defaults wired.
#[derive(::core::fmt::Debug)]
pub struct HolonCodeGenAgentBuilder<M: ::rig::completion::CompletionModel> {
    model: ::core::option::Option<M>,
    temperature: ::core::option::Option<::core::primitive::f64>,
    max_tokens_override: ::core::option::Option<::core::primitive::u64>,
    target_language: ::std::string::String,
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

    /// Set the target language the synthesis preamble advertises.
    pub fn target_language(mut self, language: ::std::string::String) -> Self {
        self.target_language = language;
        self
    }

    /// Build the Rig agent with the synthesis preamble and token budget wired.
    pub fn build(self) -> ::rig::agent::Agent<M> {
        let wired_model = match self.model {
            ::core::option::Option::Some(found_model) => found_model,
            ::core::option::Option::None => ::core::panic!("codegen model must be wired before build"),
        };
        let preamble_text = self::codegen_system_prompt(&self.target_language);
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

/// Namespace for constructing the code generation agent and invoking it.
pub struct HolonCodeGenAgent;

impl self::HolonCodeGenAgent {
    /// Start building the code generation Rig agent.
    pub fn builder<M: ::rig::completion::CompletionModel>() -> self::HolonCodeGenAgentBuilder<M> {
        self::HolonCodeGenAgentBuilder {
            model: ::core::option::Option::None,
            temperature: ::core::option::Option::None,
            max_tokens_override: ::core::option::Option::None,
            target_language: "rust".to_string(),
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
            ::core::option::Option::None => self::parse_synthesis_output(&response_text),
        }
    }
}

/// Render the user prompt from the problem model and hypothesis.
pub fn render_codegen_prompt(
    problem_model: &::holon_blackboard::ProblemModelSlice,
    hypothesis: &::holon_blackboard::StrategyHypothesis,
) -> ::std::string::String {
    let context_block = problem_model.render_prompt_context();
    let target_language = &problem_model.target_language;
    ::std::format!(
        "[PROBLEM CONTEXT]\n{context_block}\n\n[STRATEGY]\n{} - {}\n\nWrite the complete {target_language} implementation of the target module now. Never return source-language code. The output must be valid {target_language}. All translated functions, types, and constants must be declared pub so they are accessible by external test suites. Do NOT wrap the output in `pub mod <name> {{ ... }}`. Output only top-level items (functions, structs, constants) — the module is already declared by the crate root. In Rust, always annotate public structs and enums with #[derive(Clone, Debug, PartialEq)] (and also add Copy for small attribute, option, and argument structs whose fields are primitives, enums, or function pointers) so callers and test suites can copy, clone, inspect, and assert equivalence without move errors.\n\nOutput only a JSON object matching {{\"code\": \"<module source code>\"}} without prose or commentary outside the JSON.",
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
/// encode it as a SYNTAX_ERROR sentinel candidate. The sentinel keeps
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
    ::std::format!("SYNTAX_ERROR: ariadne llm synthesis fault: {sanitized_reason}")
}

/// Compatibility alias for parse_synthesis_output.
pub fn extract_code(response_text: &::core::primitive::str) -> ::std::string::String {
    self::parse_synthesis_output(response_text)
}

/// Parse the model response into module source code.
///
/// Attempts direct deserialization of `CodeSynthesisOutput`.
/// Slices a JSON object from fences or prose as a fallback.
/// Strips basic code fences from raw code output as a final fallback.
pub fn parse_synthesis_output(response_text: &::core::primitive::str) -> ::std::string::String {
    let trimmed = response_text.trim();
    if let ::core::result::Result::Ok(output) =
        ::serde_json::from_str::<self::CodeSynthesisOutput>(trimmed)
    {
        return output.code;
    }
    if let (::core::option::Option::Some(start_index), ::core::option::Option::Some(end_index)) =
        (trimmed.find('{'), trimmed.rfind('}'))
    {
        if start_index < end_index {
            let json_slice = &trimmed[start_index..=end_index];
            if let ::core::result::Result::Ok(output) =
                ::serde_json::from_str::<self::CodeSynthesisOutput>(json_slice)
            {
                return output.code;
            }
        }
    }
    self::strip_code_fences(trimmed)
}

/// Strip basic markdown code fences from model response text.
fn strip_code_fences(response_text: &::core::primitive::str) -> ::std::string::String {
    let fence_token = "```";
    let trimmed = response_text.trim();
    let fence_start = match trimmed.find(fence_token) {
        ::core::option::Option::Some(found_index) => found_index + fence_token.len(),
        ::core::option::Option::None => return trimmed.to_string(),
    };
    let content_start = match trimmed[fence_start..].find('\n') {
        ::core::option::Option::Some(line_break) => fence_start + line_break + 1,
        ::core::option::Option::None => fence_start,
    };
    let remainder = &trimmed[content_start..];
    let code_body = match remainder.rfind(fence_token) {
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
    fn test_code_synthesis_output_serde_roundtrip() {
        let payload = self::CodeSynthesisOutput {
            code: "pub fn compute() -> u32 { 42 }".to_string(),
        };
        let serialized = ::serde_json::to_string(&payload).expect("serialization succeeds");
        let deserialized = ::serde_json::from_str::<self::CodeSynthesisOutput>(&serialized)
            .expect("deserialization succeeds");
        ::std::assert_eq!(payload, deserialized);
    }

    #[test]
    fn test_parse_synthesis_output_typed_json() {
        let sample = "{\"code\": \"pub fn execute() -> u32 { 100 }\"}";
        let parsed = self::parse_synthesis_output(sample);
        ::std::assert_eq!(parsed, "pub fn execute() -> u32 { 100 }");
    }

    #[test]
    fn test_parse_synthesis_output_fenced_json() {
        let sample = "```json\n{\n    \"code\": \"pub fn run() -> bool { true }\"\n}\n```";
        let parsed = self::parse_synthesis_output(sample);
        ::std::assert_eq!(parsed, "pub fn run() -> bool { true }");
    }

    #[test]
    fn test_parse_synthesis_output_fallback_fenced_code() {
        let sample = "```rust\npub fn fallback_probe() -> u8 { 9 }\n```";
        let parsed = self::parse_synthesis_output(sample);
        ::std::assert_eq!(parsed, "pub fn fallback_probe() -> u8 { 9 }");
    }

    #[test]
    fn test_parse_synthesis_output_fallback_bare_code() {
        let sample = "pub fn bare_probe() -> u8 { 7 }";
        let parsed = self::parse_synthesis_output(sample);
        ::std::assert_eq!(parsed, sample);
    }
    #[test]
    fn test_codegen_system_prompt_instructs_no_pub_mod() {
        let prompt = self::codegen_system_prompt("rust");
        let instruction = "Do NOT wrap the output in 'pub mod <name> { ... }'. Output only top-level items (functions, structs, constants) for this single module - the module is already declared by the crate root. Do NOT output code for other modules.";
        ::std::assert!(prompt.contains(instruction));
    }
    #[test]
    fn test_codegen_system_prompt_instructs_export_obligations_alignment() {
        let prompt = self::codegen_system_prompt("rust");
        let instruction = ::core::concat!(
            "Strictly adhere to the exact function signatures, argument counts, and parameter types defined in [EXPORT OBLIGATIONS]. If an exported function is ",
            "specified with N arguments, you must define it with exactly those N arguments so that integration tests compile.",
        );
        ::std::assert!(prompt.contains(instruction));
    }
    #[test]
    fn test_codegen_system_prompt_instructs_derive() {
        let prompt = self::codegen_system_prompt("rust");
        let instruction = "In Rust, always annotate public structs and enums with #[derive(Clone, Debug, PartialEq)] (and also add Copy for small attribute, option, and argument structs whose fields are primitives, enums, or function pointers) so callers and test suites can copy, clone, inspect, and assert equivalence without move errors.";
        ::std::assert!(prompt.contains(instruction));
    }

    #[test]
    fn test_render_codegen_prompt_instructs_derive() {
        let problem_model = ::holon_blackboard::ProblemModelSlice::new();
        let hypothesis = ::holon_blackboard::StrategyHypothesis {
            strategy_id: "STRAT".to_string(),
            description: "desc".to_string(),
            prior_utility: 0.5,
            instantiated: false,
        };
        let prompt = self::render_codegen_prompt(&problem_model, &hypothesis);
        let instruction = "In Rust, always annotate public structs and enums with #[derive(Clone, Debug, PartialEq)] (and also add Copy for small attribute, option, and argument structs whose fields are primitives, enums, or function pointers) so callers and test suites can copy, clone, inspect, and assert equivalence without move errors.";
        ::std::assert!(prompt.contains(instruction));
    }
    #[test]
    fn test_codegen_system_prompt_instructs_json_schema() {
        let prompt = self::codegen_system_prompt("rust");
        let instruction = "Output only a valid JSON object matching {\"code\": \"<module source code>\"} without prose or commentary outside the JSON.";
        ::std::assert!(prompt.contains(instruction));
    }

    #[test]
    fn test_render_codegen_prompt_instructs_json_schema() {
        let problem_model = ::holon_blackboard::ProblemModelSlice::new();
        let hypothesis = ::holon_blackboard::StrategyHypothesis {
            strategy_id: "STRAT".to_string(),
            description: "desc".to_string(),
            prior_utility: 0.5,
            instantiated: false,
        };
        let prompt = self::render_codegen_prompt(&problem_model, &hypothesis);
        let instruction = "Output only a JSON object matching {\"code\": \"<module source code>\"} without prose or commentary outside the JSON.";
        ::std::assert!(prompt.contains(instruction));
    }

    #[test]
    fn test_fault_candidate_sanitizes_and_encodes_marker() {
        let sentinel = self::fault_candidate("boom\nwith \"quotes\"");
        ::std::assert!(sentinel.contains("SYNTAX_ERROR"));
        ::std::assert!(sentinel.contains("ariadne llm synthesis fault"));
        ::std::assert!(!sentinel.contains('\n'));
        ::std::assert!(!sentinel.contains('"'));
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

    #[test]
    fn test_codegen_system_prompt_portable_simd() {
        let prompt = self::codegen_system_prompt("rust");
        let arch_needle = "std::arch";
        let loop_needle = "bounds-safe loops";
        ::std::assert!(prompt.contains(arch_needle));
        ::std::assert!(prompt.contains(loop_needle));
    }

}
