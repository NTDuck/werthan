---
status: "accepted"
date: 2026-10-20
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
supersedes: ["0004-code-modernization-rig-rust-pipeline.md"]
---

# Pin METHODOLOGY.md as Authoritative System Architecture and Methodology

## Context and Problem Statement

The system documentation previously exhibited phase-count and structural divergence across references:
- `docs/references/METHODOLOGY.md` establishes a formal 7-phase Harel Statechart macro-lifecycle ($S_0 \dots S_6$), two-tier blackboard ($\mathcal{G}$ and $\mathcal{B}^{(v_i)}$), declarative scatter-gather ($D_{\max} = 1$), two-stage hybrid phase guards ($g = g_{\text{struct}} \land g_{\text{sem}}$), encapsulated ARIADNE MCTS module clusters, and a 3-tier regression calculus.
- ADR `0004-code-modernization-rig-rust-pipeline.md` and `docs/specs/product-spec.md` originally specified a 5-phase macro-FSM ($S_1 \dots S_5$).
- In-tree Rust crate implementations in `werðan/infras/statechart` historically defined a 10-state lifecycle (`ModernizationState` $S_0 \dots S_9$) alongside a 5-state `MacroState` ($S_1 \dots S_5$).

To establish unambiguous architectural discipline, which document serves as the inviolable, canonical source of truth for the modernization system's methodology, architecture, and theoretical models?

## Decision Drivers

* **Zero Specification Ambiguity**: Every component, agent boundary, state transition, and invariant must align with one definitive theoretical and practical specification.
* **Preservation of Methodological Foundations**: `docs/references/METHODOLOGY.md` provides formal definitions (Definition 1 through Definition 4), algorithmic procedures (Algorithm 1 and Algorithm 2), and verified multi-tier regression calculus necessary for repository-scale modernization.
* **Separation of Architectural Governance from Codebase Migration**: The source-of-truth document must be pinned and treated as read-only reference, while code and dependent specs are systematically brought into alignment.

## Considered Options

* **Option 1: Retain ADR 0004 5-Phase Model ($S_1 \dots S_5$) and revise METHODOLOGY.md**
  - *Shortcomings*: Violates user directive; dilutes formal pre-DAG rule extraction ($S_3$) and decoupled preflight ($S_0$) specified in METHODOLOGY.md.
* **Option 2: Pin `docs/references/METHODOLOGY.md` as the Authoritative Source of Truth**
  - Pin `docs/references/METHODOLOGY.md` as the overriding source-of-truth for all system architecture, methodology, statechart models, and agent topologies.
  - Supersede ADR 0004 on phase-count and macro-lifecycle, adopting the canonical 7-phase model ($S_0 \dots S_6$).
  - Record the Rust enum divergence (`MacroState` / `ModernizationState` in `werðan/infras/statechart`) as a known reconciliation task for subsequent implementation work.

## Decision Outcome

Chosen option: **Option 2: Pin `docs/references/METHODOLOGY.md` as the Authoritative Source of Truth**.

`docs/references/METHODOLOGY.md` is designated as the sole canonical source of truth for system methodology, formal algorithms, blackboard schemas, macro/micro statecharts, and agent evaluation protocols.

### 1. Macro-Statechart Alignment (7 Phases: $S_0 \dots S_6$)

