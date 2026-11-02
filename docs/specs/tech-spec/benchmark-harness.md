# Benchmark Harness (`werðan/bins/daemon`)

Entry point: [../tech-spec.md](../tech-spec.md). This file documents the daemon benchmark runner modules, workspace semantics, honesty gate, result artifacts, and LoC-tier scaling ([ADR-0008](../../decisions/0008-loc-tier-inference-scaling.md)).

## 1. Daemon `bench_*` Module Map

`werðan/bins/daemon/src/lib.rs` declares the harness modules (`bench_batch`, `bench_config`, `bench_harness`, `bench_llm`, `bench_report`, `bench_rules`, `bench_runner`, `bench_smoke`, `bench_validation`) as the public library surface; `src/main.rs` parses arguments and dispatches into this library. The harness logic lives in dedicated modules:

| Module | Responsibility |
| :--- | :--- |
| `bench_config` | migration.yml parsing into `configschema::CampaignConfig` via `serde_yaml_ng`; strict environment-based configuration without in-code fallback literals: `BASE_URL`, `MODEL_ID`, `INFERENCE_API_KEY`, `BENCH_CONFIGS_DIR`, `BENCH_RESULTS_DIR`, injected at compile time via `build.rs` `cargo:rustc-env` emission over `.env` ([ADR-0007](../../decisions/0007-compile-time-env-ingestion.md)). |
| `bench_harness` | Blackboard seeding: inventory walk of `paths.input_legacy_dir` filtered by `stack.source_language`; implementation module DAG modeling (excluding headers and test trees from vertices and inventory); module source targeting in `build_solver` that directs `CargoHarness` to `src/{stem}.rs` when sibling module files exist or `src/lib.rs` for single-file crates. `seed_board` copies the verified `stack.target_build_cmd` / `stack.target_test_cmd` / `stack.legacy_test_harness_cmd` into `board.inventory.harness` so the S0 INTENT gate passes for real campaigns. |
| `bench_runner` | Engine run over `Box<dyn global_engine::ModuleSolver>` from `build_solver`, with campaign timeouts forwarded to `CargoHarness::with_timeouts`; the daemon overrides `max_rollouts` from `ariadne.engine_budget_per_module` (`bench_runner.rs:329-334`); other engine constants flow from `EngineConfig::default()` ([ADR-0012](../../decisions/0012-ariadne-constant-calibration.md)). `Ok(RunStop::Terminated)` maps to no error; `Ok(RunStop::HitlGatePending)` records `campaign parked on human gate <kind>` in the result `error` field because a headless run has no operator. Post-run delegation to `bench_validation` (including differential verification against legacy harness when configured), `unimplemented!()` scan, and COCOMO-II metrics (`effort_pm = 2.4 * kloc^1.05`, `duration_months = 2.5 * effort_pm^0.38` from measured KLOC). When differential verification indicates divergence (`differential_equivalent == Some(false)`), the failure classification records in `error`. Logging initializes via `tracing_subscriber` configured with `EnvFilter` supporting `WERTHAN_LOG_LEVEL` and `RUST_LOG`. |
| `bench_report` | Campaign report schema for the THESEUS contract runner (VAL-DAEMON-01/03/04): `CampaignReport` with `ModuleOutcome` entries, the FIFO `event_log`, hardened/failed totals, and the `BackendEcho` of the backend configuration actually used; `run_stub_campaign` executes the deterministic stub campaign over a seeded board. |
| `bench_smoke` | Smoke verification entrypoint shared by the binary and the contract tests: `run_smoke_cli` probes the compiled default endpoint, `run_smoke_verification_once` probes the campaign-configured endpoint, and `smoke_endpoint_probe` performs the `GET {base_url}/models` request behind the `SmokeReport` (base URL, served model ids). Exit code 1 on probe failure. |
| `bench_validation` | Post-run validation reusing `CargoHarness::run_command` without duplicate subprocess management; runs the configured `stack.target_build_cmd` / `stack.target_test_cmd` (parsed via `CargoHarness::parse_command`) and parses their results into pass rates. When `stack.legacy_test_harness_cmd` is configured, executes differential verification via `differential::execute_differential_verification` and populates `differential_equivalent` and `differential_classification` in `WorkspaceValidation`. External benchmark pass rate evaluation (`test_pass_rate`) is strictly decoupled from internal characterization tests: `compute_pass_rate` returns `0.0` when `denominator == 0` (no `test result:` lines parsed). Standard `cargo test` skips `[[bin]]` targets by default when no lib/unit tests exist; benchmark configs MUST set `target_test_cmd` to execute the project's test target (e.g. `cargo test --bins` or `cargo test --bin <test_bin>`). `parse_test_counts` aggregates across all `test result:` lines from all test binaries invoked. Both command outcomes log at INFO: program, exit code (or the timeout/error text), and a ≤400-char output tail on failure. |
| `bench_rules` | Rule-extraction driver with counted `ChatCompletion` adapter; each prompt carries the legacy source, sibling header text, and the target API surface captured from the workspace stubs, so extracted rule cards cite the exact Rust signatures and export obligations the crate must expose. |
| `bench_llm` | OpenAI-compatible transport owned by the daemon ([ADR-0016](../../decisions/0016-daemon-owned-transport-deadlines.md)): `OpenAiCompatModel` wrapping `CompletionDefaults` over a daemon-built `reqwest::Client` with client-level `timeout`/`connect_timeout`; requests serialize from Rig's `CompletionRequest` with `chat_template_kwargs: { enable_thinking: false }` (reasoning traces burn the whole token budget under concurrency and never surface as output), responses use non-empty `content` only — unknown channels such as `reasoning_content` deserialize as ignored raw JSON: deliberation is never surfaced as an answer. The strict acceptance policy: fenced code from any language (rejecting ellipsis-sketch bodies where a full line is a bare or brace-wrapped ellipsis, ASCII or Unicode), bare JSON object/array answers (rule extraction contract), the post-`</think` conclusion when reasoning is embedded in content, and otherwise the bare trimmed content itself (thinking-off content is the direct answer; downstream syntax gates judge non-code honestly). |
| `bench_batch` | `--benchmark-all` sequential iteration over `*.yml` under `BENCH_CONFIGS_DIR` (default `benchmarks/assets/configs/ReCodeAgent`); resume by result-JSON presence in `BENCH_RESULTS_DIR`; aggregate `summary.json`. |

