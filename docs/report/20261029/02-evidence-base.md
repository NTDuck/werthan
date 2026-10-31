# 02 — Evidence Base & Operational Definitions

This document establishes the empirical ground truth, failure taxonomy (F1–F8), and operational definitions for all evaluations in the 20261029 report. Every quantitative claim is grounded in surviving primary artifacts and classified by evidence tier (E1–E4) per `01-meta-strategy.md` §4.

---

## 1. Primary Empirical Artifacts (E1)

The empirical base derives from real-inference daemon campaigns against the ReCodeAgent benchmark (`skel` category, Python→JavaScript) executed on an NVIDIA RTX 3090 with `swift-1.5-qwen3.8-27b` (27B SLM @ `127.0.0.1:8092/v1`).

### Surviving Result JSONs (`benchmarks/results/daemon/`)

| Project | Binary Stratum | Finished | Wall Time (ms) | LLM Requests | Compilation | Test Pass Rate | Stubs Remaining | Solver Dispatches | Macro States | Terminal Error |
|---|---|---|---|---|---|---|---|---|---|---|
| **heapq** | v18 | 20:03 | 11,669,900 (~3.2 h) | 107 | **true** | 1.0 | 0 | 3 | 1 | `module source.py exhausted the futile dispatch cap after 3 attempts` |
| **toml** | old pre-v18 | 21:43 | 61,443,312 (~17.1 h) | 774 | **false** | 1.0 | 0 | 6 | 2 | `module source.py exhausted the futile dispatch cap after 3 attempts` |
| **strsim** | v17 | 16:59 | — | 457 | **false** | 1.0 | 0 | 6 | — | `module source.py exhausted the futile dispatch cap after 3 attempts` |
| **mathgen** | pre-fix | 09:33 | 17,653,051 (~4.9 h) | 206 | **false** | 1.0 | 0 | 6 | 2 | `module source.py exhausted the futile dispatch cap after 3 attempts` |
| **bst** | pre-fix (v16) | 11:49 | 22,144,759 (~6.2 h) | 195 | **false** | 1.0 | 0 | 3 | 1 | `module source.py exhausted the futile dispatch cap after 3 attempts` |
| **colorsys** | pre-fix | 09:35 | 14,159,337 (~3.9 h) | 119 | **false** | 1.0 | 0 | 3 | 1 | `module source.py exhausted the futile dispatch cap after 3 attempts` |

### Unmeasured Kills (Stratum B — Host OOM at 22:20 +07)

- **rbt (v18):** Daemon killed by host OOM before terminal. Candidate intact (776 lines, 23,654 bytes); node probe passed @ 18:58; no result JSON.
- **html (v18):** Daemon killed by host OOM before terminal. Candidate truncated mid-write (76 lines vs 1,349 at 19:00); node probe timed out @ 18:58 (main loop >15 s burn); no result JSON.
- **strsim (v18):** Daemon killed by host OOM before terminal. Candidate truncated mid-write (87-byte fragment); no result JSON.

---

## 2. Failure Taxonomy (F1–F8)

The failure taxonomy catalogs the exact failure modes observed across the daemon campaigns.

| ID | Failure Mode | Class | Primary Evidence | Methodology / Engine Mechanism Stressed |
|---|---|---|---|---|
| **F1** | **Fabricated-Counterparty Futile-Cap Death** | Engine Control Defect | bst/colorsys/mathgen/strsim-v17 JSONs: `ModuleRetryExhausted` after 3 dispatches. `resolve_parent_id()` fabricated `"parent_upstream"` on parentless single-module DAGs → solver bridge remapped `BudgetExhausted` to `ContractConflict` → Tier B broadcast against nonexistent parent → futile counter incremented → terminal death at cap 3. | Tier B conflict semantics (`METHODOLOGY.md` §5.2 requires conflict to anchor a real edge $e_{pi}$); solver bridge outcome mapping. |
| **F2** | **Wave-Dispatch Deadlock** | Engine Scheduler Defect | 6/6 terminal fixverify logs: `no ready module while unhardened modules remain`, unimplemented=0, up to 8.1 h. | Wave scheduler ReadySet liveness (`METHODOLOGY.md` §4: ReadySet defined over `M0_Blocked` with hardened parents; invalid Tier C reports exited without parking reporter → orphaned unvisited modules). |
| **F3** | **Reward-vs-Artifact Gap (heapq Paradox)** | Calibration / Metric Defect | heapq v18 JSON: all four acceptance criteria green (`comp=true`, pass 1.0, 0 stubs, 107 reqs) **yet** terminal `error = futile dispatch cap`. Workspace held a green candidate, but reward scoring never certified $R = 1.0$, so no `ev_ModuleHardened` fired and engine burned ~1 h extra budget. | Gated reward function $R(s_t)$ (`METHODOLOGY.md` §4.2) vs. external acceptance gate use disjoint verification ground truth. |
| **F4** | **Stub Collapse & Vacuous Pass** | Oracle / Metric Defect | toml old-binary run: 17.1 h wall, 774 requests, candidate collapsed to an 18-line `get_input` stub (478 bytes); parser logic truncated away, but `target_test_cmd` exited 0 → recorded `test_pass_rate = 1.0`. | `M4` commit invariants; test harness stub-sensitivity; oracle blindness to contract omissions. |
| **F5** | **Infrastructural OOM Terminations** | Infrastructure / Ops | rbt/html/strsim v18 killed at 22:20 when host memory exhausted by 5 concurrent daemons + desktop session. | Multi-daemon memory management; lack of bounded resident set limits. |
| **F6** | **Transport 500s & Socket Hangs** | Infrastructure / Serving | Recurring HTTP 500 retryable warnings (420–585 ms backoff); 600 s socket-hang watchdog restart cap (3) hit repeatedly 04:00–04:32. | Transport timeout bounding; inference connection keepalive. |
| **F7** | **Rollout Budget Non-Convergence** | Methodology Limitation | heapq merge-semantics bug (assertion `25 != 3`), rbt rotation failure, html main loop >15 s timeout. $B_{\max} = 16$ rollouts × 3 dispatches = 48 attempts with no warm-start carryover across dispatches and no futile-cap reset on partial reward climb. | Static rollout allocation ($B_{\max}$ uniform); absence of state-tree branch retention across dispatches. |
| **F8** | **Compilation Success Semantics Drift** | Metric Specification Defect | Pre-fix binaries conflated target compilation with test execution; restored under VAL-DAEMON-08 (`target_build_cmd` exit 0 AND zero unimplemented stubs; error in `error` field). | Definition of `compilation_success` and acceptance gate validation. |