The macro-orchestrator operates strictly according to the 7-phase Harel Statechart defined in `docs/references/METHODOLOGY.md`:
- **$S_0$: Intent & Preflight**: Single `PreflightProbe` agent; legacy build compiles, baseline test harness executes, `INTENT` schema valid.
- **$S_1$: Assess**: Declarative scatter-gather over directory shards; `ShardAssessor` and `AssessmentSynthesizer`; evaluated via $g_{\text{struct}}$ (file inventory coverage) and $g_{\text{sem}}$ (`AssessmentCritic`).
- **$S_2$: Map (DAG Construction)**: Static LSP/tree-sitter graph builder, `ImplicitCouplingScanner`, and Tarjan's SCC condensation into acyclic DAG $G = (V, E)$; evaluated via $g_{\text{struct}}$ and `TopologyCritic`.
- **$S_3$: Extract Business Rules**: Pre-DAG global extraction; `RuleMiner` extracts `Given/When/Then` cards with `legacy/file:line` pointers; `CrossModuleRuleLinker` resolves cross-module invariants; audited by `RuleAuditor`.
- **$S_4$: Brief & Reimagine Contracts**: Global architecture idioms by `MacroArchitect`; interface stubs by `ContractStubber`; compiled and audited by `ArchitectureCritic` against $\mathcal{R}_{\text{rules}}$.
- **$S_5$: Iterative DAG Transform**: Orthogonal `AND`-state over module regions $R(v_i)$ executing encapsulated ARIADNE MCTS clusters (`M0`..`M5`) scheduled via topological wave dispatch.
- **$S_6$: System Verify & Harden**: Integration test suites and SAST scanners; `IntegrationVerifier` and `SecurityAuditor`; release parity sign-off by `ReleaseAuditor`.

### 2. Micro-FSM Alignment ($M_0 \dots M_5$)

Per-module micro-regions execute the 6-state Micro-FSM:
- **`M0_Blocked`**: Awaiting upstream parents to reach `M4_Hardened`.
- **`M1_ContextHydration`**: Deterministic, non-LLM copy of rule cards $\mathcal{R}_{\text{rules}}(v_i)$ and parent contracts $\{\mathcal{C}_{\text{contracts}}(v_p, v_i)\}$ into local blackboard slice $\mathcal{B}_{PM}^{(v_i)}$.
- **`M2_ARIADNE_MCTS`**: Active MCTS search node with Softmax-UCB controller $\mathcal{C}_{\text{MCTS}}^{(v_i)}$, 4 generative subagents ($\mathcal{A}_{\text{str}}, \mathcal{A}_{\text{gen}}, \mathcal{A}_{\text{test}}, \mathcal{A}_{\text{patch}}$), and two-stage hybrid evaluator $\mathcal{A}_{\text{eval}} = \mathcal{A}_{\text{eval}}^{\text{exec}} \circ \mathcal{A}_{\text{eval}}^{\text{distill}}$.
- **`M3_ContractNegotiate`**: Emits `ev_ContractConflict` on extrinsic contract failure and returns to `M0_Blocked`.
- **`M4_Hardened`**: Commits verified target AST $c^\star$ ($R(c^\star) = 1.0$) and test suite $\mathcal{B}_{CT}^{(v_i)}$ to global blackboard $\mathcal{A}_{\text{artifacts}}(v_i)$; emits `ev_ModuleHardened`.
- **`M5_Invalidated`**: Warm-start regression state preserving existing MCTS tree $\mathcal{T}^{(v_i)}$, strategy hypotheses $\mathcal{B}_{SH}^{(v_i)}$, and test suite $\mathcal{B}_{CT}^{(v_i)}$.

### 3. Known Issues and In-Tree Code Divergence

The in-tree Rust implementation in `werðan/infras/statechart` currently contains:
- `MacroState` implementing 5 states (`S1Assess` .. `S5VerifyRelease`).
- `ModernizationState` implementing 10 states (`S0Intent` .. `S9Completed`).

Per architectural pinning rules, this ADR records this discrepancy as an identified code divergence. Reconciling `werðan/infras/statechart`, `werðan/topology`, and related engine tests to the canonical $S_0 \dots S_6$ / $M_0 \dots M_5$ statechart model is tracked as a dedicated code refactoring phase, leaving `docs/references/METHODOLOGY.md` completely unmodified as read-only ground truth.

## Consequences

* **Good**: Unambiguous authority: `docs/references/METHODOLOGY.md` supersedes all conflicting architectural descriptions across ADRs, specs, and prompts.
* **Good**: Strict alignment between product specifications, design records, and formal theoretical models.
* **Neutral**: In-tree Rust crate enums require planned migration in a subsequent code reconciliation task to mirror $S_0 \dots S_6$.
