//! Interactive validation contracts.
//! Contracts derived from docs/specs/validation-contracts.md and they outrank implementation tests.
//! Covered assertion IDs follow. VAL-INTERACTIVE-01, VAL-INTERACTIVE-02.

/// Contract VAL-INTERACTIVE-01: the TUI renders campaign state from the same
/// persisted state vector the runner records (single source of truth). The
/// telemetry helper derives its text from the state vector, and the cockpit
/// records runner events into that one vector with no duplicate state source.
#[::tokio::test]
async fn contract_val_interactive_01_state_vector_single_source() {
    let streams = ::interactive::default_specialist_streams();
    let mut cockpit = ::interactive::InteractiveCockpit::new(streams);
    // The cockpit starts from the default vector: S0 with an empty event log.
    ::std::assert!(::core::matches!(
        cockpit.state_vector.macro_state,
        ::interactive::mock::ModernizationState::S0IntentPreflight
    ));
    ::std::assert!(cockpit.state_vector.event_log.is_empty());
    // The runner emits a transition. The cockpit consumes the stream event and
    // records it in the one persisted state vector.
    let _ = cockpit.event_tx.send(::interactive::UiStreamEvent::StateTransition {
        from: ::interactive::mock::ModernizationState::S0IntentPreflight,
        to: ::interactive::mock::ModernizationState::S1Assess,
    });
    cockpit.drain_events();
    ::std::assert!(::core::matches!(
        cockpit.state_vector.macro_state,
        ::interactive::mock::ModernizationState::S1Assess
    ));
    ::std::assert_eq!(cockpit.state_vector.event_log.len(), 1);
    // The telemetry helper renders from the same vector. The rendered text
    // names the current macro state and the event count, so the UI and the
    // runner share one source of truth.
    let text = ::interactive::telemetry_state_text(&cockpit.state_vector);
    ::std::assert!(text.contains("S1 - Assess."));
    ::std::assert!(text.contains("events: 1"));
    // A second transition appends in FIFO order on the same vector.
    let _ = cockpit.event_tx.send(::interactive::UiStreamEvent::StateTransition {
        from: ::interactive::mock::ModernizationState::S1Assess,
        to: ::interactive::mock::ModernizationState::S4BriefContracts,
    });
    cockpit.drain_events();
    ::std::assert_eq!(cockpit.state_vector.event_log.len(), 2);
    let text_two = ::interactive::telemetry_state_text(&cockpit.state_vector);
    ::std::assert!(text_two.contains("S4 - Brief Contracts."));
    ::std::assert!(text_two.contains("events: 2"));
}

