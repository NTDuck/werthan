# Engine & Blackboard Architecture

Entry point: [../tech-spec.md](../tech-spec.md). This file holds the design rationale §2.1–2.6 (stable numbering cited by `werðan/libs/global/blackboard/src/state_vector.rs:54`), the global blackboard schema, and the global-side crates.

## 2. Architectural Design Decisions and Theoretical Justifications

### 2.1 Decoupling Deterministic Control Topology from Bounded Reasoning Context
- **Failure Mode**: In manager-worker systems (e.g. GVS5H) and conversational hierarchies, an LLM orchestrator maintains workflow state in its prompt history and decides agent transitions via token generation. Over long horizons, this induces *state hallucination* and *context window saturation*, particularly in Small Language Models (SLMs).
- **Design Decision**: Partition the Macro-Orchestrator $\mathcal{O} = (\mathcal{O}_{\text{engine}}, \mathcal{O}_{\text{arbiter}})$ into a **deterministic statechart runtime engine** ($\mathcal{O}_{\text{engine}}$, programmatic Rust control code) and an on-demand, stateless **LLM Arbiter** ($\mathcal{O}_{\text{arbiter}}$) invoked exclusively for semantic ambiguity triage. All worker agents are **stateless, ephemeral functions** that never poll or maintain persistent daemon sessions; they are instantiated on demand via push-based context hydration from $\mathcal{O}_{\text{engine}}$ and terminate immediately upon committing artifacts.

### 2.2 Harel Statecharts over Flat FSMs and Static Pipelines
- **Failure Mode**: Static feed-forward pipelines (e.g. ReCodeAgent) lack backward topological edges to recover when downstream validation invalidates upstream plans. Conversely, placing a flat Finite State Machine (FSM) at the root of an agent hierarchy forces lockstep global phase rollbacks: tracking $|V|$ modules across $m$ module states requires $O(m^{|V|})$ composite states.
- **Design Decision**: Formalize control via a **Harel Statechart** $\mathcal{H}$ combining **XOR Macro-Superstates** (for repository-wide phases), **Orthogonal `AND`-Regions** (reducing module state complexity from $O(m^{|V|})$ to $O(m \cdot |V|)$), **Broadcast Event Calculus** (for surgical $O(1)$-hop cross-module regressions), and **Deep History ($H^*$)** (preserving verified modules across macro-regressions).

### 2.3 Declarative Scatter-Gather ($D_{\max}=1$) over Autonomous Recursive Subagent Spawning
- **Failure Mode**: In large repositories (500+ files, 250k+ lines of code), global discovery phases exceed single-agent context limits. Granting LLM agents an autonomous `spawn_subagent` tool inside a ReAct loop introduces three failure modes: (1) *Parent Context Implosion* — $K$ subagent outputs flow back into the parent's history ($O(K \times \text{output\_size})$); (2) *Shadow State Outside Blackboard* — execution state lives in a transient message buffer rather than in $\Sigma(\mathcal{G})$, destroying deterministic checkpointing and $H^*$ recovery; (3) *Unbounded Recursion & Cost* — confused agents trigger runaway agent trees.
- **Design Decision**: Non-ARIADNE agents are strictly prohibited from imperative subagent spawning. Parallelizable macro-phases execute as deterministic 3-stage sub-statecharts (**Scatter → Parallel Map → Reduce**) managed by $\mathcal{O}_{\text{engine}}$ over partitioned blackboard slices, enforcing a strict recursion depth invariant of $D_{\max} = 1$. Out-of-shard definitions resolve via zero-LLM deterministic LSP tools or declarative `UnresolvedCrossLink(src_id, tgt_symbol)` tags stitched during Reduce. Per-phase mechanics: [statecharts-and-ariadne.md §2.3](statecharts-and-ariadne.md).

