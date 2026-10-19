//! Unit tests for rea tools and obligation ledger.

#[test]
fn test_cocomo_calculations() {
    let ksloc = 10.0;
    let pm = ::rea::calculate_cocomo_ii(ksloc);
    let pm_valid = pm > 30.0 && pm < 40.0;
    ::std::assert!(pm_valid);

    let dur = ::rea::calculate_cocomo_duration(pm);
    let dur_valid = dur > 10.0 && dur < 15.0;
    ::std::assert!(dur_valid);
}

#[test]
fn test_obligation_ledger_candidate_seeding() {
    let dll_path = ::std::path::Path::new("tests/fixtures/engine.dll");
    let seed_res = ::rea::seed_obligation_ledger(dll_path);
    let seed_ok = seed_res.is_ok();
    ::std::assert!(seed_ok);

    let candidates = seed_res.unwrap();
    let count = candidates.len();
    ::std::assert_eq!(count, 4);

    let has_func = candidates.iter().any(|c| c.kind == ::rea::ObligationCandidateKind::Function);
    let has_export = candidates.iter().any(|c| c.kind == ::rea::ObligationCandidateKind::Export);
    let has_ipc = candidates.iter().any(|c| c.kind == ::rea::ObligationCandidateKind::IpcChannel);
    let has_struct = candidates.iter().any(|c| c.kind == ::rea::ObligationCandidateKind::Struct);

    ::std::assert!(has_func);
    ::std::assert!(has_export);
    ::std::assert!(has_ipc);
    ::std::assert!(has_struct);

    for cand in &candidates {
        let is_disc = cand.status == ::rea::ObligationStatus::Discovered;
        ::std::assert!(is_disc);
        let addr_empty = cand.address_hex.is_empty();
        ::std::assert!(!addr_empty);
    }

    let eval_res = ::rea::evaluate_reconstruction_obligation_ledger(dll_path);
    let eval_ok = eval_res.is_ok();
    ::std::assert!(eval_ok);

    let ledger = eval_res.unwrap();
    let ledger_count = ledger.candidates.len();
    ::std::assert_eq!(ledger_count, 4);
}

#[test]
fn test_verification_gate_100_percent_coverage() {
    let so_path = ::std::path::Path::new("tests/fixtures/core.so");
    let eval_res = ::rea::evaluate_reconstruction_obligation_ledger(so_path);
    let eval_ok = eval_res.is_ok();
    ::std::assert!(eval_ok);
    let mut ledger = eval_res.unwrap();

    let unlinked_res = ::rea::verify_obligation_ledger(&ledger);
    let unlinked_is_err = unlinked_res.is_err();
    ::std::assert!(unlinked_is_err);

    let err_msg = unlinked_res.unwrap_err();
    let err_has_gate = err_msg.contains("Hard verification gate failed");
    let err_has_unlinked = err_msg.contains("Unlinked candidates");
    ::std::assert!(err_has_gate);
    ::std::assert!(err_has_unlinked);

    for candidate in &mut ledger.candidates {
        let cand_name = &candidate.name;
        let target_sym = ::std::format!("modernized_{cand_name}");
        let char_test = ::std::format!("test_{cand_name}");
        candidate.link_implementation(&target_sym, &char_test);
    }

    let linked_res = ::rea::verify_obligation_ledger(&ledger);
    let linked_is_ok = linked_res.is_ok();
    ::std::assert!(linked_is_ok);

    let report = linked_res.unwrap();
    ::std::assert_eq!(report.total_candidates, 4);
    ::std::assert_eq!(report.verified_candidates, 4);
    let ratio_ok = report.coverage_ratio >= 1.0;
    ::std::assert!(ratio_ok);
    let empty_unlinked = report.unlinked_candidates.is_empty();
    ::std::assert!(empty_unlinked);
}

#[test]
fn test_role_isolation() {
    let bh_decomp = ::rea::check_role_access("binary-historian", "rea_decompile_function");
    let bh_decomp_ok = bh_decomp.is_ok();
    ::std::assert!(bh_decomp_ok);

    let bh_oracle = ::rea::check_role_access("binary-historian", "rea_binary_oracle_query");
    let bh_oracle_ok = bh_oracle.is_ok();
    ::std::assert!(bh_oracle_ok);

    let tc_decomp = ::rea::check_role_access("target-coder", "rea_decompile_function");
    let tc_decomp_err = tc_decomp.is_err();
    ::std::assert!(tc_decomp_err);

    let tc_oracle = ::rea::check_role_access("target-coder", "rea_binary_oracle_query");
    let tc_oracle_err = tc_oracle.is_err();
    ::std::assert!(tc_oracle_err);

    let bre_query = ::rea::check_role_access("business-rules-extractor", "rea_query_retained_view");
    let bre_query_ok = bre_query.is_ok();
    ::std::assert!(bre_query_ok);

    let bre_decomp = ::rea::check_role_access("business-rules-extractor", "rea_decompile_function");
    let bre_decomp_err = bre_decomp.is_err();
    ::std::assert!(bre_decomp_err);
}

#[test]
fn test_binary_oracle_query() {
    let bin_path = ::std::path::Path::new("core.so");
    let card_res = ::rea::rea_binary_oracle_query(bin_path, "0x1040", "process_packet");
    let card_ok = card_res.is_ok();
    ::std::assert!(card_ok);

    let card = card_res.unwrap();
    ::std::assert_eq!(card.anchor, "ghidra:0x1040#xref");
    let has_given = card.card.contains("GIVEN");
    let has_when = card.card.contains("WHEN");
    let has_then = card.card.contains("THEN");
    ::std::assert!(has_given);
    ::std::assert!(has_when);
    ::std::assert!(has_then);
}

#[test]
fn test_schema_profile() {
    let profile = ::rea::schema_profile();
    ::std::assert_eq!(profile, "compact");
    ::std::assert_eq!(::rea::REA_MCP_INPUT_SCHEMA_PROFILE, "compact");
}
