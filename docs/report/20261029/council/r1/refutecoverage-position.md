# RefuteCoverage — Round 1 Attack Position (Targets: Incumbent METHODOLOGY.md)

**Target:** `docs/references/METHODOLOGY.md` coverage claims vs. `docs/report/20261029/02-evidence-base.md` (F1–F8 failure taxonomy).
**Persona:** Failure Coverage Critic (Empirical defects, defect-class prevention vs. recording).

---

## 1. Steelman

The incumbent methodology asserts comprehensive architectural coverage of translation failure modes through:
1. Gated rewards preventing uncompilable or unverified code from committing.
2. An intra-module MCTS search space allowing recursive backtracking on syntax and semantic errors.
3. Orthogonal statecharts isolating module failures and preventing cascading invalidation across disjoint subgraphs.

---

## 2. Objections

### C-r1-1: The Incumbent Possesses Zero Preventative Mechanisms for Stub-Collapse (F4)
- **Severity:** **FATAL**
- **Evidence Tier:** E1 receipt (`skel-toml.json`) + formal spec audit.
- **Target:** `METHODOLOGY.md` §4.2 Acceptance & Reward Specification.
- **Mechanism:** In failure mode F4, a candidate satisfies all superficial compiler and test runner checks by deleting all business logic and leaving empty function stubs. The incumbent's formal grammar and reward formula have no predicate testing:
  $$\text{Sensitivity}(\mathcal{T}, \text{Stub}) = \mathbb{I}[\text{RunTests}(\mathcal{T}, \text{EmptyStub}) \neq 0]$$
  Because tests in the benchmark suite were synthetic or checked only that no unhandled exception was thrown on instantiation, the incumbent awarded $R=1.0$ and terminated the campaign with a false PASS. The incumbent *recorded* the run, but had zero architectural mechanisms to *prevent* it.

### C-r1-2: Incumbent Guarantees Induce the "Heapq Paradox" (F3) — Valid Candidates Rejected
- **Severity:** **MAJOR**
- **Evidence Tier:** E1 receipt (`benchmarks/results/daemon/skel-heapq.json`).
- **Target:** `METHODOLOGY.md` §4 M4/M5 State Transitions.
- **Mechanism:** In the `heapq` campaign, an intact, fully functional JavaScript port (546 LoC) passed 100% of the characterization tests. However, because the evaluator model was evaluated under a drifting scalar reward, the internal reward yielded $R=0.88 < 1.0$. The engine continued to dispatch futile rollouts for 3 additional hours until hitting the futile dispatch cap, recording an error in the result JSON despite producing a green candidate. The incumbent's rigid scalar gate turned an empirical success into an operational failure.

### C-r1-3: Tier C Macro-Regressions Address an Unobserved Failure Class while Ignoring Real Bottlenecks
- **Severity:** **MAJOR**
- **Evidence Tier:** E1 audit of 118 ReCodeAgent projects (`02-evidence-base.md` §3).
- **Target:** `METHODOLOGY.md` §5.3 Tier C (Tarjan SCC condensation & $H^*$ restore).
- **Mechanism:** The incumbent allocates significant theoretical machinery to handling cyclic import deadlocks ($S_5 \to S_2$). However, 100% of the 118 projects across `crust`, `oxidizer`, `alphatrans`, and `skel` have strictly acyclic dependency graphs ($E_{\text{back}} = \emptyset$). Meanwhile, the actual observed killer of real campaigns—unbounded memory growth from child process leaks and socket hangs (F5, F6)—is completely unaddressed in `METHODOLOGY.md`.

---

## 3. Evidence & Derivations

### Defect Coverage Matrix (Incumbent vs. Observed Failures):
| Defect Class | Observed Campaign Defect | Incumbent Handling | Evaluator Verdict |
|---|---|---|:---:|
| **F1** | Futile dispatch cap exhaustion | Records error in JSON; fails run | Post-hoc recorder, not preventative |
| **F2** | Wave-dispatch scheduler deadlock | Daemon hangs indefinitely | **Uncovered (Fatal)** |
| **F3** | Heapq paradox (green candidate rejected) | Burns budget until cap | **Uncovered (Fatal)** |
| **F4** | Stub-collapse metric gaming | Accepts 18-line empty stub as PASS | **Uncovered (Fatal)** |
| **F5** | Context window saturation / Dumb Zone | Repetitive hallucination loops | **Uncovered (Fatal)** |
| **F6** | Single-GPU token exhaustion (17h/project) | No token budget bounds | **Uncovered (Fatal)** |

---

## 4. Requests to the Council
1. Enter `C-r1-1` as **FATAL** and `C-r1-2`, `C-r1-3` as **MAJOR** in the master ledger.
2. Require that alternative candidates explicitly provide an active preventative mechanism for every observed failure F1–F8.
