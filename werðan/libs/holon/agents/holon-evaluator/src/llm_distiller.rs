//! Rig-backed diagnostic distiller for the ARIADNE evaluator.
//! Classifies fault causes from raw logs and audits business rule compliance
//! through Rig agents built via the evaluator agent builders.

use crate::DiagnosticDistiller;

/// System prompt steering the model toward fault cause classification.
pub const DISTILL_SYSTEM_PROMPT: &::core::primitive::str = "You are a fault diagnosis engine for a Rust modernization pipeline. \
Answer with exactly one label from this list and nothing else. \
INTRINSIC means the candidate code itself carries the defect. \
EXTRINSIC_CONTRACT means an upstream parent contract is insufficient. \
EXTRINSIC_CYCLE means an undocumented dependency cycle blocks progress.";

/// System prompt steering the model toward rule compliance scoring.
pub const AUDIT_SYSTEM_PROMPT: &::core::primitive::str = "You are a business rule audit engine for a Rust modernization pipeline. \
Read every rule card and answer with one decimal fraction between 0.0 and 1.0 and nothing else. \
The fraction states how many rules the candidate code satisfies.";

/// Builder assembling one evaluator Rig agent with its role preamble wired.
#[derive(::core::fmt::Debug)]
pub struct HolonEvaluatorAgentBuilder<M: ::rig::completion::CompletionModel> {
    model: ::core::option::Option<M>,
    temperature: ::core::option::Option<::core::primitive::f64>,
    max_tokens_override: ::core::option::Option<::core::primitive::u64>,
    role_preamble: &'static ::core::primitive::str,
    role_default_tokens: ::core::primitive::u64,
}

impl<M: ::rig::completion::CompletionModel> self::HolonEvaluatorAgentBuilder<M> {
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

    /// Build the Rig agent with the role preamble and token budget wired.
    pub fn build(self) -> ::rig::agent::Agent<M> {
        let wired_model = match self.model {
            ::core::option::Option::Some(found_model) => found_model,
            ::core::option::Option::None => ::core::panic!("evaluator model must be wired before build"),
        };
        let mut agent_builder = ::rig::agent::AgentBuilder::new(wired_model)
            .preamble(self.role_preamble)
            .max_tokens(self.role_default_tokens);
        if let ::core::option::Option::Some(temperature_value) = self.temperature {
            agent_builder = agent_builder.temperature(temperature_value);
        }
        if let ::core::option::Option::Some(token_budget) = self.max_tokens_override {
            agent_builder = agent_builder.max_tokens(token_budget);
        }
        agent_builder.build()
    }
}

/// Namespace for constructing the evaluator Rig agents.
/// The evaluator runs two role prompts over one model: a fault classifier
/// and a business rule auditor.
pub struct HolonEvaluatorAgent;

impl self::HolonEvaluatorAgent {
    /// Start building the fault classification agent (512-token label responses).
    pub fn classifier_builder<M: ::rig::completion::CompletionModel>() -> self::HolonEvaluatorAgentBuilder<M> {
        self::HolonEvaluatorAgentBuilder {
            model: ::core::option::Option::None,
            temperature: ::core::option::Option::None,
            max_tokens_override: ::core::option::Option::None,
            role_preamble: self::DISTILL_SYSTEM_PROMPT,
            role_default_tokens: 512,
        }
    }

    /// Start building the rule audit agent (512-token fraction responses).
    pub fn auditor_builder<M: ::rig::completion::CompletionModel>() -> self::HolonEvaluatorAgentBuilder<M> {
        self::HolonEvaluatorAgentBuilder {
            model: ::core::option::Option::None,
            temperature: ::core::option::Option::None,
            max_tokens_override: ::core::option::Option::None,
            role_preamble: self::AUDIT_SYSTEM_PROMPT,
            role_default_tokens: 512,
        }
    }
}

/// Rig-backed implementation of the diagnostic distiller contract.
#[derive(::core::clone::Clone)]
pub struct RigDiagnosticDistiller<M: ::rig::completion::CompletionModel> {
    classifier: ::std::sync::Arc<::rig::agent::Agent<M>>,
    auditor: ::std::sync::Arc<::rig::agent::Agent<M>>,
}

impl<M: ::rig::completion::CompletionModel> ::core::fmt::Debug for self::RigDiagnosticDistiller<M> {
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        formatter.debug_struct("RigDiagnosticDistiller").finish()
    }
}

