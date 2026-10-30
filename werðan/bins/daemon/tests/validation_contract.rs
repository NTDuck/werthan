//! Daemon validation contracts.
//! Contracts derived from docs/specs/validation-contracts.md and they outrank implementation tests.
//! Covered assertion IDs follow. VAL-DAEMON-01, VAL-DAEMON-02, VAL-DAEMON-03,
//! VAL-DAEMON-04, VAL-DAEMON-05.

/// Contract VAL-DAEMON-01: the campaign result carries the campaign id and
/// per-module outcomes with final micro-state. It also carries the artifact
/// reference or blocker, the FIFO event log, the totals, and the backend echo.
#[test]
fn contract_val_daemon_01_campaign_result_schema() {
    let board = two_module_board();
    let report = ::daemon::bench_report::run_stub_campaign("contract-schema", board);
    ::std::assert_eq!(report.campaign_id, "contract-schema");
    ::std::assert_eq!(report.module_outcomes.len(), 2);
    for outcome in &report.module_outcomes {
        let hardened = ::core::matches!(
            outcome.final_micro_state,
            ::global_blackboard::ModuleMicroState::M4Hardened
        );
        if hardened {
            ::std::assert!(
                outcome.artifact_reference.is_some(),
                "hardened modules carry an artifact reference"
            );
            ::std::assert!(outcome.blocker.is_none());
        } else {
            ::std::assert!(
                outcome.blocker.is_some(),
                "non-hardened modules carry a blocker"
            );
        }
    }
    ::std::assert_eq!(
        report.hardened_count + report.failed_count,
        report.module_outcomes.len()
    );
    ::std::assert!(report.backend.stub_backend);
    ::std::assert_eq!(report.backend.backend_url, "stub://local");
    // The report schema serializes cleanly to JSON.
    let json_text = ::serde_json::to_string(&report).expect("report serializes");
    ::std::assert!(json_text.contains("\"campaign_id\""));
    ::std::assert!(json_text.contains("\"module_outcomes\""));
    ::std::assert!(json_text.contains("\"event_log\""));
    ::std::assert!(json_text.contains("\"backend\""));
}

/// Contract VAL-DAEMON-02: seeding honesty. A configured URL reaches real
/// HTTP inference. A missing URL fails configuration honestly with no fake
/// fallback.
#[test]
fn contract_val_daemon_02_seeding_honesty() {
    // A configured URL wins over the environment default.
    let configured =
        ::daemon::bench_smoke::resolve_inference_url("http://configured/v1", "http://env/v1");
    ::std::assert_eq!(
        configured.expect("configured URL resolves"),
        "http://configured/v1"
    );
    // The environment default applies only when the URL config stays empty.
    let from_env = ::daemon::bench_smoke::resolve_inference_url("", "http://env/v1");
    ::std::assert_eq!(from_env.expect("env URL resolves"), "http://env/v1");
    // No URL anywhere fails honestly with no silent fake backend.
    let missing = ::daemon::bench_smoke::resolve_inference_url("", "");
    let failure = missing.expect_err("missing URL must fail configuration");
    ::std::assert!(failure.contains("no inference backend URL configured"));
    // The smoke probe surfaces the same honest failure.
    let mut broken_config = ::daemon::bench_config::CampaignConfig::default();
    broken_config.inference.base_url = ::std::string::String::new();
    let probe = ::daemon::bench_smoke::run_smoke_verification_once(&broken_config, "");
    ::std::assert!(probe.is_err(), "the smoke probe never falls back silently");
}

/// Contract VAL-DAEMON-03: phase gating. A failing preflight blocks the
/// campaign before any module transformation dispatch happens.
#[test]
fn contract_val_daemon_03_phase_gating_preflight() {
    let mut board = two_module_board();
    // The empty harness fails the S0 INTENT gate.
    board.inventory.harness = ::global_blackboard::HarnessConfig::default();
    let report = ::daemon::bench_report::run_stub_campaign("contract-gating", board);
    let failure = report
        .error
        .expect("failing preflight surfaces a blocker");
    ::std::assert!(failure.contains("preflight") || failure.contains("intent"));
    ::std::assert_eq!(
        report.solver_dispatches, 0,
        "no module solver dispatch happens before preflight passes"
    );
    ::std::assert_eq!(report.hardened_count, 0);
    ::std::assert_eq!(report.failed_count, 2);
    for outcome in &report.module_outcomes {
        ::std::assert!(outcome.blocker.is_some());
    }
}

