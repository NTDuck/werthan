//! Validation contract tests for holon-engine.
//! Covers VAL-M2-01 through VAL-M2-11.
//! Contracts derive from docs/specs/validation-contracts.md and outrank implementation tests.

#[test]
fn contract_val_m2_01_reward_formula_gated() {
    // The gated reward formula reads R = w_comp + I_compile times the
    // weighted parity and harden terms. The default weights sum to one.
    let weights = ::holon_engine::RewardWeights::default();
    let weight_sum = weights.compile + weights.parity + weights.harden;
    ::std::assert!((weight_sum - 1.0).abs() < 1e-9);
    // An uncompilable candidate earns exactly zero from every term.
    let blocked_reward = ::holon_engine::compute_gated_reward(&weights, false, 1.0, 1.0);
    ::std::assert_eq!(blocked_reward, 0.0);
    // A compiling candidate with perfect scores earns unit reward.
    let perfect_reward = ::holon_engine::compute_gated_reward(&weights, true, 1.0, 1.0);
    ::std::assert!((perfect_reward - 1.0).abs() < 1e-9);
    // Half parity and half harden yield w_comp plus the weighted halves.
    let partial_reward = ::holon_engine::compute_gated_reward(&weights, true, 0.5, 0.5);
    let expected_partial = weights.compile + weights.parity * 0.5 + weights.harden * 0.5;
    ::std::assert!((partial_reward - expected_partial).abs() < 1e-9);
    ::std::assert!((partial_reward - 0.6).abs() < 1e-9);
}

#[test]
fn contract_val_m2_02_ucb_selection_and_temperature() {
    // Child selection scores follow UCB1 plus a gamma weighted prior bias.
    let mut tree = ::holon_engine::SearchTree::new(::core::option::Option::None);
    let root_id = tree.root_id();
    let high_id = tree.add_child(
        root_id,
        ::core::option::Option::Some("pub fn high_probe() -> u8 { 1 }".to_string()),
        ::core::option::Option::Some("STRAT_HIGH".to_string()),
        0.9,
    );
    let low_id = tree.add_child(
        root_id,
        ::core::option::Option::Some("pub fn low_probe() -> u8 { 2 }".to_string()),
        ::core::option::Option::Some("STRAT_LOW".to_string()),
        0.1,
    );
    // Unvisited children score at the exploration ceiling ordered by prior.
    let ucb_high = tree.compute_ucb(high_id, root_id, 1.414, 0.15);
    let ucb_low = tree.compute_ucb(low_id, root_id, 1.414, 0.15);
    ::std::assert!((ucb_high - (1000.0 + 0.15 * 0.9)).abs() < 1e-9);
    ::std::assert!(ucb_high > ucb_low);
    // Softmax over UCB scores keeps the argmax across temperature extremes.
    let cold_pick = tree.select_softmax_child(root_id, 1.414, 0.15, 0.001);
    ::std::assert_eq!(cold_pick, ::core::option::Option::Some(high_id));
    let hot_pick = tree.select_softmax_child(root_id, 1.414, 0.15, 10.0);
    ::std::assert_eq!(hot_pick, ::core::option::Option::Some(high_id));
    // Visiting one child exposes the exploration term under c_exp growth.
    tree.backpropagate(high_id, 0.5);
    let base_score = tree.compute_ucb(high_id, root_id, 0.0, 0.15);
    let explored_score = tree.compute_ucb(high_id, root_id, 2.0, 0.15);
    ::std::assert!(explored_score > base_score);
    let explore_term = explored_score - base_score;
    let expected_term = 2.0 * ((2.0_f64).ln() / 1.0).sqrt();
    ::std::assert!((explore_term - expected_term).abs() < 1e-9);
    // Equal visit statistics leave the gamma weighted prior as the only gap.
    tree.backpropagate(low_id, 0.5);
    let visited_high = tree.compute_ucb(high_id, root_id, 0.0, 0.15);
    let visited_low = tree.compute_ucb(low_id, root_id, 0.0, 0.15);
    let prior_gap = visited_high - visited_low;
    ::std::assert!((prior_gap - 0.15 * (0.9 - 0.1)).abs() < 1e-9);
    // A fresh unvisited sibling outranks both exploited children.
    let fresh_id = tree.add_child(
        root_id,
        ::core::option::Option::Some("pub fn fresh_probe() -> u8 { 3 }".to_string()),
        ::core::option::Option::Some("STRAT_FRESH".to_string()),
        0.3,
    );
    let fresh_pick = tree.select_softmax_child(root_id, 1.414, 0.15, 0.001);
    ::std::assert_eq!(fresh_pick, ::core::option::Option::Some(fresh_id));
    // The tree walk descends through visited nodes into the unvisited leaf.
    let expandable = tree.select_expandable(1.414, 0.15, 0.001);
    ::std::assert_eq!(expandable, fresh_id);
}