## 2. Workspace Reset Semantics

Before the engine starts, the target workspace is reset to a pristine dataset copy: after copying the dataset into the workspace root, `purge_implementation_stubs` deletes all implementation stub files under `src/` (regular files matching target source extensions `.rs`, `.js`, or `.py`, excluding `lib.rs`) so translation starts strictly from scratch rather than filling in existing stubs. `unimplemented_before` counts stubs in the pristine copy and `unimplemented_after_reset` counts stubs remaining after reset and stub purging; the final count is reported as `unimplemented_remaining`. The reset maps the `benchmarks/outputs/` prefix of `paths.workspace_root` onto `benchmarks/assets/ReCodeAgent/data/tool_projects/` to locate the pristine dataset copy; a workspace root that is not under `benchmarks/outputs/` or a missing pristine dataset directory fails the reset. A missing workspace root is not an error: the root already names the pristine state the reset targets, so the reset proceeds directly to the dataset copy instead of reporting a removal failure.

## 3. Logging, Telemetry, and Observability

The benchmark harness establishes a structured tracing contract using the `tracing` ecosystem. Diagnostic events stream to `stdout` via `tracing_subscriber` with dynamic level filtering.

### 3.1 Filter Configuration

The subscriber builds an `EnvFilter` from two sources:
1. **Compile-Time Default**: `WERTHAN_LOG_LEVEL` (default: `"info"`), loaded from `.env` at build time by `envload` and read via `env!("WERTHAN_LOG_LEVEL")`.
2. **Runtime Dynamic Override**: `RUST_LOG` environment variable. When set, `RUST_LOG` overrides the compile-time default at process startup without requiring recompilation (for example, `RUST_LOG="bench_llm=debug,holon_engine=trace"`).

