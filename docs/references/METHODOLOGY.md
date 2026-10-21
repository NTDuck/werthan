## Methodology: The <...> Framework

We present <...>, a hierarchical multi-agent system (MAS) architecture for repository-scale code-to-code (`code2code`) modernization. Given a legacy repository $\mathcal{R}_{\text{leg}}$ in source language $\mathcal{L}_{\text{src}}$, a target language specification $\mathcal{L}_{\text{tgt}}$, and an execution harness $\mathcal{H}_{\text{exec}}$, the objective of <...> is to synthesize a modernized repository $\mathcal{R}_{\text{tgt}}$ that satisfies three formal criteria: (1) syntactic and type-level compilation across all inter-module boundaries, (2) behavioral parity with $\mathcal{R}_{\text{leg}}$ across all extracted domain invariants and characterization tests, and (3) adherence to target architectural idioms and static security constraints.

---

### 1 Architectural Design Decisions and Theoretical Justifications

Repository-scale modernization violates the assumptions of standard LLM multi-agent workflows because legacy codebases exhibit non-monotonic discovery: implementation faults uncovered late in module translation frequently invalidate earlier assumptions regarding interface contracts or dependency topologies. Every structural choice in <...> is designed to resolve a specific failure mode of existing paradigms:

1. **Decoupling Deterministic Control Topology from Bounded Reasoning Context:**
   In manager-worker systems (e.g., GVS5H) and conversational hierarchies, an LLM orchestrator maintains workflow state in its prompt history and decides agent transitions via token generation. Over long horizons, this induces _state hallucination_ and _context window saturation_, particularly in Small Language Models (SLMs).

- **Design Decision:** We partition the Macro-Orchestrator $\mathcal{O} = (\mathcal{O}_{\text{engine}}, \mathcal{O}_{\text{arbiter}})$ into a **deterministic statechart runtime engine** ($\mathcal{O}_{\text{engine}}$, implemented as programmatic control code) and an on-demand, stateless **LLM Arbiter** ($\mathcal{O}_{\text{arbiter}}$) invoked exclusively for semantic ambiguity triage. All worker agents are **stateless, ephemeral functions** that never poll or maintain persistent sessions; they are instantiated on demand via push-based context hydration from $\mathcal{O}_{\text{engine}}$ and terminate immediately upon committing artifacts.

2. **Harel Statecharts over Flat FSMs and Static Pipelines:**
   Static feed-forward pipelines (e.g., ReCodeAgent) lack backward topological edges to recover when downstream validation invalidates upstream plans. Conversely, placing a flat Finite State Machine (FSM) at the root of an agent hierarchy forces lockstep global phase rollbacks: tracking $\vert{}V\vert{}$ modules across $m$ module states requires $O(m^{\vert{}V\vert{}})$ composite states.

- **Design Decision:** We formalize control via a **Harel Statechart** $\mathcal{H}$ combining **XOR Macro-Superstates** (for repository-wide phases), **Orthogonal `AND`-Regions** (reducing module state complexity from $O(m^{\vert{}V\vert{}})$ to $O(m \cdot \vert{}V\vert{})$), **Broadcast Event Calculus** (for surgical $O(1)$-hop cross-module regressions), and **Deep History ($H^*$)** (preserving verified modules across macro-regressions).

**Declarative Scatter-Gather ($D_{\max}=1$) over Autonomous Recursive Subagent Spawning:**
Repository-wide discovery phases (inventory, rule mining, contract stubbing) exceed single-agent context limits and require parallel subagent fan-out. However, granting LLM agents an autonomous `spawn_subagent` tool inside a ReAct loop creates an $O(K)$ context bottleneck when $K$ subagents return outputs into the parent agent's message buffer, while creating uncheckpointed shadow state outside the blackboard.

- **Design Decision:** Non-ARIADNE agents are prohibited from imperative subagent spawning. Instead, parallelizable macro-phases are modeled as deterministic 3-stage sub-statecharts (**`Scatter` $\to$ `Parallel Map` $\to$ `Reduce**`) managed by $\mathcal{O}_{\text{engine}}$ over partitioned blackboard slices, enforcing a strict recursion depth invariant of $D_{\max} = 1$.

4. **Two-Stage Hybrid Phase Guards ($g = g_{\text{struct}} \land g_{\text{sem}}$):**
   Purely programmatic guards (schema validation, AST parsing, DAG acyclicity) are syntactically sound but semantically shallow—they cannot detect omitted business rules or over-engineered target interfaces. Conversely, evaluating every transition purely via an LLM critic wastes tokens on structurally malformed outputs.

- **Design Decision:** Every phase transition is gated by a short-circuiting conjunction of a zero-token **Deterministic Structural Guard** ($g_{\text{struct}}$) followed conditionally by a sharded, read-isolated **Adversarial Semantic Guard** ($g_{\text{sem}}$) whose boolean verdict drives $\mathcal{O}_{\text{engine}}$ deterministically.

5. **Encapsulated Intra-Module MCTS Solvers (ARIADNE Sub-Holons):**
   Single-trajectory translators greedily patch a single code draft, frequently degenerating into local repair loops when the initial refactoring strategy is flawed. However, running Monte Carlo Tree Search (MCTS) globally across a repository causes an $O(\vert{}V\vert{} \times \vert{}\mathcal{A}\vert{})$ action-space explosion.

- **Design Decision:** We encapsulate **ARIADNE** (a Softmax-UCB MCTS planner coupled with a 4-slice local blackboard and a 5-component specialist pool) as an isolated intra-module solver node (`M2`) instantiated per DAG vertex $v_i \in V$. Furthermore, we elevate **Business Rule Extraction** to a pre-DAG global phase ($S_3$) so cross-cutting domain invariants are locked before target interface design ($S_4$).

---

### 2 Formal System Model: Two-Tier Blackboard and Harel Statechart

#### Definition 1 (Global Repository Blackboard $\mathcal{G}$ and State Vector $\Sigma(\mathcal{G})$)

The Global Blackboard $\mathcal{G}$ is a persistent, schema-validated tuple:

$$\mathcal{G} = \Big\langle \mathcal{I}_{\text{inv}}, \; G=(V,E), \; \mathcal{R}_{\text{rules}}, \; \mathcal{C}_{\text{contracts}}, \; \mathcal{A}_{\text{artifacts}}, \; \Sigma(\mathcal{G}) \Big\rangle$$

where:

- $\mathcal{I}_{\text{inv}}$ is the repository file inventory, dialect profile, and verified build/test harness configuration.
- $G = (V, E)$ is the **Condensed Dependency Directed Acyclic Graph (DAG)**, where each vertex $v_i \in V$ represents an atomic translation unit (a single file/module or a collapsed Strongly Connected Component of mutually dependent files), and each directed edge $e_{pi} = (v_p, v_i) \in E$ denotes that downstream unit $v_i$ depends on symbols exported by upstream parent $v_p$.
- $\mathcal{R}_{\text{rules}}: V \to 2^{\text{RuleCard}}$ maps each vertex to its verified `Given/When/Then` business rule cards with `file:line` legacy citations.
- $\mathcal{C}_{\text{contracts}}: E \to \text{InterfaceSpec}$ maps each dependency edge to its target-language type signature and behavioral preconditions.
- $\mathcal{A}_{\text{artifacts}}: V \to (\text{TargetAST} \times \text{TestSuite})$ stores hardened target modules.
- $\Sigma(\mathcal{G})$ is the transactionally persisted state configuration:

$$\Sigma(\mathcal{G}) = \Big\langle S_{\text{macro}}, \; \mathbf{M} = \big[M(v_1), \dots, M(v_{\vert{}V\vert{}})\big], \; \Theta_{H^*}, \; \mathcal{Q}_{\text{events}} \Big\rangle$$

comprising the active macro-state $S_{\text{macro}}$, the vector of per-module micro-states $\mathbf{M}$, the deep-history snapshot map $\Theta_{H^*}$, and the FIFO asynchronous event bus $\mathcal{Q}_{\text{events}}$.

---

#### Definition 2 (Per-Module 4-Slice Local Blackboard $\mathcal{B}^{(v_i)}$)

Each active module vertex $v_i \in V$ owns an isolated **Local Blackboard** $\mathcal{B}^{(v_i)}$ invisible to other vertices, partitioned into four typed slices following the ARIADNE formalism:

$$\mathcal{B}^{(v_i)} = \Big( \mathcal{B}_{PM}^{(v_i)}, \; \mathcal{B}_{SH}^{(v_i)}, \; \mathcal{B}_{CT}^{(v_i)}, \; \mathcal{B}_{PR}^{(v_i)} \Big)$$

1. **Problem Model ($\mathcal{B}_{PM}^{(v_i)}$):** Hydrated deterministically from $\mathcal{G}$ upon module activation with legacy AST slices, verified rule cards $\mathcal{R}_{\text{rules}}(v_i)$, upstream parent contracts $\{\mathcal{C}_{\text{contracts}}(v_p, v_i)\}$, and downstream export obligations.
2. **Strategy Hypotheses ($\mathcal{B}_{SH}^{(v_i)}$):** Competing architectural translation strategies $h_k$ with prior utility estimates and instantiation metadata.
   **Counterexamples & Characterization Tests ($\mathcal{B}_{CT}^{(v_i)}$):** Black-box I/O snapshots, property-based test suites, and falsifying inputs derived from legacy behavior.
3. **Patch & Fault Records ($\mathcal{B}_{PR}^{(v_i)}$):** Distilled compiler/test fault localizations and root-cause traces indexing failed MCTS rollouts.

---

#### Definition 3 (Harel Statechart Formalism $\mathcal{H}$)

Control flow in $\mathcal{O}_{\text{engine}}$ is governed by the Harel Statechart tuple:

$$\mathcal{H} = \Big( \mathcal{S}, \; \rho, \; \psi, \; \mathcal{E}_{\text{bus}}, \; \mathcal{G}_{\text{guards}}, \; \mathcal{T}_{\text{trans}}, \; \Theta_{H^*} \Big)$$

- **Hierarchy Function ($\rho: \mathcal{S} \to 2^{\mathcal{S}}$):** Defines state nesting. The top-level root state decomposes into seven sequential XOR Macro-States $\rho(\text{Root}) = \{S_0, S_1, S_2, S_3, S_4, S_5, S_6\}$.
- **Orthogonality Function ($\psi: \mathcal{S} \to \{\texttt{XOR}, \texttt{AND}\}$):** Specifies state concurrency:
- For parallelizable discovery states $S_m \in \{S_1, S_2, S_3, S_4, S_6\}$, the internal map sub-state $S_m.2$ has $\psi(S_m.2) = \texttt{AND}$ over $K$ stateless shard workers: $S_m.2 = \bigparallel_{k=1}^K W(\text{shard}_k)$.
- For the DAG Transformation superstate $S_5$, $\psi(S_5) = \texttt{AND}$ over $\vert{}V\vert{}$ concurrent module regions:

$$S_5 = R(v_1) \;\parallel\; R(v_2) \;\parallel\; \dots \;\parallel\; R(v_{\vert{}V\vert{}})$$

where each orthogonal region $R(v_i)$ executes an isomorphic 6-state Micro-FSM $\rho(R(v_i)) = \{\texttt{M0}, \texttt{M1}, \texttt{M2}, \texttt{M3}, \texttt{M4}, \texttt{M5}\}$.

- **Guarded Broadcast Transition Relation ($\mathcal{T}_{\text{trans}}$):** Every transition $t \in \mathcal{T}_{\text{trans}}$ takes the Harel canonical form:

$$s_{\text{src}} \xrightarrow{\; \text{ev} \; [g(\mathcal{G})] \; / \; \alpha(\mathcal{G}) \;} s_{\text{dst}}$$

where $\text{ev} \in \mathcal{E}_{\text{bus}}$ is the triggering event popped from $\mathcal{Q}_{\text{events}}$, $g(\mathcal{G}) \in \{\text{True}, \text{False}\}$ is the guard predicate, and $\alpha(\mathcal{G})$ is the transition action (which updates $\Sigma(\mathcal{G})$, optionally broadcasts internal events to orthogonal regions, and invokes the `ContextAssembler`).

- **Deep History Pseudostate ($\Theta_{H^*}$):** Upon any macro-regression exiting superstate $S_5$ to an earlier state $S_m$ ($m \in \{2, 3, 4\}$), $\mathcal{O}_{\text{engine}}$ records $\Theta_{H^*}(S_5) \leftarrow \mathbf{M}$. Upon re-entering $S_5$ via the $H^*$ connector, every orthogonal region $R(v_i)$ unaffected by the topological repair is restored directly to its prior state $M(v_i)$ (preserving `M4_Hardened` modules without re-execution).

