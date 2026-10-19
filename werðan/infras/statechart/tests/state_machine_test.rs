//! Integration tests for state machine execution, human decision gates, and rejection handling.

#[test]
fn test_state_machine_auto_mode() {
    let campaign_id = "test_auto_campaign";
    let test_dir = "test_auto_mode";
    let config = self::create_test_config(campaign_id, test_dir);
    let blackboard = ::blackboard::Blackboard::new();
    let mut runner = ::statechart::StateMachineRunner::new(config, blackboard, true);

    let initial_state = runner.current_state();
    ::core::assert_eq!(initial_state, ::statechart::ModernizationState::S0Intent);

    let run_res = runner.run_until_gate_or_complete();
    let is_ok = run_res.is_ok();
    ::core::assert!(is_ok);

    let outcome = run_res.expect("Expected successful completion");
    match outcome {
        ::statechart::StepOutcome::Completed {
            receipts,
        } => {
            let count = receipts.len();
            ::core::assert_eq!(count, 6);
            for r in &receipts {
                let is_approved = r.status == ::protocol::GateStatus::Approved;
                ::core::assert!(is_approved);
            }
        },
        _ => ::core::panic!("Expected completed outcome in auto mode"),
    }

    let final_state = runner.current_state();
    ::core::assert_eq!(final_state, ::statechart::ModernizationState::S9Completed);

    let ws_root = ::std::path::Path::new(&runner.config.paths.workspace_root);
    let intent_file = ws_root.join("INTENT.md");
    let has_intent = intent_file.exists();
    ::core::assert!(has_intent);

    let map_file = ws_root.join("MAP.md");
    let has_map = map_file.exists();
    ::core::assert!(has_map);

    let rules_file = ws_root.join("BUSINESS_RULES.md");
    let has_rules = rules_file.exists();
    ::core::assert!(has_rules);

    let brief_file = ws_root.join("BRIEF.md");
    let has_brief = brief_file.exists();
    ::core::assert!(has_brief);

    let worktree_dir = ws_root.join("modernized_worktree_core");
    let has_worktree = worktree_dir.exists();
    ::core::assert!(has_worktree);

    let topology_file = ws_root.join("topology.html");
    let has_topology = topology_file.exists();
    ::core::assert!(has_topology);

    let receipt_file = ws_root.join("AUDIT_RECEIPT.json");
    let has_receipt = receipt_file.exists();
    ::core::assert!(has_receipt);
}

