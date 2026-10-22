# Statecharts, Pipeline, Guards & ARIADNE

Entry point: [../tech-spec.md](../tech-spec.md). Blackboard schemas and rationale §2.1/2.2/2.6 live in [engine-and-blackboard.md](engine-and-blackboard.md); this file covers control flow, the macro pipeline, guards, the micro-FSM, and the ARIADNE cluster.

## 2.3 Declarative Scatter-Gather: Per-Phase Mechanics

| Macro-Phase / Component | Needs Subagent Fan-Out? | Exact Scatter → Map → Reduce Mechanics on $\mathcal{G}$ |
| :--- | :--- | :--- |
| **$S_0$ Intent & Preflight** | No (Single Agent) | `PreflightProbe` runs deterministic build/test commands; writes harness config and `INTENT` to $\mathcal{G}$. |
| **$S_1$ Assess** | Yes (Directory Shards) | **Scatter**: AST/scc scanner groups repo into directory shards. **Map**: Parallel `ShardAssessor` agents analyze dialects and debt per shard. **Reduce**: `AssessmentSynthesizer` aggregates shard reports into risk matrix. |
| **$S_2$ Map (DAG)** | Hybrid (Static Tool + Targeted Map) | **Scatter**: LSP/Tree-sitter builds explicit call/import graph; flags dynamic files. **Map**: Parallel `ImplicitCouplingScanner` agents inspect flagged files for hidden edges. **Reduce**: Tarjan SCC condensation yields DAG $G=(V,E)$. |
| **$S_3$ Extract Rules** | Yes (DAG Vertex Shards) | **Scatter**: $\mathcal{O}_{\text{engine}}$ partitions $V$ into bounded vertex batches. **Map**: Parallel `RuleMiner` agents extract `Given/When/Then` cards directly to $\mathcal{G}$. **Reduce**: `CrossModuleRuleLinker` resolves `UnresolvedCrossLink` markers across edges. |
| **$S_4$ Brief & Contracts** | Yes (Domain Subgraphs) | **Scatter**: `MacroArchitect` sets global target idioms on $\mathcal{G}$. **Map**: Parallel `ContractStubber` agents generate target interface stubs for $e_{pi} \in E$. **Reduce**: Target compiler verifies that all interface stubs mutually type-check. |
| **$S_5$ Iterative DAG** | Yes (Topological Waves) | Kahn wave dispatch spawns parallel encapsulated ARIADNE clusters over $\text{ReadySet}(\mathcal{G})$. |
| **$S_6$ Verify & Harden** | Yes (Test & SAST Shards) | Parallel `IntegrationVerifier` and `SecurityAuditor` agents dispatched per failing E2E trace or SAST finding. |
| **Stage 2 Critics ($g_{\text{sem}}$)** | Yes (Batch-Sharded Auditing) | $\mathcal{O}_{\text{engine}}$ fans out parallel `ShardCritic` agents (10–15 rule cards or contracts per shard); conjuncts $\bigwedge_{b=1}^B g_{\text{sem}}(\text{batch}_b)$. |

## 2.4 Two-Stage Guard Formulation

Before transitioning out of phase $S_m$, $\mathcal{O}_{\text{engine}}$ evaluates:

$$g(\mathcal{G}) = \underbrace{\Big( \bigwedge_{k=1}^K g_{\text{struct}}\big(\mathcal{G}[\text{shard}_k]\big) \Big)}_{\text{Stage 1: Zero-Token Programmatic Check}} \;\land\; \underbrace{\Big( \bigwedge_{b=1}^{B} g_{\text{sem}}\big(\mathcal{G}[\text{batch}_b]\big) \Big)}_{\text{Stage 2: Sharded Adversarial LLM Critic}}$$

