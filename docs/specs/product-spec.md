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

### 3.1 Visual Layout Geometry (60/40 Split)
The terminal interface divides the display into a 60% Left Pane and a 40% Right Pane:

```text
+---------------------------------------------+------------------------------+
| LEFT TOP (w = 60%, h = 50%)                 | RIGHT PANE (w = 40%, h = 100%|
| [State Machine & DAG Wave Monitor]          | Dynamic Phase-Conditioned)   |
|                                             |                              |
| Tier 1 Macro-Statechart (7 Phases):         | [Phase S5: DAG Transform]    |
|   [✓ S0: Intent] -> [✓ S1: Assess]          | Module: v_ledger (Wave 2)    |
|   -> [✓ S2: Map] -> [✓ S3: Rules]           | Active Cluster: ARIADNE      |
|   -> [✓ S4: Brief] -> [* S5: DAG Trans.]    | Sub-windows (w = 40% / 5):   |
|   -> [  S6: Verify & Harden]                | +----+----+----+----+----+   |
|                                             | |Str |Gen |Test|Eval|Pat |   |
| Active Tier 2 Module Micro-FSMs (Wave 2):   | |H2: |fn  |prop|R=  |diff|   |
|   * v_core   [M4: Hardened] R = 1.00        | |Asyn|tr()|test|0.72|h4  |   |
|   * v_ledger [M2: ARIADNE]  R = 0.72 (U4)   | +----+----+----+----+----+   |
|   * v_api    [M0: Blocked]  wait: v_ledger  | > LSP: checking signature... |
|                                             | > Rollout 4/24 | Temp: 0.35  |
+---------------------------------------------+------------------------------+
| LEFT BOTTOM (w = 60%, h = 50%)              | [Parallel Cluster Slot 2]    |
| [Macro-Orchestrator Interactive Window]     | Module: v_parser (Wave 2)    |
|                                             | Active Cluster: ARIADNE      |
| Orchestrator: Config loaded from ./out/cfg. | +----+----+----+----+----+   |
| Type `/start @./out/migration.yml` to run.  | |Str |Gen |Test|Eval|Pat |   |
| +- [ /status (btw) ] ---------------------+ | |H1: |fn  |I/O |R=  |--  |   |
| | Wave 2/5: 1 Ready, 1 Active, 1 Blocked. | | |Port|p() |snap|1.00|--  |   |
| | 42% modules hardened (R = 1.0).         | +----+----+----+----+----+   |
| +-----------------------------------------+ | > Module Hardened! R = 1.00  |
| > /start @out/migration.yml                 | > Emitting ev_ModuleHardened |
+---------------------------------------------+------------------------------+
```

#### Left Pane (w = 60%)
* **Top Half (h = 50%) — State Machine & DAG Wave Monitor**:
  - Displays Tier 1 Macro-Statechart progress across all 7 canonical phases:
    $$S_0 \rightarrow S_1 \rightarrow S_2 \rightarrow S_3 \rightarrow S_4 \rightarrow S_5 \rightarrow S_6$$
  - Renders Two-Stage Hybrid Guard status at phase boundaries:
    - Stage 1 $g_{\text{struct}}$: Zero-token programmatic check (instant green tick or retry traceback).
    - Stage 2 $g_{\text{sem}}$: Sharded adversarial LLM critic (`ShardCritic`) audit progress and verdicts (`PASS` / `REJECT`).
  - Displays real-time status of active Tier 2 Module Micro-FSMs ($M_0 \dots M_5$) across topological execution waves:
    - Lists active module vertices in $\text{ReadySet}(\mathcal{G})$.
    - Highlights per-module scalar rewards ($R$), visit counts ($N$), active MCTS strategy hypotheses, and dependency wait-states.
