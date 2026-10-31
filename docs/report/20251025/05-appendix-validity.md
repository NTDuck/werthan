# 5 · Appendix — Threats to Validity & Reproducibility

## 5.1 Threats to validity

**T1 — Instrumentation-version confound (binary lineage).** The campaign spanned daemon
binaries v1–v16 → v17 → v18 with materially different failure semantics
(`01` §1.3). Cross-stratum comparison is invalid; all rates in this report are stratified.
The single v18 verdict (heapq PASS) is one data point, not a v18 success rate.

**T2 — Unmeasured kills.** Three v18 runs (rbt, html, strsim) were OOM-killed before terminal
and have no result JSONs. Any success-rate computed over "8 projects" would silently treat
unmeasured runs as failures; this report instead reports 1 PASS / 5 FAIL over 6 measured
terminal runs plus 3 unmeasured (`02` §2.3).

**T3 — Metric-semantics drift.** `compilation_success` was conflated with pass-rate semantics
on pre-fix binaries (`03` §3.6). Pre-fix rows' `test_pass_rate = 1.0` is recorded as measured
but does not certify behavioral parity under the restored definition.

**T4 — Reward-scoring gap.** heapq's PASS is artifact-level: the workspace was green, but
reward scoring never certified it (no hardening event). A metric defined on reward/hardening
would score the same run as a non-convergence (`03` §3.5).

**T5 — Single-seed, single-machine, no repetition.** Each project ran once, on one host (RTX
3090), against one local endpoint, with a moving binary. No variance estimates exist; wall
times and request counts are single observations, not means. Runs also shared the host with 5
concurrent daemons + a desktop session (the OOM context), so wall times carry contention
effects.

**T6 — Config-tier heterogeneity.** skel projects span LoC tiers T1–T3 with different
`max_rollouts` (16/32/64), shard counts, and timeouts (`01` §1.2); the "48 rollout chances"
arithmetic applies strictly to the T1-tier projects (colorsys, heapq).

**T7 — Evidence decay.** Ground truth was snapshot at `/tmp/report-ground-truth.md`
(2026-10-25 19:00 +07), a non-persistent path; candidate line/byte counts quoted at 19:00
(html 1349, strsim 1216) cannot be re-derived after the 22:20 truncations. Kill-time candidate
sizes are last-known-state, not terminal measurements.

**T8 — Timestamp ambiguity.** The original report mixed date-less times ("10-25 11:49",
"16:47") with dated ones; all are reproduced as recorded in `02` and are +07 local time unless
the source states otherwise.

## 5.2 Reproducibility statement

**Environment.** Model `swift-1.5-qwen3.8-27b` (27B SLM), OpenAI-compatible endpoint at
`127.0.0.1:8092/v1`, single NVIDIA RTX 3090. Daemon built from this repository
(`cargo run -p daemon --release -- --benchmark
benchmarks/assets/configs/ReCodeAgent/skel/<project>.yml`), with `WERTHAN_BASE_URL` and
`WERTHAN_MODEL` per `benchmarks/README.md` §4.

**Procedure.** Per `benchmarks/README.md` §3: workspace reset from pristine dataset
(`benchmarks/assets/ReCodeAgent/data/tool_projects/skel/<project>/python`), campaign config
from `benchmarks/assets/configs/ReCodeAgent/skel/<project>.yml`, result JSON to
`BENCH_RESULTS_DIR` (default `benchmarks/results/daemon`).

**Artifacts inventory (what survives).**

| Artifact | Path | Survives |
|---|---|---|
| Result JSONs (measured terminals, minus strsim v17) | `benchmarks/results/daemon/skel-{bst,colorsys,heapq,mathgen,toml}.json` | ✅ committed |
| strsim v17 result JSON | — | ❌ not present (figures from the original report only) |
| Fix-verify campaign logs (8 files) | `benchmarks/logs/*-fixverify.log` | ✅ committed (note: these record an earlier wave-deadlock era; e.g. `skel-strsim: check=failed … wall=1901.6s error=wave dispatch deadlock`, superseded by the 10-25 runs) |
| Simulated baseline (2026-10-18) | `benchmarks/results/recodeagent-benchmark-report.md` | ✅ committed |
| Campaign configs | `benchmarks/assets/configs/ReCodeAgent/skel/*.yml` | ✅ committed |
| Pristine sources | `benchmarks/assets/ReCodeAgent/data/tool_projects/skel/*/python/source.py` | ✅ committed (LoC verified) |
| Ground-truth snapshot | `/tmp/report-ground-truth.md` | ❌ non-persistent |
| Killed-run candidates | `benchmarks/outputs/…` (gitignored scratchpad) | ❌ scratch state, not reproducible post-kill |

