# THESEUS Validation Contracts

Authoritative source of behavior contracts for every component in the werðan THESEUS stack, derived exclusively from `docs/references/METHODOLOGY.md` and `docs/references/METHODOLOGY-DRV.md` (the finalized methodology: Architecture (6)+(1.2), 7-phase Harel statechart, encapsulated ARIADNE MCTS clusters). These contracts intentionally outrank implementation-derived tests.

## 1. Purpose & Precedence

1. **Contracts precede implementation.** Each `VAL-<COMPONENT>-<NN>` assertion is implementation-agnostic: written from the methodology text, not from reading crate sources. They pin observable behavior (public API outcomes, persisted state transitions, event emissions, computed formulas), never internal symbol names, module layout, or private state fields.
2. **Precedence over implementation tests.** Validation contracts have HIGHER precedence than any implementation test (including former cucumber feature files, now deleted). On conflict, the contract wins and the implementation changes.
3. **Bugs found by contracts are real bugs.** A failing contract means the implementation deviates from the methodology. Fix the implementation (src/ edits allowed and expected), not the contract.
4. **Future sessions MUST re-derive, never re-implement from src.** The decomposition and assertions live here; source code is consulted only for API alignment (compile errors, `cargo doc` output).

## 2. Execution Protocol

Every assertion maps to a Rust integration test in `tests/validation_contract.rs` of the owning crate (or a shared contract binary for bins), named `contract_<id_snake_case>`:

| Component cluster | Command |
| --- | --- |
| holon clusters | `cargo test -p <holon-crate> --test validation_contract` |
| global cluster | `cargo test -p <global-crate> --test validation_contract` |
| daemon | `cargo test -p daemon --test validation_contract` |
| interactive | `cargo test -p interactive --test validation_contract` |

Live end-to-end analog (Scrutiny/User-Testing validators): run the daemon smoke verification entrypoint (`run_smoke_verification`) against a stub/echo inference backend; contract `VAL-DAEMON-...` asserts the emitted result JSON shape and event log honesty (no silent fallbacks, no fake data).

## 3. Construction Rules

Every component under contract MUST be independently constructible in a test:
- Prefer bon builders (`#[builder]`-style fluent construction) for any struct with more than trivial fields.
- Trait-mockable where an LLM or filesystem dependency exists: every LLM-facing agent/harness trait has a deterministic in-test fake. Contract tests NEVER require a network.
- Missing components found during contract derivation (per methodology, absent in the crate skeleton) MUST be added by the implementing agent. Realized during implementation: `FsmHistory` persistence (holon-fsm), the `MacroEvent::ContractConflict` typed event (global-blackboard), and the `CritiqueRecord` verdict type (global-arbiter).

## 4. Assertion Catalog

ID scheme: `VAL-<COMPONENT>-<NN>`. Components: `MACRO` (global engine/statechart), `BLACKBOARD-G` (global blackboard), `FWM` (worker/scatter-gather), `CRITIC` (semantic guard), `ARBITER`, `M2` (holon engine/MCTS controller), `BB` (local blackboard), `ASTR`/`AGEN`/`ATEST`/`APATCH` (holon agents), `AEVAL` (two-stage evaluator), `FSM` (module micro-FSM), `DAEMON`, `INTERACTIVE`, `TOOL`.

### 4.1 MACRO — Deterministic Statechart Engine (Algorithm 1)

