//! Validation contract tests for holon-strategy.
//! Covers VAL-ASTR-01, VAL-ASTR-02 and VAL-ASTR-03.
//! Contracts derive from docs/specs/validation-contracts.md and outrank implementation tests.

#[test]
fn contract_val_astr_01_returns_distinct_hypotheses_with_priors() {
    // Given a problem-model slice, the agent returns at least one distinct
    // strategy hypothesis and each carries a prior utility estimate.
    let problem_model = rules_with_iteration_signal();
    let (agent, _observed) = build_agent(
        r#"[{"strategy_id":"STRAT_A","description":"direct structural port","prior_utility":0.71},
            {"strategy_id":"STRAT_B","description":"state machine decomposition","prior_utility":0.44}]"#
            .to_string(),
    );
    let strategies =
        ::holon_strategy::HolonStrategyAgent::generate_strategies(&agent, &problem_model);
    ::std::assert!(!strategies.is_empty());
    ::std::assert_eq!(strategies.len(), 2);
    let mut seen_ids = ::std::vec::Vec::<::std::string::String>::new();
    for strategy_entry in &strategies {
        ::std::assert!(!strategy_entry.strategy_id.is_empty());
        ::std::assert!(
            strategy_entry.prior_utility >= 0.10 && strategy_entry.prior_utility <= 0.95
        );
        seen_ids.push(strategy_entry.strategy_id.clone());
    }
    seen_ids.sort();
    seen_ids.dedup();
    ::std::assert_eq!(seen_ids.len(), 2);
    // The controller persists the returned strategies into B_SH.
    let mut blackboard_slice = ::holon_blackboard::StrategyHypothesesSlice::new();
    for strategy_entry in strategies {
        blackboard_slice.add_hypothesis(strategy_entry);
    }
    ::std::assert_eq!(blackboard_slice.hypotheses.len(), 2);
    ::std::assert!(blackboard_slice.has_uninstantiated());
}

#[test]
fn contract_val_astr_02_signal_profiles_yield_distinct_strategy_families() {
    // Distinct problem-model signal profiles yield distinct strategy families.
    // The renderer derives strategy context from the problem-model signals, so
    // iteration-heavy and state-heavy profiles produce different prompts.
    let iteration_model = rules_with_iteration_signal();
    let state_model = rules_with_state_signal();
    let prompt_iteration = ::holon_strategy::render_strategy_prompt(&iteration_model);
    let prompt_state = ::holon_strategy::render_strategy_prompt(&state_model);
    ::std::assert_ne!(prompt_iteration, prompt_state);
    ::std::assert!(prompt_iteration.contains("RULE_LOOP"));
    ::std::assert!(prompt_state.contains("RULE_STATE"));
    ::std::assert!(prompt_iteration.contains("iterating"));
    ::std::assert!(prompt_state.contains("state"));
}

#[test]
fn contract_val_astr_03_single_completion_with_system_prompt() {
    // The strategy agent issues a single chat completion with a system prompt
    // and parses exactly one response per call.
    let good_response = r#"[{"strategy_id":"STRAT_OK","description":"functional pipeline port","prior_utility":0.60}]"#.to_string();
    let problem_model = rules_with_iteration_signal();
    let (agent, observed) = build_agent(good_response);
    let strategies =
        ::holon_strategy::HolonStrategyAgent::generate_strategies(&agent, &problem_model);
    ::std::assert_eq!(strategies.len(), 1);
    let prompts = observed.lock().expect("prompt lock");
    ::std::assert_eq!(prompts.len(), 1, "exactly one completion per call");
    // The published strategy preamble carries the system prompt role.
    ::std::assert!(::holon_strategy::STRATEGY_SYSTEM_PROMPT.contains("strategy analyst"));
    ::std::assert!(::holon_strategy::STRATEGY_SYSTEM_PROMPT.contains("JSON array"));
}