/// Contract VAL-INTERACTIVE-02: the mock runner drives the same state machine
/// the real runner uses, with a deterministic sequence of states and events.
/// Identical inputs produce identical state vectors.
#[test]
fn contract_val_interactive_02_mock_runner_matches_engine_sequence() {
    // Full-auto deterministic sequence across the complete state chain.
    let full_auto_config = ::interactive::mock::ModernizeConfig {
        execution_mode: ::interactive::mock::ExecutionMode::FullAuto,
        ..::core::default::Default::default()
    };
    let mut auto_vector = ::interactive::mock::RunnerStateVector::default();
    let runner = ::interactive::mock::StateMachineRunner::new(full_auto_config);
    let (assess_runner, first_outcome) = runner.step();
    auto_vector.record(&first_outcome);
    let (approved_runner, second_outcome) = assess_runner.advance_auto();
    auto_vector.record(&second_outcome);
    let (transform_runner, third_outcome) = approved_runner.step();
    auto_vector.record(&third_outcome);
    let (verify_runner, fourth_outcome) = transform_runner.step();
    auto_vector.record(&fourth_outcome);
    let completed = verify_runner.step();
    auto_vector.record(&completed);
    // The deterministic full-auto chain visits each macro phase in order.
    let expected_chain = [
        ::interactive::mock::ModernizationState::S1Assess,
        ::interactive::mock::ModernizationState::S4BriefContracts,
        ::interactive::mock::ModernizationState::S5IterativeDag,
        ::interactive::mock::ModernizationState::S6SystemVerify,
    ];
    let mut chain_iter = expected_chain.iter();
    for event in &auto_vector.event_log {
        if let ::interactive::mock::RunnerEvent::Transition { to, .. } = event {
            let expected = chain_iter.next();
            ::std::assert_eq!(
                ::core::option::Option::Some(to),
                expected,
                "the mock runner visits macro phases in the deterministic order"
            );
        }
    }
    ::std::assert!(chain_iter.next().is_none(), "every phase transition fires");
    // The completion event records the verified receipt stamps.
    let completed_event = auto_vector.event_log.last().expect("final event");
    ::std::assert!(::core::matches!(
        completed_event,
        ::interactive::mock::RunnerEvent::Completed { receipt_count: 1 }
    ));
    // Hitl mode parks on the reimagine gate, then follows the same tail.
    let hitl_config = ::interactive::mock::ModernizeConfig::default();
    let runner_hitl = ::interactive::mock::StateMachineRunner::new(hitl_config);
    let (assess_hitl, first_hitl) = runner_hitl.step();
    let mut hitl_vector = ::interactive::mock::RunnerStateVector::default();
    hitl_vector.record(&first_hitl);
    let gate_outcome = assess_hitl.park_on_gate();
    hitl_vector.record(&gate_outcome);
    let parked_gate = ::core::matches!(
        gate_outcome,
        ::interactive::mock::StepOutcome::GateRequired {
            gate: ::interactive::mock::HumanGate::GateOnReimagine,
            ..
        }
    );
    ::std::assert!(parked_gate, "hitl mode parks on the reimagine gate");
    let decision = ::interactive::mock::GateDecision::Approved {
        approver: "operator".to_string(),
        notes: "contract".to_string(),
    };
    let (approved_hitl, approved_hitl_outcome) = assess_hitl
        .submit_gate_decision(decision)
        .expect("approval consumes the machine");
    hitl_vector.record(&approved_hitl_outcome);
    let (transform_hitl, hitl_second) = approved_hitl.step();
    hitl_vector.record(&hitl_second);
    let (verify_hitl, hitl_third) = transform_hitl.step();
    hitl_vector.record(&hitl_third);
    let hitl_completed = verify_hitl.step();
    hitl_vector.record(&hitl_completed);
    // The gate path and the auto path converge on the same state sequence
    // after the gate: the mock drives one shared state machine.
    let hitl_tail = [
        ::interactive::mock::ModernizationState::S1Assess,
        ::interactive::mock::ModernizationState::S4BriefContracts,
        ::interactive::mock::ModernizationState::S5IterativeDag,
        ::interactive::mock::ModernizationState::S6SystemVerify,
    ];
    let hitl_transitions = hitl_vector
        .event_log
        .iter()
        .filter_map(|event| match event {
            ::interactive::mock::RunnerEvent::Transition { to, .. } => {
                let copied = ::core::clone::Clone::clone(to);
                ::core::option::Option::Some(copied)
            },
            _ => ::core::option::Option::None,
        })
        .collect::<::std::vec::Vec<_>>();
    // The gate path now carries the S1 to S4 approval transition, so the
    // transition list equals the auto path sequence exactly.
    ::std::assert_eq!(hitl_transitions, &hitl_tail[..]);
    // Determinism: replaying the same inputs yields the identical vector.
    let replay_config = ::interactive::mock::ModernizeConfig::default();
    let replay_runner = ::interactive::mock::StateMachineRunner::new(replay_config);
    let (replay_assess, replay_first) = replay_runner.step();
    let mut replay_vector = ::interactive::mock::RunnerStateVector::default();
    replay_vector.record(&replay_first);
    let replay_gate = replay_assess.park_on_gate();
    replay_vector.record(&replay_gate);
    let replay_decision = ::interactive::mock::GateDecision::Approved {
        approver: "operator".to_string(),
        notes: "contract".to_string(),
    };
    let (replay_approved, replay_approved_outcome) = replay_assess
        .submit_gate_decision(replay_decision)
        .expect("approval consumes the machine");
    replay_vector.record(&replay_approved_outcome);
    let (replay_transform, replay_second) = replay_approved.step();
    replay_vector.record(&replay_second);
    let (replay_verify, replay_third) = replay_transform.step();
    replay_vector.record(&replay_third);
    let replay_completed = replay_verify.step();
    replay_vector.record(&replay_completed);
    ::std::assert_eq!(replay_vector, hitl_vector);
}
