---
status: "accepted"
date: 2026-10-19
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
supersedes: ["0001-agent-interface-architecture.md", "0002-rig-completion-model-bridges.md", "0003-master-slave-tui-chatbot.md"]
---

# Architecture (6) + (1.2): 3-Tier Harel Statechart, Dependency-DAG Blackboard, and Encapsulated ARIADNE MCTS Modernization Multi-Agent System

## Context and Problem Statement

Legacy enterprise code modernization (e.g. COBOL, PL/SQL, legacy Java/C/C++) involves large, tightly coupled codebases, non-linear dependencies, hidden runtime coupling, and non-negotiable semantic parity requirements. Existing multi-agent frameworks suffer from three systemic failure modes:
1. **Granularity Mismatch (Global State vs. Module State)**: When an agent team shares a single monolithic FSM, leaf failures force full repository rollbacks or context bloating across unrelated modules.
2. **Root Routing & Context Bottlenecks**: In top-down hierarchical agent swarms, cross-cutting interface breaks require round-trip escalation through ancestor agents, multiplying token costs and latency while degrading context windows.
3. **Static Topologies vs. Dynamic Codebase Structure**: Static hierarchies defined before inspecting the repository cannot adapt to dynamic call graphs, indirect pointers, or Strongly Connected Components (SCCs).

To solve these challenges deterministically, `werðan` establishes **Architecture (6) + (1.2)**: a 3-Tier Harel Statechart with a Dependency-DAG Blackboard and encapsulated ARIADNE MCTS search nodes, integrated with local System-1 model acceleration (**Laya**) and reverse engineering toolchains (**REA**).

The operational objective during any migration run is strictly defined: **maximize compilation status ($\mathbb{I}_{\text{compile}} = 1$) and test pass rate ($\text{PassRate}_{\text{char}} = 1.0$), while minimizing wall-clock execution time.**

## Decision Drivers

* **Separation of Macro Topology from Micro Exploration**:
  - Global phases (Discovery, Mapping, Architecture Blueprinting, Integration) are strictly non-monotonic and deterministic, requiring an explicit top-level Harel Statechart.
  - Per-module transformation is an iterative search problem guided by compiler diagnostics and characterization tests, requiring an MCTS search loop over an isolated local blackboard.
* **Hierarchical Rust Workspace Organization under `werðan/`**:
  - Eliminate flat crate structures; organize crates into strict hierarchical layers under `werðan/`: `bins/` (frontends), `topology/` (agent orchestration & assembly), and `infras/` (foundational capabilities & tools).
* **Adversarial Read/Write Isolation & Anti-Cheating**:
  - Code generators ($\mathcal{A}_{\text{gen}}$, $\mathcal{A}_{\text{patch}}$) must be strictly forbidden from mutating test assertions or reading hidden verification fixtures.
  - Test generators ($\mathcal{A}_{\text{test}}$) must generate characterization assertions against the legacy reference implementation without tailoring tests to buggy intermediate drafts.
* **Local System-1 Acceleration (`infras/laya`)**:
  - Fast, low-latency (<15 ms) classification for destructive command prevention, UCB action priors, intrinsic vs. extrinsic blocker categorization, and JEV-as-a-judge rubric evaluations.
* **Reverse Engineering Tooling (`infras/tools/rea`)**:
  - Binary/IR call-graph recovery for indirect calls and macro expansion, decompilation and CFG/DFG slicing, and symbolic/concolic branch input solving.
* **Unified Dual Frontends**:
  - Interactive Oh-My-Pi (OMP) style Ratatui TUI (`werðan/bins/interactive`) featuring a 60/40 layout, `@` file autocomplete, and conversational `/mkcfg` slash commands.
  - Headless batch CLI daemon (`werðan/bins/daemon`) with structured JSONL and ANSI tracing and graceful signal handling.

## Considered Options

* **Option 1: Static 4-Agent Pipeline (e.g. ReCodeAgent Analyzer -> Planner -> Translator -> Validator)**
  - *Shortcomings*: Fails on cyclic dependencies and non-linear regressions; when the validator fails, it must pass diagnostics to the planner or loop, creating brittle long-horizon error chains.
* **Option 2: Pure Holonic MAS (Recursive Holons with Full MOISE+ Deontic Specification)**
  - *Shortcomings*: Introducing goal trees and organizational meta-prompts into LLM prompts burns token context and degrades reasoning; BDI goal trees struggle with non-monotonic regressions and compiler loops.
* **Option 3: Architecture (6) + (1.2) - 3-Tier Harel Statechart + Dependency-DAG Blackboard + Encapsulated ARIADNE MCTS Nodes**
  - *Selected*: Combines the architectural benefits of recursive holons and deontic role barriers without organizational overhead. Statecharts govern macro lifecycles; DAG wave scheduling isolates module dependencies; MCTS explores synthesis trajectories with compiler-gated rewards.