#[test]
fn contract_val_m2_04_backpropagation_updates_path() {
    // Every rollout backpropagates reward along the path to the root and
    // refreshes visit counts and mean rewards at each ancestor.
    let mut tree = ::holon_engine::SearchTree::new(::core::option::Option::None);
    let root_id = tree.root_id();
    let mid_id = tree.add_child(
        root_id,
        ::core::option::Option::Some("pub fn mid_probe() -> u8 { 1 }".to_string()),
        ::core::option::Option::Some("STRAT_MID".to_string()),
        0.5,
    );
    let leaf_id = tree.add_child(
        mid_id,
        ::core::option::Option::Some("pub fn leaf_probe() -> u8 { 2 }".to_string()),
        ::core::option::Option::Some("STRAT_LEAF".to_string()),
        0.4,
    );
    tree.backpropagate(leaf_id, 0.8);
    let root_after_leaf = tree.get(root_id).expect("root present");
    ::std::assert_eq!(root_after_leaf.visit_count, 1);
    ::std::assert!((root_after_leaf.mean_reward - 0.8).abs() < 1e-9);
    let mid_after_leaf = tree.get(mid_id).expect("mid present");
    ::std::assert_eq!(mid_after_leaf.visit_count, 1);
    ::std::assert!((mid_after_leaf.mean_reward - 0.8).abs() < 1e-9);
    let leaf_after_leaf = tree.get(leaf_id).expect("leaf present");
    ::std::assert_eq!(leaf_after_leaf.visit_count, 1);
    ::std::assert!((leaf_after_leaf.mean_reward - 0.8).abs() < 1e-9);
    tree.backpropagate(mid_id, 0.2);
    let mid_after_second = tree.get(mid_id).expect("mid present");
    ::std::assert_eq!(mid_after_second.visit_count, 2);
    ::std::assert!((mid_after_second.mean_reward - 0.5).abs() < 1e-9);
    let root_after_second = tree.get(root_id).expect("root present");
    ::std::assert_eq!(root_after_second.visit_count, 2);
    ::std::assert!((root_after_second.mean_reward - 0.5).abs() < 1e-9);
    let leaf_after_second = tree.get(leaf_id).expect("leaf present");
    ::std::assert_eq!(leaf_after_second.visit_count, 1);
    ::std::assert!((leaf_after_second.mean_reward - 0.8).abs() < 1e-9);
}

