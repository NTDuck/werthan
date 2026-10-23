//! Macro-orchestrator engine validation contracts.
//! Contracts derived from docs/specs/validation-contracts.md. They outrank implementation tests.
//! Covered assertion IDs follow. VAL-MACRO-01 through VAL-MACRO-16.

use ::global_engine::ModuleSolver;

/// Shared audit log type for critic recordings (phase, batch pairs).
type AuditLog = ::std::vec::Vec<(::global_fsm::MacroState, ::std::vec::Vec<::std::string::String>)>;

/// Contract VAL-MACRO-01: the S0 intent and preflight gate blocks downstream
/// dispatch. A failing harness probe keeps the engine in S0 with no S1 work.
/// A valid harness lets the campaign proceed past S0.
#[test]
fn contract_val_macro_01_s0_intent_preflight_gate() {
    // Broken harness: the intent gate rejects the campaign.
    let stub_one = ::global_engine::StubModuleSolver::default();
    let solver_one = ::std::boxed::Box::new(stub_one);
    let mut engine = base_engine(solver_one);
    let mut broken_board = chain_board();
    broken_board.inventory.harness = ::global_blackboard::HarnessConfig::default();
    engine.configure(broken_board);
    let outcome = engine.run();
    let error = outcome.expect_err("broken harness must fail the preflight gate");
    ::std::assert!(
        ::core::matches!(error, ::global_engine::MacroEngineError::PreflightGateFailed { .. })
    );
    ::std::assert_eq!(
        engine.state_vector().macro_state,
        ::global_fsm::MacroState::S0IntentPreflight
    );
    ::std::assert!(
        engine.board().work_manifests.is_empty(),
        "no downstream phase scattered a manifest"
    );
    // Valid harness: the campaign proceeds past S0 into S1.
    let stub_two = ::global_engine::StubModuleSolver::default();
    let solver_two = ::std::boxed::Box::new(stub_two);
    let mut healthy = base_engine(solver_two);
    let board = chain_board();
    healthy.configure(board);
    let good = healthy.run();
    ::std::assert!(good.is_ok());
    let reached_s1 = healthy
        .board()
        .work_manifests
        .iter()
        .any(|manifest| manifest.phase == ::global_fsm::MacroState::S1Assess);
    ::std::assert!(reached_s1, "valid intent dispatches the S1 assess phase");
}

/// Contract VAL-MACRO-02: scatter-gather topology. Each phase partitions its
/// workload into K shards and issues exactly K worker activations, one per
/// shard. No worker sees another shard subject.
#[test]
fn contract_val_macro_02_scatter_gather_exactly_k_activations() {
    let recorder = recording_workers();
    let recorder_shared = ::core::clone::Clone::clone(&recorder);
    let crew = recording_crew(recorder_shared);
    let stub = ::global_engine::StubModuleSolver::default();
    let solver = ::std::boxed::Box::new(stub);
    let mut engine = ::global_engine::MacroEngine::builder().solver(solver).workers(crew).build();
    let board = chain_board();
    engine.configure(board);
    let outcome = engine.run();
    ::std::assert!(outcome.is_ok());
    let records = recorder.lock().expect("records lock");
    let s3_subjects = subjects_for_phase(&records, ::global_fsm::MacroState::S3ExtractRules);
    ::std::assert_eq!(s3_subjects, ::std::vec!["a", "b", "c"]);
    let s6_subjects = subjects_for_phase(&records, ::global_fsm::MacroState::S6SystemVerify);
    ::std::assert_eq!(s6_subjects, ::std::vec!["a", "b", "c"]);
    let s4_subjects = subjects_for_phase(&records, ::global_fsm::MacroState::S4BriefContracts);
    ::std::assert_eq!(s4_subjects, ::std::vec!["a->b", "b->c"]);
    ::std::assert_eq!(records.len(), engine.worker_executions());
}

/// Contract VAL-MACRO-03: recursion depth one. Workers expose no spawn
/// operation and a cross-shard reference surfaces as an UnresolvedCrossLink
/// tag resolved by the reduce stage, never as a nested worker activation.
#[test]
fn contract_val_macro_03_depth_one_cross_shard_tags() {
    let recorder = recording_workers();
    let recorder_shared = ::core::clone::Clone::clone(&recorder);
    let crew = recording_crew(recorder_shared);
    let stub = ::global_engine::StubModuleSolver::default();
    let solver = ::std::boxed::Box::new(stub);
    let mut engine = ::global_engine::MacroEngine::builder().solver(solver).workers(crew).build();
    let mut board = chain_board();
    board.rules.insert("a".to_string(), ::std::vec![rule_card("RA_NAMES_B")]);
    board.cross_links.push(::global_blackboard::UnresolvedCrossLink {
        src_id: "RA_NAMES_B".to_string(),
        tgt_symbol: "b".to_string(),
    });
    engine.configure(board);
    let outcome = engine.run();
    ::std::assert!(outcome.is_ok());
    // The reduce stage resolved the tag by copying the card onto vertex b.
    let linked = engine.board().rules.get("b").map(|cards| {
        cards.iter().any(|card| card.rule_id == "RA_NAMES_B")
    });
    ::std::assert_eq!(linked, ::core::option::Option::Some(true));
    ::std::assert!(engine.board().cross_links.is_empty(), "resolved tags drop off the board");
    // The tag resolution caused no extra worker activation: S3 ran its three shards once.
    let records = recorder.lock().expect("records lock");
    let s3_subjects = subjects_for_phase(&records, ::global_fsm::MacroState::S3ExtractRules);
    ::std::assert_eq!(s3_subjects, ::std::vec!["a", "b", "c"]);
}