#[test]
fn test_state_machine_hitl_mode_gates() {
    let campaign_id = "test_hitl_campaign";
    let test_dir = "test_hitl_mode";
    let config = self::create_test_config(campaign_id, test_dir);
    let blackboard = ::blackboard::Blackboard::new();
    let mut runner = ::statechart::StateMachineRunner::new(config, blackboard, false);

    // Gate 1: S0 Intent -> S1 Preflight
    let step1_res = runner.step();
    let is_step1_ok = step1_res.is_ok();
    ::core::assert!(is_step1_ok);
    let outcome1 = step1_res.expect("Step 1 outcome");
    match outcome1 {
        ::statechart::StepOutcome::GateRequired {
            gate,
            ..
        } => {
            let is_gate1 = gate == ::statechart::HumanGate::Gate1Intent;
            ::core::assert!(is_gate1);
        },
        _ => ::core::panic!("Expected GateRequired for Gate 1"),
    }
    let pending1 = runner.pending_gate();
    ::core::assert_eq!(pending1, ::core::option::Option::Some(::statechart::HumanGate::Gate1Intent));

    let dec1 = ::statechart::GateDecision::Approved {
        approver: "operator_alex".to_string(),
        notes: "Intent approved".to_string(),
    };
    let submit1_res = runner.submit_gate_decision(dec1);
    let is_submit1_ok = submit1_res.is_ok();
    ::core::assert!(is_submit1_ok);
    let no_pending1 = runner.pending_gate().is_none();
    ::core::assert!(no_pending1);

    // Gate 2: S3 Map -> S4 Extract Rules
    let step2_res = runner.run_until_gate_or_complete();
    let is_step2_ok = step2_res.is_ok();
    ::core::assert!(is_step2_ok);
    let outcome2 = step2_res.expect("Step 2 outcome");
    match outcome2 {
        ::statechart::StepOutcome::GateRequired {
            gate,
            ..
        } => {
            let is_gate2 = gate == ::statechart::HumanGate::Gate2Map;
            ::core::assert!(is_gate2);
        },
        _ => ::core::panic!("Expected GateRequired for Gate 2"),
    }
    let dec2 = ::statechart::GateDecision::Approved {
        approver: "architect_sam".to_string(),
        notes: "DAG waves approved".to_string(),
    };
    let submit2_res = runner.submit_gate_decision(dec2);
    let is_submit2_ok = submit2_res.is_ok();
    ::core::assert!(is_submit2_ok);

    // Gate 3: S4 Extract Rules -> S5 Brief
    let step3_res = runner.run_until_gate_or_complete();
    let is_step3_ok = step3_res.is_ok();
    ::core::assert!(is_step3_ok);
    let outcome3 = step3_res.expect("Step 3 outcome");
    match outcome3 {
        ::statechart::StepOutcome::GateRequired {
            gate,
            ..
        } => {
            let is_gate3 = gate == ::statechart::HumanGate::Gate3ExtractRules;
            ::core::assert!(is_gate3);
        },
        _ => ::core::panic!("Expected GateRequired for Gate 3"),
    }
    let dec3 = ::statechart::GateDecision::Approved {
        approver: "rule_owner_taylor".to_string(),
        notes: "Rules approved".to_string(),
    };
    let submit3_res = runner.submit_gate_decision(dec3);
    let is_submit3_ok = submit3_res.is_ok();
    ::core::assert!(is_submit3_ok);

    // Gate 4: S5 Brief -> S6 Transform
    let step4_res = runner.run_until_gate_or_complete();
    let is_step4_ok = step4_res.is_ok();
    ::core::assert!(is_step4_ok);
    let outcome4 = step4_res.expect("Step 4 outcome");
    match outcome4 {
        ::statechart::StepOutcome::GateRequired {
            gate,
            ..
        } => {
            let is_gate4 = gate == ::statechart::HumanGate::Gate4Brief;
            ::core::assert!(is_gate4);
        },
        _ => ::core::panic!("Expected GateRequired for Gate 4"),
    }
    let dec4 = ::statechart::GateDecision::Approved {
        approver: "tech_lead_jordan".to_string(),
        notes: "Brief approved".to_string(),
    };
    let submit4_res = runner.submit_gate_decision(dec4);
    let is_submit4_ok = submit4_res.is_ok();
    ::core::assert!(is_submit4_ok);

    // Gate 5: S7 Verify -> S8 Harden
    let step5_res = runner.run_until_gate_or_complete();
    let is_step5_ok = step5_res.is_ok();
    ::core::assert!(is_step5_ok);
    let outcome5 = step5_res.expect("Step 5 outcome");
    match outcome5 {
        ::statechart::StepOutcome::GateRequired {
            gate,
            ..
        } => {
            let is_gate5 = gate == ::statechart::HumanGate::Gate5Verify;
            ::core::assert!(is_gate5);
        },
        _ => ::core::panic!("Expected GateRequired for Gate 5"),
    }
    let dec5 = ::statechart::GateDecision::Approved {
        approver: "qa_lead_casey".to_string(),
        notes: "Verification proof approved".to_string(),
    };
    let submit5_res = runner.submit_gate_decision(dec5);
    let is_submit5_ok = submit5_res.is_ok();
    ::core::assert!(is_submit5_ok);

    // Gate 6: S9 Completed -> Finished
    let step6_res = runner.run_until_gate_or_complete();
    let is_step6_ok = step6_res.is_ok();
    ::core::assert!(is_step6_ok);
    let outcome6 = step6_res.expect("Step 6 outcome");
    match outcome6 {
        ::statechart::StepOutcome::GateRequired {
            gate,
            ..
        } => {
            let is_gate6 = gate == ::statechart::HumanGate::Gate6Completed;
            ::core::assert!(is_gate6);
        },
        _ => ::core::panic!("Expected GateRequired for Gate 6"),
    }
    let dec6 = ::statechart::GateDecision::Approved {
        approver: "release_manager_morgan".to_string(),
        notes: "Final release signed off".to_string(),
    };
    let submit6_res = runner.submit_gate_decision(dec6);
    let is_submit6_ok = submit6_res.is_ok();
    ::core::assert!(is_submit6_ok);

    // Final completion step
    let final_step_res = runner.run_until_gate_or_complete();
    let is_final_ok = final_step_res.is_ok();
    ::core::assert!(is_final_ok);
    let final_outcome = final_step_res.expect("Final step outcome");
    match final_outcome {
        ::statechart::StepOutcome::Completed {
            receipts,
        } => {
            let count = receipts.len();
            ::core::assert_eq!(count, 6);
        },
        _ => ::core::panic!("Expected Completed outcome at end of HITL workflow"),
    }

    let final_state = runner.current_state();
    ::core::assert_eq!(final_state, ::statechart::ModernizationState::S9Completed);
}

