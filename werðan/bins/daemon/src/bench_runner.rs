//! Real per-project benchmark execution: honest LLM-backed engine run plus post-run validation.
//! Total pipeline wall time, cargo toolchain checks with timeouts, deterministic COCOMO
//! metrics, and an inference honesty gate.


pub async fn run_campaign(
    bench_config: &super::bench_config::CampaignConfig,
) -> self::BenchResult {
    let (result_sender, result_receiver) = ::std::sync::mpsc::channel();
    let thread_config = bench_config.clone();
    ::std::thread::spawn(move || {
        // The LLM client is blocking HTTP. MacroEngine is not Send, so the whole
        // campaign executes on this dedicated OS thread, away from async workers.
        let outcome = self::run_campaign_blocking(&thread_config);
        let send_ok = result_sender.send(outcome).is_ok();
        if !send_ok {
            ::tracing::error!("campaign result receiver dropped before the result arrived");
        }
    });
    let campaign_res = result_receiver.recv();
    let campaign_result = match campaign_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(recv_error) => {
            let reason = ::std::format!("campaign thread join failed: {recv_error}");
            self::failed_result(&bench_config.campaign_id, &reason)
        },
    };
    campaign_result
}

/// Execute one full campaign synchronously: workspace reset, LLM-backed engine
/// run, and post-run cargo validation. Called from a dedicated OS thread.
fn run_campaign_blocking(
    bench_config: &super::bench_config::CampaignConfig,
) -> self::BenchResult {
    let campaign_started = ::std::time::Instant::now();
    let campaign_id = bench_config.campaign_id.clone();
    let workspace_root = self::workspace_root_of(bench_config);
    let unimplemented_before = super::bench_harness::count_unimplemented(&workspace_root);
    let reset_error = self::reset_workspace(bench_config, &workspace_root);
    let unimplemented_after_reset = super::bench_harness::count_unimplemented(&workspace_root);
    if let ::core::option::Option::Some(reset_failure) = reset_error {
        let mut failed = self::failed_result(&campaign_id, &reset_failure);
        failed.wall_time_ms =
            campaign_started.elapsed().as_millis() as ::core::primitive::u64;
        return failed;
    }
    let api_surface_text =
        super::bench_rules::target_api_surface(&workspace_root);
    let seed_res = super::bench_harness::seed_board(bench_config);
    let seed_output = match seed_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(seed_error) => {
            let mut failed = self::failed_result(&campaign_id, &seed_error);
            failed.wall_time_ms =
            campaign_started.elapsed().as_millis() as ::core::primitive::u64;
            return failed;
        },
    };
    let metrics = seed_output.metrics;
    let effort_pm = super::bench_harness::cocomo_effort_pm(metrics.code_lines);
    let duration_months = super::bench_harness::cocomo_duration_months(effort_pm);
    let zero_requests = ::std::sync::atomic::AtomicU64::new(0);
    let request_count = ::std::sync::Arc::new(zero_requests);
    // S3 content executes here, before the engine starts. Discovery workers
    // dispatch empty problem models, so rule cards must land on the board.
    // Otherwise testgen produces no characterization tests inside rollouts.
    let rule_entries_res = super::bench_harness::inventory_entries(bench_config);
    let rule_entries = match rule_entries_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(inventory_error) => {
            let mut failed = self::failed_result(&campaign_id, &inventory_error);
            failed.wall_time_ms =
                campaign_started.elapsed().as_millis() as ::core::primitive::u64;
            return failed;
        },
    };
    let rule_report = super::bench_rules::extract_rules_for_board(
        bench_config,
        &rule_entries,
        &request_count,
        &api_surface_text,
    );
    ::tracing::info!(
        "campaign {}: rule extraction issued {} requests over {} vertices, {} failures",
        campaign_id,
        rule_report.request_count,
        rule_entries.len(),
        rule_report.failed_vertices.len()
    );
    let mut seeded_board = seed_output.board;
    for (vertex_id, cards) in rule_report.cards_per_vertex {
        seeded_board.rules.insert(vertex_id, cards);
    }
    let solver_res = self::build_solver(bench_config, request_count.clone(), workspace_root.clone());
    let solver = match solver_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(config_failure) => {
            let mut failed = self::failed_result(&campaign_id, &config_failure);
            failed.wall_time_ms =
                campaign_started.elapsed().as_millis() as ::core::primitive::u64;
            return failed;
        },
    };
    let hitl_gates = ::global_engine::HitlGates {
        gate_on_reimagine: ::core::clone::Clone::clone(&bench_config.hitl.gate_on_reimagine),
        gate_on_macro_regression: ::core::clone::Clone::clone(
            &bench_config.hitl.gate_on_macro_regression,
        ),
        extrinsic_conflict_ask_human: ::core::matches!(
            &bench_config.hitl.extrinsic_conflict_escalation,
            ::configschema::ConflictEscalation::AskHuman
        ),
    };
    let mut engine = ::global_engine::MacroEngine::builder()
        .solver(solver)
        .hitl(hitl_gates)
        .build();
    engine.configure(seeded_board);
    ::tracing::info!("starting engine.run()");
    let engine_started = ::std::time::Instant::now();
    let run_res = engine.run();
    let _ = &engine;
    let engine_wall_time_ms = { engine_started.elapsed().as_millis() as ::core::primitive::u64 };
    ::tracing::info!("finished engine.run() in {}ms", engine_wall_time_ms);
    let engine_error = match run_res {
        ::core::result::Result::Ok(global_engine_run_stop) => {
            ::tracing::info!("engine stopped: {global_engine_run_stop:?}");
            match global_engine_run_stop {
                ::global_engine::RunStop::Terminated => ::core::option::Option::None,
                ::global_engine::RunStop::HitlGatePending { gate } => {
                    // A headless benchmark has no operator to approve the
                    // gate, so a parked campaign is an explicit outcome.
                    let detail = ::std::format!("campaign parked on human gate {gate:?}");
                    ::tracing::info!("{detail}");
                    ::core::option::Option::Some(detail)
                }
            }
        },
        ::core::result::Result::Err(engine_failure) => {
            ::core::option::Option::Some(::std::format!("{engine_failure}"))
        },
    };
    let validation = super::bench_validation::validate_workspace(bench_config, &workspace_root);
    let unimplemented_remaining = super::bench_harness::count_unimplemented(&workspace_root);
    let llm_requests = request_count.load(::std::sync::atomic::Ordering::SeqCst);
    ::tracing::info!(
        "campaign {}: llm requests {}, engine wall {} ms",
        campaign_id,
        llm_requests,
        engine_wall_time_ms
    );
    let solver_dispatches = engine.solver_dispatches();
    let dispatches_present = solver_dispatches > 0;
    let zero_inference = llm_requests == 0;
    let honesty_error = if dispatches_present && zero_inference {
        ::core::option::Option::Some("dishonest: zero inference calls".to_string())
    } else {
        ::core::option::Option::None
    };
    let engine_error = if engine_error.is_none() {
        honesty_error
    } else {
        engine_error
    };
    let compilation_success = validation.check_ok
        && unimplemented_remaining == 0
        && engine_error.is_none();
    let wall_time_ms = { campaign_started.elapsed().as_millis() as ::core::primitive::u64 };
    let state_vector = engine.state_vector();
    let result = self::BenchResult {
        campaign_id,
        wall_time_ms,
        engine_wall_time_ms,
        llm_requests,
        unimplemented_before,
        unimplemented_after_reset,
        compilation_success,
        test_pass_rate: { validation.test_pass_rate },
        unimplemented_remaining,
        solver_dispatches: { solver_dispatches },
        worker_executions: { engine.worker_executions() },
        structural_retries: { engine.structural_retries() },
        macro_states_visited: { state_vector.module_states.len() },
        total_lines: { metrics.total_lines },
        code_lines: { metrics.code_lines },
        cocomo_effort_pm: { effort_pm },
        cocomo_duration_months: { duration_months },
        error: { engine_error },
    };
    result
}