If Stage 1 fails on $\text{shard}_k$, $\mathcal{O}_{\text{engine}}$ immediately re-dispatches only $W_k$ with `ActivationMode = CRITIC_RETRY` and the deterministic traceback. Only when Stage 1 passes across all shards does $\mathcal{O}_{\text{engine}}$ fan out Stage 2: parallel, read-isolated `ShardCritic` agents audit batches of produced artifacts against legacy `file:line` source citations and return a structured `CritiqueRecord`:

```json
{
  "verdict": "PASS" | "REJECT_LOCAL" | "REGRESS_UPSTREAM",
  "violated_invariants": ["..."],
  "remediation_directive": "..."
}
```

In-tree verdict mapping: `CritiqueVerdict::RemediateLocal` re-dispatches the owning batch's shards with `CritiqueRecord.remediation_notes` as delta directive; a second consecutive `RemediateLocal` on the same batch aborts with `MacroEngineError::SemanticRemediationFailed`; `RegressUpstream` regresses to the target upstream state.

## 3. Definition 3: Formal Harel Statechart 7-Tuple ($\mathcal{H}$)

$$\mathcal{H} = \Big( \mathcal{S}, \; \rho, \; \psi, \; \mathcal{E}_{\text{bus}}, \; \mathcal{G}_{\text{guards}}, \; \mathcal{T}_{\text{trans}}, \; \Theta_{H^*} \Big)$$

- **Hierarchy Function ($\rho: \mathcal{S} \to 2^{\mathcal{S}}$)**: $\rho(\text{Root}) = \{S_0, S_1, S_2, S_3, S_4, S_5, S_6\}$.
- **Orthogonality Function ($\psi: \mathcal{S} \to \{\texttt{XOR}, \texttt{AND}\}$)**:
  - For parallel discovery states $S_m \in \{S_1, S_2, S_3, S_4, S_6\}$, substate $S_m.2$ has $\psi(S_m.2) = \texttt{AND}$ over $K$ stateless workers.
  - For $S_5$, $\psi(S_5) = \texttt{AND}$ over $|V|$ concurrent module regions $R(v_1) \parallel \dots \parallel R(v_{|V|)}$, each executing an isomorphic 6-state micro-FSM $\rho(R(v_i)) = \{\texttt{M0}, \dots, \texttt{M5}\}$.
- **Guarded Broadcast Transition Relation ($\mathcal{T}_{\text{trans}}$)**: $s_{\text{src}} \xrightarrow{\; \text{ev} \; [g(\mathcal{G})] \; / \; \alpha(\mathcal{G}) \;} s_{\text{dst}}$ where $\text{ev} \in \mathcal{E}_{\text{bus}}$ is popped from $\mathcal{Q}_{\text{events}}$, $g(\mathcal{G})$ is the short-circuiting guard predicate, and $\alpha(\mathcal{G})$ is the transition action.
- **Deep History Pseudostate ($\Theta_{H^*}$)**: Upon any macro-regression exiting $S_5$ to an earlier state $S_m$ ($m \in \{2, 3, 4\}$), $\mathcal{O}_{\text{engine}}$ records $\Theta_{H^*}(S_5) \leftarrow \mathbf{M}$. Re-entering $S_5$ via the $H^*$ connector restores every unaffected region $R(v_i)$ directly to its prior state $M(v_i)$, preserving verified `M4_Hardened` modules without re-execution.

## 4. Definition 4: Dynamic Context Hydration via `TaskActivationEnvelope`

Every transition action $\alpha(\mathcal{G})$ invokes a deterministic **`ContextAssembler`** (crate `werðan/libs/global/agents/workers`, which also owns `TaskActivationEnvelope` $\mathcal{E}_{\text{act}}$ and the stateless `PhaseWorker` contract):

$$\text{Hydrate}\big(\text{AgentRole}, \; \mu_{\text{mode}}, \; \text{target\_ref}, \; \mathcal{G}\big) \longrightarrow \mathcal{E}_{\text{act}} = \Big\langle \mu_{\text{mode}}, \; \mathcal{G}_{\text{slice}}, \; \Delta_{\text{directive}}, \; \mathcal{T}_{\text{tools}} \Big\rangle$$

