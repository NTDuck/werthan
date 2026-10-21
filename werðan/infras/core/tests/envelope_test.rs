//! Unit tests for TaskActivationEnvelope and ContextAssembler.

#[test]
fn test_envelope_hydration_and_tool_isolation() {
    let mut parent_contracts = ::std::collections::BTreeMap::new();
    let parent_spec = "pub trait ParentContract { fn execute(&self) }".to_string();
    parent_contracts.insert("v_parent".to_string(), parent_spec);

    let export_obs = ::std::vec!["target_process".to_string()];
    let rules = ::std::vec!["RULE-001: Given X when Y then Z".to_string()];
    let snippets = ::std::vec!["void process() { /* legacy */ }".to_string()];

    let slice = ::protocol::GraphSlice::builder()
        .target_ref("v_target")
        .parent_contracts(parent_contracts)
        .export_obligations(export_obs)
        .rule_cards(rules)
        .legacy_snippets(snippets)
        .build();

    let diag = ::std::vec!["error[E0308]: mismatched types".to_string()];
    let directive = ::protocol::DirectiveTrace::builder()
        .failure_critique("Previous candidate failed edge case 0".to_string())
        .compiler_diagnostics(diag)
        .build();

    let test_env = ::protocol::ContextAssembler::hydrate(
        ::protocol::AgentRole::AriadneTester,
        ::protocol::ActivationMode::Initial,
        slice.clone(),
        directive.clone(),
    );

    ::std::assert_eq!(test_env.role, ::protocol::AgentRole::AriadneTester);
    ::std::assert_eq!(test_env.mode, ::protocol::ActivationMode::Initial);
    ::std::assert_eq!(test_env.thinking_budget, ::protocol::ThinkingBudget::Standard);
    let find_refs = "lsp_find_references".to_string();
    ::std::assert!(test_env.tools.allowed_tools.contains(&find_refs));
    let apply_patch = "apply_patch".to_string();
    ::std::assert!(!test_env.tools.allowed_tools.contains(&apply_patch));

    let strat_env = ::protocol::ContextAssembler::hydrate(
        ::protocol::AgentRole::AriadneStrategy,
        ::protocol::ActivationMode::Initial,
        slice.clone(),
        directive.clone(),
    );
    ::std::assert_eq!(strat_env.thinking_budget, ::protocol::ThinkingBudget::High);

    let retry_env = ::protocol::ContextAssembler::hydrate(
        ::protocol::AgentRole::AriadnePatcher,
        ::protocol::ActivationMode::CriticRetry,
        slice.clone(),
        directive,
    );
    ::std::assert_eq!(retry_env.thinking_budget, ::protocol::ThinkingBudget::High);
    ::std::assert!(retry_env.tools.allowed_tools.contains(&apply_patch));
}

#[test]
fn test_obligation_candidate_status_transitions() {
    let mut candidate = ::protocol::ObligationCandidate::builder()
        .name("calculate_checksum")
        .kind("function")
        .address_hex("0x004012A0")
        .binary_path("liblegacy.so")
        .status(::protocol::ObligationStatus::Discovered)
        .build();

    ::std::assert_eq!(candidate.status, ::protocol::ObligationStatus::Discovered);
    ::std::assert!(candidate.target_implementation.is_none());

    candidate.status = ::protocol::ObligationStatus::Translating;
    candidate.target_implementation = ::core::option::Option::Some("modern_calculate_checksum".to_string());
    candidate.characterization_test = ::core::option::Option::Some("test_checksum_parity".to_string());
    candidate.status = ::protocol::ObligationStatus::Verified;

    ::std::assert_eq!(candidate.status, ::protocol::ObligationStatus::Verified);
    ::std::assert_eq!(
        candidate.target_implementation.as_deref(),
        ::core::option::Option::Some("modern_calculate_checksum")
    );
}

#[test]
fn test_envelope_serde_roundtrip() {
    let empty_contracts = ::std::collections::BTreeMap::new();
    let empty_exports = ::std::vec![];
    let empty_rules = ::std::vec![];
    let empty_snippets = ::std::vec![];

    let slice = ::protocol::GraphSlice::builder()
        .target_ref("v_module_1")
        .parent_contracts(empty_contracts)
        .export_obligations(empty_exports)
        .rule_cards(empty_rules)
        .legacy_snippets(empty_snippets)
        .build();

    let empty_directive = ::protocol::DirectiveTrace::default();
    let env = ::protocol::ContextAssembler::hydrate(
        ::protocol::AgentRole::AriadneGenerator,
        ::protocol::ActivationMode::Initial,
        slice,
        empty_directive,
    );

    let json_res = ::serde_json::to_string(&env);
    ::std::assert!(json_res.is_ok());
    let json = json_res.unwrap();

    let deserialized_res = ::serde_json::from_str::<::protocol::TaskActivationEnvelope>(&json);
    ::std::assert!(deserialized_res.is_ok());
    let deserialized = deserialized_res.unwrap();

    ::std::assert_eq!(deserialized.role, ::protocol::AgentRole::AriadneGenerator);
    ::std::assert_eq!(deserialized.slice.target_ref, "v_module_1");
}
