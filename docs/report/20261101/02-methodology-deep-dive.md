# 02 — Formal Architecture of the werðan (THESEUS) Modernization Engine

## 1. Mathematical System Formulation: The Two-Tier Blackboard and Harel Statechart

The architecture of **werðan** (the THESEUS framework) resolves the failure modes of static pipelines and conversational hierarchies by decomposing modernization into two mathematically rigorous control structures: a **Two-Tier Blackboard** for data isolation and a **Harel Statechart** with orthogonal concurrent regions for deterministic workflow execution.

---

### Definition 1: Global Repository Blackboard ($\mathcal{G}$) and State Vector ($\Sigma(\mathcal{G})$)

The Global Blackboard $\mathcal{G}$ is a persistent, schema-validated tuple:

$$\mathcal{G} = \Big\langle \mathcal{I}_{\text{inv}}, \; G=(V,E), \; \mathcal{R}_{\text{rules}}, \; \mathcal{C}_{\text{contracts}}, \; \mathcal{A}_{\text{artifacts}}, \; \Sigma(\mathcal{G}) \Big\rangle$$

where:
* $\mathcal{I}_{\text{inv}}$ is the repository file inventory, source/target dialect profiles, and verified execution harness configuration $\mathcal{H}_{\text{exec}}$.
* $G = (V, E)$ is the **Condensed Dependency Directed Acyclic Graph (DAG)**, where each vertex $v_i \in V$ represents an atomic translation unit (a single module or a collapsed Strongly Connected Component of mutually dependent files), and each directed edge $e_{pi} = (v_p, v_i) \in E$ denotes that downstream unit $v_i$ depends on symbols exported by upstream parent $v_p$.
* $\mathcal{R}_{\text{rules}}: V \to 2^{\text{RuleCard}}$ maps each vertex to its verified `Given/When/Then` business rule cards with exact `file:line` legacy citations.
* $\mathcal{C}_{\text{contracts}}: E \to \text{InterfaceSpec}$ maps each dependency edge to its target-language type signature, export obligations, and behavioral preconditions.
* $\mathcal{A}_{\text{artifacts}}: V \to (\text{TargetAST} \times \text{TestSuite})$ stores hardened target modules.
* $\Sigma(\mathcal{G})$ is the transactionally persisted state vector:
  $$\Sigma(\mathcal{G}) = \Big\langle S_{\text{macro}}, \; \mathbf{M} = \big[M(v_1), \dots, M(v_{|V|})\big], \; \Theta_{H^*}, \; \mathcal{Q}_{\text{events}} \Big\rangle$$
  comprising active macro-state $S_{\text{macro}}$, module micro-state vector $\mathbf{M}$, deep-history snapshot map $\Theta_{H^*}$, and FIFO asynchronous event bus $\mathcal{Q}_{\text{events}}$.

---

### Definition 2: Per-Module 4-Slice Local Blackboard ($\mathcal{B}^{(v_i)}$)

To prevent the context window saturation endemic to conversational agents, each active module vertex $v_i \in V$ owns an isolated **Local Blackboard** $\mathcal{B}^{(v_i)}$ invisible to other modules:

$$\mathcal{B}^{(v_i)} = \Big( \mathcal{B}_{PM}^{(v_i)}, \; \mathcal{B}_{SH}^{(v_i)}, \; \mathcal{B}_{CT}^{(v_i)}, \; \mathcal{B}_{PR}^{(v_i)} \Big)$$

1. **Problem Model ($\mathcal{B}_{PM}^{(v_i)}$)**: Hydrated deterministically from $\mathcal{G}$ upon activation. Contains legacy AST slices, verified rule cards $\mathcal{R}_{\text{rules}}(v_i)$, upstream parent contracts $\{\mathcal{C}_{\text{contracts}}(v_p, v_i)\}$, and downstream export obligations.
2. **Strategy Hypotheses ($\mathcal{B}_{SH}^{(v_i)}$)**: Competing architectural refactoring hypotheses $h_k$ (e.g., procedural port vs. state-pattern encapsulation) with prior utility estimates $P_{\text{prior}}$ and instantiation metadata.
3. **Counterexamples & Characterization Tests ($\mathcal{B}_{CT}^{(v_i)}$)**: Black-box I/O snapshots, differential test fixtures, and property-based test suites synthesized from legacy execution traces.
4. **Patch & Fault Records ($\mathcal{B}_{PR}^{(v_i)}$)**: Distilled compiler and test fault localizations indexing failed rollouts.

