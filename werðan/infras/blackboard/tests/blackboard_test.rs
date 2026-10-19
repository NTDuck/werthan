//! Integration and verification tests for blackboard partitions, broadcast events,
//! cryptographic audit chain tamper evidence, and role-based authorization.

#[::tokio::test]
async fn test_partition_manipulation() {
    let bb = ::blackboard::Blackboard::new();

    // 1. ObligationLedgerPartition
    let candidate = self::create_obligation_candidate("parse_packet", "0x00401000");

    let root_role = ::operative::AgentRole::RootOrchestrator;
    let add_cand_res = bb.add_obligation_candidate(root_role, candidate).await;
    let cand_ok = add_cand_res.is_ok();
    ::core::assert!(cand_ok);

    let initial_cov = bb.obligation_coverage().await;
    ::core::assert_eq!(initial_cov, 0.0);

    let arc_data = bb.data();
    {
        let mut guard = arc_data.write().await;
        let linked = guard.obligation_ledger.link_candidate(
            "parse_packet",
            "modernized::net::parse_packet",
            "tests::test_parse_packet",
        );
        ::core::assert!(linked);
        let all_linked = guard.obligation_ledger.is_100_percent_linked();
        ::core::assert!(all_linked);
    }

    let final_cov = bb.obligation_coverage().await;
    ::core::assert_eq!(final_cov, 1.0);

    // 2. BusinessRulesPartition
    let card1 = self::create_clean_card(
        "RULE-001",
        "Checksum Validation",
        "Given packet header, when verified, then accept",
        "src/net.c:100#ABCD",
    );

    let rules_role = ::operative::AgentRole::BusinessRulesExtractor;
    let add_card_res = bb.add_rule_card(rules_role, card1).await;
    let card_ok = add_card_res.is_ok();
    ::core::assert!(card_ok);

    let card2_ghidra = self::create_clean_card(
        "RULE-002",
        "Crypto Dispatch",
        "Given key, when rotated, then notify",
        "ghidra:0x00402000",
    );

    let add_card2_res = bb.add_rule_card(rules_role, card2_ghidra).await;
    let card2_ok = add_card2_res.is_ok();
    ::core::assert!(card2_ok);

    // Invalid citation anchor rejection
    let card_invalid = self::create_clean_card(
        "RULE-003",
        "Invalid Citation",
        "Given defect, when bad anchor, then reject",
        "bad_anchor_without_colon",
    );

    let invalid_res = bb.add_rule_card(rules_role, card_invalid).await;
    let is_invalid_err = invalid_res.is_err();
    ::core::assert!(is_invalid_err);

    // Deduplication check: re-adding RULE-001 does not fail, but does not duplicate
    let duplicate_card =
        self::create_clean_card("RULE-001", "Checksum Validation Duplicate", "Duplicate", "src/net.c:100#ABCD");

    let dup_res = bb.add_rule_card(rules_role, duplicate_card).await;
    let dup_ok = dup_res.is_ok();
    ::core::assert!(dup_ok);

    let cards = bb.rule_cards().await;
    let card_count = cards.len();
    ::core::assert_eq!(card_count, 2);

    // 3. DomainClustersPartition
    {
        let mut guard = arc_data.write().await;
        let cluster1 = ::blackboard::DomainCluster::builder()
            .id("CLUSTER-CORE")
            .name("Core Networking")
            .modules(::std::vec!["mod_socket".to_string(), "mod_packet".to_string()])
            .tech_debt_score(8.5)
            .dag_wave(0)
            .build();

        let cluster2 = ::blackboard::DomainCluster::builder()
            .id("CLUSTER-CRYPTO")
            .name("Crypto & TLS")
            .modules(::std::vec!["mod_tls".to_string()])
            .tech_debt_score(4.2)
            .dag_wave(1)
            .build();

        guard.domain_clusters.add_cluster(cluster1);
        guard.domain_clusters.add_cluster(cluster2);

        let total_clusters = guard.domain_clusters.total_clusters();
        ::core::assert_eq!(total_clusters, 2);

        let ranked = guard.domain_clusters.ranked_by_tech_debt();
        let top_cluster_id = &ranked[0].id;
        ::core::assert_eq!(top_cluster_id, "CLUSTER-CORE");

        let wave_groups = guard.domain_clusters.clusters_by_wave();
        let wave_count = wave_groups.len();
        ::core::assert_eq!(wave_count, 2);
    }

    // 4. ModuleWorktreesPartition
    {
        let mut guard = arc_data.write().await;
        let worktree = ::blackboard::ModuleWorktreeEntry::builder()
            .module_name("mod_socket")
            .worktree_path("/tmp/worktrees/mod_socket")
            .status(::blackboard::WorktreeStatus::Cloned)
            .todo_list(::std::vec!["pin_tests".to_string()])
            .retry_count(0)
            .max_retries(3)
            .build();
        guard.module_worktrees.add_worktree(worktree);
    }

    let lead_role = ::operative::AgentRole::ModuleLead;
    let update_status_res =
        bb.update_worktree_status(lead_role, "mod_socket", ::blackboard::WorktreeStatus::PinningTests).await;
    let update_status_ok = update_status_res.is_ok();
    ::core::assert!(update_status_ok);

    let status_opt = bb.get_worktree_status("mod_socket").await;
    let status_val = status_opt.unwrap();
    ::core::assert_eq!(status_val, ::blackboard::WorktreeStatus::PinningTests);

    let new_todos = ::std::vec!["translate_c".to_string(), "compile_rust".to_string()];
    let update_todos_res = bb.update_worktree_todo(lead_role, "mod_socket", new_todos).await;
    let update_todos_ok = update_todos_res.is_ok();
    ::core::assert!(update_todos_ok);

    {
        let mut guard = arc_data.write().await;
        let retry_res = guard.module_worktrees.increment_retry("mod_socket");
        let retry_ok = retry_res.is_ok();
        ::core::assert!(retry_ok);
        let retry_val = retry_res.unwrap();
        ::core::assert_eq!(retry_val, 1);
    }

    // 5. TaintAndSecurityPartition
    let tainted_init = bb.is_tainted().await;
    ::core::assert!(!tainted_init);

    bb.set_tainted_build_config(true, "build.rs").await;
    let tainted_after = bb.is_tainted().await;
    ::core::assert!(tainted_after);

    {
        let mut guard = arc_data.write().await;
        let cwe = ::blackboard::CweFinding::builder()
            .cwe_id("CWE-119")
            .description("Memory Buffer Overflow")
            .cvss_score(9.8)
            .path("src/net.c")
            .line(105)
            .build();
        guard.taint_and_security.add_cwe_finding(cwe);

        let max_cvss = guard.taint_and_security.max_cvss_score();
        ::core::assert_eq!(max_cvss, 9.8);

        let masked_str = guard.taint_and_security.mask_and_record_secret("Pr0dApiSecretKey123", "config/vault.env");
        ::core::assert_eq!(masked_str, "Pr0d****");

        let masked_records = guard.taint_and_security.masked_secrets();
        let masked_len = masked_records.len();
        ::core::assert_eq!(masked_len, 1);
    }

    // 6. Summary Telemetry
    let telemetry = bb.summary_telemetry().await;
    ::core::assert_eq!(telemetry.total_obligations, 1);
    ::core::assert_eq!(telemetry.verified_obligations, 1);
    ::core::assert_eq!(telemetry.obligation_coverage, 1.0);
    ::core::assert_eq!(telemetry.total_rule_cards, 2);
    ::core::assert_eq!(telemetry.total_clusters, 2);
    ::core::assert_eq!(telemetry.worktree_count, 1);
    ::core::assert!(telemetry.is_tainted);
    ::core::assert_eq!(telemetry.max_cvss, 9.8);
    let audit_count = telemetry.audit_entries_count;
    ::core::assert!(audit_count >= 5);
}