### 2.4 Two-Stage Hybrid Phase Guards ($g = g_{\text{struct}} \land g_{\text{sem}}$)
- **Failure Mode**: Purely programmatic guards (schema validation, AST parsing, DAG acyclicity) are syntactically sound but semantically shallow — they cannot detect omitted business rules or over-engineered interfaces. Evaluating transitions purely via an LLM critic wastes tokens on structurally malformed outputs.
- **Design Decision**: Every phase transition is gated by a short-circuiting conjunction of a zero-token **Deterministic Structural Guard** ($g_{\text{struct}}$) followed conditionally by a sharded, read-isolated **Adversarial Semantic Guard** ($g_{\text{sem}}$) whose typed boolean verdict drives $\mathcal{O}_{\text{engine}}$ deterministically. Formulation and per-state guards: [statecharts-and-ariadne.md §5](statecharts-and-ariadne.md).

### 2.5 Encapsulated Intra-Module MCTS Solvers (ARIADNE Sub-Holons)
- **Failure Mode**: Single-trajectory translators patch a single code draft linearly, degenerating into local repair loops when the initial refactoring strategy is flawed. Running Monte Carlo Tree Search (MCTS) globally across an entire repository causes an $O(|V| \times |\mathcal{A}|)$ action-space explosion.
- **Design Decision**: Encapsulate **ARIADNE** (a Softmax-UCB MCTS planner coupled with a 4-slice local blackboard and a specialist pool) as an isolated intra-module solver node (`M2`) instantiated per DAG vertex $v_i \in V$. Business Rule Extraction is a pre-DAG global phase ($S_3$) so cross-cutting domain invariants are locked before target interface design ($S_4$).

### 2.6 Push-Based Ephemeral Invocation & Zero-Polling Invariant
- **Failure Mode**: LLM agents running as persistent daemons that continuously watch or poll a blackboard waste process memory, saturate token budgets, and introduce race conditions.
- **Design Decision**: All non-$\mathcal{O}$ agents are stateless, ephemeral worker functions. They never poll or watch $\mathcal{G}$. Only $\mathcal{O}_{\text{engine}}$ maintains an active event-loop listener over $\mathcal{Q}_{\text{events}}$. Workers commit artifacts directly to their assigned blackboard partition on $\mathcal{G}$, push a typed completion event `ev_TaskDone` to $\mathcal{Q}_{\text{events}}$, and terminate immediately.

---

## 3. Formal Data Layouts & Blackboard Schemas

### 3.1 Definition 1: Global Repository Blackboard ($\mathcal{G}$) and State Vector $\Sigma(\mathcal{G})$

The Global Blackboard $\mathcal{G}$ is a persistent, schema-validated tuple:

$$\mathcal{G} = \Big\langle \mathcal{I}_{\text{inv}}, \; G=(V,E), \; \mathcal{R}_{\text{rules}}, \; \mathcal{C}_{\text{contracts}}, \; \mathcal{A}_{\text{artifacts}}, \; \Sigma(\mathcal{G}) \Big\rangle$$

| Component | Type | Description |
| :--- | :--- | :--- |
| $\mathcal{I}_{\text{inv}}$ | `RepositoryInventory` | Repository file inventory, dialect profile, and verified build/test harness configuration. |
| $G = (V,E)$ | `CondensedDag` | Condensed dependency DAG; vertices $v_i \in V$ are atomic translation units (or collapsed SCCs), edges $e_{pi} = (v_p, v_i) \in E$ are dependency obligations. |
| $\mathcal{R}_{\text{rules}}$ | `RuleRegistry` | Verified `Given/When/Then` business rule cards with exact `legacy_file:line` source citations. |
| $\mathcal{C}_{\text{contracts}}$ | `ContractRegistry` | Target-language type signatures, API contracts, and behavioral preconditions across dependency edges. |
| $\mathcal{A}_{\text{artifacts}}$ | `ArtifactRegistry` | Final hardened target modules and their passing characterization test suites. |
| $\Sigma(\mathcal{G})$ | `GlobalStateVector` | Transactionally persisted runtime state: $\langle S_{\text{macro}}, \mathbf{M}, \Theta_{H^*}, \mathcal{Q}_{\text{events}} \rangle$. |