/// Contract VAL-MACRO-04: structural guard short-circuit. A failing structural
/// guard on shard b re-dispatches only worker b with the validator detail. No
/// semantic critic audits the phase while the structural guard still fails.
#[test]
fn contract_val_macro_04_structural_short_circuit_targets_one_shard() {
    let recorder = recording_workers();
    let recorder_shared = ::core::clone::Clone::clone(&recorder);
    let crew = recording_crew(recorder_shared);
    let critic_pair = scripted_critic(crate::CriticScript::AlwaysPass);
    let critic = critic_pair.0;
    let critic_log = critic_pair.1;
    let stub = ::global_engine::StubModuleSolver::default();
    let solver = ::std::boxed::Box::new(stub);
    let mut engine = ::global_engine::MacroEngine::builder().solver(solver).workers(crew).critic(critic).build();
    let mut board = chain_board();
    let broken_card = ::holon_blackboard::BusinessRuleCard {
        rule_id: { "RB".to_string() },
        given: "given clause".to_string(),
        when: ::std::string::String::new(),
        then: "then clause".to_string(),
        citation: "legacy/b.c:7".to_string(),
    };
    board.rules.insert("b".to_string(), ::std::vec![broken_card]);
    engine.configure(board);
    let outcome = engine.run();
    let error = outcome.expect_err("the unrepairable card must fail the structural guard");
    ::std::assert!(::core::matches!(
        error,
        ::global_engine::MacroEngineError::StructuralGuardFailure {
            phase: ::global_fsm::MacroState::S3ExtractRules,
            ..
        }
    ));
    ::std::assert_eq!(engine.structural_retries(), 1);
    // Only shard b was re-dispatched, and it carried the guard detail.
    let records = recorder.lock().expect("records lock");
    let s3_subjects = subjects_for_phase(&records, ::global_fsm::MacroState::S3ExtractRules);
    ::std::assert_eq!(s3_subjects, ::std::vec!["a", "b", "c", "b"]);
    let retry_records = records
        .iter()
        .filter(|record| record.phase == ::global_fsm::MacroState::S3ExtractRules)
        .filter(|record| record.subject == "b")
        .collect::<::std::vec::Vec<_>>();
    ::std::assert_eq!(retry_records.len(), 2);
    ::std::assert_eq!(
        retry_records[1].directive,
        ::core::option::Option::Some("b: rule card holds an empty clause".to_string())
    );
    // No semantic critic ran for S3 while the structural guard failed.
    let audits = critic_log.lock().expect("critic lock");
    let s3_audited = audits
        .iter()
        .any(|entry| entry.0 == ::global_fsm::MacroState::S3ExtractRules);
    ::std::assert!(!s3_audited, "g_sem must not run while g_struct fails");
}

/// Contract VAL-MACRO-05: two-stage pipeline. Semantic critics evaluate only
/// after every shard passes the structural guard, and the critic audits one
/// batch per shard as composed by semantic_batches.
#[test]
fn contract_val_macro_05_two_stage_pipeline_order() {
    let critic_pair = scripted_critic(crate::CriticScript::AlwaysPass);
    let critic = critic_pair.0;
    let critic_log = critic_pair.1;
    let stub = ::global_engine::StubModuleSolver::default();
    let solver = ::std::boxed::Box::new(stub);
    let mut engine = ::global_engine::MacroEngine::builder().solver(solver).critic(critic).build();
    let board = chain_board();
    engine.configure(board);
    let outcome = engine.run();
    ::std::assert!(outcome.is_ok());
    let audits = critic_log.lock().expect("critic lock");
    let s3_batches = batches_for_phase(&audits, ::global_fsm::MacroState::S3ExtractRules);
    ::std::assert_eq!(s3_batches, ::std::vec![::std::vec!["RA"], ::std::vec!["RB"], ::std::vec!["RC"]]);
    let s6_batches = batches_for_phase(&audits, ::global_fsm::MacroState::S6SystemVerify);
    ::std::assert_eq!(s6_batches, ::std::vec![::std::vec!["a"], ::std::vec!["b"], ::std::vec!["c"]]);
    // The S4 critic audited after the structural S4 contract guard passed.
    let s4_batches = batches_for_phase(&audits, ::global_fsm::MacroState::S4BriefContracts);
    ::std::assert_eq!(s4_batches, ::std::vec![::std::vec!["a->b"], ::std::vec!["b->c"]]);
}

/// Contract VAL-MACRO-06: a REJECT_LOCAL critique re-dispatches exactly the
/// offending shard with the CritiqueRecord notes as delta directive and
/// activation mode CRITIC_RETRY. Other shards stay untouched.
#[test]
fn contract_val_macro_06_reject_local_retry_single_shard() {
    let recorder = recording_workers();
    let recorder_shared = ::core::clone::Clone::clone(&recorder);
    let crew = recording_crew(recorder_shared);
    let script = crate::CriticScript::RejectLocalOnce {
        phase: ::global_fsm::MacroState::S6SystemVerify,
        marker: "b".to_string(),
    };
    let critic_pair = scripted_critic(script);
    let critic = critic_pair.0;
    let _critic_log = critic_pair.1;
    let stub = ::global_engine::StubModuleSolver::default();
    let solver = ::std::boxed::Box::new(stub);
    let mut engine = ::global_engine::MacroEngine::builder().solver(solver).workers(crew).critic(critic).build();
    let board = chain_board();
    engine.configure(board);
    let outcome = engine.run();
    ::std::assert!(outcome.is_ok());
    ::std::assert_eq!(
        engine.state_vector().macro_state,
        ::global_fsm::MacroState::Terminated
    );
    let records = recorder.lock().expect("records lock");
    let s6_subjects = subjects_for_phase(&records, ::global_fsm::MacroState::S6SystemVerify);
    ::std::assert_eq!(s6_subjects, ::std::vec!["a", "b", "c", "b"]);
    let retry = records
        .iter()
        .filter(|record| record.phase == ::global_fsm::MacroState::S6SystemVerify)
        .filter(|record| record.subject == "b")
        .collect::<::std::vec::Vec<_>>();
    ::std::assert_eq!(retry.len(), 2);
    ::std::assert_eq!(retry[1].mode, ::holon_fsm::ActivationMode::CriticRetry);
    ::std::assert_eq!(
        retry[1].directive,
        ::core::option::Option::Some("fix module b".to_string())
    );
}