#[::tokio::test]
async fn test_event_broadcast_reception() {
    let bb = ::blackboard::Blackboard::new();
    let mut rx = bb.subscribe();

    // Trigger state change
    bb.record_state_change("S0Intent", "S1Discover").await;

    // Trigger gate encountered
    bb.record_gate_encountered("Gate1", "Preflight intent confirmation").await;

    // Trigger rule card added
    let card = self::create_clean_card(
        "RULE-NET-1",
        "Network Handshake",
        "Given SYN, when received, then send SYN-ACK",
        "src/handshake.c:50#1234",
    );
    let rules_role = ::operative::AgentRole::BusinessRulesExtractor;
    let add_res = bb.add_rule_card(rules_role, card).await;
    ::core::assert!(add_res.is_ok());

    // Trigger obligation candidate added
    let candidate = self::create_obligation_candidate("handshake_init", "0x00403000");
    let historian_role = ::operative::AgentRole::BinaryHistorian;
    let add_cand_res = bb.add_obligation_candidate(historian_role, candidate).await;
    ::core::assert!(add_cand_res.is_ok());

    // Trigger taint detection
    bb.set_tainted_build_config(true, "Makefile").await;

    // Trigger receipt stamp
    let stamp = ::protocol::ReceiptStamp::builder()
        .id("STAMP-01")
        .gate(::protocol::HumanGate::Gate1Preflight)
        .status(::protocol::GateStatus::Approved)
        .approver("ModernizationArchitect")
        .notes("Approved intent and scope")
        .signature_hash("sig_abc_123")
        .build();
    bb.record_receipt_stamp(stamp).await;

    let mut received_state_changed = false;
    let mut received_gate_encountered = false;
    let mut received_rule_card = false;
    let mut received_obligation = false;
    let mut received_taint = false;
    let mut received_gate_decided = false;
    let mut received_audit = false;

    // Drain events from subscriber channel
    while let ::core::result::Result::Ok(event) = rx.try_recv() {
        match event {
            ::blackboard::BlackboardEvent::StateChanged {
                from,
                to,
            } => {
                if from == "S0Intent" && to == "S1Discover" {
                    received_state_changed = true;
                }
            },
            ::blackboard::BlackboardEvent::GateEncountered {
                gate,
                ..
            } => {
                if gate == "Gate1" {
                    received_gate_encountered = true;
                }
            },
            ::blackboard::BlackboardEvent::RuleCardAdded {
                id,
                ..
            } => {
                if id == "RULE-NET-1" {
                    received_rule_card = true;
                }
            },
            ::blackboard::BlackboardEvent::ObligationAdded {
                name,
                ..
            } => {
                if name == "handshake_init" {
                    received_obligation = true;
                }
            },
            ::blackboard::BlackboardEvent::TaintDetected {
                path,
                ..
            } => {
                if path == "Makefile" {
                    received_taint = true;
                }
            },
            ::blackboard::BlackboardEvent::GateDecided {
                approver,
                ..
            } => {
                if approver == "ModernizationArchitect" {
                    received_gate_decided = true;
                }
            },
            ::blackboard::BlackboardEvent::AuditEntryAppended {
                ..
            } => {
                received_audit = true;
            },
            _ => {},
        }
    }

    ::core::assert!(received_state_changed);
    ::core::assert!(received_gate_encountered);
    ::core::assert!(received_rule_card);
    ::core::assert!(received_obligation);
    ::core::assert!(received_taint);
    ::core::assert!(received_gate_decided);
    ::core::assert!(received_audit);
}