The persisted runtime state vector is:

$$\Sigma(\mathcal{G}) = \Big\langle S_{\text{macro}}, \; \mathbf{M} = \big[M(v_1), \dots, M(v_{|V|})\big], \; \Theta_{H^*}, \; \mathcal{Q}_{\text{events}} \Big\rangle$$

comprising the active macro-state $S_{\text{macro}}$, the per-module micro-state vector $\mathbf{M}$, the deep-history snapshot map $\Theta_{H^*}$, and the FIFO asynchronous event bus $\mathcal{Q}_{\text{events}}$. `GlobalStateVector` (crate `global-blackboard`, module `state_vector.rs`) holds `macro_state`, `module_states: BTreeMap<String, ModuleMicroState>`, `deep_history: Option<BTreeMap<String, ModuleMicroState>>`, and `event_queue: VecDeque<MacroEvent>`, with `initialize()`, `push_event()`, `pop_event()`, `history_state_of()`, `save_deep_history()`, and `restore_deep_history()`.

#### 3.1.1 In-Tree Modular Deconstruction (`werðan/libs/global/blackboard`)

`global-blackboard` deconstructs $\mathcal{G}$ into ten decoupled modules:

| Module | Component | Key Types & Methods |
| :--- | :--- | :--- |
| `inventory.rs` | $\mathcal{I}_{\text{inv}}$ | `RepositoryInventory` (`files: Vec<String>`, `dialect: DialectProfile`, `harness: HarnessConfig`); `add_file()`, `contains()`, `Deref<Target = [String]>`, `IntoIterator`. |
| `dag.rs` | $G=(V,E)$ | `CondensedDag` (`vertices`, `edges`); deterministic Kahn topological sort in `is_acyclic()`, `parents_of()`, `children_of()`. |
| `rules.rs` | $\mathcal{R}_{\text{rules}}$ | `RuleRegistry` (`BTreeMap<String, Vec<BusinessRuleCard>>`); `insert()`, `get()`, `all_rules()`, `Deref`/`DerefMut`. |
| `contracts.rs` | $\mathcal{C}_{\text{contracts}}$ | `ContractRegistry` (`BTreeMap<(String, String), EdgeContract>`); `contains_edge()`, `contracts_for_child()`, `contracts_from_parent()`. |
| `artifacts.rs` | $\mathcal{A}_{\text{artifacts}}$ | `ArtifactRegistry` (`BTreeMap<String, Option<ModuleArtifact>>`), `ModuleArtifact` (`target_code`, `test_suite`); `commit()`, `is_hardened()`, `all_hardened()`. |
| `state_vector.rs` | $\Sigma(\mathcal{G})$ | `GlobalStateVector`, `ModuleMicroState` (`M0Blocked`..`M5Invalidated`), `MacroEvent`; deep-history save/restore (cites `tech-spec 2.6`). |
| `fsm.rs` | Macro statechart | `MacroStateMachine<S>` over `S0IntentPreflight`..`S6SystemVerify`, `Terminated`; `From`-based forward transitions and Tier C $S_5 \to S_2$; `MacroStatechart` boundary wrapper with `advance()`, `regress_to_s2()`, `history()`. |
| `scc.rs` | SCC condensation | `tarjan_condense`, collapsing cyclic components into `+`-delimited merged vertices for Tier C re-keying. |
| `path_resolver.rs` | Persistence paths | Cross-platform derivation via `dirs`: defaults to `output_dir` when provided, else platform data root (`%AppData%/werðan/{hash}` on Windows, `~/.werðan/{hash}` on Linux/macOS). |
| `work_manifest.rs` | Scatter/Reduce tags | `WorkManifest { phase, shard_subjects, covers() }` persisted per phase at scatter entry; `UnresolvedCrossLink { src_id, tgt_symbol }` declarative tag emitted by S3 map workers and resolved by the S3 reduce linker. |
| `lib.rs` | Aggregate | `GlobalBlackboard` composing all components plus `state` (including `work_manifests` and `cross_links`); re-exports every type. All components and the aggregate support the `bon` builder pattern (`::builder()`) with optional `file_path`, `output_dir`, `hash`, `create_if_missing`, and `save()` persistence. |