/// Contract VAL-MACRO-07: a REGRESS_UPSTREAM critique moves S_macro to the
/// upstream target and the loop continues from there. No restart, no
/// termination, and the campaign still completes.
#[test]
fn contract_val_macro_07_regress_upstream_continues_loop() {
    let script = crate::CriticScript::RegressOnce {
        phase: ::global_fsm::MacroState::S4BriefContracts,
        marker: "a->b".to_string(),
    };
    let critic_pair = scripted_critic(script);
    let critic = critic_pair.0;
    let _critic_log = critic_pair.1;
    let stub = ::global_engine::StubModuleSolver::default();
    let solver = ::std::boxed::Box::new(stub);
    let mut engine = ::global_engine::MacroEngine::builder().solver(solver).critic(critic).build();
    let board = chain_board();
    engine.configure(board);
    let outcome = engine.run();
    ::std::assert!(outcome.is_ok());
    ::std::assert_eq!(
        engine.state_vector().macro_state,
        ::global_fsm::MacroState::Terminated
    );
    let target = ::global_engine::regress_target(::global_fsm::MacroState::S4BriefContracts);
    ::std::assert_eq!(target, ::global_fsm::MacroState::S3ExtractRules);
    let history = engine.macro_history();
    let s4_positions = positions_of(history, ::global_fsm::MacroState::S4BriefContracts);
    ::std::assert_eq!(s4_positions.len(), 2, "S4 ran twice: before and after the regression");
    let s3_after_regress = positions_of(history, ::global_fsm::MacroState::S3ExtractRules);
    ::std::assert!(
        s3_after_regress.len() >= 2,
        "the loop resumed at the upstream S3 target"
    );
    ::std::assert!(s4_positions[1] > s3_after_regress[1]);
}

/// Contract VAL-MACRO-08: per-phase semantic guard obligations. The batch
/// composition per phase carries the methodology-mandated concern data.
#[test]
fn contract_val_macro_08_per_phase_semantic_batches() {
    let board = chain_board();
    let none_scope = ::core::option::Option::None;
    let s1 = ::global_engine::semantic_batches(&board, ::global_fsm::MacroState::S1Assess, &none_scope);
    ::std::assert_eq!(s1, ::std::vec![::std::vec!["a", "b", "c"]], "S1 audits inventory citations");
    let s2 = ::global_engine::semantic_batches(&board, ::global_fsm::MacroState::S2MapDag, &none_scope);
    ::std::assert_eq!(s2, ::std::vec![::std::vec!["a", "b", "c"]], "S2 audits the coupling map");
    let s3 = ::global_engine::semantic_batches(&board, ::global_fsm::MacroState::S3ExtractRules, &none_scope);
    ::std::assert_eq!(
        s3,
        ::std::vec![::std::vec!["RA"], ::std::vec!["RB"], ::std::vec!["RC"]],
        "S3 audits rule card completeness per vertex"
    );
    let s4 = ::global_engine::semantic_batches(&board, ::global_fsm::MacroState::S4BriefContracts, &none_scope);
    ::std::assert_eq!(
        s4,
        ::std::vec![::std::vec!["a->b"], ::std::vec!["b->c"]],
        "S4 audits interface contracts per edge"
    );
    let s6 = ::global_engine::semantic_batches(&board, ::global_fsm::MacroState::S6SystemVerify, &none_scope);
    ::std::assert_eq!(s6, ::std::vec![::std::vec!["a"], ::std::vec!["b"], ::std::vec!["c"]]);
}

/// Contract VAL-MACRO-09: a Tier C cycle event with a valid trace saves deep
/// history, quiesces the active regions, and transitions S_macro back to S2.
#[test]
fn contract_val_macro_09_tier_c_deep_history_checkpoint() {
    let engine_pair = run_cycle_campaign();
    let engine = engine_pair.0;
    ::std::assert!(engine.state_vector().deep_history.is_some());
    let history = engine.state_vector().deep_history.as_ref().expect("history present");
    ::std::assert_eq!(
        history.get("d"),
        ::core::option::Option::Some(&::global_blackboard::ModuleMicroState::M4Hardened)
    );
    let macro_history = engine.macro_history();
    let s5_positions = positions_of(macro_history, ::global_fsm::MacroState::S5IterativeDag);
    ::std::assert!(s5_positions.len() >= 2, "the campaign entered S5 twice across the regression");
    let first_s5 = s5_positions[0];
    ::std::assert!(
        macro_history.len() > first_s5 + 1
            && macro_history[first_s5 + 1] == ::global_fsm::MacroState::S2MapDag,
        "the cycle regression transitioned S5 back to S2"
    );
    ::std::assert!(engine.board().dag.is_acyclic());
}

/// Contract VAL-MACRO-10: after SCC condensation and S5 re-entry, every module
/// outside the mended cycle restores its prior micro-state without
/// re-execution. Module d hardens exactly once across the whole campaign.
#[test]
fn contract_val_macro_10_history_restoration_without_reexecution() {
    let engine_pair = run_cycle_campaign();
    let engine = engine_pair.0;
    let solves = engine_pair.1;
    let solve_log = solves.lock().expect("solver log lock");
    let d_solves = solve_log.iter().filter(|name| name.as_str() == "d").count();
    ::std::assert_eq!(d_solves, 1, "the unaffected module never re-executed");
    ::std::assert_eq!(
        engine.state_vector().module_states.get("d"),
        ::core::option::Option::Some(&::global_blackboard::ModuleMicroState::M4Hardened)
    );
    let checkpoint = engine
        .state_vector()
        .deep_history
        .as_ref()
        .expect("history present")
        .get("d")
        .expect("d checkpointed");
    ::std::assert_eq!(checkpoint, &::global_blackboard::ModuleMicroState::M4Hardened);
}

/// Contract VAL-MACRO-11: entering S5 initializes per-module states from deep
/// history where present and to M0_Blocked otherwise.
#[test]
fn contract_val_macro_11_s5_entry_initializes_from_history() {
    let engine_pair = run_cycle_campaign();
    let engine = engine_pair.0;
    // The restored module re-entered its checkpointed micro-state.
    ::std::assert_eq!(
        engine.state_vector().module_states.get("d"),
        ::core::option::Option::Some(&::global_blackboard::ModuleMicroState::M4Hardened)
    );
    // The restore wrote a second transition entry for d on S5 re-entry.
    let restore_entries = engine
        .transition_log()
        .iter()
        .filter(|entry| entry.0 == "d" && entry.1 == ::global_blackboard::ModuleMicroState::M4Hardened)
        .count();
    ::std::assert!(restore_entries >= 2, "the history restore recorded d as hardened again");
    // The merged cycle composite followed the normal M0 path on re-entry.
    let merged_state = engine.state_vector().module_states.get("a+b");
    ::std::assert!(merged_state.is_some());
}

