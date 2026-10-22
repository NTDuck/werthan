# Technical Specification: werðan Architecture (Entry Point)

This file is the technical entry point. It pins the methodology authority, defines the workspace crate hierarchy, and preserves the six design rationale sections. Domain detail lives in the four files under [`tech-spec/`](tech-spec/).

## 1. Methodology Authority

Per [ADR-0005](../decisions/0005-methodology-source-of-truth.md), the inviolable architectural ground truth of this system is:

> [`docs/references/METHODOLOGY.md`](../references/METHODOLOGY.md)
> *(supplemented by foundational derivations in [`docs/references/METHODOLOGY-DRV.md`](../references/METHODOLOGY-DRV.md))*

All module implementations, agent definitions, statechart lifecycles, and verification protocols derive from and are governed by `METHODOLOGY.md`. On any ambiguity or conflict between these specs and `METHODOLOGY.md`, `METHODOLOGY.md` supersedes. The methodology is formally defined as the **Two-Tier Harel Statechart and Encapsulated ARIADNE Multi-Agent Architecture for Repository-Scale Code Modernization**; it carries no arbitrary commercial or brand names.

One deliberate, documented divergence exists: the in-tree ARIADNE search constants ([ADR-0012](../decisions/0012-ariadne-constant-calibration.md)), calibrated for the local 27B serving stack. See [statecharts-and-ariadne.md](tech-spec/statecharts-and-ariadne.md).

### Primary System Objectives

During every modernization campaign across a legacy codebase $\mathcal{R}_{\text{leg}}$ in source language $\mathcal{L}_{\text{src}}$ producing target repository $\mathcal{R}_{\text{tgt}}$ in target language $\mathcal{L}_{\text{tgt}}$ under execution harness $\mathcal{H}_{\text{exec}}$, the system maximizes three formal criteria:

1. **Syntactic and Type-Level Compilation**: $\mathbb{I}_{\text{compile}}(\mathcal{R}_{\text{tgt}}) = 1.0$ across all inter-module boundaries without warnings or stub placeholders.
2. **Behavioral Parity Across Domain Invariants**: $\text{PassRate}_{\text{char}}(\mathcal{R}_{\text{tgt}}) = 1.0$ across all extracted business rules, property-based tests, and characterization test suites.
3. **Architectural Idiomaticity and Static Security**: $\text{Score}_{\text{harden}}(\mathcal{R}_{\text{tgt}}) = 1.0$ across static security analyzers, vulnerability audits, and idiomatic target guidelines, while minimizing execution wall-clock time via maximal topological wave parallelism and warm-started state caches.

## 2. Architectural Design Decisions and Theoretical Justifications

Repository-scale modernization violates the assumptions of standard conversational multi-agent workflows because legacy codebases exhibit **non-monotonic discovery**: implementation faults uncovered late in module translation frequently invalidate earlier assumptions regarding interface contracts or dependency topologies. Every structural choice below resolves a specific failure mode of existing multi-agent paradigms.

**Note on numbering**: sections 2.1–2.6 are stable identifiers; `werðan/libs/global/blackboard/src/state_vector.rs:54` cites `tech-spec 2.6`. The full text of these sections lives in [tech-spec/engine-and-blackboard.md §2](tech-spec/engine-and-blackboard.md), which preserves this numbering.

### 2.1 Decoupling Deterministic Control Topology from Bounded Reasoning Context
Partition the Macro-Orchestrator $\mathcal{O} = (\mathcal{O}_{\text{engine}}, \mathcal{O}_{\text{arbiter}})$ into a deterministic statechart runtime engine and an on-demand, stateless LLM Arbiter. Full text: [engine-and-blackboard.md §2.1](tech-spec/engine-and-blackboard.md).

### 2.2 Harel Statecharts over Flat FSMs and Static Pipelines
Formalize control via a Harel Statechart $\mathcal{H}$ combining XOR macro-superstates, orthogonal AND-regions, broadcast event calculus, and deep history. Full text: [engine-and-blackboard.md §2.2](tech-spec/engine-and-blackboard.md).

### 2.3 Declarative Scatter-Gather ($D_{\max}=1$) over Autonomous Recursive Subagent Spawning
Prohibit imperative subagent spawning; execute parallelizable phases as deterministic Scatter → Parallel Map → Reduce sub-statecharts. Full text and the per-phase mechanics table: [statecharts-and-ariadne.md §2.3](tech-spec/statecharts-and-ariadne.md).

### 2.4 Two-Stage Hybrid Phase Guards ($g = g_{\text{struct}} \land g_{\text{sem}}$)
Gate every transition by a short-circuiting conjunction of a zero-token structural guard and a sharded, read-isolated adversarial semantic guard. Full text: [statecharts-and-ariadne.md §2.4](tech-spec/statecharts-and-ariadne.md).

### 2.5 Encapsulated Intra-Module MCTS Solvers (ARIADNE Sub-Holons)
Encapsulate the Softmax-UCB MCTS planner, 4-slice local blackboard, and specialist pool as an isolated intra-module solver per DAG vertex. Full text: [statecharts-and-ariadne.md §2.5](tech-spec/statecharts-and-ariadne.md).