#[test]
fn contract_val_m2_03_state_conditioned_dispatch() {
    // Rollout one on a fresh blackboard dispatches three agents. The
    // strategy agent covers missing hypotheses. The codegen agent covers the
    // empty candidate. The testgen agent covers the empty test coverage.
    let blackboard_a = ::holon_blackboard::LocalBlackboard::new();
    let machine_a = make_machine();
    let config_a = ::holon_engine::EngineConfig {
        max_rollouts: 1,
        ..::core::default::Default::default()
    };
    let codegen_script_a = ::std::vec![
        "pub fn dispatch_probe() -> u8 { 1 } // MARKER_TOKEN_ALPHA".to_string()
    ];
    let (mut engine_a, log_a) = build_engine(
        config_a,
        blackboard_a,
        codegen_script_a,
        ::std::vec![],
        EMPTY_RESPONSE,
        0.0,
        ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic),
    );
    let _ = engine_a.solve(machine_a);
    let strategy_prompts = prompt_calls_for(&log_a, "strategy analyst");
    ::std::assert_eq!(strategy_prompts.len(), 1);
    let codegen_prompts = prompt_calls_for(&log_a, "code synthesis engine");
    ::std::assert_eq!(codegen_prompts.len(), 1);
    ::std::assert!(codegen_prompts[0].contains("STRAT_DISPATCH_01"));
    let testgen_prompts = prompt_calls_for(&log_a, "test generator");
    ::std::assert_eq!(testgen_prompts.len(), 1);
    // The test agent stays blind to candidate code by construction.
    for testgen_prompt in &testgen_prompts {
        ::std::assert!(!testgen_prompt.contains("MARKER_TOKEN_ALPHA"));
    }
    // A visited node holding a subunit candidate dispatches the patch agent
    // with the recorded fault summary on the next rollout.
    let machine_b = make_machine();
    let blackboard_b = make_fail_suite_blackboard();
    let config_b = ::holon_engine::EngineConfig {
        max_rollouts: 2,
        ..::core::default::Default::default()
    };
    let codegen_script_b = ::std::vec![
        "pub fn dispatch_probe() -> u8 { 1 } // MARKER_TOKEN_ALPHA".to_string()
    ];
    let patch_script_b = ::std::vec![
        "pub fn dispatch_probe() -> u8 { 1 } // MARKER_TOKEN_ALPHA".to_string()
    ];
    let (mut engine_b, log_b) = build_engine(
        config_b,
        blackboard_b,
        codegen_script_b,
        patch_script_b,
        EMPTY_RESPONSE,
        0.0,
        ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic),
    );
    let _ = engine_b.solve(machine_b);
    let patch_prompts = prompt_calls_for(&log_b, "repair engine");
    ::std::assert_eq!(patch_prompts.len(), 1);
    ::std::assert!(patch_prompts[0].contains("distilled engine fault"));
    let codegen_prompts_b = prompt_calls_for(&log_b, "code synthesis engine");
    ::std::assert_eq!(codegen_prompts_b.len(), 1);
    let strategy_prompts_b = prompt_calls_for(&log_b, "strategy analyst");
    ::std::assert_eq!(strategy_prompts_b.len(), 2);
}

#[test]
fn contract_val_m2_05_budget_exhaustion_fallback() {
    // Exactly B_max rollouts without unit reward extract the best reward
    // candidate and advance the machine into contract negotiation.
    let machine_five = make_machine();
    let blackboard_five = make_fail_suite_blackboard();
    let config_five = ::holon_engine::EngineConfig {
        max_rollouts: 2,
        ..::core::default::Default::default()
    };
    let candidate_alpha = "pub fn alpha_rollout_candidate() -> u8 { 1 }".to_string();
    let codegen_scripts = ::std::vec![candidate_alpha.clone()];
    let patch_scripts = ::std::vec![candidate_alpha.clone()];
    let (mut engine_five, _log_five) = build_engine(
        config_five,
        blackboard_five,
        codegen_scripts,
        patch_scripts,
        EMPTY_RESPONSE,
        0.0,
        ::core::option::Option::None,
    );
    let outcome_five = engine_five.solve(machine_five);
    match outcome_five {
        ::holon_engine::EngineTypestateOutcome::BudgetExhausted {
            machine,
            best_candidate,
            best_reward,
            blocker_trace,
        } => {
            ::std::assert_eq!(engine_five.rollout_count(), 2);
            ::std::assert!((best_reward - 0.2).abs() < 1e-9);
            let extracted_code = best_candidate.expect("best candidate extracted");
            ::std::assert!(extracted_code.contains("alpha_rollout_candidate"));
            ::std::assert!(blocker_trace.contains("Rollout budget exhausted"));
            let mut negotiate_machine = machine;
            let events = negotiate_machine.drain_events();
            ::std::assert_eq!(events.len(), 1);
            let first_event = events.first().expect("event present");
            let is_conflict = ::core::matches!(
                first_event,
                ::holon_fsm::ClusterBroadcastEvent::ContractConflict { .. }
            );
            ::std::assert!(is_conflict);
        }
        _ => ::std::panic!("expected budget exhausted outcome"),
    }
}