/// Contract VAL-MACRO-12: the engine exits S5 to S6 exactly when every module
/// reports M4_Hardened. A livelocked S5 never reaches S6.
#[test]
fn contract_val_macro_12_s5_completion_guard() {
    let always_cycle = crate::AlwaysCycleSolver;
    let solver = ::std::boxed::Box::new(always_cycle);
    let mut stuck = base_engine(solver);
    let stuck_board = chain_board();
    stuck.configure(stuck_board);
    let outcome = stuck.run();
    let error = outcome.expect_err("the perpetual cycle livelocks the campaign");
    ::std::assert!(::core::matches!(
        error,
        ::global_engine::MacroEngineError::StepBudgetExhausted { .. }
    ));
    let s6_positions = positions_of(stuck.macro_history(), ::global_fsm::MacroState::S6SystemVerify);
    ::std::assert!(s6_positions.is_empty(), "S5 must not advance to S6 unhardened");
    // The healthy campaign exits S5 exactly once, on full hardening.
    let stub = ::global_engine::StubModuleSolver::default();
    let solver = ::std::boxed::Box::new(stub);
    let mut healthy = base_engine(solver);
    let board = chain_board();
    healthy.configure(board);
    let good = healthy.run();
    ::std::assert!(good.is_ok());
    let history = healthy.macro_history();
    let s5_count = positions_of(history, ::global_fsm::MacroState::S5IterativeDag).len();
    ::std::assert_eq!(s5_count, 1);
    let s6_count = positions_of(history, ::global_fsm::MacroState::S6SystemVerify).len();
    ::std::assert_eq!(s6_count, 1);
    for vertex in ["a", "b", "c"] {
        ::std::assert_eq!(
            healthy.state_vector().module_states.get(vertex),
            ::core::option::Option::Some(&::global_blackboard::ModuleMicroState::M4Hardened)
        );
    }
}

/// Contract VAL-MACRO-13: Tier B and Tier C regressions execute only when the
/// arbiter validates the interrupt trace. A rejection leaves the state
/// unchanged. The engine drops the interrupt.
#[test]
fn contract_val_macro_13_arbiter_gated_regressions() {
    // Tier C: a rejected cycle event triggers no regression.
    let cycle_solver = crate::CycleThenHardenSolver { cycle_fired: false };
    let cycle_boxed = ::std::boxed::Box::new(cycle_solver);
    let rejecting = crate::RejectingArbiter;
    let arbiter = ::std::boxed::Box::new(rejecting);
    let mut engine = ::global_engine::MacroEngine::builder().solver(cycle_boxed).arbiter(arbiter).build();
    let cycle_campaign = cycle_board();
    engine.configure(cycle_campaign);
    let outcome = engine.run();
    let error = outcome.expect_err("the unhardened cycle member deadlocks the wave");
    ::std::assert!(::core::matches!(error, ::global_engine::MacroEngineError::WaveDeadlock { .. }));
    ::std::assert!(engine.state_vector().deep_history.is_none(), "no checkpoint without a validated interrupt");
    ::std::assert!(
        !engine.board().dag.vertices.iter().any(|vertex| vertex == "a+b"),
        "the engine condensed nothing without a validated interrupt"
    );
    let s2_count = positions_of(engine.macro_history(), ::global_fsm::MacroState::S2MapDag).len();
    ::std::assert_eq!(s2_count, 1);
    // Tier B: a rejected contract conflict leaves the contract untouched.
    let conflict_inner = crate::ConflictThenHardenSolver { conflicts_fired: 0 };
    let conflict_boxed = ::std::boxed::Box::new(conflict_inner);
    let rejecting_two = crate::RejectingArbiter;
    let arbiter_two = ::std::boxed::Box::new(rejecting_two);
    let mut conflict_engine = ::global_engine::MacroEngine::builder().solver(conflict_boxed).arbiter(arbiter_two).build();
    let conflict_campaign = conflict_board();
    conflict_engine.configure(conflict_campaign);
    let conflict_outcome = conflict_engine.run();
    ::std::assert!(conflict_outcome.is_ok(), "the parked child re-dispatches instead of deadlocking the wave");
    let untouched = conflict_engine
        .board()
        .contracts
        .get(&("a".to_string(), "b".to_string()))
        .expect("seeded contract present");
    ::std::assert!(untouched.preconditions.is_empty(), "the delta was never applied");
    let invalidation = conflict_engine
        .transition_log()
        .iter()
        .any(|entry| entry.1 == ::global_blackboard::ModuleMicroState::M5Invalidated);
    ::std::assert!(!invalidation, "the parent kept its hardened state");
    let parked_child = conflict_engine
        .transition_log()
        .iter()
        .any(|entry| entry.0 == "b" && entry.1 == ::global_blackboard::ModuleMicroState::M0Blocked);
    ::std::assert!(parked_child, "the rejected child parked in M0_Blocked for re-dispatch");
    let hardened = conflict_engine.state_vector().module_states.clone();
    for vertex in ["a", "b"] {
        ::std::assert_eq!(
            hardened.get(vertex),
            ::core::option::Option::Some(&::global_blackboard::ModuleMicroState::M4Hardened),
            "the wave completed after parking"
        );
    }
}