#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct BenchResult {
    /// Campaign identifier from the configuration.
    pub campaign_id: ::std::string::String,
    /// Total honest pipeline wall time in milliseconds: reset through validation.
    pub wall_time_ms: ::core::primitive::u64,
    /// Wall-clock duration of the engine run alone in milliseconds.
    pub engine_wall_time_ms: ::core::primitive::u64,
    /// Real inference backend request count observed during the run.
    pub llm_requests: ::core::primitive::u64,
    /// Stub marker count before the workspace reset.
    pub unimplemented_before: ::core::primitive::usize,
    /// Stub marker count immediately after the workspace reset.
    pub unimplemented_after_reset: ::core::primitive::usize,
    /// Whether cargo check exited zero and no stub markers remain.
    pub compilation_success: ::core::primitive::bool,
    /// Fraction of passed tests over passed plus failed tests.
    pub test_pass_rate: ::core::primitive::f64,
    /// Remaining `unimplemented!()` marker count in workspace sources.
    pub unimplemented_remaining: ::core::primitive::usize,
    /// Solver dispatch count reported by the engine.
    pub solver_dispatches: ::core::primitive::usize,
    /// Phase-shard worker execution count reported by the engine.
    pub worker_executions: ::core::primitive::usize,
    /// Structural guard retry count reported by the engine.
    pub structural_retries: ::core::primitive::usize,
    /// Number of per-module micro states visited by the engine.
    pub macro_states_visited: ::core::primitive::usize,
    /// Measured legacy total lines.
    pub total_lines: ::core::primitive::u64,
    /// Measured legacy code lines.
    pub code_lines: ::core::primitive::u64,
    /// COCOMO-II effort in person-months from measured KLOC.
    pub cocomo_effort_pm: ::core::primitive::f64,
    /// COCOMO-II duration in months from effort.
    pub cocomo_duration_months: ::core::primitive::f64,
    /// Failure description or null on success.
    pub error: ::core::option::Option<::std::string::String>,
}