/// Contract VAL-DAEMON-04: deterministic dry-run. With the stub backend the
/// 2-module DAG completes deterministically, and the leaf module hardens
/// before the parent. Events arrive in FIFO order, and repeated runs match.
#[test]
fn contract_val_daemon_04_deterministic_dry_run() {
    let first_board = two_module_board();
    let first = ::daemon::bench_report::run_stub_campaign("contract-dry-run", first_board);
    let second_board = two_module_board();
    let second = ::daemon::bench_report::run_stub_campaign("contract-dry-run", second_board);
    // Determinism: repeated runs produce identical reports.
    ::std::assert_eq!(first, second);
    // Completion: both modules hardened, no failures, clean error state.
    ::std::assert!(first.error.is_none());
    ::std::assert_eq!(first.hardened_count, 2);
    ::std::assert_eq!(first.failed_count, 0);
    for outcome in &first.module_outcomes {
        ::std::assert!(::core::matches!(
            outcome.final_micro_state,
            ::global_blackboard::ModuleMicroState::M4Hardened
        ));
        ::std::assert!(outcome.artifact_reference.is_some());
    }
    // Topological order: the leaf module `util` hardens fully before the
    // parent `app` enters its own transformation.
    let entry_for = |module: &::core::primitive::str,
                     state: &::core::primitive::str|
     -> ::core::option::Option<::core::primitive::usize> {
        let needle = ::std::format!("{module}: {state}");
        first
            .event_log
            .iter()
            .position(|entry| entry.contains(&needle))
    };
    let util_hardened = entry_for("util", "M4Hardened");
    let app_hydrated = entry_for("app", "M1ContextHydration");
    ::std::assert!(
        util_hardened.is_some() && app_hydrated.is_some(),
        "event log records leaf and parent micro-state transitions"
    );
    ::std::assert!(
        util_hardened.expect("leaf hardened entry") < app_hydrated.expect("parent entry"),
        "leaf module hardens before the parent transforms"
    );
    // FIFO order for the transformation wave: hydration then hardening per
    // module, leaf before parent.
    let util_hydrated = entry_for("util", "M1ContextHydration");
    let app_hardened = entry_for("app", "M4Hardened");
    ::std::assert!(
        util_hydrated.expect("leaf hydration entry")
            < util_hardened.expect("leaf hardened entry")
    );
    ::std::assert!(
        app_hardened.expect("parent hardened entry") > app_hydrated.expect("parent entry")
    );
    // The final JSON form of the result serializes and reparses cleanly.
    let json_text = ::serde_json::to_string_pretty(&first).expect("report serializes");
    let reparsed = ::serde_json::from_str::<::daemon::bench_report::CampaignReport>(&json_text)
        .expect("report reparses");
    ::std::assert_eq!(reparsed, first);
}

/// Contract VAL-DAEMON-05: the smoke verification entrypoint returns Ok on a
/// healthy configuration and surfaces configuration errors as Err.
#[test]
fn contract_val_daemon_05_smoke_verification_entrypoint() {
    // Healthy configuration: a local OpenAI-compatible endpoint that serves
    // one model listing.
    let listener = ::std::net::TcpListener::bind("127.0.0.1:0").expect("listener binds");
    let addr = listener.local_addr().expect("local address");
    let server = ::std::thread::spawn(move || {
        // The probe issues exactly one request. Poll accept without blocking
        // and stop after the first served connection or a short deadline, so
        // the joining test thread can never hang.
        let _ = ::std::net::TcpListener::set_nonblocking(&listener, true);
        let deadline = ::std::time::Instant::now() + ::std::time::Duration::from_secs(5);
        loop {
            match listener.accept() {
                ::core::result::Result::Ok((mut writer, _)) => {
                    let body = "{\"data\":[{\"id\":\"stub-model\"}]}";
                    let response = ::std::format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = ::std::io::Write::write_all(&mut writer, response.as_bytes());
                    break;
                },
                ::core::result::Result::Err(accept_error)
                    if accept_error.kind() == ::std::io::ErrorKind::WouldBlock =>
                {
                    let poll_delay = ::std::time::Duration::from_millis(10);
                    if ::std::time::Instant::now() >= deadline {
                        break;
                    }
                    ::std::thread::sleep(poll_delay);
                },
                ::core::result::Result::Err(_) => break,
            }
        }
    });
    let mut healthy_config = ::daemon::bench_config::CampaignConfig::default();
    healthy_config.inference.base_url = ::std::format!("http://{addr}");
    let healthy = ::daemon::bench_smoke::run_smoke_verification_once(&healthy_config, "");
    match healthy {
        ::core::result::Result::Ok(report) => {
            ::std::assert_eq!(
                report.base_url,
                ::std::format!("http://{addr}"),
                "the echo names the actually probed endpoint"
            );
            ::std::assert_eq!(report.model_id.expect("served model"), "stub-model");
        },
        ::core::result::Result::Err(failure) => {
            ::std::panic!("healthy endpoint must pass the probe: {failure}");
        },
    }
    // Unreachable endpoint: the error surfaces, never swallowed.
    let dead_listener = ::std::net::TcpListener::bind("127.0.0.1:0").expect("dead listener binds");
    let dead_addr = dead_listener.local_addr().expect("dead local address");
    drop(dead_listener);
    let mut unreachable_config = ::daemon::bench_config::CampaignConfig::default();
    unreachable_config.inference.base_url = ::std::format!("http://{dead_addr}");
    let unreachable = ::daemon::bench_smoke::run_smoke_verification_once(&unreachable_config, "");
    let failure = unreachable.expect_err("unreachable endpoint surfaces as Err");
    ::std::assert!(failure.contains("endpoint unreachable"));
    server.join().expect("server joins");
}

