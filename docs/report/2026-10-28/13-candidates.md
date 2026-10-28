# 13 — Candidates, Scoring, Front-Runner Selection

Five candidates composed from the design-space axes (`12-design-space.md`), scored against the thirteen requirements (`11-requirements.md`). Scoring: **✅ full / ◐ partial / ❌ absent** per requirement; the front-runner is the Pareto-dominant survivor of the audit waves (`16-audit-ledger.md`).

## C0 — Incumbent THESEUS as-implemented (v18 lineage)

Axis positions: A1 statechart+blackboard ✓, A2 deterministic ✓, A3 soft-max-UCB MCTS per module, A4 test-only oracle, A5 blackboard ✓, A6 whole-file vertices, A7 uniform B_max + futile cap 3, A8 single model, A9 partial (VAL-DAEMON-02 only).

| R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 | R9 | R10 | R11 | R12 | R13 |
|----|----|----|----|----|----|----|----|----|-----|-----|-----|-----|
| ❌ | ❌ | ◐ (v18 fix, not in text) | ◐ (landed, not in text) | ❌ | ❌ | ❌ | ❌ | ◐ | ◐ | ❌ | ◐ | ❌ |

Realized record: 1 honest PASS / 8 skel campaigns; F1–F8; wall-clock 3.2–17.1 h. **Not acceptable** — kept as baseline.

## C1 — ReCodeAgent-style four-agent pipeline (literature front-runner as-published)

Analyzer → Planner → Translator → Validator with MCP/LSP tools, 5-iteration repair, independent validator. Axis: A1 pipeline, A2 LLM-proposed-ish (per-phase agents, fixed loop), A3 greedy repair, A4 test-only + coverage-gap analysis, A5 plan artifacts, A6 fragment-level (their planner), A7 fixed maxIter=5, A8 frontier model (Claude 4.5), A9 minimal.

| R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 | R9 | R10 | R11 | R12 | R13 |
|----|----|----|----|----|----|----|----|----|-----|-----|-----|-----|
| ◐ | ❌ | ◐ (bounded loop) | ❌ (no DAG) | ❌ | ◐ (tool-based) | ◐ | ◐ (validator separated) | ❌ | ❌ | ❌ | ❌ | ❌ |

Published results are excellent (86.5% test pass, 97.8% function-level) but in a different regime: frontier model, $15.3/57 min per project, no per-campaign honesty gates, and the whole repository in one pipeline with no DAG scheduler, no regression calculus, no warm-start. Adapted to the 27B regime it loses its headroom (LitEcon S19: naive agent loops on small models collapse).

## C2 — GVS5H-style ledger manager–worker

plan.md/tasks.json/notes.md ledger; fresh-context workers; executed-tests-override-manager. Axis: A1 ledger, A2 deterministic harness + LLM workers, A3 fresh-context resampling, A4 executed tests as override, A5 ledger files, A6 task-granularity, A7 round budget 10 + no-progress guard, A8 single model, A9 verifier-overrides.

| R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 | R9 | R10 | R11 | R12 | R13 |
|----|----|----|----|----|----|----|----|----|-----|-----|-----|-----|
| ✅ | ◐ | ◐ (no-progress guard) | ◐ | ◐ | ❌ | ❌ | ◐ | ◐ | ◐ | ❌ | ❌ | ❌ |

Simplest design that satisfies R1 — but no dependency DAG (its regime is one problem, not 100+ ordered modules), no cross-module regression machinery, no oracle-strength gate, and GVS5H's own failure mode (early wrong plan anchors downstream) is unaddressed. Strong as a *component* (the override rule and ledger persistence), too weak as the skeleton.

## C3 — Incumbent + G-gap repairs (no structural change)

C0 plus: total fallback (G3/R3), liveness invariant (G5/R4), reward≡gate single truth (G4/R1), harness adapter (G6/R6), honesty gates (G7/R9), futile-cap reset + warm-start resume (G3/R5). No oracle-strength gate, no sub-file units, no differential layer, no cascade.