---

## 3. Operational Definitions & Predicates

### 3.1 Hardened-Accept Predicate ($\text{Accept}_{\text{hardened}}$)

Because native benchmark test suites can be weak or vacuous (F4), raw pass rates reward specification gaming. A candidate module $c$ yields **Hardened Accept** iff all three conditions hold:

$$\text{Accept}_{\text{hardened}}(c) \iff \text{Criteria}_{\S6}(c) \;\land\; \text{StubSensitive}(\mathcal{T}) \;\land\; \text{Parity}_{\text{diff}}(c, \mathcal{R}_{\text{leg}})$$

1. **Four Acceptance Criteria ($\text{Criteria}_{\S6}$):**
   - Clean target compilation: $\mathbb{I}_{\text{compile}}(c) = 1$ (`node -c translated.js` exits 0).
   - Characterization test pass: $\text{PassRate}_{\text{char}}(c, \mathcal{T}) = 1.0$ (`node translated.js` exits 0).
   - Zero unimplemented stubs: $\text{StubsRemaining}(c) = 0$.
   - Real inference receipts: $\text{LLMRequests}(c) > 0$.
2. **Stub-Sensitivity Gate ($\text{StubSensitive}$):**
   - The test suite $\mathcal{T}$ MUST fail when executed against an empty or collapsed stub implementation. If a stub passes $\mathcal{T}$, $\mathcal{T}$ is defective and cannot certify acceptance.
3. **Differential Behavioral Parity ($\text{Parity}_{\text{diff}}$):**
   - Whenever the legacy source is executable (always in `skel` and `crust`), execution of identical inputs across legacy and target implementations must achieve $\ge 0.95$ normalized output similarity (or exact byte equality when `require_byte_identical_io: true`).

### 3.2 Conditional Cost Mixture ($\text{Cost}_{\text{mixture}}$)

To prevent dimensional degeneracy, cost is formulated as the expectation over the conditional mixture of successes and failures:

$$\mathbb{E}[\text{Cost}] = \Pr[\text{Accept}] \cdot \mathbb{E}[\text{Cost} \mid \text{Accept}] + (1 - \Pr[\text{Accept}]) \cdot \mathbb{E}[\text{Cost} \mid \text{Fail}]$$

where cost is measured in GPU-seconds:

$$\text{Cost} = \frac{\text{Tokens}_{\text{in}} + \text{Tokens}_{\text{out}}}{\mu_{\text{throughput}}} + T_{\text{deterministic\_tools}}$$

with measured aggregate inference throughput $\mu_{\text{throughput}} \approx 268 \text{ tokens/s}$ on the local RTX 3090, discounted for concurrent worker slot contention.

### 3.3 Pareto-Optimal Methodology ($\pi^\star$)

A candidate methodology $\pi$ is evaluated on the multi-objective vector:

$$\mathbf{J}(\pi) = \Big\langle \Pr\big[\text{Accept}_{\text{hardened}}\big], \; -\mathbb{E}\big[\text{Cost}_{\text{mixture}}\big], \; \text{Locality}(\pi), \; \text{SLM\_Robustness}(\pi) \Big\rangle$$

where:
- $\text{Locality}(\pi) \in [0, 1]$ measures the fraction of rollbacks contained to Tier A (intra-module) or Tier B (DAG 1-hop neighborhood) without triggering global macro-state rollbacks ($S_5 \to S_2$).
- $\text{SLM\_Robustness}(\pi) \in [0, 1]$ measures the probability that prompt envelopes remain within the SLM "smart zone" ($\le 16\text{k tokens}$) and do not degrade into degenerate loops under constrained local reasoning.
