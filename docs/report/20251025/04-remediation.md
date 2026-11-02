# 4 · Remediation Strategy

## 4.1 Already landed on 2026-10-25 (verify-only)

| Fix | What it does | Expected effect |
|---|---|---|
| v17 parentless fault gate | Controller `step_core()` treats an EXTRINSIC_CONTRACT/EXTRINSIC_CYCLE fault on a parentless module as intrinsic and keeps rolling instead of fabricating a "parent_upstream" counterparty. | Stops the fabricated Tier B broadcast (methodology-invalid per METHODOLOGY.md §5.2, which requires a block on a real edge e_pi); removes the dominant driver of 3-dispatch futile-cap death. |
| v18 honest `BudgetExhausted` outcome | New `SolverOutcome::BudgetExhausted` variant: parentless module with exhausted rollout budget parks in M0_Blocked, still counts futile, and fires NO Tier B broadcast. VAL-M2-05 contract + specs updated (validation-contracts.md, statecharts-and-ariadne.md §10, engine-and-blackboard.md §4); 277 workspace tests green, 0 clippy. | v17 remapped BudgetExhausted to a conflict shape in the solver bridge → futile++ (heapq v17: comp=true, 194 reqs, still died at 3 dispatches). v18 makes the terminal honest (ModuleRetryExhausted vs hardening) and eliminates the spurious-conflict futile counter path. |
| `compilation_success` semantics fix | Definition restored to: build_cmd ok AND zero unimplemented stubs; terminal error lives in `error` (VAL-DAEMON-08, bench_validation reuses `CargoHarness::run_command`). | Earlier runs read as comp=false partly from semantics drift (pre-fix finished runs show comp=false with pass_rate 1.0, gating them out of acceptance criterion 1/2). Restored semantics make criterion 1 measurable against the real build command — heapq v18 is the first run to read comp=true and pass all four criteria. |
| Watchdog remediation | Hang-restart + error-exit-restart capped at 3 per campaign; socket-hang shape (2026-10-23 class) guarded via watchdog + `inference_timeout` 600 s. | Contains transport 500s and hangs (html/strsim/toml/mathgen hit the cap repeatedly 04:00–04:32) instead of burning hours on a wedged campaign; keeps wall_time inside the 10 min–2 h window. |

## 4.2 Immediate actions (next 24 h from 2026-10-25)

1. **Re-run toml on the v18 binary.** The toml campaign finished at 21:43 on the old pre-v18
   binary (started 04:39): 17:04 wall, 774 requests, comp=false, candidate collapsed to an
   18-line stub. It has never been measured on v18. A v18 re-run yields the first honest toml
   measurement and likely moves wall_time from a 17 h overrun into a bounded run.
2. **Re-run rbt / html / strsim on the v18 binary and judge them only on honest terminals.**
   The v18 daemons for all three were OOM-killed at 22:20 (host memory exhaustion; the kernel
   killed the session scope) before reaching a terminal — no result JSONs, candidates on disk
   truncated or stale. heapq already produced the verdict (PASS at 20:03). The open question
   for the re-runs is whether the patch/testgen loop converges inside 3 dispatches × 16
   rollouts. Only a `ModuleRetryExhausted` terminal or a hardening completion is an
   interpretable v18 data point.
3. **After the v18 baseline lands, consider raising `engine_budget_per_module`
   (`ariadne.max_rollouts`, currently 16 for the T1-tier skel projects) for the skel category.**
   The config already exposes it, so this is an operator-side change, not a code change. skel
   projects are single-vertex DAGs: B_max = 16 rollouts/dispatch × 3 dispatches = 48 rollout
   chances total, and heapq (merge semantics: assertion `25 != 3`), rbt (rotation test), html
   (main loop > 15 s) fail characterization tests without converging inside it. Gate the bump
   on v18 baseline data so the effect of the honest-outcome fix is measured before budget
   semantics change too.

## 4.3 Methodology-level proposals

> All four items below are **PROPOSALs, not landed changes**. METHODOLOGY.md is read-only
> ground truth (ADR-0005); each adopted change routes through a MADR ADR in `docs/decisions/`.

- **(a) Per-category rollout budgets — PROPOSAL.** Today every dispatch gets B_max = 16
  rollouts regardless of category. Harder single-module translations (heapq's merge semantics,
  rbt's rotations, html's 1.3 k-LoC candidate) plausibly need > 16 rollouts, or a
  multi-dispatch **warm-start resume** that carries the best-so-far candidate and failure
  distilled knowledge into the next dispatch instead of restarting from a fresh candidate
  under a 3-dispatch cap.
