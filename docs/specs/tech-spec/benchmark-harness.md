# Benchmark Harness (`werðan/bins/daemon`)

Entry point: [../tech-spec.md](../tech-spec.md). This file documents the daemon benchmark runner modules, workspace semantics, honesty gate, result artifacts, and LoC-tier scaling ([ADR-0008](../../decisions/0008-loc-tier-inference-scaling.md)).

## 1. Daemon `bench_*` Module Map

`werðan/bins/daemon/src/main.rs` is a thin argument dispatcher; the harness logic lives in dedicated modules:

| Module | Responsibility |
| :--- | :--- |
| `bench_config` | migration.yml parsing into `configschema::CampaignConfig` via `serde_yaml_ng`; strict environment-based configuration without in-code fallback literals: `BASE_URL`, `MODEL_ID`, `INFERENCE_API_KEY`, `BENCH_CONFIGS_DIR`, `BENCH_RESULTS_DIR`, injected at compile time via `build.rs` `cargo:rustc-env` emission over `.env` ([ADR-0007](../../decisions/0007-compile-time-env-ingestion.md)). |
| `bench_harness` | Blackboard seeding: inventory walk of `paths.input_legacy_dir` filtered by `stack.source_language`; header-as-parent DAG modeling; module source targeting in `build_solver` that directs `CargoHarness` to `src/{stem}.rs` when sibling module files exist or `src/lib.rs` for single-file crates. `seed_board` copies the verified `stack.target_build_cmd` / `stack.target_test_cmd` / `stack.legacy_test_harness_cmd` into `board.inventory.harness` so the S0 INTENT gate passes for real campaigns. |
| `bench_runner` | Engine run over `Box<dyn global_engine::ModuleSolver>` from `build_solver`, with campaign timeouts forwarded to `CargoHarness::with_timeouts`; the daemon overrides `max_rollouts` from `ariadne.engine_budget_per_module` (`bench_runner.rs:329-334`); other engine constants flow from `EngineConfig::default()` ([ADR-0012](../../decisions/0012-ariadne-constant-calibration.md)). `Ok(RunStop::Terminated)` maps to no error; `Ok(RunStop::HitlGatePending)` records `campaign parked on human gate <kind>` in the result `error` field because a headless run has no operator. Post-run delegation to `bench_validation`, `unimplemented!()` scan, and COCOMO-II metrics (`effort_pm = 2.4 * kloc^1.05`, `duration_months = 2.5 * effort_pm^0.38` from measured KLOC). Also installs the only logging setup: `tracing_subscriber::fmt().try_init()` producing ANSI/stdout formatted logs. |
| `bench_validation` | Post-run validation reusing `CargoHarness::run_command` without duplicate subprocess management; parses cargo test results and pass rates. |
| `bench_rules` | Rule-extraction driver with counted `ChatCompletion` adapter; each prompt carries the legacy source, sibling header text, and the target API surface captured from the workspace stubs, so extracted rule cards cite the exact Rust signatures and export obligations the crate must expose. |
| `bench_llm` | OpenAI-compatible transport over Rig's stock `openai` provider ([ADR-0006](../../decisions/0006-de-qwen-inference-transport.md)): `OpenAiCompatModel` wrapping `CompletionDefaults`; the strict fallback accepts only a fenced code block or a post-`</think` conclusion and otherwise propagates the original error instead of emitting reasoning prose as code. |
| `bench_batch` | `--benchmark-all` sequential iteration over `*.yml` under `BENCH_CONFIGS_DIR` (default `benchmarks/assets/configs/ReCodeAgent`); resume by result-JSON presence in `BENCH_RESULTS_DIR`; aggregate `summary.json`. |

## 2. Workspace Reset Semantics

Before the engine starts, the target workspace is reset to a pristine dataset copy: the stub layout is normalized — sibling stub module files are inlined into `src/lib.rs` and removed from disk — so the `unimplemented!()` count and every compiled artifact live in the single file the evaluation harness replaces. `unimplemented_before` counts stubs in the pristine copy and `unimplemented_after_reset` counts stubs after normalization; the difference is reported as `unimplemented_remaining`.

## 3. Header-as-Parent DAG Modeling

DAG vertices are implementation source files only. Header and test-tree files never become modules. Header files instead become pre-hardened parent vertices with edge contracts derived from header contents and directed edges `(header, module)`, so downstream modules see the exact upstream interface obligations without headers entering the translation set.

## 4. Honesty Gate

A run whose engine completes but issued zero inference requests (`llm_requests == 0`) is recorded as dishonest and fails the campaign. The `CountingCompletionModel` atomic request counter supplies the audit. A failed LLM exchange never degrades to reasoning prose; the original error propagates. No mock output exists anywhere in the benchmark paths.

## 5. Result Artifacts (`BenchResult`)

