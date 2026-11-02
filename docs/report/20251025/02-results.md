# 2 · Results

Verdicts follow the exact four-condition criteria of `01-setup.md` §1.4. Results are
**stratified by binary version**; measured terminal verdicts (stratum A) are never blended with
unmeasured kills (stratum B). Wall times are normalized to h:mm from the raw `wall_time_ms`
fields (raw values preserved in `05-appendix-validity.md`).

## 2.1 Stratum A — measured terminal verdicts (result JSON exists)

| Project | Binary stratum | Finished (2026-10-25, +07) | Wall time | LLM requests | Compilation | Test pass rate | Solver dispatches | Worker exec. | Macro states | Candidate LoC / bytes | Verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|
| heapq | v18 | 20:03 | 3:14 | 107 | **true** | 1.0 | 3 | 4 | 1 | 546 / 13,809 | **PASS** (all 4 criteria; `error` field noted — see `03` §3.5) |
| toml | old pre-v18 | 21:43 | 17:04 | 774 | **false** | 1.0 | 6 | 5 | 2 | 18 (collapsed) / 478 | **FAIL** |
| strsim | v17 | 16:59 | — (no v17 JSON; 457 requests, comp=false, pass 1.0, 6 dispatches recorded in report) | 457 | **false** | 1.0 | 6 | — | — | — | **FAIL** |
| mathgen | pre-fix | 09:33 | 4:54 | 206 | **false** | 1.0 | 6 | 5 | 2 | 80 (truncated) / 1218 code · 1684 total at finish | **FAIL** |
| bst | pre-fix (v16-era) | 11:49 | 6:09 | 195 | **false** | 1.0 | 3 | 4 | 1 | 601 / 808 (total) | **FAIL** |
| colorsys | pre-fix | 09:35 | 3:56 | 119 | **false** | 1.0 | 3 | 4 | 1 | 277 / 355 (total) | **FAIL** |

Notes per row, preserving original qualifiers:

- **heapq (v18)** — honest terminal: the module parked at futile dispatch 3 with an
  all-4-criteria workspace (comp true, pass 1.0, 0 stubs, 107 requests). The recorded `error`
  is the futile-cap message, but the artifact passes every §6 criterion — reward scoring never
  certified the already-green candidate (`03` §3.5).
- **toml (old pre-v18)** — started 04:39, terminated 21:43 after 61,443,312 ms (17:04 wall,
  8.5× over the 2 h bound): `compilation_success = false`, `test_pass_rate = 1.0`, 774 LLM
  requests, 6 solver dispatches across 2 macro states, terminal error the futile-cap message.
  The candidate **collapsed** to an 18-line/478-byte `get_input` stub — the parser module was
  truncated away at terminal (the test command still exited 0, so pass 1.0 reflects a test
  suite that passes against the stub); comp=false gates it out. The stale 10-24 JSON (wave
  dispatch deadlock, 14 reqs) is superseded by this run. toml has not been measured on a v18
  binary and should be re-run.
- **strsim (v17)** — futile-cap death (terminal error verbatim in `03` §3.1). The v17 row's
  wall time and candidate fields were not recorded per-run in the surviving sources; the 457
  requests / comp=false / pass 1.0 / 6 dispatches figures come from the original report.
- **mathgen (pre-fix)** — futile dispatch cap after 3 attempts; candidate truncated in
  workspace (80 lines vs 1218 code lines in the JSON at finish).
- **bst, colorsys (pre-fix)** — terminal error `module source.py exhausted the futile dispatch
  cap after 3 attempts` via the fabricated `parent_upstream` conflict path; both within the 2 h
  window but comp=false.

## 2.2 Stratum B — unmeasured kills (no result JSON; excluded from all rates)