---

#### Definition 4 (Dynamic Context Hydration via `TaskActivationEnvelope`)

Because $\mathcal{O}_{\text{engine}}$ is programmatic code while worker agents require rich task context, every transition action $\alpha(\mathcal{G})$ invokes a deterministic **`ContextAssembler`** function:

$$\text{Hydrate}\big(\text{AgentRole}, \; \text{mode}, \; \text{target\_ref}, \; \mathcal{G}\big) \longrightarrow \mathcal{E}_{\text{act}} = \Big\langle \mu_{\text{mode}}, \; \mathcal{G}_{\text{slice}}, \; \Delta_{\text{directive}}, \; \mathcal{T}_{\text{tools}} \Big\rangle$$

where $\mu_{\text{mode}} \in \{\texttt{INITIAL}, \texttt{CRITIC\_RETRY}, \texttt{WARM\_START\_REGRESSION}\}$ selects the system prompt template, $\mathcal{G}_{\text{slice}}$ queries only the bounded 1-hop subgraph relevant to `target_ref`, $\Delta_{\text{directive}}$ injects the exact failure critique or contract conflict trace, and $\mathcal{T}_{\text{tools}}$ binds read-only deterministic AST/LSP navigation tools.

---

### 3 Macro-Level Orchestration: The 7-Phase Pipeline & Declarative Scatter-Gather

Aligning with the domain lifecycle of the Claude Code Modernization workflow, $\mathcal{O}_{\text{engine}}$ enforces five global discovery and specification phases ($S_0 \to S_4$) prior to DAG transformation ($S_5$) and final hardening ($S_6$).

#### 1. Declarative Scatter-Gather Sub-Statechart ($S_m.1 \to S_m.2 \to S_m.3$)

For any macro-state $S_m \in \{S_1, S_2, S_3, S_4, S_6\}$ operating over a large repository, execution proceeds through three internal substates:

1. **$S_m.1$ (`Scatter` / Partitioning):** A deterministic partitioner (using AST/directory metrics or DAG vertex batching) divides the phase workload into $K$ bounded shards $\{\text{shard}_1, \dots, \text{shard}_K\}$ such that each shard satisfies the SLM context constraint $\text{Tokens}(\text{shard}_k) \le L_{\max}$, writing a `WorkManifest` to $\mathcal{G}$.
2. **$S_m.2$ (`Parallel Map` over Orthogonal Shards):** $\mathcal{O}_{\text{engine}}$ spawns $K$ parallel, stateless Map subagents $W_1, \dots, W_K$. Each worker $W_k$ reads only $\mathcal{G}[\text{shard}_k]$ (augmented with read-only deterministic LSP symbol lookup tools for out-of-shard definitions). If $W_k$ encounters a complex cross-file dependency, it emits a declarative tag `UnresolvedCrossLink(src_id, tgt_symbol)` rather than spawning a child agent. Each $W_k$ writes its output directly to $\mathcal{G}[\text{shard}_k]$, pushes `ev_ShardDone(k)`, and terminates.
   **$S_m.3$ (`Reduce` / Cross-Shard Linker):** Once all $K$ shards complete, $\mathcal{O}_{\text{engine}}$ invokes a lightweight Linker agent (or deterministic graph reducer) that queries only the exported boundary headers and `UnresolvedCrossLink` tags across $\mathcal{G}[\text{shard}_{1\dots K}]$ to synthesize unified cross-module links.

#### 2. Two-Stage Guard Verification Pipeline ($g = g_{\text{struct}} \land g_{\text{sem}}$)

Before transitioning out of phase $S_m$, $\mathcal{O}_{\text{engine}}$ evaluates:

$$g(\mathcal{G}) = \underbrace{\Big( \bigwedge_{k=1}^K g_{\text{struct}}\big(\mathcal{G}[\text{shard}_k]\big) \Big)}_{\text{Stage 1: Zero-Token Programmatic Check}} \;\land\; \underbrace{\Big( \bigwedge_{b=1}^{B} g_{\text{sem}}\big(\mathcal{G}[\text{batch}_b]\big) \Big)}_{\text{Stage 2: Sharded Adversarial LLM Critic}}$$

If Stage 1 ($g_{\text{struct}}$) fails on $\text{shard}_k$, $\mathcal{O}_{\text{engine}}$ immediately re-dispatches only $W_k$ with `ActivationMode = CRITIC_RETRY` and the deterministic validator traceback. Only when Stage 1 passes across all shards does $\mathcal{O}_{\text{engine}}$ fan out Stage 2: parallel, read-isolated **Semantic Critic** subagents (`ShardCritic`) that audit batches of produced artifacts against legacy `file:line` source citations and output a structured `CritiqueRecord`.

#### Specification of the 7 Macro-States ($S_0 \dots S_6$)

| State     | Phase Name               | Scatter-Gather Topology ($S_m.1 \to S_m.3$)  | Structural Guard ($g_{\text{struct}}$)                                        | Adversarial Semantic Guard ($g_{\text{sem}}$) |
| --------- | ------------------------ | -------------------------------------------- | ----------------------------------------------------------------------------- | --------------------------------------------- |
| **$S_0$** | **`Intent & Preflight`** | Single `PreflightProbe` agent (no sharding). | Legacy build compiles; baseline test harness executes; `INTENT` schema valid. | N/A (Purely empirical execution gate).        |
| **$S_1$** | **`Assess`**             | **Scatter:** Directory shards.<br>           |

<br>**Map:** `ShardAssessor` (dialect, debt).<br>

<br>**Reduce:** `AssessmentSynthesizer`. | 100% file inventory coverage against `scc`/`cloc`; valid metric ranges. | `AssessmentCritic` verifies every risk claim cites valid legacy files and flags unmapped dialects. |
| **$S_2$** | **`Map` (DAG Construction)** | **Scatter:** Static LSP/Tree-sitter builds explicit graph & flags dynamic files.<br>

<br>**Map:** `ImplicitCouplingScanner` on flagged files.<br>

<br>**Reduce:** Tarjan’s SCC algorithm condenses cycles into DAG $G=(V,E)$. | $G=(V,E)$ is strictly acyclic; all repository entry points and files belong to exactly one $v_i \in V$. | `TopologyCritic` audits flagged reflection, shared DB tables, and event buses for missed implicit edges. |
| **$S_3$** | **`Extract Business Rules`** | **Scatter:** Vertex batches from $V$.<br>

