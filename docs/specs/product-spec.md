# Product Specification: werðan Code Modernization System

## 1. Product Overview

`werðan` is an enterprise-grade, offline-capable code modernization multi-agent system. It transforms legacy codebases (COBOL, PL/SQL, legacy Java/C/C++) into modern, verified, idiomatic target architectures (e.g. safe, idiomatic Rust).

### Architectural Ground Truth
Per **ADR 0005** (`docs/decisions/0005-methodology-source-of-truth.md`), the product design and execution lifecycle are strictly governed by:
- **`docs/references/METHODOLOGY.md`**: Authoritative formal architecture (7-phase Harel Statechart, two-tier blackboard, declarative scatter-gather, encapsulated ARIADNE MCTS clusters, and multi-tier regressions).
- **`docs/references/METHODOLOGY-DRV.md`**: Foundational derivations, design justifications, and paradigm comparisons.
- **`docs/specs/tech-spec.md`**: Technical specification, data schemas, algorithms, and crate boundaries.

In accordance with architectural governance directives, the system methodology has no arbitrary commercial or brand names; it is formally defined as the **Two-Tier Harel Statechart and Encapsulated ARIADNE Multi-Agent Architecture for Repository-Scale Code Modernization**.

### Primary Objectives & Operational Invariants
During every execution run, whether interactive or headless, the system strictly enforces three optimization goals:
1. **Maximize Compilation Status**: 100% clean compilation of target code drafts ($\mathbb{I}_{\text{compile}} = 1.0$) with zero compiler errors or stub placeholders.
2. **Maximize Test Pass Rate**: 100% equivalence test pass rate ($\text{PassRate}_{\text{char}} = 1.0$) across characterization test suites extracted from legacy behavior.
3. **Minimize Wall-Clock Time**: Maximize execution parallelism via topological wave scheduling and warm-started MCTS state caches.

---

## 2. User Interfaces & Frontends

The system provides two execution frontends residing hierarchically under `werðan/bins/`:
* `werðan/bins/interactive`: Real-time interactive cockpit powered by `ratatui` with an Oh-My-Pi (OMP) inspired workflow.
* `werðan/bins/daemon`: Headless batch CLI runner with dual-sink structured `tracing` (ANSI stdout + rolling JSONL files) and clean signal handling.

---

## 3. Interactive Cockpit Specification (`bins/interactive`)

The terminal interface divides the display into a 60% Left Pane and a 40% Right Pane:

```text
---------------------------------------------+------------------------------+
| LEFT TOP (w = 60%, h = 50%)                 | RIGHT PANE (w = 40%, h = 100%|
| [State Machine & DAG Wave Monitor]          | Dynamic Stream Vector)       |
|                                             | 1-char horizontal margin     |
| Tier 1 Macro-Statechart (7 Phases):         | +--------------------------+ |
|   [✓ S0: Intent] -> [✓ S1: Assess]          | | legacy-analyst           | |
|   -> [✓ S2: Map] -> [✓ S3: Rules]           | | Status: idle             | |
|   -> [✓ S4: Brief] -> [* S5: DAG Trans.]    | +--------------------------+ |
|   -> [  S6: Verify & Harden]                | +--------------------------+ |
|                                             | | binary-historian         | |
| Active Tier 2 Module Micro-FSMs (Wave 2):   | | Status: idle             | |
|   * v_core   [M4: Hardened] R = 1.00        | +--------------------------+ |
|   * v_ledger [M2: ARIADNE]  R = 0.72 (U4)   | +--------------------------+ |
|   * v_api    [M0: Blocked]  wait: v_ledger  | | architecture-critic      | |
|                                             | | Status: idle             | |
---------------------------------------------+ +--------------------------+ |
| LEFT BOTTOM (w = 60%, h = 50%)              | +--------------------------+ |
| [Macro-Orchestrator Window + Composer]      | | test-engineer            | |
| Orchestrator: Config loaded from ./out/cfg. | | Status: idle             | |
| Type /start @./out/migration.yml to run.    | +--------------------------+ |
| +- [ /status (btw) ] ---------------------+ | +--------------------------+ |
| | Wave 2/5: 1 Ready, 1 Active, 1 Blocked. | | | target-coder             | |
| | 42% modules hardened (R = 1.0).         | | | Status: idle             | |
| +-----------------------------------------+ | +--------------------------+ |
| > /start @out/migration.yml                 |                              |
---------------------------------------------+------------------------------+
```

