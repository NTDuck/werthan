# Product Specification: werðan Code Modernization System

## 1. Product Overview

`werðan` is an enterprise-grade, offline-capable code modernization multi-agent system. It transforms legacy codebases (COBOL, PL/SQL, legacy Java/C/C++) into modern, verified, idiomatic target architectures.

### Primary Objectives & Operational Invariants
During every execution run, whether interactive or headless, the system strictly enforces three optimization goals:
1. **Maximize Compilation Status**: 100% clean compilation of target code drafts ($\mathbb{I}_{\text{compile}} = 1.0$) with zero compiler errors.
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
| [State Machine & DAG Wave Monitor]          | divided by N static entries) |
|                                             |                              |
| Tier 1 Macro-FSM:                           | [1/4] RepoAssessor (h = 25%) |
|   [✓ S1: Assess] -> [✓ S2: Map]             | > Scanning build.gradle...   |
|   -> [✓ S3: Reimagine] -> [* S4: DAG Trans.]|------------------------------|
|                                             | [2/4] DependencyMapper (25%) |
| Active Tier 2 Module Micro-FSMs (Wave 2):   | > Collapsed 3 SCCs in G...   |
|   * v_core   [M4: Hardened] R = 1.00        |------------------------------|
|   * v_ledger [M2: ARIADNE]  R = 0.72 (U4)   | [3/4] TargetArchitect (25%)  |
|   * v_api    [M0: Blocked]  wait: v_ledger  | > Seeded 14 edge contracts   |
+---------------------------------------------+------------------------------+
| LEFT BOTTOM (w = 60%, h = 50%)              | [4/4] Cluster: ARIADNE (25%) |
| [Macro-Orchestrator Interactive Window]     | (Each sub-pane w = 40% / 6)  |
|                                             | +----+----+----+----+----+--+|
| Orchestrator: Config written to ./out/cfg.  | |Rule|Str |Gen |Test|Eval|Pa||
| Type `/start @./out/migration.yml` to run.  | |    |    |    |    |    |tc||
| +- [ /status (btw) ] ---------------------+ | |Extr|H2: |fn  |prop|R = |h ||
| | Wave 2/5 (42% modules M4_HARDENED).     | | |inv.|Asyn|tr()|test|0.72|..||
| +-----------------------------------------+ | +----+----+----+----+----+--+|
| > /start @out/migration.yml                 |                              |
+---------------------------------------------+------------------------------+
```

#### Left Pane (w = 60%)
* **Top Half (h = 50%) — State Machine & DAG Wave Monitor**:
  - Displays Tier 1 Macro-FSM progress ($S_1 \rightarrow S_2 \rightarrow S_3 \rightarrow S_4 \rightarrow S_5$).
  - Displays real-time status of active Tier 2 Module Micro-FSMs ($M_0..M_5$) across topological execution waves.
  - Highlights per-module current scalar reward ($R$), UCB rollouts, and dependency wait-states.
* **Bottom Half (h = 50%) — Macro-Orchestrator Interactive Window**:
  - Scrollable chat transcript showing orchestrator prompts, human gate prompts, user messages, notices, and errors.
  - Dismissible inline `/status` callout blocks.
  - Command input prompt buffer at the bottom.

#### Right Pane (w = 40%)
* Statically allocated into $N$ equal vertical slots ($h = 100\% / N$) for configured agent operatives and clusters.
* **Single Operative Slot**: Displays role title, telemetry metrics, and continuous streaming logs.
* **Multi-Operative Cluster Slot (e.g. ARIADNE)**:
  - Horizontally subdivided into $K$ equal sub-windows ($w = 40\% / K$, e.g. 6 sub-windows for $A_{\text{rule}}$, $A_{\text{str}}$, $A_{\text{gen}}$, $A_{\text{test}}$, $A_{\text{eval}}$, $A_{\text{patch}}$).
  - Streams real-time operative sub-tokens simultaneously without blocking parent redraws.

#### Terminal Output Synchronization
* Wraps all frame updates with CSI-2026 sequences (`\x1b[?2026h` and `\x1b[?2026l`) to eliminate visual tearing and flickering on modern terminals.

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

### 3.3 Slash Command Workflow

| Command | Behavior |
| :--- | :--- |
| `/mkcfg` | Launches conversational configuration wizard (`OrchestratorMode::ConfigWizard`). Asks structured questions one at a time (ask pattern) covering input directory, output directory, target language, toolchains, verification strictness, MCTS budget, and HITL gate preferences. Automatically validates directory paths, computes legacy source file previews, and serializes `{output-dir}/migration.yml`. |
| `/start @{path}` | Reads and parses the referenced YAML configuration file, validates paths and prerequisites, transitions the Daedalus engine into $S_1$, and starts streaming live telemetry to the TUI. |
| `/pause` | Sets a global watch flag that pauses the execution loop cleanly at the next MCTS rollout or wave boundary. In-flight requests settle before parking. |
| `/unpause` | Resumes the paused execution loop. |
| `/stop` | Cancels the active engine cancellation token, aborts running background workers, and persists Blackboard checkpoints to disk. |
| `/status` (or `/btw`) | Spawns an ephemeral, non-blocking read task that snapshots `GlobalBlackboard` and active Micro-FSM metrics without acquiring execution locks or interrupting streaming agents. Renders the response in a dismissible inline callout inside the orchestrator window. |
| `/clear` | Clears the scrollable chat transcript. |
| `/quit` (or `/q`) | Gracefully stops any active migration, flushes checkpoints, restores terminal state, and exits. |

---

## 4. Headless Daemon CLI Specification (`bins/daemon`)

### 4.1 Invocation & Arguments
```bash
werthan-daemon --config <path/to/migration.yml>
```

### 4.2 Behavior & Tracing
* Operates non-interactively, linking the exact same `topology` engine as `bins/interactive`.
* Dual-sink logging infrastructure:
  - ANSI colored formatted logs directed to `stdout`.
  - Structured JSONL log events directed to `{output_dir}/logs/daemon.jsonl` using daily rolling file appenders.
* Signal Handling: Listens for `SIGINT` (Ctrl+C); on capture, cleanly issues `EngineCommand::Stop`, flushes all Blackboard checkpoints, and exits with code 0.

---

## 5. Migration Configuration Schema (`migration.yml`)

The configuration file generated by `/mkcfg` and accepted by `/start` and `werthan-daemon` adheres to the following specification:

```yaml
version: "1.0"

project:
  input_dir: "./legacy-source"
  output_dir: "./modernized-target"
  source_language: "cpp14"
  output_language: "rust"
  target_edition: "2024"

hitl:
  gate_on_reimagine: true                  # Human signoff required after S3 before S4
  gate_on_macro_regression: true           # Human signoff required before rolling S4 -> S2/S3
  extrinsic_conflict_escalation: "auto_negotiate" # "auto_negotiate" | "ask_human"

ariadne:
  mcts_budget_per_module: 24              # Maximum rollouts (B_max) in state M2
  ucb_exploration_c: 1.414                 # Exploration constant for Softmax-UCB
  softmax_temperature: 0.35               # Softmax temperature for prior sampling
  reward_weights:
    compile: 0.20                          # w1 weight for clean compilation
    characterization_parity: 0.60          # w2 weight for equivalence test pass rate
    hardening_score: 0.20                  # w3 weight for idiomatic/linter quality

infras:
  enable_rea: true                         # Enable Reverse Engineering Assistant tools
  laya_guardrail_mode: "strict"            # "strict" | "audit_only"
```