## Decision Outcome

Chosen option: **Option 3: Architecture (6) + (1.2)**.

### 1. Hierarchical Architecture & Crate Organization

All workspace crates reside hierarchically under `werðan/` adhering to High Cohesion and Loose Coupling:

```text
werðan/
├── bins/
│   ├── interactive/                # Oh-My-Pi (omp) style Ratatui TUI cockpit (60/40 layout)
│   └── daemon/                     # Headless batch CLI runner with tracing & signal handling
│
├── topology/                       # Architectural assembly layer (consumes infras/*)
│   ├── operative/                  # Atomic agent construction (OperativeBlueprint, tool loop, stream emitter)
│   ├── cluster/                    # Multi-operative ensemble & ARIADNE MCTS search node implementation
│   └── daedalus/                   # Turnkey runtime engine binding Tier 1 Statechart, Tier 2 DAG, & Tier 3 Clusters
│
└── infras/                         # Foundational capabilities & external tool integrations
    ├── core/                       # Shared traits, Config schema (migration.yml), Event/Command buses, Directives
    ├── blackboard/                 # Tier 1 Global Blackboard (G) & Tier 3 4-Slice Local Blackboard (B) stores
    ├── statechart/                 # Deterministic Harel Statechart (S1..S5) & Module Micro-FSM (M0..M5) engines
    ├── laya/                       # Local System-1 fast model (Guardrail, UCB Action Prior, JEV-as-a-Judge)
    └── tools/
        ├── mcp/                    # Standard filesystem sandbox, LSP, compiler, and test-runner bridges
        └── rea/                    # Reverse Engineering Assistant (CFG/DFG, binary/ABI recovery, branch solving)
```

### 2. The 3-Tier State Machine & Control Flow

The architecture decomposes state and execution across three tightly scoped tiers:

#### Tier 1: Macro-Orchestrator & Global Repository Blackboard ($\mathcal{G}$)
Operates one instance per modernization campaign. Governs high-level phase gates via a deterministic Harel Statechart:
* **$S_1$: Preflight & Assess**: Scans repository structure, validates build tools, inventories legacy sources, and computes risk scores.
* **$S_2$: Dependency & Domain Map**: Employs `DependencyMapper` and `REA` to extract cross-module call graphs, collapsing Strongly Connected Components (SCCs) into a topological DAG $\mathcal{G} = (V, E)$.
* **$S_3$: Target Architecture Reimagine**: Employs `TargetArchitect` to define target language idioms, inter-module contract interfaces, schema layouts, and data transfer types.
* **$S_4$: Iterative DAG Transformation**: Hands execution over to the Tier 2 Topological Wave Scheduler, dispatching independent module nodes in parallel waves.
* **$S_5$: Final System Integration & E2E Verification**: Aggregates all hardened module artifacts, executes cross-module integration tests, verifies global invariants, and generates migration audit receipts.

#### Tier 2: Topological Wave Scheduler & Module Micro-FSMs
For each module vertex $v_i \in V$, the scheduler tracks its lifecycle through a 6-state Micro-FSM:
* **$M_0$: Blocked**: Awaits upstream dependencies: $\forall (v_j, v_i) \in E, \text{state}(v_j) = M_4$.
* **$M_1$: Rule & Contract Extraction**: Employs $\mathcal{A}_{\text{rule}}$ and `REA` to populate the local Problem Model $\mathcal{B}_{PM}^{(v_i)}$ with legacy I/O rules, boundary invariants, and parent edge contracts.
* **$M_2$: ARIADNE Search Node**: Instantiates an encapsulated `AriadneCluster` to execute an MCTS synthesis loop.
* **$M_3$: Contract Negotiation**: If an unresolvable interface conflict arises with an upstream module, pauses $M_2$ and negotiates contract adjustments.
* **$M_4$: Hardened & Published**: The module draft $c^*$ achieves full parity ($R(c^*) = 1.0$), exports verified interfaces, and unblocks downstream dependents.
* **$M_5$: Invalidated / Regressed**: When an upstream contract modification invalidates downstream assumptions, $v_i$ rolls back from $M_4 \rightarrow M_5 \rightarrow M_2$ with warm-started state.

#### Tier 3: Encapsulated ARIADNE MCTS Search Node
Each module vertex $v_i$ runs an independent MCTS search node over an isolated 4-slice local blackboard $\mathcal{B}^{(v_i)} = (\mathcal{B}_{PM}, \mathcal{B}_{SH}, \mathcal{B}_{CT}, \mathcal{B}_{PR})$:
* $\mathcal{B}_{PM}$ (**Problem Model**): Legacy AST, extracted business rules, boundary contracts, side-effect invariants.
* $\mathcal{B}_{SH}$ (**Strategy Hypotheses**): High-level translation plans, data layout choices, async patterns.
* $\mathcal{B}_{CT}$ (**Characterization Tests**): Executable test fixtures, property tests, counterexample inputs.
* $\mathcal{B}_{PR}$ (**Patched Code & Evaluation**): Candidate target implementation drafts $c_t$, compiler diagnostics, mutation scores.

