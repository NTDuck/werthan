# Council Ledger (Mirror of 06-audit-ledger.md)

*This file is the official mirror of `docs/report/20261029/06-audit-ledger.md` maintained under the protocol of `01-meta-strategy.md` §5.3.*

---

## Master Ledger Summary

- **Total Rounds Executed:** 5 (Round 0 calibration, Round 0.5 protocol self-attack, Round 1 incumbent attack, Round 2 candidate attack, Round 3 re-attack & convergence check).
- **Total Position Papers Filed:** 34 across `council/r0/`, `council/r0.5/`, `council/r1/`, `council/r2/`, `council/r3/`.
- **Total Objections Ledgered:** 30 formal findings.
- **Consecutive Fatal-Free Rounds:** 2 (Round 2: 0 fatal; Round 3: 0 fatal).
- **Outcome:** **CONVERGED** — Unanimous ratification of **`DAEDALUS` ($C_3$)**.

---

## Multi-Round Objection & Disposition Ledger

### Round 1: Attacks on the Incumbent (`THESEUS` / `METHODOLOGY.md`)
| Finding ID | Lens | Target in `METHODOLOGY.md` | Severity | Tier | Objection Summary | Formal Disposition & Text Amendment |
|---|:---:|---|:---:|:---:|---|---|
| **T-r1-1** | T | §4.2 Acceptance Criteria | **FATAL** | E1/E4 | $R(s_t) = 1.0$ is not sufficient for semantic parity; passes 18-line empty stub (`skel-toml.json`). | **UPHELD.** Scalar reward replaced by conjoined boolean predicate $\text{Accept}_{\text{hardened}}$ with active Stub-Sensitivity Gate. (`02` §3.1, `03` RQ1) |
| **T-r1-2** | T | §4.1 Softmax-UCB | **MAJOR** | E4 | Softmax over UCB destroys polynomial regret bound under non-stationary LLM judge noise. | **UPHELD.** Replaced MCTS with parallel Portfolio Race ($K=3$) and zero-token compiler pre-filtering. (`03` RQ3, `05` §1) |
| **T-r1-3** | T | §1.2 State-Space Claim | **MAJOR** | E4 | $O(m^{|V|})$ blowup claim is a strawman; topological sort over DAGs requires only $O(\|V\|+\|E\|)$. | **UPHELD.** Conflation documented; dependency management mapped directly to topological DAG order. (`03` RQ2) |
| **F-r1-1** | F | §5.2 Tier B Rollback | **FATAL** | E1 | Single-vertex modules lack parent edge; `resolve_parent_id()` fabricated `"parent_upstream"`, crashing engine. | **UPHELD.** Fabricated parent generation removed; single-vertex budget exhaustion mapped to honest termination. (`02` §2, `03` RQ2) |
| **F-r1-2** | F | §4 Alg 1 / §5.3 Tier C | **MAJOR** | E1 | Scheduler deadlocks when cycle detection returns empty report; module never returned to ReadySet. | **UPHELD.** ReadySet liveness invariant enforced: non-terminal modules must be either runnable or parked. (`02` §2, `03` RQ2) |
| **F-r1-3** | F | §2 Shared Blackboard | **MAJOR** | E4 | Central Tokio `RwLock<Blackboard>` serializes execution during multi-subagent concurrent access. | **UPHELD.** Adopted localized event-driven Actor model with Read-Copy-Update (RCU) versioned snapshots. (`05` §1) |
| **C-r1-1** | C | §4.2 Acceptance Gate | **FATAL** | E1 | Incumbent has zero preventative mechanisms for stub-collapse (F4); records failure only post-hoc. | **UPHELD.** Mandated active exception-sentinel mutant testing prior to module hardening. (`02` §2, `03` RQ1) |
| **C-r1-2** | C | §4 M4/M5 Transitions | **MAJOR** | E1 | Heapq paradox (F3): valid, fully functioning candidate rejected because scalar reward yielded $R=0.88$. | **UPHELD.** Unified acceptance criteria: if characterization and differential tests pass, code is committed. (`02` §2, `03` RQ1) |
| **C-r1-3** | C | §5.3 Tier C Cycles | **MAJOR** | E1 | Tier C $H^*$ restore addresses circular imports, but 100% of the 118 benchmark projects are strictly acyclic. | **UPHELD.** Tier C stripped as dead weight; static cycle preflight assertion added to $S_0$. (`03` RQ2, `04` §1) |
| **L-r1-1** | L | §3 Subagent Topology | **MAJOR** | E2 | Autonomous multi-turn agent loops empirically underperform simpler hierarchical workflows (*Agentless*). | **UPHELD.** Replaced multi-turn ReAct loops with two-stage localization and AST repair. (`03` RQ4, `07`) |
| **L-r1-2** | L | §4 Tree Search | **MAJOR** | E2 | Single-draft iterative mutation is strictly dominated by initial diverse portfolio generation (*AlphaCode*). | **UPHELD.** Adopted parallel Portfolio Race generating $K=3$ diverse initial hypotheses. (`05` §1, `07`) |
| **M-r1-1** | M | §4.2 Test Verification | **FATAL** | E1 | Native oracle lacks stub-sensitivity; vacuous test suites pass trivial return stubs (`skel-toml.json`). | **UPHELD.** Formalized Stub-Sensitivity Gate: $\text{ExitCode}(\text{Run}(\mathcal{T}, \text{Stub})) \neq 0$. (`02` §3.1) |
| **M-r1-2** | M | §4.2 Reporting Fields | **MAJOR** | E1 | Pre-fix binaries conflated compilation failure with 100% test pass rates. | **UPHELD.** Gating invariant enforced: test runner never executes on non-compiling targets. (`02` §1, `05` §1) |
| **S-r1-1** | S | §4 MCTS Rollout Budget | **FATAL** | E1 | Unpruned MCTS rollouts burned 774 requests / 17.1 hours on `toml`, exceeding single-GPU throughput limits. | **UPHELD.** MCTS replaced by bounded Portfolio Race ($K=3$) with zero-token compiler filtering. (`03` RQ3, `05` §1) |
| **S-r1-2** | S | §2 Memory Model | **FATAL** | E2 | Absence of compaction pushes 27B SLM into the "Dumb Zone" (>16k tokens), inducing repetitive repair loops. | **UPHELD.** Implemented Frequent Intentional Compaction maintaining prompt envelope $\le 12\text{k tokens}$. (`03` RQ4, `05` §1) |
| **B-r1-1** | B | §1 Baseline Set | **MAJOR** | E2 | Omission of ReCodeAgent ($C_0$) and CentralLedger ($C_1$) as formal executable baselines. | **UPHELD.** ReCodeAgent and CentralLedger integrated into formal 9-axis candidate scoring rubric. (`04` §2, `05` §1) |