- **VAL-MACRO-01 (S0 Intent & Preflight gate).** The engine cannot dispatch a worker for a downstream phase until the harness gate holds: legacy build compiles, baseline test harness executes, INTENT schema valid. Given a blackboard whose harness verification reports failure, `run()` remains in S0 and dispatches no worker; given success, dispatch proceeds to S1.
- **VAL-MACRO-02 (Scatter-Gather topology).** For every macro-phase in {S1 Assess, S2 Map, S3 Extract Rules, S4 Brief & Reimagine, S6 Verify & Harden}: given a workload partitioned into K shards, the engine issues exactly K worker activations (one per shard) and no worker receives another shard's slice.
- **VAL-MACRO-03 (D_max=1 recursion-depth invariant).** Workers have no subagent-spawning capability: the worker contract surface contains no spawn/dispatch operation. Any cross-shard reference is surfaced as an `UnresolvedCrossLink` tag, not a nested invocation.
- **VAL-MACRO-04 (Structural guard short-circuit).** If `g_struct` fails on shard k, only worker k is re-dispatched with `CRITIC_RETRY` plus the deterministic validator traceback; no semantic critic is invoked; other shards are not re-run.
- VAL-MACRO-05 (Two-stage guard pipeline). g = g_struct ∧ g_sem evaluated with short-circuit: if any structural guard fails, g_sem is never evaluated for that shard; semantic critics run only when all shards pass g_struct.
- **VAL-MACRO-06 (CritiqueRecord-driven REJECT_LOCAL retry).** When a shard critic returns `REJECT_LOCAL` with a CritiqueRecord, exactly the offending shard's worker is re-dispatched with `DeltaDirective` = that CritiqueRecord, activation mode `CRITIC_RETRY`; other shards untouched.
- **VAL-MACRO-07 (REGRESS_UPSTREAM macro-regression).** When a critic verdict is `REGRESS_UPSTREAM` targeting upstream macro-state S_target, the engine transitions S_macro to S_target and continues the loop (no full restart, no terminate).
- **VAL-MACRO-08 (Per-phase semantic guard obligations).** Each macro-phase's semantic guard inspects the methodology-mandated concern: S1 assessment citations, S2 implicit coupling (reflection/shared tables/event buses), S3 rule-card completeness vs legacy AST, S4 over-engineering & rule-card coverage, S6 release sign-off (per-module test suite + compilation verdict, zero security findings).
- **VAL-MACRO-09 (Deep History checkpoint).** On Tier C (`ev_CyclicDependencyFound`) with a valid trace: the engine saves deep history Θ_H*(S5) ← M (the full per-module state vector), quiesces the active regions in the cycle, and transitions S_macro to S2.
- **VAL-MACRO-10 (H\* restoration on re-entry).** After SCC condensation completes and S5 is re-entered, every module region unaffected by the repair is restored to its prior micro-state (e.g. previously M4_Hardened modules are M4_Hardened again without re-execution); only cycle members follow the normal M0→… path.
- **VAL-MACRO-11 (S4→S5 initialization).** When the engine advances from S4 into S5, per-module states initialize from deep history where present (Θ_H*(S5)[vi]) and to M0_Blocked otherwise.
- **VAL-MACRO-12 (S5 completion guard).** The engine exits S5 to S6 exactly when all modules report M4_Hardened; not before.
- **VAL-MACRO-13 (Arbiter-gated regressions).** Tier B/C regressions execute only when `g_struct(trace) ∧ O_arbiter(...) = VALID`; when the arbiter returns invalid (or the conflict trace fails the party-reference check), no state transition occurs, the reporting module parks in M0_Blocked so the wave scheduler can re-dispatch it, and the campaign never deadlocks on unhardened modules.
- **VAL-MACRO-14 (Phase dispatch via typed activation envelopes).** Every worker activation carries an envelope with (activation mode, scoped blackboard slice, delta directive, tool bindings); the scoped slice contains exactly the target ref's 1-hop neighborhood.
- **VAL-MACRO-15 (Push-based, no polling).** The engine consumes a FIFO event queue; agents are ephemeral. There is no API by which an agent can block-poll the blackboard; completion is signaled exclusively by typed events (ev_TaskDone / ev_ShardDone / regression interrupts / ev_ModuleHardened).
- **VAL-MACRO-16 (Termination artifact).** On termination, the produced artifact set R_tgt equals the accumulated artifacts on G (A_artifacts), and the final macro-state is terminal.

### 4.2 BLACKBOARD-G — Global Blackboard G (Definitions 1)

