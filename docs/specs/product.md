# werðan Product Specification

## 1. Product Overview

`werðan` is an offline-capable code modernization multi-agent system. It transforms legacy codebases (COBOL, PL/SQL, legacy Java/C/C++) into modern, verified, idiomatic target architectures (e.g. safe, idiomatic Rust), enforcing 100% clean compilation of target code drafts with zero compiler errors or stub placeholders, 100% equivalence test pass rate across characterization suites extracted from legacy behavior, and maximal topological wave parallelism to minimize wall-clock time.

Per [ADR-0005](../decisions/0005-methodology-source-of-truth.md), the product design and execution lifecycle are governed by [`../references/METHODOLOGY.md`](../references/METHODOLOGY.md) (authoritative formal architecture: 7-phase Harel Statechart, two-tier blackboard, declarative scatter-gather, encapsulated ARIADNE MCTS clusters, multi-tier regressions), supplemented by [`../references/METHODOLOGY-DRV.md`](../references/METHODOLOGY-DRV.md) (foundational derivations and design justifications). The methodology is formally defined as the **Two-Tier Harel Statechart and Encapsulated ARIADNE Multi-Agent Architecture for Repository-Scale Code Modernization**.

## 2. User Interfaces & Frontends

Two execution frontends reside under `werðan/bins/`:

- **`werðan/bins/interactive`**: real-time interactive cockpit powered by `ratatui` with an Oh-My-Pi (OMP) inspired workflow.
- **`werðan/bins/daemon`**: headless batch CLI runner with ANSI/stdout structured `tracing` logging. CLI and daemon modes: [product-config.md](product-config.md); harness internals: [tech-spec/benchmark-harness.md](tech-spec/benchmark-harness.md).

## 3. Interactive Cockpit Specification (`bins/interactive`)

### 3.1 60/40 Layout (`TuiLayout6040`)

The terminal interface divides the display into a 60% left pane and a 40% right pane:

```text
---------------------------------------------+------------------------------+
| LEFT TOP (w = 60%, h = 50%)                 | RIGHT PANE (w = 40%, h = 100%|
| [State Machine & DAG Wave Monitor]          | Dynamic Stream Vector)       |
|                                             | 1-char horizontal margin     |
| Tier 1 Macro-Statechart (7 Phases):         | +--------------------------+ |
|   [✓ S0: Intent] -> [✓ S1: Assess]          | | legacy-analyst           | |
|   -> [✓ S2: Map] -> [✓ S3: Rules]           | +--------------------------+ |
|   -> [✓ S4: Brief] -> [* S5: DAG Trans.]    | | binary-historian         | |
|   -> [  S6: Verify & Harden]                | +--------------------------+ |
|                                             | | architecture-critic      | |
| Active Tier 2 Module Micro-FSMs (Wave 2):   | +--------------------------+ |
|   * v_core   [M4: Hardened] R = 1.00        | | test-engineer            | |
|   * v_ledger [M2: ARIADNE]  R = 0.72 (U4)   | +--------------------------+ |
|   * v_api    [M0: Blocked]  wait: v_ledger  | | target-coder             | |
|                                             +------------------------------+
| LEFT BOTTOM (w = 60%, h = 50%)              |
| [Macro-Orchestrator Window + Composer]      |
| Type /start @./out/migration.yml to run.    |
| > /start @out/migration.yml                 |
---------------------------------------------+------------------------------+
```

- **Left pane (w = 60%)**:
  - **Top half (h = 50%) — State Machine & DAG Wave Monitor**: renders the macro-statechart progression $S_0 \rightarrow S_1 \rightarrow S_2 \rightarrow S_3 \rightarrow S_4 \rightarrow S_5 \rightarrow S_6$ and the active Tier 2 module micro-FSMs with per-module rewards and blockers.
  - **Bottom half (h = 50%) — Macro-Orchestrator Window with merged prompt composer**: orchestrator transcript and the slash-command input line.
- **Right pane (w = 40%) — Dynamic Stream Vector of Specialist Operatives**: accepts an unconstrained vector of streams (`Vec<SpecialistCardState>`) rather than a fixed set; the vertical space divides into $N$ equal slots, $\text{Slot Height} = \lfloor \text{Total Height} / N \rfloor$. Each specialist window applies a 1-character horizontal margin (`Margin { horizontal: 1, vertical: 0 }`), preventing truncation against terminal edges and separating stacked windows.
- **Naming Convention**: agent titles and stream identifiers are strictly lowercased, hyphenated (e.g. `legacy-analyst`, `binary-historian`, `architecture-critic`, `test-engineer`, `target-coder`), and omit numbering.

### 3.2 Terminal Output Synchronization