#### Left Pane (w = 60%)
* **Top Half (h = 50%) — State Machine & DAG Wave Monitor**:
    $$S_0 \rightarrow S_1 \rightarrow S_2 \rightarrow S_3 \rightarrow S_4 \rightarrow S_5 \rightarrow S_6$$
* **Bottom Half (h = 50%) — Macro-Orchestrator Window with Merged Prompt Composer**:

#### Right Pane (w = 40%) — Dynamic Stream Vector of Specialist Operatives
* **Dynamic Stream Ingestion**: The right pane accepts an unconstrained vector of streams/DTOs (`Vec<SpecialistCardState>`) rather than hardcoded 5 items. The vertical space is dynamically divided into $N$ equal vertical slots:
  $$\text{Slot Height} = \left\lfloor \frac{\text{Total Height}}{N} \right\rfloor$$
* **Window Margins**: Each specialist window in the right panel applies a 1-character horizontal margin (`Margin { horizontal: 1, vertical: 0 }`), preventing text truncation against terminal edges and separating stacked windows cleanly.
* **Naming Convention**: Agent titles and stream identifiers are strictly lowercased, hyphenated (e.g. `legacy-analyst`, `binary-historian`, `architecture-critic`, `test-engineer`, `target-coder`), and omit numbering.

#### Terminal Output Synchronization
* Wraps all frame redraws with CSI-2026 synchronized output sequences (`\x1b[?2026h` and `\x1b[?2026l`) to eliminate visual tearing across modern terminal emulators.


### 3.2 Oh-My-Pi (OMP) User Experience & Keystroke Interceptor

1. **`@` Mentions Autocomplete Dropdown**:
2. **`/` Slash Command Menu Dropdown**:
3. **Esc Key & Exit Semantics**:


### 3.3 Slash Command Workflow & Human Decision Gates

| Command | Behavior |
| :--- | :--- |
| `/mkcfg` | Launches the configurator agent wizard (`configurator::ConfiguratorAgent`), which drives a typed question script through the `ask` tool and produces a `configschema::CampaignConfig` draft saved via `/save-config`. |
| `/init` | Same configurator wizard semantics as `/mkcfg`: no arguments run the full question script, and `key=value` arguments take the fast path through `apply_answer`. |
| `/start @{path}` | Reads and parses the referenced YAML configuration file, transitions into $S_0$, and starts streaming. |
| `/pause` | Pauses execution loop cleanly at rollout or topological wave boundary. |
| `/unpause` | Resumes paused execution loop. |
| `/stop` | Cancels execution token, cleanly halts background tasks, and flushes checkpoints. |
| `/status` (or `/btw`) | Spawns non-blocking snapshot task rendering point-in-time progress inside the orchestrator window. |
| `/approve` | Resolves pending Human-in-the-Loop decision gate with approval notes. |
| `/reject` | Resolves pending Human-in-the-Loop decision gate with rejection reason. |
| `/clear` | Clears the scrollable chat transcript. |
| `/quit` (or `/exit`) | Gracefully stops execution, flushes checkpoints, restores terminal state, and terminates. |

#### Human-in-the-Loop (HITL) Decision Gates
When `execution_mode: "hitl"` is configured in `migration.yml`, the engine pauses at critical architectural junctures and prompts the operator through the interactive window:
1. **Gate on Reimagine (`gate_on_reimagine: true`)**: Fires at the transition boundary from Phase $S_4$ to $S_5$.
2. **Gate on Macro-Regression (`gate_on_macro_regression: true`)**: Fires when an encapsulated ARIADNE cluster discovers an undocumented dependency cycle.
3. **Extrinsic Conflict Escalation (`extrinsic_conflict_escalation: "ask_human"`)**: Fires when an upstream parent interface conflict cannot be resolved automatically.

The interactive workflow runner implements these phases with the state-as-struct Hoverbear pattern: each run phase is one struct, transitions consume the machine and yield the next phase at compile time, and Hitl mode parks the machine on the gate until `/approve` or `/reject` resolves it.

## 4. Headless Daemon CLI Specification (`bins/daemon`)

### 4.1 Invocation & Arguments
```bash
werðan-daemon --config <path/to/migration.yml>
```

