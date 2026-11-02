//! Validation contract tests for holon-blackboard.
//! Covers VAL-BB-01, VAL-BB-02, VAL-BB-03 and VAL-BB-04.
//! Contracts derive from docs/specs/validation-contracts.md and outrank implementation tests.

#[test]
fn contract_val_bb_01_four_typed_slices() {
    // The local blackboard exposes exactly: problem model B_PM, strategy
    // hypotheses B_SH, characterization tests B_CT and patch/fault records B_PR.
    // Each slice is independently readable and writable.
    let mut board = ::holon_blackboard::LocalBlackboard::new();
    ::std::assert!(board.problem_model.rules.is_empty());
    ::std::assert!(board.strategy_hypotheses.hypotheses.is_empty());
    ::std::assert!(board.characterization_tests.test_cases.is_empty());
    ::std::assert!(board.patch_records.records.is_empty());
    // Independent writes to each slice.
    let rule_card = ::holon_blackboard::BusinessRuleCard {
        rule_id: "RULE_B01".to_string(),
        given: "valid payload".to_string(),
        when: "parser invoked".to_string(),
        then: "syntax tree returned".to_string(),
        citation: "legacy/parser.c:10".to_string(),
    };
    board.problem_model.add_rule(rule_card);
    board.strategy_hypotheses.add_hypothesis(::holon_blackboard::StrategyHypothesis {
        strategy_id: "STRAT_B01".to_string(),
        description: "direct structural port".to_string(),
        prior_utility: 0.7,
        instantiated: false,
    });
    board.characterization_tests.add_test_case("fn case_one() {}".to_string());
    board.patch_records.record_fault(::holon_blackboard::FaultRecord {
        summary: "type mismatch".to_string(),
        diagnostic_trace: "expected u32 found i32".to_string(),
        cause: ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic),
    });
    // Each slice is independently readable and unaffected by writes to the others.
    ::std::assert_eq!(board.problem_model.rule_count(), 1);
    ::std::assert_eq!(board.strategy_hypotheses.hypotheses.len(), 1);
    ::std::assert_eq!(board.characterization_tests.test_count(), 1);
    ::std::assert_eq!(board.patch_records.fault_count(), 1);
}

#[test]
fn contract_val_bb_02_hydration_copies_rules_contracts_obligations() {
    // Hydrating a module context copies rules, parent contracts, and downstream
    // export obligations into B_PM. B_SH, B_CT and B_PR start empty on INITIAL hydration.
    let mut problem_model = ::holon_blackboard::ProblemModelSlice::new();
    problem_model.add_rule(::holon_blackboard::BusinessRuleCard {
        rule_id: "RULE_H01".to_string(),
        given: "empty input".to_string(),
        when: "parse invoked".to_string(),
        then: "error surfaced".to_string(),
        citation: "legacy/lexer.c:77".to_string(),
    });
    problem_model.add_parent_contract(::holon_blackboard::EdgeContract {
        parent_id: "vertex_alpha".to_string(),
        interface_spec: "pub fn lex(input: &str) -> Vec<Token>".to_string(),
        preconditions: ::std::vec::Vec::new(),
    });
    problem_model
        .export_obligations
        .push("must export lex and parse".to_string());
    let board = ::holon_blackboard::LocalBlackboard {
        module_id: "vertex_beta".to_string(),
        problem_model,
        strategy_hypotheses: ::holon_blackboard::StrategyHypothesesSlice::new(),
        characterization_tests: ::holon_blackboard::CharacterizationTestsSlice::new(),
        patch_records: ::holon_blackboard::PatchRecordsSlice::new(),
    };
    // Hydrated problem model mirrors the global context exactly.
    ::std::assert!(board.is_hydrated());
    ::std::assert_eq!(board.problem_model.rule_count(), 1);
    ::std::assert_eq!(board.problem_model.parent_contract_count(), 1);
    ::std::assert_eq!(board.problem_model.export_obligations.len(), 1);
    // Remaining slices start empty on INITIAL hydration.
    ::std::assert!(board.strategy_hypotheses.hypotheses.is_empty());
    ::std::assert!(!board.characterization_tests.is_populated());
    ::std::assert_eq!(board.patch_records.fault_count(), 0);
}

#[test]
fn contract_val_bb_03_warm_start_preservation_on_delta_append() {
    // Appending a delta constraint to B_PM must not clear or reset B_SH, B_CT,
    // B_PR, or any other retained slice content.
    let mut board = ::holon_blackboard::LocalBlackboard::new();
    board.strategy_hypotheses.add_hypothesis(::holon_blackboard::StrategyHypothesis {
        strategy_id: "STRAT_KEEP".to_string(),
        description: "retained hypothesis".to_string(),
        prior_utility: 0.6,
        instantiated: true,
    });
    board.characterization_tests.add_test_case("fn retained_case() {}".to_string());
    board.patch_records.record_fault(::holon_blackboard::FaultRecord {
        summary: "rollout fault".to_string(),
        diagnostic_trace: "trace kept".to_string(),
        cause: ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic),
    });
    board.problem_model.add_delta_constraint("DELTA_NEW_CONTRACT".to_string());
    // The appended delta is present. Everything else stays untouched.
    ::std::assert!(board.problem_model.delta_constraints.contains(&"DELTA_NEW_CONTRACT".to_string()));
    ::std::assert_eq!(board.strategy_hypotheses.hypotheses.len(), 1);
    ::std::assert_eq!(
        board.strategy_hypotheses.hypotheses[0].strategy_id,
        "STRAT_KEEP".to_string()
    );
    ::std::assert_eq!(board.characterization_tests.test_count(), 1);
    ::std::assert_eq!(board.patch_records.fault_count(), 1);
}

#[test]
fn contract_val_bb_04_per_vertex_isolation() {
    // Two local blackboards for distinct vertices share no mutable state.
    // Writes to one stay invisible to the other.
    let mut board_first = ::holon_blackboard::LocalBlackboard::new();
    board_first.module_id = "vertex_one".to_string();
    let mut board_second = ::holon_blackboard::LocalBlackboard::new();
    board_second.module_id = "vertex_two".to_string();
    board_first.problem_model.add_rule(::holon_blackboard::BusinessRuleCard {
        rule_id: "RULE_ISOLATED".to_string(),
        given: "given clause".to_string(),
        when: "when clause".to_string(),
        then: "then clause".to_string(),
        citation: "legacy/one.c:1".to_string(),
    });
    board_first.characterization_tests.add_test_case("fn isolated_case() {}".to_string());
    let empty_fault = ::holon_blackboard::FaultRecord::default();
    board_first.patch_records.record_fault(empty_fault);
    board_first.strategy_hypotheses.add_hypothesis(::holon_blackboard::StrategyHypothesis {
        strategy_id: "STRAT_ONE".to_string(),
        description: "first vertex strategy".to_string(),
        prior_utility: 0.5,
        instantiated: false,
    });
    // The second vertex blackboard sees none of the writes from the first vertex.
    ::std::assert_eq!(board_second.problem_model.rule_count(), 0);
    ::std::assert!(!board_second.characterization_tests.is_populated());
    ::std::assert_eq!(board_second.patch_records.fault_count(), 0);
    ::std::assert!(board_second.strategy_hypotheses.hypotheses.is_empty());
    ::std::assert_ne!(board_first.module_id, board_second.module_id);
}