---

### Definition 3: Harel Statechart Formalism ($\mathcal{H}$)

Control flow in the Macro-Orchestrator $\mathcal{O}_{\text{engine}}$ is governed by the Harel Statechart tuple:

$$\mathcal{H} = \Big( \mathcal{S}, \; \rho, \; \psi, \; \mathcal{E}_{\text{bus}}, \; \mathcal{G}_{\text{guards}}, \; \mathcal{T}_{\text{trans}}, \; \Theta_{H^*} \Big)$$

* **State Hierarchy ($\rho: \mathcal{S} \to 2^{\mathcal{S}}$)**: Top-level root state decomposes into seven sequential XOR Macro-States:
  $$\rho(\text{Root}) = \{S_0, S_1, S_2, S_3, S_4, S_5, S_6\}$$
* **Orthogonality ($\psi: \mathcal{S} \to \{\texttt{XOR}, \texttt{AND}\}$)**:
  * For global discovery states $S_m \in \{S_1, S_2, S_3, S_4, S_6\}$, internal substate $S_m.2$ has $\psi(S_m.2) = \texttt{AND}$ over $K$ stateless shard workers: $S_m.2 = \bigparallel_{k=1}^K W(\text{shard}_k)$.
  * For DAG Transformation $S_5$, $\psi(S_5) = \texttt{AND}$ over $|V|$ concurrent module regions:
    $$S_5 = R(v_1) \parallel R(v_2) \parallel \dots \parallel R(v_{|V|})$$
    where each orthogonal region $R(v_i)$ executes an isomorphic 6-state Micro-FSM $\rho(R(v_i)) = \{M_0, M_1, M_2, M_3, M_4, M_5\}$.
* **State Complexity Reduction**:
  By modeling module transformation as an orthogonal product rather than a flat FSM, the composite state complexity drops exponentially:
  $$|\Sigma_{\text{flat}}| = O(m^{|V|}) \quad \Longrightarrow \quad |\Sigma_{\text{Harel}}| = O(m \cdot |V|)$$
* **Guarded Transition Relation ($\mathcal{T}_{\text{trans}}$)**:
  Every transition takes the canonical form:
  $$s_{\text{src}} \xrightarrow{\; \text{ev} \; [g(\mathcal{G})] \; / \; \alpha(\mathcal{G}) \;} s_{\text{dst}}$$
  where $\text{ev} \in \mathcal{E}_{\text{bus}}$ is popped from $\mathcal{Q}_{\text{events}}$, $g(\mathcal{G})$ is the two-stage guard predicate, and $\alpha(\mathcal{G})$ updates $\Sigma(\mathcal{G})$.
* **Deep History Pseudostate ($\Theta_{H^*}$)**:
  Upon any macro-regression exiting $S_5$ to an earlier state $S_m$ ($m \in \{2, 3, 4\}$), $\mathcal{O}_{\text{engine}}$ checkpoints $\Theta_{H^*}(S_5) \leftarrow \mathbf{M}$. Re-entering $S_5$ via $H^*$ restores all unaffected regions $R(v_k)$ directly to their verified state $M(v_k)$, preserving `M4_Hardened` modules without re-execution.

---

### Definition 4: Dynamic Context Hydration via Task Activation Envelope

Because $\mathcal{O}_{\text{engine}}$ is algorithmic code while LLM workers require rich domain context, all subagent dispatches invoke a deterministic `ContextAssembler`:

$$\text{Hydrate}\big(\text{AgentRole}, \; \text{mode}, \; \text{target\_ref}, \; \mathcal{G}\big) \longrightarrow \mathcal{E}_{\text{act}} = \Big\langle \mu_{\text{mode}}, \; \mathcal{G}_{\text{slice}}, \; \Delta_{\text{directive}}, \; \mathcal{T}_{\text{tools}} \Big\rangle$$

where $\mu_{\text{mode}} \in \{\texttt{INITIAL}, \texttt{CRITIC\_RETRY}, \texttt{WARM\_START\_REGRESSION}\}$ selects prompt templates, $\mathcal{G}_{\text{slice}}$ limits visibility to the bounded 1-hop subgraph of `target_ref`, $\Delta_{\text{directive}}$ injects the failure critique or contract conflict trace, and $\mathcal{T}_{\text{tools}}$ binds deterministic read-only AST navigation tools.

---

## 2. Macro-Level Orchestration: The 7-Phase Pipeline & Declarative Scatter-Gather

```
 [S0: Intent & Preflight] ──> Baseline verification, schema validation
           │
 [S1: Assess] ─────────────> Sharded directory metrics (g_struct & g_sem)
           │
 [S2: Map (DAG)] ──────────> LSP graph, cycle condensation (Tarjan's SCC)
           │
 [S3: Extract Rules] ──────> Pre-DAG Given/When/Then cards with legacy citations
           │
 [S4: Reimagine Contracts] ─> Cross-module target interface stubs
           │
 [S5: Iterative Transform] ─> Kahn wave dispatch over orthogonal ARIADNE MCTS nodes
           │                     └── Multi-Tier Regression (Tier A / B / C)
 [S6: System Verify] ──────> End-to-end integration parity & release sign-off
```

### 1. Declarative Scatter-Gather Sub-Statecharts ($D_{\max} = 1$)

For any macro-state $S_m \in \{S_1, S_2, S_3, S_4, S_6\}$ operating over a repository, execution proceeds through three internal substates:
1. **$S_m.1$ (`Scatter` / Partitioning)**: A deterministic partitioner divides the phase workload into $K$ bounded shards $\{\text{shard}_1, \dots, \text{shard}_K\}$ such that $\text{Tokens}(\text{shard}_k) \le L_{\max}$, writing a `WorkManifest` to $\mathcal{G}$.
2. **$S_m.2$ (`Parallel Map`)**: $\mathcal{O}_{\text{engine}}$ spawns $K$ parallel, stateless Map subagents $W_1, \dots, W_K$. Workers read only $\mathcal{G}[\text{shard}_k]$. If a worker encounters an external dependency, it emits a declarative tag `UnresolvedCrossLink(src, tgt_symbol)` rather than spawning child agents. Each worker writes output to $\mathcal{G}[\text{shard}_k]$, emits $\texttt{ev\_ShardDone}(k)$, and terminates.
3. **$S_m.3$ (`Reduce` / Linker)**: Once all shards complete, a lightweight Linker agent synthesizes cross-module links across the exported headers.

### 2. Two-Stage Hybrid Phase Guards ($g = g_{\text{struct}} \land g_{\text{sem}}$)

Before transitioning between macro-phases, $\mathcal{O}_{\text{engine}}$ evaluates:

$$g(\mathcal{G}) = \underbrace{\Big( \bigwedge_{k=1}^K g_{\text{struct}}\big(\mathcal{G}[\text{shard}_k]\big) \Big)}_{\text{Stage 1: Zero-Token Programmatic Check}} \;\land\; \underbrace{\Big( \bigwedge_{b=1}^{B} g_{\text{sem}}\big(\mathcal{G}[\text{batch}_b]\big) \Big)}_{\text{Stage 2: Sharded Adversarial LLM Critic}}$$

If Stage 1 fails on $\text{shard}_k$, $\mathcal{O}_{\text{engine}}$ immediately re-dispatches only worker $W_k$ with the deterministic traceback. Only when Stage 1 passes across all shards does Stage 2 invoke read-isolated **Semantic Critic** subagents (`ShardCritic`) to audit outputs against legacy source citations.

---

## 3. Topological Wave Scheduling & Encapsulated ARIADNE MCTS Nodes