/// Contract VAL-MACRO-14: every worker activation carries a typed envelope
/// holding the activation mode, a scoped one-hop slice, the delta directive,
/// and the read-only tool bindings.
#[test]
fn contract_val_macro_14_typed_activation_envelopes() {
    let recorder = recording_workers();
    let recorder_shared = ::core::clone::Clone::clone(&recorder);
    let crew = recording_crew(recorder_shared);
    let script = crate::CriticScript::RejectLocalOnce {
        phase: ::global_fsm::MacroState::S6SystemVerify,
        marker: "b".to_string(),
    };
    let critic_pair = scripted_critic(script);
    let critic = critic_pair.0;
    let _critic_log = critic_pair.1;
    let stub = ::global_engine::StubModuleSolver::default();
    let solver = ::std::boxed::Box::new(stub);
    let mut engine = ::global_engine::MacroEngine::builder().solver(solver).workers(crew).critic(critic).build();
    let board = chain_board();
    engine.configure(board);
    let outcome = engine.run();
    ::std::assert!(outcome.is_ok());
    let records = recorder.lock().expect("records lock");
    for record in records.iter() {
        ::std::assert!(!record.bound_tools.is_empty(), "tool bindings ride every envelope");
        ::std::assert!(
            record.slice_summary.contains(&::std::format!("target: {}", record.subject)),
            "the slice names the shard subject"
        );
    }
    // One-hop scoping: the slice for c names parent b but never grandparent a.
    let c_record = records
        .iter()
        .filter(|record| record.phase == ::global_fsm::MacroState::S6SystemVerify)
        .find(|record| record.subject == "c")
        .expect("c dispatched");
    ::std::assert!(c_record.slice_summary.contains("parent: b"));
    ::std::assert!(!c_record.slice_summary.contains("parent: a"));
    // Initial activations ride the Initial mode, retries ride CriticRetry.
    let initial = records
        .iter()
        .filter(|record| record.phase == ::global_fsm::MacroState::S6SystemVerify)
        .find(|record| record.subject == "a")
        .expect("a dispatched");
    ::std::assert_eq!(initial.mode, ::holon_fsm::ActivationMode::Initial);
    ::std::assert_eq!(initial.directive, ::core::option::Option::None);
    let retry = records
        .iter()
        .filter(|record| record.phase == ::global_fsm::MacroState::S6SystemVerify)
        .filter(|record| record.subject == "b")
        .collect::<::std::vec::Vec<_>>();
    ::std::assert_eq!(retry[1].mode, ::holon_fsm::ActivationMode::CriticRetry);
    ::std::assert_eq!(
        retry[1].directive,
        ::core::option::Option::Some("fix module b".to_string())
    );
}

/// Contract VAL-MACRO-15: push-based completion. Workers observe queued typed
/// events only, the queue drains FIFO, and no polling surface exists.
#[test]
fn contract_val_macro_15_push_based_event_completion() {
    let recorder = recording_workers();
    let recorder_shared = ::core::clone::Clone::clone(&recorder);
    let crew = recording_crew(recorder_shared);
    let stub = ::global_engine::StubModuleSolver::default();
    let solver = ::std::boxed::Box::new(stub);
    let mut engine = ::global_engine::MacroEngine::builder().solver(solver).workers(crew).build();
    let board = chain_board();
    engine.configure(board);
    let outcome = engine.run();
    ::std::assert!(outcome.is_ok());
    let records = recorder.lock().expect("records lock");
    // During S3, each later shard observed the earlier shards' queued
    // ev_ShardDone markers: completion arrives as typed events, not polls.
    let s3_records = records
        .iter()
        .filter(|record| record.phase == ::global_fsm::MacroState::S3ExtractRules)
        .collect::<::std::vec::Vec<_>>();
    ::std::assert!(s3_records[0].queued_events.is_empty());
    ::std::assert_eq!(s3_records[1].queued_events.len(), 1);
    ::std::assert!(s3_records[1].queued_events[0].contains("ShardDone"));
    ::std::assert!(s3_records[1].queued_events[0].contains("shard_index: 0"));
    ::std::assert_eq!(s3_records[2].queued_events.len(), 2);
    ::std::assert!(s3_records[2].queued_events[1].contains("shard_index: 1"));
    // The queue drains FIFO and pops empty when exhausted.
    let mut state_vector = ::global_blackboard::GlobalStateVector::initialize();
    // Drain the seeded Start marker so the probe observes pure FIFO order.
    while state_vector.pop_event().is_some() {}
    state_vector.push_event(::global_blackboard::MacroEvent::ShardDone { shard_index: 0 });
    state_vector.push_event(::global_blackboard::MacroEvent::ShardDone { shard_index: 1 });
    let first = state_vector.pop_event();
    ::std::assert!(::core::matches!(first, ::core::option::Option::Some(::global_blackboard::MacroEvent::ShardDone { shard_index: 0 })));
    let second = state_vector.pop_event();
    ::std::assert!(::core::matches!(second, ::core::option::Option::Some(::global_blackboard::MacroEvent::ShardDone { shard_index: 1 })));
    ::std::assert!(state_vector.pop_event().is_none());
    // The campaign drained its queue completely before terminating.
    ::std::assert!(engine.state_vector().event_queue.is_empty());
}

/// Contract VAL-MACRO-16: the termination artifact set equals the accumulated
/// artifacts on G and the final macro-state is terminal.
#[test]
fn contract_val_macro_16_termination_artifact_set() {
    let stub = ::global_engine::StubModuleSolver::default();
    let solver = ::std::boxed::Box::new(stub);
    let mut engine = base_engine(solver);
    let board = chain_board();
    engine.configure(board);
    let outcome = engine.run();
    ::std::assert!(outcome.is_ok());
    ::std::assert_eq!(
        engine.state_vector().macro_state,
        ::global_fsm::MacroState::Terminated
    );
    for vertex in &engine.board().dag.vertices {
        let artifact = engine.board().artifacts.get(vertex);
        let produced = match artifact {
            ::core::option::Option::Some(::core::option::Option::Some(entry)) => entry,
            other => ::std::panic!("vertex {vertex} lacks a produced artifact: {other:?}"),
        };
        ::std::assert!(!produced.target_code.is_empty());
        ::std::assert!(!produced.test_suite.is_empty());
    }
    ::std::assert_eq!(engine.board().dag.vertices.len(), 3);
    let hardened_count = engine
        .state_vector()
        .module_states
        .iter()
        .filter(|(_, state)| **state == ::global_blackboard::ModuleMicroState::M4Hardened)
        .count();
    ::std::assert_eq!(hardened_count, 3);
}

/// Contract VAL-MACRO-02 supporting probe. The deterministic engine commits
/// each shard output directly and the manifest records the scatter.
#[test]
fn contract_val_macro_02_supporting_work_manifests() {
    let stub = ::global_engine::StubModuleSolver::default();
    let solver = ::std::boxed::Box::new(stub);
    let mut engine = base_engine(solver);
    let board = chain_board();
    engine.configure(board);
    let outcome = engine.run();
    ::std::assert!(outcome.is_ok());
    let manifests = &engine.board().work_manifests;
    ::std::assert!(manifests.len() >= 6, "every discovery phase scattered one manifest");
    let s3_manifest = manifests
        .iter()
        .find(|manifest| manifest.phase == ::global_fsm::MacroState::S3ExtractRules)
        .expect("S3 manifest present");
    ::std::assert!(s3_manifest.covers("a"));
    ::std::assert!(s3_manifest.covers("c"));
    ::std::assert!(!s3_manifest.covers("missing"));
}



