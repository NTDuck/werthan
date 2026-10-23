# werðan Operator Configuration & Invocation

Behavior-facing overview: [product.md](product.md). Canonical schema definition: [tech-spec/tools-and-configschema.md](tech-spec/tools-and-configschema.md).

## 1. Migration Configuration (`migration.yml`)

The configuration file accepted by `/start`, `/load-config`, and the daemon binary adheres to the canonical schema `configschema::CampaignConfig` (crate `werðan/libs/shared/configschema`, [ADR-0009](../decisions/0009-canonical-campaign-configschema.md)). Example:

```yaml
campaign_id: "modernize-campaign-001"
execution_mode: "hitl"                        # "hitl" | "full_auto"

paths:
  input_legacy_dir: "./legacy-source"
  workspace_root: "./modernized-target"
  off_limits_globs:
    - "**/.git/**"
    - "**/target/**"

stack:
  source_language: "cpp14"
  binary_artifacts: []
  use_ghidra_rea: true
  target_language: "rust"
  target_edition_or_framework: "2024"
  target_build_cmd: "cargo check"
  target_test_cmd: "cargo test"
  legacy_test_harness_cmd: "ctest"

inference:
  max_parallel_shards: 2
  base_url: "http://localhost:8000/v1"
  model: "swift-1.5-qwen3.8-27b"
  max_shards: 2
  inference_max_tokens: 32768
  inference_temperature: 0.2
  inference_timeout_secs: 600
  build_timeout_secs: 120
  test_timeout_secs: 120

verification:
  max_self_heal_retries: 3
  require_byte_identical_io: false

hitl:
  gate_on_reimagine: true                     # Human signoff required after S4 before S5
  gate_on_macro_regression: true              # Human signoff required before rolling S5 -> S2
  extrinsic_conflict_escalation: "auto_negotiate" # "auto_negotiate" | "ask_human"

ariadne:
  engine_budget_per_module: 24                # Maximum rollouts (B_max) in state M2; serde alias: max_rollouts
  ucb_exploration_c: 1.414                    # Exploration constant for Softmax-UCB
  softmax_temperature: 0.50                   # Softmax temperature for prior sampling
  reward_weights:
    compile: 0.20                             # w_comp weight for clean compilation
    characterization_parity: 0.60             # w_char weight for equivalence test pass rate
    hardening_score: 0.20                     # w_hard weight for semantic rule adherence
  llm_backend:                                # Schema-complete; daemon consumption pending (see §1.1)
    base_url: "http://127.0.0.1:8092/v1"
    model: "swift-1.5-qwen3.8-27b"
    max_tokens: 32768
    temperature: 0.2
    timeout_secs: 600
  cargo_harness:                              # Schema-complete; daemon consumption pending (see §1.1)
    build_timeout_secs: 120
    test_timeout_secs: 120
```

The in-code defaults for the `ariadne` section are `EngineConfig::default()`: `max_rollouts` 16, `ucb_c_exp` 1.414, `gamma_prior` 0.15, `softmax_temperature` 0.35, `generative_temperature` 0.20, reward weights 0.20/0.60/0.20 ([ADR-0012](../decisions/0012-ariadne-constant-calibration.md)). The daemon maps the parsed `ariadne` tuning knobs into the solver `EngineConfig` in `build_solver`: `engine_budget_per_module` → `max_rollouts` (when nonzero), `ucb_exploration_c` → `ucb_c_exp`, `softmax_temperature` → `softmax_temperature`, and `reward_weights` (`characterization_parity` → `parity`, `hardening_score` → `harden`).

### 1.1 Schema-Complete but Pending Consumption

The `ariadne.llm_backend` and `ariadne.cargo_harness` sections parse into the schema but the daemon does not read them yet: the inference transport comes from the `inference.*` fields (`bench_llm::OpenAiCompatModel::from_inference`) and the harness timeouts from `inference.build_timeout_secs` / `inference.test_timeout_secs`. The two sections exist for schema completeness; consumer wiring is pending. Generate drafts with `/init`; edit them with any YAML tooling; load them back with `/load-config`.

## 2. Environment Variables

All runtime keys are ingested at compile time: every workspace `build.rs` calls `::envload::load()`, which walks up from `CARGO_MANIFEST_DIR`, ingests the nearest `.env`, and emits `cargo:rustc-env=KEY=VALUE` per pair; code reads via `env!("KEY")` ([ADR-0007](../decisions/0007-compile-time-env-ingestion.md), [ADR-0011](../decisions/0011-env-config-parameters.md)). Defaults are documented in `.env.example`.

### 2.1 Consumed Today

