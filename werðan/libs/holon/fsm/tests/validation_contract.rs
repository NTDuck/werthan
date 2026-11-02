//! Validation contract tests for holon-fsm.
//! Covers VAL-FSM-01, VAL-FSM-02, VAL-FSM-03, VAL-FSM-04, VAL-FSM-05 and VAL-FSM-06.
//! Contracts derive from docs/specs/validation-contracts.md and outrank implementation tests.

const SAMPLE_SUITE_ITEM: &::core::primitive::str = "assert_eq!(execute(), 42)";
const SAMPLE_CODE_ITEM: &::core::primitive::str = "pub fn execute() -> i32 { 42 }";

#[test]
fn contract_val_fsm_01_legal_transition_set_only() {
    // Permitted transitions: M0 to M1, M1 to M2, M2 to M4, M2 to M3,
    // M3 to M0, M4 to M5 and M5 to M2. The typestate API exposes exactly
    // these moves and no other state conversion paths.
    let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("vertex_t1");
    ::std::assert_eq!(machine_m0.state_tag(), ::holon_fsm::MicroStateTag::M0Blocked);
    let machine_m1 = machine_m0.wake_ready();
    ::std::assert_eq!(machine_m1.state_tag(), ::holon_fsm::MicroStateTag::M1ContextHydration);
    let machine_m2 = machine_m1.hydration_completed();
    ::std::assert_eq!(machine_m2.state_tag(), ::holon_fsm::MicroStateTag::M2AriadneMcts);
    let machine_m4 = machine_m2.into_hardened(SAMPLE_CODE_ITEM.to_string(), ::std::vec![SAMPLE_SUITE_ITEM.to_string()], 1.0);
    ::std::assert_eq!(machine_m4.state_tag(), ::holon_fsm::MicroStateTag::M4Hardened);
    let machine_m5 = machine_m4.into_invalidated("delta after invalidation".to_string());
    ::std::assert_eq!(machine_m5.state_tag(), ::holon_fsm::MicroStateTag::M5Invalidated);
    let machine_m2_warm = machine_m5.warm_start_ready();
    ::std::assert_eq!(machine_m2_warm.state_tag(), ::holon_fsm::MicroStateTag::M2AriadneMcts);
    ::std::assert_eq!(
        machine_m2_warm.activation_mode(),
        ::holon_fsm::ActivationMode::WarmStartRegression
    );
    // M2 to M3 and the return M3 to M0 complete the legal set.
    let machine_m0_b = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("vertex_t1b");
    let machine_m2_b = machine_m0_b.wake_ready().hydration_completed();
    let machine_m3 = machine_m2_b.into_contract_negotiate(
        "vertex_parent".to_string(),
        "delta spec".to_string(),
        "proof trace".to_string(),
    );
    ::std::assert_eq!(machine_m3.state_tag(), ::holon_fsm::MicroStateTag::M3ContractNegotiate);
    let machine_m0_back = machine_m3.conflict_dispatched();
    ::std::assert_eq!(machine_m0_back.state_tag(), ::holon_fsm::MicroStateTag::M0Blocked);
}

#[test]
fn contract_val_fsm_02_m3_posts_request_and_parks() {
    // Entering M3 emits the contract patch request to the parent on G. The
    // machine parks into M0_Blocked after conflict dispatch.
    let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("vertex_t2");
    let machine_m2 = machine_m0.wake_ready().hydration_completed();
    let mut machine_m3 = machine_m2.into_contract_negotiate(
        "vertex_parent_t2".to_string(),
        "delta spec t2".to_string(),
        "proof trace t2".to_string(),
    );
    let events = machine_m3.drain_events();
    ::std::assert_eq!(events.len(), 1);
    match &events[0] {
        ::holon_fsm::ClusterBroadcastEvent::ContractConflict {
            parent_id,
            module_id,
            delta_spec,
            proof_trace,
        } => {
            ::std::assert_eq!(parent_id, "vertex_parent_t2");
            ::std::assert_eq!(module_id, "vertex_t2");
            ::std::assert_eq!(delta_spec, "delta spec t2");
            ::std::assert_eq!(proof_trace, "proof trace t2");
        },
        other => ::std::panic!("expected contract conflict event, got {other:?}"),
    }
    // The machine parks back into M0 blocked.
    let parked = machine_m3.conflict_dispatched();
    ::std::assert_eq!(parked.state_tag(), ::holon_fsm::MicroStateTag::M0Blocked);
    ::std::assert!(parked.is_blocked());
}