#[test]
fn contract_val_m2_06_hardening_event_on_unit_reward() {
    // A rollout reaching unit reward emits the hardened broadcast event and
    // terminates the cluster loop after one rollout.
    let machine_gold = make_machine();
    let blackboard_gold = ::holon_blackboard::LocalBlackboard::new();
    let config_gold = ::holon_engine::EngineConfig {
        max_rollouts: 4,
        ..::core::default::Default::default()
    };
    let codegen_gold = ::std::vec!["pub fn gold_hardened_module() -> u8 { 7 }".to_string()];
    let (mut engine_gold, _log_gold) = build_engine(
        config_gold,
        blackboard_gold,
        codegen_gold,
        ::std::vec![],
        EMPTY_RESPONSE,
        1.0,
        ::core::option::Option::None,
    );
    let outcome_gold = engine_gold.solve(machine_gold);
    match outcome_gold {
        ::holon_engine::EngineTypestateOutcome::Hardened(hardened_machine) => {
            ::std::assert_eq!(engine_gold.rollout_count(), 1);
            let final_reward = hardened_machine.state.reward;
            ::std::assert!((final_reward - 1.0).abs() < 1e-9);
            ::std::assert!(hardened_machine.state.target_code.contains("gold_hardened_module"));
            let mut drainable_machine = hardened_machine;
            let events = drainable_machine.drain_events();
            ::std::assert_eq!(events.len(), 1);
            let first_event = events.first().expect("event present");
            let is_hardened = ::core::matches!(
                first_event,
                ::holon_fsm::ClusterBroadcastEvent::ModuleHardened { .. }
            );
            ::std::assert!(is_hardened);
        }
        _ => ::std::panic!("expected hardened outcome"),
    }
}

#[test]
fn contract_val_m2_07_initial_mode_seeds_characterization() {
    // INITIAL mode seeds the characterization suite through the test agent
    // before any generation rollout and resets the rollout counter.
    let config_seed = ::holon_engine::EngineConfig {
        max_rollouts: 1,
        ..::core::default::Default::default()
    };
    let seed_response = "#[test]\nfn t_probe_one() {\n    ::core::assert!(true)\n}\n\n#[test]\nfn t_probe_two() {\n    ::core::assert!(true)\n}".to_string();
    let seed_blackboard = ::holon_blackboard::LocalBlackboard::new();
    let (mut engine_seed, log_seed) = build_engine(
        config_seed,
        seed_blackboard,
        ::std::vec![],
        ::std::vec![],
        seed_response,
        1.0,
        ::core::option::Option::None,
    );
    engine_seed.initialize(::holon_fsm::ActivationMode::Initial);
    let seeded_suite = &engine_seed.blackboard().characterization_tests;
    ::std::assert!(seeded_suite.is_populated());
    ::std::assert_eq!(seeded_suite.test_count(), 2);
    ::std::assert_eq!(engine_seed.rollout_count(), 0);
    let testgen_prompts = prompt_calls_for(&log_seed, "test generator");
    ::std::assert_eq!(testgen_prompts.len(), 1);
}

#[test]
fn contract_val_m2_08_warm_start_retention() {
    // Warm start regression keeps the tree, repoints the active root at the
    // previously best node, and retains its candidate for further selection.
    let machine_warm = make_machine();
    let blackboard_warm = make_fail_suite_blackboard();
    let config_warm = ::holon_engine::EngineConfig {
        max_rollouts: 1,
        ..::core::default::Default::default()
    };
    let candidate_warm = "pub fn alpha_rollout_candidate() -> u8 { 1 }".to_string();
    let (mut engine_warm, _log_warm) = build_engine(
        config_warm,
        blackboard_warm,
        ::std::vec![candidate_warm.clone()],
        ::std::vec![candidate_warm.clone()],
        EMPTY_RESPONSE,
        0.0,
        ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic),
    );
    let _ = engine_warm.solve(machine_warm);
    let node_count_before = engine_warm.search_tree().len();
    let best_id = engine_warm.search_tree().best_node();
    engine_warm.initialize(::holon_fsm::ActivationMode::WarmStartRegression);
    ::std::assert_eq!(engine_warm.search_tree().root_id(), best_id);
    ::std::assert_eq!(engine_warm.search_tree().len(), node_count_before);
    let retained_node = engine_warm.search_tree().get(best_id).expect("node retained");
    let retained_code = retained_node.candidate_code.as_ref().expect("code retained");
    ::std::assert!(retained_code.contains("alpha_rollout_candidate"));
    // New contract constraints extend the retained problem model.
    engine_warm
        .blackboard_mut()
        .problem_model
        .add_delta_constraint("delta constraint: parent export widened".to_string());
    let constraint_present = engine_warm
        .blackboard()
        .problem_model
        .delta_constraints
        .iter()
        .any(|entry| entry.contains("parent export widened"));
    ::std::assert!(constraint_present);
}