* **Bottom Half (h = 50%) — Macro-Orchestrator Interactive Window**:
  - Scrollable chat transcript showing orchestrator directives, Human-in-the-Loop (HITL) gate prompts, operator messages, system notices, and error logs.
  - Dismissible inline `/status` (or `/btw`) callout blocks showing point-in-time progress without interrupting active streaming.
  - Interactive command input buffer with `@` file autocomplete and `/` slash command popup.

#### Right Pane (w = 40%) — Dynamic Phase-Conditioned Operatives
The Right Pane allocates vertical slots dynamically conditioned on the active Macro-State $S_{\text{macro}}$ and configured `PaneRole` channels:

| Active Macro-Phase | Right Pane Display & Dynamic Allocation |
| :--- | :--- |
| **$S_0$ Intent & Preflight** | Single operative window for `PreflightProbe`: streams legacy build commands, environment inspections, and baseline test suite executions. |
| **$S_1$ Assess** | Sharded into $K$ slots for active `ShardAssessor` workers analyzing dialect quirks and tech debt, plus a bottom slot for `AssessmentSynthesizer`. |
| **$S_2$ Map (DAG)** | Upper slot for static AST/LSP graph building and Tarjan SCC condensation; lower slot for `ImplicitCouplingScanner` and `TopologyCritic`. |
| **$S_3$ Extract Rules** | Upper slot streaming parallel `RuleMiner` shards extracting `Given/When/Then` cards; lower slot streaming `CrossModuleRuleLinker` and `RuleAuditor`. |
| **$S_4$ Brief & Contracts** | Upper slot for `MacroArchitect` global idiom definitions; middle slots for parallel `ContractStubber` shards; lower slot for `ArchitectureCritic`. |
| **$S_5$ Iterative DAG** | Divided equally among active parallel **ARIADNE Clusters** (up to concurrency cap $K_{\max}$). Each cluster slot is horizontally subdivided into 5 sub-windows for its 5 specialized operatives:<br>1. $\mathcal{A}_{\text{str}}$: Modernization strategy hypotheses ($h_k$).<br>2. $\mathcal{A}_{\text{gen}}$: Candidate target code synthesis ($c'$).<br>3. $\mathcal{A}_{\text{test}}$: Adversarial characterization test generation ($\mathcal{B}_{CT}$, strictly read-isolated from candidate code).<br>4. $\mathcal{A}_{\text{eval}}$: Two-Stage Hybrid Evaluator (compiler/test runner + diagnostic distiller).<br>5. $\mathcal{A}_{\text{patch}}$: Surgical code repair diffs ($c''$). |
| **$S_6$ System Verify** | Upper slot for `IntegrationVerifier` E2E test executions; lower slot for `SecurityAuditor` SAST scans and `ReleaseAuditor` parity signoff. |

#### Terminal Output Synchronization
* Wraps all frame redraws with CSI-2026 synchronized output sequences (`\x1b[?2026h` and `\x1b[?2026l`) to eliminate visual tearing and flickering across modern terminal emulators.

---

### 3.2 Oh-My-Pi (OMP) User Experience & Keystroke Interceptor

1. **`@` File Autocomplete Dropdown**:
   - Keystroke interceptor detects `@` token prefix within the input buffer.
   - Pops up a floating list widget above the cursor backed by fast directory walking and `nucleo` fuzzy matching.
   - Pressing `Enter` or `Tab` selects the highlighted file and inserts `@path/to/file` into the prompt.
2. **`/` Slash Command Menu**:
   - Detects leading `/` at column 0 of the input line.
   - Pops up an autocomplete overlay listing available commands and descriptions.

---

### 3.3 Slash Command Workflow & Human Decision Gates

| Command | Behavior |
| :--- | :--- |
| `/mkcfg` | Launches conversational configuration wizard (`OrchestratorMode::ConfigWizard`). Asks structured questions one at a time covering input directory, output directory, target language, toolchains, verification strictness, MCTS budget, and HITL gate preferences. Automatically validates directory paths, computes legacy source file previews, and serializes `{output-dir}/migration.yml`. |
| `/start @{path}` | Reads and parses the referenced YAML configuration file, validates paths and prerequisites, transitions the macro-orchestrator engine into $S_0$ (Intent & Preflight) followed sequentially by $S_1 \dots S_6$, and starts streaming live telemetry to the TUI. |
| `/pause` | Sets a global watch flag that pauses the execution loop cleanly at the next MCTS rollout or topological wave boundary. In-flight tool executions settle before parking. |
| `/unpause` | Resumes the paused execution loop. |
| `/stop` | Cancels the active engine cancellation token, cleanly aborts running background workers, and persists Blackboard checkpoints to disk. |
| `/status` (or `/btw`) | Spawns an ephemeral, non-blocking read task that snapshots $\mathcal{G}$'s state vector $\Sigma(\mathcal{G}) = \langle S_{\text{macro}}, \mathbf{M}, \Theta_{H^*}, \mathcal{Q}_{\text{events}} \rangle$ and active Micro-FSM metrics without acquiring execution locks or interrupting streaming agents. Renders the response in a dismissible inline callout inside the orchestrator window. |
| `/clear` | Clears the scrollable chat transcript. |
| `/quit` (or `/q`) | Gracefully stops any active migration, flushes checkpoints, restores terminal state, and exits. |

#### Human-in-the-Loop (HITL) Decision Gates
When `execution_mode: "hitl"` is configured in `migration.yml`, the engine pauses at critical architectural junctures and prompts the operator through the interactive window:
1. **Gate on Reimagine (`gate_on_reimagine: true`)**:
   - Fires at the transition boundary from Phase $S_4$ (`Brief & Reimagine Contracts`) to Phase $S_5$ (`Iterative DAG Transform`).
   - Presents a summary of extracted domain rules ($\mathcal{R}_{\text{rules}}$) and compiled target interface stubs ($\mathcal{C}_{\text{contracts}}$).
   - Prompts the user to approve contracts or inject manual amendments before topological code generation begins.
2. **Gate on Macro-Regression (`gate_on_macro_regression: true`)**:
   - Fires when an encapsulated ARIADNE cluster discovers an undocumented dependency cycle (Tier C fault: $\kappa = \texttt{EXTRINSIC\_CYCLE}$).
   - Presents the cycle trace ($E_{\text{cycle}}$) and requests confirmation before $\mathcal{O}_{\text{engine}}$ checkpoints Deep History ($\Theta_{H^*}$) and rolls back $S_5 \to S_2$ for Tarjan SCC condensation.
3. **Extrinsic Conflict Escalation (`extrinsic_conflict_escalation: "ask_human"`)**:
   - Fires when an upstream parent interface conflict cannot be resolved automatically via $\mathcal{O}_{\text{arbiter}}$.
   - Presents the conflicting signatures and requests an operator decision on which module boundary contract to adapt.

---

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

---

## 5. Migration Configuration Schema (`migration.yml`)

The configuration file generated by `/mkcfg` and accepted by `/start` and `werðan-daemon` adheres to the canonical schema:

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
  ninfer_base_url: "http://localhost:8000/v1"
  deep_model: "ashalliant/qwen3.8-27b-abliterated"
  fast_model: "ashalliant/qwen3.8-27b-abliterated"
  max_parallel_shards: 2
  user_input_mode: "wait"                     # "wait" | "immediate"
  max_shards: 2
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
  mcts_budget_per_module: 24                 # Maximum rollouts (B_max) in state M2
  ucb_exploration_c: 1.414                    # Exploration constant for Softmax-UCB
  softmax_temperature: 0.35                  # Softmax temperature for prior sampling
  reward_weights:
    compile: 0.20                             # w_comp weight for clean compilation
    characterization_parity: 0.60             # w_char weight for equivalence test pass rate
    hardening_score: 0.20                     # w_hard weight for semantic rule adherence
```