#[test]
fn contract_val_fsm_03_m4_publication_event() {
    // Entering M4 publishes the target code and verified test suite. It emits
    // ev_ModuleHardened(v_i) for scheduler recomputation.
    let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("vertex_t3");
    let machine_m2 = machine_m0.wake_ready().hydration_completed();
    let mut machine_m4 = machine_m2.into_hardened(SAMPLE_CODE_ITEM.to_string(), ::std::vec![SAMPLE_SUITE_ITEM.to_string()], 1.0);
    ::std::assert!(machine_m4.is_hardened());
    let events = machine_m4.drain_events();
    ::std::assert_eq!(events.len(), 1);
    match &events[0] {
        ::holon_fsm::ClusterBroadcastEvent::ModuleHardened {
            module_id,
            target_code,
            test_suite,
        } => {
            ::std::assert_eq!(module_id, "vertex_t3");
            ::std::assert_eq!(target_code, SAMPLE_CODE_ITEM);
            ::std::assert_eq!(test_suite, &::std::vec![SAMPLE_SUITE_ITEM.to_string()]);
        },
        other => ::std::panic!("expected module hardened event, got {other:?}"),
    }
}

#[test]
fn contract_val_fsm_04_m5_delta_preserving_warm_regression() {
    // Entering M5 appends the invalidation delta while preserving the retained
    // history. The machine re-enters M2 with WARM_START_REGRESSION.
    let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("vertex_t4");
    let machine_m2 = machine_m0.wake_ready().hydration_completed();
    let machine_m4 = machine_m2.into_hardened(SAMPLE_CODE_ITEM.to_string(), ::std::vec![SAMPLE_SUITE_ITEM.to_string()], 1.0);
    let machine_m5 = machine_m4.into_invalidated("new interface constraint".to_string());
    // The delta spec stays in the M5 payload.
    ::std::assert_eq!(machine_m5.state.delta_spec, "new interface constraint");
    // History and the last hardened code survive the invalidation.
    ::std::assert!(machine_m5.history.last_hardened_code.is_some());
    let machine_m2_warm = machine_m5.warm_start_ready();
    ::std::assert_eq!(machine_m2_warm.state_tag(), ::holon_fsm::MicroStateTag::M2AriadneMcts);
    ::std::assert_eq!(
        machine_m2_warm.activation_mode(),
        ::holon_fsm::ActivationMode::WarmStartRegression
    );
}

#[test]
fn contract_val_fsm_05_m0_entry_and_parent_release() {
    // A module starts parked in M0 blocked and re-enters M0 after contract
    // negotiation. The state tag stays M0 until the scheduler releases it.
    let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("vertex_t5");
    ::std::assert!(machine_m0.is_blocked());
    ::std::assert!(!machine_m0.is_active());
    ::std::assert_eq!(machine_m0.activation_mode(), ::holon_fsm::ActivationMode::Initial);
    // Re-entry after negotiation parks in M0 again.
    let machine_m2 = machine_m0.wake_ready().hydration_completed();
    let machine_m3 = machine_m2.into_contract_negotiate(
        "vertex_parent_t5".to_string(),
        "delta".to_string(),
        "trace".to_string(),
    );
    let re_entered = machine_m3.conflict_dispatched();
    ::std::assert_eq!(re_entered.state_tag(), ::holon_fsm::MicroStateTag::M0Blocked);
    ::std::assert!(re_entered.is_blocked());
}

#[test]
fn contract_val_fsm_06_microstate_persistence_roundtrip() {
    // Micro-state reflection tags and the retained history payload survive a
    // JSON save and reload of the machine state.
    let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("vertex_t6");
    let machine_m2 = machine_m0.wake_ready().hydration_completed();
    let machine_m4 = machine_m2.into_hardened(SAMPLE_CODE_ITEM.to_string(), ::std::vec![SAMPLE_SUITE_ITEM.to_string()], 0.95);
    let serialized_history =
        ::serde_json::to_string(&machine_m4.history).expect("history serializes");
    let reloaded_history =
        ::serde_json::from_str::<::holon_fsm::FsmHistory>(&serialized_history).expect("history deserializes");
    ::std::assert_eq!(
        reloaded_history.visited_states,
        ::std::vec![
            ::holon_fsm::MicroStateTag::M0Blocked,
            ::holon_fsm::MicroStateTag::M1ContextHydration,
            ::holon_fsm::MicroStateTag::M2AriadneMcts,
            ::holon_fsm::MicroStateTag::M4Hardened,
        ]
    );
    ::std::assert_eq!(
        reloaded_history.last_hardened_code,
        ::core::option::Option::Some(SAMPLE_CODE_ITEM.to_string())
    );
    // The hardened payload with target code and test suite also round-trips.
    let serialized_state = ::serde_json::to_string(&machine_m4.state).expect("state serializes");
    let reloaded_state = ::serde_json::from_str::<::holon_fsm::M4Hardened>(&serialized_state)
        .expect("state deserializes");
    ::std::assert_eq!(reloaded_state.target_code, SAMPLE_CODE_ITEM);
    ::std::assert_eq!(reloaded_state.test_suite, ::std::vec![SAMPLE_SUITE_ITEM.to_string()]);
    ::std::assert!((reloaded_state.reward - 0.95).abs() < 1e-9);
}