- **VAL-BB-G-01 (Schema tuple completeness).** G exposes: inventory, condensed DAG (V,E), rules map V→2^RuleCard, contracts map E→InterfaceSpec, artifacts map V→(TargetAST, TestSuite), and the persisted state vector Σ(G) = ⟨S_macro, M vector, Θ_H*, Q_events⟩.
- **VAL-BB-G-02 (Transactional state persistence + recovery).** The state vector persists transactionally: given a state change followed by re-open (fresh handle over the same store), S_macro, M vector, deep history, and event queue are exactly recovered (crash recovery / deterministic replay).
- **VAL-BB-G-03 (DAG acyclicity after condensation).** After SCC condensation, the stored dependency graph is a strict DAG (no cycles); SCC collapse maps a cycle to exactly one composite vertex.
- **VAL-BB-G-04 (Partitioned slice writes).** A worker writing shard k's artifact does not mutate any other shard's partition; shard partitions are addressable and isolated.
- **VAL-BB-G-05 (Contract patch application).** Applying a validated Δ_spec to contract C(vp,vi) merges (⊕) the delta into the edge contract without discarding prior preconditions.
- **VAL-BB-G-06 (Event bus FIFO).** Events are consumed in FIFO order; the queue is observable (peekable) and drains deterministically.
- **VAL-BB-G-07 (Inventory coverage).** The inventory records every repository file exactly once (basis for S1's 100% coverage guard).

### 4.3 FWM — Stateless Shard Workers (Scatter Map)

- **VAL-FWM-01 (Shard-scoped reads).** A worker invoked with envelope for shard k reads only that shard's slice (attempting to observe another shard's partition is impossible via the provided API).
- **VAL-FWM-02 (Direct partition writes + completion event).** The worker returns its shard-scoped output; the engine commits it to G[shard_k] and emits ev_ShardDone(k) on completion. Workers never write the board themselves and never spawn agents.
- **VAL-FWM-03 (UnresolvedCrossLink emission).** Given an artifact referencing a symbol outside the shard, the worker records an UnresolvedCrossLink(src_id, tgt_symbol) tag rather than resolving it inline.
- **VAL-FWM-04 (Ephemeral lifecycle).** Worker execution is single-shot: one activation → produce → terminate; no retained conversational state across activations.
- **VAL-FWM-05 (Worker type obligations).** The phase worker set covers the methodology roles: S1 assessors (inventory, dialect, risk), S2 dependency mapper + implicit-coupling scanner, S3 rule miners (Given/When/Then + citations + observed/inferred tags), S4 contract stubbers, S6 integration verifier + security auditor.

### 4.4 CRITIC — Adversarial Semantic Guard

- **VAL-CRITIC-01 (Three-valued verdict).** A critic verdict is exactly one of PASS | REJECT_LOCAL | REGRESS_UPSTREAM, carrying violated invariants and a remediation directive.
- **VAL-CRITIC-02 (Read isolation).** The critic receives only the produced artifact batch plus legacy citations; it cannot mutate blackboard state.
- **VAL-CRITIC-03 (Conjunction semantics).** The phase passes g_sem only if every shard batch verdict is PASS (logical conjunction over shard critics).

### 4.5 ARBITER — LLM Regression Triage

- **VAL-ARBITER-01 (VALID/INVALID binary).** Given (delta_spec, trace) for Tier B, or (cycle_edges, trace) for Tier C, the arbiter returns exactly VALID or INVALID.
- **VAL-ARBITER-02 (On-demand only).** The arbiter is invoked exclusively on regression interrupts (Tier B/C), never during normal phase advancement.

### 4.6 M2 — Holon Engine (ARIADNE MCTS Cluster Controller, Algorithm 2)