/// Return the repo-rooted target workspace path.
fn workspace_root_of(bench_config: &super::bench_config::CampaignConfig) -> ::std::path::PathBuf {
    super::bench_config::repo_root().join(&bench_config.paths.workspace_root)
}

/// Restore the target workspace to the pristine ReCodeAgent dataset stub state.
///
/// The dataset layout `benchmarks/assets/ReCodeAgent/data/tool_projects/<category>/<project>/<lang>`
/// mirrors the generated `benchmarks/outputs/<category>/<project>/<lang>` roots, so the
/// mapped dataset path replaces the `benchmarks/outputs/` prefix of the workspace root.
fn reset_workspace(
    _bench_config: &super::bench_config::CampaignConfig,
    workspace_root: &::std::path::Path,
) -> ::core::option::Option<::std::string::String> {
    let repo_root = super::bench_config::repo_root();
    let root_text = workspace_root
        .strip_prefix(&repo_root)
        .unwrap_or(workspace_root)
        .to_string_lossy()
        .to_string();
    let outputs_prefix = "benchmarks/outputs/";
    let not_outputs_rooted = !root_text.starts_with(outputs_prefix);
    if not_outputs_rooted {
        return ::core::option::Option::Some(::std::format!(
            "workspace root {} is not under {outputs_prefix}",
            root_text
        ));
    }
    let dataset_relative = root_text.replacen(
        outputs_prefix,
        "benchmarks/assets/ReCodeAgent/data/tool_projects/",
        1,
    );
    let dataset_dir = repo_root.join(&dataset_relative);
    let dataset_missing = !dataset_dir.is_dir();
    if dataset_missing {
        return ::core::option::Option::Some(::std::format!(
            "pristine dataset dir missing: {}",
            dataset_dir.display()
        ));
    }
    let remove_res = ::std::fs::remove_dir_all(workspace_root);
    if let ::core::result::Result::Err(remove_error) = remove_res {
        // A missing workspace root is already the pristine state the reset
        // targets. Only a real removal failure blocks the campaign.
        let already_clean = remove_error.kind() == ::std::io::ErrorKind::NotFound;
        if !already_clean {
            return ::core::option::Option::Some(::std::format!(
                "workspace reset remove failed: {remove_error}"
            ));
        }
    }
    let copy_res = self::copy_dir_recursive(&dataset_dir, workspace_root);
    if let ::core::result::Result::Err(copy_error) = copy_res {
        return ::core::option::Option::Some(::std::format!(
            "workspace reset copy failed: {copy_error}"
        ));
    }
    ::core::option::Option::None
}

/// Copy one directory tree recursively, creating missing parents.
fn copy_dir_recursive(
    source_dir: &::std::path::Path,
    target_dir: &::std::path::Path,
) -> ::core::result::Result<(), ::std::string::String> {
    let create_res = ::std::fs::create_dir_all(target_dir);
    if let ::core::result::Result::Err(create_error) = create_res {
        return ::core::result::Result::Err(::std::format!(
            "dir creation failed for {}: {create_error}",
            target_dir.display()
        ));
    }
    let entries = ::std::fs::read_dir(source_dir)
        .map_err(|read_error| ::std::format!("read_dir failed: {read_error}"))?;
    for entry in entries.flatten() {
        let entry_path = entry.path();
        let entry_name = entry.file_name();
        let entry_target = target_dir.join(&entry_name);
        let entry_is_dir = entry_path.is_dir();
        if entry_is_dir {
            let nested_res = self::copy_dir_recursive(&entry_path, &entry_target);
            nested_res?;
        } else {
            let copy_res = ::std::fs::copy(&entry_path, &entry_target);
            if let ::core::result::Result::Err(copy_error) = copy_res {
                return ::core::result::Result::Err(::std::format!(
                    "copy failed for {}: {copy_error}",
                    entry_path.display()
                ));
            }
        }
    }
    ::core::result::Result::Ok(())
}

