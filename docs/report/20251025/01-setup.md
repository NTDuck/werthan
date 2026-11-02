# 1 · Experimental Setup

All evidence in this report is grounded in `/tmp/report-ground-truth.md` (status @
2026-10-25 19:00 +07), cross-checked against `benchmarks/results/daemon/skel-{bst,colorsys,mathgen,toml}.json`
(and `skel-heapq.json` post-terminal) — see `05-appendix-validity.md` for the field-level
cross-check table.

## 1.1 Model and serving stack

| Component | Value |
|---|---|
| Model | `swift-1.5-qwen3.8-27b` (27B SLM) |
| Serving endpoint | OpenAI-compatible at `127.0.0.1:8092/v1` |
| GPU | NVIDIA RTX 3090 (single card, local inference) |
| Dataset category | `skel` (ReCodeAgent), 8 projects |

## 1.2 Dataset category

`skel`: Python→JavaScript translation (PL1 = python, PL2 = javascript, target executed via
`node`); the source file is `source.py`. Per-project legacy LoC (verified against
`benchmarks/assets/ReCodeAgent/data/tool_projects/skel/*/python/source.py`):

| Project | Source LoC | Config tier | `ariadne.max_rollouts` | `max_shards` | `inference_max_tokens` | self-heal retries |
|---|---|---|---|---|---|---|
| bst | 808 | T2 | 32 | 4 | 40,960 | 4 |
| colorsys | 355 | T1 | 16 | 2 | 32,768 | 3 |
| heapq | 497 | T1 | 16 | 2 | 32,768 | 3 |
| html | 1479 | T2 | 32 | 4 | 40,960 | 4 |
| mathgen | 1636 | T2 | 32 | 4 | 40,960 | 4 |
| rbt | 906 | T2 | 32 | 4 | 40,960 | 4 |
| strsim | 1591 | T2 | 32 | 4 | 40,960 | 4 |
| toml | 2060 | T3 | 64 | 6 | 49,152 | 6 |

Configs follow the LoC-tier scaling of ADR-0008
(`benchmarks/assets/configs/ReCodeAgent/skel/<project>.yml`). Common to all: build command
`node -c translated.js`, test command `node translated.js`, `inference_temperature 0.2`,
`require_byte_identical_io: false`.

## 1.3 Engine / binary lineage — instrumentation-version confound

The campaign ran across a moving daemon binary. Version boundaries are treated as an
instrumentation confound throughout this report: results are only comparable within a stratum.

| Stratum | Window | Behavior |
|---|---|---|
| **Pre-fix (v1–v16)** | before 2026-10-25 12:54 | Fabricated a `parent_upstream` counterparty for parentless modules → Tier B broadcast against nothing → futile cap after 3 dispatches (plus a wave-deadlock variant). |
| **v17** | built 12:54 | Treated the fault as intrinsic (parentless gate) but still remapped `BudgetExhausted` to a conflict in the solver bridge → `futile++`. |
| **v18** | built 16:49 (current) | Honest `SolverOutcome::BudgetExhausted`: parentless module with exhausted rollout budget parks in M0_Blocked, counts futile, fires **no** Tier B broadcast. VAL-M2-05 contract + specs updated; 277 workspace tests green, 0 clippy violations. |

Run-start assignments (as recorded in the original report; some rows carry no recorded start
time and are annotated):

- bst, colorsys, mathgen — finished on pre-fix binaries (bst explicitly noted "v16-era").
- strsim — v17 run finished-error (16:59); v18 daemon started 17:07, later OOM-killed.
- heapq, rbt — v18 daemons started 16:47.
- html — v18 daemon started 16:49.
- toml — started 04:39 on the old pre-v18 binary (pre-`BudgetExhausted`), finished 21:43.

## 1.4 Verdict criteria (exact)

A run **PASSes** iff all four acceptance conditions from `benchmarks/README.md` §6 hold:

1. **Compilation:** clean compilation with the target toolchain — `compilation_success == true`
   (defined as `target_build_cmd` `node -c translated.js` OK **and** zero unimplemented stubs;
   terminal error lives separately in the `error` field — VAL-DAEMON-08, `bench_validation`
   reuses `CargoHarness::run_command`).
2. **Behavioral parity:** `test_pass_rate == 1.0`.
3. **No unimplemented stubs:** `unimplemented_remaining == 0`.
4. **Inference honesty:** `llm_requests > 0`.

Additionally, the expected wall-time window per project is **10 min – 2 h**
(`benchmarks/README.md` §5): under 10 min indicates bypassed inference or empty stubs; over 2 h
indicates infinite repair loops or deadlock (the daemon times out and records an error). The
window is an anomaly bound, not a fifth acceptance condition.

## 1.5 Research questions and evaluation metrics

To adhere to empirical software engineering guidelines (e.g., ACM/IEEE empirical standards), this campaign is structured around four formal research questions:

- **RQ1 (Compilation Feasibility):** Can a local 27B SLM operating within the ARIADNE intra-module statechart synthesize target code that satisfies target toolchain syntax and structural constraints (`compilation_success`) without human scaffolding?
- **RQ2 (Behavioral Parity):** What fraction of characterization test suites pass under real target execution (`test_pass_rate == 1.0`), and does passing certify semantic equivalence?
- **RQ3 (Search Budget Bounds):** How does the Softmax-UCB MCTS planner perform within constrained rollout allocations ($B_{\max} \in [16, 64]$) across LoC tiers T1–T3?
- **RQ4 (Failure Attribution):** Are terminal failures attributable to intrinsic code generation limitations, or to extrinsic statechart/control-flow faults in the orchestration runtime?

### Mathematical definitions

$$\text{CompRate}(\mathcal{P}) = \frac{1}{|\mathcal{P}|} \sum_{p \in \mathcal{P}} \mathbb{I}\big(\text{compilation\_success}(p) \land \text{unimplemented}(p) = 0\big)$$

$$\text{MeanPassRate}(\mathcal{P}) = \frac{1}{|\mathcal{P}|} \sum_{p \in \mathcal{P}} \text{test\_pass\_rate}(p)$$

$$\text{EndToEndPassRate}(\mathcal{P}) = \frac{1}{|\mathcal{P}|} \sum_{p \in \mathcal{P}} \mathbb{I}\Big(\text{compilation\_success}(p) \land \text{test\_pass\_rate}(p) = 1.0 \land \text{unimplemented}(p) = 0 \land \text{requests}(p) > 0\Big)$$