The MCTS controller navigates search tree states using Softmax-UCB selection:
$$
\text{UCB}(u) = \bar{X}(u) + C \sqrt{\frac{\ln N(v)}{N(u)}}
$$
At each step, the controller delegates to one of six specialized operatives:
1. $\mathcal{A}_{\text{rule}}$: Extracts business logic, pre/post-conditions, and DDL constraints into $\mathcal{B}_{PM}$.
2. $\mathcal{A}_{\text{str}}$: Generates target modernization strategies into $\mathcal{B}_{SH}$.
3. $\mathcal{A}_{\text{gen}}$: Synthesizes initial target source code implementations $c_t$.
4. $\mathcal{A}_{\text{test}}$: Generates characterization tests into $\mathcal{B}_{CT}$ against legacy behavior.
5. $\mathcal{A}_{\text{eval}}$: Evaluates candidate drafts against the gated scalar reward $R(s_t)$.
6. $\mathcal{A}_{\text{patch}}$: Performs localized surgical fixes on $c_t$ based on compiler and test diagnostics.

### 3. Gated Scalar Objective Function

To guarantee behavioral parity and prevent reward hacking, the MCTS evaluation operative ($\mathcal{A}_{\text{eval}}$) calculates a multi-objective gated scalar reward:
$$
R(s_t) = w_1 \cdot \mathbb{I}_{\text{compile}}(c_t) + \mathbb{I}_{\text{compile}}(c_t) \cdot \Big( w_2 \cdot \text{PassRate}_{\text{char}}(c_t, \mathcal{B}_{CT}) + w_3 \cdot \text{Score}_{\text{harden}}(c_t) \Big)
$$
* $\mathbb{I}_{\text{compile}}(c_t) \in \{0, 1\}$: Deterministic binary check (0 if compilation fails, 1 if clean).
* $\text{PassRate}_{\text{char}}(c_t, \mathcal{B}_{CT}) \in [0, 1]$: Fraction of legacy characterization tests passing.
* $\text{Score}_{\text{harden}}(c_t) \in [0, 1]$: Linter cleanliness, security checks, and idiomatic rubric score evaluated by Laya-as-a-Judge.
* Weights default to: $w_1 = 0.20$, $w_2 = 0.60$, $w_3 = 0.20$ (with $w_1 + w_2 + w_3 = 1.0$).
* A draft reaches **Hardened** status when $R(s_t) = 1.0$.

### 4. Integration Guidelines: Laya & REA

#### Infras / Laya (Local System-1 Model)
* **Role 1 (Guardrail & Isolation)**: Validates operative tool invocations in <15 ms to intercept dangerous shell commands (`rm -rf`, unbound filesystem writes) and enforces adversarial read/write fences.
* **Role 2 (Decision Prior)**: Computes action priors $P(a \mid s_t)$ to guide Softmax-UCB tree exploration and classifies compiler diagnostics as intrinsic vs. extrinsic contract conflicts.
* **Role 3 (Laya-as-a-Judge)**: Scrutinizes code drafts for reward-hacking anti-patterns (e.g. hardcoded test-case branches, `todo!()`, or swallowed errors) and computes $\text{Score}_{\text{harden}}(c_t)$.

#### Infras / Tools / REA (Reverse Engineering Assistant)
* **Global Discovery ($S_1, S_2$)**: Binary and IR call-graph lifting (via Ghidra, Rizin, or LLVM IR) to discover indirect function pointers, dynamic bindings, and vtables missed by source AST parsers.
* **Contract Extraction ($M_1$)**: Decompilation and CFG/DFG slicing to extract struct memory layouts, alignment, bitfield semantics, and side effects.
* **Intra-Module Verification ($M_2$)**: Concolic execution and symbolic path solving to discover edge-case inputs covering rare legacy error branches.

### 5. Verification & Operational Metrics

Every execution run must record and optimize:
1. **Compilation Status**: Zero compiler warnings or errors across all target modules ($\mathbb{I}_{\text{compile}} = 1.0$).
2. **Test Pass Rate**: 100% equivalence test pass rate ($\text{PassRate}_{\text{char}} = 1.0$).
3. **Execution Wall Time**: Minimized search duration through warm-started state caches and parallel wave scheduling.
4. **Zero Workspace Linter Drift**: Clean passes across all 6 repository check linters (`flatmods`, `pathqual`, `plumbing`, `ste100`, `stepdown`, `turbofish`).
