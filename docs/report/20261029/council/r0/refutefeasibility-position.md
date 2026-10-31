# Round 0 Calibration — RefuteFeasibility

Persona: implementability against the actual codebase (trait boundaries, crate
hierarchy, config schema per ADR-0009, harness adapters, migration cost, dead config,
single-maintainer burden). Evidence for every anchor below was read from the repo this
round. No substantive attacks; Round 0 states priors, claims, and intended targets.

## Steelman

The incumbent methodology is not a vaporware proposal; it is *already implemented* at
~24.6k first-party Rust LoC (`werðan/libs/{global,holon,shared}`, `werðan/bins`,
excluding 419k vendored lines under `werðan/libs/shared/tools/references/`), with a
compile-enforced crate hierarchy (ADR-0010), a canonical config schema
(ADR-0009), and a contract-first VAL-* catalog (ADR-0015,
`docs/specs/validation-contracts.md`) that pins methodology-to-code behavior. Its
strongest implementability claims, which I will steelman before attacking:

- **F-C1 (deterministic control is real code, not prose).** `O_engine` exists:
  `werðan/libs/global/engine/src/lib.rs` implements Algorithm 1 with phase loop,
  structural guards, semantic remediation, Kahn wave dispatch, Tier B/C handling, and
  HITL gates (`docs/specs/tech-spec/engine-and-blackboard.md` §4). Macro-LLM control is
  genuinely zero-token; state persistence is a typed `GlobalStateVector`
  (`state_vector.rs`), matching METHODOLOGY.md Def. 1.
- **F-C2 (typestate FSMs are compile-checked).** "invalid FSM transitions fail to
  compile" (ADR-0010 Consequences); `holon/fsm` is pure typestate M0–M5
  (`docs/specs/tech-spec/statecharts-and-ariadne.md` §6).
- **F-C3 (the reward gate is implemented as specified).**
  `werðan/libs/holon/engine/src/reward.rs` computes the gated R exactly per
  METHODOLOGY.md §4.4; VAL-M2-01 pins compile=false ⇒ R=0.
- **F-C4 (config surface is single-sourced).** ADR-0009 deletes per-frontend parse
  structs; both frontends consume `configschema::CampaignConfig`; schema changes
  compile-break consumers by design.
- **F-C5 (machinery boundaries give alternatives cheap footholds).** The
  `ModuleSolver` trait (`global/engine/src/solver.rs`) and generic
  `ClusterEngine<M, H, D>` / `AriadneSolver<M, H, D>` (ADR-0010) are exactly the seams a
  baseline (c) "incumbent minus machinery" variant or a linear pipeline would plug into.

Any X6 complexity number I later produce must treat these as sunk assets, not costs.

## Objections

Round 0 — planned attack surfaces only (no objection IDs; F-r1-* starts Round 1).
Each lists target + the falsifier class I intend to attempt.