### 3.2 Definition 2: Per-Module 4-Slice Local Blackboard ($\mathcal{B}^{(v_i)}$)

Each active module vertex $v_i \in V$ owns an isolated local blackboard invisible to other vertices:

$$\mathcal{B}^{(v_i)} = \Big( \mathcal{B}_{PM}^{(v_i)}, \; \mathcal{B}_{SH}^{(v_i)}, \; \mathcal{B}_{CT}^{(v_i)}, \; \mathcal{B}_{PR}^{(v_i)} \Big)$$

1. **Problem Model ($\mathcal{B}_{PM}^{(v_i)}$)**: Hydrated deterministically from $\mathcal{G}$ upon module activation with legacy AST slices, verified rule cards $\mathcal{R}_{\text{rules}}(v_i)$, upstream parent contracts $\{\mathcal{C}_{\text{contracts}}(v_p, v_i)\}$, and downstream export obligations.
2. **Strategy Hypotheses ($\mathcal{B}_{SH}^{(v_i)}$)**: Competing refactoring strategies $h_k$ (e.g. direct structural port vs. state-pattern decomposition) with prior utility estimates $P_{\text{prior}}$ from Laya.
3. **Counterexamples & Characterization Tests ($\mathcal{B}_{CT}^{(v_i)}$)**: Black-box I/O fixtures, property-based tests, and falsifying test inputs derived from legacy behavior.
4. **Patch & Fault Records ($\mathcal{B}_{PR}^{(v_i)}$)**: Distilled compiler diagnostics, test failure traces, and root-cause fault localizations indexing failed rollouts.

In-tree (`werðan/libs/holon/blackboard`, package `holon-blackboard`): `problem_model.rs` ($\mathcal{B}_{PM}$: `BusinessRuleCard`, `EdgeContract`, `ProblemModelSlice` with `render_prompt_context`), `strategy_hypotheses.rs` ($\mathcal{B}_{SH}$: `StrategyHypothesis`, `StrategyHypothesesSlice`), `characterization_tests.rs` ($\mathcal{B}_{CT}$: `CharacterizationTestsSlice`), `patch_records.rs` ($\mathcal{B}_{PR}$: `FaultCause`, `FaultRecord`, `PatchRecordsSlice`). The crate root `lib.rs` re-exports every slice type at `::holon_blackboard::*` and composes the four slices as pub fields of `LocalBlackboard` with `is_hydrated` and `reset_transient_records` delegating methods.

## 4. Global-Side Crates