#[test]
fn contract_val_m2_09_extrinsic_escalation() {
    // An extrinsic contract classification terminates the local loop after one
    // rollout and hands the machine to contract negotiation with the parent.
    let machine_contract = make_machine();
    let blackboard_contract = make_parent_blackboard();
    let config_contract = ::holon_engine::EngineConfig {
        max_rollouts: 4,
        ..::core::default::Default::default()
    };
    let (mut engine_contract, _log_contract) = build_engine(
        config_contract,
        blackboard_contract,
        ::std::vec!["pub fn contract_probe() -> u8 { 1 }".to_string()],
        ::std::vec![],
        EMPTY_RESPONSE,
        0.0,
        ::core::option::Option::Some(::holon_blackboard::FaultCause::ExtrinsicContract),
    );
    let outcome_contract = engine_contract.solve(machine_contract);
    match outcome_contract {
        ::holon_engine::EngineTypestateOutcome::ContractConflict(negotiate_machine) => {
            ::std::assert_eq!(engine_contract.rollout_count(), 1);
            let mut drainable_machine = negotiate_machine;
            ::std::assert_eq!(drainable_machine.state.parent_id, "vertex_parent_p9");
            ::std::assert!(drainable_machine.state.delta_spec.contains("distilled engine fault"));
            let events = drainable_machine.drain_events();
            ::std::assert_eq!(events.len(), 1);
            let first_event = events.first().expect("event present");
            let conflict_ok = match first_event {
                ::holon_fsm::ClusterBroadcastEvent::ContractConflict { parent_id, .. } => {
                    parent_id == "vertex_parent_p9"
                }
                _ => false,
            };
            ::std::assert!(conflict_ok);
        }
        _ => ::std::panic!("expected contract conflict outcome"),
    }
    // An extrinsic cycle classification emits the cycle broadcast and keeps
    // the machine inside the M2 search state.
    let machine_cycle = make_machine();
    let blackboard_cycle = make_parent_blackboard();
    let config_cycle = ::holon_engine::EngineConfig {
        max_rollouts: 4,
        ..::core::default::Default::default()
    };
    let (mut engine_cycle, _log_cycle) = build_engine(
        config_cycle,
        blackboard_cycle,
        ::std::vec!["pub fn cycle_probe() -> u8 { 1 }".to_string()],
        ::std::vec![],
        EMPTY_RESPONSE,
        0.0,
        ::core::option::Option::Some(::holon_blackboard::FaultCause::ExtrinsicCycle),
    );
    let outcome_cycle = engine_cycle.solve(machine_cycle);
    match outcome_cycle {
        ::holon_engine::EngineTypestateOutcome::CycleFound {
            machine,
            cycle_edges,
            ..
        } => {
            ::std::assert_eq!(engine_cycle.rollout_count(), 1);
            let cycle_ok = ::core::matches!(machine.state_tag(), ::holon_fsm::MicroStateTag::M2AriadneMcts);
            ::std::assert!(cycle_ok);
            let first_edge = cycle_edges.first().expect("cycle edge present");
            ::std::assert_eq!(first_edge.0, "vertex_parent_p9");
            let mut drainable_machine = machine;
            let events = drainable_machine.drain_events();
            let cycle_event_present = events
                .iter()
                .any(|event_item| ::core::matches!(
                    event_item,
                    ::holon_fsm::ClusterBroadcastEvent::CyclicDependencyFound { .. }
                ));
            ::std::assert!(cycle_event_present);
        }
        _ => ::std::panic!("expected cycle found outcome"),
    }
}

