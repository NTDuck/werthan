---
Title: Branch • actualizer - Google Gemini
Platform: Gemini
Date: 2026-10-21 04:16:43 UTC
Model: Gemini
Turns: 7
Source: https://gemini.google.com/app/3156ae39624b6288
---

> **[SYSTEM DIRECTIVE FOR CONSUMING LLM]**
> This document contains a structured transcript of an interaction between a human user and an AI model.
> You must parse this document using the following strict block delimiters:
> - `::USER_MESSAGE::` indicates the start of the human's input.
> - `::MODEL_REASONING::` indicates the start of the AI's internal thought process when available.
> - `::MODEL_RESPONSE::` indicates the start of the AI's final output.
> - `::END_TURN::` indicates the conclusion of a single interaction pair.
> Treat these boundaries strictly to maintain context.


::USER_MESSAGE::
##### You said I would like to actualize Claude Modernization Plugin (transform) into a MAS that transforms code2co…

 I would like to actualize Claude Modernization Plugin (transform) into a MAS that transforms code2code.  Some architecture I looked into was: (1) Static graph + static routing (ReCodeAgent), (2) Static hierarchy (tree) + root node (orchestrator) owns a state machine (for controlling phase transitions/regressions). Evaluate whether (2) is optimal - I believe there are many other architectures I haven't touched.

::MODEL_RESPONSE::
Architecture **(2) — a static agent hierarchy paired with a root orchestrator owning a phase-controlling Finite State Machine (FSM)** — is a substantial improvement over **(1) ReCodeAgent’s static pipeline**, yet it remains **sub-optimal for large-scale, repository-level code2code transformation**.

While (2) solves the biggest weakness of (1) by enabling explicit phase regressions, centralizing the state machine inside the root node of a static agent tree creates severe bottlenecks when translating tightly coupled modules.

## 1. Deconstructing the Design Space: Topology vs. Control Flow

Evaluating multi-agent systems (MAS) for code modernization requires separating three orthogonal architectural dimensions that are frequently conflated:

- Agent Topology (Structural): The communication and delegation graph governing who can talk to whom and how context is isolated (e.g., Pipeline, Tree Hierarchy, Peer-to-Peer Mesh, or Shared Blackboard).
- Control Flow / State Machine (Temporal): The mechanism governing phase transitions, preconditions, and regressions (e.g., Assess → Extract Rules → Map → Transform → Harden in the Claude Code Modernization workflow).
- Artifact & Memory Coupling (Data): Where the intermediate representations (Abstract Syntax Trees, Program Dependence Graphs, characterization tests, and extracted business rules) live and how mutations propagate.

## 2. Why Architecture (2) Outperforms (1) — And Where It Breaks Down

### Where (2) Wins Over Static Pipelines (ReCodeAgent)

In **ReCodeAgent** (`Analyzer` → `Planner` → `Translator` → `Validator`), routing is essentially a static feed-forward pipeline with local tool-calling loops (via LSP/MCP) inside the `Translator` and `Validator` phases. That works well when the `Planner` produces a dependency order that holds true through execution.

However, legacy modernization (`transform`) is inherently **non-monotonic**:

- During Transform or Harden, an agent often discovers hidden runtime coupling, undocumented reflection, or a broken characterization test that invalidates the domain boundaries established back in Map or Extract Rules.
- By giving the root orchestrator an explicit State Machine, Architecture (2) can evaluate guard conditions at phase boundaries and trigger deterministic regressions (e.g., transitioning Harden → Extract Rules or Transform → Map) rather than trapping a leaf agent in an infinite local repair loop.

### The Three Failure Modes of Architecture (2)

1. Granularity Mismatch (Global State vs. Module State): In a real repository, the entire codebase does not move through Extract Rules → Transform → Harden in lockstep. Leaf modules in the dependency graph may already be in Hardened state while a mid-tier service hits a failure and needs to regress to Extract Rules. If the root orchestrator owns a single monolithic FSM, you either roll back the entire repository phase or bloat the root orchestrator into tracking NNN entangled state machines.
2. The Root Routing & Context Bottleneck: When Worker Agent BBB (translating OrderService) discovers that an already-migrated interface in Worker Agent AAA (UserRepository) needs an asynchronous signature change, a strict tree topology forces BBB to report failure up to its supervisor/root, which must diagnose the cross-cutting conflict, transition state, and dispatch instructions back down to AAA. Every hop up and down the tree risks lossy context summarization.
3. Static Topology vs. Dynamic Codebase Structure: A static hierarchy defines agent teams before inspecting the repository’s actual Call Graph and Strongly Connected Components (SCCs). A monolith with circular dependencies requires a fundamentally different coordination pattern than a cleanly layered microservice repo.

## 3. Alternative MAS Architectures for Code2Code Transformation

To overcome the bottlenecks of (1) and (2), four distinct architectural patterns from distributed systems and MAS research apply directly to code modernization:

### Architecture (3): Shared Blackboard + Event-Driven Controller

Originated in classical AI (Hearsay-II) and adapted for LLM multi-agent systems (such as LbMAS), the **Blackboard pattern** replaces hierarchical message-passing with a structured, shared workspace.

- Structure: The "Blackboard" is a persistent, queryable Code Property Graph (CPG) / Dependency Graph where every module node stores its legacy source, extracted business-rule cards, characterization test suite, target code, and verification status.
- Control Flow: Specialist agents (RuleExtractor, TestPinner, CodeTranslator, SecurityHardener) do not form a tree. Instead, they act as independent Knowledge Sources watching the Blackboard. An event-driven controller triggers an agent whenever a node’s state preconditions are met.
- Why it fits transform: If CodeTranslator modifies a shared data model and breaks a downstream contract, it posts a ContractInvalidated marker directly onto the affected nodes on the Blackboard. Only the impacted subgraph automatically regresses its node state and re-triggers specialist agents—zero root-orchestrator bottleneck.

### Architecture (4): Dependency-DAG Actor Model (Distributed Micro-FSMs)

Instead of one central state machine at the root, state ownership is **distributed across the codebase's topological dependency graph**.

- Structure: Each Translation Unit (file, class, or module SCC) is encapsulated as an autonomous Actor (or localized agent sub-graph) connected by edges mirroring the repository's import/call graph.
- Control Flow: Every Actor owns its own local Finite State Machine (Unmapped → RulesExtracted → TestsPinned → Transformed → Verified). Transitions are governed by local verification tools (compiler, LSP, test runner) and asynchronous message-passing between parent and child Actors.   arXiv
- Why it fits transform: Enables maximum parallelism across independent branches of the dependency DAG while keeping regressions surgical. If Module BBB cannot satisfy a type constraint exported by Module AAA, Actor BBB sends a NegotiateInterfacePatch message directly to Actor AAA. Actor AAA regresses locally, updates its export, re-runs its characterization tests, and notifies its dependents.

### Architecture (5): Dialectical / Adversarial Triad (Generator–Verifier–Auditor)

Within the `Transform` and `Harden` stages, a single agent or simple sequential handoff often suffers from **semantic drift**—for example, weakening a unit test so that buggy translated code passes.