<br>**Map:** `RuleMiner` extracts `Given/When/Then` cards.<br>

<br>**Reduce:** `CrossModuleRuleLinker` resolves cross-vertex rules. | Every rule card contains valid `Given/When/Then` clauses, `legacy/file:line` pointers, and `Observed` vs. `Inferred` tags. | `RuleAuditor` (independent agent) re-verifies every rule card against legacy AST snippets for completeness and hallucination. |
| **$S_4$** | **`Brief & Reimagine Contracts`** | **Scatter:** `MacroArchitect` sets global idioms; edges $E$ sharded by domain.<br>

<br>**Map:** `ContractStubber` writes interface stubs.<br>

<br>**Reduce:** Target compiler checks stub compatibility. | Every edge $e_{pi} \in E$ has a target interface stub in $\mathcal{C}_{\text{contracts}}$ that compiles cleanly as a header/stub. | `ArchitectureCritic` audits target interfaces against $\mathcal{R}_{\text{rules}}$ to prevent over-engineering or unrepresentable domain invariants. |
| **$S_5$** | **`Iterative DAG Transform`** | **Composite Orthogonal `AND`-State:** Topological Wave Scheduler over $R(v_1) \parallel \dots \parallel R(v_{\vert{}V\vert{}})$ running **ARIADNE Clusters**. | $\forall v_i \in V, \; M(v_i) = \texttt{M4\_Hardened}$ with composite reward $R(c_i^\star) = 1.0$. | Enforced intra-cluster via $\mathcal{A}_{\text{eval}}^{\text{distill}}$ rule-parity verification prior to entering `M4`. |
| **$S_6$** | **`System Verify & Harden`** | **Scatter:** E2E test suites & SAST scanners.<br>

<br>**Map:** `IntegrationVerifier` & `SecurityAuditor`.<br>

<br>**Reduce:** Final parity sign-off generator. | 100% E2E integration test pass rate; zero critical SAST/CVE vulnerabilities. | `ReleaseAuditor` verifies end-to-end compliance with `INTENT` and signs off per-module verdicts. |

---

### 4 Topological Wave Scheduling & Encapsulated ARIADNE MCTS Nodes

Upon entering Composite State $S_5$, $\mathcal{O}_{\text{engine}}$ activates its **Topological Wave Scheduler**, a deterministic subroutine based on Kahn’s algorithm that continuously evaluates the set of unblocked vertices:

$$\text{ReadySet}(\mathcal{G}) = \Big\{ v_i \in V \;\Big\vert{}\; M(v_i) = \texttt{M0\_Blocked} \;\land\; \forall (v_p, v_i) \in E, \; M(v_p) = \texttt{M4\_Hardened} \Big\}$$

#### 1. Per-Module Micro-FSM ($\mathcal{M}_{v_i}$)

Each orthogonal region $R(v_i)$ transitions through six micro-states:

- **`M0_Blocked`:** Waiting for upstream parents $\{v_p \mid (v_p, v_i) \in E\}$ to reach `M4_Hardened`.
- **`M1_ContextHydration` (Deterministic, Non-LLM):** Triggered when $v_i \in \text{ReadySet}(\mathcal{G})$. Deterministically copies $\mathcal{R}_{\text{rules}}(v_i)$ and parent contracts $\{\mathcal{C}_{\text{contracts}}(v_p, v_i)\}$ from $\mathcal{G}$ into the Problem Model slice $\mathcal{B}_{PM}^{(v_i)}$ and transitions immediately to `M2`.
- **`M2_ARIADNE_MCTS` (Active Transformation & Verification Node):** Executes the encapsulated ARIADNE cluster. Transitions to `M4_Hardened` if a candidate $c^\star$ achieves $R(c^\star) = 1.0$; transitions to `M3_ContractNegotiate` if an extrinsic upstream contract conflict is proved; or emits a Tier-C macro-regression if an undiscovered dependency cycle is found.
- **`M3_ContractNegotiate`:** Emits broadcast event `ev_ContractConflict(v_p, v_i, delta_spec, proof_trace)` to $\mathcal{Q}_{\text{events}}$ on $\mathcal{G}$ and parks $v_i$ back in `M0_Blocked`.
- **`M4_Hardened`:** Commits verified target code $c^\star$ and characterization test suite $\mathcal{B}_{CT}^{(v_i)}$ to $\mathcal{A}_{\text{artifacts}}(v_i)$ on $\mathcal{G}$ and emits `ev_ModuleHardened(v_i)`, triggering $\mathcal{O}_{\text{engine}}$ to recompute $\text{ReadySet}(\mathcal{G})$.
- **`M5_Invalidated` (Warm-Start Regression State):** Entered when a `M4_Hardened` module receives a validated `ev_ContractConflict` from a downstream child (or when its own parent updates an interface). Deterministically appends the delta constraint $\Delta_{\text{spec}}$ to $\mathcal{B}_{PM}^{(v_i)}$ **while preserving** the existing MCTS search tree $\mathcal{T}^{(v_i)}$, strategy hypotheses $\mathcal{B}_{SH}^{(v_i)}$, and test suite $\mathcal{B}_{CT}^{(v_i)}$, then transitions back to `M2_ARIADNE_MCTS`.

---

#### 2. Internal Architecture of the Encapsulated ARIADNE Cluster (`M2`)

Inside state `M2`, module $v_i$ is solved by an isolated ARIADNE cluster consisting of a **Deterministic MCTS Controller ($\mathcal{C}_{\text{MCTS}}^{(v_i)}$)**, **4 Generative LLM Subagents**, and a **Two-Stage Hybrid Evaluator ($\mathcal{A}_{\text{eval}}$)** operating over $\mathcal{B}^{(v_i)}$:

1. **Deterministic MCTS Controller ($\mathcal{C}_{\text{MCTS}}^{(v_i)}$ — Algorithmic Code):**
   Maintains a search tree $\mathcal{T}^{(v_i)}$ where each node $u$ represents a state $s_t = (c_t, \mathcal{B}_t^{(v_i)})$. At each rollout $t \le B_{\max}$, $\mathcal{C}_{\text{MCTS}}^{(v_i)}$ traverses from the root $s_0$ by computing Upper Confidence Bound (UCB) scores over child states $u \in \text{Children}(v)$:

$$\text{UCB}(u) = \bar{R}(u) + C_{\text{exp}} \sqrt{\frac{\ln(N(v) + 1)}{N(u)}} + \gamma \cdot P_{\text{prior}}(u)$$

where $\bar{R}(u)$ is the mean backpropagated reward, $N(\cdot)$ denotes visit counts, and $P_{\text{prior}}(u)$ is the strategy utility prior stored in $\mathcal{B}_{SH}^{(v_i)}$. To avoid premature greedy convergence on flawed refactoring templates, child selection follows a temperature-controlled softmax distribution:

$$p(u \mid v) = \frac{\exp\big((\text{UCB}(u) - \max_{w} \text{UCB}(w)) / \tau\big)}{\sum_{w \in \text{Children}(v)} \exp\big((\text{UCB}(w) - \max_{w'} \text{UCB}(w')) / \tau\big)}$$

2. **State-Conditioned Dispatch of the 4 Generative LLM Subagents:**
   Once node $u = (c_t, \mathcal{B}_t^{(v_i)})$ is selected, $\mathcal{C}_{\text{MCTS}}^{(v_i)}$ inspects deterministic state flags on $u$ to invoke one of four ephemeral LLM agents:

- **$\mathcal{A}_{\text{str}}$ (Strategy Analysis Agent):** Invoked when $u$ lacks unexpanded strategy branches; reads $\mathcal{B}_{PM}^{(v_i)}$ and writes $K_s$ distinct refactoring hypotheses (e.g., structural port vs. state-pattern decomposition) with prior scores $P_{\text{prior}}$ to $\mathcal{B}_{SH}^{(v_i)}$.
- **$\mathcal{A}_{\text{gen}}$ (Code Generation Agent):** Invoked on an uninstantiated strategy $h \in \mathcal{B}_{SH}^{(v_i)}$; synthesizes candidate target code $c'$ conforming to $\mathcal{B}_{PM}^{(v_i)}$.
- **$\mathcal{A}_{\text{test}}$ (Adversarial Test Generation Agent):** Reads $\mathcal{B}_{PM}^{(v_i)}$ and legacy behavior (**strictly read-isolated from candidate $c'$** to prevent test weakening) to synthesize characterization I/O snapshots and property-based tests into $\mathcal{B}_{CT}^{(v_i)}$.
- **$\mathcal{A}_{\text{patch}}$ (Surgical Code Repair Agent):** Invoked when $c_t$ has failed verification (`0 < R(s_t) < 1`); reads $c_t$ and the localized fault record in $\mathcal{B}_{PR}^{(v_i)}$ to emit a minimal AST/diff patch $c''$.

**Two-Stage Hybrid Evaluator ($\mathcal{A}_{\text{eval}} = \mathcal{A}_{\text{eval}}^{\text{exec}} \circ \mathcal{A}_{\text{eval}}^{\text{distill}}$):**
Every generated or patched candidate $c'$ is evaluated in two sequential stages:

- **Stage 3a — Deterministic Execution Harness ($\mathcal{A}_{\text{eval}}^{\text{exec}}$, Non-LLM):** Compiles $c'$ against upstream target artifacts in $\mathcal{G}$, runs LSP type-checking, executes the characterization test suite $\mathcal{B}_{CT}^{(v_i)}$ via differential test runners, and runs static security/lint analyzers.
- **Stage 3b — Diagnostic Distiller & Parity Critic ($\mathcal{A}_{\text{eval}}^{\text{distill}}$, Lightweight LLM):**
- _On Stage 3a Failure:_ Distills raw compiler/test stack traces into a concise, localized fault card in $\mathcal{B}_{PR}^{(v_i)}$ and classifies the fault cause $\kappa \in \{\texttt{INTRINSIC}, \; \texttt{EXTRINSIC\_CONTRACT}, \; \texttt{EXTRINSIC\_CYCLE}\}$.
- _On Stage 3a Pass:_ Audits $c'$ against all `Given/When/Then` business rule cards in $\mathcal{B}_{PM}^{(v_i)}$ to compute the semantic parity & hardening score $\text{Score}_{\text{harden}}(c') \in [0, 1]$.

4. **Gated Hierarchical Reward Function $R(s_t)$:**
   The scalar reward backpropagated up $\mathcal{T}^{(v_i)}$ is strictly gated so uncompilable code receives zero credit from partial heuristics:

$$R(s_t) = w_{\text{comp}} \cdot \mathbb{I}_{\text{compile}}(c_t) + \mathbb{I}_{\text{compile}}(c_t) \cdot \Big[ w_{\text{char}} \cdot \text{PassRate}_{\text{char}}\big(c_t, \mathcal{B}_{CT}^{(v_i)}\big) + w_{\text{hard}} \cdot \text{Score}_{\text{harden}}\big(c_t, \mathcal{B}_{PM}^{(v_i)}\big) \Big]$$

where $w_{\text{comp}} + w_{\text{char}} + w_{\text{hard}} = 1.0$ (default $0.2, 0.6, 0.2$). Because $\mathbb{I}_{\text{compile}} \in \{0,1\}$, $\text{PassRate}_{\text{char}} \in [0,1]$, and $\text{Score}_{\text{harden}} \in [0,1]$, achieving $R(s_t) = 1.0$ is a necessary and sufficient condition for full compilation, 100% characterization test parity, and full semantic rule adherence.

---

### 5 Multi-Tier Regression Calculus and Warm-Start Recovery

<...> classifies and resolves discovered anomalies across three strict locality tiers, ensuring that errors are contained to the smallest possible scope:

1. **Tier A — Intra-Module MCTS Backtrack ($\kappa = \texttt{INTRINSIC}$, Zero Global Traffic):**
   When $R(s_t) \lt 1.0$ and $\mathcal{A}_{\text{eval}}^{\text{distill}}$ classifies the fault as $\texttt{INTRINSIC}$ to module $v_i$, recovery is handled entirely inside $\mathcal{C}_{\text{MCTS}}^{(v_i)}$: the reward $R(s_t)$ is backpropagated up $\mathcal{T}^{(v_i)}$, lowering $\bar{R}(u)$ and shifting subsequent Softmax-UCB rollouts either to invoke $\mathcal{A}_{\text{patch}}$ using $\mathcal{B}_{PR}^{(v_i)}$ or to branch to an alternative refactoring strategy in $\mathcal{B}_{SH}^{(v_i)}$. Macro-Orchestrator $\mathcal{O}$ is never interrupted.