Per-project result JSON lands in `BENCH_RESULTS_DIR` (default `benchmarks/results/daemon`) as `<label>.json`, where the label derives from the `workspace_root` path segments after `outputs/` (e.g. `outputs/matrix/crust-vec` → `crust-vec`). `BenchResult` fields:

| Field | Meaning |
| :--- | :--- |
| `campaign_id` | Campaign identifier from migration.yml. |
| `wall_time_ms` | Total campaign wall-clock time. |
| `engine_wall_time_ms` | Engine execution wall-clock time. |
| `llm_requests` | Counted inference requests; zero implies a dishonest failure. |
| `unimplemented_before` | `unimplemented!()` count in the pristine dataset copy. |
| `unimplemented_after_reset` | Count after stub normalization. |
| `compilation_success` | `cargo check` outcome under configured timeout. |
| `test_pass_rate` | Characterization/parity test pass rate from `cargo test`. |
| `unimplemented_remaining` | Stub count remaining in the final target. |
| `solver_dispatches` | S5 module dispatch count. |
| `worker_executions` | Discovery phase-shard execution count. |
| `structural_retries` | Stage-one `CRITIC_RETRY` re-dispatch count. |
| `macro_states_visited` | Macro-statechart transition count. |
| `total_lines`, `code_lines` | `scc` line metrics. |
| `cocomo_effort_pm`, `cocomo_duration_months` | COCOMO-II estimates. |
| `error` | Failure detail, if any. |

Batch mode writes `summary.json` alongside the per-project files with `{total, compilation_rate, mean_test_pass_rate, mean_wall_time_ms, total_wall_time_ms, per_category}`. Batch resume skips projects whose result JSON already exists in `BENCH_RESULTS_DIR`. Operator-facing artifact locations: [../product-config.md](../product-config.md).

## 6. LoC-Tier Inference Scaling

All 118 benchmark campaign configs are retuned by legacy-codebase LoC tier ([ADR-0008](../../decisions/0008-loc-tier-inference-scaling.md)); rationale table in `benchmarks/README.md`. Model id: served alias `swift-1.5-qwen3.8-27b` over the local `ninfer` continuous-batching stack.

| Tier | Legacy LoC | Projects | max_rollouts | Shards | max_tokens | Timeouts (s) | Self-Heal |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| T1 | ≤ 500 | 24 | 16 | 2 | 32768 | 600 | 3 |
| T2 | 500–2000 | (see `benchmarks/README.md`) | ↑ | ↑ | ↑ | ↑ | ↑ |
| T3 | 2000–10000 | (see `benchmarks/README.md`) | ↑ | ↑ | ↑ | ↑ | ↑ |
| T4 | > 10000 | 5 | 96 | 6 | 49152 | 1800 | 8 |

Scaling runs monotonically from the T1 row to the T4 row: `max_rollouts` 16 → 96, shards 2 → 6 (ninfer slot cap), `max_tokens` 32768 → 49152 (75% of the 65536 context), timeouts 600 → 1800s, self-heal retries 3 → 8.

## 7. Known Divergence & Reconciliation Roadmap

Components whose implementation and `METHODOLOGY.md` differ, with the reconciliation direction:

1. **Global blackboard (Definition 1)**: implemented in `global-blackboard` as ten modules per [engine-and-blackboard.md §3.1.1](engine-and-blackboard.md); parity holds.
2. **Deterministic macro engine (Algorithm 1)**: implemented in `global-engine`; stage-two semantic remediation (`RemediateLocal`) and the `solver_dispatches`/`worker_executions` counter split are in-tree refinements of the methodology's retry loop.
3. **ARIADNE solver adapter (Algorithm 2 boundary)**: `AriadneSolver` in `holon-engine/src/solver.rs` maps `EngineTypestateOutcome` onto `global_engine::SolverOutcome`.
4. **Search constants**: in-tree defaults deliberately deviate from methodology τ=0.35/γ=0.15; calibrated per [ADR-0012](../../decisions/0012-ariadne-constant-calibration.md).
5. **Inference transport**: Rig stock OpenAI provider with `OpenAiCompatModel`/`CountingCompletionModel` ([ADR-0006](../../decisions/0006-de-qwen-inference-transport.md)); no per-model transport in tree.
6. **Config schema**: single canonical `configschema::CampaignConfig` parser ([ADR-0009](../../decisions/0009-canonical-campaign-configschema.md)); `ariadne.max_rollouts` remains an accepted serde alias for the 118 generated configs.
7. **configschema consumer wiring**: `ariadne.llm_backend` and `ariadne.cargo_harness` are schema-complete but the daemon reads `inference.*` and `ariadne.engine_budget_per_module` only; consuming these sections in the daemon is pending.
8. **Daemon logging**: single ANSI/stdout `tracing_subscriber` sink; a JSONL file sink and SIGINT handling are planned but not implemented.