where $\mu_{\text{mode}} \in \{\texttt{INITIAL}, \texttt{CRITIC\_RETRY}, \texttt{WARM\_START\_REGRESSION}\}$ selects the system prompt template, $\mathcal{G}_{\text{slice}}$ queries only the bounded 1-hop subgraph relevant to `target_ref`, $\Delta_{\text{directive}}$ injects the failure critique or contract conflict trace, and $\mathcal{T}_{\text{tools}}$ binds read-only deterministic AST/LSP navigation tools.

## 5. Macro-Level Orchestration: The 7-Phase Pipeline

| State | Phase Name | Scatter-Gather Topology ($S_m.1 \to S_m.3$) | Structural Guard ($g_{\text{struct}}$) | Adversarial Semantic Guard ($g_{\text{sem}}$) |
| :--- | :--- | :--- | :--- | :--- |
| **$S_0$** | **Intent & Preflight** | Single `PreflightProbe` agent (no sharding). | Legacy build compiles; baseline test harness executes; `INTENT` schema valid. | N/A (Empirical execution gate). |
| **$S_1$** | **Assess** | **Scatter**: Directory shards. **Map**: `ShardAssessor`. **Reduce**: `AssessmentSynthesizer`. | 100% file inventory coverage against `scc`/`cloc`; valid metric ranges. | `AssessmentCritic` verifies every risk claim cites valid legacy files; flags unmapped dialects. |
| **$S_2$** | **Map (DAG)** | **Scatter**: Static LSP/Tree-sitter graph, flags dynamic files. **Map**: `ImplicitCouplingScanner`. **Reduce**: Tarjan SCC condensation. | $G=(V,E)$ strictly acyclic; all entry points and files belong to exactly one $v_i \in V$. | `TopologyCritic` audits flagged reflection, shared DB tables, and event buses for missed implicit edges. |
| **$S_3$** | **Extract Rules** | **Scatter**: Vertex batches from $V$. **Map**: `RuleMiner`. **Reduce**: `CrossModuleRuleLinker`. | Every rule card contains valid `Given/When/Then` clauses, `legacy/file:line` pointers, and `Observed` vs. `Inferred` tags. | `RuleAuditor` re-verifies every rule card against legacy AST snippets for completeness and hallucination. |
| **$S_4$** | **Brief & Contracts** | **Scatter**: `MacroArchitect` idioms; edges sharded by domain. **Map**: `ContractStubber`. **Reduce**: Target compiler checks stub compatibility. | Every edge $e_{pi} \in E$ has a compilable target interface stub in $\mathcal{C}_{\text{contracts}}$. | `ArchitectureCritic` audits target interfaces against $\mathcal{R}_{\text{rules}}$ to prevent over-engineering or unrepresentable domain invariants. |
| **$S_5$** | **Iterative DAG** | **Composite AND-State**: Topological Wave Scheduler over $R(v_1) \parallel \dots \parallel R(v_{|V|)}$ running encapsulated ARIADNE clusters. | $\forall v_i \in V, M(v_i) = \texttt{M4\_Hardened}$ with composite reward $R(c_i^\star) = 1.0$. | Enforced intra-cluster via $\mathcal{A}_{\text{eval}}^{\text{distill}}$ rule-parity verification prior to entering `M4`. |
| **$S_6$** | **System Verify** | **Scatter**: E2E test suites & SAST scanners. **Map**: `IntegrationVerifier` & `SecurityAuditor`. **Reduce**: Final parity sign-off generator. | 100% E2E integration test pass rate; zero critical SAST/CVE vulnerabilities. | `ReleaseAuditor` verifies end-to-end compliance with `INTENT` and signs off per-module verdicts. |

## 6. Per-Module Micro-FSM ($\mathcal{M}_{v_i}$)

Each orthogonal region $R(v_i)$ transitions through six micro-states:

```text
M0_Blocked --> M1_ContextHydration      : all upstream parents in M4_Hardened
M1_ContextHydration --> M2_ARIADNE_MCTS : deterministic copy to B_PM (non-LLM)
M2_ARIADNE_MCTS --> M4_Hardened         : reward R(c*) = 1.0
M2_ARIADNE_MCTS --> M3_ContractNegotiate: extrinsic contract blocker
M3_ContractNegotiate --> M0_Blocked     : ev_ContractConflict broadcast
M4_Hardened --> M5_Invalidated          : upstream contract changed
M5_Invalidated --> M2_ARIADNE_MCTS      : warm-start regression (preserve tree)
```

- **`M0_Blocked`**: Waiting for upstream parents $\{v_p \mid (v_p, v_i) \in E\}$ to reach `M4_Hardened`.
- **`M1_ContextHydration` (deterministic, non-LLM)**: Triggered when $v_i \in \text{ReadySet}(\mathcal{G})$; copies $\mathcal{R}_{\text{rules}}(v_i)$ and parent contracts into $\mathcal{B}_{PM}^{(v_i)}$ and transitions immediately to `M2`.
- **`M2_ARIADNE_MCTS`**: Executes the encapsulated ARIADNE cluster. Hardens on $R(c^\star) = 1.0$; negotiates on a proved extrinsic upstream contract conflict; emits a Tier C macro-regression on an undiscovered dependency cycle.
- **`M3_ContractNegotiate`**: Broadcasts `ev_ContractConflict(v_p, v_i, \Delta_{\text{spec}}, \text{trace})` on $\mathcal{G}$ and parks $v_i$ back in `M0_Blocked`.
- **`M4_Hardened`**: Commits $c^\star$ and $\mathcal{B}_{CT}^{(v_i)}$ to $\mathcal{A}_{\text{artifacts}}(v_i)$ and emits `ev_ModuleHardened(v_i)`, triggering `ReadySet` recomputation.
- **`M5_Invalidated` (warm-start regression state)**: Entered when a hardened module's interface is invalidated by a downstream child or parent update. Appends $\Delta_{\text{spec}}$ to $\mathcal{B}_{PM}^{(v_i)}$ while preserving the MCTS search tree $\mathcal{T}^{(v_i)}$, strategy hypotheses $\mathcal{B}_{SH}^{(v_i)}$, and test suite $\mathcal{B}_{CT}^{(v_i)}$, then transitions back to `M2`.

**In-tree** (`werðan/libs/holon/fsm`): pure compile-time typestate `StateMachine<S>` over data-carrying markers `M0Blocked`..`M5Invalidated`. Each marker has inherent impls exposing consuming transitions (`wake_ready`, `hydration_completed`, `into_hardened`, `into_contract_negotiate`, `conflict_dispatched`, `into_invalidated`, `warm_start_ready`, `emit_cycle_regression`), the `FsmHistory` audit trail, and the `ClusterBroadcastEvent` outbox (`drain_events`). Reflection predicates: `state_tag`, `is_blocked`, `is_hardened`, `is_active`, `history`, `activation_mode`. Disallowed transitions are unrepresentable at compile time; the warm-start path returns `M2AriadneMcts` with `ActivationMode::WarmStartRegression`. Consumers (the ARIADNE controller, cucumber harnesses, interactive mock) drive the typed machine directly, mapping `EngineTypestateOutcome` onto `fsm::SolverOutcome`.

## 7. Topological Wave Dispatch (Kahn's Algorithm)

Upon entering $S_5$, $\mathcal{O}_{\text{engine}}$ executes Kahn's algorithm over $G=(V,E)$ to continuously evaluate the unblocked vertex set:

$$\text{ReadySet}(\mathcal{G}) = \Big\{ v_i \in V \;\Big\vert{}\; M(v_i) = \texttt{M0\_Blocked} \;\land\; \forall (v_p, v_i) \in E, \; M(v_p) = \texttt{M4\_Hardened} \Big\}$$