- **`werðan/libs/global/fsm` (`global-fsm`)**: State-as-struct Hoverbear typestate macro-statechart (`MacroStatechart`) advancing across seven canonical macro states ($S_0 \dots S_6$, `Terminated`) with transition history, regression target navigation, and bon builder.
- **`werðan/libs/global/agents/arbiter` (`global-arbiter`)**: Global arbitration and semantic critics implementing Algorithm 1 stage-two validation. Exports `SemanticCritic`, `Arbiter`, `CritiqueVerdict`, `CritiqueRecord`, `DeterministicCritic`, `DeterministicArbiter` with bon builders.
- **`werðan/libs/global/agents/workers` (`global-workers`)**: Stateless macro-phase worker crew per Algorithm 1 substate $S_m.2$. Exports `TaskActivationEnvelope` (mode, slice summary, delta directive, bound tools), the deterministic `ContextAssembler` hydrating envelopes from the blackboard, `phase_role_name` mapping macro phases to worker roles, the `PhaseWorker` contract (`execute` returning typed `PhaseWorkOutput`), `DeterministicWorkers` covering S0–S6 with zero tokens, and the typed output family (`Preflight`, `Assessment`, `Coupling`, `Rules`, `Contracts`, `Verification`, `Skipped`). Hydration semantics: [statecharts-and-ariadne.md §4](statecharts-and-ariadne.md).
- **`werðan/libs/global/agents/global-critics` (`global-critics`)**: Rig-backed LLM crews. `lib.rs` hosts `RigCriticCrew` (one Rig agent per phase verdict) and `RigArbiter` (ACCEPT/REJECT adjudication); `workers.rs` hosts `RigWorkerCrew` implementing `global-workers::PhaseWorker` over one worker agent per phase, parsing model responses into typed `PhaseWorkOutput` variants; malformed responses fall back to the deterministic no-op so structural guards observe the real board gap.
- **`werðan/libs/global/engine` (`global-engine`)**: Deterministic macro-statechart engine implementing Algorithm 1. Modules: `batches.rs` (`PhaseShard`, `guard_failing_shards`), `error.rs` (`MacroEngineError`), `guards.rs` (`guard_s0`..`guard_s6`, `structural_guard`), `solver.rs` (`ModuleSolver`, `StubModuleSolver`, `SolverOutcome`), `lib.rs` (`MacroEngine` with bon builder). Operates over `statechart: MacroStatechart` and `board: GlobalBlackboard` with $\Sigma(\mathcal{G})$ inside `board.state`. Coordinates phase-loop progression, stage-one structural guards with a single `CRITIC_RETRY` re-dispatch carrying the guard detail as `delta_directive`, stage-two semantic critic batch audits, Kahn topological wave dispatch in $S_5$, Tier B contract-conflict warm-start regression, and Tier C cyclic dependency macro-regression ($S_5 \to S_2$) with deep-history $\Theta_{H^*}$ capture and restore. A failed guard after the one retry round aborts with `MacroEngineError::StructuralGuardFailure`; a second consecutive `RemediateLocal` on the same batch aborts with `MacroEngineError::SemanticRemediationFailed`. Invalid Tier C cycle reports (empty trace, non-DAG edges, arbiter rejection) park the reporter in `M0_Blocked` so the wave re-dispatches it; each such park, each contract-conflict dispatch, and each parentless `SolverOutcome::BudgetExhausted` increments the module's futile-dispatch counter, hardening resets it, and a module reaching the cap terminates the campaign with `MacroEngineError::ModuleRetryExhausted` (`futile_dispatch_cap`, default 3, on `MacroEngine::builder()`). A `BudgetExhausted` park fires no Tier B broadcast and invalidates no parent (no counterparty exists). Pipeline and guard detail: [statecharts-and-ariadne.md §5](statecharts-and-ariadne.md).
- **`HitlGates` and `RunStop`**: The `MacroEngine::builder().hitl(HitlGates)` config enables human-in-the-loop pause points: `gate_on_reimagine`, `gate_on_macro_regression`, `extrinsic_conflict_ask_human` (`HitlGateKind`). A triggered gate parks `pending_gate`, suspends the event drain with the triggering event requeued, and returns `Ok(RunStop::HitlGatePending { gate })`; the operator resolves it with `approve_pending_gate()` (recording the kind in `approved_gates`) and re-invokes `run()`. Headless benchmark runs surface a parked gate as an explicit campaign error instead of a silent success.
- **Scatter WorkManifest and S3 cross-link reduce**: Each discovery phase persists a `WorkManifest { phase, shard_subjects }` onto `board.work_manifests` at scatter entry ($S_m.1$). The deterministic S3 workers emit declarative `UnresolvedCrossLink { src_id, tgt_symbol }` tags in `PhaseWorkOutput::Rules` (deduped onto `board.cross_links` at commit); the S3 reduce stage resolves each tag by copying the source rule card onto the DAG vertex the tag names. `TaskDone` and `ShardDone` events drain without effect (worker outputs commit synchronously through `dispatch_phase_worker`). The S0 probe-reported INTENT gate binds after the guard round: an empty `HarnessConfig` on `board.inventory.harness` fails the phase with `MacroEngineError::PreflightGateFailed`.