2. **Tier B — Cross-Module Micro-Regression via Broadcast Calculus ($\kappa = \texttt{EXTRINSIC\_CONTRACT}$):**
   When downstream module $v_i$ proves that an upstream parent $v_p$’s exported contract $\mathcal{C}_{\text{contracts}}(v_p, v_i)$ is insufficient (or when $v_i$ exhausts $B_{\max}$ rollouts blocked on $e_{pi}$), $R(v_i)$ transitions $\texttt{M2} \to \texttt{M3}$ and broadcasts `ev_ContractConflict(v_p, v_i, delta_spec, proof_trace)` to $\mathcal{Q}_{\text{events}}$.

- **Verification by $\mathcal{O}$:** $\mathcal{O}_{\text{engine}}$ runs a structural check confirming the symbol reference across $e_{pi}$, and invokes $\mathcal{O}_{\text{arbiter}}$ to validate that `delta_spec` is a legitimate contract requirement rather than a hallucinated excuse by $v_i$.
- **Surgical Orthogonal Transition:** Upon validation, $\mathcal{O}_{\text{engine}}$ updates $\mathcal{C}_{\text{contracts}}(v_p, v_i)$ on $\mathcal{G}$, parks $M(v_i) \leftarrow \texttt{M0\_Blocked}$, and fires the broadcast transition on orthogonal region $R(v_p)$:

$$\texttt{M4\_Hardened} \xrightarrow{\; \text{ev\_ContractConflict}(v_p, v_i, \Delta_{\text{spec}}) \; [\text{ArbiterValid}] \; / \; \mathcal{B}_{PM}^{(v_p)} \leftarrow \mathcal{B}_{PM}^{(v_p)} \cup \{\Delta_{\text{spec}}\} \;} \texttt{M5\_Invalidated} \longrightarrow \texttt{M2\_ARIADNE\_MCTS}$$

- **Warm-Start Efficiency:** Because $\mathcal{B}_{SH}^{(v_p)}$, $\mathcal{B}_{CT}^{(v_p)}$, and the prior winning node $c_p^\star \in \mathcal{T}^{(v_p)}$ are retained in memory, $R(v_p)$ initializes a child repair branch directly from $c_p^\star$ under the updated $\mathcal{B}_{PM}^{(v_p)}$, adapting its interface in a few $\mathcal{A}_{\text{patch}}$ rollouts without re-discovering $v_p$’s core logic.

**Tier C — Macro-Architectural Regression with Deep History $H^*$ ($\kappa = \texttt{EXTRINSIC\_CYCLE}$):**
When module $v_i$ discovers an undocumented dependency cycle $(v_i \to \dots \to v_p \to v_i)$ via runtime reflection or dynamic loading, it emits `ev_CyclicDependencyFound(cycle_edges, proof_trace)` to $\mathcal{Q}_{\text{events}}$.

- **Action by $\mathcal{O}_{\text{engine}}$:** $\mathcal{O}_{\text{engine}}$ checkpoints the current module state vector $\Theta_{H^*}(S_5) \leftarrow \mathbf{M}$, pauses only the active regions belonging to `cycle_edges`, and transitions the macro-state $S_5 \to S_2$ (`Map`).
- **SCC Condensation & $H^*$ Restoration:** In $S_2$, Tarjan’s algorithm collapses the cyclic vertices into a single composite Strongly Connected Component vertex $v_{\text{SCC}} \in V$, followed by delta rule linking ($S_3$) and boundary stubbing ($S_4$) strictly for $v_{\text{SCC}}$. Finally, $\mathcal{O}_{\text{engine}}$ transitions $S_4 \to H^*(S_5)$: all unaffected vertices $v_k \notin v_{\text{SCC}}$ are immediately restored to their checkpointed states $\Theta_{H^*}(S_5)[v_k]$ (preserving all `M4_Hardened` modules), while $v_{\text{SCC}}$ enters the wave scheduler as a unified translation unit.

---

### 6 Formal Execution Algorithms

Below are the two core algorithms governing the deterministic Macro-Orchestrator ($\mathcal{O}_{\text{engine}}$) and the Encapsulated ARIADNE Cluster (`M2`).

---

#### Algorithm 1: Deterministic Macro-Statechart Engine ($\mathcal{O}_{\text{engine}}$) & Declarative Scatter-Gather

- **Input:** Legacy Repository $\mathcal{R}_{\text{leg}}$, Target Spec $\mathcal{L}_{\text{tgt}}$, Execution Harness $\mathcal{H}_{\text{exec}}$
- **Output:** Hardened Target Repository $\mathcal{R}_{\text{tgt}}$ stored in $\mathcal{A}_{\text{artifacts}}$ on $\mathcal{G}$

1. **Initialize** Global Blackboard $\mathcal{G} \leftarrow \emptyset$, State Vector $\Sigma(\mathcal{G}) \leftarrow \langle S_{\text{macro}} = S_0, \; \mathbf{M} = [], \; \Theta_{H^*} = \emptyset, \; \mathcal{Q}_{\text{events}} = [\texttt{ev\_Start}] \rangle$.
2. **While** $S_{\text{macro}} \neq \texttt{TERMINATED}$ **do:**
3. **If** $S_{\text{macro}} \in \{S_0, S_1, S_2, S_3, S_4, S_6\}$ _(Global Discovery, Specification & Verification Phases)_ **then:**
4. **Substate $S_m.1$ (Scatter):** Partition phase workload into bounded shards $\{\text{shard}_1, \dots, \text{shard}_K\}$ with $\text{Tokens}(\text{shard}_k) \le L_{\max}$ and write `WorkManifest` to $\mathcal{G}$.
5. **Substate $S_m.2$ (Parallel Map):** For each $k \in \{1, \dots, K\}$ **in parallel**, construct envelope $\mathcal{E}_{\text{act}}^{(k)} \leftarrow \text{Hydrate}(\text{MapAgent}(S_{\text{macro}}), \texttt{INITIAL}, \text{shard}_k, \mathcal{G})$ and invoke stateless worker $W_k(\mathcal{E}_{\text{act}}^{(k)})$, which writes output directly to $\mathcal{G}[\text{shard}_k]$ and terminates.
   **Substate $S_m.3$ (Reduce):** Invoke Cross-Shard Linker over boundary headers and `UnresolvedCrossLink` tags in $\mathcal{G}[\text{shard}_{1\dots K}]$.