All three v18 daemons were **OOM-killed at 2026-10-25 22:20 +07** — the kernel killed the
session scope (`app-ghostty-surface-transient`), taking the daemons with it before any honest
terminal. No result JSONs exist for them; last-known candidate state is recorded below. These
are **not interpretable v18 data points** and are excluded from every rate in this report.

| Project | v18 started | Killed | Wall time at kill | Candidate on disk at kill | Node probe @ 18:58 | Verdict |
|---|---|---|---|---|---|---|
| rbt | 16:47 | 22:20 | ~5:30 (killed before terminal) | intact, 776 lines / 23,654 bytes | **0 (passes)** | ⏳ no verdict |
| html | 16:49 | 22:20 | ~5:30 (killed before terminal) | truncated mid-write: 76 lines vs 1349 at 19:00; 28,756 bytes | **1 (timeout)** (main loop > 15 s burn) | ⏳ no verdict |
| strsim | 17:07 | 22:20 | ~5:12 (killed before terminal) | truncated mid-write to a 1-line export fragment: 87 bytes vs 1216 LoC at 19:00 | — | ⏳ no verdict (v17 run: FAIL, stratum A) |

- **rbt** — candidate on disk intact and node probe = **0 (passes)** @ 18:58, but reward state
  at kill unknown; prior failure mode was the red-black rotation test fail.
- **html** — on-disk candidate truncated mid-write (76 lines vs 1349 at 19:00); node probe was
  **1 (timeout)** @ 18:58 (main loop > 15 s burn).
- **strsim** — v18 daemon OOM-killed; on-disk candidate truncated mid-write to a 1-line export
  fragment (87 bytes vs 1216 LoC at 19:00). Also hit the watchdog restart cap (3) earlier
  04:00–04:32 (`06-appendix-ops-timeline.md`).

## 2.3 Verdict summary (no blending)

- **Measured terminal verdicts: 1 PASS / 5 FAIL** (6 runs). heapq meets all four acceptance
  criteria — the first honest real-inference `skel` PASS; every other measured run died with
  `compilation_success == false` on the fabricated-conflict/futile-cap path.
- **Unmeasured kills: 3** (rbt, html, strsim v18 — host OOM at 22:20, no result JSONs). They
  count toward neither success nor failure.
- Only the heapq row is a final v18 verdict; rbt/html/strsim v18 remain unmeasured and need
  re-runs.

## 2.4 Aggregates (stratum A only)

| Aggregate | Value |
|---|---|
| Total `llm_requests` across the 6 measured runs | **1,858** = 119 (colorsys) + 206 (mathgen) + 195 (bst) + 457 (strsim v17) + 774 (toml old-binary) + 107 (heapq v18) |
| … of which on honest pre-fix/v17/v18 runs | 1,084 |
| … of which on the old-binary toml run | 774 |
| Terminal error shared by the 5 FAILs | `module source.py exhausted the futile dispatch cap after 3 attempts` |
| `test_pass_rate` across all 6 measured runs | 1.0 (see `03` §3.6 for the semantics caveat) |
| `compilation_success` | true only for heapq v18 |

Wall times are reported per run in §2.1; strsim's v17 wall time is not recoverable from
surviving artifacts.

## 2.5 Baseline contrast (simulated vs real)

| Metric | Simulated (2026-10-18 ReCodeAgent report) | Real (2026-10-25, stratum A) |
|---|---|---|
| skel compile rate | 100.0% | 16.7% (1/6 measured terminal runs) |
| skel mean pass rate | 87.5% | 100% of measured rows read 1.0, but under the drift-affected semantics of `03` §3.6 — not comparable |
| skel passed | 7/8 | 1/8 honest (heapq); 3/8 unmeasured |

The 2026-10-18 numbers were produced under stub-campaign semantics with simulated receipts
(mean wall time 65.12 ms per skel project), not real engine runs. The 2026-10-25 runs are the
first honest `skel` measurements. The two columns are **not directly comparable**: different
verdict semantics, different binaries, and (for stratum B) missing measurements.