#[test]
fn contract_val_m2_10_intrinsic_containment() {
    // Intrinsic faults stay local: the controller applies a patch with the recorded
    // fault, backpropagates the lowered reward, and pushes no cycle broadcast.
    let machine_local = make_machine();
    let blackboard_local = make_fail_suite_blackboard();
    let config_local = ::holon_engine::EngineConfig {
        max_rollouts: 2,
        ..::core::default::Default::default()
    };
    let candidate_local = "pub fn local_rollout_candidate() -> u8 { 1 }".to_string();
    let (mut engine_local, log_local) = build_engine(
        config_local,
        blackboard_local,
        ::std::vec![candidate_local.clone()],
        ::std::vec![candidate_local.clone()],
        EMPTY_RESPONSE,
        0.0,
        ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic),
    );
    let outcome_local = engine_local.solve(machine_local);
    let exhausted_ok = ::core::matches!(
        outcome_local,
        ::holon_engine::EngineTypestateOutcome::BudgetExhausted { .. }
    );
    ::std::assert!(exhausted_ok);
    let patch_prompts = prompt_calls_for(&log_local, "repair engine");
    ::std::assert_eq!(patch_prompts.len(), 1);
    ::std::assert!(patch_prompts[0].contains("distilled engine fault"));
    // The low reward backpropagated into the visited child shifts selection.
    let child_node = engine_local.search_tree().get(1).expect("child present");
    ::std::assert!((child_node.mean_reward - 0.2).abs() < 1e-9);
    ::std::assert_eq!(child_node.visit_count, 2);
    // No cycle broadcast reaches the global event queue for intrinsic faults.
    let events = prompt_calls_for(&log_local, "strategy analyst");
    ::std::assert!(!events.is_empty());
}

#[test]
fn contract_val_m2_11_rollout_budget_honesty() {
    // The controller performs at most B_max rollouts for any configured cap.
    let machine_capped = make_machine();
    let blackboard_capped = make_fail_suite_blackboard();
    let config_capped = ::holon_engine::EngineConfig {
        max_rollouts: 5,
        ..::core::default::Default::default()
    };
    let candidate_capped = "pub fn capped_rollout_candidate() -> u8 { 1 }".to_string();
    let (mut engine_capped, _log_capped) = build_engine(
        config_capped,
        blackboard_capped,
        ::std::vec![candidate_capped.clone()],
        ::std::vec![candidate_capped.clone()],
        EMPTY_RESPONSE,
        0.0,
        ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic),
    );
    let outcome_capped = engine_capped.solve(machine_capped);
    let capped_ok = ::core::matches!(
        outcome_capped,
        ::holon_engine::EngineTypestateOutcome::BudgetExhausted { .. }
    );
    ::std::assert!(capped_ok);
    ::std::assert_eq!(engine_capped.rollout_count(), 5);
    // A zero budget performs no rollouts at all.
    let machine_zero = make_machine();
    let blackboard_zero = make_fail_suite_blackboard();
    let config_zero = ::holon_engine::EngineConfig {
        max_rollouts: 0,
        ..::core::default::Default::default()
    };
    let (mut engine_zero, _log_zero) = build_engine(
        config_zero,
        blackboard_zero,
        ::std::vec![],
        ::std::vec![],
        EMPTY_RESPONSE,
        0.0,
        ::core::option::Option::None,
    );
    let outcome_zero = engine_zero.solve(machine_zero);
    let zero_ok = ::core::matches!(
        outcome_zero,
        ::holon_engine::EngineTypestateOutcome::BudgetExhausted { .. }
    );
    ::std::assert!(zero_ok);
    ::std::assert_eq!(engine_zero.rollout_count(), 0);
}

/// Empty scripted response for agents under test.
const EMPTY_RESPONSE: ::std::string::String = ::std::string::String::new();

/// Build one M2 state machine through the legal M0 to M2 progression.
fn make_machine() -> ::holon_fsm::StateMachine<::holon_fsm::M2AriadneMcts> {
    let blocked_machine = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("module_contract_engine");
    let hydrated_machine = blocked_machine.wake_ready();
    let active_machine = hydrated_machine.hydration_completed();
    active_machine
}