For every $v_i \in \text{ReadySet}(\mathcal{G})$, $\mathcal{O}_{\text{engine}}$ activates an orthogonal region $R(v_i)$ running an isolated ARIADNE cluster.

## 8. Deterministic MCTS Controller ($\mathcal{C}_{\text{MCTS}}^{(v_i)}$)

The controller is deterministic algorithmic code (non-LLM) maintaining search tree $\mathcal{T}^{(v_i)}$ where nodes $u$ represent states $s_t = (c_t, \mathcal{B}_t^{(v_i)})$. Rollout traversal samples child states via temperature-controlled Softmax-UCB:

$$\text{UCB}(u) = \bar{R}(u) + C_{\text{exp}} \sqrt{\frac{\ln(N(v) + 1)}{N(u)}} + \gamma \cdot P_{\text{prior}}(u)$$

$$p(u \mid v) = \frac{\exp\big((\text{UCB}(u) - \max_{w} \text{UCB}(w)) / \tau\big)}{\sum_{w \in \text{Children}(v)} \exp\big((\text{UCB}(w) - \max_{w'} \text{UCB}(w')) / \tau\big)}$$

**In-tree constants** (`werðan/libs/holon/engine/src/types.rs`, `EngineConfig::default()`): $C_{\text{exp}}$ = `ucb_c_exp` = **1.414**, $\gamma$ = `gamma_prior` = **0.50**, $\tau$ = `softmax_temperature` = **0.50**, `generative_temperature` = **0.20**, `max_rollouts` = **16** (rollout budget $B_{\max}$ in state `M2`), `RewardWeights` = compile **0.20** / characterization parity **0.60** / hardening **0.20**.

These values deliberately diverge from `METHODOLOGY.md` ($\tau = 0.35$, $\gamma = 0.15$): the methodology file is read-only ground truth and stays untouched, while the in-tree defaults are calibrated for the local 27B SLM serving stack. See [ADR-0012](../../decisions/0012-ariadne-constant-calibration.md) for the deviation rationale; the `ariadne` section of `migration.yml` exposes overrides, and the daemon currently overrides `max_rollouts` only (`ariadne.engine_budget_per_module`); other constants flow from `EngineConfig::default()`.

## 9. Ephemeral Generative Subagents (Strict Read/Write Isolation)

Each generative holon is one workspace crate `werðan/libs/holon/agents/holon-{name}` exposing a `Holon{name}Agent` namespace: `builder()` assembles the `::rig::agent::Agent` (role preamble, temperature, max tokens) over a caller-supplied `::rig::completion::CompletionModel`; a namespace call drives the prompt on a dedicated blocking thread and parses the domain result. No crate defines its own transport or completion client ([ADR-0006](../../decisions/0006-de-qwen-inference-transport.md), [ADR-0010](../../decisions/0010-holon-modular-architecture.md)). Per-crate max tokens: strategy 4096, codegen 16384, testgen 8192, patch 16384, evaluator roles 512.