6. **Stage-1 Structural Guard:**

- **For each** $k \in \{1, \dots, K\}$ where $g_{\text{struct}}(\mathcal{G}[\text{shard}_k]) = \text{False}$:
  Re-hydrate $\mathcal{E}_{\text{act}}^{(k)}$ with $\mu_{\text{mode}} = \texttt{CRITIC\_RETRY}$ and validator traceback; re-invoke $W_k$ until $g_{\text{struct}}$ passes.

5. **Stage-2 Adversarial Semantic Guard:**

- Fan out read-isolated `ShardCritic` subagents across artifact batches $b \in \{1, \dots, B\}$.
- **If** $\exists b$ such that $g_{\text{sem}}(\mathcal{G}[\text{batch}_b]) = \texttt{REJECT\_LOCAL}$ **then:**
  Re-hydrate and re-invoke only the offending shard workers with `DeltaDirective` set to the Critic’s `CritiqueRecord`; **continue loop**.
- **Else if** $\exists b$ such that $g_{\text{sem}}(\mathcal{G}[\text{batch}_b]) = \texttt{REGRESS\_UPSTREAM}$ **then:**
  Transition $S_{\text{macro}}$ to the target upstream macro-state $S_{\text{target}}$; **continue loop**.

6. **Advance Phase:** If $S_{\text{macro}} = S_4$, initialize $\mathbf{M}[v_i] \leftarrow \Theta_{H^*}(S_5)[v_i]$ (if $v_i \in \Theta_{H^*}$) else $\texttt{M0\_Blocked}$ for all $v_i \in V$. Transition $S_{\text{macro}} \leftarrow \text{NextState}(S_{\text{macro}})$.

7. **Else if** $S_{\text{macro}} = S_5$ _(Composite Orthogonal State: Iterative DAG Transformation)_ **then:**
8. **Topological Wave Dispatch:** Compute $\text{ReadySet}(\mathcal{G}) = \{v_i \in V \mid M(v_i) = \texttt{M0\_Blocked} \land \forall (v_p, v_i) \in E, M(v_p) = \texttt{M4\_Hardened}\}$.
9. **For each** $v_i \in \text{ReadySet}(\mathcal{G})$ **in parallel do:**

- **State `M1`:** Deterministically copy $\mathcal{R}_{\text{rules}}(v_i)$ and parent contracts $\{\mathcal{C}_{\text{contracts}}(v_p, v_i)\}$ from $\mathcal{G}$ into $\mathcal{B}_{PM}^{(v_i)}$.
- **State `M2`:** Transition $M(v_i) \leftarrow \texttt{M2\_ARIADNE\_MCTS}$ and asynchronously execute **Algorithm 2** ($\text{ARIADNE\_Cluster}(v_i, \mathcal{B}^{(v_i)}, \mathcal{G})$).

**Event Bus Processing:** Pop next event $\text{ev} \leftarrow \mathcal{Q}_{\text{events}}.\text{pop}()$:

- **Case** $\text{ev} = \texttt{ev\_ModuleHardened}(v_i, c_i^\star, \mathcal{B}_{CT}^{(v_i)})$:
  Commit $(c_i^\star, \mathcal{B}_{CT}^{(v_i)})$ to $\mathcal{A}_{\text{artifacts}}(v_i)$; set $M(v_i) \leftarrow \texttt{M4\_Hardened}$. If $\forall v \in V, M(v) = \texttt{M4\_Hardened}$, transition $S_{\text{macro}} \leftarrow S_6$.
- **Case** $\text{ev} = \texttt{ev\_ContractConflict}(v_p, v_i, \Delta_{\text{spec}}, \text{trace})$ _(Tier B Micro-Regression)_:
  **If** $g_{\text{struct}}(\text{trace}) \land \mathcal{O}_{\text{arbiter}}(\Delta_{\text{spec}}, \text{trace}) = \texttt{VALID}$ **then:**
  Update $\mathcal{C}_{\text{contracts}}(v_p, v_i) \leftarrow \mathcal{C}_{\text{contracts}}(v_p, v_i) \oplus \Delta_{\text{spec}}$ on $\mathcal{G}$;
  Set $M(v_i) \leftarrow \texttt{M0\_Blocked}$;
  Set $M(v_p) \leftarrow \texttt{M5\_Invalidated}$; append $\Delta_{\text{spec}}$ to $\mathcal{B}_{PM}^{(v_p)}$ _(preserving $\mathcal{T}^{(v_p)}, \mathcal{B}_{SH}^{(v_p)}, \mathcal{B}_{CT}^{(v_p)}$)_;
  Transition $M(v_p) \leftarrow \texttt{M2\_ARIADNE\_MCTS}$ with $\mu_{\text{mode}} = \texttt{WARM\_START\_REGRESSION}$.
- **Case** $\text{ev} = \texttt{ev\_CyclicDependencyFound}(E_{\text{cycle}}, \text{trace})$ _(Tier C Macro-Regression)_:
  **If** $g_{\text{struct}}(E_{\text{cycle}}) \land \mathcal{O}_{\text{arbiter}}(E_{\text{cycle}}, \text{trace}) = \texttt{VALID}$ **then:**
  Save Deep History $\Theta_{H^*}(S_5) \leftarrow \mathbf{M}$; quiesce active regions in $E_{\text{cycle}}$;
  Transition $S_{\text{macro}} \leftarrow S_2$ to condense $E_{\text{cycle}}$ into $v_{\text{SCC}}$ via Tarjan's SCC and resume via $H^*(S_5)$.

**Return** $\mathcal{R}_{\text{tgt}} \leftarrow \mathcal{A}_{\text{artifacts}}(\mathcal{G})$.

---

#### Algorithm 2: Encapsulated ARIADNE Module Cluster (`M2`) & Multi-Tier Regression Trigger

- **Input:** Vertex $v_i \in V$, Local 4-Slice Blackboard $\mathcal{B}^{(v_i)} = (\mathcal{B}_{PM}^{(v_i)}, \mathcal{B}_{SH}^{(v_i)}, \mathcal{B}_{CT}^{(v_i)}, \mathcal{B}_{PR}^{(v_i)})$, Activation Mode $\mu_{\text{mode}}$, Rollout Budget $B_{\max}$
- **Output:** Emits completion or regression interrupt event to $\mathcal{Q}_{\text{events}}$ on Global Blackboard $\mathcal{G}$