### 3.2 Structured Logging Targets

Tracing events partition strictly across five canonical targets:

| Target | Subsystem Scope |
| :--- | :--- |
| `global_engine` | Macro-statechart transitions, S0–S6 phase entries, DAG task dispatch envelopes, and batch guard evaluations. |
| `holon_engine` | Holon solver execution, M0–M5 micro-state transitions, ARIADNE MCTS rollouts, UCB exploration choices, and stagnation detection. |
| `bench_llm` | OpenAI-compatible HTTP transport requests, completion latencies, token consumption counts, and retry backoffs. |
| `tools` | Shared agent tool invocations, input arguments, execution durations, and exit statuses. |
| `evaluator` | `CargoHarness` compilation and test commands, compiler error diagnostics, pass rate computations, and Gate 5 proof receipts. |

### 3.3 Event Guarantees

The harness guarantees the emission of the following diagnostic events across execution:

1. **Macro Phase Entries (S0–S6)**: Target `global_engine` emits an `info` event upon entering each macro phase: S0 (Intent), S1 (Assess), S2 (Map DAG), S3 (Extract Rules), S4 (Brief Contracts), S5 (Holon Modernization), and S6 (System Verify).
2. **Module Micro-State Transitions (M0–M5)**: Target `holon_engine` emits an `info` event on every module micro-state transition: M0 (Init), M1 (Hypothesize), M2 (Rollout/Ariadne), M3 (Patch/Repair), M4 (Verify), and M5 (Hardened/Done).
3. **LLM Completion Latency & Token Usage**: Target `bench_llm` emits a `debug` event on each successful completion recording duration (milliseconds), prompt tokens, completion tokens, and total tokens. Transport errors and retries emit `warn` events with attempt counts and backoff intervals.
4. **Rollout Progression**: Target `holon_engine` emits an `info` event on each rollout step recording module ID, rollout index, candidate compilation outcome, characterization test pass rate, and computed scalar reward.
5. **Compilation Error Snippets**: Target `evaluator` emits a `warn` event on compiler failures containing truncated compiler error diagnostics (stderr snippet) to aid debugging without flooding log buffers.
6. **Gate 5 Proof Acceptance / Rejection**: Target `evaluator` emits an `info` event upon formal test validation recording total test counts, passed counts, failed counts, computed `test_pass_rate`, and explicit acceptance or rejection under the Gate 5 invariant.
7. **Tool Executions**: Target `tools` emits a `debug` event on each tool call recording tool name, key input arguments, execution duration, and success/failure status.

## 4. Header and Test-Tree Exclusion from DAG Vertices & Self-Contract Seeding

DAG vertices ($V$) and repository inventory ($\mathcal{I}_{\text{inv}}$) carry implementation source files only. Header and test-tree files never become modules or vertices. Symmetrically excluding headers from both $\mathcal{I}_{\text{inv}}$ and $V$ preserves the $S_2$ coverage invariant ($V \equiv \mathcal{I}_{\text{inv}}$) while preventing the engine from dispatching C header declarations as standalone compilation units (which would otherwise exhaust the futile dispatch cap).

Header declarations and target API surfaces reach codegen through two formal mechanisms:
1. **Rule Extraction Ingestion**: `bench_rules::sibling_header_text` matches companion headers across directory boundaries (e.g. matching `include/{stem}.h` to `src/{stem}.c` via file stem equivalence) and injects declared interfaces into rule extraction prompts alongside the target workspace's existing Rust stub signatures. `bench_rules::target_api_surface` resolves the pristine dataset directory (`dataset_dir`) before workspace purging, ensuring target signatures are preserved.
2. **Self-Contract Seeding & Target Interface Alignment**: `bench_harness::seed_contracts` iterates over DAG vertices $V$, discovers matching companion header artifacts in `board.artifacts`, and extracts exported target interface declarations (`pub fn ...`, `pub struct ...`, `pub enum ...`) from matching target files in `dataset_dir`. These obligations are aggregated and seeded as self-contracts $\mathcal{C}(v_i, v_i)$ on `board.contracts`. Upon module activation, `hydrate_problem_model` populates these self-contracts into `ProblemModelSlice.export_obligations`, and the synthesis and repair system prompts explicitly steer agents to adhere strictly to these exported signatures so that integration tests compile.
## 5. Honesty Gate