/// Build a cluster engine over the fake model with scripted agent responses.
/// The returned call log records every role preamble plus prompt pair.
#[allow(clippy::type_complexity)]
fn build_engine(
    config: ::holon_engine::EngineConfig,
    blackboard: ::holon_blackboard::LocalBlackboard,
    codegen_scripts: ::std::vec::Vec<::std::string::String>,
    patch_scripts: ::std::vec::Vec<::std::string::String>,
    testgen_response: ::std::string::String,
    audit_score: ::core::primitive::f64,
    fault_cause: ::core::option::Option<::holon_blackboard::FaultCause>,
) -> (
    ::holon_engine::ClusterEngine<
        self::EngineFakeModel,
        ::holon_evaluator::DefaultExecutionHarness,
        self::FixedAuditDistiller,
    >,
    ::std::sync::Arc<
        ::std::sync::Mutex<::std::vec::Vec<(::std::string::String, ::std::string::String)>>,
    >,
) {
    let empty_log = ::std::vec::Vec::new();
    let log_mutex = ::std::sync::Mutex::new(empty_log);
    let call_log = ::std::sync::Arc::new(log_mutex);
    let model = self::EngineFakeModel {
        call_log: ::std::sync::Arc::clone(&call_log),
        codegen_script: {
            let codegen_deque = ::std::collections::VecDeque::from(codegen_scripts);
            let codegen_lock = ::std::sync::Mutex::new(codegen_deque);
            ::std::sync::Arc::new(codegen_lock)
        },
        patch_script: {
            let patch_deque = ::std::collections::VecDeque::from(patch_scripts);
            let patch_lock = ::std::sync::Mutex::new(patch_deque);
            ::std::sync::Arc::new(patch_lock)
        },
        testgen_response: { testgen_response },
    };
    let strategy_agent = ::holon_strategy::HolonStrategyAgent::builder()
        .model(model.clone())
        .build();
    let codegen_agent = ::holon_codegen::HolonCodeGenAgent::builder()
        .model(model.clone())
        .build();
    let testgen_agent = ::holon_testgen::HolonTestGenAgent::builder()
        .model(model.clone())
        .build();
    let patch_agent = ::holon_patch::HolonPatchAgent::builder()
        .model(model.clone())
        .build();
    let strategy_handle = ::std::sync::Arc::new(strategy_agent);
    let codegen_handle = ::std::sync::Arc::new(codegen_agent);
    let testgen_handle = ::std::sync::Arc::new(testgen_agent);
    let patch_handle = ::std::sync::Arc::new(patch_agent);
    let agents = ::holon_engine::SubagentPool::builder()
        .strategy_agent(strategy_handle)
        .code_gen_agent(codegen_handle)
        .test_gen_agent(testgen_handle)
        .patch_agent(patch_handle)
        .build();
    let harness = ::holon_evaluator::DefaultExecutionHarness::new();
    let distiller = self::FixedAuditDistiller {
        fault_cause: { fault_cause },
        audit_score: { audit_score },
    };
    let evaluator = ::holon_evaluator::TwoStageEvaluator::builder()
        .harness(harness)
        .distiller(distiller)
        .build();
    let engine = ::holon_engine::ClusterEngine::builder()
        .config(config)
        .blackboard(blackboard)
        .evaluator(evaluator)
        .agents(agents)
        .build();
    (engine, call_log)
}

/// Collect the prompt texts recorded for one agent role marker.
#[allow(clippy::type_complexity)]
fn prompt_calls_for(
    call_log: &::std::sync::Arc<
        ::std::sync::Mutex<::std::vec::Vec<(::std::string::String, ::std::string::String)>>,
    >,
    role_marker: &::core::primitive::str,
) -> ::std::vec::Vec<::std::string::String> {
    let log_guard = call_log.lock().expect("call log lock");
    let mut matched_prompts = ::std::vec::Vec::new();
    for log_entry in log_guard.iter() {
        let role_preamble = &log_entry.0;
        if role_preamble.contains(role_marker) {
            matched_prompts.push(log_entry.1.clone());
        }
    }
    matched_prompts
}

/// Distiller double returning a constant fault cause and audit score.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
struct FixedAuditDistiller {
    fault_cause: ::core::option::Option<::holon_blackboard::FaultCause>,
    audit_score: ::core::primitive::f64,
}