| R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 | R9 | R10 | R11 | R12 | R13 |
|----|----|----|----|----|----|----|----|----|-----|-----|-----|-----|
| ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ◐ | ❌ | ✅ | ❌ |

Kills every realized engine failure (F1–F8) — this is the *minimum honest engineering floor*. But the ceiling stays: toml's vacuous pass (R2/R8) and synthesis-difficulty budgets (R7/R10) remain; the methodology's differentiating claims stay unvalidated (G8).

## C4 — **PASIPHAE** (front-runner): grounded racing statechart

C3's engineering floor **plus** the five structural upgrades the literature forces (audit-hardened):

1. **Portfolio-race solver (A3)** replaces softmax-UCB rollout spending: strategies race on short depth-2 probe *repair* rollouts (not zero-shot drafts; depth-1 is myopic, RecheckTheory W2-F4) under a per-arm-per-race exploration floor $N_{\min}$ ($K\le3$, $N_{\min}{=}2$ ≈ 26 s/race); winner gets repair-focused budget with test anchors (AlphaCodium) and rethink-in-place feedback (RethinkMCTS); stalled winners ($\Delta R < 0.6/N_{\text{suite}}$ for $K_{\text{stall}}$) trigger `ev_StrategyStalled` → demote + revive runner-up (≤2 revivals). Claim status (RefuteThe F1–F3, corrected): an empirical compute-budget heuristic justified by F7's measured dispersion failure and Snell's allocation evidence — **no regret bound claimed**; all-strategies-eliminated is a total outcome (`ExhaustedStrategies` → increment futile counter → below cap, park with `strategies_exhausted` marker and excluded from ReadySet pending external unblock; at cap, terminal `M_Failed(StrategyExhaustion)` + `ev_ModuleFailed` with deterministic downstream `Failed(DependencyUnmet)` cascade, RefuteThe F5 + RecheckTheory W2-F1).
2. **Layered oracle (A4)** with execution-cost budget policy (RefuteScale F1): mandatory stub-sensitivity (L1-lite, ~1.4% of campaign budget) on every suite; budgeted mutation kills (L1-full, per-suite ≤30 mutants with 10 s per-mutant timeout, early-abort at first survivor) where the tier affords it, else `oracle_strength: stub_sensitivity_only` receipt label; differential execution legacy-vs-candidate on shared inputs (tHinter/ACToR) probed during the race (bounded subset) and enforced at hardening (RefuteThe F4: the race must see the strong oracle); property-based invariants (PBT +23–37%) → verdict aggregation only where execution impossible (MatchFixAgent). Test agents read-isolated from code candidates (AgentCoder); no same-model self-endorsement (Articulate-but-Wrong 31.7%).
3. **Sub-file vertices with assembly contracts (A6)** where coupling analysis demands — rewritten per RecheckScale W2-F3: coupling-tier decomposition groups shared-binding blocks only when the group fits the ~300 LoC context budget; binding-closures larger than that stay one whole-file vertex (decomposition refused where coupling forbids it — the Skel reality for bst/rbt/html/toml/mathgen); explicit directed assembly edges; file-level container with deterministic blame attribution (RefuteScale F3 / RefuteThe F6); contracts checked deterministically at assembly.
4. **Verification-capacity concentration (A8)**: fleet 27B for generation volume; mechanized verification (differential execution, stub-sensitivity, coverage/parity checkers) + read-isolated multi-sample procedural validation at arbiter/critic/selection nodes; self-endorsement flagged and barred from solo certification (RefuteScale F2 single-GPU form; ProST corrected per RefuteLit F2; IBM WCA4Z combine-checkers).
5. **R13 validation duties**: SCC/composite/regression machinery ships with synthetic VAL-SPLIT/VAL-ASSEMBLY contract families; differentiating claims the corpus cannot exercise are labeled `unexercised` in receipts (RefuteCover F6).