The daemon runs headless from a declarative campaign config file (`migration.yml`) against a target project and invokes the modernization system honestly: every rollout and phase dispatches real LLM inference requests without synthetic mocks or simulated shortcuts. A run whose engine completes but issued zero inference requests (`llm_requests == 0`) is recorded as dishonest and fails the campaign. The `CountingCompletionModel` atomic request counter supplies the audit. A failed LLM exchange never degrades to reasoning prose; the original error propagates. No mock output exists anywhere in the benchmark paths.

## 6. Result Artifacts (`BenchResult`)

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
| `compilation_success` | Workspace artifact state only: configured build command (`stack.target_build_cmd`) outcome under the configured timeout AND zero remaining stubs. The engine terminal error reports in `error` and does not poison this flag (benchmark acceptance: compilation success AND `test_pass_rate` ≥ 0.5). |
| `test_pass_rate` | Pass rate from the configured test command (`stack.target_test_cmd`); `parse_test_counts` aggregates across all `test result:` lines from all test binaries invoked. Internal characterization tests generated/executed by the Holon solver (e.g. `tests/characterization.rs`) do NOT count toward this metric. If `denominator == 0` (no `test result:` lines parsed), `compute_pass_rate` returns `0.0`. Acceptance criteria: compilation success AND `test_pass_rate` ≥ 0.50 (or 100%). |
| `differential_equivalent` | Differential behavioral equivalence outcome (`Some(true)` = equivalent, `Some(false)` = diverged, `None` = skipped/unconfigured). When divergence occurs (`Some(false)`), failure detail records in `error`. |
| `unimplemented_remaining` | Stub count remaining in the final target. |
| `solver_dispatches` | S5 module dispatch count. |
| `worker_executions` | Discovery phase-shard execution count. |
| `structural_retries` | Stage-one `CRITIC_RETRY` re-dispatch count. |
| `macro_states_visited` | Macro-statechart transition count. |
| `total_lines`, `code_lines` | `scc` line metrics. |
| `cocomo_effort_pm`, `cocomo_duration_months` | COCOMO-II estimates. |
| `error` | Failure detail, if any (engine termination errors, honesty violations, or differential verification divergence). |

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

## 7. Gate 5 (Proof Receipt) & Benchmark Validation Invariants

Gate 5 governs the formal validation of modernized artifacts against external ground-truth criteria.

### 7.1 Critical Metric Invariant

Internal characterization tests generated or executed by the Holon solver (e.g. `tests/characterization.rs`, $B_{CT}$) do **NOT** count toward the external benchmark test pass rate metric (`test_pass_rate`). Only the external benchmark test suites shipped with the dataset (executed via `target_test_cmd` in `bench_validation.rs`) count toward the evaluation criteria. A run is only acceptable if the external benchmark test suite achieves `test_pass_rate >= 0.50` (or 100%).

### 7.2 Gate 5 Proof Receipt & Test Suite Immutability

$$\text{TargetTestSuite} \cap \text{RepairCandidatePool} = \emptyset$$

Under the THESEUS verification methodology (Criteria 2, [ADR-0015](../../decisions/0015-methodology-validation-contracts.md)), characterization tests and external test harnesses are immutable formal axioms. In contrast to systems that allow agents to mutate test assertions or generate dummy passing tests when confronted with compiler or runtime errors, Gate 5 prohibits the engine and agents from altering test files or expected test outputs. An honest benchmark receipt reflecting `test_pass_rate: 0.0` or a compilation defect strictly outranks a fabricated pass obtained via test tampering.

### 7.3 Technical Mechanism & Benchmark Command Mandate