### 4.2 Behavior & Tracing
* Operates non-interactively, linking the exact same `topology` engine and statechart runtime as `bins/interactive`.
* Dual-sink logging infrastructure:
  - ANSI colored formatted logs directed to `stdout`.
  - Structured JSONL log events directed to `{output_dir}/logs/daemon.jsonl` using daily rolling file appenders.
* Signal Handling: Listens for `SIGINT` (Ctrl+C); on capture, cleanly issues `EngineCommand::Stop`, flushes all Blackboard checkpoints, and exits with code 0.

### 4.3 Benchmark Modes
* `--benchmark <migration.yml>` (alias `--config`): run one real modernization campaign over the configured legacy source, then validate the target workspace with `cargo check` and `cargo test` under configured timeouts, and write a per-project result JSON to `benchmarks/results/daemon/` (overridable via `BENCH_RESULTS_DIR`).
* `--benchmark-all [dir]`: iterate every `*.yml` under `dir` (default `benchmarks/configs/generated`, overridable via `BENCH_CONFIGS_DIR`) sequentially, skip projects whose result JSON already exists (resume), and write the aggregate `summary.json`.
* `--smoke`: verify the inference endpoint is reachable via `GET /v1/models` and print the served model id. No mock output exists anywhere in the benchmark paths.

### 4.4 Environment Variable Configuration
The daemon, agent harnesses, tools, and LLM clients load configuration strictly from the environment with zero in-code fallback literals. Default values are centrally configured in `.env` and documented in `.env.example`. The daemon's `build.rs` ingests `.env` at compile time and emits `cargo:rustc-env=KEY=VALUE` (with `cargo:rerun-if-changed=.env`), exposing variables via `env!("KEY")` macros with zero runtime dotenv overhead:
* `BASE_URL`: Endpoint base URL for OpenAI-compatible chat completions (`http://127.0.0.1:8092/v1`).
* `MODEL_ID`: Default model identifier (`swift-1.5-qwen3.8-27b`).
* `INFERENCE_API_KEY`: Bearer credential for OpenAI-compatible endpoints (`""`; local servers ignore it).
* `BENCH_CONFIGS_DIR`: Default directory searched by `--benchmark-all` (`benchmarks/configs/generated`).
* `BENCH_RESULTS_DIR`: Directory where benchmark JSON results and `summary.json` are written (`benchmarks/results/daemon`).
* `BENCH_BUILD_TIMEOUT_SECS`: Timeout for cargo check in seconds (`120`).
* `BENCH_TEST_TIMEOUT_SECS`: Timeout for cargo test in seconds (`120`).
* `BENCH_INFERENCE_TIMEOUT_SECS`: Timeout for LLM inference requests in seconds (`600`).
* `EVALUATOR_BUILD_TIMEOUT_SECS`: Standalone CargoHarness build timeout in seconds (`120`).
* `EVALUATOR_TEST_TIMEOUT_SECS`: Standalone CargoHarness test timeout in seconds (`120`).
* `TOOL_BASH_TIMEOUT_SECS`: Timeout for `BashTool` execution in seconds (`60`).
* `TOOL_EVAL_TIMEOUT_SECS`: Timeout for `EvalTool` snippet execution in seconds (`60`).
* `TOOL_GLOB_LIMIT`: Maximum matches returned by `GlobTool` (`100`).
* `LLM_MAX_TOKENS`: Upper bound on completion tokens for daemon inference requests (`32768`).
* `LLM_TEMPERATURE`: Sampling temperature for daemon inference requests (`0.2`).
* `LLM_TIMEOUT_SECS`: Request timeout for daemon inference requests (`600`).

The campaign harness guarantees honest execution end to end:
* Before the engine starts, the target workspace stub layout is normalized: sibling stub module files are inlined into `src/lib.rs` and removed from disk, so the `unimplemented!()` count and every compiled artifact live in the single file the evaluation harness replaces.
* DAG vertices are implementation source files only. Header and test-tree files never become modules, and extracted rule cards carry the exact Rust API surface (captured from the pristine stubs) plus sibling header declarations so code generation targets the real required interface.
* The inference transport is Rig's stock OpenAI-compatible provider. A failed exchange never degrades to reasoning prose; the original error propagates instead.
* A run whose engine completes but issued zero inference requests is recorded as dishonest and fails the campaign.