All frame redraws wrap with CSI-2026 synchronized output sequences (`\x1b[?2026h` and `\x1b[?2026l`) to eliminate visual tearing across modern terminal emulators.

### 3.3 OMP User Experience Dropdowns

1. **`@` Mentions Autocomplete Dropdown**: typing `@` opens a file/mention completion dropdown for referencing paths and entities.
2. **`/` Slash Command Menu Dropdown**: typing `/` opens the registered slash-command menu with descriptions (see §3.4).
3. **Esc Key & Exit Semantics**: Esc dismisses open dropdowns and cancels in-progress input; exit goes through `/quit`/`/exit`.

### 3.4 Slash Commands

Eleven tokens are registered (`werðan/bins/interactive/src/lib.rs`):

| Command | Behavior |
| :--- | :--- |
| `/help` | Display the command manual. |
| `/init` | Run the configuration wizard (no args) or set fields (`key=value` args) via the `configurator::ConfiguratorAgent` through the ask bridge. |
| `/start @{path}` | Read and parse the referenced YAML configuration file, transition into $S_0$, and start streaming. |
| `/approve [notes]` | Resolve a pending Human-in-the-Loop decision gate with approval notes. |
| `/reject <reason>` | Resolve a pending Human-in-the-Loop decision gate with rejection reason. |
| `/save-config` | Save the current `configschema::CampaignConfig` draft as YAML. |
| `/load-config <path>` | Load an existing YAML config. |
| `/status` | Spawn a non-blocking snapshot task rendering point-in-time progress inside the orchestrator window. |
| `/clear` | Clear the scrollable chat transcript. |
| `/quit` | Gracefully stop execution, flush checkpoints, restore terminal state, and terminate. |
| `/exit` | Alias of `/quit`. |

### 3.5 Human-in-the-Loop (HITL) Decision Gates

When `execution_mode: "hitl"` is configured in `migration.yml`, the engine pauses at critical architectural junctures and prompts the operator through the interactive window. Gates come from the `configschema` `hitl` section:

1. **`gate_on_reimagine`**: fires at the transition boundary from $S_4$ to $S_5$.
2. **`gate_on_macro_regression`**: fires at the $S_5 \rightarrow S_2$ macro-regression boundary.
3. **`extrinsic_conflict_escalation: "ask_human"`**: fires when an upstream parent interface conflict cannot be resolved automatically (`"auto_negotiate"` keeps it in-engine).

Gates resolve via `/approve [notes]` or `/reject <reason>`. The interactive workflow runner implements the phases with the state-as-struct Hoverbear pattern: each run phase is one struct, transitions consume the machine and yield the next phase at compile time, and HITL mode parks the machine on the gate until resolved.

In headless benchmark runs there is no operator: a campaign that parks on a gate returns `RunStop::HitlGatePending` from the engine, and the daemon records it in the result `error` field as `campaign parked on human gate <kind>` instead of reporting success. The triggering event stays queued, so an operator-supervised resume (interactive mode) reprocesses it after `/approve`.

## 4. Benchmark Artifacts (User View)

Benchmark runs write per-project result JSON files and a batch `summary.json` to the results directory (`BENCH_RESULTS_DIR`, default `benchmarks/results/daemon`). Artifacts:

- **Per-project result**: `<label>.json` where the label derives from the `workspace_root` path segments after `outputs/` (e.g. `outputs/matrix/crust-vec` → `crust-vec`). Fields: `campaign_id`, `wall_time_ms`, `engine_wall_time_ms`, `llm_requests`, `unimplemented_before`, `unimplemented_after_reset`, `compilation_success`, `test_pass_rate`, `unimplemented_remaining`, `solver_dispatches`, `worker_executions`, `structural_retries`, `macro_states_visited`, `total_lines`, `code_lines`, `cocomo_effort_pm`, `cocomo_duration_months`, `error`.
- **Batch summary**: `summary.json` with `{total, compilation_rate, mean_test_pass_rate, mean_wall_time_ms, total_wall_time_ms, per_category}`.
- **Benchmark logs**: under `benchmarks/logs`.
- **Run scoping**: the `{yyyymmdd}` run directory component is fixed at run start, so all artifacts of one run share it.

## 5. Success Criteria

A completed campaign must satisfy all of:

1. **Compilation 100%**: target code drafts compile cleanly ($\mathbb{I}_{\text{compile}} = 1.0$) with zero compiler errors or stub placeholders.
2. **Test pass rate 100%**: equivalence characterization suites pass at $\text{PassRate}_{\text{char}} = 1.0$.
3. **Zero stubs**: `unimplemented_remaining == 0` in the final target workspace.
4. **`llm_requests > 0`**: the honesty gate records zero-inference completions as dishonest failures.

Operator configuration, CLI invocation, and environment variables: [product-config.md](product-config.md).
