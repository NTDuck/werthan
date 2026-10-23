# Benchmark Harness (`werðan/bins/daemon`)

Entry point: [../tech-spec.md](../tech-spec.md). This file documents the daemon benchmark runner modules, workspace semantics, honesty gate, result artifacts, and LoC-tier scaling ([ADR-0008](../../decisions/0008-loc-tier-inference-scaling.md)).

## 1. Daemon `bench_*` Module Map

`werðan/bins/daemon/src/lib.rs` declares the harness modules (`bench_batch`, `bench_config`, `bench_harness`, `bench_llm`, `bench_report`, `bench_rules`, `bench_runner`, `bench_smoke`, `bench_validation`) as the public library surface; `src/main.rs` parses arguments and dispatches into this library. The harness logic lives in dedicated modules:

| Module | Responsibility |
| :--- | :--- |
| `bench_config` | migration.yml parsing into `configschema::CampaignConfig` via `serde_yaml_ng`; strict environment-based configuration without in-code fallback literals: `BASE_URL`, `MODEL_ID`, `INFERENCE_API_KEY`, `BENCH_CONFIGS_DIR`, `BENCH_RESULTS_DIR`, injected at compile time via `build.rs` `cargo:rustc-env` emission over `.env` ([ADR-0007](../../decisions/0007-compile-time-env-ingestion.md)). |
| `bench_harness` | Blackboard seeding: inventory walk of `paths.input_legacy_dir` filtered by `stack.source_language`; header-as-parent DAG modeling; module source targeting in `build_solver` that directs `CargoHarness` to `src/{stem}.rs` when sibling module files exist or `src/lib.rs` for single-file crates. `seed_board` copies the verified `stack.target_build_cmd` / `stack.target_test_cmd` / `stack.legacy_test_harness_cmd` into `board.inventory.harness` so the S0 INTENT gate passes for real campaigns. |
| `bench_runner` | Engine run over `Box<dyn global_engine::ModuleSolver>` from `build_solver`, with campaign timeouts forwarded to `CargoHarness::with_timeouts`; the daemon overrides `max_rollouts` from `ariadne.engine_budget_per_module` (`bench_runner.rs:329-334`); other engine constants flow from `EngineConfig::default()` ([ADR-0012](../../decisions/0012-ariadne-constant-calibration.md)). `Ok(RunStop::Terminated)` maps to no error; `Ok(RunStop::HitlGatePending)` records `campaign parked on human gate <kind>` in the result `error` field because a headless run has no operator. Post-run delegation to `bench_validation`, `unimplemented!()` scan, and COCOMO-II metrics (`effort_pm = 2.4 * kloc^1.05`, `duration_months = 2.5 * effort_pm^0.38` from measured KLOC). Also installs the only logging setup: `tracing_subscriber::fmt().try_init()` producing ANSI/stdout formatted logs. |
| `bench_report` | Campaign report schema for the THESEUS contract runner (VAL-DAEMON-01/03/04): `CampaignReport` with `ModuleOutcome` entries, the FIFO `event_log`, hardened/failed totals, and the `BackendEcho` of the backend configuration actually used; `run_stub_campaign` executes the deterministic stub campaign over a seeded board. |
| `bench_smoke` | Smoke verification entrypoint shared by the binary and the contract tests: `run_smoke_cli` probes the compiled default endpoint, `run_smoke_verification_once` probes the campaign-configured endpoint, and `smoke_endpoint_probe` performs the `GET {base_url}/models` request behind the `SmokeReport` (base URL, served model ids). Exit code 1 on probe failure. |
| `bench_validation` | Post-run validation reusing `CargoHarness::run_command` without duplicate subprocess management; parses cargo test results and pass rates. |
| `bench_rules` | Rule-extraction driver with counted `ChatCompletion` adapter; each prompt carries the legacy source, sibling header text, and the target API surface captured from the workspace stubs, so extracted rule cards cite the exact Rust signatures and export obligations the crate must expose. |
| `bench_llm` | OpenAI-compatible transport owned by the daemon ([ADR-0016](../../decisions/0016-daemon-owned-transport-deadlines.md)): `OpenAiCompatModel` wrapping `CompletionDefaults` over a daemon-built `reqwest::Client` with client-level `timeout`/`connect_timeout`; requests serialize from Rig's `CompletionRequest`, responses parse `content` with the `reasoning_content` fallback channel; the strict fallback accepts only a fenced code block or a post-`</think` conclusion and otherwise propagates the original error instead of emitting reasoning prose as code. |
| `bench_batch` | `--benchmark-all` sequential iteration over `*.yml` under `BENCH_CONFIGS_DIR` (default `benchmarks/assets/configs/ReCodeAgent`); resume by result-JSON presence in `BENCH_RESULTS_DIR`; aggregate `summary.json`. |