**Reproduction caveats.** (1) The binary lineage matters: reproducing requires a daemon built
at the corresponding lineage point; the current tree is v18-era. (2) Inference is served by a
local model whose exact weights/serving build are outside this repository; bit-identical
reproduction is not expected. (3) The three killed runs cannot be reproduced as killed runs;
only re-runs (recommended in `04` §4.2) can produce v18 verdicts for them.

## 5.3 Raw-field cross-check (surviving JSONs)

Values below were re-verified field-by-field against the committed JSONs on 2026-10-30.

| Field | bst | colorsys | heapq | mathgen | toml |
|---|---|---|---|---|---|
| `wall_time_ms` | 22,144,759 | 14,159,337 | 11,669,900 | 17,653,051 | 61,443,312 |
| `engine_wall_time_ms` | 21,894,681 | 13,946,228 | 11,648,962 | 17,476,711 | 61,290,300 |
| `llm_requests` | 195 | 119 | 107 | 206 | 774 |
| `compilation_success` | false | false | **true** | false | false |
| `test_pass_rate` | 1.0 | 1.0 | 1.0 | 1.0 | 1.0 |
| `unimplemented_remaining` | 0 | 0 | 0 | 0 | 0 |
| `solver_dispatches` | 3 | 3 | 3 | 6 | 6 |
| `worker_executions` | 4 | 4 | 4 | 5 | 5 |
| `structural_retries` | 0 | 0 | 0 | 0 | 0 |
| `macro_states_visited` | 1 | 1 | 1 | 2 | 2 |
| `code_lines` | 601 | 277 | 329 | 1218 | 1510 |
| `total_lines` | 808 | 355 | 497 | 1684 | 2092 |
| `error` | futile cap | futile cap | futile cap | futile cap | futile cap |

Discrepancies vs the original report's narrative (preserved as ambiguities, not resolved):
the JSONs carry no timestamps (the report's start/finish times come from the ground-truth
snapshot); heapq's JSON `code_lines` is 329 while the report quotes candidate LoC 546 and
13,809 bytes from the workspace snapshot; mathgen's JSON shows `code_lines` 1218 / `total_lines`
1684 at finish while the report quotes "80 (truncated)" from the 19:00 workspace state; toml's
JSON shows 1510/2092 lines while the report quotes the collapsed 18-line/478-byte stub at
terminal. These measure different snapshots (JSON at run finalization vs workspace at
19:00/kill) and the original report did not reconcile them.

## 5.4 Facts that could not be re-verified against artifacts

- **strsim v17 run figures** (457 requests, comp=false, pass 1.0, 6 dispatches, finished 16:59):
  no strsim result JSON exists; taken from the original report.
- **Run start/finish wall-clock times** for all runs (16:47, 16:49, 17:07, 09:33, 09:35, 11:49,
  04:39, 20:03, 21:43, 22:20 kill): the JSONs carry no timestamps; taken from the original
  report / ground-truth snapshot.
- **Node probe results @ 18:58** (rbt exit 0; html exit 1 timeout > 15 s): from the
  ground-truth snapshot only.
- **Candidate line/byte counts at 19:00 and at kill** (html 1349→76 lines / 28,756 bytes;
  strsim 1216 LoC → 87 bytes; rbt 776 lines / 23,654 bytes; heapq 546 / 13,809): workspace
  snapshots, not in surviving artifacts.
- **v17 boundary times (12:54) and v18 build time (16:49)**: taken from the original report.
- **heapq v17 run** (comp=true, pass=0.0, 194 requests, died at 3 dispatches): no artifact.
- **Watchdog restart cap hits 04:00–04:32 (html/strsim/toml/mathgen) and transport 500
  warnings with 420–585 ms backoff**: from the original report's ops observations
  (`06-appendix-ops-timeline.md`).
