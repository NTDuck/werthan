# 6 · Appendix — Operations Timeline (demoted from main body)

Ops events are contextual, not verdicts. They explain *why* runs lack measurements; they do
not enter any success rate.

## 6.1 Timeline of operational events (2026-10-25, +07)

| Time | Event |
|---|---|
| 04:00–04:32 | html/strsim/toml/mathgen repeatedly hit the watchdog restart cap (3 per campaign): hang-restarts and error-exit restarts driven by transport 500s and the socket-hang shape. |
| 04:39 | toml campaign started on the old pre-v18 binary. |
| 09:33 / 09:35 | mathgen / colorsys finished-error (pre-fix binaries). |
| 11:49 | bst finished-error (v16-era binary). |
| 12:54 | v17 binary built (parentless fault gate). |
| 16:47 | v18 daemons started: heapq, rbt. |
| 16:49 | v18 binary built; html v18 daemon started. |
| 16:59 | strsim v17 run finished-error. |
| 17:07 | strsim v18 daemon started. |
| 18:58 | Node probes: rbt candidate exit 0 (passes); html candidate exit 1 (timeout, main loop > 15 s burn). |
| 19:00 | Ground-truth snapshot taken (`/tmp/report-ground-truth.md`): html candidate 1349 lines, strsim 1216 LoC on disk. |
| 20:03 | heapq finished (v18): PASS, all 4 criteria. |
| 21:43 | toml finished (old binary): FAIL, 17:04 wall, stub collapse. |
| 22:20 | **Host OOM kill.** The kernel killed the session scope (`app-ghostty-surface-transient`), taking the rbt/html/strsim v18 daemons with it before any honest terminal. No result JSONs. |
| 2026-10-26 07:00 | Postscript status check: all campaigns terminal or killed. |

## 6.2 Recurring transport failures

- **Transport 500s.** Recurring `inference endpoint returned retryable status ... status=500`
  warnings; the daemon retries with 420–585 ms backoff.
- **Hang shape.** The 2026-10-23 socket-hang class; mitigated by watchdog restarts and the
  600 s `inference_timeout`, but it consumed restart budget earlier in the day.
- **Watchdog policy.** Hang-restart + error-exit-restart capped at 3 per campaign.

## 6.3 The 22:20 OOM kill (root cause of stratum B)

The kernel killed the session scope (`app-ghostty-surface-transient`) under host memory
exhaustion — 5 concurrent daemons + the desktop session exceeded host RAM. The kill destroyed
three in-flight v18 runs (rbt, html, strsim) before terminal:

- No result JSONs were written for them.
- On-disk candidates were left truncated mid-write (html: 76 lines vs 1349 at 19:00; strsim:
  87-byte 1-line export fragment vs 1216 LoC) or intact (rbt: 776 lines / 23,654 bytes).
- These runs are unmeasured, not failed (`02` §2.2). Recommended mitigation before re-runs:
  stagger daemons or cap concurrent campaigns (`04` §4.5, rank 2).