## 2. Workspace Reset Semantics

Before the engine starts, the target workspace is reset to a pristine dataset copy: the stub layout is normalized — sibling stub module files are inlined into `src/lib.rs` and removed from disk — so the `unimplemented!()` count and every compiled artifact live in the single file the evaluation harness replaces. `unimplemented_before` counts stubs in the pristine copy and `unimplemented_after_reset` counts stubs after normalization; the difference is reported as `unimplemented_remaining`. The reset maps the `benchmarks/outputs/` prefix of `paths.workspace_root` onto `benchmarks/assets/ReCodeAgent/data/tool_projects/` to locate the pristine dataset copy; a workspace root that is not under `benchmarks/outputs/` or a missing pristine dataset directory fails the reset. A missing workspace root is not an error: the root already names the pristine state the reset targets, so the reset proceeds directly to the dataset copy instead of reporting a removal failure.

## 3. Header-as-Parent DAG Modeling

DAG vertices are implementation source files only. Header and test-tree files never become modules. Header files instead become pre-hardened parent vertices with edge contracts derived from header contents and directed edges `(header, module)`, so downstream modules see the exact upstream interface obligations without headers entering the translation set.

## 4. Honesty Gate

A run whose engine completes but issued zero inference requests (`llm_requests == 0`) is recorded as dishonest and fails the campaign. The `CountingCompletionModel` atomic request counter supplies the audit. A failed LLM exchange never degrades to reasoning prose; the original error propagates. No mock output exists anywhere in the benchmark paths.

## 5. Result Artifacts (`BenchResult`)

Smoke verification (`--smoke`, shared with `--dry-run`) does not produce a result artifact: `bench_smoke::run_smoke_cli` resolves the inference URL (campaign `inference.base_url`, falling back to the compiled `BASE_URL` default), issues one `GET {base_url}/models` probe, prints the reachable base URL and the first served model id, and exits nonzero on failure.

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

The validation contracts (VAL-DAEMON-01/03/04) consume a separate report schema, `bench_report::CampaignReport`, produced by `bench_report::run_stub_campaign` over a seeded board and serialized under the same serde rules:

| Field | Meaning |
| :--- | :--- |
| `campaign_id` | Campaign identifier passed to `run_stub_campaign`. |
| `module_outcomes` | One `ModuleOutcome` per DAG vertex. Each entry carries `module_id`, the final per-module micro-state (`final_micro_state`, `ModuleMicroState`), the `artifact_reference` of the hardened artifact when the module hardened, and a `blocker` description when the module failed to harden. |
| `event_log` | FIFO event log: one entry per recorded engine transition, in dispatch order. |
| `hardened_count` | Count of modules that reached the hardened micro-state. |
| `failed_count` | Count of modules that failed to harden. |
| `backend` | `BackendEcho` of the backend configuration actually used: `backend_url` (configured inference base URL, or the stub marker `stub://local` when the campaign runs the deterministic stub solver), `model` (configured model identifier, when present), and `stub_backend` (`true` when the campaign ran against the deterministic stub solver). |
| `solver_dispatches` | Solver dispatch count reported by the engine. |
| `error` | Failure description, or none on success. A run parked on a human gate records `campaign parked on human gate before termination`; an engine failure also marks every unhardened module outcome with the engine error as its blocker. |

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
5. **Inference transport**: daemon-owned reqwest client with client-level deadlines inside `OpenAiCompatModel`/`CountingCompletionModel` ([ADR-0016](../../decisions/0016-daemon-owned-transport-deadlines.md)); no per-model transport in tree.
6. **Config schema**: single canonical `configschema::CampaignConfig` parser ([ADR-0009](../../decisions/0009-canonical-campaign-configschema.md)); `ariadne.max_rollouts` remains an accepted serde alias for the 118 generated configs.
7. **configschema consumer wiring**: `ariadne.llm_backend` and `ariadne.cargo_harness` are schema-complete but the daemon reads `inference.*` and `ariadne.engine_budget_per_module` only; consuming these sections in the daemon is pending.
8. **Daemon logging**: single ANSI/stdout `tracing_subscriber` sink; a JSONL file sink and SIGINT handling are planned but not implemented.
