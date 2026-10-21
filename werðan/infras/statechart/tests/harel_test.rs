//! Integration tests for Harel Statechart Engine, Kahn wave scheduler, and Deep History.

#[test]
fn test_harel_macro_phase_transitions_and_guards() {
    let mut dag = ::blackboard::CondensedDAG::new();
    dag.add_edge("v_auth", "v_gateway");

    let mut engine = ::statechart::HarelEngine::new(dag);
    ::std::assert_eq!(engine.macro_state, ::statechart::HarelMacroState::S0IntentPreflight);

    // Guard failure stops transition
    let fail_res = engine.step_macro_phase(false, true);
    ::std::assert!(fail_res.is_err());
    ::std::assert_eq!(engine.macro_state, ::statechart::HarelMacroState::S0IntentPreflight);

    let sem_fail_res = engine.step_macro_phase(true, false);
    ::std::assert!(sem_fail_res.is_err());
    ::std::assert_eq!(engine.macro_state, ::statechart::HarelMacroState::S0IntentPreflight);

    // S0 -> S1
    let s1 = engine.step_macro_phase(true, true).unwrap();
    ::std::assert_eq!(s1, ::statechart::HarelMacroState::S1Assess);

    // S1 -> S2
    let s2 = engine.step_macro_phase(true, true).unwrap();
    ::std::assert_eq!(s2, ::statechart::HarelMacroState::S2Map);

    // S2 -> S3
    let s3 = engine.step_macro_phase(true, true).unwrap();
    ::std::assert_eq!(s3, ::statechart::HarelMacroState::S3ExtractBusinessRules);

    // S3 -> S4
    let s4 = engine.step_macro_phase(true, true).unwrap();
    ::std::assert_eq!(s4, ::statechart::HarelMacroState::S4BriefReimagineContracts);

    // S4 -> S5
    let s5 = engine.step_macro_phase(true, true).unwrap();
    ::std::assert_eq!(s5, ::statechart::HarelMacroState::S5IterativeDagTransform);
}

#[test]
fn test_kahn_wave_scheduling_and_module_hardening() {
    let mut dag = ::blackboard::CondensedDAG::new();
    // v_core -> v_service -> v_api
    dag.add_edge("v_core", "v_service");
    dag.add_edge("v_service", "v_api");

    let mut engine = ::statechart::HarelEngine::new(dag);
    engine.macro_state = ::statechart::HarelMacroState::S5IterativeDagTransform;

    // Wave 1: Only v_core has in-degree 0 (unblocked)
    let wave1 = engine.compute_ready_set();
    let expected_v_core = "v_core".to_string();
    ::std::assert_eq!(wave1, ::std::vec![expected_v_core.clone()]);

    let dispatched = engine.dispatch_ready_modules();
    ::std::assert_eq!(dispatched, ::std::vec![expected_v_core.clone()]);
    ::std::assert_eq!(
        engine.module_states.get("v_core"),
        ::core::option::Option::Some(&::statechart::ModuleMicroState::M2AriadneMcts)
    );

    // Harden v_core
    let ev_harden_core = ::statechart::HarelEvent::ModuleHardened {
        vertex_id: expected_v_core,
        reward: 1.0,
    };
    let outcome1 = engine.process_event(ev_harden_core).unwrap();
    ::std::assert_eq!(
        outcome1,
        ::statechart::EventOutcome::ModuleHardened {
            vertex_id: "v_core".to_string()
        }
    );

    // Wave 2: Now v_service is unblocked
    let wave2 = engine.compute_ready_set();
    let expected_v_service = "v_service".to_string();
    ::std::assert_eq!(wave2, ::std::vec![expected_v_service.clone()]);

    let _ = engine.dispatch_ready_modules();
    let ev_harden_service = ::statechart::HarelEvent::ModuleHardened {
        vertex_id: expected_v_service,
        reward: 1.0,
    };
    let _ = engine.process_event(ev_harden_service).unwrap();

    // Wave 3: Now v_api is unblocked
    let wave3 = engine.compute_ready_set();
    let expected_v_api = "v_api".to_string();
    ::std::assert_eq!(wave3, ::std::vec![expected_v_api.clone()]);

    let _ = engine.dispatch_ready_modules();
    let ev_harden_api = ::statechart::HarelEvent::ModuleHardened {
        vertex_id: expected_v_api,
        reward: 1.0,
    };
    // Hardening the last module automatically advances macro-state to S6
    let outcome3 = engine.process_event(ev_harden_api).unwrap();
    ::std::assert_eq!(
        outcome3,
        ::statechart::EventOutcome::MacroPhaseAdvanced(::statechart::HarelMacroState::S6SystemVerifyHarden)
    );
    ::std::assert_eq!(engine.macro_state, ::statechart::HarelMacroState::S6SystemVerifyHarden);
}