/// One recorded worker activation.
#[derive(::core::fmt::Debug, ::core::clone::Clone)]
struct ExecutionRecord {
    phase: ::global_fsm::MacroState,
    subject: ::std::string::String,
    mode: ::holon_fsm::ActivationMode,
    directive: ::core::option::Option<::std::string::String>,
    slice_summary: ::std::string::String,
    bound_tools: ::std::vec::Vec<::std::string::String>,
    queued_events: ::std::vec::Vec<::std::string::String>,
}

/// Worker crew that records every activation and delegates deterministically.
#[derive(::core::fmt::Debug)]
struct RecordingWorkers {
    inner: ::global_workers::DeterministicWorkers,
    records: ::std::sync::Arc<::std::sync::Mutex<::std::vec::Vec<ExecutionRecord>>>,
}

/// Script for the scripted semantic critic.
#[derive(::core::fmt::Debug, ::core::clone::Clone)]
enum CriticScript {
    AlwaysPass,
    RejectLocalOnce {
        phase: ::global_fsm::MacroState,
        marker: ::std::string::String,
    },
    RegressOnce {
        phase: ::global_fsm::MacroState,
        marker: ::std::string::String,
    },
}

/// Semantic critic driven by a fixed script with an audit trail.
#[derive(::core::fmt::Debug)]
struct ScriptedCritic {
    script: crate::CriticScript,
    fired: ::std::sync::Arc<::std::sync::Mutex<::core::primitive::bool>>,
    audits: ::std::sync::Arc<::std::sync::Mutex<AuditLog>>,
}

#[derive(::core::fmt::Debug)]
struct RejectingArbiter;

/// Solver that records every dispatch and delegates to an inner solver.
#[derive(::core::fmt::Debug)]
struct RecordingSolver {
    inner: ::std::boxed::Box<dyn ::global_engine::ModuleSolver>,
    solves: ::std::sync::Arc<::std::sync::Mutex<::std::vec::Vec<::std::string::String>>>,
}

/// Solver that reports one undocumented cycle from module b, then hardens.
#[derive(::core::fmt::Debug)]
struct CycleThenHardenSolver {
    cycle_fired: ::core::primitive::bool,
}

/// Solver that reports one contract conflict against parent a, then hardens.
#[derive(::core::fmt::Debug)]
struct ConflictThenHardenSolver {
    conflicts_fired: ::core::primitive::usize,
}

/// Solver that reports an undocumented cycle on every dispatch.
#[derive(::core::fmt::Debug)]
struct AlwaysCycleSolver;

/// Build a shared empty recording worker log handle.
fn recording_workers(
) -> ::std::sync::Arc<::std::sync::Mutex<::std::vec::Vec<ExecutionRecord>>> {
    let empty_log = ::std::vec::Vec::new();
    let mutex_log = ::std::sync::Mutex::new(empty_log);
    let shared = ::std::sync::Arc::new(mutex_log);
    shared
}

fn recording_crew(
    records: ::std::sync::Arc<::std::sync::Mutex<::std::vec::Vec<ExecutionRecord>>>,
) -> ::std::boxed::Box<dyn ::global_workers::PhaseWorker> {
    let inner = ::global_workers::DeterministicWorkers::new();
    let crew = crate::RecordingWorkers { inner, records };
    let boxed = ::std::boxed::Box::new(crew);
    let trait_object: ::std::boxed::Box<dyn ::global_workers::PhaseWorker> = boxed;
    trait_object
}

fn pass_record() -> ::global_arbiter::CritiqueRecord {
    ::global_arbiter::CritiqueRecord {
        verdict: ::global_arbiter::CritiqueVerdict::Pass,
        violated_invariants: ::std::vec::Vec::new(),
        remediation_notes: ::std::string::String::new(),
    }
}

fn reject_record(
    note: &::core::primitive::str,
    verdict: ::global_arbiter::CritiqueVerdict,
) -> ::global_arbiter::CritiqueRecord {
    ::global_arbiter::CritiqueRecord {
        verdict,
        violated_invariants: ::std::vec::Vec::new(),

        remediation_notes: { note.to_string() },
    }
}

fn scripted_critic(
    script: crate::CriticScript,
) -> (
    ::std::boxed::Box<dyn ::global_arbiter::SemanticCritic>,
    ::std::sync::Arc<::std::sync::Mutex<AuditLog>>,
) {
    let empty_log = ::std::vec::Vec::new();
    let mutex_log = ::std::sync::Mutex::new(empty_log);
    let audits = ::std::sync::Arc::new(mutex_log);
    let fired_false = false;
    let fired_mutex = ::std::sync::Mutex::new(fired_false);
    let fired = ::std::sync::Arc::new(fired_mutex);
    let critic = crate::ScriptedCritic {
        script,
        fired: ::core::clone::Clone::clone(&fired),
        audits: ::core::clone::Clone::clone(&audits),
    };
    let boxed = ::std::boxed::Box::new(critic);
    let trait_object: ::std::boxed::Box<dyn ::global_arbiter::SemanticCritic> = boxed;
    (trait_object, audits)
}

fn base_engine(solver: ::std::boxed::Box<dyn ModuleSolver>) -> ::global_engine::MacroEngine {
    ::global_engine::MacroEngine::builder().solver(solver).build()
}

/// Build the three vertex chain board a, b, c with edges a->b and b->c.
fn chain_board() -> ::global_blackboard::GlobalBlackboard {
    let mut board = ::global_blackboard::GlobalBlackboard::default();
    board.inventory.files = ::std::vec!["a".to_string(), "b".to_string(), "c".to_string()];
    board.inventory.harness = ::global_blackboard::HarnessConfig {
        build_cmd: "cargo build".to_string(),
        test_cmd: "cargo test".to_string(),
        legacy_harness_cmd: "legacy test".to_string(),
    };
    board.dag.vertices = ::std::vec!["a".to_string(), "b".to_string(), "c".to_string()];
    board.dag.edges = ::std::vec![
        ("a".to_string(), "b".to_string()),
        ("b".to_string(), "c".to_string()),
    ];
    board.rules.insert("a".to_string(), ::std::vec![rule_card("RA")]);
    board.rules.insert("b".to_string(), ::std::vec![rule_card("RB")]);
    board.rules.insert("c".to_string(), ::std::vec![rule_card("RC")]);
    board.contracts.insert(
        ("a".to_string(), "b".to_string()),
        ::holon_blackboard::EdgeContract {
            parent_id: "a".to_string(),
            interface_spec: "iface_a".to_string(),
            preconditions: ::std::vec::Vec::new(),
        },
    );
    board.contracts.insert(
        ("b".to_string(), "c".to_string()),
        ::holon_blackboard::EdgeContract {
            parent_id: "b".to_string(),
            interface_spec: "iface_b".to_string(),
            preconditions: ::std::vec::Vec::new(),
        },
    );
    board
}