#[::tokio::test]
async fn test_hash_chained_audit_verification() {
    let bb = ::blackboard::Blackboard::new();

    let hash1 = bb.append_audit_event("WorkflowStarted", "init payload").await;
    let hash2 = bb.append_audit_event("GatePassed", "Gate 1 approved").await;
    let hash3 = bb.append_audit_event("WorktreeCreated", "mod_crypto cloned").await;

    let not_empty1 = !hash1.is_empty();
    let not_empty2 = !hash2.is_empty();
    let not_empty3 = !hash3.is_empty();
    ::core::assert!(not_empty1);
    ::core::assert!(not_empty2);
    ::core::assert!(not_empty3);

    let arc_data = bb.data();

    // Verify pristine audit chain integrity
    {
        let guard = arc_data.read().await;
        let integrity_res = guard.audit_ledger.verify_integrity();
        let is_valid = integrity_res.is_ok();
        ::core::assert!(is_valid);

        let entries = guard.audit_ledger.entries();
        let count = entries.len();
        ::core::assert_eq!(count, 3);

        let genesis = ::blackboard::GENESIS_HASH;
        let entry0_prev = &entries[0].prev_hash;
        ::core::assert_eq!(entry0_prev, genesis);

        let entry1_prev = &entries[1].prev_hash;
        let entry0_curr = &entries[0].curr_hash;
        ::core::assert_eq!(entry1_prev, entry0_curr);

        let entry2_prev = &entries[2].prev_hash;
        let entry1_curr = &entries[1].curr_hash;
        ::core::assert_eq!(entry2_prev, entry1_curr);
    }

    // Tamper with payload in entry 1
    {
        let mut guard = arc_data.write().await;
        guard.audit_ledger.entries[1].payload = "ForgedPayloadTampered".to_string();
        let integrity_res = guard.audit_ledger.verify_integrity();
        let is_err = integrity_res.is_err();
        ::core::assert!(is_err);
        let err_msg = integrity_res.unwrap_err();
        let has_tamper_flag = err_msg.contains("tamper detected");
        ::core::assert!(has_tamper_flag);
    }

    // Tamper with prev_hash in entry 2
    {
        let mut guard = arc_data.write().await;
        guard.audit_ledger.entries[2].prev_hash = "deadbeef".to_string();
        let integrity_res = guard.audit_ledger.verify_integrity();
        let is_err = integrity_res.is_err();
        ::core::assert!(is_err);
    }
}