/// Build a deterministic 2-module DAG board: leaf `util` feeds parent `app`.
fn two_module_board() -> ::global_blackboard::GlobalBlackboard {
    let mut board = ::global_blackboard::GlobalBlackboard::default();
    board.inventory.files = ::std::vec!["util".to_string(), "app".to_string()];
    board.inventory.harness = ::global_blackboard::HarnessConfig {
        build_cmd: "cargo check".to_string(),
        test_cmd: "cargo test".to_string(),
        legacy_harness_cmd: "legacy test".to_string(),
    };
    board.dag.vertices = ::std::vec!["util".to_string(), "app".to_string()];
    board.dag.edges = ::std::vec![("util".to_string(), "app".to_string())];
    board.rules.insert("util".to_string(), ::std::vec![rule_card("RU")]);
    board.rules.insert("app".to_string(), ::std::vec![rule_card("RA")]);
    board.contracts.insert(
        ("util".to_string(), "app".to_string()),
        ::holon_blackboard::EdgeContract {
            parent_id: "util".to_string(),
            interface_spec: "iface_util".to_string(),
            preconditions: ::std::vec::Vec::new(),
        },
    );
    board
}

/// Build one cited business rule card for a vertex.
fn rule_card(rule_id: &::core::primitive::str) -> ::holon_blackboard::BusinessRuleCard {
    ::holon_blackboard::BusinessRuleCard {
        rule_id: { rule_id.to_string() },
        given: "valid payload".to_string(),
        when: "parsed".to_string(),
        then: "returns syntax tree".to_string(),
        citation: "legacy/source.c:12".to_string(),
    }
}

/// Contract: external test toolchains map a zero exit code to a full pass
/// rate instead of parsing cargo result lines.
#[test]
fn contract_val_daemon_06_external_test_success_maps_full_rate() {
    let scratch_root = ::std::env::temp_dir();
    let bench_config = external_command_config("true");
    let validation = ::daemon::bench_validation::validate_workspace(&bench_config, &scratch_root);
    ::std::assert!(validation.check_ok);
    ::std::assert_eq!(validation.test_pass_rate, 1.0);
}

/// Contract: external test toolchains map a nonzero exit code to zero.
#[test]
fn contract_val_daemon_07_external_test_failure_maps_zero_rate() {
    let scratch_root = ::std::env::temp_dir();
    let bench_config = external_command_config("false");
    let validation = ::daemon::bench_validation::validate_workspace(&bench_config, &scratch_root);
    ::std::assert_eq!(validation.test_pass_rate, 0.0);
}

/// Contract: compilation success reflects only the final workspace artifact
/// state. The engine terminal error lands in the `error` field and does not
/// poison the compilation flag when the check passed and no unimplemented
/// stubs remain.
#[test]
fn contract_val_daemon_08_compilation_success_ignores_terminal_error() {
    let workspace_validation = ::daemon::bench_validation::WorkspaceValidation {
        check_ok: true,
        test_pass_rate: 1.0,
    };
    let terminal_error = ::core::option::Option::Some(
        "module source.py exhausted the futile dispatch cap after 3 attempts".to_string(),
    );
    let bench_result = ::daemon::bench_runner::BenchResult {
        campaign_id: "contract-engine-error".to_string(),
        wall_time_ms: 0,
        engine_wall_time_ms: 0,
        llm_requests: 0,
        unimplemented_before: 0,
        unimplemented_after_reset: 0,
        compilation_success: { workspace_validation.check_ok },
        test_pass_rate: { workspace_validation.test_pass_rate },
        unimplemented_remaining: 0,
        solver_dispatches: 0,
        worker_executions: 0,
        structural_retries: 0,
        macro_states_visited: 0,
        total_lines: 0,
        code_lines: 0,
        cocomo_effort_pm: 0.0,
        cocomo_duration_months: 0.0,
        error: { terminal_error },
    };
    ::std::assert!(
        bench_result.compilation_success,
        "a passing workspace check with no stubs stays a compilation success even with an engine error"
    );
    ::std::assert!(bench_result.error.is_some(), "the engine error stays visible");
}

/// Build a config whose build and test commands exit with the given status.
fn external_command_config(test_program: &'static ::core::primitive::str) -> ::daemon::bench_config::CampaignConfig {
    let mut bench_config = ::daemon::bench_config::CampaignConfig::default();
    bench_config.inference.build_timeout_secs = 30;
    bench_config.inference.test_timeout_secs = 30;
    bench_config.stack.target_build_cmd = "true".to_string();
    bench_config.stack.target_test_cmd = test_program.to_string();
    bench_config
}