#[::bon::bon]
impl<M: ::rig::completion::CompletionModel> self::RigDiagnosticDistiller<M> {
    /// Construct the distiller around ready-built classifier and auditor agents.
    #[builder]
    pub fn builder(
        classifier: ::rig::agent::Agent<M>,
        auditor: ::rig::agent::Agent<M>,
    ) -> Self {
        let classifier_arc = ::std::sync::Arc::new(classifier);
        let auditor_arc = ::std::sync::Arc::new(auditor);
        Self {
            classifier: { classifier_arc },
            auditor: { auditor_arc },
        }
    }

    /// Render the classification prompt from raw logs and candidate code.
    fn render_distill_prompt(
        raw_logs: &::core::primitive::str,
        candidate_code: &::core::primitive::str,
    ) -> ::std::string::String {
        ::std::format!(
            "[RAW LOGS]\n{raw_logs}\n\n[CANDIDATE CODE]\n{candidate_code}\n\nClassify the fault cause now.",
        )
    }

    /// Render the rule audit prompt from candidate code and rule cards.
    fn render_audit_prompt(
        candidate_code: &::core::primitive::str,
        problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::std::string::String {
        let context_block = problem_model.render_prompt_context();
        ::std::format!(
            "[RULE CARDS]\n{context_block}\n\n[CANDIDATE CODE]\n{candidate_code}\n\nScore rule compliance now.",
        )
    }

    /// Map a model label onto the fault cause taxonomy.
    /// Unrecognized labels fall back to Intrinsic with the raw label kept in mind.
    fn parse_cause(response_text: &::core::primitive::str) -> ::holon_blackboard::FaultCause {
        let upper_label = response_text.trim().to_uppercase();
        if upper_label.contains("EXTRINSIC_CYCLE") {
            return ::holon_blackboard::FaultCause::ExtrinsicCycle;
        }
        if upper_label.contains("EXTRINSIC_CONTRACT") || upper_label.contains("EXTRINSIC CONTRACT") {
            return ::holon_blackboard::FaultCause::ExtrinsicContract;
        }
        ::holon_blackboard::FaultCause::Intrinsic
    }

    /// Parse a rule compliance fraction from the model response.
    /// Returns 0.0 when the response carries no parseable number.
    fn parse_score(response_text: &::core::primitive::str) -> ::core::primitive::f64 {
        let trimmed_text = response_text.trim();
        if let ::core::result::Result::Ok(parsed_value) = trimmed_text.parse::<::core::primitive::f64>() {
            return parsed_value.clamp(0.0, 1.0);
        }
        let mut scan_chars = trimmed_text.chars().peekable();
        while let ::core::option::Option::Some(first_char) = scan_chars.next() {
            if first_char.is_ascii_digit() {
                let mut number_text = ::std::string::String::new();
                number_text.push(first_char);
                while let ::core::option::Option::Some(next_char) = scan_chars.peek() {
                    if next_char.is_ascii_digit() || *next_char == '.' {
                        number_text.push(*next_char);
                        scan_chars.next();
                    } else {
                        break;
                    }
                }
                if let ::core::result::Result::Ok(parsed_value) = number_text.parse::<::core::primitive::f64>() {
                    return parsed_value.clamp(0.0, 1.0);
                }
            }
        }
        0.0
    }
}

impl<M: ::rig::completion::CompletionModel + 'static> self::DiagnosticDistiller for self::RigDiagnosticDistiller<M> {
    fn distill_failure(
        &self,
        raw_logs: &::core::primitive::str,
        candidate_code: &::core::primitive::str,
        _problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::holon_blackboard::FaultRecord {
        let user_prompt = self::RigDiagnosticDistiller::<M>::render_distill_prompt(raw_logs, candidate_code);
        let classifier_for_call = ::std::sync::Arc::clone(&self.classifier);
        let (sender, receiver) = ::std::sync::mpsc::channel::<
            ::core::result::Result<::std::string::String, ::std::string::String>,
        >();
        let handle_result = ::std::thread::Builder::new()
            .name("ariadne-distill-call".to_string())
            .spawn(move || {
                let runtime = ::tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("runtime builds");
                let outcome = runtime.block_on(async {
                    ::rig::completion::Prompt::prompt(&*classifier_for_call, user_prompt)
                        .await
                        .map_err(|prompt_error| prompt_error.to_string())
                });
                let _ = sender.send(outcome);
            });
        let _ = handle_result.expect("thread spawns").join();
        let completion_outcome = receiver.recv().unwrap_or_else(|_| ::core::result::Result::Err("distill thread panicked".to_string()));
        let (cause, summary) = match completion_outcome {
            ::core::result::Result::Ok(response_text) => {
                let parsed_cause = Self::parse_cause(&response_text);
                let trimmed_label = response_text.trim().to_string();
                (parsed_cause, trimmed_label)
            }
            ::core::result::Result::Err(inference_error) => {
                let failure_note =
                    ::std::format!("llm classification unavailable: {inference_error}");
                // Intrinsic is the conservative default when classification is unavailable.
                (::holon_blackboard::FaultCause::Intrinsic, failure_note)
            }
        };
        ::holon_blackboard::FaultRecord {
            summary,
            diagnostic_trace: { raw_logs.to_string() },
            cause: ::core::option::Option::Some(cause),
        }
    }

    fn audit_rules(
        &self,
        candidate_code: &::core::primitive::str,
        problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::core::primitive::f64 {
        if problem_model.rules.is_empty() {
            return 1.0;
        }
        let user_prompt =
            self::RigDiagnosticDistiller::<M>::render_audit_prompt(candidate_code, problem_model);
        let auditor_for_call = ::std::sync::Arc::clone(&self.auditor);
        let (sender, receiver) = ::std::sync::mpsc::channel::<
            ::core::result::Result<::std::string::String, ::std::string::String>,
        >();
        let handle_result = ::std::thread::Builder::new()
            .name("ariadne-audit-call".to_string())
            .spawn(move || {
                let runtime = ::tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("runtime builds");
                let outcome = runtime.block_on(async {
                    ::rig::completion::Prompt::prompt(&*auditor_for_call, user_prompt)
                        .await
                        .map_err(|prompt_error| prompt_error.to_string())
                });
                let _ = sender.send(outcome);
            });
        let _ = handle_result.expect("thread spawns").join();
        let completion_outcome = receiver.recv().unwrap_or_else(|_| ::core::result::Result::Err("audit thread panicked".to_string()));
        match completion_outcome {
            ::core::result::Result::Ok(response_text) => Self::parse_score(&response_text),
            ::core::result::Result::Err(_) => 0.0,
        }
    }
}

#[cfg(test)]
mod llm_distiller_tests {
    use super::*;

    #[test]
    fn test_parse_cause_maps_extrinsic_cycle() {
        let cause = self::RigDiagnosticDistiller::<::rig::providers::openai::CompletionModel>::parse_cause("EXTRINSIC_CYCLE");
        ::core::assert!(::core::matches!(cause, ::holon_blackboard::FaultCause::ExtrinsicCycle));
    }

    #[test]
    fn test_parse_cause_maps_extrinsic_contract() {
        let cause = self::RigDiagnosticDistiller::<::rig::providers::openai::CompletionModel>::parse_cause(" EXTRINSIC_CONTRACT ");
        ::core::assert!(::core::matches!(cause, ::holon_blackboard::FaultCause::ExtrinsicContract));
    }

    #[test]
    fn test_parse_cause_falls_back_to_intrinsic() {
        let cause = self::RigDiagnosticDistiller::<::rig::providers::openai::CompletionModel>::parse_cause("garbage");
        ::core::assert!(::core::matches!(cause, ::holon_blackboard::FaultCause::Intrinsic));
    }

    #[test]
    fn test_parse_score_plain_fraction() {
        let score = self::RigDiagnosticDistiller::<::rig::providers::openai::CompletionModel>::parse_score("0.75");
        ::core::assert!((score - 0.75).abs() < 0.000_001);
    }

    #[test]
    fn test_parse_score_embedded_fraction() {
        let score = self::RigDiagnosticDistiller::<::rig::providers::openai::CompletionModel>::parse_score("score: 0.5 of rules");
        ::core::assert!((score - 0.5).abs() < 0.000_001);
    }

    #[test]
    fn test_parse_score_without_number_is_zero() {
        let score = self::RigDiagnosticDistiller::<::rig::providers::openai::CompletionModel>::parse_score("no digits here");
        ::core::assert!(score == 0.0);
    }

    #[test]
    fn test_render_distill_prompt_includes_sections() {
        let prompt = self::RigDiagnosticDistiller::<::rig::providers::openai::CompletionModel>::render_distill_prompt("log line", "fn demo() {}");
        ::core::assert!(prompt.contains("[RAW LOGS]"));
        ::core::assert!(prompt.contains("[CANDIDATE CODE]"));
        ::core::assert!(prompt.contains("Classify the fault cause now."));
    }

    #[test]
    fn test_render_audit_prompt_includes_sections() {
        let problem_model = ::holon_blackboard::ProblemModelSlice::new();
        let prompt = self::RigDiagnosticDistiller::<::rig::providers::openai::CompletionModel>::render_audit_prompt("fn demo() {}", &problem_model);
        ::core::assert!(prompt.contains("[RULE CARDS]"));
        ::core::assert!(prompt.contains("[CANDIDATE CODE]"));
        ::core::assert!(prompt.contains("Score rule compliance now."));
    }
}