/// Build a failure result for a campaign that could not even seed.
fn failed_result(campaign_id: &::core::primitive::str, reason: &::core::primitive::str) -> self::BenchResult {
    self::BenchResult {
        campaign_id: { campaign_id.to_string() },
        wall_time_ms: 0,
        engine_wall_time_ms: 0,
        llm_requests: 0,
        unimplemented_before: 0,
        unimplemented_after_reset: 0,
        compilation_success: false,
        test_pass_rate: 0.0,
        unimplemented_remaining: 0,
        solver_dispatches: 0,
        worker_executions: 0,
        structural_retries: 0,
        macro_states_visited: 0,
        total_lines: 0,
        code_lines: 0,
        cocomo_effort_pm: 0.0,
        cocomo_duration_months: 0.0,
        error: ::core::option::Option::Some(reason.to_string()),
    }
}

/// Build the real LLM-backed ARIADNE solver for the engine.
///
/// This is the only place in the daemon that references the mcts crate so a
/// later solver substitution adjusts a single function. Every per-module
/// controller gets an LLM subagent pool and a real CargoHarness evaluator
/// that compiles and tests candidates in the target workspace.
fn build_solver(
    bench_config: &super::bench_config::CampaignConfig,
    request_count: ::std::sync::Arc<::std::sync::atomic::AtomicU64>,
    workspace_root: ::std::path::PathBuf,
) -> ::core::result::Result<::std::boxed::Box<dyn ::global_engine::ModuleSolver>, ::std::string::String> {
    let inference = &bench_config.inference;
    let compat_model_res = super::bench_llm::OpenAiCompatModel::from_inference(inference);
    let compat_model = match compat_model_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(failure) => {
            return ::core::result::Result::Err(failure);
        },
    };
    ::tracing::info!("campaign {}: LLM backend configured", bench_config.campaign_id);
    let mut mcts_config = ::holon_engine::EngineConfig::default();
    let rollout_budget = bench_config.ariadne.engine_budget_per_module;
    let rollout_nonzero = rollout_budget > 0;
    if rollout_nonzero {
        mcts_config.max_rollouts = rollout_budget;
    }
    mcts_config.ucb_c_exp = bench_config.ariadne.ucb_exploration_c;
    mcts_config.softmax_temperature = bench_config.ariadne.softmax_temperature;
    let engine_weights = ::holon_engine::RewardWeights {
        compile: ::core::clone::Clone::clone(&bench_config.ariadne.reward_weights.compile),
        parity: ::core::clone::Clone::clone(
            &bench_config.ariadne.reward_weights.characterization_parity,
        ),
        harden: ::core::clone::Clone::clone(&bench_config.ariadne.reward_weights.hardening_score),
    };
    mcts_config.reward_weights = engine_weights;
    let counting_model =
        super::bench_llm::CountingCompletionModel::new(compat_model, request_count.clone());
    let base_temperature = inference.inference_temperature;
    let closure_root = workspace_root.clone();
    let build_timeout_secs = bench_config.inference.build_timeout_secs;
    let test_timeout_secs = bench_config.inference.test_timeout_secs;

    let strategy_agent = ::holon_strategy::HolonStrategyAgent::builder()
        .model(counting_model.clone())
        .temperature(base_temperature)
        .build();
    let codegen_agent = ::holon_codegen::HolonCodeGenAgent::builder()
        .model(counting_model.clone())
        .temperature(base_temperature)
        .build();
    let testgen_agent = ::holon_testgen::HolonTestGenAgent::builder()
        .model(counting_model.clone())
        .temperature(base_temperature)
        .build();
    let patch_agent = ::holon_patch::HolonPatchAgent::builder()
        .model(counting_model.clone())
        .temperature(base_temperature)
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
    let classifier_agent = ::holon_evaluator::HolonEvaluatorAgent::classifier_builder()
        .model(counting_model.clone())
        .temperature(base_temperature)
        .build();
    let auditor_agent = ::holon_evaluator::HolonEvaluatorAgent::auditor_builder()
        .model(counting_model.clone())
        .temperature(base_temperature)
        .build();
    let distiller = ::holon_evaluator::RigDiagnosticDistiller::builder()
        .classifier(classifier_agent)
        .auditor(auditor_agent)
        .build();
    let harness = ::holon_evaluator::cargo_harness::CargoHarness::builder()
        .workspace_root(closure_root)
        .build_timeout_secs(build_timeout_secs)
        .test_timeout_secs(test_timeout_secs)
        .build();

    let solver = ::holon_engine::solver::AriadneSolver::<
        super::bench_llm::CountingCompletionModel<super::bench_llm::OpenAiCompatModel>,
        ::holon_evaluator::cargo_harness::CargoHarness,
        ::holon_evaluator::RigDiagnosticDistiller<
            super::bench_llm::CountingCompletionModel<super::bench_llm::OpenAiCompatModel>,
        >,
    >::builder()
    .config(mcts_config)
    .agents(agents)
    .harness(harness)
    .distiller(distiller)
    .build();
    let boxed_solver = ::std::boxed::Box::new(solver);
    let dyn_solver: ::std::boxed::Box<dyn ::global_engine::ModuleSolver> = boxed_solver;
    let solver_ok = ::core::result::Result::Ok(dyn_solver);
    solver_ok
}