impl ::holon_evaluator::DiagnosticDistiller for self::FixedAuditDistiller {
    fn distill_failure(
        &self,
        raw_logs: &::core::primitive::str,
        _candidate_code: &::core::primitive::str,
        _problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::holon_blackboard::FaultRecord {
        ::holon_blackboard::FaultRecord {
            summary: ::std::string::String::from("distilled engine fault"),
            diagnostic_trace: { raw_logs.to_string() },
            cause: { self.fault_cause },
        }
    }

    fn audit_rules(
        &self,
        _candidate_code: &::core::primitive::str,
        _problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::core::primitive::f64 {
        self.audit_score
    }
}

/// Deterministic completion model routing canned responses by role preamble.
/// Each specialist agent carries a distinct system preamble so the fake can
/// serve scripted per role responses and record every observed prompt.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
struct EngineFakeModel {
    call_log: ::std::sync::Arc<
        ::std::sync::Mutex<::std::vec::Vec<(::std::string::String, ::std::string::String)>>,
    >,
    codegen_script: ::std::sync::Arc<
        ::std::sync::Mutex<::std::collections::VecDeque<::std::string::String>>,
    >,
    patch_script: ::std::sync::Arc<
        ::std::sync::Mutex<::std::collections::VecDeque<::std::string::String>>,
    >,
    testgen_response: ::std::string::String,
}

impl ::rig::completion::CompletionModel for self::EngineFakeModel {
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
        let preamble_text = request.preamble.clone().unwrap_or_default();
        let prompt_text = match request.prompt {
            ::rig::message::Message::User { content } => {
                let prompt_block = content.first();
                match prompt_block {
                    ::rig::message::UserContent::Text(text_value) => text_value.text.clone(),
                    _ => ::std::string::String::new(),
                }
            }
            _ => ::std::string::String::new(),
        };
        let log_guard = self.call_log.lock().expect("call log lock");
        let mut log_guard = log_guard;
        log_guard.push((preamble_text.clone(), prompt_text));
        ::core::mem::drop(log_guard);
        let is_strategy_call = preamble_text.contains("strategy analyst");
        let is_codegen_call = preamble_text.contains("code synthesis engine");
        let is_testgen_call = preamble_text.contains("test generator");
        let response_text = if is_strategy_call {
            ::std::string::String::from(
                r#"[{"strategy_id":"STRAT_DISPATCH_01","description":"direct port strategy","prior_utility":0.70}]"#,
            )
        } else if is_codegen_call {
            let script_guard = self.codegen_script.lock().expect("codegen script lock");
            let mut script_guard = script_guard;
            match script_guard.pop_front() {
                ::core::option::Option::Some(scripted_code) => scripted_code,
                ::core::option::Option::None => {
                    ::std::string::String::from("pub fn fallback_candidate() -> u8 { 0 }")
                }
            }
        } else if is_testgen_call {
            self.testgen_response.clone()
        } else {
            let script_guard = self.patch_script.lock().expect("patch script lock");
            let mut script_guard = script_guard;
            match script_guard.pop_front() {
                ::core::option::Option::Some(scripted_code) => scripted_code,
                ::core::option::Option::None => {
                    ::std::string::String::from("pub fn fallback_candidate() -> u8 { 0 }")
                }
            }
        };
        let content_item = ::rig::message::AssistantContent::text(response_text);
        let choice_bundle = ::rig::OneOrMany::one(content_item);
        let outcome = ::rig::completion::CompletionResponse {
            choice: { choice_bundle },
            raw_response: ::std::string::String::from("engine_contract"),
        };
        ::core::future::ready(::core::result::Result::Ok(outcome))
    }
}

/// Build a blackboard whose characterization suite always fails evaluation.
fn make_fail_suite_blackboard() -> ::holon_blackboard::LocalBlackboard {
    let mut blackboard = ::holon_blackboard::LocalBlackboard::new();
    let failing_entry = ::std::string::String::from("FAIL: parity probe missing");
    blackboard.characterization_tests.add_test_case(failing_entry);
    blackboard
}

/// Build a blackboard carrying one upstream parent contract.
fn make_parent_blackboard() -> ::holon_blackboard::LocalBlackboard {
    let mut blackboard = ::holon_blackboard::LocalBlackboard::new();
    let failing_entry = ::std::string::String::from("FAIL: parent parity probe missing");
    blackboard.characterization_tests.add_test_case(failing_entry);
    let parent_contract = ::holon_blackboard::EdgeContract {
        parent_id: "vertex_parent_p9".to_string(),
        interface_spec: "pub fn parent_export() -> u8".to_string(),
        preconditions: ::std::vec::Vec::new(),
    };
    blackboard.problem_model.add_parent_contract(parent_contract);
    blackboard
}

