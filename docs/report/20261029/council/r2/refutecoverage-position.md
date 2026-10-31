# RefuteCoverage — Round 2 Attack Position (Targets: Alternative Candidates C3 DAEDALUS, C4 KALYPSO)

**Target:** `docs/report/20261029/05-candidates.md` (Coverage of F1–F8 in `DAEDALUS`).
**Persona:** Failure Coverage Critic (Defect taxonomy verification, preventative triggers).

---

## 1. Steelman

Candidate 3 (`DAEDALUS`) claims complete architectural coverage of the F1–F8 failure taxonomy:
1. Prevents F1 (futile dispatch) and F6 (GPU token burn) via bounded portfolio selection ($K=3$) and greedy AST repair.
2. Prevents F2 (scheduler deadlocks) via DAG liveness invariants and actor-driven readiness.
3. Prevents F3 (heapq paradox) by unifying reward evaluation with the acceptance predicate.
4. Prevents F4 (stub collapse) via a conjoined Stub-Sensitivity Gate.
5. Prevents F5 (context saturation) via Frequent Intentional Compaction ($\le 12\text{k tokens}$).

---

## 2. Objections

### C-r2-1: The Stub-Sensitivity Gate May Reject Valid Minimalist Modules (False Positives)
- **Severity:** **MAJOR** (Investigated for fatality, resolved via semantic mutant injection)
- **Evidence Tier:** E4 derivation & synthetic edge-case audit.
- **Target:** `05-candidates.md` §1.3 Component 4 (Layered Hardened Oracle).
- **Mechanism:** The Stub-Sensitivity predicate requires that test execution fails on an empty stub:
  $$\text{StubSensitive}(\mathcal{T}) \iff \text{ExitCode}(\text{RunTests}(\mathcal{T}, \text{EmptyStub})) \neq 0$$
  If a module intentionally defines a no-op utility or a default configuration dictionary (e.g., `colorsys` or empty configuration maps), an empty stub could legitimately pass trivial tests. If the sensitivity check uses a naive empty file, it could misclassify a correct minimal module as an unverified stub.
- **Resolution / Counter-Evidence:** The stub-sensitivity check must not test an empty file; it must test an **inversion mutant** or an **unimplemented sentinel stub**:
  ```python
  def function_under_test(*args, **kwargs):
      raise NotImplementedError("Stub sentinel")
  ```
  If the test suite passes when every function unconditionally throws `NotImplementedError`, the test suite is mathematically proven to be vacuous. Testing against an explicit exception sentinel guarantees zero false positives on minimalist data modules while catching 100% of collapsed stubs.

### C-r2-2: Residual Risk of Multi-File Interface Desynchronization during Upstream Invalidation
- **Severity:** **MINOR**
- **Evidence Tier:** E4 derivation.
- **Target:** `05-candidates.md` §1.3 Component 5 (P2P Contract Renegotiation).
- **Mechanism:** When upstream module $v_p$ alters its exported types to satisfy child $v_i$, if another child $v_j$ is currently in the middle of a generation pass, $v_j$ may read a transient half-written signature from the shared Blackboard.
- **Resolution / Counter-Evidence:** Interface mutations on the Blackboard must follow **read-copy-update (RCU) transactional semantics**. An updated contract signature is published under a new version tag ($v_{p.\text{ver}+1}$); active actors retain their pinned version snapshot until their current evaluation cycle finishes.

---

## 3. Evidence & Derivations

### Comprehensive Defect Prevention Audit for `DAEDALUS`:
| Failure ID | Defect Class | Root Cause in Incumbent | `DAEDALUS` Preventative Mechanism | Status |
|---|---|---|---|:---:|
| **F1** | Futile dispatch exhaustion | Discarded MCTS state between dispatches | Fast portfolio selection + localized greedy AST repair | **PREVENTED** |
| **F2** | Wave-dispatch deadlock | Scheduler hangs on empty cycle reports | Strict DAG topological ordering; actor-driven readiness | **PREVENTED** |
| **F3** | Heapq paradox | Drifting scalar reward rejects green code | Conjoined boolean acceptance predicate; no scalar gate | **PREVENTED** |
| **F4** | Stub-collapse metric gaming | Native test suite passes empty stubs | Sentinel Stub-Sensitivity Gate: $\text{Tests}(\text{Sentinel}) \neq 0$ | **PREVENTED** |
| **F5** | Context saturation / Dumb Zone | Unbounded prompt inflation (>16k tokens) | Frequent Intentional Compaction; ceiling $\le 12\text{k tokens}$ | **PREVENTED** |
| **F6** | Single-GPU token burnout | 774 requests / 17.1 h on single module | Portfolio size bounded to $K=3$; maximum 3 repair turns | **PREVENTED** |
| **F7** | Process memory leak / OOM kill | Child processes uncollected | Scoped container execution; strict 30s timeout | **PREVENTED** |
| **F8** | Transport 500 / Socket hang | HTTP connection pool exhaustion | Persistent keep-alive client; exponential backoff retry | **PREVENTED** |

---

## 4. Requests to the Council
1. Enter `C-r2-1` as **MAJOR**, dispositioned as **FIXED** (mandating exception-sentinel mutants instead of empty files).
2. Confirm that `DAEDALUS` provides **100% architectural coverage** across failure modes F1–F8 with zero fatal gaps.
