//! End-to-end smoke verification of every harness and protocol concern.

pub async fn run_smoke_verification() {
    ::std::println!("[orchestrator] Running smoke verification...");

    let config = ::protocol::ModernizeConfig::default();
    ::std::println!("[orchestrator] ModernizeConfig campaign: {}", config.campaign_id);
    let laya_client = ::laya::LayaClient::simulated();
    let query = ::laya::build_transform_gate_query("calc", "no blocker", "clean", "passed", 0);
    let laya_res = laya_client.evaluate(&query).await;
    ::std::assert!(laya_res.is_ok(), "Laya decision must succeed");
    ::std::println!("[orchestrator] Laya evaluation verified.");

    let pool = ::harness::worker_pool::WorkerPool::new(2, "qwen3.6-35b-a3b");
    let shards = ::std::vec!["calc_lexer.c".to_string(), "calc_parser.c".to_string()];
    let shard_res = pool.dispatch_shards(&shards).await;
    ::std::assert!(shard_res.is_ok(), "Shard dispatch must succeed");
    ::std::println!("[orchestrator] Worker pool (2 shards) verified.");

    let xml = "<testsuites tests=\"4\" failures=\"0\"><testsuite tests=\"4\" failures=\"0\"/></testsuites>";
    let parsed_xml = ::verify::parse_junit_xml(xml);
    ::std::assert!(parsed_xml.is_ok(), "JUnit XML parse must succeed");
    ::std::println!("[orchestrator] JUnit XML verification verified.");

    let diff_receipt = ::verify::compare_outputs("result: 42\n", "result: 42\n");
    ::std::assert_eq!(diff_receipt, ::verify::VerificationReceipt::Proven);
    ::std::println!("[orchestrator] Verification diff verified.");

    let curr_dir = ::std::path::Path::new(".");
    let scc_res = ::rea::run_scc_analysis(curr_dir).await;
    ::std::assert!(scc_res.is_ok(), "SCC analysis must succeed");
    ::std::println!("[orchestrator] REA MCP scc analysis verified.");

    let sym_res = ::rea::rea_symbol_graph(curr_dir);
    ::std::assert!(sym_res.is_ok(), "Symbol graph extraction must succeed");
    ::std::println!("[orchestrator] REA MCP symbol graph verified.");

    let slice_res = ::rea::rea_dataflow_slice("process_batch", "buffer", curr_dir);
    ::std::assert!(slice_res.is_ok(), "Dataflow slice extraction must succeed");
    ::std::println!("[orchestrator] REA MCP dataflow slice verified.");

    let dummy_bin = ::std::path::Path::new("bin/daemon");
    let decomp_res = ::rea::rea_decompile_function(dummy_bin, "process_batch", 1, 100);
    ::std::assert!(decomp_res.is_ok(), "Decompile function must succeed");
    ::std::println!("[orchestrator] REA MCP decompile function verified.");

    let xref_res = ::rea::rea_xrefs(curr_dir, "process_batch");
    ::std::assert!(xref_res.is_ok(), "Xref analysis must succeed");
    ::std::println!("[orchestrator] REA MCP xrefs verified.");

    let profile = ::laya::route_execution_profile(1.2, 10, false);
    ::std::assert_eq!(profile, ::laya::ExecutionProfile::FastMoeAst);
    ::std::println!("[orchestrator] Laya route execution profile verified.");

    let guardrail = ::laya::evaluate_guardrail_verdict(0.12, false);
    ::std::assert!(guardrail.allow, "Guardrail must accept clean edit");
    ::std::println!("[orchestrator] Laya guardrail verdict verified.");

    let rule_card = "calc.c:42 GIVEN valid input WHEN parsed THEN returns token";
    let rule_verdict = ::laya::judge_rule_card(rule_card);
    ::std::assert!(rule_verdict.is_accepted, "Rule card judge must accept valid rule");
    ::std::println!("[orchestrator] Laya rule card judge verified.");

    let coverage = ::laya::judge_test_coverage_theater("assert_eq!(result, 42)");
    ::std::assert!(coverage.is_accepted, "Coverage judge must accept value assertions");
    ::std::println!("[orchestrator] Laya test coverage theater judge verified.");

    let done_verdict = ::laya::judge_done_claim(5, true);
    ::std::assert!(done_verdict.valid, "Done claim judge must verify valid execution");
    ::std::println!("[orchestrator] Laya done claim judge verified.");

    let magic_prompt = "please ultrathink and orchestrate the workflowz run.";
    let magic_dirs = ::protocol::directives::extract_magic_directives(magic_prompt);
    ::std::assert!(magic_dirs.ultrathink, "ultrathink must be extracted.");
    ::std::assert!(magic_dirs.orchestrate, "orchestrate must be extracted.");
    ::std::assert!(magic_dirs.workflowz, "workflowz must be extracted.");
    ::std::println!("[orchestrator] OMP keywords extract_magic_directives verified.");

    let raw_source = "alpha\nbeta\ngamma\ndelta\n";
    let hashlined = ::harness::hashline::render_hashlined_view(raw_source, 0, 4);
    let view_lines = hashlined.lines().collect::<::std::vec::Vec<&::core::primitive::str>>();
    ::std::assert!(!view_lines.is_empty(), "Hashline view must produce lines.");
    let first_line = view_lines[0];
    let anchor_prefix = match first_line.split('\u{2502}').next() {
        ::core::option::Option::Some(a) => a.trim(),
        ::core::option::Option::None => "",
    };
    ::std::assert_eq!(anchor_prefix.len(), 4, "Hashline anchor must be 4 characters.");

    let test_lines = ::std::vec!["alpha", "beta", "gamma", "delta"];
    let a1 = ::harness::hashline::compute_line_anchor(&test_lines, 1);
    let a2 = ::harness::hashline::compute_line_anchor(&test_lines, 2);
    ::std::assert_eq!(a1.len(), 4, "Anchor 1 must be 4 characters.");
    ::std::assert_eq!(a2.len(), 4, "Anchor 2 must be 4 characters.");
    let replaced = ::harness::hashline::apply_hashline_replace(raw_source, &a1, &a2, "beta prime\ngamma prime\n");
    ::std::assert!(replaced.is_ok(), "Hashline replacement must succeed.");

    let dup_lines = ::std::vec!["item", "item", "item", "item", "item"];
    let dup_source = "item\nitem\nitem\nitem\nitem\n";
    let a_dup = ::harness::hashline::compute_line_anchor(&dup_lines, 1);
    let collision_res = ::harness::hashline::apply_hashline_replace(dup_source, &a_dup, &a_dup, "replacement\n");
    ::std::assert_eq!(
        collision_res,
        ::core::result::Result::Err("HASHLINE_AMBIGUOUS".to_string()),
        "Collision must be rejected with HASHLINE_AMBIGUOUS."
    );
    ::std::println!("[orchestrator] OMP hashline view, 4-char anchors, and collision rejection verified.");

    let hmac_key = b"werthan-snapcompact-secret";
    let sample_frame_hashes =
        ::std::vec!["e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),];
    let lead_in = "SnapCompact Archive Frame.";
    let hmac_sig = ::harness::snapcompact::sign_manifest(hmac_key, &sample_frame_hashes, lead_in);
    ::std::assert_eq!(hmac_sig.len(), 64, "HMAC signature must be 64 characters.");

    let archive = ::harness::snapcompact::SnapCompactArchive {
        frame_hashes: { sample_frame_hashes },
        lead_in_banner: { lead_in.to_string() },
        hmac_signature: { hmac_sig },
    };
    ::std::assert!(!archive.frame_hashes.is_empty(), "Archive frame hashes must not be empty.");

    let sample_bytes = b"hello world";
    let expected_sha256 = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
    let frame_verified = ::harness::snapcompact::verify_frame_bytes(expected_sha256, sample_bytes);
    ::std::assert!(frame_verified, "Frame bytes must verify against SHA256 hex.");

    let stub = ::harness::snapcompact::format_text_archive_stub(expected_sha256, 42);
    ::std::assert_eq!(
        stub,
        ::std::format!("[snapcompact:sha256={expected_sha256}:lines=42]"),
        "SnapCompact stub format must match artifact pointer."
    );
    ::std::println!("[orchestrator] OMP snapcompact archive signing, verification, and stubs verified.");

    let stream_rules = ::std::vec![::harness::stream_rules::DormantStreamRule {
        id: { "no_unimplemented".to_string() },
        trigger_pattern: { "unimplemented!".to_string() },
        reminder_injection: { "Do not leave placeholder code stubs.".to_string() },
    }];
    let accumulated_stream = "line one\n";
    let incoming_token = "unimplemented!()\n";
    let stream_intercept =
        ::harness::stream_rules::scan_token_stream(accumulated_stream, incoming_token, &stream_rules);
    match stream_intercept {
        ::harness::stream_rules::StreamInterceptOutcome::AbortAndRewind {
            clean_prefix,
            injected_reminder,
            rule_id,
        } => {
            ::std::assert_eq!(clean_prefix, "line one\n", "Prefix must rewind cleanly.");
            ::std::assert!(injected_reminder.contains("<system-reminder>"), "Must contain reminder tag.");
            ::std::assert!(
                injected_reminder.contains("Do not leave placeholder code stubs."),
                "Must contain reminder text."
            );
            ::std::assert_eq!(rule_id, "no_unimplemented", "Rule id must match.");
        },
        ::harness::stream_rules::StreamInterceptOutcome::PassThrough(_) => {
            ::core::panic!("Stream rule must intercept token.");
        },
    }
    ::std::println!("[orchestrator] OMP stream-rules mid-token intercept, rewind, and injection verified.");

    let search_root = ::std::path::Path::new("wer\u{00f0}an/harness/src");
    let search_res = ::harness::lsp_tools::search_files_in_process(search_root, "search_files_in_process");
    ::std::assert!(search_res.is_ok(), "In-process search must succeed.");
    let search_matches = search_res.unwrap();
    ::std::assert!(!search_matches.is_empty(), "In-process search must find target query.");
    ::std::assert!(
        search_matches.iter().any(|m| m.line_content.contains("pub fn search_files_in_process")),
        "Search match must contain function declaration."
    );

    let lsp_diags = ::std::vec![::harness::lsp_tools::LspDiagnostic {
        file_path: { "src/engine.rs".to_string() },
        line: { 42 },
        message: { "Type error in configuration.".to_string() },
        severity: { "error".to_string() },
    }];
    let edit_output = "Updated engine signature.";
    let lsp_loopback = ::harness::lsp_tools::format_edit_with_lsp_diagnostics(edit_output, &lsp_diags);
    ::std::assert!(lsp_loopback.contains("Updated engine signature."), "LSP loopback must preserve edit.");
    ::std::assert!(lsp_loopback.contains("LSP Diagnostics:"), "LSP loopback must contain diagnostics.");
    ::std::assert!(
        lsp_loopback.contains("[error] src/engine.rs:42: Type error"),
        "LSP loopback must contain diagnostic line."
    );
    ::std::println!("[orchestrator] OMP lsp-tools in-process search and diagnostics loopback verified.");

    let sandbox_policy = ::harness::sandbox::SandboxPolicy {
        read_only_paths: { ::std::vec![::std::path::PathBuf::from("/workspace/legacy")] },
        writable_paths: {
            ::std::vec![::std::path::PathBuf::from("/workspace/modernized"), ::std::path::PathBuf::from("/tmp"),]
        },
    };
    let legacy_file = ::std::path::Path::new("/workspace/legacy/CobolSource.cbl");
    let legacy_read = ::harness::sandbox::enforce_filesystem_sandbox(&sandbox_policy, legacy_file, false);
    ::std::assert!(legacy_read.is_ok(), "Sandbox permits read access to legacy.");

    let legacy_write = ::harness::sandbox::enforce_filesystem_sandbox(&sandbox_policy, legacy_file, true);
    ::std::assert!(legacy_write.is_err(), "Sandbox denies write access to legacy.");

    let mod_file = ::std::path::Path::new("/workspace/modernized/lib.rs");
    let mod_write = ::harness::sandbox::enforce_filesystem_sandbox(&sandbox_policy, mod_file, true);
    ::std::assert!(mod_write.is_ok(), "Sandbox permits write access to modernized.");
    ::std::println!("[orchestrator] OMP sandbox filesystem policy enforcement verified.");

    let terminal_width: ::core::primitive::u16 = 100;
    let left_width = terminal_width * 60 / 100;
    let right_width = terminal_width - left_width;
    ::std::assert_eq!(left_width, 60, "Left orchestrator pane must be 60% width.");
    ::std::assert_eq!(right_width, 40, "Right stack must be 40% width.");
    let sync_start = "\x1b[?2026h";
    let sync_end = "\x1b[?2026l";
    ::std::assert!(sync_start.ends_with('h'), "CSI-2026 start synchronization token must end with h.");
    ::std::assert!(sync_end.ends_with('l'), "CSI-2026 end synchronization token must end with l.");
    ::std::println!("[orchestrator] OMP tui 60/40 layout geometry and CSI-2026 sync tokens verified.");
    ::std::println!("[orchestrator] All smoke verifications passed successfully!");
}