---

## 5. Migration Configuration Schema (`migration.yml`)

The configuration file generated by `/mkcfg` and accepted by `/start` and `werðan-daemon` adheres to the canonical schema. The canonical code equivalent is `configschema::CampaignConfig` (crate `werðan/libs/shared/configschema`) with API `from_yaml_text`, `from_yaml_path`, and `to_yaml_text`. Every frontend deserializes into this type:

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
  model: "ashalliant/qwen3.8-27b-abliterated"
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
  engine_budget_per_module: 24                 # Maximum rollouts (B_max) in state M2
  ucb_exploration_c: 1.414                    # Exploration constant for Softmax-UCB
  softmax_temperature: 0.35                  # Softmax temperature for prior sampling
  reward_weights:
    compile: 0.20                             # w_comp weight for clean compilation
    characterization_parity: 0.60             # w_char weight for equivalence test pass rate
    hardening_score: 0.20                     # w_hard weight for semantic rule adherence
  llm_backend:                                # Generative agent inference backend (daemon OpenAiCompatModel completion source)
    base_url: "http://127.0.0.1:8092/v1"      # Local ninfer OpenAI-compatible server
    model: "swift-1.5-qwen3.8-27b"
    max_tokens: 32768
    temperature: 0.2
    timeout_secs: 600
  cargo_harness:                              # Stage 3a real-workspace harness timeouts
    build_timeout_secs: 120
    test_timeout_secs: 120