#[::tokio::test]
async fn test_role_based_write_authorization() {
    let bb = ::blackboard::Blackboard::new();

    let card = self::create_clean_card(
        "AUTH-RULE-1",
        "Auth check",
        "Given request, when authorized, then proceed",
        "src/auth.c:20#5678",
    );

    // 1. add_rule_card authorization
    let coder_role = ::operative::AgentRole::TargetCoder;
    let coder_rule_res = bb.add_rule_card(coder_role, card.clone()).await;
    let coder_rule_err = coder_rule_res.is_err();
    ::core::assert!(coder_rule_err);
    let err_msg = coder_rule_res.unwrap_err();
    let has_auth_word = err_msg.contains("not authorized");
    ::core::assert!(has_auth_word);

    let test_eng_role = ::operative::AgentRole::TestEngineer;
    let test_rule_res = bb.add_rule_card(test_eng_role, card.clone()).await;
    let test_rule_err = test_rule_res.is_err();
    ::core::assert!(test_rule_err);

    let sec_auditor_role = ::operative::AgentRole::SecurityAuditor;
    let sec_rule_res = bb.add_rule_card(sec_auditor_role, card.clone()).await;
    let sec_rule_err = sec_rule_res.is_err();
    ::core::assert!(sec_rule_err);

    let extractor_role = ::operative::AgentRole::BusinessRulesExtractor;
    let extractor_res = bb.add_rule_card(extractor_role, card.clone()).await;
    let extractor_ok = extractor_res.is_ok();
    ::core::assert!(extractor_ok);

    // 2. add_obligation_candidate authorization
    let candidate = self::create_obligation_candidate("decrypt_block", "0x00405000");

    let coder_cand_res = bb.add_obligation_candidate(coder_role, candidate.clone()).await;
    let coder_cand_err = coder_cand_res.is_err();
    ::core::assert!(coder_cand_err);

    let lead_role = ::operative::AgentRole::ModuleLead;
    let lead_cand_res = bb.add_obligation_candidate(lead_role, candidate.clone()).await;
    let lead_cand_err = lead_cand_res.is_err();
    ::core::assert!(lead_cand_err);

    let historian_role = ::operative::AgentRole::BinaryHistorian;
    let historian_cand_res = bb.add_obligation_candidate(historian_role, candidate.clone()).await;
    let historian_cand_ok = historian_cand_res.is_ok();
    ::core::assert!(historian_cand_ok);

    // 3. update_worktree_todo authorization
    {
        let arc_data = bb.data();
        let mut guard = arc_data.write().await;
        let worktree = ::blackboard::ModuleWorktreeEntry::builder()
            .module_name("mod_auth")
            .worktree_path("/tmp/worktrees/mod_auth")
            .status(::blackboard::WorktreeStatus::Cloned)
            .todo_list(::std::vec!["initial_step".to_string()])
            .retry_count(0)
            .max_retries(3)
            .build();
        guard.module_worktrees.add_worktree(worktree);
    }

    let todos = ::std::vec!["step1".to_string(), "step2".to_string()];

    let coder_todo_res = bb.update_worktree_todo(coder_role, "mod_auth", todos.clone()).await;
    let coder_todo_err = coder_todo_res.is_err();
    ::core::assert!(coder_todo_err);

    let analyst_role = ::operative::AgentRole::LegacyAnalyst;
    let analyst_todo_res = bb.update_worktree_todo(analyst_role, "mod_auth", todos.clone()).await;
    let analyst_todo_err = analyst_todo_res.is_err();
    ::core::assert!(analyst_todo_err);

    let lead_todo_res = bb.update_worktree_todo(lead_role, "mod_auth", todos).await;
    let lead_todo_ok = lead_todo_res.is_ok();
    ::core::assert!(lead_todo_ok);
}

// -----------------------------------------------------------------------------
// Test helper builders ordered in Clean Code stepdown order
// -----------------------------------------------------------------------------

fn create_clean_card(
    id: &::core::primitive::str,
    title: &::core::primitive::str,
    given_when_then: &::core::primitive::str,
    citation_anchor: &::core::primitive::str,
) -> ::laya::CleanRuleCard {
    let id_str = id.to_string();
    let title_str = title.to_string();
    let gwt_str = given_when_then.to_string();
    let anchor_str = citation_anchor.to_string();
    let evidence_str = ::std::string::String::from("source");
    let verified_str = ::std::string::String::from("verified");
    ::laya::CleanRuleCard::builder()
        .id(id_str)
        .title(title_str)
        .given_when_then(gwt_str)
        .citation_anchor(anchor_str)
        .evidence_kind(evidence_str)
        .verified_anchor(verified_str)
        .build()
}

fn create_obligation_candidate(
    name: &::core::primitive::str,
    address_hex: &::core::primitive::str,
) -> ::rea::ObligationCandidate {
    let name_str = name.to_string();
    let addr_str = address_hex.to_string();
    let bin_str = ::std::string::String::from("bin/legacy.exe");
    ::rea::ObligationCandidate::builder()
        .name(name_str)
        .kind(::rea::ObligationCandidateKind::Function)
        .address_hex(addr_str)
        .binary_path(bin_str)
        .status(::rea::ObligationStatus::Discovered)
        .build()
}