- **VAL-M2-01 (Reward formula, gated).** For every evaluated candidate: R = w_comp·I_compile + I_compile·(w_char·PassRate_char + w_hard·Score_harden), with w_comp+w_char+w_hard = 1.0. Uncompilable candidates score exactly w_comp·0 + 0 = 0 credit from char/harden terms (I_compile=0 ⇒ R=w_comp·0=0). Concretely: compile=false ⇒ R=0; compile=true, pass=1.0, harden=1.0 ⇒ R=1.0; compile=true, pass=0.5, harden=0.5 ⇒ R = w_char·0.5 + w_hard·0.5.
- **VAL-M2-02 (UCB1 selection).** Child selection score = X̄(u) + C_exp·√(ln(N(v)+1)/N(u)) + γ·P_prior(u); with unvisited children, selection explores them before exploiting; increasing C_exp biases toward exploration; P_prior adds a γ-weighted prior bias. τ controls softmax temperature over UCB scores.
- **VAL-M2-03 (State-conditioned dispatch).** Given node state flags: (a) no uninstantiated strategies ⇒ strategy agent invoked; (b) empty candidate c_t ⇒ codegen invoked with the selected hypothesis; (c) candidate with 0<R<1 ⇒ patch agent invoked with fault records; (d) missing/insufficient test coverage ⇒ testgen invoked (blind to candidate code).
- **VAL-M2-04 (Backpropagation).** After each rollout: child node created with candidate+blackboard snapshot; reward backpropagated along path to root, updating visit counts and mean rewards at each ancestor.
- **VAL-M2-05 (Budget exhaustion fallback).** After exactly B_max rollouts without R=1.0: the best-reward node is extracted and an ev_ContractConflict(Parent(v_i), v_i, Δ_blocker(best), fault records) is pushed to the event queue.
- **VAL-M2-06 (Hardening event on R=1.0).** A rollout reaching R=1.0 emits ev_ModuleHardened(v_i, c*, B_CT) and terminates the cluster loop.
- **VAL-M2-07 (INITIAL mode seeding).** With activation mode INITIAL: root node has empty candidate; the test agent seeds the characterization suite B_CT before generation rollouts.
- **VAL-M2-08 (WARM_START_REGRESSION retention).** With warm start: the existing tree is retained, active root pointer set to the previously verified node, and its problem model updated with new contract constraints; previously verified drafts remain selectable.
- **VAL-M2-09 (Extrinsic escalation).** When the evaluator classifies κ=EXTRINSIC_CONTRACT on parent v_p: controller transitions M(v_i) to M3_ContractNegotiate, pushes ev_ContractConflict(v_p, v_i, Δ_spec, trace), and returns (halts local loop). When κ=EXTRINSIC_CYCLE: pushes ev_CyclicDependencyFound(E_cycle, trace) and returns.
- **VAL-M2-10 (Tier A containment).** For INTRINSIC faults: no global events pushed; recovery happens locally via reward backpropagation (lower mean reward shifts subsequent selection) and/or patch invocation. The macro-orchestrator is not involved.
- **VAL-M2-11 (Rollout budget honesty).** The controller performs at most B_max rollouts; callers can configure B_max and observe that rollout count never exceeds it.

### 4.7 BB — Local 4-Slice Blackboard

- **VAL-BB-01 (Four typed slices).** The local blackboard exposes exactly: problem model B_PM, strategy hypotheses B_SH, characterization tests B_CT, patch/fault records B_PR, each independently readable and writable.
- **VAL-BB-02 (Hydration from G at M1).** Hydrating a module context copies from the global blackboard: rules R_rules(v_i), parent contracts {C(vp,vi)}, and downstream export obligations into B_PM; B_SH/B_CT/B_PR start empty on INITIAL hydration.
- **VAL-BB-03 (Warm-start preservation).** Appending a delta constraint Δ_spec to B_PM does not clear or reset B_SH, B_CT, B_PR, or the retained search tree; previously written hypotheses/tests/fault records remain readable.
- **VAL-BB-04 (Per-vertex isolation).** Two local blackboards for distinct vertices share no mutable state; writes to one are invisible to the other.

