# 04 — Failure Taxonomy: What Actually Happened on Real-Inference Runs

Sources: `benchmarks/results/daemon/skel-{bst,colorsys,heapq,mathgen,toml}.json`, `benchmarks/logs/*-fixverify.log` (8 files), `benchmarks/results/recodeagent-benchmark-report.md` (simulated baseline), `docs/report/2025-10-25/README.md` (authoritative deep-dive).

## Result JSONs (all fields)

| Field | bst | colorsys | heapq | mathgen | toml |
|---|---|---|---|---|---|
| wall_time_ms | 22,144,759 (~6.2 h) | 14,159,337 (~3.9 h) | 11,669,900 (~3.2 h) | 17,653,051 (~4.9 h) | 61,443,312 (~17.1 h) |
| llm_requests | 195 | 119 | 107 | 206 | 774 |
| compilation_success | **false** | **false** | **true** | **false** | **false** |
| test_pass_rate | 1.0 | 1.0 | 1.0 | 1.0 | 1.0 |
| unimplemented_remaining | 0 | 0 | 0 | 0 | 0 |
| solver_dispatches | 3 | 3 | 3 | 6 | 6 |
| worker_executions | 4 | 4 | 4 | 5 | 5 |
| structural_retries | 0 | 0 | 0 | 0 | 0 |
| macro_states_visited | 1 | 1 | 1 | 2 | 2 |
| error | futile dispatch cap after 3 attempts | same | same | same | same |

Shared anomaly: four of five record `compilation_success = false` **with** `test_pass_rate = 1.0` — the pass rate was computed against a stub-passing/collapsed workspace, not a validated candidate.

Fixverify logs (pre-fix binary): all six terminal lines read `error=wave dispatch deadlock: no ready module while unhardened modules remain` with `unimplemented=0` — walls 1,759.3 s (toml) to 29,111.1 s (rbt); html is the only `rate=0.0%`.

## Failure modes

| # | Mode | Evidence | Class | Methodology mechanism stressed |
|---|---|---|---|---|
| F1 | Fabricated-counterparty futile-cap death | bst/colorsys/mathgen/strsim-v17 JSONs: `ModuleRetryExhausted` after 3 dispatches, 195/119/206/457 requests, 3.9–6.2 h each. v1–v16 chain: `resolve_parent_id()` fabricated `"parent_upstream"` for parentless modules → solver bridge remapped `BudgetExhausted` → `ContractConflict` → Tier B broadcast against nothing → futile++ → cap death | ENGINE bug (violates METHODOLOGY §5.2: a conflict broadcast must anchor a real edge $e_{pi}$) | Tier B conflict semantics; outcome classification |
| F2 | Wave-dispatch deadlock | 6/6 terminal fixverify logs: `no ready module while unhardened modules remain`, unimplemented=0, up to 8.1 h | ENGINE bug | Wave scheduler ReadySet liveness (§4: ReadySet defined over M0_Blocked + hardened parents; invalid Tier C reporters returned early without parking → orphaned modules) |
| F3 | Reward-vs-artifact gap (heapq paradox) | heapq v18 JSON: all four acceptance criteria green (`comp=true`, pass 1.0, 0 stubs, 107 reqs) **yet** `error = futile dispatch cap`; the workspace held a solved module but reward scoring never certified $R = 1.0$, so no `ev_ModuleHardened` fired and budget kept burning (~1 h post-convergence) | ENGINE bug | $R(s_t)$ gating (§4.2) vs the acceptance gate use different ground truth |
| F4 | Stub collapse + vacuous pass | toml old-binary: 17.1 h (8.5× over the 2 h bound), 774 requests, candidate collapsed to an 18-line `get_input` stub (478 bytes), parser truncated away at terminal; `target_test_cmd` still exited 0 → `test_pass_rate = 1.0` certified a stub | ENGINE (persistence) + METHODOLOGY gap (artifact-blind metric) | $M4$ commit semantics; oracle stub-sensitivity |
| F5 | OOM kill of daemons | rbt/html/strsim v18 killed 2026-10-25 22:20 with the session scope; **no result JSONs exist**; candidates truncated mid-write (html 76 lines, strsim 87 bytes) | INFRA/OPEX | Campaign orchestration; measurement validity (2/3 of the v18 cohort uninterpretable) |
| F6 | Transport 500s / socket hangs / watchdog restart cap | report §2.3: recurring retryable 500s, 420–585 ms backoff; 2026-10-23 socket-hang class; restart cap 3 hit repeatedly 04:00–04:32 | INFRA/OPEX | Wall-clock bounding; harness robustness |
| F7 | Non-convergence in budget | heapq merge-semantics bug (assertion `25 != 3`), rbt rotation test, html main loop > 15 s timeout; B_max 16 × 3 dispatches = 48 rollout chances, no warm-start resume across dispatches, no futile-cap reset on reward gain | METHODOLOGY gap | Rollout budget sizing (§4: $B_{\max}$ uniform); $\mathcal{B}_{PR}$ fault distillation quality |
| F8 | `compilation_success` semantics drift | pre-fix JSONs conflate comp with pass; fixed same day (VAL-DAEMON-08: build_cmd ok AND 0 stubs; error field separated) | ENGINE/METRICS bug | Acceptance-gate definitions |

## Quantified honesty gap (simulated receipts vs real inference)

| Metric | Simulated (2026-10-18) | Real (2025-10-25, finished runs) |
|---|---|---|
| skel compile rate | 100.0% | 16.7% (1/6) |
| skel pass rate | 87.5% (7/8) | 12.5% (1/8 honest) |
| wall per project | 53–121 ms | 3.2–17.1 h |
| LLM requests | 0 (simulated receipts) | 1,858 across finished runs |

The simulated baseline certified artifacts that never existed; every real terminal except heapq died on an accounting artifact (F1), failed to converge (F7), or was destroyed by infra (F5/F6). **The simulated campaign measured harness plumbing; only real inference measures translation capability — a 7× pass-rate collapse and ~8 orders-of-magnitude wall-time gap.**

## Cross-cutting themes

1. **Reward and acceptance gate use different ground truth.** heapq (green artifact, dead reward) and toml (dead artifact, green tests) are the same defect in both directions: the reward/metric layer does not share execution facts with the gate.
2. **Single-vertex DAGs stress machinery built for multi-module waves.** Tier B has no anchor $e_{pi}$; ReadySet goes empty; parent fabrication was invented to fill the void. The dominant engine failures are category-shape mismatches, not translation failures.
3. **Budget exhaustion is terminal when it is retryable.** One misclassification (`BudgetExhausted` → `ContractConflict`) drove the v1–v16 lineage; even v18 lacks reward-gain cap resets (F7).
4. **No targeted feedback in the failure path.** Opaque timeouts (html) and vacuously-passing suites (toml) mean the patch loop patches against noise.
5. **Operational fragility dominates measurement validity.** OOM kills, restart caps, and unenforced wall clocks destroyed 3/8 of the v18 cohort's measurements outright.
