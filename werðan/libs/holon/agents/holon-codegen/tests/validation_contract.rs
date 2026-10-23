//! Validation contract tests for holon-codegen.
//! Covers VAL-AGEN-01, VAL-AGEN-02 and VAL-AGEN-03.
//! Contracts derive from docs/specs/validation-contracts.md and outrank implementation tests.

#[test]
fn contract_val_agen_01_complete_candidate_from_hypothesis() {
    // Given problem model and one hypothesis, the agent produces a complete
    // candidate module source string.
    let problem_model = ::holon_blackboard::ProblemModelSlice::new();
    let hypothesis = ::holon_blackboard::StrategyHypothesis {
        strategy_id: "STRAT_PORT".to_string(),
        description: "direct structural port".to_string(),
        prior_utility: 0.70,
        instantiated: false,
    };
    let response_text =
        "pub fn execute_module() -> u32 {\n    42\n}".to_string();
    let agent = build_codegen_agent(response_text);
    let candidate = ::holon_codegen::HolonCodeGenAgent::generate_candidate(
        &agent,
        &problem_model,
        &hypothesis,
    );
    ::std::assert!(candidate.contains("pub fn execute_module"));
    ::std::assert!(candidate.contains("42"));
    ::std::assert!(!candidate.trim().is_empty());
    ::std::assert!(!candidate.contains("```"));
}

#[test]
fn contract_val_agen_02_honest_failure_sentinel() {
    // On inference failure the agent returns a compile_error sentinel
    // candidate, so the failure stays visible to the evaluator.
    let problem_model = ::holon_blackboard::ProblemModelSlice::new();
    let hypothesis = ::holon_blackboard::StrategyHypothesis {
        strategy_id: "STRAT_FAIL".to_string(),
        description: "failing transport".to_string(),
        prior_utility: 0.50,
        instantiated: false,
    };
    let agent = build_failing_codegen_agent();
    let candidate = ::holon_codegen::HolonCodeGenAgent::generate_candidate(
        &agent,
        &problem_model,
        &hypothesis,
    );
    ::std::assert!(candidate.contains("compile_error!"));
    ::std::assert!(!candidate.contains("pub fn"));
}

#[test]
fn contract_val_agen_03_prompt_carries_rules_contracts_and_strategy() {
    // The codegen prompt includes the problem-model rules and contracts
    // relevant to the hypothesis. Rule cards and parent contracts referenced
    // in the slice appear in the rendered prompt.
    let mut problem_model = ::holon_blackboard::ProblemModelSlice::new();
    problem_model.add_rule(::holon_blackboard::BusinessRuleCard {
        rule_id: "RULE_AG_01".to_string(),
        given: "valid payload".to_string(),
        when: "decoder invoked".to_string(),
        then: "payload decoded".to_string(),
        citation: "legacy/decoder.c:9".to_string(),
    });
    problem_model.add_parent_contract(::holon_blackboard::EdgeContract {
        parent_id: "vertex_parent_ag".to_string(),
        interface_spec: "pub fn decode(input: &str) -> Frame".to_string(),
        preconditions: ::std::vec!["input stays utf8".to_string()],
    });
    let hypothesis = ::holon_blackboard::StrategyHypothesis {
        strategy_id: "STRAT_PROMPT".to_string(),
        description: "decoder port with contract fidelity".to_string(),
        prior_utility: 0.65,
        instantiated: false,
    };
    let prompt = ::holon_codegen::render_codegen_prompt(&problem_model, &hypothesis);
    ::std::assert!(prompt.contains("RULE_AG_01"));
    ::std::assert!(prompt.contains("vertex_parent_ag"));
    ::std::assert!(prompt.contains("pub fn decode(input: &str) -> Frame"));
    ::std::assert!(prompt.contains("STRAT_PROMPT"));
    ::std::assert!(prompt.contains("decoder port with contract fidelity"));
}

/// Deterministic completion model returning one fixed response.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
struct RecordingCodegenModel {
    response_outcome: ::core::result::Result<::std::string::String, ::std::string::String>,
}

impl ::rig::completion::CompletionModel for self::RecordingCodegenModel {
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
        let mapped = self
            .response_outcome
            .clone()
            .map_err(::rig::completion::CompletionError::ProviderError);
        let response = mapped.map(|raw_text| {
            let content_item = ::rig::message::AssistantContent::text(raw_text);
            let choice_bundle = ::rig::OneOrMany::one(content_item);
            ::rig::completion::CompletionResponse {
                choice: { choice_bundle },
                raw_response: "contract".to_string(),
            }
        });
        ::core::future::ready(response)
    }
}

fn build_codegen_agent(
    response_text: ::std::string::String,
) -> ::std::sync::Arc<::rig::agent::Agent<RecordingCodegenModel>> {
    let model = RecordingCodegenModel {
        response_outcome: ::core::result::Result::Ok(response_text),
    };
    let agent_builder = ::holon_codegen::HolonCodeGenAgent::builder();
    let wired_builder = agent_builder.model(model);
    ::std::sync::Arc::new(wired_builder.build())
}

fn build_failing_codegen_agent() -> ::std::sync::Arc<::rig::agent::Agent<RecordingCodegenModel>> {
    let model = RecordingCodegenModel {
        response_outcome: ::core::result::Result::Err("transport down".to_string()),
    };
    let agent_builder = ::holon_codegen::HolonCodeGenAgent::builder();
    let wired_builder = agent_builder.model(model);
    ::std::sync::Arc::new(wired_builder.build())
}