### 4.8 ASTR — Strategy Analysis Agent

- **VAL-ASTR-01 (Input→Output).** Given a problem-model slice, the agent returns ≥1 distinct modernization strategy hypotheses, each carrying a prior utility estimate; written to B_SH by the controller (agent returns values; controller persists).
- **VAL-ASTR-02 (Strategy diversity signals).** Hypotheses derive from problem-model signals (iteration/state/functional/zero-copy signals, rule/contract/constraint counts); distinct signal profiles yield distinct strategy families (e.g. direct structural port vs state-machine decomposition vs async/stream pipeline).
- **VAL-ASTR-03 (LLM interface contract).** The LLM-backed strategy agent issues a single chat completion with a system prompt; response parsing failures produce an honest error (no silent empty-hypothesis success).

### 4.9 AGEN — Code Generation Agent

- **VAL-AGEN-01 (Candidate synthesis).** Given problem model + one hypothesis, the agent produces a complete candidate module source string implementing the contracts/rules of that hypothesis.
- **VAL-AGEN-02 (Honest failure sentinel).** On inference failure, the agent returns a `compile_error!` sentinel candidate (failure stays visible to the evaluator; no silent empty string).
- **VAL-AGEN-03 (Prompt fidelity).** The codegen prompt includes the problem-model rules/contracts relevant to the hypothesis; contract/rule cards referenced in the slice appear in the rendered prompt.

### 4.10 ATEST — Adversarial Test Generation Agent