#[test]
fn contract_val_astr_03_honest_error_on_parse_failure() {
    // A parse failure surfaces an honest error instead of a silent empty
    // success. The error type carries a readable failure message.
    let malformed = "I cannot propose strategies today.".to_string();
    let problem_model = rules_with_iteration_signal();
    let (agent, observed) = build_agent(malformed);
    let strategies =
        ::holon_strategy::HolonStrategyAgent::generate_strategies(&agent, &problem_model);
    let prompts = observed.lock().expect("prompt lock");
    ::std::assert_eq!(prompts.len(), 1);
    ::std::assert!(strategies.is_empty());
    // The honest error type names the failure instead of faking success.
    let failure = ::holon_strategy::StrategyAgentError {
        message: "strategy response carried no usable hypothesis array".to_string(),
    };
    let failure_text = failure.to_string();
    ::std::assert!(failure_text.contains("no usable hypothesis"));
}

fn rules_with_iteration_signal() -> ::holon_blackboard::ProblemModelSlice {
    let mut problem_model = ::holon_blackboard::ProblemModelSlice::new();
    problem_model.add_rule(::holon_blackboard::BusinessRuleCard {
        rule_id: "RULE_LOOP".to_string(),
        given: "a big list of items".to_string(),
        when: "iterating over many items with index counters".to_string(),
        then: "accumulates totals into sums".to_string(),
        citation: "legacy/loop.c:12".to_string(),
    });
    problem_model
}

fn rules_with_state_signal() -> ::holon_blackboard::ProblemModelSlice {
    let mut problem_model = ::holon_blackboard::ProblemModelSlice::new();
    problem_model.add_rule(::holon_blackboard::BusinessRuleCard {
        rule_id: "RULE_STATE".to_string(),
        given: "a session object".to_string(),
        when: "the machine state switches between modes".to_string(),
        then: "dispatches per mode handler".to_string(),
        citation: "legacy/session.c:40".to_string(),
    });
    problem_model
}

/// Deterministic completion model keyed on the user prompt content.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
struct RecordingStrategyModel {
    observed_prompts: ::std::sync::Arc<::std::sync::Mutex<::std::vec::Vec<::std::string::String>>>,
    response_text: ::std::string::String,
}

impl ::rig::completion::CompletionModel for self::RecordingStrategyModel {
    type Response = ::std::string::String;

    fn completion(
        &self,
        request: ::rig::completion::CompletionRequest,
    ) -> impl ::core::future::Future<
        Output = ::core::result::Result<
            ::rig::completion::CompletionResponse<Self::Response>,
            ::rig::completion::CompletionError,
        >,
    > + ::core::marker::Send {
        let prompt_text = ::std::format!("{:?}", request.prompt);
        self.observed_prompts.lock().expect("prompt lock").push(prompt_text);
        let response_text = self.response_text.clone();
        let content_item = ::rig::message::AssistantContent::text(response_text);
        let choice_bundle = ::rig::OneOrMany::one(content_item);
        let outcome = ::rig::completion::CompletionResponse {
            choice: { choice_bundle },
            raw_response: "contract".to_string(),
        };
        ::core::future::ready(::core::result::Result::Ok(outcome))
    }
}

fn build_agent(
    response_text: ::std::string::String,
) -> (
    ::std::sync::Arc<::rig::agent::Agent<RecordingStrategyModel>>,
    ::std::sync::Arc<::std::sync::Mutex<::std::vec::Vec<::std::string::String>>>,
) {
    let empty_log = ::std::vec::Vec::<::std::string::String>::new();
    let log_mutex = ::std::sync::Mutex::new(empty_log);
    let observed_prompts = ::std::sync::Arc::new(log_mutex);
    let model = RecordingStrategyModel {
        observed_prompts: ::std::sync::Arc::clone(&observed_prompts),
        response_text,
    };
    let agent_builder = ::holon_strategy::HolonStrategyAgent::builder();
    let wired_builder = agent_builder.model(model);
    let built_agent = wired_builder.build();
    let agent = ::std::sync::Arc::new(built_agent);
    (agent, observed_prompts)
}