#[test]
fn test_tier_b_micro_regression_calculus() {
    let mut dag = ::blackboard::CondensedDAG::new();
    dag.add_edge("v_parent", "v_child");

    let mut engine = ::statechart::HarelEngine::new(dag);
    engine.macro_state = ::statechart::HarelMacroState::S5IterativeDagTransform;

    // Harden parent
    let _ = engine.module_states.insert("v_parent".to_string(), ::statechart::ModuleMicroState::M4Hardened);
    // Child is currently in M2
    let _ = engine.module_states.insert("v_child".to_string(), ::statechart::ModuleMicroState::M2AriadneMcts);

    // Child discovers contract conflict on parent
    let ev_conflict = ::statechart::HarelEvent::ContractConflict {
        provider_vertex: "v_parent".to_string(),
        consumer_vertex: "v_child".to_string(),
        delta_spec: "Missing auth token parameter in execute()".to_string(),
        proof_trace: "Compiler trace at child line 42".to_string(),
    };

    let outcome = engine.process_event(ev_conflict).unwrap();
    ::std::assert_eq!(
        outcome,
        ::statechart::EventOutcome::TierBMicroRegression {
            parent_invalidated: "v_parent".to_string(),
            child_blocked: "v_child".to_string(),
        }
    );

    // Child is parked back in M0_Blocked
    ::std::assert_eq!(
        engine.module_states.get("v_child"),
        ::core::option::Option::Some(&::statechart::ModuleMicroState::M0Blocked)
    );
    // Parent is in warm-start M2
    ::std::assert_eq!(
        engine.module_states.get("v_parent"),
        ::core::option::Option::Some(&::statechart::ModuleMicroState::M2AriadneMcts)
    );
}

#[test]
fn test_tier_c_macro_regression_with_deep_history() {
    let mut dag = ::blackboard::CondensedDAG::new();
    dag.add_edge("v_leaf1", "v_consumer");
    dag.add_edge("v_leaf2", "v_consumer");

    let mut engine = ::statechart::HarelEngine::new(dag);
    engine.macro_state = ::statechart::HarelMacroState::S5IterativeDagTransform;

    // v_leaf1 is hardened
    let _ = engine.module_states.insert("v_leaf1".to_string(), ::statechart::ModuleMicroState::M4Hardened);
    let _ = engine.module_states.insert("v_leaf2".to_string(), ::statechart::ModuleMicroState::M2AriadneMcts);

    // Undocumented cycle found between v_leaf2 and v_consumer
    let cycle_edges = ::std::vec![
        ("v_leaf2".to_string(), "v_consumer".to_string()),
        ("v_consumer".to_string(), "v_leaf2".to_string()),
    ];
    let ev_cycle = ::statechart::HarelEvent::CyclicDependencyFound {
        cycle_edges,
        proof_trace: "Dynamic reflection cycle".to_string(),
    };

    let outcome = engine.process_event(ev_cycle).unwrap();
    ::std::assert_eq!(
        outcome,
        ::statechart::EventOutcome::TierCMacroRegression {
            target_macro_state: ::statechart::HarelMacroState::S2Map,
            snapshot_saved: 3,
        }
    );

    // Macro-state regressed to S2
    ::std::assert_eq!(engine.macro_state, ::statechart::HarelMacroState::S2Map);
    ::std::assert!(engine.deep_history_snapshot.is_some());

    // Advance through S2 -> S3 -> S4 -> S5 (resuming via Deep History H*)
    let _ = engine.step_macro_phase(true, true).unwrap(); // S2 -> S3
    let _ = engine.step_macro_phase(true, true).unwrap(); // S3 -> S4
    let _ = engine.step_macro_phase(true, true).unwrap(); // S4 -> S5 (H* restored)

    ::std::assert_eq!(engine.macro_state, ::statechart::HarelMacroState::S5IterativeDagTransform);
    // Unaffected v_leaf1 remained M4_Hardened without re-execution!
    ::std::assert_eq!(
        engine.module_states.get("v_leaf1"),
        ::core::option::Option::Some(&::statechart::ModuleMicroState::M4Hardened)
    );
}