Upon entering Composite State $S_5$, $\mathcal{O}_{\text{engine}}$ activates its **Topological Wave Scheduler**, an algorithmic subroutine based on Kahn's algorithm that continuously evaluates the set of unblocked vertices:

$$\text{ReadySet}(\mathcal{G}) = \Big\{ v_i \in V \;\Big|\; M(v_i) = \texttt{M0\_Blocked} \;\land\; \forall (v_p, v_i) \in E, \; M(v_p) = \texttt{M4\_Hardened} \Big\}$$

### 1. Per-Module Micro-FSM ($M_0 \dots M_5$)

Each orthogonal region $R(v_i)$ transitions through six micro-states:

* **`M0_Blocked`**: Awaiting upstream parents to reach `M4_Hardened`.
* **`M1_ContextHydration` (Deterministic)**: Triggered when $v_i \in \text{ReadySet}(\mathcal{G})$. Copies rule cards $\mathcal{R}_{\text{rules}}(v_i)$, parent contracts $\{\mathcal{C}(v_p, v_i)\}$, and companion header self-contracts $\mathcal{C}(v_i, v_i)$ into $\mathcal{B}_{PM}^{(v_i)}$, transitioning immediately to `M2`.
* **`M2_ARIADNE_MCTS`**: Executes the encapsulated MCTS solver. Transitions to `M4_Hardened` if candidate $c^\star$ achieves composite reward $R(c^\star) = 1.0$; transitions to `M3_ContractNegotiate` if an extrinsic upstream contract conflict is proved.
* **`M3_ContractNegotiate`**: Emits broadcast event $\texttt{ev\_ContractConflict}(v_p, v_i, \Delta_{\text{spec}}, \text{trace})$ to $\mathcal{Q}_{\text{events}}$ and parks $v_i$ back in `M0_Blocked`.
* **`M4_Hardened`**: Commits verified target code $c^\star$ and test suite $\mathcal{B}_{CT}^{(v_i)}$ to $\mathcal{A}_{\text{artifacts}}(v_i)$ on $\mathcal{G}$, emitting $\texttt{ev\_ModuleHardened}(v_i)$ to recompute $\text{ReadySet}(\mathcal{G})$.
* **`M5_Invalidated` (Warm-Start Regression)**: Entered when a hardened module receives a validated contract conflict from a downstream child. Deterministically appends $\Delta_{\text{spec}}$ to $\mathcal{B}_{PM}^{(v_i)}$ **while preserving** search tree $\mathcal{T}^{(v_i)}$, hypotheses $\mathcal{B}_{SH}^{(v_i)}$, and test suite $\mathcal{B}_{CT}^{(v_i)}$, transitioning back to `M2`.

---

### 2. Internal Architecture of the Encapsulated ARIADNE Cluster (`M2`)

Inside state `M2`, module $v_i$ is transformed by an isolated cluster comprising a **Deterministic MCTS Controller ($\mathcal{C}_{\text{MCTS}}^{(v_i)}$)**, **4 Generative LLM Subagents**, and a **Two-Stage Hybrid Evaluator ($\mathcal{A}_{\text{eval}}$)**:

```
               ┌─────────────────────────────────────────┐
               │    Deterministic MCTS Controller        │
               │   (Softmax-UCB Node Selection, τ, γ)    │
               └────────────────────┬────────────────────┘
                                    │
         ┌──────────────────────────┼──────────────────────────┐
         │                          │                          │
         ▼                          ▼                          ▼
 ┌───────────────┐          ┌───────────────┐          ┌───────────────┐
 │ A_str         │          │ A_gen         │          │ A_patch       │
 │ Strategy      │          │ Target Code   │          │ Surgical      │
 │ Analysis      │          │ Synthesis     │          │ Repair        │
 └───────┬───────┘          └───────┬───────┘          └───────┬───────┘
         │                          │                          │
         └──────────────────────────┼──────────────────────────┘
                                    │
                                    ▼
                ┌───────────────────────────────────────┐
                │        Two-Stage Hybrid Evaluator     │
                │ 3a. A_exec  : Deterministic test/check│
                │ 3b. A_distill: Fault classification  │
                └───────────────────┬───────────────────┘
                                    │
                        ┌───────────┴───────────┐
                        │ Gated Hierarchical    │
                        │ Reward Calculation    │
                        │ R(s_t) -> Backprop    │
                        └───────────────────────┘
```