### 2.6 Push-Based Ephemeral Invocation & Zero-Polling Invariant
All non-$\mathcal{O}$ agents are stateless ephemeral worker functions that never poll the blackboard. Full text: [engine-and-blackboard.md §2.6](tech-spec/engine-and-blackboard.md).

## 3. Workspace Crate Layout

The codebase is a strict hierarchical Cargo workspace rooted at `werðan/` ([ADR-0010](../decisions/0010-holon-modular-architecture.md)):

```text
werðan/
├── bins/
│   ├── interactive/                # OMP Ratatui TUI cockpit (60/40 layout, dynamic stream vector, CSI-2026)
│   └── daemon/                     # Headless batch CLI runner (bench_* harness, arg dispatcher)
├── libs/
│   ├── global/                     # Global orchestration & macro-state subsystem
│   │   ├── blackboard/             # Global blackboard, state vector, Tarjan SCC (global-blackboard)
│   │   ├── engine/                 # Deterministic macro-orchestrator engine, Kahn wave dispatch (global-engine)
│   │   ├── fsm/                    # State-as-struct Hoverbear typestate macro-statechart (global-fsm)
│   │   └── agents/
│   │       ├── arbiter/            # Global arbitration and semantic critics (global-arbiter)
│   │       ├── workers/            # Phase workers, TaskActivationEnvelope, ContextAssembler (global-workers)
│   │       └── global-critics/     # Rig-backed LLM critic and worker crews (global-critics)
│   ├── shared/
│   │   ├── configschema/           # Canonical migration.yml schema (CampaignConfig, from/to YAML API)
│   │   ├── tools/                  # 13 OMP agent tools (read, write, edit, glob, grep, ast_grep, ast_edit, lsp, bash, eval, debug, github, ask)
│   │   └── agents/
│   │       └── configurator/       # Configurator agent driving the /init wizard via the ask bridge
│   └── holon/                      # Multi-agent holonic modernization subsystem
│       ├── blackboard/             # Local 4-slice blackboard (holon-blackboard)
│       ├── engine/                 # Encapsulated ARIADNE cluster engine (holon-engine)
│       ├── fsm/                    # Per-module M0–M5 pure typestate machine (holon-fsm)
│       └── agents/                 # holon-strategy, holon-codegen, holon-testgen, holon-patch, holon-evaluator
├── .checks/                        # Build-time linters enforced via build.rs
│   ├── flatmods/                   # Flat module hierarchies in src/
│   ├── pathqual/                   # Fully-qualified paths
│   ├── turbofish/                  # Turbofish syntax on generic call expressions
│   ├── stepdown/                   # Caller-first function ordering
│   ├── plumbing/                   # No nested function calls
│   └── ste100/                     # ASD-STE100 technical English
└── .experiments/                   # laya-guardrails, laya-as-a-judge, laya-decision-model
```

Build-time environment ingestion: every workspace `build.rs` calls `::envload::load()` first, ingesting the nearest `.env` at compile time ([ADR-0007](../decisions/0007-compile-time-env-ingestion.md)). Operational parameters follow the no-hardcoded-config rule ([ADR-0011](../decisions/0011-env-config-parameters.md)).

Cucumber BDD suites ([ADR-0014](../decisions/0014-cucumber-bdd-verification.md)) verify behavior parity in holon-engine tests (`search.feature`, `reward.feature`, `regression.feature`) and per-linter `.checks` features (e.g. `flatmods` `legacy/modern-layout.feature`).

## 4. Domain File Map

| Topic | Location |
| :--- | :--- |
| Rationale §2.1–2.6 full text, Definition 1 global blackboard, `GlobalStateVector`, `global-blackboard` 10-module deconstruction, Definition 2 local blackboard, `global-engine`/`global-fsm`/`global-arbiter`/`global-workers`/`global-critics`, `holon-blackboard` | [tech-spec/engine-and-blackboard.md](tech-spec/engine-and-blackboard.md) |
| Definition 3 statechart 7-tuple, Definition 4 hydration, S0–S6 pipeline and guards, micro-FSM M0–M5, Kahn dispatch, MCTS controller + constants, generative subagents, evaluator, reward, regression tiers, warm-start | [tech-spec/statecharts-and-ariadne.md](tech-spec/statecharts-and-ariadne.md) |
| 13-tool registry, `DisplayExt` error model, `configschema::CampaignConfig` shape, configurator agent | [tech-spec/tools-and-configschema.md](tech-spec/tools-and-configschema.md) |
| Daemon `bench_*` modules, workspace reset, honesty gate, `BenchResult`, batch resume, LoC tiers, divergence roadmap | [tech-spec/benchmark-harness.md](tech-spec/benchmark-harness.md) |

Operator-facing configuration and CLI behavior: [product-config.md](product-config.md). User-facing behavior: [product.md](product.md).