- **$\mathcal{A}_{\text{str}}$ (Strategy Analysis, `holon-strategy`)**: `HolonStrategyAgent::generate_strategies(&agent, problem_model)`; invoked when node $u$ lacks unexpanded strategies; reads $\mathcal{B}_{PM}^{(v_i)}$, writes $K_s$ refactoring hypotheses with $P_{\text{prior}}$ clamped to $[0.10, 0.95]$ into $\mathcal{B}_{SH}^{(v_i)}$; unparsable responses yield an empty plan.
- **$\mathcal{A}_{\text{gen}}$ (Code Generation, `holon-codegen`)**: `HolonCodeGenAgent::generate_candidate(&agent, problem_model, hypothesis)`; invoked on uninstantiated hypothesis $h \in \mathcal{B}_{SH}^{(v_i)}$; synthesizes candidate target code $c'$ conforming to $\mathcal{B}_{PM}^{(v_i)}$.
- **$\mathcal{A}_{\text{test}}$ (Adversarial Test Generation, `holon-testgen`)**: `HolonTestGenAgent::generate_characterization_tests(&agent, problem_model)`; reads $\mathcal{B}_{PM}^{(v_i)}$ and legacy reference behavior, **strictly read-isolated from candidate code $c'$** to prevent test weakening or cheat fitting; generates characterization I/O snapshots and property tests into $\mathcal{B}_{CT}^{(v_i)}$.
- **$\mathcal{A}_{\text{patch}}$ (Surgical Code Repair, `holon-patch`)**: `HolonPatchAgent::repair_code(&agent, candidate, fault, problem_model)`; invoked when $0 < R(s_t) < 1$; reads $c_t$ and the localized fault record in $\mathcal{B}_{PR}^{(v_i)}$ to emit minimal diff patch $c''$.
- **$\mathcal{A}_{\text{eval}}^{\text{distill}}$ (Diagnostic Distiller, `holon-evaluator`)**: `HolonEvaluatorAgent::{classifier_builder, auditor_builder}`; `RigDiagnosticDistiller::new(classifier, auditor)` implements the sync `DiagnosticDistiller` trait. Deterministic fallbacks (`DefaultDiagnosticDistiller`) remain the controller default.

## 10. Two-Stage Hybrid Evaluator ($\mathcal{A}_{\text{eval}} = \mathcal{A}_{\text{eval}}^{\text{exec}} \circ \mathcal{A}_{\text{eval}}^{\text{distill}}$)

1. **Stage 3a — Deterministic Verification Harness (non-LLM)**: Compiles $c'$ against parent artifacts, runs LSP type-checking, executes characterization tests $\mathcal{B}_{CT}^{(v_i)}$ via differential test runners, and runs static security linters. In-tree: `TwoStageEvaluator<H, D>` with the `ExecutionHarness` contract; benchmark mode uses `cargo_harness::CargoHarness`, which writes candidate and characterization files into a real target workspace and derives compilation status and $\text{PassRate}_{\text{char}}$ from genuine `cargo check --all-targets` / `cargo test --all-targets -- --format terse` output with configured timeouts (defaults 120s each).
2. **Stage 3b — Diagnostic Distiller & Parity Critic (lightweight LLM)**: On failure, distills raw compiler/test stack traces into a localized fault record in $\mathcal{B}_{PR}^{(v_i)}$ and classifies fault cause $\kappa \in \{\texttt{INTRINSIC}, \texttt{EXTRINSIC\_CONTRACT}, \texttt{EXTRINSIC\_CYCLE}\}$. On pass, audits $c'$ against all `Given/When/Then` rule cards in $\mathcal{B}_{PM}^{(v_i)}$ to compute semantic parity score $\text{Score}_{\text{harden}}(c') \in [0, 1]$.

The daemon supplies the completion model: `OpenAiCompatModel` posts OpenAI-compatible chat completions through Rig's stock `openai` provider, applies the configured token budget and sampling temperature, and returns `CompletionError::ProviderError` when no usable answer exists; `CountingCompletionModel` wraps any model with an atomic request counter for honesty auditing. Inference failures surface as `::core::compile_error!` sentinel candidates so the evaluator records a genuine fault record; no silent template fallback exists.

Controller generics: `ClusterEngine<M, H, D>` and `AriadneSolver<M, H, D>` assemble through bon builders over explicitly constructed components (`config`, `blackboard`, `evaluator`, `agents`, `harness`, `distiller`); `SubagentPool::builder()` assembles the four generative Rig agents passed in as `Arc<::rig::agent::Agent<M>>` role handles. `ClusterEngine::solve` consumes the module's `StateMachine<M2AriadneMcts>` directly and returns `EngineTypestateOutcome` variants (`Hardened(M4)`, `ContractConflict(M3)`, `CycleFound{machine, cycle_edges, proof_trace}`, `BudgetExhausted{machine, best_candidate, best_reward, blocker_trace}`).