#[test]
fn test_state_machine_rejection_handling() {
    let campaign_id = "test_rejection_campaign";
    let test_dir = "test_rejection_handling";
    let config = self::create_test_config(campaign_id, test_dir);
    let blackboard = ::blackboard::Blackboard::new();
    let mut runner = ::statechart::StateMachineRunner::new(config, blackboard, false);

    let step_res = runner.step();
    let is_step_ok = step_res.is_ok();
    ::core::assert!(is_step_ok);

    let outcome = step_res.expect("Step outcome");
    match outcome {
        ::statechart::StepOutcome::GateRequired {
            gate,
            ..
        } => {
            let is_gate1 = gate == ::statechart::HumanGate::Gate1Intent;
            ::core::assert!(is_gate1);
        },
        _ => ::core::panic!("Expected GateRequired at Gate 1"),
    }

    let dec_rejection = ::statechart::GateDecision::Rejected {
        reason: "Security budget revoked".to_string(),
    };
    let submit_res = runner.submit_gate_decision(dec_rejection);
    let is_err = submit_res.is_err();
    ::core::assert!(is_err);
    let err_msg = submit_res.unwrap_err();
    let has_reason = err_msg.contains("Security budget revoked");
    ::core::assert!(has_reason);

    // Subsequent step attempt must immediately fail due to halted pipeline
    let next_step_res = runner.step();
    let is_next_err = next_step_res.is_err();
    ::core::assert!(is_next_err);
    let next_err = next_step_res.unwrap_err();
    let halted_msg = next_err.contains("halted due to rejection");
    ::core::assert!(halted_msg);

    let receipts = runner.receipts();
    let receipt_count = receipts.len();
    ::core::assert_eq!(receipt_count, 1);
    let first_receipt = &receipts[0];
    let is_rejected_status = first_receipt.status == ::protocol::GateStatus::Rejected;
    ::core::assert!(is_rejected_status);
}

fn create_test_config(
    campaign_id: &::core::primitive::str,
    test_subfolder: &::core::primitive::str,
) -> ::protocol::ModernizeConfig {
    let base_temp = ::std::env::temp_dir();
    let root = base_temp.join("werthan_state_machine_tests").join(test_subfolder);
    let ws_root = root.join("workspace").to_string_lossy().to_string();
    let legacy_dir = root.join("legacy").to_string_lossy().to_string();

    let paths = ::protocol::ModernizePaths {
        workspace_root: ws_root,
        input_legacy_dir: legacy_dir,
        ..::protocol::ModernizePaths::default()
    };

    let stack = ::protocol::ModernizeStack {
        target_language: "rust".to_string(),
        target_edition_or_framework: "2021".to_string(),
        ..::protocol::ModernizeStack::default()
    };
    ::protocol::ModernizeConfig {
        campaign_id: campaign_id.to_string(),
        paths,
        stack,
        ..::protocol::ModernizeConfig::default()
    }
}