#[cfg(test)]
mod tests {
    /// Real end-to-end smoke against the existing crust-vec stub workspace.
    /// Runs board seeding, a real cargo check, and a real cargo test.
    #[::tokio::test]
    #[ignore]
    async fn honest_smoke_crust_vec() {
        let repo_root = super::super::bench_config::repo_root();
        let config_path = repo_root.join("benchmarks/assets/configs/ReCodeAgent/crust/vec.yml");
        let load_res = super::super::bench_config::load_bench_config(&config_path);
        let mut bench_config = load_res.expect("crust-vec config loads");
        let _ = ::tracing_subscriber::fmt().try_init();
        bench_config.ariadne.engine_budget_per_module = 1;
        bench_config.verification.max_self_heal_retries = 0;
        bench_config.inference.inference_max_tokens = 4096;
        bench_config.inference.inference_timeout_secs = 120;
        let result = super::run_campaign(&bench_config).await;
        ::std::println!("campaign_id: {}", result.campaign_id);
        ::std::println!("wall_time_ms: {}", result.wall_time_ms);
        ::std::println!(
            "compilation_success: {}",
            result.compilation_success
        );
        ::std::println!("test_pass_rate: {:.3}", result.test_pass_rate);
        ::std::println!(
            "unimplemented_remaining: {}",
            result.unimplemented_remaining
        );
        ::std::println!(
            "solver_dispatches: {} structural_retries: {} macro_states_visited: {}",
            result.solver_dispatches,
            result.structural_retries,
            result.macro_states_visited
        );
        ::std::println!(
            "total_lines: {} code_lines: {}",
            result.total_lines,
            result.code_lines
        );
        ::std::println!(
            "cocomo_effort_pm: {:.2} cocomo_duration_months: {:.2}",
            result.cocomo_effort_pm,
            result.cocomo_duration_months
        );
        ::std::println!("error: {:?}", result.error);
        let smoke_json = ::serde_json::to_string_pretty(&result)
            .expect("BenchResult serializes for the smoke printout");
        ::std::println!("[smoke] BenchResult JSON:\n{smoke_json}");
        // A bounded smoke against a pristine stub workspace proves harness
        // honesty, not full migration. The solver really dispatched, the
        // backend really got called, and the reset restored the stub state
        // (six markers) before the engine started. Rollout evaluation writes
        // candidates into the workspace by design, so the remaining marker
        // count reflects the last evaluated candidate, not the stubs.
        ::std::assert!(
            result.solver_dispatches > 0,
            "the real solver must dispatch at least once"
        );
        ::std::assert!(
            result.llm_requests > 0,
            "the engine must issue real inference calls"
        );
        ::std::assert_eq!(
            result.unimplemented_after_reset, 6,
            "the reset must restore the six dataset stub markers regardless of the start state"
        );
        let honesty_failure = result
            .error
            .as_ref()
            .map(|error_text| error_text.contains("dishonest"))
            .unwrap_or(false);
        ::std::assert!(!honesty_failure, "run must pass the inference honesty gate");
    }
}