```

---

## 6. Standardized Shared Agent Tools Suite (`werðan/libs/shared/tools`)

The system provides 13 standardized, omp-conforming agent tools exposed as `::rig::tool::Tool` implementations under `werðan/libs/shared/tools/`:

| Tool | Rig Tool ID | Purpose & Behavioral Contract | Engine & Pruning Rules |
| :--- | :--- | :--- | :--- |
| **Read** | `read` | Reads whole or sliced source files with exact line counts. | Standard asynchronous filesystem I/O. |
| **Write** | `write` | Creates files or overwrites existing files; automatically creates missing parent directories. | `tokio::fs::create_dir_all` + atomic payload write. |
| **Edit** | `edit` | Applies surgical hashline patches (`[PATH#TAG]`, `PUT N.=M:`, `PUT <N:`, `PUT >N:`). | Hashline patch parser with order preservation and splice application. |
| **Glob** | `glob` | Discovers filesystem paths matching glob patterns, honoring `.gitignore` and `hidden` flags. | `globset::GlobBuilder` + `ignore::WalkBuilder`. Automatically prunes `.git`, `node_modules`, and `target` unless explicitly scoped. |
| **Grep** | `grep` | Searches regular expression patterns across files with line numbers and pagination `skip`. | `ignore::WalkBuilder` + `regex::RegexBuilder`. Automatically prunes `.git`, `node_modules`, and `target`. Caps matches at 500 entries per call. |
| **AST Grep** | `ast_grep` | Searches structural code syntax patterns across files with AST metavariables (`$VAR`, `$$$VAR`). | `ast-grep-core` pattern matching over tree-sitter grammars (rust, ts, tsx, js, py, json, toml, md) + `ignore::WalkBuilder` file discovery. |
| **AST Edit** | `ast_edit` | Performs structural AST syntax rewrites across files using metavariable template substitution. | `ast-grep-core` pattern matching with native `replace_by` replacement and per-file language inference from `ast_lang` (rs, ts, tsx, js, py, json, toml, md) + byte-range splice rewrites + `ignore::WalkBuilder` file collection. |
| **LSP** | `lsp` | Interacts with Language Server Protocol actions: `status`, `diagnostics`, `definition`, `references`, `hover`, `symbols`, `rename`. | Language server dispatch with workspace file analysis powered by `ignore::WalkBuilder`. |
| **Bash** | `bash` | Executes shell commands in child processes with configurable timeouts, capturing exit codes, stdout, and stderr. | `tokio::process::Command` under POSIX shell runner. |
| **Eval** | `eval` | Dynamically executes Python (`python3`) or JavaScript (`node`) code snippets in isolated subprocesses. | Cross-runtime process execution harness. |
| **Debug** | `debug` | Interacts with Debug Adapter Protocol (DAP) sessions and adapter status. | Structured JSON DAP state inspector. |
| **GitHub** | `github` | Queries GitHub repository issues, pull requests, releases, and repository metadata via GitHub CLI (`gh`). | Headless `gh` CLI runner with JSON serialization. |
| **Ask** | `ask` | Surfaces typed questions (id, text, options, recommended) to the human operator and collects answers. | Process-global canned-answer sink for tests and frontend bridges; errors honestly when no operator answer exists. |

### Tool Error Handling Model
All tools standardize error propagation through the `DisplayExt` blanket trait (`impl<Target: ::core::fmt::Display + ?::core::marker::Sized> DisplayExt for Target`), converting any displayable error into `::rig::tool::ToolError::ToolCallError`.

---

## 7. Global Orchestration Subsystem (`werðan/libs/global/*`)

Repository-level state and the top-level macro-statechart execution are organized into dedicated global crates under `werðan/libs/global/`:

* **`werðan/libs/global/blackboard` (`global-blackboard`)**: Global repository blackboard $\mathcal{G}$ and state vector $\Sigma(\mathcal{G})$ per METHODOLOGY.md Definition 1. Deconstructed into six cohesive modules (`inventory.rs`, `dag.rs`, `rules.rs`, `contracts.rs`, `artifacts.rs`, `state_vector.rs`), plus Tarjan strongly connected component condensation (`scc.rs`), cross-platform path resolution (`path_resolver.rs`), and re-exporting macro-statechart typestate from `global-fsm`. Manages the file inventory $\mathcal{I}_{\text{inv}}$ (`RepositoryInventory`), condensed dependency DAG $G = (V, E)$ (`CondensedDag`), business rule cards $\mathcal{R}_{\text{rules}}$ (`RuleRegistry`), interface contracts $\mathcal{C}_{\text{contracts}}$ (`ContractRegistry`), hardened module artifacts $\mathcal{A}_{\text{artifacts}}$ (`ArtifactRegistry`), and global state vector $\Sigma(\mathcal{G})$ (`GlobalStateVector`). Every component and the aggregate `GlobalBlackboard` support the `bon` builder pattern (`::builder()`) with optional `file_path`, `output_dir`, `hash`, and `create_if_missing`, resolving default storage to platform data dirs (`AppData/werðan/{hash}` on Windows, `~/.werðan/{hash}` on Linux/Unix).
* **`werðan/libs/global/fsm` (`global-fsm`)**: State-as-struct Hoverbear typestate macro-statechart (`MacroStatechart`) advancing across seven canonical macro states ($S_0 \dots S_6$, `Terminated`) with deep history recording, regression target navigation, and bon builder.
* **`werðan/libs/global/agents/arbiter` (`global-arbiter`)**: Global arbitration and semantic critics implementing Algorithm 1 stage-two validation. Exports `SemanticCritic`, `Arbiter`, `CritiqueVerdict`, `CritiqueRecord`, `DeterministicCritic`, and `DeterministicArbiter` with bon builders.
* **`werðan/libs/global/agents/workers` (`global-workers`)**: Stateless macro-phase worker crew per Algorithm 1 substate S_m.2. Exports `TaskActivationEnvelope` (mode, slice summary, delta directive, bound tools), the deterministic `ContextAssembler` hydrating envelopes from the blackboard, `phase_role_name` mapping macro phases to worker roles, the `PhaseWorker` contract (`execute` returning typed `PhaseWorkOutput`), `DeterministicWorkers` covering S0 through S6 discovery and verification roles with zero tokens, and the typed output family (`Preflight`, `Assessment`, `Coupling`, `Rules`, `Contracts`, `Verification`, `Skipped`) the engine commits to the board.
* **`werðan/libs/global/agents/global-critics` (`global-critics`)**: Rig-backed LLM crews for the arbiter contracts and the worker contract. `lib.rs` hosts `RigCriticCrew` (one Rig agent per phase verdict) and `RigArbiter` (ACCEPT/REJECT adjudication); `workers.rs` hosts `RigWorkerCrew` implementing `global-workers::PhaseWorker` over one worker agent per phase, parsing model responses into typed `PhaseWorkOutput` variants with malformed responses falling back to the deterministic no-op so structural guards observe the real board gap.
* **`werðan/libs/global/engine` (`global-engine`)**: Deterministic macro-statechart engine implementing Algorithm 1. Modularized into `batches.rs`, `error.rs`, `guards.rs`, `solver.rs`, and `lib.rs` with bon builder (`MacroEngine::builder()`). Operates over `statechart: MacroStatechart` and `board: GlobalBlackboard` where state configuration $\Sigma(\mathcal{G})$ is held inside `board.state`. Coordinates phase loop progression, stage-one structural guards (`guard_s0`..`guard_s6`), stage-two semantic critic batch audits, Kahn topological wave dispatch in $S_5$, Tier B contract conflict handling, and Tier C cyclic dependency macro-regression ($S_5 \to S_2$) with deep history $\Theta_{H^*}$ preservation and restore. Discovery shards dispatch through the injected `PhaseWorker` crew (`dispatch_phase_worker` + `commit_phase_output`), the CRITIC_RETRY re-dispatch carries the guard detail as `delta_directive`, and `worker_executions` counts phase-shard executions.
* **`werðan/libs/holon/blackboard` (`holon-blackboard`)**: Per-module 4-slice local blackboard $\mathcal{B}^{(v_i)}$ per METHODOLOGY.md Definition 2, one instance per DAG vertex inside each ARIADNE MCTS sub-holon. Deconstructed into four cohesive slice modules: `problem_model.rs` ($\mathcal{B}_{PM}$: `BusinessRuleCard`, `EdgeContract`, `ProblemModelSlice` with `render_prompt_context`), `strategy_hypotheses.rs` ($\mathcal{B}_{SH}$: `StrategyHypothesis`, `StrategyHypothesesSlice`), `characterization_tests.rs` ($\mathcal{B}_{CT}$: `CharacterizationTestsSlice`), and `patch_records.rs` ($\mathcal{B}_{PR}$: `FaultCause`, `FaultRecord`, `PatchRecordsSlice`). The crate root `lib.rs` re-exports every slice type at `::holon_blackboard::*` and composes the four slices as pub fields of `LocalBlackboard` with `is_hydrated` and `reset_transient_records`.
* **`werðan/libs/holon/fsm` (`holon-fsm`)**: Per-module six-micro-state machine ($M_0$..$M_5$) for each ARIADNE cluster region $R(v_i)$, following the Hoverbear pattern as a pure compile-time typestate machine: `StateMachine<S>` over state markers `M0Blocked`, `M1ContextHydration`, `M2AriadneMcts`, `M3ContractNegotiate`, `M4Hardened`, `M5Invalidated`. Each state marker carries its own data (e.g. `M3ContractNegotiate.blocker_delta`, `M4Hardened.target_code`/`test_suite`/`reward`, `M5Invalidated.delta_spec`), and per-state inherent impls expose consuming transitions (`wake_ready`, `hydration_completed`, `into_hardened`, `into_contract_negotiate`, `conflict_dispatched`, `into_invalidated`, `warm_start_ready`, `emit_cycle_regression`) plus reflection predicates (`state_tag`, `is_blocked`, `is_hardened`, `is_active`, `history`, `drain_events`, `activation_mode`). Disallowed transitions are unrepresentable at compile time (no dynamic `transition` API); a warm-start regression path returns `M2AriadneMcts` with `ActivationMode::WarmStartRegression`. The former dynamic `ClusterFsm` wrapper, its `MicroStateBehavior` trait, `ClusterFsmContext`, `MicroEvent`, `TransitionEffect`, and `FsmError` types were removed: consumers drive the typed machine directly and the ARIADNE controller maps typed outcomes per Algorithm 2.
* **`werðan/libs/holon/engine` (`holon-engine`)**: Encapsulated ARIADNE cluster MCTS search, rollout controller, and macro-engine solver adapter (`ClusterEngine`, `AriadneSolver`) with bon builders. `ClusterEngine::solve` consumes the module's `StateMachine<M2AriadneMcts>` directly (compile-time typestate entry, no dynamic FSM), advancing it through `M3ContractNegotiate`/`M4Hardened` and returning `EngineTypestateOutcome` variants that carry the post-transition machine (`Hardened(M4)`, `ContractConflict(M3)`, `CycleFound{machine, cycle_edges, proof_trace}`, `BudgetExhausted{machine, best_candidate, best_reward, blocker_trace}`).