| # | Target | Intended falsifier class |
|---|---|---|
| 1 | **Tier C (SCC condensation + Deep History) cost/benefit.** Implemented (`scc.rs` `tarjan_condense` + re-keying, `deep_history` save/restore, VAL-MACRO-09/10) but the multi-file population has zero SCCs. I will quantify its carried cost: LoC, two contract-test families, SCC re-keying complexity, maintenance of a path no campaign executes. | E1: code-path + campaign-log audit for any `CycleFound`/`ev_CyclicDependencyFound` occurrence in real inference; synthetic VAL run receipts as the alternative evidence route (RQ2). |
| 2 | **Dead and drift-prone config surface (X6's config-field axis).** `ariadne.llm_backend` and `ariadne.cargo_harness` are schema-complete but unconsumed (`docs/specs/tech-spec/benchmark-harness.md` §7.7); meanwhile `bench_runner.rs:386-393` now reads `ucb_exploration_c`, `softmax_temperature`, `reward_weights`, contradicting §7.7's "daemon reads … `engine_budget_per_module` only" — a live spec↔code drift in the X6 measurement instrument itself. | E1: schema-field grep across `werðan/bins` + `libs/*/engine`; diff of §7.7 against `bench_runner.rs`. |
| 3 | **Methodology-vs-binary constant drift.** METHODOLOGY.md pins τ=0.35, γ=0.15; the binary that produced every E1 receipt ran τ=0.50, γ=0.50 (ADR-0012, `holon/engine/src/types.rs`). All formal-claim tests (e.g. RQ1's R=1.0 sufficiency) must be scored against the constants that actually executed. | E1 + quote-level diff METHODOLOGY.md §4.4 vs ADR-0012 table. |
| 4 | **Exercised-vs-specified worker machinery.** METHODOLOGY.md §3 specifies LLM `ShardAssessor`/`RuleMiner`/`ContractStubber` fan-out; in-tree `DeterministicWorkers` covers S0–S6 "with zero tokens" (`engine-and-blackboard.md` §4), and receipts show `worker_executions` 4–6 with tiny request counts. If benchmark campaigns run deterministic workers, the methodology's scatter-gather LLM machinery is *specified but unexercised* — its X6 cost is carried, its X1 evidence nil. | E1: per-phase request accounting in campaign receipts vs code path inspection (`RigWorkerCrew` vs `DeterministicWorkers` selection). |
| 5 | **Migration cost of leading alternatives (K4 mandate).** For every frontier candidate I will produce X6: incremental LoC, config fields added/retired, complexity tier (S/M/L/rewrite) against the seams in F-C5. A candidate with no X6 estimate cannot reach the frontier (01 §7 K4). | E4 derivations against the actual trait boundaries; compile-boundary feasibility for baseline (c). |
| 6 | **Single-maintainer burden is measured, not asserted.** The fabricated-counterparty bug lived v1–v16 and killed 5/6 finished campaigns (20251025 §2.1); v1→v18 lineage spans ~1.5 days; 5 concurrent daemons were OOM-killed together (20251025 §1 postscript). This is the empirical price of the incumbent's own complexity — any candidate proposing *more* machinery owes a burden argument. | E1: 20251025 report + binary lineage facts; per-candidate failure-surface comparison. |
| 7 | **Incumbent's own X6 hygiene.** `global/engine/src/lib.rs` is 1,934 lines in one file under `.checks/stepdown`/`flatmods` constraints; `METHODOLOGY-DRV.md` is a raw Gemini transcript (`Model: Gemini`, header) with interactive-visual placeholders — the derivation chain's maintainability is itself in scope. | E1: file metrics; quote-level DRV→METHODOLOGY.md lineage check. |

## Evidence & Derivations

Implementation/cost claims I will hold the incumbent (and every candidate) to, with
anchors and the evidence class that could falsify each:

| Claim under audit | Anchor | Falsifying evidence class |
|---|---|---|
| "Orthogonality reduces $O(m^{\vert V\vert})$ to $O(m \cdot \vert V\vert)$" | METHODOLOGY.md §1.2 | E4/E1: no $O(m^{\vert V\vert})$ artifact exists to compare against; the claim is a design-space argument, not a measured win. Cost side is real: ADR-0010 "Bad, because generic engine signatures grow monomorphized instantiations per (model, harness, distiller) tuple". |
| "$R=1.0$ necessary and sufficient" | METHODOLOGY.md §4.4; `reward.rs` (60 LoC); VAL-M2-01 | E1: the heapq paradox — all-criteria workspace never certified (20251025 §2.5) shows the sufficiency direction untested in real inference (RQ1, RefuteTheorist leads; I audit the certification code path). |
| Tier C machinery is validated | VAL-MACRO-09/10; `scc.rs` | E1: zero executed Tier C in corpus (RQ2); synthetic VAL runs would be the only rescue. |
| Config schema is canonical and drift-free | ADR-0009 | E1: dead fields (#2 above) show the schema carries surface consumers never read. |
| Deterministic engine is audit-replayable | VAL-BB-G-02; `state_vector.rs` | E1: can a third party reconstruct why a campaign terminated from persisted state alone? heapq's green-workspace/failed-certification gap suggests partial (RQ8-adjacent). |
| Validation contracts outrank implementation | validation-contracts.md §1 | E1: contract tests exist (16 `tests/validation_contract.rs` files) but contracts "derived exclusively from METHODOLOGY.md" inherit the constant drift (#3). |

Measured baseline for X6 deltas (all E1, counted this round): first-party src ≈ 24.6k
LoC — global 6.4k, holon 5.8k, bins 6.0k, shared 4.1k (419k vendored excluded); 16
validation-contract test files; configschema 64 `pub` items; daemon overrides
`engine_budget_per_module`, `ucb_exploration_c`, `softmax_temperature`,
`reward_weights` (`bench_runner.rs:380-393`).

## Requests to the council

1. **RQ ownership.** I claim: **RQ4** primary (forced-vs-aesthetic verdicts need
   implementability and migration-cost evidence; the StateFlow-style inference leaps are
   also X6 leaps); **RQ6** shared with RefuteScaler — I own the implementation-cost and
   config-surface half, they own throughput arithmetic; **RQ2** cost-side (unexercised
   machinery pricing, per 01 §2 X5 "machinery-verification tax"); **RQ3** limited to
   baseline (c) construction feasibility. RQ1, RQ5, RQ7 (candidate enumeration), RQ8 are
   not mine; I supply X6 inputs to RQ7 scoring.
2. **X6 gating.** Per 01 §7 K4, confirm the chair will require an X6 estimate
   (LoC + config-field delta + tier) on every frontier candidate before dominance
   scoring; I commit to producing those estimates in attack rounds.
3. **Evidence-class discipline for X6.** X6 claims are E1 (measured against this tree)
   or E4 (explicit derivation with seam-by-seam accounting); E5 "this is simple" is
   inadmissible for complexity claims in either direction.
4. **Spec↔code drift channel.** Findings like surface #2 (stale §7.7 vs `bench_runner.rs`)
   live in `docs/specs/`, not in the critique targets. I ask the council to treat
   spec-code drift discovered during audit as ledgerable evidence (E1) but route fixes
   to the docs workstream, not into methodology objections.
5. **Constant-drift standing rule.** Any formal claim tested against receipts must name
   which binary/constants produced them (methodology τ=0.35/γ=0.15 vs in-tree
   0.50/0.50, ADR-0012). Request the chair stamp this on all RQ1/RQ2 adjudications.