| Variable | Default | Consumer | Purpose |
| :--- | :--- | :--- | :--- |
| `BASE_URL` | `http://127.0.0.1:8092/v1` | daemon `bench_llm` | OpenAI-compatible chat completions endpoint. |
| `MODEL_ID` | `swift-1.5-qwen3.8-27b` | daemon `bench_llm` | Served model identifier. |
| `INFERENCE_API_KEY` | `""` | daemon `bench_llm` | Bearer credential; local servers ignore it. |
| `BENCH_CONFIGS_DIR` | `benchmarks/assets/configs/ReCodeAgent` | daemon `bench_config` | Default directory searched by `--benchmark-all`. |
| `BENCH_RESULTS_DIR` | `benchmarks/results/daemon` | daemon `bench_config` | Directory for result JSONs and `summary.json`. |
| `TOOL_BASH_TIMEOUT_SECS` | `60` | shared tools `bash` | `BashTool` execution timeout (seconds). |
| `TOOL_EVAL_TIMEOUT_SECS` | `60` | shared tools `eval` | `EvalTool` snippet timeout (seconds). |
| `TOOL_GLOB_LIMIT` | `100` | shared tools `glob` | Maximum matches returned by `GlobTool`. |

### 2.2 Defined but Currently Unconsumed

These keys exist in `.env`/`.env.example` but no code reads them today; they are reserved for evaluator/daemon LLM configuration. Do not rely on them:

| Variable | Documented Default |
| :--- | :--- |
| `BENCH_BUILD_TIMEOUT_SECS` | `120` |
| `BENCH_TEST_TIMEOUT_SECS` | `120` |
| `BENCH_INFERENCE_TIMEOUT_SECS` | `600` |
| `EVALUATOR_BUILD_TIMEOUT_SECS` | `120` |
| `EVALUATOR_TEST_TIMEOUT_SECS` | `120` |
| `LLM_MAX_TOKENS` | `32768` |
| `LLM_TEMPERATURE` | `0.2` |
| `LLM_TIMEOUT_SECS` | `600` |

## 3. Invocation

The daemon binary (`werðan/bins/daemon`) accepts:

```bash
# Run one benchmark campaign over a migration.yml
cargo run -p daemon -- --benchmark <path/to/migration.yml>

# Alias: --config
cargo run -p daemon -- --config <path/to/migration.yml>

# Run every *.yml under a directory sequentially (default: benchmarks/assets/configs/ReCodeAgent)
cargo run -p daemon -- --benchmark-all [dir]

# Smoke probe: verify inference endpoint reachability (GET /v1/models),
# print the served model id, and exit nonzero on failure. Alias: --dry-run
cargo run -p daemon -- --smoke
```

With no arguments, the daemon falls through to `interactive::run_daemon_server()`, which launches the interactive TUI cockpit ([product.md §3](product.md)).

## 4. Daemon Modes

- **Single benchmark (`--benchmark` / `--config`)**: run one real modernization campaign over the configured legacy source, then validate the target workspace with `cargo check` and `cargo test` under configured timeouts, and write the per-project result JSON to `BENCH_RESULTS_DIR`.
- **Batch (`--benchmark-all [dir]`)**: iterate every `*.yml` under the directory sequentially, skip projects whose result JSON already exists (resume), and write the aggregate `summary.json`.
- **Smoke (`--smoke` / `--dry-run`)**: resolve the inference URL from the campaign `inference.base_url` (fall back to the compiled `BASE_URL` default; fail when neither is set), probe `GET {base_url}/models`, print the base URL and the first served model id, and exit nonzero on failure; both flags dispatch to the same `daemon::bench_smoke::run_smoke_cli` path and no campaign executes.
- **Interactive (no args)**: launch the TUI cockpit.

### Planned (not implemented)

The daemon currently logs through a single ANSI/stdout `tracing_subscriber::fmt()` sink. The following are planned but unimplemented: a structured JSONL file sink with daily rolling appenders, and a `SIGINT` handler performing clean checkpoint flush and exit-code-0 termination. Do not build on these behaviors today.

## 5. Results & Workspace Layout

- **Results directory**: `BENCH_RESULTS_DIR` (default `benchmarks/results/daemon`), overridable per environment.
- **Result labels**: each project result is `<label>.json`, where the label derives from the `workspace_root` path segments after `outputs/` (e.g. `outputs/matrix/crust-vec` → `crust-vec.json`).
- **Batch aggregate**: `summary.json` next to the per-project files.
- **Workspace reset**: each campaign resets `paths.workspace_root` to a pristine dataset copy and normalizes the stub layout before the engine starts; the reset maps the `benchmarks/outputs/` prefix of `paths.workspace_root` onto `benchmarks/assets/ReCodeAgent/data/tool_projects/`, fails for roots outside that prefix or missing dataset directories, and treats a missing workspace root as already pristine; see [tech-spec/benchmark-harness.md §2](tech-spec/benchmark-harness.md).