- **(b) Futile-cap reset policy on partial reward improvement — PROPOSAL.** Today only
  hardening resets the futile dispatch counter. A module whose best reward climbs
  (e.g. 0.2 → 0.6) across dispatches still dies at 3 attempts even though the loop is
  demonstrably converging. An ADR could define a reward-delta threshold that resets (or
  extends) the cap.
- **(c) Candidate time-budget guard in generated characterization tests — PROPOSAL.** html's
  candidate main loop runs > 15 s under `node translated.js` and burns the whole test timeout;
  the harness sees the timeout → fault distill → patch cycle. A generated per-test (or
  per-suite) time budget would fail fast with a targeted fault instead of an opaque timeout.
- **(d) Classifier honesty in blocker text — PROPOSAL (cosmetic).** After v17's gate, a
  parentless module's fault is treated as intrinsic, but the fabricated-parent trace text can
  still appear in the recorded blocker. An honest INTRINSIC classification should also suppress
  the fabricated "parent_upstream" narrative in blocker output — it misleads failure diagnosis.

## 4.4 Expected metric impact per item

| Item | compilation_success | test_pass_rate | wall_time | llm_requests |
|---|---|---|---|---|
| §4.1 v17/v18 gate + honest outcome | Indirect: stops fabricated-conflict death, letting runs reach a real verdict instead of comp=false via cap death | Enables honest convergence attempts on the real tests | ↓ (no 3-dispatch churn against a nonexistent parent) | ↓ fewer wasted broadcast/patch rounds |
| §4.1 comp-semantics fix | ✓ directly restores criterion-1 measurability (build_cmd ok + 0 stubs) | Neutral | Neutral | Neutral |
| §4.1 watchdog | Neutral | Neutral | ↓ bounds runaway campaigns to the 2 h window | ↓ on transport-failure retries |
| §4.2 toml re-run on v18 | First honest toml measurement; plausible comp=true if build_cmd passes | First honest toml measurement | From 17 h overrun → bounded | 774 (old binary, stub collapse) → real count |
| §4.2 judge v18 runs on honest terminals | Verdict-quality data for heapq/rbt/html/strsim | Real pass_rate for the actual failures | Neutral | Neutral |
| §4.2 raise B_max for skel | ↑ plausible (more rollout chances per dispatch) | ↑ plausible (convergence room), at GPU-cost increase | ↑ per dispatch (more rollouts), ↓ from fewer futile-cap deaths — net unclear until baseline | ↑ proportionally with rollouts |
| §4.3(a) per-category budgets / warm-start resume | ↑ | ↑ (resume keeps best-so-far instead of restarting) | ↑/↓ — more rollouts each, fewer full restarts | ↑ |
| §4.3(b) futile-cap reset on reward gain | ↑ for modules mid-climb (0.2→0.6) that currently die | ↑ same set | ↑ (extra dispatches only where converging) | ↑ |
| §4.3(c) test time-budget guard | Neutral | ↓ diagnostic burn; ↑ convergence odds (targeted patch vs opaque timeout) | ↓ per failed candidate (fail fast) | ↓ fewer timeout-distill cycles |
| §4.3(d) blocker honesty | Neutral | Neutral | Neutral | Neutral; improves diagnosis only |

## 4.5 Ranking: effort vs expected impact

| Rank | Item | Effort | Expected impact |
|---|---|---|---|
| 1 | Re-run toml + rbt/html/strsim on v18 (§4.2) | Low (restarts + memory headroom) | High — heapq v18 is the first honest skel result (PASS); the other four projects have no interpretable v18 measurement (OOM-killed or old binary) |
| 2 | Add memory headroom before re-runs (§4.2) | Trivial (stagger daemons or cap concurrent campaigns) | High — the 22:20 OOM kill destroyed three in-flight v18 runs; 5 concurrent daemons + session exceeded host RAM |
| 3 | Raise `ariadne.max_rollouts` for skel after baseline (§4.2) | Low (config-only, already exposed) | Medium-high — targets root cause #2 (no convergence in 48 rollouts) at GPU-time cost |
| 4 | Test time-budget guard (§4.3c) | Low-medium (codegen change) | Medium — directly fixes the observed html >15 s burn |
| 5 | Futile-cap reset on reward gain (§4.3b) | Medium (needs ADR + controller policy) | Medium — rescues mid-convergence modules, unquantified today |
| 6 | Per-category budgets / warm-start resume (§4.3a) | Medium-high (ADR + engine) | Medium — overlaps #3; the config lever may suffice for skel |
| 7 | Blocker honesty cleanup (§4.3d) | Low (cosmetic) | Low — diagnosis quality only, no metric movement |