1. **Zero-Test Handling**: `bench_validation.rs::compute_pass_rate` returns `0.0` when `denominator == 0` (no `test result:` lines parsed from the command output). A run with zero executed tests cannot claim a pass.
2. **Binary Target Test Execution**: Standard `cargo test` skips `[[bin]]` test targets by default when no library or unit tests exist in the translated workspace.
3. **Target Test Command Mandate**: Benchmark configs (`migration.yml`) MUST set `stack.target_test_cmd` to a command that explicitly executes the project's external test target (e.g. `cargo test --bins` or `cargo test --bin <test_bin>`).
4. **Multi-Target Count Aggregation**: `bench_validation.rs::parse_test_counts` aggregates passed and failed counts across all `test result:` lines emitted by all test binaries invoked by `target_test_cmd`.

## 8. LoC-Tier Inference Scaling

All 118 benchmark campaign configs are retuned by legacy-codebase LoC tier ([ADR-0008](../../decisions/0008-loc-tier-inference-scaling.md)); rationale table in `benchmarks/README.md`. Model id: served alias `swift-1.5-qwen3.8-27b` over the local `ninfer` continuous-batching stack.

| Tier | Legacy LoC | Projects | max_rollouts | Shards | max_tokens | Timeouts (s) | Self-Heal |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| T1 | ≤ 500 | 24 | 16 | 2 | 32768 | 600 | 3 |
| T2 | 500–2000 | (see `benchmarks/README.md`) | ↑ | ↑ | ↑ | ↑ | ↑ |
| T3 | 2000–10000 | (see `benchmarks/README.md`) | ↑ | ↑ | ↑ | ↑ | ↑ |
| T4 | > 10000 | 5 | 96 | 6 | 49152 | 1800 | 8 |

Scaling runs monotonically from the T1 row to the T4 row: `max_rollouts` 16 → 96, shards 2 → 6 (ninfer slot cap), `max_tokens` 32768 → 49152 (75% of the 65536 context), timeouts 600 → 1800s, self-heal retries 3 → 8.

## 9. Known Divergence & Reconciliation Roadmap

Components whose implementation and `METHODOLOGY.md` differ, with the reconciliation direction:

1. **Global blackboard (Definition 1)**: implemented in `global-blackboard` as ten modules per [engine-and-blackboard.md §3.1.1](engine-and-blackboard.md); parity holds.
2. **Deterministic macro engine (Algorithm 1)**: implemented in `global-engine`; stage-two semantic remediation (`RemediateLocal`) and the `solver_dispatches`/`worker_executions` counter split are in-tree refinements of the methodology's retry loop.
3. **ARIADNE solver adapter (Algorithm 2 boundary)**: `AriadneSolver` in `holon-engine/src/solver.rs` maps `EngineTypestateOutcome` onto `global_engine::SolverOutcome`.
4. **Search constants**: in-tree defaults deliberately deviate from methodology τ=0.35/γ=0.15; calibrated per [ADR-0012](../../decisions/0012-ariadne-constant-calibration.md).
5. **Inference transport**: daemon-owned reqwest client with client-level deadlines inside `OpenAiCompatModel`/`CountingCompletionModel` ([ADR-0016](../../decisions/0016-daemon-owned-transport-deadlines.md)); no per-model transport in tree.
6. **Config schema**: single canonical `configschema::CampaignConfig` parser ([ADR-0009](../../decisions/0009-canonical-campaign-configschema.md)); `ariadne.max_rollouts` remains an accepted serde alias for the 118 generated configs.
7. **configschema consumer wiring**: `ariadne.llm_backend` and `ariadne.cargo_harness` are schema-complete but the daemon reads `inference.*` and `ariadne.engine_budget_per_module` only; consuming these sections in the daemon is pending.
8. **Daemon logging**: ANSI/stdout tracing subscriber configured with `EnvFilter` (`WERTHAN_LOG_LEVEL` / `RUST_LOG`) and structured event targets; a JSONL file sink and SIGINT handling are planned but not implemented.