Axis positions: A1 hybrid statechart+blackboard (retained), A2 deterministic ✓, A3 race+repair, A4 layered, A5 blackboard+checkpointed events, A6 sub-file where needed, A7 difficulty-adaptive + progress-gated, A8 verification-capacity concentration, A9 single-ground-truth gates.

| R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 | R9 | R10 | R11 | R12 | R13 |
|----|----|----|----|----|----|----|----|----|-----|-----|-----|-----|
| ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

## C0-pooled — token-matched single-agent baseline (audit requirement, RecheckMeas W2-F6.2 / RefuteLit F1)

One 27B agent, same total output-token budget as C4 (≈1.93 M tokens/campaign), same oracle stack (harness receipts, stub-sensitivity, differential execution), no statechart/DAG/regression machinery: iterative retrieve→edit→test until budget. This is the BenchAgent/Tran & Kiela challenge made explicit — the mandatory comparison the anti-MAS literature demands. It is *not* scored to win; it exists so the front-runner claim is falsifiable. Expected result (hypothesis, not measurement): C0-pooled matches C4 on isolated patch-tier vertices but lacks cross-module regression control (F2-class deadlocks), warm-start resume (F7-class), and assembly contracts (D4-class) — its deficit should concentrate exactly on multi-module projects.

| R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 | R9 | R10 | R11 | R12 | R13 |
|----|----|----|----|----|----|----|----|----|-----|-----|-----|-----|
| ✅ | ✅ | ❌ | ❌ | ◐ | ✅ | ❌ | ✅ | ✅ | ❌ | ◐ | ❌ | ❌ |

## Scoring summary

| Candidate | R-coverage | Realized failures killed | Ceiling | Audit verdict |
|---|---|---|---|---|
| C0 incumbent | 1/13 full | none (F1–F8 realized) | stays at 1/8 pass | rejected (baseline) |
| C0-pooled | 5/13 full | engine-class failures return (F2/F7) | single-module parity, multi-module deficit | falsification baseline (not a rival skeleton) |
| C1 ReCodeAgent-style | 0/13 full, 5 partial | n/a (different regime) | frontier-dependent | rejected for regime mismatch; components absorbed |
| C2 GVS5H ledger | 1/13 full, 7 partial | F1/F3-class | no DAG/regression machinery | rejected as skeleton; override rule + ledger absorbed |
| C3 incumbent+repairs | 7/13 full | F1–F8 (engine class) | oracle ceiling, granularity ceiling | accepted as engineering floor; **absorbed into C4** |
| **C4 PASIPHAE** | 13/13 | F1–F8 + oracle/granularity classes | bounded by regime (D5–D8) | **front-runner** → audit waves |

## Why not "C4 = C3 + everything" (Pareto-simplicity pre-check)

Each structural upgrade pays for itself in *requirement* currency, not decoration:

- Portfolio racing replaces UCB rollout dispersion whose failure we measured (F7: 48 dispersed chances, no convergence) with the two-phase structure the bandit literature recommends for simple-regret objectives; it reuses the existing tree/blackboard (no new substrate).
- The layered oracle is the only mechanism that converts toml-class vacuous passes into measurable failures — without it R2/R8 are unsatisfiable at any budget. (Claim hygiene per RefuteLit F4/RefuteThe F3: oracle defects make rewards *deterministically wrong* — only oracle strengthening fixes that; generator sampling remains genuinely stochastic and keeps its exploration floor. Both are addressed, by A4 and by $N_{\min}$ respectively.)
- Sub-file vertices exist because D4's cross-block defects are *invisible* at file granularity (03-skel-block-granularity.md); they are gated by a coupling analysis, so simple repositories keep whole-file vertices (no cost added where not needed).
- Verification-capacity concentration is the only answer to R11's evidence (verifier too weak to steer toml's 774 requests); in the single-GPU regime its cost is bounded because the added capacity is mechanized (model-free checkers) and procedural (read-isolated prompts), not a second model.

Simpler alternatives (C2 alone, C3 alone) fail requirements the failure evidence makes non-negotiable. C4 is the minimal requirement-complete design in the audited space.
