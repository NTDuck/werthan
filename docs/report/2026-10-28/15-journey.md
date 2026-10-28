# 15 — Design Journey: v0 → v4

The versioned trail of how the front-runner emerged, with the audit finding that forced each change. Companion files: requirements (`11`), design space (`12`), candidates (`13`), audit ledger (`16`).

## v0 — Incumbent THESEUS as-implemented

Starting point: the 7-phase statechart + per-module ARIADNE MCTS + two-tier blackboard (`METHODOLOGY.md`), as realized in the v1–v18 daemon lineage.

**Status after evidence audit (LocalAudit brief):** the architecture's *instincts* are literature-aligned (deterministic shell ≈ StateFlow's measured 3–5× cost advantage; blackboard ≈ Hearsay-II/LbMAS; encapsulated search ≈ ARIADNE), but five real campaigns died on engine-accounting artifacts (F1), six deadlocked at the scheduler (F2), one burned ~1 h post-convergence because reward and acceptance disagreed (F3), one "passed" an 18-line stub (F4), and budgets never sized for synthesis difficulty (F7). The flagship formal claim ("$R=1.0$ necessary and sufficient") is false under the shipped oracles (T4). **Not acceptable as-is.**

## v1 — "Fix the engine, keep the architecture" (becomes C3)

First hypothesis: the architecture is sound; the implementations diverged from it (T1, T6). Repair list = the G-gaps: total fallback, liveness invariant, single ground truth, harness adapter, honesty gates, futile-cap reset, warm-start resume.

**What it solves:** every *realized* failure F1–F8 is killed by a named mechanism (coverage table in `16-audit-ledger.md` §wave1). Cost: small, mostly landing already-written fixes into spec text.

**Why insufficient:** the *ceiling* failures survive. toml's vacuous pass is an oracle-strength failure, not an engine bug — no repair of the fallback path detects a suite that passes a stub. D4's cross-block defects are invisible at file-granularity vertices regardless of engine correctness. And the toml 17.1 h runaway shows budget policy needs difficulty adaptivity, not just caps. v1 = necessary engineering floor, insufficient as "optimal methodology."

## v2 — Import the literature front-runner wholesale (becomes C1/C2 evaluations)

Second hypothesis: replace the architecture with the empirically strongest published configuration — ReCodeAgent's four-agent pipeline (86.5% test pass on 118 projects) or GVS5H's ledger (27B → frontier parity).

**What kills it:** regime mismatch, found in the cost/economics evidence. ReCodeAgent runs Claude 4.5 at $15.3/57 min per project; its ablations attribute most of the gain to planner/validator quality that a 27B fleet cannot reproduce (LitEcon S19: naive agent loops on small models *collapse* — completion 99.67→72.01%). GVS5H's ledger solves *one problem*, not a 100-module dependency DAG; it has no cross-module regression machinery and no oracle-strength gate. **Verdict: absorb components (override rule, ledger persistence, executed-tests-override), reject as skeletons.**

## v3 — Hybrid composition (first PASIPHAE sketch)

Keep the statechart+blackboard shell (validated by VAL-* contracts); replace the two components the evidence actually indicts:

- Solver: softmax-UCB rollout spending → **portfolio race, commit, repair** (simple-regret-optimal shape under epistemic noise; Coquelin & Munos show UCT unfounded under violated assumptions; Tolpin & Shimony's SR+CR is the two-phase shape).
- Oracle: trust-the-pass-rate → **layered oracle** with suite-strength gate (TestGenEval's mutation score as the honest adequacy metric), differential execution (translation's unfair advantage: the oracle is the original program), property invariants, verdict aggregation only where execution is impossible.
- Plus: sub-file vertices with assembly contracts (D4), verification-capacity concentration (mechanized + procedural validation; ProST corrected per RefuteLit F2 — trained homogeneous triad, not zero-shot cascade), difficulty-adaptive budgets (MaAS/Snell).

**Self-audit at composition time (Pareto-simplicity pre-check, `13-candidates.md`):** each upgrade pays in requirement currency (R2/R7/R8/R10/R11 are unsatisfiable without it); simpler alternatives fail non-negotiable requirements. **v3 becomes the front-runner candidate C4.**

## v4 — Audit-hardened PASIPHAE (final)

Wave-1 REFUTE pool (six lenses) attacks v3. Findings and dispositions (full ledger in `16-audit-ledger.md`):

1. **Theory lens** — successive elimination is *not* valid for non-stationary arms (repair shifts strategy reward distributions). **Fixed:** race restarts on *external* oracle upgrades only; claim downgraded to an empirical compute-budget heuristic — no optimality or regret language survives (RecheckTheory W2-F5 struck the residual "simple-regret-optimal shape" phrasing from the journey too). All-strategies-eliminated and assembly-contract-failure outcomes made total (new `ExhaustedStrategies` outcome; slice-owned intrinsic faults).
2. **Scale lens** — mutation scoring at TestGenEval's 1 h/file is infeasible per-candidate. **Fixed:** mutant sample bounded by budget class; mutation gate runs at suite-strength time (per suite, not per candidate); stub-sensitivity (cheap) becomes the mandatory gate, mutation kills the strong gate with an explicit budget.
3. **Coverage lens** — F6 (transport) and F5 (OOM) were "killed" by the honesty gates alone; detection ≠ prevention. **Fixed:** operator-contract resource caps + watchdog restart budgets promoted to explicit harness-bounds fields (R6/R9), with campaign-level resource telemetry in receipts.
4. **Literature lens** — two citations needed regime caveats (ReCodeAgent single-run; GVS5H 2026 corpus); cherry-picking risk acknowledged by adding the disconfirming line (BenchAgent, Tran & Kiela, AgentModernize, SWE-Bench+) to the bibliography with explicit engagement in `12-design-space.md` A1/A2. **Disposition: valid, fixed.**
5. **Feasibility lens** — portfolio racing supersedes ADR-0012 (constants calibration); harness adapter requires ADR-0009 schema extension. **Fixed:** governance row added to the ADR change list (`14-final-methodology.md` §7); supersession relations recorded.
6. **Measurement lens** — campaign-exceeds-wall-clock semantics undefined (toml). **Fixed:** §6 truncation semantics — final harness pass on best-known candidates, partial-credit classification, honest terminal receipt.

Second wave re-ran the two lenses whose targets changed most (theory, scale): no new fatal findings → **converged**. v4 is the final proposal (`14-final-methodology.md`).

## What the journey shows

- Every design move traces to either a measured local failure (F/D ids) or a verified literature result — no component exists because it sounded good.
- The two "import the winner" attempts (v2) failed for *regime* reasons, not implementation reasons — the sharpest lesson: published front-runners are regime-bound; the 27B bounded-context regime has its own Pareto frontier (deterministic orchestration + SLM + executable verification).
- The audit waves changed the design four ways (non-stationarity handling, oracle budget policy, resource caps, truncation semantics) — none visible from composition alone.

## v5 - Wave-3 audit and the phantom-disposition lesson

Wave 3 re-audited only the wave-2 amendments and found that **three of the seven wave-2 fixes had never landed in `12`/`14`** — the ledger recorded dispositions for edits that were never written (a stale batching formula, a mutation spot-audit existing only in the ledger, a superseded 0.05 quantum). Root cause: wave-2 fixes were applied by in-memory string replacement, some of which silently matched nothing or wrote to a string that was never flushed. Wave 3 also found a real spec hole (below-cap park deadlocks the scheduler invisibly to the liveness invariant) and forced explicit tier-admission arithmetic (hard-tier quota, zero-sum envelope). The audit process itself is the deliverable's quality control: without the re-read, the report would have certified fixes that existed only in the ledger.

## v6 - Wave-4 audit: arithmetic truth from re-derivation

Wave 4 caught the wave-3 hard-tier quota breaching the very envelope it was written to protect (⌈0.15·|V|⌉ × 64 steps + patch remainder = 1,640 > 1,608 candidate-steps) — a fix that recreated the class of bug it fixed — plus a third phantom disposition (O_SYNC text certified but never written). Root causes: arithmetic asserted against a parenthetical instead of re-derived, and unasserted string replacement. Both process defects now have countermeasures: every fix script asserts its anchor; every quantitative claim is re-derived from the budget constants, not copied. The quota is corrected to ⌈0.10·|V|⌉ with the derivation inline (`14` §6).