1. **Initialize MCTS Tree $\mathcal{T}^{(v_i)}$:**

- **If** $\mu_{\text{mode}} = \texttt{INITIAL}$ **then:** Create root node $u_0 = (c_0 = \emptyset, \mathcal{B}_0^{(v_i)})$ and invoke Adversarial Test Agent $\mathcal{A}_{\text{test}}(\mathcal{B}_{PM}^{(v_i)})$ to seed characterization test suite $\mathcal{B}_{CT}^{(v_i)}$.
- **Else if** $\mu_{\text{mode}} = \texttt{WARM\_START\_REGRESSION}$ **then:** Retain existing tree $\mathcal{T}^{(v_i)}$ and set active root pointer to previously verified node $u_{\text{warm}} = (c_i^\star, \mathcal{B}^{(v_i)})$ updated with new $\mathcal{B}_{PM}^{(v_i)}$ contract constraints.

2. **For** rollout $t = 1$ **to** $B_{\max}$ **do:**
1. **Selection (Deterministic Softmax-UCB in $\mathcal{C}_{\text{MCTS}}^{(v_i)}$):**
   Starting from root, traverse $\mathcal{T}^{(v_i)}$ by sampling child $u \sim p(u \mid v) \propto \exp\big(\text{UCB}(u)/\tau\big)$ until reaching an expandable state $u_t = (c_t, \mathcal{B}_t^{(v_i)})$.
1. **State-Conditioned Subagent Expansion:**

- **If** $\mathcal{B}_{SH}^{(v_i)}$ has no uninstantiated strategies at $u_t$ **then:** Invoke $\mathcal{A}_{\text{str}}(\mathcal{B}_{PM}^{(v_i)})$ to append refactoring hypotheses $\{h_1, \dots, h_{K_s}\}$ to $\mathcal{B}_{SH}^{(v_i)}$.
- **If** $c_t = \emptyset$ **then:** Select unexpanded hypothesis $h \in \mathcal{B}_{SH}^{(v_i)}$ and invoke $\mathcal{A}_{\text{gen}}(\mathcal{B}_{PM}^{(v_i)}, h)$ to synthesize candidate code $c'$.
- **Else** ($0 \lt R(u_t) \lt 1.0$): Invoke $\mathcal{A}_{\text{patch}}(c_t, \mathcal{B}_{PR}^{(v_i)}, \mathcal{B}_{SH}^{(v_i)})$ to synthesize patched AST $c'$ and optionally invoke $\mathcal{A}_{\text{test}}(\mathcal{B}_{PM}^{(v_i)})$ to expand edge-case coverage in $\mathcal{B}_{CT}^{(v_i)}$.

**Stage 3a Evaluation — Deterministic Execution Harness ($\mathcal{A}_{\text{eval}}^{\text{exec}}$):**
Compile $c'$ against parent artifacts in $\mathcal{G}$, run LSP type-checker, execute characterization tests $\mathcal{B}_{CT}^{(v_i)}$, and run static security linters to obtain $\mathbb{I}_{\text{compile}}(c')$, $\text{PassRate}_{\text{char}}(c')$, and raw diagnostic logs $\mathcal{L}_{\text{raw}}$. 4. **Stage 3b Evaluation — Diagnostic Distiller & Parity Critic ($\mathcal{A}_{\text{eval}}^{\text{distill}}$):**

- **If** $\mathbb{I}_{\text{compile}}(c') = 0$ **or** $\text{PassRate}_{\text{char}}(c') \lt 1.0$ **then:**
  Invoke $\mathcal{A}_{\text{eval}}^{\text{distill}}(\mathcal{L}_{\text{raw}}, c', \mathcal{B}_{PM}^{(v_i)})$ to write distilled fault record to $\mathcal{B}_{PR}^{(v_i)}$ and classify fault cause $\kappa \in \{\texttt{INTRINSIC}, \texttt{EXTRINSIC\_CONTRACT}, \texttt{EXTRINSIC\_CYCLE}\}$.
- **If** $\kappa = \texttt{EXTRINSIC\_CONTRACT}$ on parent $v_p$: Transition $M(v_i) \leftarrow \texttt{M3\_ContractNegotiate}$, push $\texttt{ev\_ContractConflict}(v_p, v_i, \Delta_{\text{spec}}, \text{trace})$ to $\mathcal{Q}_{\text{events}}$, and **return**.
- **If** $\kappa = \texttt{EXTRINSIC\_CYCLE}$: Push $\texttt{ev\_CyclicDependencyFound}(E_{\text{cycle}}, \text{trace})$ to $\mathcal{Q}_{\text{events}}$ and **return**.

- **Else** ($\mathbb{I}_{\text{compile}}(c') = 1 \land \text{PassRate}_{\text{char}}(c') = 1.0$):
  Invoke $\mathcal{A}_{\text{eval}}^{\text{distill}}(c', \mathcal{B}_{PM}^{(v_i)})$ to verify 100% `Given/When/Then` business rule coverage and compute $\text{Score}_{\text{harden}}(c') \in [0, 1]$.

5. **Reward Computation & Backpropagation (Tier A Local Loop):**
   Compute $R(u') = w_{\text{comp}}\mathbb{I}_{\text{compile}}(c') + \mathbb{I}_{\text{compile}}(c')\big[w_{\text{char}}\text{PassRate}_{\text{char}}(c') + w_{\text{hard}}\text{Score}_{\text{harden}}(c')\big]$.
   Create child node $u' = (c', \mathcal{B}'^{(v_i)})$ in $\mathcal{T}^{(v_i)}$ and backpropagate $R(u')$ along the path to root, updating $N(\cdot)$ and $\bar{R}(\cdot)$.

- **If** $R(u') = 1.0$ **then:** Push $\texttt{ev\_ModuleHardened}(v_i, c', \mathcal{B}_{CT}^{(v_i)})$ to $\mathcal{Q}_{\text{events}}$ and **return**.

**Budget Exhaustion Fallback:** If loop terminates at $t = B_{\max}$ without $R = 1.0$, extract highest-reward node $u_{\text{best}} = \arg\max_{u \in \mathcal{T}^{(v_i)}} R(u)$ and push $\texttt{ev\_ContractConflict}(\text{Parent}(v_i), v_i, \Delta_{\text{blocker}}(u_{\text{best}}), \mathcal{B}_{PR}^{(v_i)})$ to $\mathcal{Q}_{\text{events}}$.

---