---

### Round 2: Attacks on Alternative Candidates (`DAEDALUS` $C_3$, `KALYPSO` $C_4$)
| Finding ID | Lens | Target in Candidate $C_3$ | Severity | Tier | Objection Summary | Formal Disposition & Text Amendment |
|---|:---:|---|:---:|:---:|---|---|
| **T-r2-1** | T | P2P Contract Renegotiation | **MAJOR** | E4 | P2P renegotiation risks ping-pong rollback loops across sibling actors if parent interface changes. | **FIXED.** Constrained contract patches to **monotonic union expansion** ($\Sigma' \supseteq \Sigma$) and capped renegotiation at 2 hops. (`05` §1.3) |
| **F-r2-1** | F | DAG Actor Concurrency | **MAJOR** | E1 | Unbounded concurrent actor generation saturates 24 GB VRAM KV-cache, triggering CUDA OOM. | **FIXED.** Added physical inference semaphore capping active LLM generation to $\text{MaxConcurrentWorkers} = 2$. (`05` §1.3) |
| **C-r2-1** | C | Stub-Sensitivity Gate | **MAJOR** | E4 | Naive empty-file stub test could reject legitimate minimalist data modules (false positives). | **FIXED.** Replaced empty-file test with **Semantic Exception-Sentinel Mutation** (`raise NotImplementedError`). (`05` §1.3) |
| **L-r2-1** | L | Portfolio Size $K=3$ | **MAJOR** | E2 | AlphaCode used thousands of samples; is $K=3$ sufficient to capture translation variability? | **ACCEPTED-WITH-RATIONALE.** Rationale: Translation search space is bounded by source AST; $K=3$ distinct prompt templates capture $\ge 85\%$ variance per literature. (`07`) |
| **M-r2-1** | M | Differential Parity $\ge 0.95$ | **MAJOR** | E1 | Differential testing fails on non-deterministic outputs (timestamps, random seeds, hash map orders). | **FIXED.** Mandated **semantic output normalization** (JSON key sorting, epsilon float bands, regex pointer masking). (`05` §1.3) |
| **S-r2-1** | S | 10-Line Fault Card | **MAJOR** | E1 | Arbitrary 10-line error truncation risks cutting off critical compiler trait bounds and suggestion notes. | **FIXED.** Replaced raw truncation with **Syntax-Aware JSON Diagnostic Parsing** (`rustc --error-format=json`). (`05` §1.3) |
| **B-r2-1** | B | Baseline $C_0$ Scoring | **MAJOR** | E1 | Must ensure $C_0$ (ReCodeAgent) is scored under the exact same 27B SLM regime, not frontier models. | **ACCEPTED-WITH-RATIONALE.** Rationale: Fixed deployment regime held strictly uniform across all 5 candidates. (`05` §2) |

---

### Round 3: Re-Attacks on Repaired Candidates & Convergence Check
| Finding ID | Lens | Target in Repaired $C_3$ | Severity | Tier | Re-Audit Finding Summary | Convergence Verdict |
|---|:---:|---|:---:|:---:|---|:---:|
| **T-r3-1** | T | Monotonic Contract Invariant | **MINOR** | E4 | Audit confirms monotonic contract addition guarantees acyclic DAG termination; ambiguity resolved by path qualification. | **CLEAN (Fatal-Free)** |
| **F-r3-1** | F | Worker Pool & Sandboxing | **MINOR** | E1 | Audit confirms 2-worker semaphore keeps peak VRAM at $\le 21\text{ GB}$; child processes safely reaped by 30s/60s deadlines. | **CLEAN (Fatal-Free)** |
| **C-r3-1** | C | Sentinel Mutant Injection | **MINOR** | E4 | Audit confirms exception-sentinel mutation eliminates 100% of false positives while blocking collapsed stubs. | **CLEAN (Fatal-Free)** |
| **L-r3-1** | L | Literature Grounding | **MINOR** | E2 | Audit confirms all 12 primary citations in `07-bibliography.md` are verified against public repositories. | **CLEAN (Fatal-Free)** |
| **M-r3-1** | M | Output Normalization | **MINOR** | E4 | Audit confirms semantic normalization preserves true functional equivalence without masking logic errors. | **CLEAN (Fatal-Free)** |
| **S-r3-1** | S | Structured Fault Cards | **MINOR** | E1 | Audit confirms JSON compiler diagnostics extract actionable repair hints within $\le 300\text{ tokens}$; context stays $\le 12\text{k}$. | **CLEAN (Fatal-Free)** |
| **B-r3-1** | B | Dominance Table Recalculation | **NON-ISSUE** | E4 | Audit confirms Candidate 3 strictly Pareto-dominates Candidate 2 across all 9 evaluation axes. | **CLEAN (Fatal-Free)** |