- Structure: A three-node adversarial loop isolated by strict information barriers:Characterization & Rule Auditor: Has read access only to the legacy code and extracted business rules; generates black-box I/O snapshots, property-based tests, and invariant assertions.Transformation Agent: Has write access only to modernized/** and cannot modify test assertions.   Skills DirectorySymbolic / Execution Referee: Runs deterministic tools (LSP diagnostics, compiler, differential test harness, mutation testing) and feeds structured counterexamples back to the Transformation Agent.
- Why it fits transform: Directly operationalizes the adversarial verification principle in the Claude Code Modernization workflow, guaranteeing behavioral parity before a module can transition to Hardened.

### Architecture (6): Hybrid Two-Tier Harel Statechart + DAG Blackboard (Recommended)

No single pure pattern is optimal across *all* phases of the Claude Modernization workflow because **repository discovery is global, whereas code transformation is local and graph-structured**. The optimal architecture combines them into two tiers using a **Hierarchical State Machine (Harel Statechart)**:

1. Tier 1 — Macro-Orchestrator (Hierarchical FSM) for Global Phases: A root orchestrator owns the high-level state machine for repository-wide phases: Preflight → Assess → Map → Reimagine. It delegates to read-only exploration agents to build the global Dependency DAG and Domain Map on a shared Blackboard.
2. Tier 2 — Per-Module Micro-FSMs over a Shared Blackboard for Execution: Once Map locks in the dependency graph, Tier 1 enters a composite state (Iterative Transformation). Inside this state, execution shifts to Blackboard + DAG-scheduled Adversarial Triads. Each module progresses through its own local state machine (Extract Rules → Pin Tests → Transform → Harden) in topological order, with localized regressions propagating along dependency edges on the Blackboard and only escalating to the Tier 1 Macro-Orchestrator if a fundamental architectural assumption from Reimagine fails.

## 4. Architectural Trade-Off Comparison

| Architecture Pattern | Topology (Structure) | Control & State Ownership | Cross-Module Regression Handling | Context Efficiency | Parallelism Across Repo | Best Fit in Modernization |
| --- | --- | --- | --- | --- | --- | --- |
| (1) Static Pipeline (e.g., ReCodeAgent) | Linear DAG of 4 specialist agents | Static handoffs; local tool loops only | Poor — cannot regress to replan if translation invalidates plan | High (passes static docs forward) | Low (sequential file order) | Small-to-medium repos (~2K LoC) with clean dependencies |
| (2) Static Hierarchy + Root FSM | Fixed Supervisor–Worker Tree | Single centralized FSM at root node | Moderate — supports regressions, but coarse-grained & root-bottlenecked | Moderate (suffers from vertical summary drift) | Moderate (worker fan-out per phase) | Medium repos with strict phase gates |
| (3) Shared Blackboard (e.g., LbMAS) | Star around shared CPG / Knowledge Base | Event-driven controller over Blackboard state | High — invalidated symbols trigger only affected specialists | Very High (agents query exact AST/rule slices) | High (asynchronous specialist triggers) | Legacy monoliths with tangled, emergent business rules |
| (4) Dependency-DAG Actor Model | Dynamic graph isomorphic to repo Call/Import DAG | Distributed Micro-FSMs (1 per module/SCC) | Very High — localized P2P contract negotiation along DAG edges | High (strict local interface scope) | Very High (all independent DAG branches) | Large multi-module repos with deep dependency trees |
| (5) Adversarial Triad | 3-node cyclic triad with read/write isolation | Referee-gated state transition | N/A (operates intra-module) | High (focused on single translation unit) | N/A (unit-level pattern) | High-assurance Transform & Harden verification |
| (6) Hybrid Statechart + DAG Blackboard | Tier 1 Tree + Tier 2 Blackboard & Adversarial Triads | Hierarchical Statechart (Macro-FSM + Micro-FSMs) | Optimal — local DAG repair first; escalates to Macro-FSM only on architectural deadlocks | Very High | Very High | Enterprise-scale code-modernization (transform plugin) |

## 5. Key Literature & Reference Implementations to Anchor Your Design

To formalize this architecture with strong software engineering and MAS grounding, the following foundational and contemporary works directly address the separation of topology, statecharts, and repository-level translation:

- Repository-Level Multi-Agent Code Translation: Ibrahimzada et al. (2026), "ReCodeAgent: A Multi-agent Workflow for Language-Agnostic Translation and Validation of Large-Scale Repositories" — Demonstrates the baseline 4-agent pipeline (Analyzer, Planner, Translator, Validator) augmented with LSP/MCP static analysis tools, and highlights why dependency-aware ordering and trajectory pruning matter.   arXiv
- Hierarchical State Machines (Separating Macro vs. Micro Regressions): David Harel (1987), "Statecharts: A Visual Formalism for Complex Systems" (Science of Computer Programming) — The canonical mathematical formalism for orthogonality (concurrent per-module states) and depth (composite macro-states containing micro-FSMs). Using a Harel Statechart instead of a flat FSM directly solves the granularity problem of Architecture (2).
- Blackboard Systems in LLM MAS: Salemi et al. / LbMAS (2025–2026), "LLM-based Multi-Agent Blackboard System" and classical Erman et al. (1980), "The Hearsay-II Speech-Understanding System" — Formalizes how decoupled specialist agents self-select or trigger reactively off a shared multi-level representation (syntax → lexical → semantic → architectural) when workflow paths cannot be statically predetermined.
- Graph-Structured Agent Workflows & Actor Systems: LangGraph (Cyclic State Graphs with Checkpointers) and Wu et al. (2023/2024), "AutoGen: Enabling Next-Gen LLM Applications via Multi-Agent Conversation" — Practical execution substrates for implementing Tier 1 as a cyclic state graph with subgraph spawning (Send / map-reduce over DAG nodes) and Tier 2 as isolated actor triads.

## Interactive Architecture Explorer

Use the interactive diagram below to compare how the four primary architectures route control, manage state, and handle a cross-module regression when `Module B` discovers an interface mismatch with an already-translated `Module A` during transformation:
Generating interactive visual...Structuring code, this may take a minute

::END_TURN::


::USER_MESSAGE::
I have read ARIADNE, which I am thinking of integrating either via (1.1) extending its agent pool from the default of 5 to better suit my project, or (1.2) treat the whole ARIADNE as a node in the graph/hierarchy (although to whice node is still not decided). Your architecture (6) seems simple enough, and (1.2) could be applied. Expand on that architecture. Provide its components - topology, state machines, etc.

::MODEL_RESPONSE::
Treating **ARIADNE as an encapsulated node inside Architecture (6) — option (1.2) paired with a minimal boundary adaptation from (1.1)** — is the strongest architectural design for repository-scale `code2code` transformation.

In **ARIADNE** (*Agentic Reward-Informed Adaptive Decision Exploration*), the core mechanism is a **Monte Carlo Tree Search (MCTS)** planner using softmax-UCB selection over states $s_t = (c_t, \mathcal{B}_t)$, backed by a 4-slice Blackboard $\mathcal{B} = (\mathcal{B}_{PM}, \mathcal{B}_{SH}, \mathcal{B}_{CT}, \mathcal{B}_{PR})$ and a 5-agent action space: Strategy Analysis ($\mathcal{A}_{\text{str}}$), Code Generation ($\mathcal{A}_{\text{gen}}$), Test Generation ($\mathcal{A}_{\text{test}}$), Scoring ($\mathcal{A}_{\text{eval}}$), and Code Repair ($\mathcal{A}_{\text{patch}}$).

## 1. Architectural Verdict: Why (1.2) Beats Pure (1.1), and Where the Node Lives

### Why Pure (1.1) Breaks at Repository Scale

If you simply expand ARIADNE’s flat 5-agent pool to 8–10 agents (adding repository scanners, dependency mappers, and architecture planners) and let its MCTS loop orchestrate the entire repository:

1. Combinatorial Branching Explosion: MCTS is designed to explore competing algorithmic/implementation trajectories for a bounded program ccc where each rollout can be rapidly compiled, tested, and scored with a scalar reward RRR. Across MMM modules and KKK agents, the action space ∣A(st)∣\vert{}\mathcal{A}(s_t)\vert{}∣A(st​)∣ scales as O(M×K)O(M \times K)O(M×K), causing UCB exploration to fragment across unrelated files.
2. Non-Searchable Global Phases: Repository discovery (Preflight, Assess, Map) is deterministic graph construction, not a stochastic search problem. Running MCTS over repository inventory wastes rollouts on steps that require deterministic state transitions.

### How (1.2) + Surgical (1.1) Fits into Architecture (6)

- Apply (1.2) for Topology: Embed the entire ARIADNE MCTS + 5-Agent + Local Blackboard engine as the Intra-Module Transformation & Verification Node inside Tier 2 of Architecture (6). Every node (file, module, or Strongly Connected Component) in the repository’s Dependency DAG instantiates a scoped ARIADNE solver to transform and harden that specific translation unit.
- Apply Surgical (1.1) at the Node Boundary: Stock ARIADNE initializes its Problem Model Blackboard BPM\mathcal{B}_{PM}BPM​ via BuildProblemModel(x) from a natural-language prompt. For code2code modernization, you extend ARIADNE’s boundary with one pre-MCTS specialist (Arule\mathcal{A}_{\text{rule}}Arule​: Legacy Rule & Contract Extractor) to populate BPM\mathcal{B}_{PM}BPM​ from legacy ASTs/execution traces, and add an Extrinsic Blocker Detector inside Aeval\mathcal{A}_{\text{eval}}Aeval​ so MCTS halts early if a failure is caused by an upstream module’s interface rather than a local bug.

## 2. System Topology: The 3-Layer Component Blueprint

The expanded Architecture (6) separates **Global Repository Coordination (Tier 1)**, **Topological Dependency Scheduling (Tier 2)**, and **Intra-Module MCTS Search (Tier 3 / ARIADNE Node)**.

### Layer A: Tier 1 — Macro-Orchestrator & Global Repository Blackboard (G\mathcal{G}G)

Tier 1 operates once per repository phase and owns the **Global Repository Blackboard (G\mathcal{G}G)**, which is strictly separated from ARIADNE’s local blackboards to prevent context pollution.

- Macro-Orchestrator (Deterministic Harel Statechart): Controls global phase gates (Preflight → Assess → Map → Reimagine → Iterative DAG Transformation → System Integration). It contains no LLM reasoning loop of its own; it evaluates guard predicates over G\mathcal{G}G and dispatches Tier 1 agents or the Tier 2 DAG Scheduler.
- Tier 1 Specialist Agents (Read/Write to G\mathcal{G}G):RepoAssessor: Builds the inventory, language/framework matrix, build/test harness adapters, and risk scores.DependencyMapper: Constructs the Repository Dependency DAG G=(V,E)G = (V, E)G=(V,E) via static analysis (LSP/Tree-sitter/CodeQL), collapsing circular dependencies (SCCs) into single composite vertices vi∈Vv_i \in Vvi​∈V.TargetArchitect: Executes the Reimagine phase, defining target language idioms, module boundaries, and initial Interface Contracts on edges eij∈Ee_{ij} \in Eeij​∈E.
- Global Blackboard (G\mathcal{G}G) Schema:DAG Topology: Vertices vi∈Vv_i \in Vvi​∈V (translation units) and directed dependency edges eij∈Ee_{ij} \in Eeij​∈E.Global Type & Contract Registry: Exported signatures, data schemas, and semantic invariants for every migrated module.Cross-Module Issue Queue: Contract violation tickets raised when a downstream module cannot satisfy an upstream interface.

### Layer B: Tier 2 — Topological Wave Scheduler & Module Actors

When Tier 1 enters the composite state `Iterative DAG Transformation`, control passes to the **DAG Wave Scheduler**:

- It identifies all vertices vi∈Vv_i \in Vvi​∈V whose upstream dependencies {vj∣(vj,vi)∈E}\{v_j \mid (v_j, v_i) \in E\}{vj​∣(vj​,vi​)∈E} are in the Hardened state.
- For each ready vertex viv_ivi​, it spawns an isolated Module Actor running a Micro-FSM.

### Layer C: Tier 3 — The Encapsulated ARIADNE Node (Inside Each Module Actor)

When a Module Actor $v_i$ enters its transformation state, it invokes **ARIADNE** as a self-contained black-box solver scoped strictly to module $v_i$.

Each ARIADNE instance owns a **Local 4-Slice Blackboard** $\mathcal{B}^{(v_i)} = (\mathcal{B}_{PM}, \mathcal{B}_{SH}, \mathcal{B}_{CT}, \mathcal{B}_{PR})$ and adapted agent pool:

| Agent / Component | Scope in ARIADNE | Adaptation for Code2Code (transform) | Blackboard I/O |
| --- | --- | --- | --- |
| Arule​: Rule & Contract Initializer (1.1 Boundary Addition) | Pre-MCTS Root Setup (BuildProblemModel) | Extracts business rules, side-effect invariants, I/O types, and upstream contracts from G for legacy module vi​. | Reads: Legacy AST + G contractsWrites: BPM​ (Problem Model) |
| MCTS Global Planner | Tree Search Controller | Selects tree nodes u=(c,B) via softmax over UCB scores; balances exploring distinct refactoring strategies vs. patching a working draft. | Reads: Scalar rewards R, visit counts N(u) |
| Astr​: Strategy Analysis Agent | Action family Astr​(st​) | Proposes distinct modernization hypotheses for vi​ (e.g., H1: Direct structural port, H2: State-machine refactor, H3: Async stream pipeline) with utility priors. | Reads: BPM​Writes: BSH​ (Strategy Hypotheses) |
| Agen​: Code Generation Agent | Action family Agen​(st​) | Synthesizes target-language draft c′ conditioned on the active strategy hypothesis in BSH​ and upstream interfaces. | Reads: BPM​,BSH​Writes: Draft c′, metadata to BSH​ |
| Atest​: Test Generation Agent | Action family Atest​(st​) | Generates characterization snapshot tests, property-based tests, and edge-case counterexamples against legacy behavior (read-isolated from c′). | Reads: BPM​,BCT​Writes: BCT​ (Counterexamples & Tests) |
| Aeval​: Scoring & Parity Agent | Action family Aeval​(st​) | Runs staged verification (Compiler/LSP → Differential Characterization Tests → Security/Idiomatic Linter). Computes scalar reward R and classifies errors as Intrinsic vs. Extrinsic (Upstream Contract). | Reads: Draft c′, BCT​,BPM​Writes: Reward R, diagnostics to BCT​,BPR​ |
| Apatch​: Code Repair Agent | Action family Apatch​(st​) | Applies surgical AST/diff repairs to c′ using localized fault traces and counterexamples from BPR​. | Reads: Draft c′, BPR​,BSH​Writes: Patched draft c′′ |

## 3. The Three-Level State Machine Specification

To prevent state explosion, control flow is factored into three nested levels: **(1) Tier 1 Macro-FSM**, **(2) Tier 2 Module Micro-FSM**, and **(3) Tier 3 ARIADNE MCTS Loop**.

### Level 1: Tier 1 Macro-Statechart (Repository Scope)

```
[S1: Preflight & Assess] 
       │ (Guard: Harness & inventory verified)
       ▼
[S2: Dependency & Domain Map] ◄─────────────────────────┐
       │ (Guard: DAG G=(V,E) & SCCs constructed)        │ (Macro-Regression:
       ▼                                                │  Undiscovered dynamic
[S3: Target Architecture Reimagine] ◄───────────┐       │  coupling / circular SCC)
       │ (Guard: Target contracts seeded in G)  │       │
       ▼                                        │       │
╔═══════════════════════════════════════════════╧═══════╧══╗
║ [S4: Composite State — Iterative DAG Transformation]     ║
║   Spawns concurrent Tier 2 Micro-FSMs for each v_i in V  ║
╚══════════════════════════════════════════════════════════╝
       │ (Guard: For all v_i in V, state(v_i) == M4_HARDENED)
       ▼
[S5: System Integration & E2E Hardening]

```

### Level 2: Tier 2 Per-Module Micro-FSM (Owned by Each DAG Vertex viv_ivi​)

Every translation unit $v_i \in V$ transitions independently through six states:

| State | Entry Condition | Active Component | Exit Transitions & Guards |
| --- | --- | --- | --- |
| M0: Blocked | Initial state upon DAG creation. | None (Waiting on DAG edges) | → M1 when ∀vj​∈Parents(vi​),state(vj​)=M4. |
| M1: Rule & Contract Extraction | All parent dependencies are hardened. | Arule​ (Initializer) | → M2 once BPM(vi​)​ is populated with business rules, legacy I/O fixtures, and parent target interfaces. |
| M2: ARIADNE Search Node | BPM(vi​)​ initialized; root state s0​=(∅,B0(vi​)​) created. | ARIADNE MCTS + 5-Agent Pool | → M4 if MCTS finds draft c⋆ with R(c⋆)=1.0.→ M3 if Aeval​ flags an ExtrinsicContractConflict or MCTS exhausts budget Bmax​ on an interface mismatch.→ Tier 1 S2/S3 if an unresolvable architectural cycle is discovered. |
| M3: Contract Negotiation | ARIADNE halted due to incompatible parent export on edge (vp​,vi​). | DAG Contract Mediator | Posts ContractPatchRequest to parent vp​ on G (forcing vp​: M4 → M5) and transitions vi​ → M0 (Blocked). |
| M4: Hardened & Published | Draft c⋆ achieves R(c⋆)=1.0. | Artifact Publisher | Publishes target AST, exported types, and verified test suite to G; unblocks downstream children in E. |
| M5: Invalidated (Regressed) | Downstream child vc​ raises a valid ContractPatchRequest, or a parent vp​ changes its signature. | Delta Context Updater | Appends new interface constraint to BPM(vi​)​ (preserving existing MCTS tree nodes in BSH​,BCT​ that remain valid!) and transitions → M2. |

### Level 3: Inside State M2 — The ARIADNE MCTS Formulation for Code2Code   arXiv

Inside `M2`, control is governed by ARIADNE’s **reward-informed tree search** rather than a rigid FSM:   

1. Selection (Softmax-UCB): Starting from root v0=(c0,B0(vi))v_0 = (c_0, \mathcal{B}_0^{(v_i)})v0​=(c0​,B0(vi​)​), traverse child states using ARIADNE’s UCB score:UCB(u)=Xˉ(u)+Cln⁡(N(v)+1)N(u)\text{UCB}(u) = \bar{X}(u) + C \sqrt{\frac{\ln(N(v) + 1)}{N(u)}}UCB(u)=Xˉ(u)+CN(u)ln(N(v)+1)​​sampling the next branch via temperature-controlled softmax p(u)∝exp⁡((UCB(u)−max⁡wUCB(w))/τ)p(u) \propto \exp((\text{UCB}(u) - \max_w \text{UCB}(w))/\tau)p(u)∝exp((UCB(u)−maxw​UCB(w))/τ) so the module actor does not prematurely lock into a flawed translation strategy.
2. State-Conditioned Action Expansion: Depending on the node’s current diagnostics on B(vi)\mathcal{B}^{(v_i)}B(vi​), the MCTS controller invokes one of the five agents:Empty or exhausted strategy → invoke Astr\mathcal{A}_{\text{str}}Astr​ to write new modernization strategies to BSH\mathcal{B}_{SH}BSH​.Uninstantiated strategy → invoke Agen\mathcal{A}_{\text{gen}}Agen​ to produce candidate target module c′c'c′.Low test coverage or ambiguous parity → invoke Atest\mathcal{A}_{\text{test}}Atest​ to add discriminative characterization tests to BCT\mathcal{B}_{CT}BCT​.Local compilation or test failure in c′c'c′ → invoke Apatch\mathcal{A}_{\text{patch}}Apatch​ using fault localization in BPR\mathcal{B}_{PR}BPR​.
3. Staged Evaluation & Composite Reward (Aeval\mathcal{A}_{\text{eval}}Aeval​): To make MCTS backpropagation meaningful for code2code transformation, define Aeval\mathcal{A}_{\text{eval}}Aeval​'s scalar reward R(st)∈[0,1]R(s_t) \in [0, 1]R(st​)∈[0,1] as a gated hierarchical objective:R(st)=w1⋅Icompile(ct)+Icompile(ct)⋅(w2⋅PassRatechar(ct,BCT)+w3⋅Scoreharden(ct))R(s_t) = w_1 \cdot \mathbb{I}_{\text{compile}}(c_t) + \mathbb{I}_{\text{compile}}(c_t) \cdot \Big( w_2 \cdot \text{PassRate}_{\text{char}}(c_t, \mathcal{B}_{CT}) + w_3 \cdot \text{Score}_{\text{harden}}(c_t) \Big)R(st​)=w1​⋅Icompile​(ct​)+Icompile​(ct​)⋅(w2​⋅PassRatechar​(ct​,BCT​)+w3​⋅Scoreharden​(ct​))where:Icompile(ct)∈{0,1}\mathbb{I}_{\text{compile}}(c_t) \in \{0, 1\}Icompile​(ct​)∈{0,1} checks clean target compilation and type-checking against upstream interfaces in G\mathcal{G}G.PassRatechar(ct,BCT)∈[0,1]\text{PassRate}_{\text{char}}(c_t, \mathcal{B}_{CT}) \in [0, 1]PassRatechar​(ct​,BCT​)∈[0,1] is the fraction of legacy characterization I/O and invariant tests passed.Scoreharden(ct)∈[0,1]\text{Score}_{\text{harden}}(c_t) \in [0, 1]Scoreharden​(ct​)∈[0,1] evaluates static security rules, idiomatic target patterns, and complexity bounds.With weights w1=0.2,w2=0.6,w3=0.2w_1 = 0.2, w_2 = 0.6, w_3 = 0.2w1​=0.2,w2​=0.6,w3​=0.2, R(st)=1.0R(s_t) = 1.0R(st​)=1.0 strictly guarantees compilation, 100% behavioral parity, and passed hardening checks.

## 4. Warm-Start Regressions: The Hidden Advantage of (1.2)

Embedding ARIADNE as the per-module node (`M2`) yields a major efficiency advantage during **cross-module regressions** (`M4` → `M5` → `M2`):

When Module $B$ forces Module $A$ to regress because Module $B$ needs an updated return type or async signature from Module $A$:

- Module AAA does not throw away its Local Blackboard B(A)\mathcal{B}^{(A)}B(A).
- Its characterization test suite in BCT(A)\mathcal{B}_{CT}^{(A)}BCT(A)​ and prior strategy hypotheses in BSH(A)\mathcal{B}_{SH}^{(A)}BSH(A)​ are retained in memory.
- The delta updater simply updates the interface constraint in BPM(A)\mathcal{B}_{PM}^{(A)}BPM(A)​ and re-enters ARIADNE (M2), allowing Apatch\mathcal{A}_{\text{patch}}Apatch​ or Agen\mathcal{A}_{\text{gen}}Agen​ to warm-start from the previously verified code draft c⋆c^\starc⋆ rather than re-translating Module AAA from scratch.

## Interactive Blueprint: 3-Tier Statechart & ARIADNE Node Explorer

Use the interactive diagram below to inspect the three architectural tiers and step through how a module transitions from Tier 1 DAG scheduling into the Tier 3 ARIADNE MCTS loop, including how an upstream contract regression warm-starts a previously hardened ARIADNE node:

::END_TURN::


::USER_MESSAGE::
Set the methodology to architecture (6) + (1.2).
[1] So the architecture is: there's 1 macro-orchestrator agent O owning FSM right? FSM is persisted to blackboard G right? After a non-O agent finishes, it produces to G, then O would validate and decide on next action right? When a state-regression happens, it originates from the non-O agent right? Then how does that non-O agent trigger the state change, is it communicate directly immediately back to O? Then O verify and take action right? Which actions?

[2] Then for the first few phases, for each phase, O communicates with a dedicated agent, those agents will not communicate with any other agents and just consume-produce to G right? Then when reach phase "Iterative DAG Transformation", what's the DAG Wave Scheduler, is that O or an agent that O directly communicates with? Then that DAG Wave Scheduler would orchestrate parallel clusters, each is ARIADNE, owning 4 subagents, local B_(v_i) right? Does a cluster have its own orchestrator?

[3] Why called Harel statechart? Show in the methodology.

[4] Holistically & exhaustively evaluate-compare against other paradigms:
[4.1] Static graph-pipeline (ReCodeAgent)
[4.2] Manager-worker (GVS5H)
[4.3] Hierarchy-tree (static agents, dynamic-adaptive routing, root node (orchestrator) owns FSM)
[4.4] Holonic/MOISE+

::MODEL_RESPONSE::
Setting the methodology to **Architecture (6) + (1.2)** establishes a **Hierarchical Harel Statechart over a Global DAG Blackboard (G\mathcal{G}G) with Encapsulated ARIADNE MCTS Clusters**.

Below is the exact operational mechanics of the orchestrator, the phase-by-phase execution model, the formal Harel Statechart specification, and an exhaustive comparison against competing multi-agent paradigms.

## [1] Orchestrator Mechanics, State Persistence, Validation & Regression Triggers

### 1.1 Does 1 Macro-Orchestrator O\mathcal{O}O own the FSM, and is it persisted on G\mathcal{G}G?

**Yes—with one critical engineering constraint: O\mathcal{O}O is split into a Deterministic Statechart Engine (Oengine\mathcal{O}_{\text{engine}}Oengine​) and an on-demand LLM Arbiter (Oarbiter\mathcal{O}_{\text{arbiter}}Oarbiter​).**

- Why O\mathcal{O}O is primarily deterministic code: If the state machine controller is a conversational LLM deciding state transitions via prompt reasoning, it can hallucinate illegal phase jumps or lose track of 100+ concurrent module states. Therefore, Oengine\mathcal{O}_{\text{engine}}Oengine​ is a deterministic runtime state machine (e.g., implemented in Python/Rust or LangGraph's state persistence layer).
- Persistence on G\mathcal{G}G: The entire state configuration is persisted transactionally inside the Global Blackboard (G\mathcal{G}G) as a state vector:Σ(G)=⟨Smacro,  M=[M(v1),M(v2),…,M(v∣V∣)],  Qevents⟩\Sigma(\mathcal{G}) = \Big\langle S_{\text{macro}}, \; \mathbf{M} = \big[M(v_1), M(v_2), \dots, M(v_{\vert{}V\vert{}})\big], \; \mathcal{Q}_{\text{events}} \Big\rangleΣ(G)=⟨Smacro​,M=[M(v1​),M(v2​),…,M(v∣V∣​)],Qevents​⟩Because Σ(G)\Sigma(\mathcal{G})Σ(G) lives on G\mathcal{G}G rather than inside an agent’s context window, the system supports zero-cost crash recovery and deterministic replay at any point.

### 1.2 Commit-and-Validate Cycle: How Non-O\mathcal{O}O Agents Interact with O\mathcal{O}O

Non-$\mathcal{O}$ agents never hold conversational chats with $\mathcal{O}$. Every interaction follows a strict **3-step Commit-and-Signal loop**:

1. Produce to G\mathcal{G}G: When a non-O\mathcal{O}O agent finishes its task, it writes its structured output artifact (e.g., DomainDAG.json, InterfaceContract.ts, or a verified module AST) to its designated partition on G\mathcal{G}G and pushes a typed completion event ev_TaskDone(agent_id, target_ref) to G\mathcal{G}G’s event queue Qevents\mathcal{Q}_{\text{events}}Qevents​.
2. Deterministic Guard Validation by O\mathcal{O}O: Oengine\mathcal{O}_{\text{engine}}Oengine​ consumes ev_TaskDone and runs a Transition Guard Predicate g(G)→{True,False}g(\mathcal{G}) \to \{\text{True}, \text{False}\}g(G)→{True,False} before allowing any state change (for example, verifying that the dependency graph is acyclic after SCC condensation, or that exported types parse cleanly under the target compiler/LSP).
3. Transition & Dispatch: If g(G)=Trueg(\mathcal{G}) = \text{True}g(G)=True, Oengine\mathcal{O}_{\text{engine}}Oengine​ updates Σ(G)\Sigma(\mathcal{G})Σ(G) to the next state and dispatches the next agent(s).

### 1.3 How State Regressions Originate, Propagate, and Execute

Yes, **all state regressions originate from empirical discoveries made by non-O\mathcal{O}O agents** (specifically the Evaluation Agent $\mathcal{A}_{\text{eval}}$ inside an ARIADNE node, or the System Integration Verifier in $S_5$).

#### How the Non-O\mathcal{O}O Agent Triggers It

When a non-$\mathcal{O}$ agent detects a blocking conflict that cannot be solved within its local scope, it **immediately halts its local loop**, writes a structured **RegressionInterrupt payload** to $\mathcal{G}$, and emits `ev_RegressionInterrupt(source_node, fault_type, target_dependency, evidence_trace)` directly to $\mathcal{O}$’s event bus.

#### How O\mathcal{O}O Verifies and Which Actions It Takes

When $\mathcal{O}$ intercepts `ev_RegressionInterrupt`, it first verifies the `evidence_trace` (to prevent a lazy worker agent from falsely blaming an upstream module for its own syntax error). If the conflict is ambiguous, $\mathcal{O}_{\text{engine}}$ invokes $\mathcal{O}_{\text{arbiter}}$ (a specialized LLM triage call) once to classify the fault. $\mathcal{O}$ then executes one of **three deterministic action protocols**:

| Regression Tier | Triggering Condition (Origin) | Who Verifies | Exact Actions Executed by O |
| --- | --- | --- | --- |
| Tier A: Intra-Module Backtrack (Local to ARIADNE) | Target code in module vi​ fails compilation, unit parity tests, or security linting due to a flaw in vi​'s own translation logic. | ARIADNE’s local Aeval​ + MCTS Controller (Does not wake O) | 1. Backpropagates low scalar reward R(st​)<1.0 up the local MCTS tree.2. Selects an alternative strategy branch in BSH(vi​)​ or invokes Apatch​ using fault diagnostics in BPR(vi​)​. |
| Tier B: Cross-Module Micro-Regression (DAG Neighborhood Scope) | ARIADNE working on downstream module vi​ proves that upstream parent vp​’s already-hardened interface contract on G is incompatible (e.g., missing fields, sync vs. async mismatch, wrong exception type). | Oengine​ checks LSP/compiler type trace + Oarbiter​ approves contract patch | 1. Park Downstream: Transitions M(vi​):M2_Search→M3_Negotiate→M0_Blocked.2. Invalidate Upstream: Transitions parent M(vp​):M4_Hardened→M5_Invalidated.3. Warm-Start Patch: Appends the new interface constraint to vp​'s local Problem Model BPM(vp​)​ without wiping vp​'s existing MCTS tree or test suite BCT(vp​)​.4. Re-schedule: Transitions M(vp​):M5_Invalidated→M2_Search to patch and re-harden vp​. |
| Tier C: Macro-Architectural Regression (Global Repo Scope) | An ARIADNE node discovers undocumented runtime reflection, dynamic class loading, or a hidden circular dependency edge (vi​→vk​) that invalidates the global DAG G=(V,E) built in Phase S2​. | Oengine​ verifies missing call-graph edge + Oarbiter​ confirms topological cycle | 1. Quiesce Affected Subgraph: Pauses active ARIADNE clusters in the cyclic subgraph (checkpointing their local blackboards B(v) to disk).2. Macro Rollback: Transitions Smacro​:S4_Transform→S2_Map (or S3_Reimagine).3. Graph Repair: Dispatches DependencyMapper to merge the cycle into a single Strongly Connected Component (SCC) vertex vSCC​ on G.4. Deep-History Resume (H∗): Re-enters S4_Transform; all unaffected modules remain in M4_Hardened, while only vSCC​ initializes a new ARIADNE cluster. |

## [2] Early Phase Isolation, the DAG Wave Scheduler, and ARIADNE Cluster Anatomy

### 2.1 Early Global Phases (S1,S2,S3S_1, S_2, S_3S1​,S2​,S3​): Isolated Hub-and-Spoke over G\mathcal{G}G

**Yes.** During the initial global phases, execution is strictly sequential and isolated:

- S1S_1S1​ (Preflight & Assess): O\mathcal{O}O invokes RepoAssessor. It reads the legacy codebase, writes the inventory and test-harness configuration to G\mathcal{G}G, and terminates.
- S2S_2S2​ (Map): O\mathcal{O}O invokes DependencyMapper. It reads the inventory from G\mathcal{G}G, builds the Condensed Dependency DAG G=(V,E)G=(V,E)G=(V,E) and domain map, writes them to G\mathcal{G}G, and terminates.
- S3S_3S3​ (Reimagine): O\mathcal{O}O invokes TargetArchitect. It reads G=(V,E)G=(V,E)G=(V,E) from G\mathcal{G}G, defines target architectural idioms and initial interface signatures on the edges of GGG, writes them to G\mathcal{G}G, and terminates.

None of these three agents ever communicate with one another. Each starts with a clean context window, consumes only the structured slices of $\mathcal{G}$ it needs, and produces structured artifacts back to $\mathcal{G}$.

### 2.2 What Is the "DAG Wave Scheduler"? Is It O\mathcal{O}O or a Separate Agent?

**The DAG Wave Scheduler is a deterministic algorithmic subroutine inside O\mathcal{O}O itself—it is NOT an LLM agent.**

When $\mathcal{O}$ transitions into macro-state $S_4$ (`Iterative DAG Transformation`), activating the DAG Wave Scheduler simply means $\mathcal{O}_{\text{engine}}$ runs an asynchronous **Topological Readiness Loop** (based on Kahn’s algorithm) over $G=(V,E)$ in $\mathcal{G}$:

$$
\text{ReadySet}(\mathcal{G}) = \Big\{ v_i \in V \;\Big\vert{}\; M(v_i) = \texttt{M0\_Blocked} \;\land\; \forall (v_p, v_i) \in E, \; M(v_p) = \texttt{M4\_Hardened} \Big\}
$$

Every time a module reaches `M4_Hardened` (or when $S_4$ first starts with leaf modules that have zero dependencies), $\mathcal{O}$’s scheduler recomputes $\text{ReadySet}(\mathcal{G})$ in milliseconds and spawns parallel **ARIADNE Clusters** for every ready vertex $v_i$. Making this a deterministic loop inside $\mathcal{O}$ avoids wasting LLM tokens on graph traversal.

### 2.3 Anatomy of an ARIADNE Cluster: Subagents, Local Blackboard B(vi)\mathcal{B}^{(v_i)}B(vi​), and Its Local Orchestrator

When $\mathcal{O}$’s DAG Wave Scheduler dispatches a ready module $v_i$, it spins up an isolated **ARIADNE Cluster** scoped exclusively to $v_i$:

1. Does each ARIADNE cluster have its own orchestrator?
Yes—each cluster has its own deterministic Local Orchestrator: the MCTS Tree Search Controller (CMCTS(vi)\mathcal{C}_{\text{MCTS}}^{(v_i)}CMCTS(vi​)​).
Just as O\mathcal{O}O orchestrates the repository DAG, CMCTS(vi)\mathcal{C}_{\text{MCTS}}^{(v_i)}CMCTS(vi​)​ orchestrates the search tree inside module viv_ivi​. It computes the Softmax-UCB formula over candidate code states st=(ct,Bt(vi))s_t = (c_t, \mathcal{B}_t^{(v_i)})st​=(ct​,Bt(vi​)​) and decides which subagent inside the cluster to invoke next.
2. Local Blackboard B(vi)\mathcal{B}^{(v_i)}B(vi​) (4 Slices):BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​ (Problem Model): Extracted legacy business rules, I/O contracts, and upstream interfaces copied from G\mathcal{G}G during state M1 (populated once by initializer Arule\mathcal{A}_{\text{rule}}Arule​).BSH(vi)\mathcal{B}_{SH}^{(v_i)}BSH(vi​)​ (Strategy Hypotheses): Candidate refactoring strategies and utility priors.BCT(vi)\mathcal{B}_{CT}^{(v_i)}BCT(vi​)​ (Counterexamples & Tests): Characterization test suites, differential I/O snapshots, and failing inputs.BPR(vi)\mathcal{B}_{PR}^{(v_i)}BPR(vi​)​ (Patch & Repair Records): Localized fault stack traces and repair history.
3. The Subagents Owned by the Cluster (4 Generative Subagents + 1 Scoring/Verification Harness):
Inside state M2, CMCTS(vi)\mathcal{C}_{\text{MCTS}}^{(v_i)}CMCTS(vi​)​ dispatches:Astr\mathcal{A}_{\text{str}}Astr​ (Strategy Agent): Reads BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​ →\to→ produces modernization strategies to BSH(vi)\mathcal{B}_{SH}^{(v_i)}BSH(vi​)​.Agen\mathcal{A}_{\text{gen}}Agen​ (Code Generation Agent): Reads BPM(vi),BSH(vi)\mathcal{B}_{PM}^{(v_i)}, \mathcal{B}_{SH}^{(v_i)}BPM(vi​)​,BSH(vi​)​ →\to→ produces candidate target code c′c'c′.Atest\mathcal{A}_{\text{test}}Atest​ (Adversarial Test Agent): Reads BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​ (blind to c′c'c′) →\to→ produces characterization tests to BCT(vi)\mathcal{B}_{CT}^{(v_i)}BCT(vi​)​.Apatch\mathcal{A}_{\text{patch}}Apatch​ (Code Repair Agent): Reads c′,BPR(vi)c', \mathcal{B}_{PR}^{(v_i)}c′,BPR(vi​)​ →\to→ produces surgical AST diff c′′c''c′′.Aeval\mathcal{A}_{\text{eval}}Aeval​ (Evaluation & Scoring Harness): Executes compiler, LSP type-check, characterization test runner, and security linter on c′c'c′, writing diagnostics to BCT(vi)/BPR(vi)\mathcal{B}_{CT}^{(v_i)}/\mathcal{B}_{PR}^{(v_i)}BCT(vi​)​/BPR(vi​)​ and returning the scalar reward R(st)∈[0,1]R(s_t) \in [0,1]R(st​)∈[0,1] to CMCTS(vi)\mathcal{C}_{\text{MCTS}}^{(v_i)}CMCTS(vi​)​.

## [3] Why Is It Called a "Harel Statechart"? Formalization in the Methodology

A classical **Finite State Machine (FSM)** (Mealy or Moore machine) is flat and sequential: the system can only be in *one single state* at any instant. If you model a repository with $N = 100$ modules where each module has 6 micro-states (`M0`–`M5`) plus 4 global phases, a flat FSM requires $4 + 6^{100}$ states—a catastrophic **state-space explosion**.

In 1987, **David Harel** introduced **Statecharts** (*"Statecharts: A Visual Formalism for Complex Systems"*), extending flat FSMs with four mathematical mechanisms that map directly to our methodology:

$$
\text{Harel Statechart } \mathcal{H} = \big( \mathcal{S}, \; \rho, \; \psi, \; \mathcal{T}, \; \mathcal{E}, \; \mathcal{G}_{\text{guards}}, \; \Theta_{\text{history}} \big)
$$

1. Hierarchy / Depth (ρ:S→2S\rho: \mathcal{S} \to 2^{\mathcal{S}}ρ:S→2S) — XOR Superstates:
Thehierarchy function ρ\rhoρ allows a state to contain child substates. In our methodology, S4S_4S4​ (Iterative_DAG_Transformation) is a Composite Superstate. High-level transitions—such as an emergency Macro-Regression arrow drawn from the outer boundary of S4S_4S4​ back to S2S_2S2​ (Map)—apply polymorphically to the entire composite state without needing 6N6^N6N individual transition arrows.
2. Orthogonality / Concurrency (ψ(S4)=AND\psi(S_4) = \texttt{AND}ψ(S4​)=AND) — Parallel Regions:
Harel decomposes a superstate into Orthogonal Regions (AND-states) separated visually by dashed lines. Inside S4S_4S4​, every module vi∈Vv_i \in Vvi​∈V is an independent orthogonal region R(vi)R(v_i)R(vi​):S4=R(v1)  ∥  R(v2)  ∥  …  ∥  R(v∣V∣)S_4 = R(v_1) \;\parallel\; R(v_2) \;\parallel\; \dots \;\parallel\; R(v_{\vert{}V\vert{}})S4​=R(v1​)∥R(v2​)∥…∥R(v∣V∣​)Instead of 6N6^N6N states, the state space scales linearly as 6×N6 \times N6×N. Each region R(vi)R(v_i)R(vi​) advances or regresses through its local Micro-FSM (M0–M5) concurrently.
3. Broadcast Event Calculus (ev[guard]/action) Over G\mathcal{G}G:
In Harel Statecharts, a transition in one orthogonal region R(vi)R(v_i)R(vi​) can emit an internal broadcast event that immediately triggers a guarded transition in another orthogonal region R(vp)R(v_p)R(vp​).In our methodology: When downstream module R(vi)R(v_i)R(vi​) fails due to an upstream contract mismatch, its transition M2→M3\texttt{M2} \to \texttt{M3}M2→M3 fires the action /emit(ev_ContractConflict(v_p, delta)). Region R(vp)R(v_p)R(vp​) has a transition listening for that exact broadcast event:M4_Hardened→ev_ContractConflict(vp,Δ)  [valid(Δ)]  /  append(BPM(vp),Δ)M5_Invalidated\texttt{M4\_Hardened} \xrightarrow{\text{ev\_ContractConflict}(v_p, \Delta)\;[\text{valid}(\Delta)] \; / \; \text{append}(\mathcal{B}_{PM}^{(v_p)}, \Delta)} \texttt{M5\_Invalidated}M4_Hardenedev_ContractConflict(vp​,Δ)[valid(Δ)]/append(BPM(vp​)​,Δ)​M5_Invalidated
4. Deep History Connectors (Θhistory=H∗\Theta_{\text{history}} = H^*Θhistory​=H∗):
When a Macro-Regression forces the system to exit superstate S4S_4S4​ back to S2S_2S2​ (Map) to resolve a newly discovered dependency cycle, a standard FSM forgets all internal progress when re-entering S4S_4S4​. Harel’s Deep History pseudostate (H∗H^*H∗) remembers the exact substate of every orthogonal region R(vk)R(v_k)R(vk​) upon exit. When O\mathcal{O}O transitions S2→H∗(S4)S_2 \to H^*(S_4)S2​→H∗(S4​), all 85 already-hardened modules resume directly in M4_Hardened without re-running a single token.

## [4] Holistic & Exhaustive Evaluation Against Other MAS Paradigms

Below is a rigorous architectural evaluation comparing **Architecture (6)+(1.2)** against the four reference paradigms across topology, state/memory semantics, regression locality, search capability, and token scalability.

### [4.1] Static Graph-Pipeline (ReCodeAgent)

- Architectural Mechanics: A 4-stage feed-forward DAG (Analyzer →\to→ Planner →\to→ Translator →\to→ Validator) where agents pass static planning documents forward, and tool-assisted repair loops (LSP/MCP compiler feedback) exist only inside the Translator/Validator nodes.
- Where It Fails vs. (6)+(1.2):Zero Topological Backtracking: If Validator discovers that Planner ordered two modules incorrectly or missed a dynamic dependency, ReCodeAgent has no backward control edge to re-invoke Planner. It is trapped in a local repair loop trying to hack around a structural planning error.Greedy Single-Draft Translation: Translator generates one draft and patches it linearly. If the initial design choice for a complex legacy class is fundamentally flawed, local patching degrades into spaghetti code. Our embedded ARIADNE (M2) node uses Softmax-UCB MCTS to explore multiple distinct refactoring strategies in parallel (BSH(vi)\mathcal{B}_{SH}^{(v_i)}BSH(vi​)​) and abandons dead-end drafts automatically.

### [4.2] Manager-Worker with Ledger Control (GVS5H)

- Architectural Mechanics: Introduced in GVS5H: Zero-Shot Self-Orchestration with Ledger-Based Control, a central LLM Manager maintains a text ledger (notes.md + task list) and dispatches stateless, fresh-context Workers (Brainstorm, Execute, Finalizer) one single task at a time, gated by an execution Verifier whose test verdict overrides the Manager’s "done" signal.   arXiv
- Where It Excels: Fresh context per worker call eliminates context-window degradation across long horizons, and the ground-truth Verifier prevents premature termination.
- Where It Fails vs. (6)+(1.2) at Repository Scale:Serial Single-Task Bottleneck: GVS5H enforces a strict single-file, single-worker turn loop (Manager \to 1 Worker \to Verifier →\to→ Manager). Modernizing a 300-module enterprise repository sequentially through an LLM Manager after every single file edit causes O(∣V∣×steps)O(\vert{}V\vert{} \times \text{steps})O(∣V∣×steps) serial latency. Architecture (6)+(1.2) uses O\mathcal{O}O's deterministic DAG Wave Scheduler to run dozens of orthogonal ARIADNE clusters in parallel.   arXivIn-Place File Overwrite vs. MCTS Branch Retention: In GVS5H, each Execute worker directly overwrites solution.py and replaces notes.md with a curated whole. If task #14 breaks an invariant established in task #6, GVS5H has lost the intermediate verified AST branching states. In contrast, our ARIADNE node stores code states st=(ct,Bt)s_t = (c_t, \mathcal{B}_t)st​=(ct​,Bt​) as nodes in an explicit MCTS tree, enabling instant, zero-loss switching between competing implementation branches.   arXivUnstructured Markdown Ledger Drift: While notes.md works well for competitive programming or single-module tasks, summarizing hundreds of cross-module type contracts into a single overwritten markdown file causes silent contract omission. Our Global Blackboard G\mathcal{G}G enforces a schema-validated ContractRegistry keyed to DAG edges.

### [4.3] Hierarchy-Tree (Static Agents + Dynamic Adaptive Routing + Root FSM)

- Architectural Mechanics: A multi-tier supervisor-worker tree where the Root Orchestrator owns a single flat FSM and dynamically routes tasks down through mid-level domain supervisors to leaf coder/tester agents.
- Where It Fails vs. (6)+(1.2):Lockstep Phase Coupling: Because the Root owns a flat FSM rather than a Harel Statechart with orthogonal AND-regions (R(vi)R(v_i)R(vi​)), a single module regression forces either a coarse phase rollback across the entire tree or requires the Root LLM to manually juggle per-module exceptions in its prompt context.Vertical Context Telephone Game: When Leaf Worker BBB needs an interface change in Leaf Worker AAA (located under a different branch supervisor), the request must travel up BBB's supervisor →\to→ Root →\to→ down AAA's supervisor →\to→ Leaf AAA. Crucial stack-trace and AST type details get summarized away at each hop. In (6)+(1.2), broadcast event ev_ContractConflict(v_p, delta) on G\mathcal{G}G routes the exact compiler diagnostic directly into vpv_pvp​'s Problem Model BPM(vp)\mathcal{B}_{PM}^{(v_p)}BPM(vp​)​ in O(1)O(1)O(1) hops.

### [4.4] Holonic / Organizational MAS (MOISE+ / ASPECS)   ResearchGate

- Architectural Mechanics: MOISE+ (Model of Organization for multI-agent SystEms) formalizes multi-agent organizations across three explicit dimensions:   ResearchGateStructural Specification (SS): Roles, inheritance, and recursive Holons (self-similar groups of agents that act as a single composite node to the outside world).Functional Specification (FS): Global goals decomposed into social schemes and missions.Deontic Specification (DS): Normative rules (Obligations, Permissions, Prohibitions) binding roles to missions at runtime.
- Deep Comparison with (6)+(1.2):Structural Isomorphism: Architecture (6)+(1.2) actually adopts the best structural aspect of Holonic MAS: each encapsulated ARIADNE cluster is a Holon (specifically, a Module Transformation Sub-Holon containing CMCTS\mathcal{C}_{\text{MCTS}}CMCTS​ + 5 internal agents + B(vi)\mathcal{B}^{(v_i)}B(vi​)) that appears to the Tier 1 Macro-Orchestrator O\mathcal{O}O as a single atomic vertex viv_ivi​ on the DAG!Why Pure MOISE+ Is Sub-Optimal for LLM Code2Code: In full MOISE+ / BDI agent frameworks, agents spend runtime compute reasoning over Deontic Norms, Coalition Formation, and Dynamic Role Adoption (deciding which agent should adopt which role or renegotiating social obligations when exceptions occur). When implemented with LLMs, dynamic role and norm negotiation causes severe meta-coordination chatter and role drift. In software modernization, agent roles (RuleExtractor, Strategy, CodeGen, TestGen, Eval, Patch) are known a priori; the uncertainty lies in the code search space and dependency graph, not in social organization. By compiling the Deontic permissions statically into Blackboard Read/Write ACLs and Harel Statechart Guards, (6)+(1.2) achieves Holonic modularity with zero normative reasoning overhead.

### Master Paradigm Comparison Matrix

| Evaluation Dimension | [4.1] Static Pipeline (ReCodeAgent) | [4.2] Manager-Worker Ledger (GVS5H) | [4.3] Hierarchy-Tree + Root Flat FSM | [4.4] Holonic / Organizational (MOISE+) | Proposed (6)+(1.2): Harel Statechart + DAG G + ARIADNE Nodes |
| --- | --- | --- | --- | --- | --- |
| Control Topology | Linear feed-forward DAG | Central LLM Manager loop + single Worker per turn | Multi-level Supervisor–Worker Tree | Recursive Holons + Deontic Norm Engine | 2-Tier Harel Statechart (O) + Parallel MCTS Sub-Holons (ARIADNE) |
| State Formalism | Implicit (current pipeline stage) | Task checklist in notes.md + Verifier pass/fail | Single flat FSM at Root node | Functional Social Schemes + Deontic States | Formal Harel Statechart (XOR Macro-States, AND Module Regions, H∗ History) |
| Shared Memory Substrate | Static pass-forward artifacts | Curated/overwritten notes.md + solution.py | Supervisor conversational message passing | Organizational Norm Base + Group Beliefs | Two-Level Blackboard: Global DAG G + Per-Module 4-Slice B(vi​) |
| Intra-Module Exploration | Single trajectory + local retry | Single trajectory (in-place file rewrite) | Single trajectory per worker | Mission-dependent (no native tree search) | Multi-branch MCTS (Softmax-UCB) over competing strategies (BSH​) |
| Cross-Module Regression | Fails (no backward edges to Planner) | Serial & Destructive (Manager re-tasks worker to overwrite files) | Bottlenecked (vertical hops through Root; coarse rollback) | High Overhead (triggers organizational norm/mission replanning) | Surgical & Warm-Started (ev_ContractConflict on G regresses only parent vp​, retaining its MCTS tree) |
| Parallelism Across Repo | Low (sequential file translation) | None (O(1)) — 1 worker executes per Manager turn | Moderate (top-down fan-out within a single global phase) | High (concurrent holons) | Maximum (O(width(G))) — all independent DAG waves run concurrently |
| LLM Coordination Overhead | Very Low (static handoffs) | High (Manager LLM called after every single task) | High (Supervisor LLMs summarize up/down tree) | Very High (Deontic/role deliberation overhead) | Minimal (Oengine​, DAG Wave Scheduler, and CMCTS​ are deterministic code) |

## Interactive Formalism Explorer: Harel Statechart (H\mathcal{H}H) & Regression Event Simulator

Use the interactive simulator below to inspect how the **Harel Statechart (O\mathcal{O}O)** manages concurrent **Orthogonal AND-Regions (R(vi)R(v_i)R(vi​))** inside Composite State $S_4$, how an **ARIADNE Cluster** triggers a broadcast `ev_ContractConflict` or `ev_CyclicDependencyFound` event on $\mathcal{G}$, and how **Deep History (H∗H^*H∗)** restores hardened modules after a macro-regression:
Generating interactive visual...Structuring code, this may take a minute

::END_TURN::


::USER_MESSAGE::
Also give the methodology name. Previously it's ARCMiS (Agentic Repository-level Code Migration for SLMs, however that's too generic; it doesn't need to resemble or contain parts from the seed name; generate plenty)

::MODEL_RESPONSE::
To replace **ARCMiS** with a name that stands out in top software engineering or AI venues (ICSE, FSE, ASE, NeurIPS, ICLR), the strongest names do two things simultaneously:

1. Evoke a clear structural metaphor (especially one that pairs naturally with ARIADNE as your inner node).
2. Decode into the exact technical mechanisms that differentiate your architecture (Harel Statecharts, Orthogonal Regions, DAG Blackboards, Surgical Regressions, and Encapsulated MCTS).

Below are **28 curated methodology names** organized across five thematic directions, complete with exact backronyms and paper-positioning rationales.

## 1. The "Labyrinth & Thread" Lineage (Direct Pairings with ARIADNE)

Because your inner Tier-3 solver is **ARIADNE** (who gave Theseus the thread to navigate the Labyrinth), naming the outer macro-architecture after the **Ship of Theseus**, the **Labyrinth’s Architect**, or the **Loom** creates an unforgettable conceptual hook for reviewers.

| Acronym / Name | Full Technical Expansion | Why the Metaphor Fits Your Architecture |
| --- | --- | --- |
| THESEUS (Top Pick) | Two-Tier Harel-Statechart Engine for Scoped Exploratory Unit Synthesis(Alt: Topological Harel-Statechart Execution with Sub-Holon Exploratory UCB Search) | Dual Metaphor: (1) In myth, Theseus is guided through the labyrinth by ARIADNE; (2) In philosophy, the Ship of Theseus replaces legacy planks one by one while preserving systemic identity—the exact definition of DAG-ordered code2code modernization with behavioral parity. |
| DAEDALUS | DAG-Aligned Event-Driven Architecture with Local UCB-Search Sub-holons | In Greek myth, Daedalus is the master architect who built the Labyrinth and crafted the floor for Ariadne. Position DAEDALUS as the deterministic Tier-1/Tier-2 architect that houses ARIADNE inside each module room. |
| LOOM | Layered Orthogonal Orchestration for Modernization(Alt: Local-MCTS and Orthogonal-Statechart Orchestration for Migration) | If each module runs an ARIADNE thread, LOOM is the macro-machine that weaves those orthogonal threads together across the warp and weft of the repository Dependency DAG. Short, punchy, and memorable. |
| CLEW | Concurrent Layered Execution with Warm-Start Statecharts | A "clew" is the archaic English word for Ariadne’s ball of thread (and the etymological root of the word clue). It highlights how your Blackboard G unrolls and rewinds threads during warm-start regressions. |
| MINOS | Macro-Statechart Integration of Nested Orthogonal Search-Clusters | King Minos presided over the Labyrinth where Ariadne operated; the backronym captures the exact mathematical nesting of Orthogonal Search-Clusters inside a Macro-Statechart. |
| ARGUS | Adaptive Repository Graph with UCB-Search Statecharts | Argus Panoptes (the hundred-eyed giant) watches over all concurrent orthogonal module regions R(v1​)∥⋯∥R(vN​) simultaneously on the Global Blackboard G. |
| ARACHNE | Adaptive Repository Architecture with Concurrent Harel Nodes & Exploration | Directly evokes web/graph construction (Map phase DAG) combined with multi-threaded concurrent execution. |

## 2. Formal Systems, Statechart & Concurrency Names

If you want the name to immediately signal **rigorous formal methods, concurrency, and deterministic control** (highlighting Harel Statecharts, `AND`-orthogonality, and DAG waves), these systems-style names work exceptionally well:

| Acronym / Name | Full Technical Expansion | Why It Works |
| --- | --- | --- |
| CHORD (Top Pick) | Concurrent Harel Orchestration over Repository DAGs | In music and geometry, a chord is multiple independent notes sounding simultaneously in structural harmony—mirroring Harel’s concurrent Orthogonal AND-Regions executing in topological waves. |
| MOSAIC | Multi-Tier Orthogonal Statecharts with ARIADNE-Integrated Clusters | Explicitly includes ARIADNE inside the acronym while conveying how isolated, verified module "tiles" are assembled into a complete modernized repository. |
| STRATA | Statechart-Treed Repository Architecture with Topological Agents | Captures the strict 3-layer stratification of your system: Tier 1 (Macro Statechart), Tier 2 (Topological DAG Wave Scheduler), and Tier 3 (Local MCTS Tree). |
| PRISM | Parallel Repository-DAG Integration via Statecharts and MCTS | Clean 5-letter systems acronym that names all three core primitives in the expansion: Repository-DAG, Statecharts, and MCTS. |
| WEAVE | Wave-Scheduled Encapsulated ARIADNE Vertices with Event-Driven Statecharts | Captures the DAG Wave Scheduler, the Encapsulated ARIADNE Vertices, and the Event-Driven Statechart in five letters. |
| CADENCE | Concurrent ARIADNE DAG-Execution with Nested Control Events | Highlights the rhythmic wave scheduling and broadcast control events (ev_ContractConflict) over the Global Blackboard. |
| PHALANX | Parallel Harel-Statechart Architecture with Local ARIADNE Nodes for X-Translation | Evokes a disciplined formation of autonomous units advancing in topological waves, capable of surgical fallback and reformation when a contract regression occurs. |
| VECTRA | Verified Event-Driven Code Transformation via Recursive Agents | Strong engineering/industrial tone suited for enterprise modernization tooling. |

## 3. Literary, Architectural & Metamorphosis Metaphors

Legacy modernization is fundamentally about **rewriting a text while preserving its underlying meaning and history**. These names emphasize transformation, deep history ($H^*$), and structural integrity:

| Acronym / Name | Full Technical Expansion | Why It Works |
| --- | --- | --- |
| PALIMPSEST | Parallel ARIADNE-Layered Iterative Modernization with Persistent Statechart Execution and Surgical Transitions | A palimpsest is a manuscript where legacy text is scraped back and new text is written over the exact same structural foundation while retaining deep traces of the original—a poetic, scholarly match for code2code transformation with Deep History (H∗). |
| JANUS | Joint ARIADNE-Nested Unified Statecharts | Janus is the two-faced Roman god of transitions and gates—simultaneously looking backward at legacy business rules (BPM​) and forward at target modernized code (c′). |
| KEYSTONE | Kahn-Scheduled Execution Yielding Statechart Transitions over Orthogonal Node Ensembles | Reference to Kahn’s topological DAG scheduling algorithm + the architectural keystone that locks independent masonry arches together. |
| TECTON | Topological Execution of Code Transformation via Orthogonal Nodes | Derived from tectonics / Greek tekton (master builder); highlights structural, plate-by-plate repository transformation. |
| CRUCIBLE | Concurrent Repository Upgrading via Contract-Informed Blackboards and Local Exploration | Emphasizes the adversarial hardening (Atest​ vs. Agen​) and high-assurance verification inside each module cluster. |
| TAPESTRY | Two-Tier Architecture for Parallel Encapsulated Search and Topological Repository Yielding | Conveys large-scale coordination across hundreds of interconnected modules. |

## 4. SLM-Specialized Names (If Retaining the Small Language Model Focus)

Your original name (**ARCMiS**) included *"for SLMs"*. One of the strongest scientific arguments for **Architecture (6)+(1.2)** is that **deterministic Harel Statecharts (Oengine\mathcal{O}_{\text{engine}}Oengine​) + scoped 4-slice Local Blackboards (B(vi)\mathcal{B}^{(v_i)}B(vi​))** eliminate long-context reasoning bloat, allowing **Small Language Models (SLMs)** to achieve repository-level migration that normally requires frontier models. If SLMs remain central to your thesis, use one of these:

| Acronym / Name | Full Technical Expansion | Why It Works for an SLM Thesis |
| --- | --- | --- |
| TESSERA (Top SLM Pick) | Two-Tier Encapsulated Statecharts for SLM-Enabled Repository Adaptation | A tessera is a small, bounded glass tile used to build a massive mosaic—just as bounded-context SLMs inside small ARIADNE nodes collaborate to modernize a massive repository. |
| BONSAI | Blackboard-Orchestrated Nested Statecharts and ARIADNE Instances | Evokes pruning, bounded tree search (MCTS), and compact/small models achieving complex structural form through disciplined state pruning. |
| SCALPEL | Statechart-Controlled ARIADNE Local-Search for Parallel Enterprise Legacy-Migration | Contrasts "brute-force monolithic LLM prompting" with surgical, bounded-context precision and localized micro-regressions. |
| LATTICE | Local ARIADNE Trees in Topological Isolation for Code Evolution | Emphasizes the strict context isolation that allows small models to operate without context overflow. |
| MICRON | Multi-Tier Isolated Code Refactoring via Orthogonal Nodes | Signals compact, micro-scoped state machines and small model efficiency. |

## 5. How the Top Candidates Look as Paper Titles

To see how these read in an academic submission, compare these four title framings:

1. The Mythological + Philosophical Hook (THESEUS):THESEUS: Two-Tier Harel-Statechart Execution with Scoped Exploratory Unit Solvers for Repository-Scale Code Modernization
Why reviewers love it: In the introduction, you write one memorable sentence: "Just as Theseus navigated the Labyrinth using Ariadne's thread—and just as the Ship of Theseus replaces every legacy timber while preserving the vessel's functional identity—THESEUS orchestrates repository-wide modernization through a deterministic Harel Statechart while delegating intra-module transformation to encapsulated ARIADNE search clusters."
2. The Formal Concurrency Hook (CHORD):CHORD: Concurrent Harel Orchestration over Repository DAGs for Multi-Agent Code-to-Code Transformation
Why reviewers love it: Crisp, zero fluff, immediately tells a software engineering reviewer (ICSE/FSE) that the core contribution is formal concurrency control (Harel Statecharts) over dependency DAGs.
3. The Explicit ARIADNE Integration Hook (MOSAIC or WEAVE):MOSAIC: Multi-Tier Orthogonal Statecharts with ARIADNE-Integrated Clusters for Legacy Code Migration
Why reviewers love it: Directly credits and connects the sub-node architecture (ARIADNE) in the acronym itself.
4. The SLM Efficiency Hook (TESSERA):TESSERA: Two-Tier Encapsulated Statecharts for SLM-Enabled Repository Adaptation
Why reviewers love it: Positions the architecture as the structural key that unlocks repository-scale code2code transformation for small, bounded-context models.
Want me to draft the formal Methodology section (with LaTeX equations and algorithms) using your chosen name?

::END_TURN::


::USER_MESSAGE::
Let the previous (most recent) conversation QA section be A. Now is B.
[1] In [A.1.2], when an agent produces an event to G, to decide what agent comes next, does that agent have to watch / continuouslly poll G? You didn't mention this. Also you are assuming guard can always be deterministically verified - I believe otherwise - either that's not true or you are only performing structural validation that is too shallow to be full validation. Also you are saying O_engine dispatches the next agent(s), is this programmatically coded? I mean if the next agent(s) needs some kind of state/prompt/instructions/etc. then woudn't that be insufficient?
[2] In [A.1.3], why would tier A (intra-module) requires O step-in, I thought since it was local to ARIADNE cluster, it should be handled internally? Is it for standardization, is it because it affects other modules as well?
[3] So inside a cluster there's a controller (is it LLM?), 4 subagents, 1 eval-scoring non-agent (non-LLM) right?
[4] Are the phases before DAG too few, compared to Claude Modernization Plugin?

::MODEL_RESPONSE::
### [B.1] Event Dispatching, Guard Verification Depth, and Dynamic Context Hydration

Your critique exposes three areas where a naive implementation of $\mathcal{O}$ would fail. Below is the exact engineering mechanics required for each.

#### 1. Do Non-O\mathcal{O}O Agents Watch or Poll G\mathcal{G}G?

**No. Agents never poll or watch G\mathcal{G}G.** Having LLM agents run as persistent daemons polling a blackboard wastes memory and compute.

- Push-Based Ephemeral Invocation: All non-O\mathcal{O}O agents are stateless, ephemeral worker functions that do not exist as active processes until invoked.
- Single Event-Loop Listener: Only the deterministic statechart runtime (Oengine\mathcal{O}_{\text{engine}}Oengine​) runs an active event loop listening to G\mathcal{G}G’s event queue Qevents\mathcal{Q}_{\text{events}}Qevents​ (implemented via an asynchronous queue or pub/sub bus).
- Execution Lifecycle: When Agent XXX finishes its task, its wrapper commits its artifact to G\mathcal{G}G, pushes ev_TaskDone(Agent_X) to Qevents\mathcal{Q}_{\text{events}}Qevents​, and terminates immediately (freeing its context window). Oengine\mathcal{O}_{\text{engine}}Oengine​ wakes on the event, evaluates the transition guard, and programmatically instantiates the next agent process.

#### 2. Can Guards Always Be Deterministically Verified? (Fixing Shallow Structural Validation)

**You are completely right:** purely deterministic checks (JSON schema validation, AST parsing, DAG acyclicity, or compiler exit codes) only verify **syntax and structure**. They are blind to **semantic flaws**—for instance, whether `DependencyMapper` missed implicit coupling through shared database tables, whether extracted business rules omitted an edge case, or whether `TargetArchitect` over-engineered an interface.

To prevent shallow validation without making the state machine itself non-deterministic, every phase-boundary guard $g(\mathcal{G})$ is factored into a **Two-Stage Guard Pipeline**:

$$
g(\mathcal{G}) = g_{\text{struct}}(\mathcal{G}) \;\land\; g_{\text{sem}}(\mathcal{G})
$$

1. Stage 1: Deterministic Structural Guard (gstructg_{\text{struct}}gstruct​ — Non-LLM, 0 Tokens):
Programmatic assertions checking schema completeness, valid file:line source pointers, DAG acyclicity after Strongly Connected Component (SCC) condensation, and LSP/compiler syntax checks. If gstructg_{\text{struct}}gstruct​ fails, Oengine\mathcal{O}_{\text{engine}}Oengine​ immediately loops the producing agent with the deterministic error trace (never wasting tokens on Stage 2).
2. Stage 2: Adversarial Semantic Guard (gsemg_{\text{sem}}gsem​ — Read-Isolated LLM Critic Agent):
Only when gstruct(G)=Trueg_{\text{struct}}(\mathcal{G}) = \text{True}gstruct​(G)=True does Oengine\mathcal{O}_{\text{engine}}Oengine​ invoke a lightweight, read-isolated Semantic Critic Agent (mirroring the official Claude Code Modernization plugin’s dual-agent rule verification and architecture-critic).The Critic inspects the produced artifact on G\mathcal{G}G against the legacy source citations and outputs a strictly typed schema:JSON{
  "verdict": "PASS" | "REJECT_LOCAL" | "REGRESS_UPSTREAM",
  "violated_invariants": ["..."],
  "remediation_directive": "..."
}
Why the Statechart Remains Deterministic: The LLM Critic does not mutate the state machine directly; it evaluates the boolean predicate gsem(G)∈{PASS,REJECT}g_{\text{sem}}(\mathcal{G}) \in \{\texttt{PASS}, \texttt{REJECT}\}gsem​(G)∈{PASS,REJECT} and writes a CritiqueRecord to G\mathcal{G}G. Oengine\mathcal{O}_{\text{engine}}Oengine​ then deterministically executes the state transition conditioned on that boolean verdict.

#### 3. How Programmatic Dispatch Passes Dynamic Instructions & State to the Next Agent

If $\mathcal{O}_{\text{engine}}$ is programmatic code, simply calling `spawn_agent("TargetArchitect")` with a static prompt template would fail whenever an agent is woken up by a guard rejection or a cross-module regression.

Instead, every transition edge $e = (S_{\text{from}} \xrightarrow{\text{ev}[g]} S_{\text{to}})$ in $\mathcal{O}_{\text{engine}}$ executes a **Transition Action** called the **ContextAssembler (Prompt & State Hydrator)**. Rather than passing a static prompt, $\mathcal{O}_{\text{engine}}$ constructs and injects a typed **TaskActivationEnvelope** into the dispatched agent:

| Envelope Field | How It Is Constructed from G | Purpose |
| --- | --- | --- |
| ActivationMode | Set by the active transition edge in Oengine​: INITIAL | CRITIC_RETRY | WARM_START_REGRESSION. | Switches the agent's system prompt between greenfield synthesis vs. surgical delta repair. |
| ScopedBlackboardSlice | Deterministic graph query over G pulling only the relevant node vi​, its 1-hop DAG neighbors N(vi​), and cited legacy snippets. | Gives the agent complete local context without polluting its window with unrelated repository files. |
| DeltaDirective | Populated from the triggering CritiqueRecord (from gsem​) or RegressionInterrupt (from a downstream module). | Provides the exact natural-language + formal instruction on what failed, why, and which existing invariants must not be broken. |

### [B.2] Clarification on [A.1.3] Tier A (Intra-Module Backtrack): Does O\mathcal{O}O Step In?

**Good catch—Tier A does NOT involve O\mathcal{O}O at all.** In [A.1.3], placing Tier A inside a table whose column header read *"Exact Actions Executed by O\mathcal{O}O"* was misleading phrasing on my part.

- Strict Internal Autonomy: Tier A (a syntax error, failed characterization test, or suboptimal refactoring approach inside module viv_ivi​) is 100% private to viv_ivi​’s ARIADNE cluster. Because viv_ivi​ has not yet published a hardened artifact to G\mathcal{G}G and is not requesting any upstream interface changes, no other module is affected.
- Zero Global Traffic: Tier A backtracks happen strictly inside the cluster’s local MCTS tree T(vi)\mathcal{T}^{(v_i)}T(vi​) and local blackboard B(vi)\mathcal{B}^{(v_i)}B(vi​). O\mathcal{O}O is never notified, and nothing is written to G\mathcal{G}G.
- When Does a Local Struggle Escalate to O\mathcal{O}O? An ARIADNE cluster only crosses the boundary to wake O\mathcal{O}O under two conditions:Extrinsic Blocker Detected (Immediate Escalation →\to→ Tier B/C): The cluster’s evaluator proves the failure is caused by an incompatible upstream contract on G\mathcal{G}G or an undiscovered dependency cycle.MCTS Rollout Budget Exhausted (t=Bmax⁡t = B_{\max}t=Bmax​): The cluster exhausts all Bmax⁡B_{\max}Bmax​ MCTS iterations without reaching reward R=1.0R = 1.0R=1.0. It then emits ev_BudgetExhausted(v_i, best_draft, blocker_trace) to O\mathcal{O}O so O\mathcal{O}O can trigger a contract renegotiation or split the module.

### [B.3] Exact Anatomy Inside an ARIADNE Cluster: Controller, Subagents, and Evaluator

Inside each parallel ARIADNE cluster (instantiated for a ready vertex $v_i$ in state `M2`), the components break down as follows:

1. Local Cluster Orchestrator (CMCTS(vi)\mathcal{C}_{\text{MCTS}}^{(v_i)}CMCTS(vi​)​) — Non-LLM (Algorithmic Code):Not an LLM. CMCTS(vi)\mathcal{C}_{\text{MCTS}}^{(v_i)}CMCTS(vi​)​ is a lightweight, deterministic MCTS loop.In each iteration, it traverses the search tree of states st=(ct,Bt(vi))s_t = (c_t, \mathcal{B}_t^{(v_i)})st​=(ct​,Bt(vi​)​) by computing Softmax-UCB over scalar rewards RRR, selects a node uuu, and inspects deterministic status flags on uuu to dispatch one of the cluster's subagents (e.g., unexpanded strategy →Astr\to \mathcal{A}_{\text{str}}→Astr​; uninstantiated draft →Agen\to \mathcal{A}_{\text{gen}}→Agen​; missing test coverage →Atest\to \mathcal{A}_{\text{test}}→Atest​; unverified draft →Aeval\to \mathcal{A}_{\text{eval}}→Aeval​; failed verification →Apatch\to \mathcal{A}_{\text{patch}}→Apatch​).
2. 4 Generative Subagents — LLM Agents:Astr\mathcal{A}_{\text{str}}Astr​ (Strategy Agent — LLM): Writes candidate refactoring hypotheses to BSH(vi)\mathcal{B}_{SH}^{(v_i)}BSH(vi​)​.Agen\mathcal{A}_{\text{gen}}Agen​ (Code Gen Agent — LLM): Synthesizes candidate target code c′c'c′ from BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​ and BSH(vi)\mathcal{B}_{SH}^{(v_i)}BSH(vi​)​.Atest\mathcal{A}_{\text{test}}Atest​ (Test Gen Agent — LLM): Synthesizes characterization and property-based tests into BCT(vi)\mathcal{B}_{CT}^{(v_i)}BCT(vi​)​ (blind to c′c'c′).Apatch\mathcal{A}_{\text{patch}}Apatch​ (Code Repair Agent — LLM): Applies surgical AST/diff repairs to c′c'c′ using BPR(vi)\mathcal{B}_{PR}^{(v_i)}BPR(vi​)​.
3. What About the 5th Component (Aeval\mathcal{A}_{\text{eval}}Aeval​: Eval-Scoring)? Is It Non-LLM or LLM?Aeval\mathcal{A}_{\text{eval}}Aeval​ is a Two-Stage Hybrid (Deterministic Execution Harness + Diagnostic LLM Distiller):Stage 3a: Deterministic Verification Harness (Non-LLM Tooling): Runs the compiler, LSP type-checker, characterization test suite (BCT(vi)\mathcal{B}_{CT}^{(v_i)}BCT(vi​)​), differential I/O harness, and static security linter. This deterministically computes the compilation bit Icompile\mathbb{I}_{\text{compile}}Icompile​ and test pass rate PassRatechar\text{PassRate}_{\text{char}}PassRatechar​.Stage 3b: Diagnostic Distiller & Parity Critic (Lightweight LLM Agent): Why can't Aeval\mathcal{A}_{\text{eval}}Aeval​ be purely a non-LLM script? For two critical reasons in ARIADNE and code2code:Fault Localization into BPR(vi)\mathcal{B}_{PR}^{(v_i)}BPR(vi​)​: If Stage 3a fails with a 400-line compiler or test traceback, dumping raw stderr directly into Apatch\mathcal{A}_{\text{patch}}Apatch​ degrades SLM repair accuracy. In ARIADNE, Aeval\mathcal{A}_{\text{eval}}Aeval​ distills raw execution failures into structured root-cause records on BPR(vi)\mathcal{B}_{PR}^{(v_i)}BPR(vi​)​ and classifies whether the error is Intrinsic (Tier A) or an Extrinsic Upstream Contract Conflict (Tier B).Semantic Rule & Idiom Audit (Scoreharden\text{Score}_{\text{harden}}Scoreharden​): If Stage 3a passes 100% of unit tests, the LLM critic performs a fast check verifying that c′c'c′ genuinely implements all Given/When/Then business rule cards in BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​ and uses idiomatic target patterns (preventing line-by-line "COBOL-written-in-Java").

### [B.4] Are the Pre-DAG Phases Too Few Compared to the Claude Modernization Plugin?

**Yes—collapsing the pre-DAG workflow into just 3 phases (Assess, Map, Reimagine) was too coarse compared to the official Claude Code Modernization Plugin.**

In Anthropic’s official `code-modernization` plugin (`anthropics/claude-plugins-official`), the pre-build pipeline consists of **5 distinct discovery and specification phases** before module transformation (`transform`) ever begins:

1. modernize + modernize-preflight (Intent capture + proving the legacy build/test harness actually works)
2. modernize-assess (Static inventory, dialect analysis, complexity, technical debt, security posture)
3. modernize-map (Structural dependencies, entry points, data flows, and implicit shared-state coupling)
4. modernize-extract-rules + modernize-review (Mining testable Given/When/Then business rule cards with file:line citations, separating observed behavior from inferred intent, re-checked by a second adversarial agent)
5. modernize-brief + architecture-critic (Phased migration plan, target architecture contracts, audited by an adversarial architecture-critic agent to prevent over-engineering)

#### Why Moving Extract-Rules Pre-DAG (S3S_3S3​) Improves Our Architecture

In our earlier draft, business rule extraction (`A_rule`) was pushed inside the per-module DAG wave (`M1`). Aligning with the Claude plugin by elevating **Extract-Rules into a dedicated Pre-DAG Phase (S3S_3S3​)** solves a major architectural issue:

- Legacy business rules frequently span multiple files (cross-cutting domain workflows). If you define target architecture contracts (Reimagine / Brief) before extracting business rules, the target interfaces will miss domain invariants!
- By running Extract-Rules globally in S3S_3S3​ (with a dual-agent Semantic Guard gsemg_{\text{sem}}gsem​) and attaching verified Given/When/Then rule cards to the DAG vertices on G\mathcal{G}G, Phase S4S_4S4​ (Target Brief & Contract Reimagine) can design module interfaces that are guaranteed to support every extracted business rule.
- Even better: inside the DAG wave, state M1 shrinks to a zero-LLM ContextHydrator that simply copies viv_ivi​’s already-verified rule cards from G\mathcal{G}G into BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​.

#### The Refined 7-Phase Macro-Statechart (O\mathcal{O}O) Aligned with Claude code-modernization

| Macro-State | Official Claude Plugin Equivalent | Active Tier-1 Agent(s) | Two-Stage Guard g(G)=gstruct​∧gsem​ |
| --- | --- | --- | --- |
| S0​: Intent & Preflight | modernize + modernize-preflight | PreflightProbe | gstruct​: Legacy build compiles; baseline test harness executes; INTENT.md schema valid. |
| S1​: Assess | modernize-assess | LegacyAnalyst (RepoAssessor) | gstruct​: 100% file inventory coverage (scc/cloc).gsem​: AssessmentCritic verifies no unsupported metrics or unflagged dialects. |
| S2​: Map | modernize-map | DependencyMapper | gstruct​: Condensed DAG G=(V,E) is acyclic; all entry points reachable.gsem​: TopologyCritic audits for implicit data-flow/DB/file coupling missed by static imports. |
| S3​: Extract Rules | modernize-extract-rules + review | BusinessRuleExtractor | gstruct​: Every rule card has valid Given/When/Then fields and legacy/file:line citations.gsem​: RuleAuditor (second agent) re-verifies every card against legacy source and checks observed behavior vs. inferred intent separation. |
| S4​: Brief & Reimagine Contracts | modernize-brief + target architecture spec | TargetArchitect | gstruct​: All DAG edges eij​∈E have parseable target interface stub definitions.gsem​: ArchitectureCritic audits target design for over-engineering and checks 100% rule-card coverage. |
| S5​: DAG Transform (Composite AND-State) | transform (module-by-module build + pinning tests) | DAG Wave Scheduler → Parallel ARIADNE Clusters | Per-Module Exit (M4): R(c⋆)=1.0 (Stage 3a compiler + characterization tests + Stage 3b semantic rule parity). |
| S6​: Verify & Harden | modernize-verify + modernize-harden | IntegrationVerifier + SecurityHardener | gstruct​: Full repository E2E test suite passes; zero critical SAST/CVE findings.gsem​: Independent module verdict sign-off report generated. |

### Interactive Visualizer: Two-Stage Guard Pipeline & 7-Phase Claude-Aligned Statechart

Use the interactive diagram below to step through the expanded **7-Phase Pre-DAG & Execution Pipeline (S0→S6S_0 \to S_6S0​→S6​)**, inspect how the **Two-Stage Guard (gstruct∧gsemg_{\text{struct}} \land g_{\text{sem}}gstruct​∧gsem​)** and **ContextAssembler** operate at each phase boundary, and examine the exact **Non-LLM vs. LLM split** inside an ARIADNE cluster:
Generating interactive visual...Structuring code, this may take a minute

::END_TURN::


::USER_MESSAGE::
C.
[1] Should non-ARIADNE agents be able to dynamically spawn and orchestrate subagents to complete their goal?

::MODEL_RESPONSE::
**No — non-ARIADNE agents must NOT autonomously spawn and orchestrate subagents inside their own LLM execution loop; instead, they must use Declarative Scatter-Gather (Map-Reduce) orchestrated deterministically by Oengine\mathcal{O}_{\text{engine}}Oengine​ over G\mathcal{G}G.**

This distinction is one of the most important architectural boundaries in a repository-scale MAS (especially when targeting bounded-context models or SLMs). Here is why raw LLM subagent orchestration fails, why parallel fan-out is still mandatory for scale, and how to formalize **Declarative Scatter-Gather** inside your Harel Statechart.

## 1. The Core Dilemma: Why Scale Demands Fan-Out, But LLM Orchestration Breaks the Architecture

In phases $S_1$ (`Assess`), $S_2$ (`Map`), $S_3$ (`Extract Rules`), and $S_4$ (`Brief & Reimagine`), a real enterprise repository may contain 500+ files and 250,000+ lines of code. A single invocation of `BusinessRuleExtractor` or `RuleAuditor` cannot fit the entire repository into one context window without suffering severe "lost-in-the-middle" degradation or exceeding token limits. (Indeed, the official Claude Code Modernization plugin fans out parallel subagents during `assess`, `map`, and `extract-rules`.)

However, there are **two fundamentally different ways** to implement subagent fan-out:

### Pattern A: Autonomous LLM-Driven Subagent Orchestration (REJECT)

In this pattern, a Tier-1 agent (e.g., `BusinessRuleExtractor`) is given a `spawn_subagent(role, prompt, files)` tool and acts as an autonomous LLM Manager in a multi-turn ReAct loop—spawning workers, receiving their text responses back into its own conversation history, deciding whether to spawn more, and finally summarizing everything to $\mathcal{G}$.

**Why Pattern A violates Architecture (6)+(1.2):**

1. Context Window Implosion at the Parent Agent: If BusinessRuleExtractor spawns 40 subagents and receives 40 sets of extracted business rules back into its own prompt history to synthesize them, the parent agent's context window still overflows.
2. Shadow State Outside G\mathcal{G}G: While the parent LLM is mid-conversation with 40 subagents, the execution state lives inside the parent LLM’s transient message buffer rather than the Global Blackboard’s state vector Σ(G)\Sigma(\mathcal{G})Σ(G). If worker #38 times out, you lose deterministic checkpointing and Harel Deep History (H∗H^*H∗).
3. Unbounded Recursion & Cost: If subagents can also spawn subagents, a confused SLM can trigger recursive fan-out loops (Manager →\to→ Sub-Manager →\to→ Worker), re-introducing the exact "telephone game" context drift we rejected in Architecture [4.3].

### Pattern B: Declarative Scatter-Gather via Oengine\mathcal{O}_{\text{engine}}Oengine​ (ADOPT)

Instead of letting an LLM agent *orchestrate* subagents in its private context window, **we separate Task Decomposition (LLM/Static) from Subagent Orchestration (Oengine\mathcal{O}_{\text{engine}}Oengine​)**.

Whenever a non-ARIADNE phase needs subagents, the Macro-State $S_m \in \{S_1, S_2, S_3, S_4, S_6\}$ is modeled in the Harel Statechart as a **3-Stage Internal Sub-Statechart (Scatter →\to→ Parallel Map →\to→ Reduce)**:

```
╔══════════════════════════════════════════════════════════════════════════════╗
║ Macro-State S_m (e.g., S3: Extract Business Rules)                           ║
║                                                                              ║
║  [S_m.1: Partition / Scatter]                                                ║
║     │ Writes WorkManifest {shard_1 ... shard_K} to Blackboard G              ║
║     ▼                                                                        ║
║  ┌────────────────────────────────────────────────────────────────────────┐  ║
║  │ [S_m.2: Parameterized AND-Region — Parallel Stateless Map Subagents]   │  ║
║  │   Worker(shard_1)  ||  Worker(shard_2)  ||  ...  ||  Worker(shard_K)   │  ║
║  │   (Each reads only shard_k from G and writes directly to G[shard_k])   │  ║
║  └────────────────────────────────────────────────────────────────────────┘  ║
║     │ Guard: For all k in 1..K, g_struct(G[shard_k]) == PASS                 ║
║     ▼                                                                        ║
║  [S_m.3: Cross-Shard Linker / Reduce]                                        ║
║     │ Reads only exported boundary metadata on G to link cross-file items    ║
╚══════════════════════════════════════════════════════════════════════════════╝

```

**Why Pattern B is superior:**

- Zero Parent Context Accumulation: Map subagents never report back to a parent LLM. Each worker WkW_kWk​ writes its structured output (e.g., RuleCard[]) directly into its partition G[shardk]\mathcal{G}[\text{shard}_k]G[shardk​] and terminates.
- 1-Level Depth Invariant (Dmax⁡=1D_{\max} = 1Dmax​=1): Subagents have no spawn capability. All parallel scheduling, retries on failed shards, and rate-limiting are handled deterministically by Oengine\mathcal{O}_{\text{engine}}Oengine​ (identical to LangGraph’s Send() map-reduce API).
- Bounded SLM Context: Because shard size is capped deterministically during Scatter (e.g., ≤1,500\le 1,500≤1,500 LoC or 1 module cluster per shard), every single subagent call fits comfortably inside an SLM’s optimal context window.

## 2. What If a Subagent Needs Information Outside Its Shard?

A common reason designers give LLM agents dynamic subagent-spawning tools is **discovery**: *"What if Shard Worker W4W_4W4​ is extracting business rules from BillingService.java and encounters a call to TaxPolicy.calculate() located in another shard?"*

You still do not need recursive subagent spawning. You handle this via two bounded mechanisms:

1. Read-Only Deterministic Tooling (Intra-Turn): Every Map subagent has access to deterministic, zero-LLM code navigation tools over the repository index (LSP go_to_definition, find_references, AST symbol body lookup). It can pull the exact 20-line definition of TaxPolicy.calculate() without spawning another LLM agent.
2. Declarative UnresolvedLink Markers (Resolved in S_m.3 Reduce): If the external dependency represents a complex cross-module workflow, Worker W4W_4W4​ simply writes an UnresolvedCrossLink(source_rule_id, target_symbol) tag into G[shard4]\mathcal{G}[\text{shard}_4]G[shard4​]. Once all Map workers finish, the S_m.3 Cross-Shard Linker queries G\mathcal{G}G for all UnresolvedCrossLink tags and stitches the two shards' rule cards together.

## 3. Phase-by-Phase Subagent Policy for All Non-ARIADNE Agents

Applying this rule across the entire 7-Phase architecture gives every non-ARIADNE phase a clean, predictable execution topology:

| Macro-Phase / Component | Needs Subagent Fan-Out? | Exact Scatter → Map → Reduce Mechanics on G |
| --- | --- | --- |
| S0​: Intent & Preflight | No (Single Agent) | PreflightProbe runs deterministic build/test CLI commands and writes HarnessConfig + INTENT to G. |
| S1​: Assess | Yes (Directory Shards) | Scatter: Deterministic scc/AST scanner groups repo into directory shards.Map: Parallel ShardAssessor subagents analyze dialect quirks & tech debt per shard.Reduce: AssessmentSynthesizer aggregates shard reports into the global risk matrix. |
| S2​: Map (DAG Construction) | Hybrid (Static Tool + Targeted Map) | Scatter: Deterministic LSP/Tree-sitter builds 95% of the explicit Call/Import graph and flags ambiguous files (reflection, raw SQL, event buses).Map: Parallel ImplicitCouplingScanner subagents inspect only those flagged files to emit hidden dependency edges.Reduce: Deterministic Tarjan SCC algorithm condenses cycles into DAG G=(V,E). |
| S3​: Extract Rules | Yes (DAG Vertex / Cluster Shards) | Scatter: Oengine​ partitions V into bounded vertex batches.Map: Parallel RuleMiner subagents extract Given/When/Then rule cards per vertex directly to G.Reduce: CrossModuleRuleLinker resolves UnresolvedCrossLink markers across connected DAG edges. |
| S4​: Brief & Reimagine Contracts | Yes (Domain Subgraph Shards) | Scatter: MacroArchitect defines global target idioms/conventions on G.Map: Parallel ContractStubber subagents generate target type signatures for each DAG edge eij​∈E.Reduce: Deterministic target compiler/LSP verifies that all interface stubs mutually type-check! |
| Stage-2 Semantic Critics (gsem​) | Yes (Batch-Sharded Auditing) | Auditing 500 rule cards in S3​ or 200 contracts in S4​ in a single Critic call would overflow context. Oengine​ fans out parallel ShardCritic subagents (each auditing 10–15 rule cards against their exact file:line citations) and takes the logical conjunction ⋀k=1K​gsem​(shardk​). |
| S6​: Verify & Harden | Yes (Test Suite & SAST Shards) | Parallel IntegrationFixer and SecurityAuditor subagents dispatched per failing E2E test trace or SAST finding. |

## 4. Formal Comparison: Autonomous LLM Spawning vs. Declarative Scatter-Gather

| Architectural Property | Autonomous LLM Subagent Spawning (spawn_agent tool inside Agent) | Declarative Scatter-Gather via Oengine​ (Scatter → Map → Reduce on G) |
| --- | --- | --- |
| Who Orchestrates the Subagents? | The parent LLM agent via multi-turn prompt reasoning | Deterministic statechart engine Oengine​ |
| Where Do Subagent Outputs Go? | Returned into the parent LLM’s context window | Written directly to partitioned slices G[shardk​] |
| Parent Context Scaling | O(K×output_size) — overflows on large repos | O(1) per Map worker; O(K×boundary_summary) at Reduce |
| Max Recursion Depth | Unbounded (risk of runaway agent trees) | Strictly 1 (Macro-Phase → Shard Worker) |
| Failure Recovery | Fragile — parent LLM loses state if interrupted | Atomic per shard — Oengine​ retries only failed shardk​ |
| Harel Statechart Formalism | Opaque imperative box (invisible to statechart) | Parameterized Orthogonal AND-State \bigparallelk=1K​W(shardk​) |

## Interactive Architecture Comparator: Autonomous LLM Spawning vs. Declarative Scatter-Gather

Use the interactive diagram below to compare what happens inside a non-ARIADNE phase (such as **S3S_3S3​: Extract Business Rules**) when using **Autonomous LLM Subagent Spawning** versus **Oengine\mathcal{O}_{\text{engine}}Oengine​-Orchestrated Declarative Scatter-Gather**, including how cross-shard links and shard-level Semantic Critics ($g_{\text{sem}}$) execute without overflowing context:

::END_TURN::


::USER_MESSAGE::
Lets go with pattern B. Methodology is now finalized.
Export it as a formalization-algorithm(s) of the methodology, like the methodology section of a latex paper. It should contain justifications & decisions, not just mere descriptions.

::MODEL_RESPONSE::
## 3. Methodology: The THESEUS Framework

We present **THESEUS** (*Two-Tier Harel-Statechart Execution with Scoped Exploratory Unit Solvers*), a hierarchical multi-agent system (MAS) architecture for repository-scale code-to-code (`code2code`) modernization. Given a legacy repository $\mathcal{R}_{\text{leg}}$ in source language $\mathcal{L}_{\text{src}}$, a target language specification $\mathcal{L}_{\text{tgt}}$, and an execution harness $\mathcal{H}_{\text{exec}}$, the objective of THESEUS is to synthesize a modernized repository $\mathcal{R}_{\text{tgt}}$ that satisfies three formal criteria: (1) syntactic and type-level compilation across all inter-module boundaries, (2) behavioral parity with $\mathcal{R}_{\text{leg}}$ across all extracted domain invariants and characterization tests, and (3) adherence to target architectural idioms and static security constraints.

### 3.1 Architectural Design Decisions and Theoretical Justifications

Repository-scale modernization violates the assumptions of standard LLM multi-agent workflows because legacy codebases exhibit non-monotonic discovery: implementation faults uncovered late in module translation frequently invalidate earlier assumptions regarding interface contracts or dependency topologies. Every structural choice in THESEUS is designed to resolve a specific failure mode of existing paradigms:

1. Decoupling Deterministic Control Topology from Bounded Reasoning Context:
In manager-worker systems (e.g., GVS5H) and conversational hierarchies, an LLM orchestrator maintains workflow state in its prompt history and decides agent transitions via token generation. Over long horizons, this induces state hallucination and context window saturation, particularly in Small Language Models (SLMs).Design Decision: We partition the Macro-Orchestrator O=(Oengine,Oarbiter)\mathcal{O} = (\mathcal{O}_{\text{engine}}, \mathcal{O}_{\text{arbiter}})O=(Oengine​,Oarbiter​) into a deterministic statechart runtime engine (Oengine\mathcal{O}_{\text{engine}}Oengine​, implemented as programmatic control code) and an on-demand, stateless LLM Arbiter (Oarbiter\mathcal{O}_{\text{arbiter}}Oarbiter​) invoked exclusively for semantic ambiguity triage. All worker agents are stateless, ephemeral functions that never poll or maintain persistent sessions; they are instantiated on demand via push-based context hydration from Oengine\mathcal{O}_{\text{engine}}Oengine​ and terminate immediately upon committing artifacts.
2. Harel Statecharts over Flat FSMs and Static Pipelines:
Static feed-forward pipelines (e.g., ReCodeAgent) lack backward topological edges to recover when downstream validation invalidates upstream plans. Conversely, placing a flat Finite State Machine (FSM) at the root of an agent hierarchy forces lockstep global phase rollbacks: tracking ∣V∣\vert{}V\vert{}∣V∣ modules across mmm module states requires O(m∣V∣)O(m^{\vert{}V\vert{}})O(m∣V∣) composite states.Design Decision: We formalize control via a Harel Statechart H\mathcal{H}H combining XOR Macro-Superstates (for repository-wide phases), Orthogonal AND-Regions (reducing module state complexity from O(m∣V∣)O(m^{\vert{}V\vert{}})O(m∣V∣) to O(m⋅∣V∣)O(m \cdot \vert{}V\vert{})O(m⋅∣V∣)), Broadcast Event Calculus (for surgical O(1)O(1)O(1)-hop cross-module regressions), and Deep History (H∗H^*H∗) (preserving verified modules across macro-regressions).
3. Declarative Scatter-Gather (Dmax⁡=1D_{\max}=1Dmax​=1) over Autonomous Recursive Subagent Spawning:
Repository-wide discovery phases (inventory, rule mining, contract stubbing) exceed single-agent context limits and require parallel subagent fan-out. However, granting LLM agents an autonomous spawn_subagent tool inside a ReAct loop creates an O(K)O(K)O(K) context bottleneck when KKK subagents return outputs into the parent agent's message buffer, while creating uncheckpointed shadow state outside the blackboard.Design Decision: Non-ARIADNE agents are prohibited from imperative subagent spawning. Instead, parallelizable macro-phases are modeled as deterministic 3-stage sub-statecharts (Scatter →\to→ Parallel Map →\to→ Reduce) managed by Oengine\mathcal{O}_{\text{engine}}Oengine​ over partitioned blackboard slices, enforcing a strict recursion depth invariant of Dmax⁡=1D_{\max} = 1Dmax​=1.
4. Two-Stage Hybrid Phase Guards (g=gstruct∧gsemg = g_{\text{struct}} \land g_{\text{sem}}g=gstruct​∧gsem​):
Purely programmatic guards (schema validation, AST parsing, DAG acyclicity) are syntactically sound but semantically shallow—they cannot detect omitted business rules or over-engineered target interfaces. Conversely, evaluating every transition purely via an LLM critic wastes tokens on structurally malformed outputs.Design Decision: Every phase transition is gated by a short-circuiting conjunction of a zero-token Deterministic Structural Guard (gstructg_{\text{struct}}gstruct​) followed conditionally by a sharded, read-isolated Adversarial Semantic Guard (gsemg_{\text{sem}}gsem​) whose boolean verdict drives Oengine\mathcal{O}_{\text{engine}}Oengine​ deterministically.
5. Encapsulated Intra-Module MCTS Solvers (ARIADNE Sub-Holons):
Single-trajectory translators greedily patch a single code draft, frequently degenerating into local repair loops when the initial refactoring strategy is flawed. However, running Monte Carlo Tree Search (MCTS) globally across a repository causes an O(∣V∣×∣A∣)O(\vert{}V\vert{} \times \vert{}\mathcal{A}\vert{})O(∣V∣×∣A∣) action-space explosion.Design Decision: We encapsulate ARIADNE (a Softmax-UCB MCTS planner coupled with a 4-slice local blackboard and a 5-component specialist pool) as an isolated intra-module solver node (M2) instantiated per DAG vertex vi∈Vv_i \in Vvi​∈V. Furthermore, we elevate Business Rule Extraction to a pre-DAG global phase (S3S_3S3​) so cross-cutting domain invariants are locked before target interface design (S4S_4S4​).

### 3.2 Formal System Model: Two-Tier Blackboard and Harel Statechart

#### Definition 1 (Global Repository Blackboard G\mathcal{G}G and State Vector Σ(G)\Sigma(\mathcal{G})Σ(G))

The Global Blackboard $\mathcal{G}$ is a persistent, schema-validated tuple:

$$
\mathcal{G} = \Big\langle \mathcal{I}_{\text{inv}}, \; G=(V,E), \; \mathcal{R}_{\text{rules}}, \; \mathcal{C}_{\text{contracts}}, \; \mathcal{A}_{\text{artifacts}}, \; \Sigma(\mathcal{G}) \Big\rangle
$$

where:

- Iinv\mathcal{I}_{\text{inv}}Iinv​ is the repository file inventory, dialect profile, and verified build/test harness configuration.
- G=(V,E)G = (V, E)G=(V,E) is the Condensed Dependency Directed Acyclic Graph (DAG), where each vertex vi∈Vv_i \in Vvi​∈V represents an atomic translation unit (a single file/module or a collapsed Strongly Connected Component of mutually dependent files), and each directed edge epi=(vp,vi)∈Ee_{pi} = (v_p, v_i) \in Eepi​=(vp​,vi​)∈E denotes that downstream unit viv_ivi​ depends on symbols exported by upstream parent vpv_pvp​.
- Rrules:V→2RuleCard\mathcal{R}_{\text{rules}}: V \to 2^{\text{RuleCard}}Rrules​:V→2RuleCard maps each vertex to its verified Given/When/Then business rule cards with file:line legacy citations.
- Ccontracts:E→InterfaceSpec\mathcal{C}_{\text{contracts}}: E \to \text{InterfaceSpec}Ccontracts​:E→InterfaceSpec maps each dependency edge to its target-language type signature and behavioral preconditions.
- Aartifacts:V→(TargetAST×TestSuite)\mathcal{A}_{\text{artifacts}}: V \to (\text{TargetAST} \times \text{TestSuite})Aartifacts​:V→(TargetAST×TestSuite) stores hardened target modules.
- Σ(G)\Sigma(\mathcal{G})Σ(G) is the transactionally persisted state configuration:Σ(G)=⟨Smacro,  M=[M(v1),…,M(v∣V∣)],  ΘH∗,  Qevents⟩\Sigma(\mathcal{G}) = \Big\langle S_{\text{macro}}, \; \mathbf{M} = \big[M(v_1), \dots, M(v_{\vert{}V\vert{}})\big], \; \Theta_{H^*}, \; \mathcal{Q}_{\text{events}} \Big\rangleΣ(G)=⟨Smacro​,M=[M(v1​),…,M(v∣V∣​)],ΘH∗​,Qevents​⟩comprising the active macro-state SmacroS_{\text{macro}}Smacro​, the vector of per-module micro-states M\mathbf{M}M, the deep-history snapshot map ΘH∗\Theta_{H^*}ΘH∗​, and the FIFO asynchronous event bus Qevents\mathcal{Q}_{\text{events}}Qevents​.

#### Definition 2 (Per-Module 4-Slice Local Blackboard B(vi)\mathcal{B}^{(v_i)}B(vi​))

Each active module vertex $v_i \in V$ owns an isolated **Local Blackboard** $\mathcal{B}^{(v_i)}$ invisible to other vertices, partitioned into four typed slices following the ARIADNE formalism:

$$
\mathcal{B}^{(v_i)} = \Big( \mathcal{B}_{PM}^{(v_i)}, \; \mathcal{B}_{SH}^{(v_i)}, \; \mathcal{B}_{CT}^{(v_i)}, \; \mathcal{B}_{PR}^{(v_i)} \Big)
$$

1. Problem Model (BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​): Hydrated deterministically from G\mathcal{G}G upon module activation with legacy AST slices, verified rule cards Rrules(vi)\mathcal{R}_{\text{rules}}(v_i)Rrules​(vi​), upstream parent contracts {Ccontracts(vp,vi)}\{\mathcal{C}_{\text{contracts}}(v_p, v_i)\}{Ccontracts​(vp​,vi​)}, and downstream export obligations.
2. Strategy Hypotheses (BSH(vi)\mathcal{B}_{SH}^{(v_i)}BSH(vi​)​): Competing architectural translation strategies hkh_khk​ with prior utility estimates and instantiation metadata.
3. Counterexamples & Characterization Tests (BCT(vi)\mathcal{B}_{CT}^{(v_i)}BCT(vi​)​): Black-box I/O snapshots, property-based test suites, and falsifying inputs derived from legacy behavior.
4. Patch & Fault Records (BPR(vi)\mathcal{B}_{PR}^{(v_i)}BPR(vi​)​): Distilled compiler/test fault localizations and root-cause traces indexing failed MCTS rollouts.

#### Definition 3 (Harel Statechart Formalism H\mathcal{H}H)

Control flow in $\mathcal{O}_{\text{engine}}$ is governed by the Harel Statechart tuple:

$$
\mathcal{H} = \Big( \mathcal{S}, \; \rho, \; \psi, \; \mathcal{E}_{\text{bus}}, \; \mathcal{G}_{\text{guards}}, \; \mathcal{T}_{\text{trans}}, \; \Theta_{H^*} \Big)
$$

- Hierarchy Function (ρ:S→2S\rho: \mathcal{S} \to 2^{\mathcal{S}}ρ:S→2S): Defines state nesting. The top-level root state decomposes into seven sequential XOR Macro-States ρ(Root)={S0,S1,S2,S3,S4,S5,S6}\rho(\text{Root}) = \{S_0, S_1, S_2, S_3, S_4, S_5, S_6\}ρ(Root)={S0​,S1​,S2​,S3​,S4​,S5​,S6​}.
- Orthogonality Function (ψ:S→{XOR,AND}\psi: \mathcal{S} \to \{\texttt{XOR}, \texttt{AND}\}ψ:S→{XOR,AND}): Specifies state concurrency:For parallelizable discovery states Sm∈{S1,S2,S3,S4,S6}S_m \in \{S_1, S_2, S_3, S_4, S_6\}Sm​∈{S1​,S2​,S3​,S4​,S6​}, the internal map sub-state Sm.2S_m.2Sm​.2 has ψ(Sm.2)=AND\psi(S_m.2) = \texttt{AND}ψ(Sm​.2)=AND over KKK stateless shard workers: Sm.2=\bigparallelk=1KW(shardk)S_m.2 = \bigparallel_{k=1}^K W(\text{shard}_k)Sm​.2=\bigparallelk=1K​W(shardk​).For the DAG Transformation superstate S5S_5S5​, ψ(S5)=AND\psi(S_5) = \texttt{AND}ψ(S5​)=AND over ∣V∣\vert{}V\vert{}∣V∣ concurrent module regions:S5=R(v1)  ∥  R(v2)  ∥  …  ∥  R(v∣V∣)S_5 = R(v_1) \;\parallel\; R(v_2) \;\parallel\; \dots \;\parallel\; R(v_{\vert{}V\vert{}})S5​=R(v1​)∥R(v2​)∥…∥R(v∣V∣​)where each orthogonal region R(vi)R(v_i)R(vi​) executes an isomorphic 6-state Micro-FSM ρ(R(vi))={M0,M1,M2,M3,M4,M5}\rho(R(v_i)) = \{\texttt{M0}, \texttt{M1}, \texttt{M2}, \texttt{M3}, \texttt{M4}, \texttt{M5}\}ρ(R(vi​))={M0,M1,M2,M3,M4,M5}.
- Guarded Broadcast Transition Relation (Ttrans\mathcal{T}_{\text{trans}}Ttrans​): Every transition t∈Ttranst \in \mathcal{T}_{\text{trans}}t∈Ttrans​ takes the Harel canonical form:ssrc→  ev  [g(G)]  /  α(G)  sdsts_{\text{src}} \xrightarrow{\; \text{ev} \; [g(\mathcal{G})] \; / \; \alpha(\mathcal{G}) \;} s_{\text{dst}}ssrc​ev[g(G)]/α(G)​sdst​where ev∈Ebus\text{ev} \in \mathcal{E}_{\text{bus}}ev∈Ebus​ is the triggering event popped from Qevents\mathcal{Q}_{\text{events}}Qevents​, g(G)∈{True,False}g(\mathcal{G}) \in \{\text{True}, \text{False}\}g(G)∈{True,False} is the guard predicate, and α(G)\alpha(\mathcal{G})α(G) is the transition action (which updates Σ(G)\Sigma(\mathcal{G})Σ(G), optionally broadcasts internal events to orthogonal regions, and invokes the ContextAssembler).
- Deep History Pseudostate (ΘH∗\Theta_{H^*}ΘH∗​): Upon any macro-regression exiting superstate S5S_5S5​ to an earlier state SmS_mSm​ (m∈{2,3,4}m \in \{2, 3, 4\}m∈{2,3,4}), Oengine\mathcal{O}_{\text{engine}}Oengine​ records ΘH∗(S5)←M\Theta_{H^*}(S_5) \leftarrow \mathbf{M}ΘH∗​(S5​)←M. Upon re-entering S5S_5S5​ via the H∗H^*H∗ connector, every orthogonal region R(vi)R(v_i)R(vi​) unaffected by the topological repair is restored directly to its prior state M(vi)M(v_i)M(vi​) (preserving M4_Hardened modules without re-execution).

#### Definition 4 (Dynamic Context Hydration via TaskActivationEnvelope)

Because $\mathcal{O}_{\text{engine}}$ is programmatic code while worker agents require rich task context, every transition action $\alpha(\mathcal{G})$ invokes a deterministic **ContextAssembler** function:

$$
\text{Hydrate}\big(\text{AgentRole}, \; \text{mode}, \; \text{target\_ref}, \; \mathcal{G}\big) \longrightarrow \mathcal{E}_{\text{act}} = \Big\langle \mu_{\text{mode}}, \; \mathcal{G}_{\text{slice}}, \; \Delta_{\text{directive}}, \; \mathcal{T}_{\text{tools}} \Big\rangle
$$

where $\mu_{\text{mode}} \in \{\texttt{INITIAL}, \texttt{CRITIC\_RETRY}, \texttt{WARM\_START\_REGRESSION}\}$ selects the system prompt template, $\mathcal{G}_{\text{slice}}$ queries only the bounded 1-hop subgraph relevant to `target_ref`, $\Delta_{\text{directive}}$ injects the exact failure critique or contract conflict trace, and $\mathcal{T}_{\text{tools}}$ binds read-only deterministic AST/LSP navigation tools.

### 3.3 Macro-Level Orchestration: The 7-Phase Pipeline & Declarative Scatter-Gather

Aligning with the domain lifecycle of the Claude Code Modernization workflow, $\mathcal{O}_{\text{engine}}$ enforces five global discovery and specification phases ($S_0 \to S_4$) prior to DAG transformation ($S_5$) and final hardening ($S_6$).

#### 1. Declarative Scatter-Gather Sub-Statechart (Sm.1→Sm.2→Sm.3S_m.1 \to S_m.2 \to S_m.3Sm​.1→Sm​.2→Sm​.3)

For any macro-state $S_m \in \{S_1, S_2, S_3, S_4, S_6\}$ operating over a large repository, execution proceeds through three internal substates:

1. Sm.1S_m.1Sm​.1 (Scatter / Partitioning): A deterministic partitioner (using AST/directory metrics or DAG vertex batching) divides the phase workload into KKK bounded shards {shard1,…,shardK}\{\text{shard}_1, \dots, \text{shard}_K\}{shard1​,…,shardK​} such that each shard satisfies the SLM context constraint Tokens(shardk)≤Lmax⁡\text{Tokens}(\text{shard}_k) \le L_{\max}Tokens(shardk​)≤Lmax​, writing a WorkManifest to G\mathcal{G}G.
2. Sm.2S_m.2Sm​.2 (Parallel Map over Orthogonal Shards): Oengine\mathcal{O}_{\text{engine}}Oengine​ spawns KKK parallel, stateless Map subagents W1,…,WKW_1, \dots, W_KW1​,…,WK​. Each worker WkW_kWk​ reads only G[shardk]\mathcal{G}[\text{shard}_k]G[shardk​] (augmented with read-only deterministic LSP symbol lookup tools for out-of-shard definitions). If WkW_kWk​ encounters a complex cross-file dependency, it emits a declarative tag UnresolvedCrossLink(src_id, tgt_symbol) rather than spawning a child agent. Each WkW_kWk​ writes its output directly to G[shardk]\mathcal{G}[\text{shard}_k]G[shardk​], pushes ev_ShardDone(k), and terminates.
3. Sm.3S_m.3Sm​.3 (Reduce / Cross-Shard Linker): Once all KKK shards complete, Oengine\mathcal{O}_{\text{engine}}Oengine​ invokes a lightweight Linker agent (or deterministic graph reducer) that queries only the exported boundary headers and UnresolvedCrossLink tags across G[shard1…K]\mathcal{G}[\text{shard}_{1\dots K}]G[shard1…K​] to synthesize unified cross-module links.

#### 2. Two-Stage Guard Verification Pipeline (g=gstruct∧gsemg = g_{\text{struct}} \land g_{\text{sem}}g=gstruct​∧gsem​)

Before transitioning out of phase $S_m$, $\mathcal{O}_{\text{engine}}$ evaluates:

$$
g(\mathcal{G}) = \underbrace{\Big( \bigwedge_{k=1}^K g_{\text{struct}}\big(\mathcal{G}[\text{shard}_k]\big) \Big)}_{\text{Stage 1: Zero-Token Programmatic Check}} \;\land\; \underbrace{\Big( \bigwedge_{b=1}^{B} g_{\text{sem}}\big(\mathcal{G}[\text{batch}_b]\big) \Big)}_{\text{Stage 2: Sharded Adversarial LLM Critic}}
$$

If Stage 1 ($g_{\text{struct}}$) fails on $\text{shard}_k$, $\mathcal{O}_{\text{engine}}$ immediately re-dispatches only $W_k$ with `ActivationMode = CRITIC_RETRY` and the deterministic validator traceback. Only when Stage 1 passes across all shards does $\mathcal{O}_{\text{engine}}$ fan out Stage 2: parallel, read-isolated **Semantic Critic** subagents (`ShardCritic`) that audit batches of produced artifacts against legacy `file:line` source citations and output a structured `CritiqueRecord`.

#### 3. Specification of the 7 Macro-States (S0…S6S_0 \dots S_6S0​…S6​)

| State | Phase Name | Scatter-Gather Topology (Sm​.1→Sm​.3) | Structural Guard (gstruct​) | Adversarial Semantic Guard (gsem​) |
| --- | --- | --- | --- | --- |
| S0​ | Intent & Preflight | Single PreflightProbe agent (no sharding). | Legacy build compiles; baseline test harness executes; INTENT schema valid. | N/A (Purely empirical execution gate). |
| S1​ | Assess | Scatter: Directory shards.Map: ShardAssessor (dialect, debt).Reduce: AssessmentSynthesizer. | 100% file inventory coverage against scc/cloc; valid metric ranges. | AssessmentCritic verifies every risk claim cites valid legacy files and flags unmapped dialects. |
| S2​ | Map (DAG Construction) | Scatter: Static LSP/Tree-sitter builds explicit graph & flags dynamic files.Map: ImplicitCouplingScanner on flagged files.Reduce: Tarjan’s SCC algorithm condenses cycles into DAG G=(V,E). | G=(V,E) is strictly acyclic; all repository entry points and files belong to exactly one vi​∈V. | TopologyCritic audits flagged reflection, shared DB tables, and event buses for missed implicit edges. |
| S3​ | Extract Business Rules | Scatter: Vertex batches from V.Map: RuleMiner extracts Given/When/Then cards.Reduce: CrossModuleRuleLinker resolves cross-vertex rules. | Every rule card contains valid Given/When/Then clauses, legacy/file:line pointers, and Observed vs. Inferred tags. | RuleAuditor (independent agent) re-verifies every rule card against legacy AST snippets for completeness and hallucination. |
| S4​ | Brief & Reimagine Contracts | Scatter: MacroArchitect sets global idioms; edges E sharded by domain.Map: ContractStubber writes interface stubs.Reduce: Target compiler checks stub compatibility. | Every edge epi​∈E has a target interface stub in Ccontracts​ that compiles cleanly as a header/stub. | ArchitectureCritic audits target interfaces against Rrules​ to prevent over-engineering or unrepresentable domain invariants. |
| S5​ | Iterative DAG Transform | Composite Orthogonal AND-State: Topological Wave Scheduler over R(v1​)∥⋯∥R(v∣V∣​) running ARIADNE Clusters. | ∀vi​∈V,M(vi​)=M4_Hardened with composite reward R(ci⋆​)=1.0. | Enforced intra-cluster via Aevaldistill​ rule-parity verification prior to entering M4. |
| S6​ | System Verify & Harden | Scatter: E2E test suites & SAST scanners.Map: IntegrationVerifier & SecurityAuditor.Reduce: Final parity sign-off generator. | 100% E2E integration test pass rate; zero critical SAST/CVE vulnerabilities. | ReleaseAuditor verifies end-to-end compliance with INTENT and signs off per-module verdicts. |

### 3.4 Topological Wave Scheduling & Encapsulated ARIADNE MCTS Nodes

Upon entering Composite State $S_5$, $\mathcal{O}_{\text{engine}}$ activates its **Topological Wave Scheduler**, a deterministic subroutine based on Kahn’s algorithm that continuously evaluates the set of unblocked vertices:

$$
\text{ReadySet}(\mathcal{G}) = \Big\{ v_i \in V \;\Big\vert{}\; M(v_i) = \texttt{M0\_Blocked} \;\land\; \forall (v_p, v_i) \in E, \; M(v_p) = \texttt{M4\_Hardened} \Big\}
$$

#### 1. Per-Module Micro-FSM (Mvi\mathcal{M}_{v_i}Mvi​​)

Each orthogonal region $R(v_i)$ transitions through six micro-states:

- M0_Blocked: Waiting for upstream parents {vp∣(vp,vi)∈E}\{v_p \mid (v_p, v_i) \in E\}{vp​∣(vp​,vi​)∈E} to reach M4_Hardened.
- M1_ContextHydration (Deterministic, Non-LLM): Triggered when vi∈ReadySet(G)v_i \in \text{ReadySet}(\mathcal{G})vi​∈ReadySet(G). Deterministically copies Rrules(vi)\mathcal{R}_{\text{rules}}(v_i)Rrules​(vi​) and parent contracts {Ccontracts(vp,vi)}\{\mathcal{C}_{\text{contracts}}(v_p, v_i)\}{Ccontracts​(vp​,vi​)} from G\mathcal{G}G into the Problem Model slice BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​ and transitions immediately to M2.
- M2_ARIADNE_MCTS (Active Transformation & Verification Node): Executes the encapsulated ARIADNE cluster. Transitions to M4_Hardened if a candidate c⋆c^\starc⋆ achieves R(c⋆)=1.0R(c^\star) = 1.0R(c⋆)=1.0; transitions to M3_ContractNegotiate if an extrinsic upstream contract conflict is proved; or emits a Tier-C macro-regression if an undiscovered dependency cycle is found.
- M3_ContractNegotiate: Emits broadcast event ev_ContractConflict(v_p, v_i, delta_spec, proof_trace) to Qevents\mathcal{Q}_{\text{events}}Qevents​ on G\mathcal{G}G and parks viv_ivi​ back in M0_Blocked.
- M4_Hardened: Commits verified target code c⋆c^\starc⋆ and characterization test suite BCT(vi)\mathcal{B}_{CT}^{(v_i)}BCT(vi​)​ to Aartifacts(vi)\mathcal{A}_{\text{artifacts}}(v_i)Aartifacts​(vi​) on G\mathcal{G}G and emits ev_ModuleHardened(v_i), triggering Oengine\mathcal{O}_{\text{engine}}Oengine​ to recompute ReadySet(G)\text{ReadySet}(\mathcal{G})ReadySet(G).
- M5_Invalidated (Warm-Start Regression State): Entered when a M4_Hardened module receives a validated ev_ContractConflict from a downstream child (or when its own parent updates an interface). Deterministically appends the delta constraint Δspec\Delta_{\text{spec}}Δspec​ to BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​ while preserving the existing MCTS search tree T(vi)\mathcal{T}^{(v_i)}T(vi​), strategy hypotheses BSH(vi)\mathcal{B}_{SH}^{(v_i)}BSH(vi​)​, and test suite BCT(vi)\mathcal{B}_{CT}^{(v_i)}BCT(vi​)​, then transitions back to M2_ARIADNE_MCTS.

#### 2. Internal Architecture of the Encapsulated ARIADNE Cluster (M2)

Inside state `M2`, module $v_i$ is solved by an isolated ARIADNE cluster consisting of a **Deterministic MCTS Controller (CMCTS(vi)\mathcal{C}_{\text{MCTS}}^{(v_i)}CMCTS(vi​)​)**, **4 Generative LLM Subagents**, and a **Two-Stage Hybrid Evaluator (Aeval\mathcal{A}_{\text{eval}}Aeval​)** operating over $\mathcal{B}^{(v_i)}$:

1. Deterministic MCTS Controller (CMCTS(vi)\mathcal{C}_{\text{MCTS}}^{(v_i)}CMCTS(vi​)​ — Algorithmic Code):
Maintains a search tree T(vi)\mathcal{T}^{(v_i)}T(vi​) where each node uuu represents a state st=(ct,Bt(vi))s_t = (c_t, \mathcal{B}_t^{(v_i)})st​=(ct​,Bt(vi​)​). At each rollout t≤Bmax⁡t \le B_{\max}t≤Bmax​, CMCTS(vi)\mathcal{C}_{\text{MCTS}}^{(v_i)}CMCTS(vi​)​ traverses from the root s0s_0s0​ by computing Upper Confidence Bound (UCB) scores over child states u∈Children(v)u \in \text{Children}(v)u∈Children(v):UCB(u)=Rˉ(u)+Cexpln⁡(N(v)+1)N(u)+γ⋅Pprior(u)\text{UCB}(u) = \bar{R}(u) + C_{\text{exp}} \sqrt{\frac{\ln(N(v) + 1)}{N(u)}} + \gamma \cdot P_{\text{prior}}(u)UCB(u)=Rˉ(u)+Cexp​N(u)ln(N(v)+1)​​+γ⋅Pprior​(u)where Rˉ(u)\bar{R}(u)Rˉ(u) is the mean backpropagated reward, N(⋅)N(\cdot)N(⋅) denotes visit counts, and Pprior(u)P_{\text{prior}}(u)Pprior​(u) is the strategy utility prior stored in BSH(vi)\mathcal{B}_{SH}^{(v_i)}BSH(vi​)​. To avoid premature greedy convergence on flawed refactoring templates, child selection follows a temperature-controlled softmax distribution:p(u∣v)=exp⁡((UCB(u)−max⁡wUCB(w))/τ)∑w∈Children(v)exp⁡((UCB(w)−max⁡w′UCB(w′))/τ)p(u \mid v) = \frac{\exp\big((\text{UCB}(u) - \max_{w} \text{UCB}(w)) / \tau\big)}{\sum_{w \in \text{Children}(v)} \exp\big((\text{UCB}(w) - \max_{w'} \text{UCB}(w')) / \tau\big)}p(u∣v)=∑w∈Children(v)​exp((UCB(w)−maxw′​UCB(w′))/τ)exp((UCB(u)−maxw​UCB(w))/τ)​
2. State-Conditioned Dispatch of the 4 Generative LLM Subagents:
Once node u=(ct,Bt(vi))u = (c_t, \mathcal{B}_t^{(v_i)})u=(ct​,Bt(vi​)​) is selected, CMCTS(vi)\mathcal{C}_{\text{MCTS}}^{(v_i)}CMCTS(vi​)​ inspects deterministic state flags on uuu to invoke one of four ephemeral LLM agents:Astr\mathcal{A}_{\text{str}}Astr​ (Strategy Analysis Agent): Invoked when uuu lacks unexpanded strategy branches; reads BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​ and writes KsK_sKs​ distinct refactoring hypotheses (e.g., structural port vs. state-pattern decomposition) with prior scores PpriorP_{\text{prior}}Pprior​ to BSH(vi)\mathcal{B}_{SH}^{(v_i)}BSH(vi​)​.Agen\mathcal{A}_{\text{gen}}Agen​ (Code Generation Agent): Invoked on an uninstantiated strategy h∈BSH(vi)h \in \mathcal{B}_{SH}^{(v_i)}h∈BSH(vi​)​; synthesizes candidate target code c′c'c′ conforming to BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​.Atest\mathcal{A}_{\text{test}}Atest​ (Adversarial Test Generation Agent): Reads BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​ and legacy behavior (strictly read-isolated from candidate c′c'c′ to prevent test weakening) to synthesize characterization I/O snapshots and property-based tests into BCT(vi)\mathcal{B}_{CT}^{(v_i)}BCT(vi​)​.Apatch\mathcal{A}_{\text{patch}}Apatch​ (Surgical Code Repair Agent): Invoked when ctc_tct​ has failed verification (0 < R(s_t) < 1); reads ctc_tct​ and the localized fault record in BPR(vi)\mathcal{B}_{PR}^{(v_i)}BPR(vi​)​ to emit a minimal AST/diff patch c′′c''c′′.
3. Two-Stage Hybrid Evaluator (Aeval=Aevalexec∘Aevaldistill\mathcal{A}_{\text{eval}} = \mathcal{A}_{\text{eval}}^{\text{exec}} \circ \mathcal{A}_{\text{eval}}^{\text{distill}}Aeval​=Aevalexec​∘Aevaldistill​):
Every generated or patched candidate c′c'c′ is evaluated in two sequential stages:Stage 3a — Deterministic Execution Harness (Aevalexec\mathcal{A}_{\text{eval}}^{\text{exec}}Aevalexec​, Non-LLM): Compiles c′c'c′ against upstream target artifacts in G\mathcal{G}G, runs LSP type-checking, executes the characterization test suite BCT(vi)\mathcal{B}_{CT}^{(v_i)}BCT(vi​)​ via differential test runners, and runs static security/lint analyzers.Stage 3b — Diagnostic Distiller & Parity Critic (Aevaldistill\mathcal{A}_{\text{eval}}^{\text{distill}}Aevaldistill​, Lightweight LLM):On Stage 3a Failure: Distills raw compiler/test stack traces into a concise, localized fault card in BPR(vi)\mathcal{B}_{PR}^{(v_i)}BPR(vi​)​ and classifies the fault cause κ∈{INTRINSIC,  EXTRINSIC_CONTRACT,  EXTRINSIC_CYCLE}\kappa \in \{\texttt{INTRINSIC}, \; \texttt{EXTRINSIC\_CONTRACT}, \; \texttt{EXTRINSIC\_CYCLE}\}κ∈{INTRINSIC,EXTRINSIC_CONTRACT,EXTRINSIC_CYCLE}.On Stage 3a Pass: Audits c′c'c′ against all Given/When/Then business rule cards in BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​ to compute the semantic parity & hardening score Scoreharden(c′)∈[0,1]\text{Score}_{\text{harden}}(c') \in [0, 1]Scoreharden​(c′)∈[0,1].
4. Gated Hierarchical Reward Function R(st)R(s_t)R(st​):
The scalar reward backpropagated up T(vi)\mathcal{T}^{(v_i)}T(vi​) is strictly gated so uncompilable code receives zero credit from partial heuristics:R(st)=wcomp⋅Icompile(ct)+Icompile(ct)⋅[wchar⋅PassRatechar(ct,BCT(vi))+whard⋅Scoreharden(ct,BPM(vi))]R(s_t) = w_{\text{comp}} \cdot \mathbb{I}_{\text{compile}}(c_t) + \mathbb{I}_{\text{compile}}(c_t) \cdot \Big[ w_{\text{char}} \cdot \text{PassRate}_{\text{char}}\big(c_t, \mathcal{B}_{CT}^{(v_i)}\big) + w_{\text{hard}} \cdot \text{Score}_{\text{harden}}\big(c_t, \mathcal{B}_{PM}^{(v_i)}\big) \Big]R(st​)=wcomp​⋅Icompile​(ct​)+Icompile​(ct​)⋅[wchar​⋅PassRatechar​(ct​,BCT(vi​)​)+whard​⋅Scoreharden​(ct​,BPM(vi​)​)]where wcomp+wchar+whard=1.0w_{\text{comp}} + w_{\text{char}} + w_{\text{hard}} = 1.0wcomp​+wchar​+whard​=1.0 (default 0.2,0.6,0.20.2, 0.6, 0.20.2,0.6,0.2). Because Icompile∈{0,1}\mathbb{I}_{\text{compile}} \in \{0,1\}Icompile​∈{0,1}, PassRatechar∈[0,1]\text{PassRate}_{\text{char}} \in [0,1]PassRatechar​∈[0,1], and Scoreharden∈[0,1]\text{Score}_{\text{harden}} \in [0,1]Scoreharden​∈[0,1], achieving R(st)=1.0R(s_t) = 1.0R(st​)=1.0 is a necessary and sufficient condition for full compilation, 100% characterization test parity, and full semantic rule adherence.

### 3.5 Multi-Tier Regression Calculus and Warm-Start Recovery

THESEUS classifies and resolves discovered anomalies across three strict locality tiers, ensuring that errors are contained to the smallest possible scope:

1. Tier A — Intra-Module MCTS Backtrack (κ=INTRINSIC\kappa = \texttt{INTRINSIC}κ=INTRINSIC, Zero Global Traffic):
When R(st)<1.0R(s_t) \lt 1.0R(st​)<1.0 and Aevaldistill\mathcal{A}_{\text{eval}}^{\text{distill}}Aevaldistill​ classifies the fault as INTRINSIC\texttt{INTRINSIC}INTRINSIC to module viv_ivi​, recovery is handled entirely inside CMCTS(vi)\mathcal{C}_{\text{MCTS}}^{(v_i)}CMCTS(vi​)​: the reward R(st)R(s_t)R(st​) is backpropagated up T(vi)\mathcal{T}^{(v_i)}T(vi​), lowering Rˉ(u)\bar{R}(u)Rˉ(u) and shifting subsequent Softmax-UCB rollouts either to invoke Apatch\mathcal{A}_{\text{patch}}Apatch​ using BPR(vi)\mathcal{B}_{PR}^{(v_i)}BPR(vi​)​ or to branch to an alternative refactoring strategy in BSH(vi)\mathcal{B}_{SH}^{(v_i)}BSH(vi​)​. Macro-Orchestrator O\mathcal{O}O is never interrupted.
2. Tier B — Cross-Module Micro-Regression via Broadcast Calculus (κ=EXTRINSIC_CONTRACT\kappa = \texttt{EXTRINSIC\_CONTRACT}κ=EXTRINSIC_CONTRACT):
When downstream module viv_ivi​ proves that an upstream parent vpv_pvp​’s exported contract Ccontracts(vp,vi)\mathcal{C}_{\text{contracts}}(v_p, v_i)Ccontracts​(vp​,vi​) is insufficient (or when viv_ivi​ exhausts Bmax⁡B_{\max}Bmax​ rollouts blocked on epie_{pi}epi​), R(vi)R(v_i)R(vi​) transitions M2→M3\texttt{M2} \to \texttt{M3}M2→M3 and broadcasts ev_ContractConflict(v_p, v_i, delta_spec, proof_trace) to Qevents\mathcal{Q}_{\text{events}}Qevents​.Verification by O\mathcal{O}O: Oengine\mathcal{O}_{\text{engine}}Oengine​ runs a structural check confirming the symbol reference across epie_{pi}epi​, and invokes Oarbiter\mathcal{O}_{\text{arbiter}}Oarbiter​ to validate that delta_spec is a legitimate contract requirement rather than a hallucinated excuse by viv_ivi​.Surgical Orthogonal Transition: Upon validation, Oengine\mathcal{O}_{\text{engine}}Oengine​ updates Ccontracts(vp,vi)\mathcal{C}_{\text{contracts}}(v_p, v_i)Ccontracts​(vp​,vi​) on G\mathcal{G}G, parks M(vi)←M0_BlockedM(v_i) \leftarrow \texttt{M0\_Blocked}M(vi​)←M0_Blocked, and fires the broadcast transition on orthogonal region R(vp)R(v_p)R(vp​):M4_Hardened→  ev_ContractConflict(vp,vi,Δspec)  [ArbiterValid]  /  BPM(vp)←BPM(vp)∪{Δspec}  M5_Invalidated⟶M2_ARIADNE_MCTS\texttt{M4\_Hardened} \xrightarrow{\; \text{ev\_ContractConflict}(v_p, v_i, \Delta_{\text{spec}}) \; [\text{ArbiterValid}] \; / \; \mathcal{B}_{PM}^{(v_p)} \leftarrow \mathcal{B}_{PM}^{(v_p)} \cup \{\Delta_{\text{spec}}\} \;} \texttt{M5\_Invalidated} \longrightarrow \texttt{M2\_ARIADNE\_MCTS}M4_Hardenedev_ContractConflict(vp​,vi​,Δspec​)[ArbiterValid]/BPM(vp​)​←BPM(vp​)​∪{Δspec​}​M5_Invalidated⟶M2_ARIADNE_MCTSWarm-Start Efficiency: Because BSH(vp)\mathcal{B}_{SH}^{(v_p)}BSH(vp​)​, BCT(vp)\mathcal{B}_{CT}^{(v_p)}BCT(vp​)​, and the prior winning node cp⋆∈T(vp)c_p^\star \in \mathcal{T}^{(v_p)}cp⋆​∈T(vp​) are retained in memory, R(vp)R(v_p)R(vp​) initializes a child repair branch directly from cp⋆c_p^\starcp⋆​ under the updated BPM(vp)\mathcal{B}_{PM}^{(v_p)}BPM(vp​)​, adapting its interface in a few Apatch\mathcal{A}_{\text{patch}}Apatch​ rollouts without re-discovering vpv_pvp​’s core logic.
3. Tier C — Macro-Architectural Regression with Deep History H∗H^*H∗ (κ=EXTRINSIC_CYCLE\kappa = \texttt{EXTRINSIC\_CYCLE}κ=EXTRINSIC_CYCLE):
When module viv_ivi​ discovers an undocumented dependency cycle (vi→⋯→vp→vi)(v_i \to \dots \to v_p \to v_i)(vi​→⋯→vp​→vi​) via runtime reflection or dynamic loading, it emits ev_CyclicDependencyFound(cycle_edges, proof_trace) to Qevents\mathcal{Q}_{\text{events}}Qevents​.Action by Oengine\mathcal{O}_{\text{engine}}Oengine​: Oengine\mathcal{O}_{\text{engine}}Oengine​ checkpoints the current module state vector ΘH∗(S5)←M\Theta_{H^*}(S_5) \leftarrow \mathbf{M}ΘH∗​(S5​)←M, pauses only the active regions belonging to cycle_edges, and transitions the macro-state S5→S2S_5 \to S_2S5​→S2​ (Map).SCC Condensation & H∗H^*H∗ Restoration: In S2S_2S2​, Tarjan’s algorithm collapses the cyclic vertices into a single composite Strongly Connected Component vertex vSCC∈Vv_{\text{SCC}} \in VvSCC​∈V, followed by delta rule linking (S3S_3S3​) and boundary stubbing (S4S_4S4​) strictly for vSCCv_{\text{SCC}}vSCC​. Finally, Oengine\mathcal{O}_{\text{engine}}Oengine​ transitions S4→H∗(S5)S_4 \to H^*(S_5)S4​→H∗(S5​): all unaffected vertices vk∉vSCCv_k \notin v_{\text{SCC}}vk​∈/vSCC​ are immediately restored to their checkpointed states ΘH∗(S5)[vk]\Theta_{H^*}(S_5)[v_k]ΘH∗​(S5​)[vk​] (preserving all M4_Hardened modules), while vSCCv_{\text{SCC}}vSCC​ enters the wave scheduler as a unified translation unit.

### 3.6 Formal Execution Algorithms

Below are the two core algorithms governing the deterministic Macro-Orchestrator ($\mathcal{O}_{\text{engine}}$) and the Encapsulated ARIADNE Cluster (`M2`).

#### Algorithm 1: Deterministic Macro-Statechart Engine (Oengine\mathcal{O}_{\text{engine}}Oengine​) & Declarative Scatter-Gather

- Input: Legacy Repository Rleg\mathcal{R}_{\text{leg}}Rleg​, Target Spec Ltgt\mathcal{L}_{\text{tgt}}Ltgt​, Execution Harness Hexec\mathcal{H}_{\text{exec}}Hexec​
- Output: Hardened Target Repository Rtgt\mathcal{R}_{\text{tgt}}Rtgt​ stored in Aartifacts\mathcal{A}_{\text{artifacts}}Aartifacts​ on G\mathcal{G}G

1. Initialize Global Blackboard G←∅\mathcal{G} \leftarrow \emptysetG←∅, State Vector Σ(G)←⟨Smacro=S0,  M=[],  ΘH∗=∅,  Qevents=[ev_Start]⟩\Sigma(\mathcal{G}) \leftarrow \langle S_{\text{macro}} = S_0, \; \mathbf{M} = [], \; \Theta_{H^*} = \emptyset, \; \mathcal{Q}_{\text{events}} = [\texttt{ev\_Start}] \rangleΣ(G)←⟨Smacro​=S0​,M=[],ΘH∗​=∅,Qevents​=[ev_Start]⟩.
2. While Smacro≠TERMINATEDS_{\text{macro}} \neq \texttt{TERMINATED}Smacro​=TERMINATED do:If Smacro∈{S0,S1,S2,S3,S4,S6}S_{\text{macro}} \in \{S_0, S_1, S_2, S_3, S_4, S_6\}Smacro​∈{S0​,S1​,S2​,S3​,S4​,S6​} (Global Discovery, Specification & Verification Phases) then:Substate Sm.1S_m.1Sm​.1 (Scatter): Partition phase workload into bounded shards {shard1,…,shardK}\{\text{shard}_1, \dots, \text{shard}_K\}{shard1​,…,shardK​} with Tokens(shardk)≤Lmax⁡\text{Tokens}(\text{shard}_k) \le L_{\max}Tokens(shardk​)≤Lmax​ and write WorkManifest to G\mathcal{G}G.Substate Sm.2S_m.2Sm​.2 (Parallel Map): For each k∈{1,…,K}k \in \{1, \dots, K\}k∈{1,…,K} in parallel, construct envelope Eact(k)←Hydrate(MapAgent(Smacro),INITIAL,shardk,G)\mathcal{E}_{\text{act}}^{(k)} \leftarrow \text{Hydrate}(\text{MapAgent}(S_{\text{macro}}), \texttt{INITIAL}, \text{shard}_k, \mathcal{G})Eact(k)​←Hydrate(MapAgent(Smacro​),INITIAL,shardk​,G) and invoke stateless worker Wk(Eact(k))W_k(\mathcal{E}_{\text{act}}^{(k)})Wk​(Eact(k)​), which writes output directly to G[shardk]\mathcal{G}[\text{shard}_k]G[shardk​] and terminates.Substate Sm.3S_m.3Sm​.3 (Reduce): Invoke Cross-Shard Linker over boundary headers and UnresolvedCrossLink tags in G[shard1…K]\mathcal{G}[\text{shard}_{1\dots K}]G[shard1…K​].Stage-1 Structural Guard:For each k∈{1,…,K}k \in \{1, \dots, K\}k∈{1,…,K} where gstruct(G[shardk])=Falseg_{\text{struct}}(\mathcal{G}[\text{shard}_k]) = \text{False}gstruct​(G[shardk​])=False:
Re-hydrate Eact(k)\mathcal{E}_{\text{act}}^{(k)}Eact(k)​ with μmode=CRITIC_RETRY\mu_{\text{mode}} = \texttt{CRITIC\_RETRY}μmode​=CRITIC_RETRY and validator traceback; re-invoke WkW_kWk​ until gstructg_{\text{struct}}gstruct​ passes.Stage-2 Adversarial Semantic Guard:Fan out read-isolated ShardCritic subagents across artifact batches b∈{1,…,B}b \in \{1, \dots, B\}b∈{1,…,B}.If ∃b\exists b∃b such that gsem(G[batchb])=REJECT_LOCALg_{\text{sem}}(\mathcal{G}[\text{batch}_b]) = \texttt{REJECT\_LOCAL}gsem​(G[batchb​])=REJECT_LOCAL then:
Re-hydrate and re-invoke only the offending shard workers with DeltaDirective set to the Critic’s CritiqueRecord; continue loop.Else if ∃b\exists b∃b such that gsem(G[batchb])=REGRESS_UPSTREAMg_{\text{sem}}(\mathcal{G}[\text{batch}_b]) = \texttt{REGRESS\_UPSTREAM}gsem​(G[batchb​])=REGRESS_UPSTREAM then:
Transition SmacroS_{\text{macro}}Smacro​ to the target upstream macro-state StargetS_{\text{target}}Starget​; continue loop.Advance Phase: If Smacro=S4S_{\text{macro}} = S_4Smacro​=S4​, initialize M[vi]←ΘH∗(S5)[vi]\mathbf{M}[v_i] \leftarrow \Theta_{H^*}(S_5)[v_i]M[vi​]←ΘH∗​(S5​)[vi​] (if vi∈ΘH∗v_i \in \Theta_{H^*}vi​∈ΘH∗​) else M0_Blocked\texttt{M0\_Blocked}M0_Blocked for all vi∈Vv_i \in Vvi​∈V. Transition Smacro←NextState(Smacro)S_{\text{macro}} \leftarrow \text{NextState}(S_{\text{macro}})Smacro​←NextState(Smacro​).Else if Smacro=S5S_{\text{macro}} = S_5Smacro​=S5​ (Composite Orthogonal State: Iterative DAG Transformation) then:Topological Wave Dispatch: Compute ReadySet(G)={vi∈V∣M(vi)=M0_Blocked∧∀(vp,vi)∈E,M(vp)=M4_Hardened}\text{ReadySet}(\mathcal{G}) = \{v_i \in V \mid M(v_i) = \texttt{M0\_Blocked} \land \forall (v_p, v_i) \in E, M(v_p) = \texttt{M4\_Hardened}\}ReadySet(G)={vi​∈V∣M(vi​)=M0_Blocked∧∀(vp​,vi​)∈E,M(vp​)=M4_Hardened}.For each vi∈ReadySet(G)v_i \in \text{ReadySet}(\mathcal{G})vi​∈ReadySet(G) in parallel do:State M1: Deterministically copy Rrules(vi)\mathcal{R}_{\text{rules}}(v_i)Rrules​(vi​) and parent contracts {Ccontracts(vp,vi)}\{\mathcal{C}_{\text{contracts}}(v_p, v_i)\}{Ccontracts​(vp​,vi​)} from G\mathcal{G}G into BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​.State M2: Transition M(vi)←M2_ARIADNE_MCTSM(v_i) \leftarrow \texttt{M2\_ARIADNE\_MCTS}M(vi​)←M2_ARIADNE_MCTS and asynchronously execute Algorithm 2 (ARIADNE_Cluster(vi,B(vi),G)\text{ARIADNE\_Cluster}(v_i, \mathcal{B}^{(v_i)}, \mathcal{G})ARIADNE_Cluster(vi​,B(vi​),G)).Event Bus Processing: Pop next event ev←Qevents.pop()\text{ev} \leftarrow \mathcal{Q}_{\text{events}}.\text{pop}()ev←Qevents​.pop():Case ev=ev_ModuleHardened(vi,ci⋆,BCT(vi))\text{ev} = \texttt{ev\_ModuleHardened}(v_i, c_i^\star, \mathcal{B}_{CT}^{(v_i)})ev=ev_ModuleHardened(vi​,ci⋆​,BCT(vi​)​):
Commit (ci⋆,BCT(vi))(c_i^\star, \mathcal{B}_{CT}^{(v_i)})(ci⋆​,BCT(vi​)​) to Aartifacts(vi)\mathcal{A}_{\text{artifacts}}(v_i)Aartifacts​(vi​); set M(vi)←M4_HardenedM(v_i) \leftarrow \texttt{M4\_Hardened}M(vi​)←M4_Hardened. If ∀v∈V,M(v)=M4_Hardened\forall v \in V, M(v) = \texttt{M4\_Hardened}∀v∈V,M(v)=M4_Hardened, transition Smacro←S6S_{\text{macro}} \leftarrow S_6Smacro​←S6​.Case ev=ev_ContractConflict(vp,vi,Δspec,trace)\text{ev} = \texttt{ev\_ContractConflict}(v_p, v_i, \Delta_{\text{spec}}, \text{trace})ev=ev_ContractConflict(vp​,vi​,Δspec​,trace) (Tier B Micro-Regression):
If gstruct(trace)∧Oarbiter(Δspec,trace)=VALIDg_{\text{struct}}(\text{trace}) \land \mathcal{O}_{\text{arbiter}}(\Delta_{\text{spec}}, \text{trace}) = \texttt{VALID}gstruct​(trace)∧Oarbiter​(Δspec​,trace)=VALID then:
Update Ccontracts(vp,vi)←Ccontracts(vp,vi)⊕Δspec\mathcal{C}_{\text{contracts}}(v_p, v_i) \leftarrow \mathcal{C}_{\text{contracts}}(v_p, v_i) \oplus \Delta_{\text{spec}}Ccontracts​(vp​,vi​)←Ccontracts​(vp​,vi​)⊕Δspec​ on G\mathcal{G}G;
Set M(vi)←M0_BlockedM(v_i) \leftarrow \texttt{M0\_Blocked}M(vi​)←M0_Blocked;
Set M(vp)←M5_InvalidatedM(v_p) \leftarrow \texttt{M5\_Invalidated}M(vp​)←M5_Invalidated; append Δspec\Delta_{\text{spec}}Δspec​ to BPM(vp)\mathcal{B}_{PM}^{(v_p)}BPM(vp​)​ (preserving T(vp),BSH(vp),BCT(vp)\mathcal{T}^{(v_p)}, \mathcal{B}_{SH}^{(v_p)}, \mathcal{B}_{CT}^{(v_p)}T(vp​),BSH(vp​)​,BCT(vp​)​);
Transition M(vp)←M2_ARIADNE_MCTSM(v_p) \leftarrow \texttt{M2\_ARIADNE\_MCTS}M(vp​)←M2_ARIADNE_MCTS with μmode=WARM_START_REGRESSION\mu_{\text{mode}} = \texttt{WARM\_START\_REGRESSION}μmode​=WARM_START_REGRESSION.Case ev=ev_CyclicDependencyFound(Ecycle,trace)\text{ev} = \texttt{ev\_CyclicDependencyFound}(E_{\text{cycle}}, \text{trace})ev=ev_CyclicDependencyFound(Ecycle​,trace) (Tier C Macro-Regression):
If gstruct(Ecycle)∧Oarbiter(Ecycle,trace)=VALIDg_{\text{struct}}(E_{\text{cycle}}) \land \mathcal{O}_{\text{arbiter}}(E_{\text{cycle}}, \text{trace}) = \texttt{VALID}gstruct​(Ecycle​)∧Oarbiter​(Ecycle​,trace)=VALID then:
Save Deep History ΘH∗(S5)←M\Theta_{H^*}(S_5) \leftarrow \mathbf{M}ΘH∗​(S5​)←M; quiesce active regions in EcycleE_{\text{cycle}}Ecycle​;
Transition Smacro←S2S_{\text{macro}} \leftarrow S_2Smacro​←S2​ to condense EcycleE_{\text{cycle}}Ecycle​ into vSCCv_{\text{SCC}}vSCC​ via Tarjan's SCC and resume via H∗(S5)H^*(S_5)H∗(S5​).
3. Return Rtgt←Aartifacts(G)\mathcal{R}_{\text{tgt}} \leftarrow \mathcal{A}_{\text{artifacts}}(\mathcal{G})Rtgt​←Aartifacts​(G).

#### Algorithm 2: Encapsulated ARIADNE Module Cluster (M2) & Multi-Tier Regression Trigger

- Input: Vertex vi∈Vv_i \in Vvi​∈V, Local 4-Slice Blackboard B(vi)=(BPM(vi),BSH(vi),BCT(vi),BPR(vi))\mathcal{B}^{(v_i)} = (\mathcal{B}_{PM}^{(v_i)}, \mathcal{B}_{SH}^{(v_i)}, \mathcal{B}_{CT}^{(v_i)}, \mathcal{B}_{PR}^{(v_i)})B(vi​)=(BPM(vi​)​,BSH(vi​)​,BCT(vi​)​,BPR(vi​)​), Activation Mode μmode\mu_{\text{mode}}μmode​, Rollout Budget Bmax⁡B_{\max}Bmax​
- Output: Emits completion or regression interrupt event to Qevents\mathcal{Q}_{\text{events}}Qevents​ on Global Blackboard G\mathcal{G}G

1. Initialize MCTS Tree T(vi)\mathcal{T}^{(v_i)}T(vi​):If μmode=INITIAL\mu_{\text{mode}} = \texttt{INITIAL}μmode​=INITIAL then: Create root node u0=(c0=∅,B0(vi))u_0 = (c_0 = \emptyset, \mathcal{B}_0^{(v_i)})u0​=(c0​=∅,B0(vi​)​) and invoke Adversarial Test Agent Atest(BPM(vi))\mathcal{A}_{\text{test}}(\mathcal{B}_{PM}^{(v_i)})Atest​(BPM(vi​)​) to seed characterization test suite BCT(vi)\mathcal{B}_{CT}^{(v_i)}BCT(vi​)​.Else if μmode=WARM_START_REGRESSION\mu_{\text{mode}} = \texttt{WARM\_START\_REGRESSION}μmode​=WARM_START_REGRESSION then: Retain existing tree T(vi)\mathcal{T}^{(v_i)}T(vi​) and set active root pointer to previously verified node uwarm=(ci⋆,B(vi))u_{\text{warm}} = (c_i^\star, \mathcal{B}^{(v_i)})uwarm​=(ci⋆​,B(vi​)) updated with new BPM(vi)\mathcal{B}_{PM}^{(v_i)}BPM(vi​)​ contract constraints.
2. For rollout t=1t = 1t=1 to Bmax⁡B_{\max}Bmax​ do:Selection (Deterministic Softmax-UCB in CMCTS(vi)\mathcal{C}_{\text{MCTS}}^{(v_i)}CMCTS(vi​)​):
Starting from root, traverse T(vi)\mathcal{T}^{(v_i)}T(vi​) by sampling child u∼p(u∣v)∝exp⁡(UCB(u)/τ)u \sim p(u \mid v) \propto \exp\big(\text{UCB}(u)/\tau\big)u∼p(u∣v)∝exp(UCB(u)/τ) until reaching an expandable state ut=(ct,Bt(vi))u_t = (c_t, \mathcal{B}_t^{(v_i)})ut​=(ct​,Bt(vi​)​).State-Conditioned Subagent Expansion:If BSH(vi)\mathcal{B}_{SH}^{(v_i)}BSH(vi​)​ has no uninstantiated strategies at utu_tut​ then: Invoke Astr(BPM(vi))\mathcal{A}_{\text{str}}(\mathcal{B}_{PM}^{(v_i)})Astr​(BPM(vi​)​) to append refactoring hypotheses {h1,…,hKs}\{h_1, \dots, h_{K_s}\}{h1​,…,hKs​​} to BSH(vi)\mathcal{B}_{SH}^{(v_i)}BSH(vi​)​.If ct=∅c_t = \emptysetct​=∅ then: Select unexpanded hypothesis h∈BSH(vi)h \in \mathcal{B}_{SH}^{(v_i)}h∈BSH(vi​)​ and invoke Agen(BPM(vi),h)\mathcal{A}_{\text{gen}}(\mathcal{B}_{PM}^{(v_i)}, h)Agen​(BPM(vi​)​,h) to synthesize candidate code c′c'c′.Else (0<R(ut)<1.00 \lt R(u_t) \lt 1.00<R(ut​)<1.0): Invoke Apatch(ct,BPR(vi),BSH(vi))\mathcal{A}_{\text{patch}}(c_t, \mathcal{B}_{PR}^{(v_i)}, \mathcal{B}_{SH}^{(v_i)})Apatch​(ct​,BPR(vi​)​,BSH(vi​)​) to synthesize patched AST c′c'c′ and optionally invoke Atest(BPM(vi))\mathcal{A}_{\text{test}}(\mathcal{B}_{PM}^{(v_i)})Atest​(BPM(vi​)​) to expand edge-case coverage in BCT(vi)\mathcal{B}_{CT}^{(v_i)}BCT(vi​)​.Stage 3a Evaluation — Deterministic Execution Harness (Aevalexec\mathcal{A}_{\text{eval}}^{\text{exec}}Aevalexec​):
Compile c′c'c′ against parent artifacts in G\mathcal{G}G, run LSP type-checker, execute characterization tests BCT(vi)\mathcal{B}_{CT}^{(v_i)}BCT(vi​)​, and run static security linters to obtain Icompile(c′)\mathbb{I}_{\text{compile}}(c')Icompile​(c′), PassRatechar(c′)\text{PassRate}_{\text{char}}(c')PassRatechar​(c′), and raw diagnostic logs Lraw\mathcal{L}_{\text{raw}}Lraw​.Stage 3b Evaluation — Diagnostic Distiller & Parity Critic (Aevaldistill\mathcal{A}_{\text{eval}}^{\text{distill}}Aevaldistill​):If Icompile(c′)=0\mathbb{I}_{\text{compile}}(c') = 0Icompile​(c′)=0 or PassRatechar(c′)<1.0\text{PassRate}_{\text{char}}(c') \lt 1.0PassRatechar​(c′)<1.0 then:
Invoke Aevaldistill(Lraw,c′,BPM(vi))\mathcal{A}_{\text{eval}}^{\text{distill}}(\mathcal{L}_{\text{raw}}, c', \mathcal{B}_{PM}^{(v_i)})Aevaldistill​(Lraw​,c′,BPM(vi​)​) to write distilled fault record to BPR(vi)\mathcal{B}_{PR}^{(v_i)}BPR(vi​)​ and classify fault cause κ∈{INTRINSIC,EXTRINSIC_CONTRACT,EXTRINSIC_CYCLE}\kappa \in \{\texttt{INTRINSIC}, \texttt{EXTRINSIC\_CONTRACT}, \texttt{EXTRINSIC\_CYCLE}\}κ∈{INTRINSIC,EXTRINSIC_CONTRACT,EXTRINSIC_CYCLE}.If κ=EXTRINSIC_CONTRACT\kappa = \texttt{EXTRINSIC\_CONTRACT}κ=EXTRINSIC_CONTRACT on parent vpv_pvp​: Transition M(vi)←M3_ContractNegotiateM(v_i) \leftarrow \texttt{M3\_ContractNegotiate}M(vi​)←M3_ContractNegotiate, push ev_ContractConflict(vp,vi,Δspec,trace)\texttt{ev\_ContractConflict}(v_p, v_i, \Delta_{\text{spec}}, \text{trace})ev_ContractConflict(vp​,vi​,Δspec​,trace) to Qevents\mathcal{Q}_{\text{events}}Qevents​, and return.If κ=EXTRINSIC_CYCLE\kappa = \texttt{EXTRINSIC\_CYCLE}κ=EXTRINSIC_CYCLE: Push ev_CyclicDependencyFound(Ecycle,trace)\texttt{ev\_CyclicDependencyFound}(E_{\text{cycle}}, \text{trace})ev_CyclicDependencyFound(Ecycle​,trace) to Qevents\mathcal{Q}_{\text{events}}Qevents​ and return.Else (Icompile(c′)=1∧PassRatechar(c′)=1.0\mathbb{I}_{\text{compile}}(c') = 1 \land \text{PassRate}_{\text{char}}(c') = 1.0Icompile​(c′)=1∧PassRatechar​(c′)=1.0):
Invoke Aevaldistill(c′,BPM(vi))\mathcal{A}_{\text{eval}}^{\text{distill}}(c', \mathcal{B}_{PM}^{(v_i)})Aevaldistill​(c′,BPM(vi​)​) to verify 100% Given/When/Then business rule coverage and compute Scoreharden(c′)∈[0,1]\text{Score}_{\text{harden}}(c') \in [0, 1]Scoreharden​(c′)∈[0,1].Reward Computation & Backpropagation (Tier A Local Loop):
Compute R(u′)=wcompIcompile(c′)+Icompile(c′)[wcharPassRatechar(c′)+whardScoreharden(c′)]R(u') = w_{\text{comp}}\mathbb{I}_{\text{compile}}(c') + \mathbb{I}_{\text{compile}}(c')\big[w_{\text{char}}\text{PassRate}_{\text{char}}(c') + w_{\text{hard}}\text{Score}_{\text{harden}}(c')\big]R(u′)=wcomp​Icompile​(c′)+Icompile​(c′)[wchar​PassRatechar​(c′)+whard​Scoreharden​(c′)].
Create child node u′=(c′,B′(vi))u' = (c', \mathcal{B}'^{(v_i)})u′=(c′,B′(vi​)) in T(vi)\mathcal{T}^{(v_i)}T(vi​) and backpropagate R(u′)R(u')R(u′) along the path to root, updating N(⋅)N(\cdot)N(⋅) and Rˉ(⋅)\bar{R}(\cdot)Rˉ(⋅).If R(u′)=1.0R(u') = 1.0R(u′)=1.0 then: Push ev_ModuleHardened(vi,c′,BCT(vi))\texttt{ev\_ModuleHardened}(v_i, c', \mathcal{B}_{CT}^{(v_i)})ev_ModuleHardened(vi​,c′,BCT(vi​)​) to Qevents\mathcal{Q}_{\text{events}}Qevents​ and return.
3. Budget Exhaustion Fallback: If loop terminates at t=Bmax⁡t = B_{\max}t=Bmax​ without R=1.0R = 1.0R=1.0, extract highest-reward node ubest=arg⁡max⁡u∈T(vi)R(u)u_{\text{best}} = \arg\max_{u \in \mathcal{T}^{(v_i)}} R(u)ubest​=argmaxu∈T(vi​)​R(u) and push ev_ContractConflict(Parent(vi),vi,Δblocker(ubest),BPR(vi))\texttt{ev\_ContractConflict}(\text{Parent}(v_i), v_i, \Delta_{\text{blocker}}(u_{\text{best}}), \mathcal{B}_{PR}^{(v_i)})ev_ContractConflict(Parent(vi​),vi​,Δblocker​(ubest​),BPR(vi​)​) to Qevents\mathcal{Q}_{\text{events}}Qevents​.

### Interactive Formalism & Algorithmic Trace Explorer

Use the interactive diagram below to step through **Algorithm 1** and **Algorithm 2** side-by-side, inspecting the exact mathematical state transitions, guard predicates, and blackboard mutations at each step of execution:
Generating interactive visual...Structuring code, this may take a minuteTo extend this paper draft further:

::END_TURN::