- **VAL-ATEST-01 (Blind to candidate).** The test agent's inputs contain no candidate code c' (read-isolated from the draft); it sees only problem-model slices (rules, I/O contracts, fixtures).
- **VAL-ATEST-02 (Characterization suite shape).** Given a problem model, the agent returns a set of self-contained test functions (each carrying its own #[test] attribute, no shared harness state), seeds B_CT on INITIAL mode.
- **VAL-ATEST-03 (Splitting semantics).** A response containing multiple `#[test]` functions splits into individual test entries; a response with no test functions yields zero entries (no fabricated tests).

### 4.11 APATCH — Code Repair Agent

- **VAL-APATCH-01 (Repair input/output).** Given candidate code + fault records (+ problem model), the agent returns repaired candidate source; unchanged code in, unchanged code out.
- **VAL-APATCH-02 (Fault-informed prompt).** The rendered prompt embeds the fault records (fault cause, signature, problem model) so repairs target the distilled diagnostics.
- **VAL-APATCH-03 (Honest failure sentinel).** Inference failure surfaces as a compile_error sentinel, never a silent success.

### 4.12 AEVAL — Two-Stage Hybrid Evaluator

- **VAL-AEVAL-01 (Stage order is fixed).** Stage 3a (deterministic execution harness: compile + characterize tests) runs strictly before Stage 3b (diagnostic distiller); Stage 3b runs only on Stage 3a failure or for harden-scoring on success.
- **VAL-AEVAL-02 (Stage 3a determinism).** The execution harness computes compile indicator + characterization pass rate from actual cargo invocation (compile + test commands, timed out); with an inaccessible toolchain the harness reports failure honestly (no fabricated pass).
- **VAL-AEVAL-03 (Distiller fault taxonomy).** The distiller classifies every failure into exactly one of INTRINSIC | EXTRINSIC_CONTRACT | EXTRINSIC_CYCLE, writing a distilled fault record to B_PR; unrecognized labels are INTRINSIC.
- **VAL-AEVAL-04 (Business rule audit).** On candidate success, the distiller's parity critic verifies Given/When/Then rule coverage and yields Score_harden ∈ [0,1] (fraction of rules satisfied; 0.0 when no rules, 1.0 when all terms present).
- **VAL-AEVAL-05 (Unified result).** The evaluator composes harness output + distiller output into a single EvaluationResult carrying compile indicator, pass rate, harden score, fault records.

### 4.13 FSM — Module Micro-FSM (M0–M5, Level-2 statechart)

- **VAL-FSM-01 (Legal transition set).** Permitted transitions: M0→M1 (all parents hardened), M1→M2 (B_PM populated), M2→M4 (R=1.0), M2→M3 (extrinsic contract conflict), M3→M0 (contract patch request posted), M4→M5 (validated downstream contract conflict), M5→M2 (warm-start re-entry). All other transitions are rejected.
- **VAL-FSM-02 (M3 posts request and parks).** Entering M3 emits the contract patch request (ContractPatchRequest to parent v_p on G) and parks v_i into M0_Blocked.
- **VAL-FSM-03 (M4 publication).** Entering M4 publishes target AST + verified test suite to A_artifacts(v_i) on G and emits ev_ModuleHardened(v_i), triggering scheduler recomputation.
- **VAL-FSM-04 (M5 delta-preserving regression).** Entering M5 appends the interface constraint to B_PM while preserving tree/hypotheses/tests, then transitions to M2 with WARM_START_REGRESSION.
- **VAL-FSM-05 (M0 entry condition).** A module enters M0 initially and upon contract negotiation; it leaves M0 only when all parent dependencies report M4_Hardened.
- **VAL-FSM-06 (Persistence of micro-states).** Micro-state + retained artifacts survive save/reload of the state vector (deep history per region).

### 4.14 DAEMON — Orchestration Binary

- **VAL-DAEMON-01 (Campaign result schema).** The campaign result carries: campaign id, per-module outcomes (module id, final micro-state, hardened artifact reference or blocker), event log, totals (hardened/failed counts), and backend configuration echo.
- **VAL-DAEMON-02 (Seeding honesty).** Given an inference backend URL, the daemon configures real HTTP inference; given none, configuration fails honestly (no fake/silent fallback backend).
- **VAL-DAEMON-03 (Phase gating).** The daemon cannot enter module transformation before preflight verification reports a passing harness (S0 INTENT gate) — a failing preflight yields a campaign that blocks/aborts before any LLM call.
- **VAL-DAEMON-04 (Deterministic dry-run).** With a stub backend, a small 2-module DAG campaign completes deterministically: leaf module hardens first, parent second (topological order), events in FIFO order, result JSON serializes cleanly.
- **VAL-DAEMON-05 (Smoke verification entrypoint).** `run_smoke_verification` executes the deterministic pipeline end-to-end and returns Ok on a healthy configuration; configuration errors surface as Err, never silently swallowed.

### 4.15 INTERACTIVE — TUI surface

- **VAL-INTERACTIVE-01 (State-mapped rendering).** The TUI renders campaign state (macro-state, per-module states, event log) from the same persisted state vector used by the engine (single source of truth); no duplicate state source.
- **VAL-INTERACTIVE-02 (Mock runner fidelity).** The mock campaign runner drives the same state machine the real runner uses (same states/events, deterministic sequence); TUI behavior against mock == behavior against engine for identical state vectors.

### 4.16 TOOL — Shared agent tools

- **VAL-TOOL-01 (Read-only navigation tools).** Definition/references/symbol-body tools resolve legacy symbols without mutating any state (methodology: read-only deterministic tooling for out-of-shard lookups).
- **VAL-TOOL-02 (Write tools scope).** Mutation tools write only within their declared target partition; attempts to touch other partitions are rejected.
- **VAL-TOOL-03 (rig Tool trait compliance).** Each shared tool implements ::rig::tool::Tool with deterministic error types; tool errors display consistently (DisplayExt convention).

## 5. Maintenance Rules

- Any code change that alters a contracted behavior MUST update the corresponding assertion here and in the owning crate's `tests/validation_contract.rs` in the same commit.
- New methodology-driven behavior MUST add assertions here BEFORE implementation (contract-first).
- Cucumber/Gherkin suites are deleted and MUST NOT be reintroduced; this spec is the single behavior authority alongside `docs/specs/product-spec.md` and `docs/specs/tech-spec.md`.