## 11. Gated Hierarchical Reward Function $R(s_t)$

$$R(s_t) = w_{\text{comp}} \cdot \mathbb{I}_{\text{compile}}(c_t) + \mathbb{I}_{\text{compile}}(c_t) \cdot \Big[ w_{\text{char}} \cdot \text{PassRate}_{\text{char}}\big(c_t, \mathcal{B}_{CT}^{(v_i)}\big) + w_{\text{hard}} \cdot \text{Score}_{\text{harden}}\big(c_t, \mathcal{B}_{PM}^{(v_i)}\big) \Big]$$

Default weights ($w_{\text{comp}} + w_{\text{char}} + w_{\text{hard}} = 1.0$): $w_{\text{comp}} = 0.20$, $w_{\text{char}} = 0.60$, $w_{\text{hard}} = 0.20$. Uncompilable code strictly receives $R(s_t) = 0.0$; achieving $R(s_t) = 1.0$ is the necessary and sufficient condition for module hardening.

## 12. Multi-Tier Regression Calculus and Warm-Start Recovery

Discovered anomalies are classified and resolved across three strict locality tiers:

| Tier | Fault Class ($\kappa$) | Scope & Traffic | Action & Recovery Protocol |
| :--- | :--- | :--- | :--- |
| **Tier A** | `INTRINSIC` | Intra-module only (zero global traffic) | Handled entirely inside $\mathcal{C}_{\text{MCTS}}^{(v_i)}$: $R(s_t) < 1.0$ is backpropagated up $\mathcal{T}^{(v_i)}$, shifting subsequent rollouts to $\mathcal{A}_{\text{patch}}$ using $\mathcal{B}_{PR}^{(v_i)}$ or branching to an alternate hypothesis in $\mathcal{B}_{SH}^{(v_i)}$. $\mathcal{O}$ is never interrupted. |
| **Tier B** | `EXTRINSIC_CONTRACT` | Cross-module 1-hop via broadcast event bus | Downstream $v_i$ transitions `M2` → `M3` and broadcasts `ev_ContractConflict(v_p, v_i, \Delta_{\text{spec}}, \text{trace})`. $\mathcal{O}_{\text{engine}}$ verifies symbol reference and $\mathcal{O}_{\text{arbiter}}$ confirms validity; $\mathcal{O}_{\text{engine}}$ updates $\mathcal{C}_{\text{contracts}}$, parks $v_i \to$ `M0`, and invalidates parent $v_p \to$ `M5_Invalidated` → `M2` with warm-started tree. |
| **Tier C** | `EXTRINSIC_CYCLE` | Repository-wide macro-regression with Deep History ($H^*$) | Emits `ev_CyclicDependencyFound`. Tier C accepts a reported cycle only when the proof trace is nonempty and every reported edge traverses known DAG vertices ($g_{\text{struct}}(E_{\text{cycle}})$ validation), then runs arbiter adjudication. $\mathcal{O}_{\text{engine}}$ parks mid-flight modules (`M1`/`M2`/`M3`) into `M5Invalidated`, checkpoints $\Theta_{H^*}(S_5) \leftarrow \mathbf{M}$, transitions $S_5 \to S_2$, condenses cyclic vertices via Tarjan's SCC into composite unit $v_{\text{SCC}}$, updates rules ($S_3$) and stubs ($S_4$), and re-enters $S_5$ via $H^*$, restoring unaffected modules without re-execution. |

Budget exhaustion fallback: if the rollout loop reaches $B_{\max}$ without $R = 1.0$, the controller extracts the highest-reward node $u_{\text{best}} = \arg\max_{u \in \mathcal{T}^{(v_i)}} R(u)$ and pushes `ev_ContractConflict(Parent(v_i), v_i, \Delta_{\text{blocker}}(u_{\text{best}}), \mathcal{B}_{PR})` to $\mathcal{Q}_{\text{events}}$.
