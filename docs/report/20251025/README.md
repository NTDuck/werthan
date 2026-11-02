# 20251025 Benchmark Report — Real-Inference ARIADNE Campaign (`skel`)

> **Naming note.** This report is filed under the repository's `yyyymmdd` directory convention
> (`20251025`). The original report was titled "2026-10-25 Benchmark Report"; all body evidence
> timestamps below are preserved exactly as recorded (2026-10-25/26, +07) and are unaffected by
> the directory name.

Real-inference ARIADNE campaign status for the `skel` ReCodeAgent category (8 projects,
Python→JavaScript), daemon lineage v1–v18, RTX 3090 (`swift-1.5-qwen3.8-27b` @
`127.0.0.1:8092/v1`). All campaigns are terminal as of **2026-10-26 07:00 +07**: one honest v18
PASS (heapq), five FAILs, and three v18 runs killed by host OOM before reaching a terminal
(§ Reading order, `02-results.md`).

This report treats the binary lineage (v16-era / v17 / v18) as an **instrumentation-version
confound**: every result is stratified by binary version, and *measured terminal verdicts* are
kept strictly separate from *unmeasured kills*. No blended success rate is reported.

## Headline verdicts

| Stratum | Runs | Outcome |
|---|---|---|
| **A — measured terminal verdicts** | 6 | 1 PASS (heapq, v18) · 5 FAIL (bst, colorsys, mathgen — pre-fix binary; strsim — v17; toml — old pre-v18 binary) |
| **B — unmeasured kills** (host OOM 2026-10-25 22:20 +07, no result JSON) | 3 | rbt, html, strsim — v18 daemons, ⏳ no verdict |

- heapq is the **first honest real-inference `skel` PASS**: all four acceptance criteria met
  (`compilation_success == true`, `test_pass_rate == 1.0`, `unimplemented_remaining == 0`,
  `llm_requests 107 > 0`).
- Every finished run other than heapq died with `compilation_success == false` on the
  fabricated-conflict / futile-cap path (now fixed in v18).
- Aggregate `llm_requests` across the six measured terminal runs: 119 + 206 + 195 + 457
  (strsim v17) + 774 (toml old-binary) + 107 (heapq v18) = **1,858 requests**, of which 1,084
  on the honest pre-fix/v17/v18 runs and 774 on the old-binary toml run. The three killed runs
  have no request counts (no result JSONs).
- Baseline contrast: the 2026-10-18 ReCodeAgent report showed `skel` at 100% compile rate,
  87.5% mean pass rate, 7/8 passed — but under stub-campaign semantics with simulated receipts,
  not real engine runs. The 2026-10-25 runs are the first honest `skel` measurements.

## Key findings

1. **Dominant failure (now fixed):** binaries v1–v16 fabricated a `parent_upstream` counterparty
   for parentless (single-vertex) modules, remapped `BudgetExhausted` to a contract conflict, and
   died at the futile-dispatch cap (3) on a methodology-invalid Tier B broadcast against nothing.
2. **Genuine convergence difficulty remains:** heapq (merge semantics, assertion `25 != 3`),
   rbt (red-black rotation test), and html (main loop > 15 s under `node translated.js`) failed
   their characterization tests without converging inside the rollout budget.
3. **Reward-scoring gap (heapq paradox):** heapq's v18 workspace held a fully green candidate,
   but reward scoring never certified it (no hardening event), so the engine spent ~1 h of extra
   budget before parking at futile dispatch 3 — artifact-level PASS, internal calibration defect.
4. **Ops loss:** the host OOM kill at 22:20 destroyed three in-flight v18 runs (rbt, html,
   strsim); they remain unmeasured and need re-runs judged only on honest terminals.

## Reading order

| # | File | Content |
|---|---|---|
| 1 | [`01-setup.md`](01-setup.md) | Experimental setup: model, hardware, engine/binary lineage, dataset category, per-project configuration, and the exact four-condition verdict criteria. |
| 2 | [`02-results.md`](02-results.md) | Results stratified by binary version; measured terminal verdicts vs unmeasured kills; aggregates; baseline contrast. |
| 3 | [`03-failure-analysis.md`](03-failure-analysis.md) | Failure taxonomy with root-cause chains, causal diagram, the heapq paradox, and `compilation_success` semantics drift. |
| 4 | [`04-remediation.md`](04-remediation.md) | Landed fixes (verify-only), immediate actions, methodology-level PROPOSALs (ADR-gated), expected metric impact, effort/impact ranking. |
| 5 | [`05-appendix-validity.md`](05-appendix-validity.md) | Threats to validity and the reproducibility appendix (artifact inventory, raw-field cross-checks, unverifiable facts). |
| 6 | [`06-appendix-ops-timeline.md`](06-appendix-ops-timeline.md) | Operations timeline: watchdog restarts, transport 500s, socket-hang shape, and the 22:20 host OOM kill. |