fn rule_card(card_rule_id: &::core::primitive::str) -> ::holon_blackboard::BusinessRuleCard {
    ::holon_blackboard::BusinessRuleCard {
        rule_id: { card_rule_id.to_string() },
        given: "given clause".to_string(),
        when: "when clause".to_string(),
        then: "then clause".to_string(),
        citation: ::std::format!("legacy/{}.c:42", card_rule_id),
    }
}

/// Collect the recorded subjects for one phase in dispatch order.
fn subjects_for_phase(
    records: &[ExecutionRecord],
    phase: ::global_fsm::MacroState,
) -> ::std::vec::Vec<::std::string::String> {
    let mut subjects = ::std::vec::Vec::new();
    for record in records {
        if record.phase == phase {
            let owned = ::core::clone::Clone::clone(&record.subject);
            subjects.push(owned);
        }
    }
    subjects
}

/// Collect the recorded audit batches for one phase.
fn batches_for_phase(
    audits: &[(::global_fsm::MacroState, ::std::vec::Vec<::std::string::String>)],
    phase: ::global_fsm::MacroState,
) -> ::std::vec::Vec<::std::vec::Vec<::std::string::String>> {
    let mut batches = ::std::vec::Vec::new();
    for entry in audits {
        if entry.0 == phase {
            let owned = ::core::clone::Clone::clone(&entry.1);
            batches.push(owned);
        }
    }
    batches
}

fn run_cycle_campaign() -> (
    ::global_engine::MacroEngine,
    ::std::sync::Arc<::std::sync::Mutex<::std::vec::Vec<::std::string::String>>>,
) {
    let empty_log = ::std::vec::Vec::new();
    let mutex_log = ::std::sync::Mutex::new(empty_log);
    let solves = ::std::sync::Arc::new(mutex_log);
    let cycler = crate::CycleThenHardenSolver { cycle_fired: false };
    let recorder = crate::RecordingSolver { inner: ::std::boxed::Box::new(cycler), solves: ::core::clone::Clone::clone(&solves) };
    let boxed = ::std::boxed::Box::new(recorder);
    let mut engine = ::global_engine::MacroEngine::new(boxed);
    let cycle_campaign = cycle_board();
    engine.configure(cycle_campaign);
    let outcome = engine.run();
    ::std::assert!(outcome.is_ok());
    ::std::assert_eq!(
        engine.state_vector().macro_state,
        ::global_fsm::MacroState::Terminated
    );
    (engine, solves)
}

fn cycle_board() -> ::global_blackboard::GlobalBlackboard {
    let mut board = ::global_blackboard::GlobalBlackboard::default();
    board.inventory.files = ::std::vec!["a".to_string(), "b".to_string(), "d".to_string()];
    board.inventory.harness = ::global_blackboard::HarnessConfig {
        build_cmd: "cargo build".to_string(),
        test_cmd: "cargo test".to_string(),
        legacy_harness_cmd: "legacy test".to_string(),
    };
    board.dag.vertices = ::std::vec!["a".to_string(), "b".to_string(), "d".to_string()];
    board.dag.edges = ::std::vec![("a".to_string(), "b".to_string())];
    board.contracts.insert(
        ("a".to_string(), "b".to_string()),
        ::holon_blackboard::EdgeContract {
            parent_id: "a".to_string(),
            interface_spec: "iface".to_string(),
            preconditions: ::std::vec::Vec::new(),
        },
    );
    board
}

/// Collect the positions of one macro state in the traversal history.
fn positions_of(
    history: &[::global_fsm::MacroState],
    phase: ::global_fsm::MacroState,
) -> ::std::vec::Vec<::core::primitive::usize> {
    let mut positions = ::std::vec::Vec::new();
    for (position, state) in history.iter().enumerate() {
        if *state == phase {
            positions.push(position);
        }
    }
    positions
}

fn conflict_board() -> ::global_blackboard::GlobalBlackboard {
    let mut board = ::global_blackboard::GlobalBlackboard::default();
    board.inventory.files = ::std::vec!["a".to_string(), "b".to_string()];
    board.inventory.harness = ::global_blackboard::HarnessConfig {
        build_cmd: "cargo build".to_string(),
        test_cmd: "cargo test".to_string(),
        legacy_harness_cmd: "legacy test".to_string(),
    };
    board.dag.vertices = ::std::vec!["a".to_string(), "b".to_string()];
    board.dag.edges = ::std::vec![("a".to_string(), "b".to_string())];
    board.contracts.insert(
        ("a".to_string(), "b".to_string()),
        ::holon_blackboard::EdgeContract {
            parent_id: "a".to_string(),
            interface_spec: "iface".to_string(),
            preconditions: ::std::vec::Vec::new(),
        },
    );
    board
}

impl ::global_workers::PhaseWorker for crate::RecordingWorkers {
    fn execute(
        &self,
        phase: ::global_fsm::MacroState,
        shard_subject: &::core::primitive::str,
        envelope: &::global_workers::TaskActivationEnvelope,
        board: &::global_blackboard::GlobalBlackboard,
    ) -> ::global_workers::PhaseWorkOutput {
        let mut queued_events = ::std::vec::Vec::new();
        for event in &board.state.event_queue {
            queued_events.push(::std::format!("{event:?}"));
        }
        let record = crate::ExecutionRecord {
            phase: { phase },
            subject: { shard_subject.to_string() },
            mode: { envelope.mode },
            directive: ::core::clone::Clone::clone(&envelope.delta_directive),
            slice_summary: ::core::clone::Clone::clone(&envelope.slice_summary),
            bound_tools: ::core::clone::Clone::clone(&envelope.bound_tools),
            queued_events,
        };
        self.records.lock().expect("records lock").push(record);
        self.inner.execute(phase, shard_subject, envelope, board)
    }
}