1. **Deterministic Softmax-UCB Controller ($\mathcal{C}_{\text{MCTS}}^{(v_i)}$)**:
   Traverses search tree $\mathcal{T}^{(v_i)}$ where each node represents state $s_t = (c_t, \mathcal{B}_t^{(v_i)})$. Computes Upper Confidence Bound (UCB) scores:
   $$\text{UCB}(u) = \bar{R}(u) + C_{\text{exp}} \sqrt{\frac{\ln(N(v) + 1)}{N(u)}} + \gamma \cdot P_{\text{prior}}(u)$$
   To avoid premature greedy convergence, child selection samples from a temperature-controlled softmax:
   $$p(u \mid v) = \frac{\exp\big((\text{UCB}(u) - \max_w \text{UCB}(w)) / \tau\big)}{\sum_{w \in \text{Children}(v)} \exp\big((\text{UCB}(w) - \max_{w'} \text{UCB}(w')) / \tau\big)}$$

2. **State-Conditioned Subagent Expansion**:
   * **$\mathcal{A}_{\text{str}}$ (Strategy Analysis)**: Invoked when node $u$ lacks unexpanded strategy branches; reads $\mathcal{B}_{PM}^{(v_i)}$ and writes $K_s$ refactoring hypotheses with priors $P_{\text{prior}}$ to $\mathcal{B}_{SH}^{(v_i)}$.
   * **$\mathcal{A}_{\text{gen}}$ (Code Generation)**: Invoked on an uninstantiated hypothesis $h \in \mathcal{B}_{SH}^{(v_i)}$; synthesizes candidate target code $c'$.
   * **$\mathcal{A}_{\text{test}}$ (Adversarial Test Generation)**: Reads $\mathcal{B}_{PM}^{(v_i)}$ and legacy behavior (**strictly read-isolated from candidate $c'$**) to synthesize characterization I/O snapshots and property-based tests into $\mathcal{B}_{CT}^{(v_i)}$.
   * **$\mathcal{A}_{\text{patch}}$ (Surgical Code Repair)**: Invoked when $c_t$ fails verification ($0 < R(s_t) < 1$); reads $c_t$ and the localized fault record in $\mathcal{B}_{PR}^{(v_i)}$ to emit a minimal diff patch $c''$.

3. **Two-Stage Hybrid Evaluator ($\mathcal{A}_{\text{eval}} = \mathcal{A}_{\text{eval}}^{\text{exec}} \circ \mathcal{A}_{\text{eval}}^{\text{distill}}$)**:
   * **Stage 3a (Deterministic Harness $\mathcal{A}_{\text{eval}}^{\text{exec}}$)**: Compiles $c'$ against parent artifacts, runs type checking, and executes characterization suite $\mathcal{B}_{CT}^{(v_i)}$.
   * **Stage 3b (Diagnostic Distiller $\mathcal{A}_{\text{eval}}^{\text{distill}}$)**: On failure, distills compiler/test traces into a localized fault record in $\mathcal{B}_{PR}^{(v_i)}$ and classifies cause $\kappa \in \{\texttt{INTRINSIC}, \texttt{EXTRINSIC\_CONTRACT}, \texttt{EXTRINSIC\_CYCLE}\}$.

4. **Gated Hierarchical Reward Function ($R(s_t)$)**:
   The scalar reward backpropagated up $\mathcal{T}^{(v_i)}$ strictly gates uncompilable code to zero credit:
   $$R(s_t) = w_{\text{comp}} \cdot \mathbb{I}_{\text{compile}}(c_t) + \mathbb{I}_{\text{compile}}(c_t) \cdot \Big[ w_{\text{char}} \cdot \text{PassRate}_{\text{char}}\big(c_t, \mathcal{B}_{CT}^{(v_i)}\big) + w_{\text{hard}} \cdot \text{Score}_{\text{harden}}\big(c_t, \mathcal{B}_{PM}^{(v_i)}\big) \Big]$$
   where $w_{\text{comp}} + w_{\text{char}} + w_{\text{hard}} = 1.0$ (default $0.20, 0.60, 0.20$).

---

## 4. Multi-Tier Regression Calculus & Recovery Boundaries

Discovered anomalies are classified and contained to the smallest possible locality tier:

```
 [Fault Detected during M2 Evaluation]
                 │
                 ├── (κ = INTRINSIC) ───────────> Tier A: Intra-Module MCTS Backtrack
                 │                                (0 global traffic; update UCB, invoke A_patch)
                 │
                 ├── (κ = EXTRINSIC_CONTRACT) ──> Tier B: Cross-Module Micro-Regression
                 │                                (ev_ContractConflict; parent M4 -> M5 -> M2)
                 │
                 └── (κ = EXTRINSIC_CYCLE) ─────> Tier C: Macro-Architectural Regression
                                                  (Deep History H*; Tarjan SCC condensation in S2)
```

1. **Tier A — Intra-Module MCTS Backtrack ($\kappa = \texttt{INTRINSIC}$, Zero Global Traffic)**:
   Recovery is handled entirely inside $\mathcal{C}_{\text{MCTS}}^{(v_i)}$. Reward $R(s_t)$ is backpropagated up $\mathcal{T}^{(v_i)}$, shifting subsequent rollouts to invoke $\mathcal{A}_{\text{patch}}$ or branch to alternative refactoring hypotheses in $\mathcal{B}_{SH}^{(v_i)}$. Macro-orchestrator $\mathcal{O}$ is never interrupted.
2. **Tier B — Cross-Module Micro-Regression ($\kappa = \texttt{EXTRINSIC\_CONTRACT}$)**:
   When downstream module $v_i$ proves that parent $v_p$'s exported contract $\mathcal{C}(v_p, v_i)$ is deficient, $R(v_i)$ broadcasts $\texttt{ev\_ContractConflict}(v_p, v_i, \Delta_{\text{spec}}, \text{trace})$. $\mathcal{O}_{\text{arbiter}}$ validates the conflict. Upon validation, $\mathcal{O}_{\text{engine}}$ updates $\mathcal{C}(v_p, v_i)$, parks $M(v_i) \leftarrow \texttt{M0\_Blocked}$, and fires the broadcast transition:
   $$\texttt{M4\_Hardened} \xrightarrow{\; \texttt{ev\_ContractConflict}(v_p, v_i, \Delta_{\text{spec}}) \; [\text{ArbiterValid}] \;} \texttt{M5\_Invalidated} \longrightarrow \texttt{M2\_ARIADNE\_MCTS}$$
   **Warm-Start Efficiency**: Because $\mathcal{B}_{SH}^{(v_p)}$, $\mathcal{B}_{CT}^{(v_p)}$, and prior winning node $c_p^\star \in \mathcal{T}^{(v_p)}$ are preserved, $R(v_p)$ repairs its interface in a few rollouts without re-discovering its core logic.
3. **Tier C — Macro-Architectural Regression with Deep History $H^*$ ($\kappa = \texttt{EXTRINSIC\_CYCLE}$)**:
   When an undocumented dependency cycle $(v_i \to \dots \to v_p \to v_i)$ is discovered via dynamic linkage, $\mathcal{O}_{\text{engine}}$ checkpoints $\Theta_{H^*}(S_5) \leftarrow \mathbf{M}$, transitions $S_5 \to S_2$ (`Map`), and runs Tarjan's SCC algorithm to condense the cyclic vertices into a single composite vertex $v_{\text{SCC}} \in V$. Finally, $\mathcal{O}_{\text{engine}}$ resumes via $H^*(S_5)$: all unaffected vertices $v_k \notin v_{\text{SCC}}$ are immediately restored to their checkpointed states $\Theta_{H^*}(S_5)[v_k]$ without re-execution.