impl ::global_arbiter::SemanticCritic for crate::ScriptedCritic {
    fn audit_batch(
        &self,
        phase: ::global_fsm::MacroState,
        batch: &[::std::string::String],
    ) -> ::global_arbiter::CritiqueRecord {
        let mut owned_batch = ::std::vec::Vec::new();
        owned_batch.extend_from_slice(batch);
        self.audits
            .lock()
            .expect("critic lock")
            .push((phase, owned_batch));
        let phase_gate = match &self.script {
            crate::CriticScript::AlwaysPass => ::core::option::Option::None,
            crate::CriticScript::RejectLocalOnce { phase, .. } => ::core::option::Option::Some(*phase),
            crate::CriticScript::RegressOnce { phase, .. } => ::core::option::Option::Some(*phase),
        };
        let gate_open = match phase_gate {
            ::core::option::Option::Some(gate) => gate == phase,
            ::core::option::Option::None => true,
        };
        let marker_hit = match &self.script {
            crate::CriticScript::AlwaysPass => ::core::option::Option::None,
            crate::CriticScript::RejectLocalOnce { marker, .. } => {
                let hits = batch.iter().any(|item| item.contains(marker.as_str()));
                if hits {
                    ::core::option::Option::Some("fix module b")
                } else {
                    ::core::option::Option::None
                }
            }
            crate::CriticScript::RegressOnce { marker, .. } => {
                let hits = batch.iter().any(|item| item.contains(marker.as_str()));
                if hits {
                    ::core::option::Option::Some("upstream interface drift")
                } else {
                    ::core::option::Option::None
                }
            }
        };
        let verdict_of = match &self.script {
            crate::CriticScript::RejectLocalOnce { .. } => ::core::option::Option::Some("local"),
            crate::CriticScript::RegressOnce { .. } => ::core::option::Option::Some("regress"),
            crate::CriticScript::AlwaysPass => ::core::option::Option::None,
        };
        let scripted_hit = match (gate_open, marker_hit, verdict_of) {
            (true, ::core::option::Option::Some(note), ::core::option::Option::Some("local")) => {
                let verdict = ::global_arbiter::CritiqueVerdict::RemediateLocal(::std::vec![note.to_string()]);
                ::core::option::Option::Some(verdict)
            }
            (true, ::core::option::Option::Some(note), _) => {
                let verdict = ::global_arbiter::CritiqueVerdict::RegressUpstream(::std::vec![note.to_string()]);
                ::core::option::Option::Some(verdict)
            }
            _ => ::core::option::Option::None,
        };
        match scripted_hit {
            ::core::option::Option::Some(verdict) => {
                let mut fired = self.fired.lock().expect("script fired lock");
                let already = *fired;
                if !already {
                    *fired = true;
                }
                drop(fired);
                if already {
                    return pass_record();
                }
                let note = match &verdict {
                    ::global_arbiter::CritiqueVerdict::RemediateLocal(_) => "fix module b",
                    _ => "upstream interface drift",
                };
                reject_record(note, verdict)
            }
            ::core::option::Option::None => pass_record(),
        }
    }
}

impl ::global_arbiter::Arbiter for crate::RejectingArbiter {
    fn adjudicate(
        &self,
        _subject: &::core::primitive::str,
        _trace: &::core::primitive::str,
    ) -> ::core::primitive::bool {
        false
    }
}

impl ::global_engine::ModuleSolver for crate::RecordingSolver {
    fn solve(
        &mut self,
        module_id: &::core::primitive::str,
        problem_model: &::holon_blackboard::ProblemModelSlice,
        mode: ::holon_fsm::ActivationMode,
    ) -> ::global_engine::SolverOutcome {
        let owned = module_id.to_string();
        self.solves.lock().expect("solver log lock").push(owned);
        self.inner.solve(module_id, problem_model, mode)
    }
}

impl ::global_engine::ModuleSolver for crate::CycleThenHardenSolver {
    fn solve(
        &mut self,
        module_id: &::core::primitive::str,
        problem_model: &::holon_blackboard::ProblemModelSlice,
        mode: ::holon_fsm::ActivationMode,
    ) -> ::global_engine::SolverOutcome {
        let module_is_b = module_id == "b";
        let hydrated = !problem_model.parent_contracts.is_empty();
        if module_is_b && hydrated && !self.cycle_fired {
            self.cycle_fired = true;
            return ::global_engine::SolverOutcome::CycleFound {
                cycle_edges: ::std::vec![("b".to_string(), "a".to_string())],
                proof_trace: "undocumented cycle between a and b".to_string(),
            };
        }
        let mut stub = ::global_engine::StubModuleSolver::default();
        stub.solve(module_id, problem_model, mode)
    }
}

impl ::global_engine::ModuleSolver for crate::ConflictThenHardenSolver {
    fn solve(
        &mut self,
        module_id: &::core::primitive::str,
        problem_model: &::holon_blackboard::ProblemModelSlice,
        mode: ::holon_fsm::ActivationMode,
    ) -> ::global_engine::SolverOutcome {
        let module_is_b = module_id == "b";
        let warm_mode = ::core::matches!(mode, ::holon_fsm::ActivationMode::WarmStartRegression);
        let hydrated = !problem_model.parent_contracts.is_empty();
        let fires = module_is_b && !warm_mode && hydrated && self.conflicts_fired == 0;
        if fires {
            self.conflicts_fired += 1;
            return ::global_engine::SolverOutcome::ContractConflict {
                parent_id: "a".to_string(),
                delta_spec: "delta_from_b".to_string(),
                proof_trace: "trace names b and parent a".to_string(),
            };
        }
        let mut stub = ::global_engine::StubModuleSolver::default();
        stub.solve(module_id, problem_model, mode)
    }
}

impl ::global_engine::ModuleSolver for crate::AlwaysCycleSolver {
    fn solve(
        &mut self,
        _module_id: &::core::primitive::str,
        _problem_model: &::holon_blackboard::ProblemModelSlice,
        _mode: ::holon_fsm::ActivationMode,
    ) -> ::global_engine::SolverOutcome {
        ::global_engine::SolverOutcome::CycleFound {
            cycle_edges: ::std::vec![("a".to_string(), "b".to_string())],
            proof_trace: "cycle".to_string(),
        }
    }
}
