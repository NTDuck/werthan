# RefuteMeasurement — Round 1 Attack Position (Targets: Incumbent METHODOLOGY.md)

**Target:** `docs/references/METHODOLOGY.md` §4.2 (Metrics, reward calculation, and validation contracts).
**Persona:** Measurement & Oracle Auditor (Ground truth verification, metric gaming, receipt provenance).

---

## 1. Steelman

The incumbent provides an explicit, quantitative verification regime:
1. It separates compilation success from test passage via a scalar formula:
   $$R(s_t) = w_{\text{comp}}\mathbb{I}_{\text{compile}} + \mathbb{I}_{\text{compile}}[w_{\text{char}}\text{PassRate}_{\text{char}} + w_{\text{hard}}\text{Score}_{\text{harden}}]$$
2. It mandates automated validation contracts (`docs/specs/validation-contracts.md`) running during every build to ensure engine behavior adheres to methodology specifications.
3. It logs explicit runtime receipts (`wall_time_ms`, `llm_requests`, `solver_dispatches`, `test_pass_rate`).

---

## 2. Objections

### M-r1-1: Native Oracle Lacks Stub-Sensitivity, Enabling Metric Gaming
- **Severity:** **FATAL**
- **Evidence Tier:** E1 receipt (`skel-toml.json`).
- **Target:** `METHODOLOGY.md` §4.2 Acceptance Criteria.
- **Mechanism:** In `skel-toml.json`, the candidate passed 100% of characterization tests (`test_pass_rate: 1.0`), yet consisted of an 18-line collapsed stub (478 bytes vs. original 400+ lines). The benchmark test suite verified only that the module exported valid symbols and that function invocations returned without throwing fatal exceptions. Under this oracle, an empty stub that returns default values achieves a perfect score. The incumbent's measurement framework cannot distinguish between a complete implementation and an empty shell.

### M-r1-2: Conflation of Compilation Success with Test Pass Rate in Reporting
- **Severity:** **MAJOR**
- **Evidence Tier:** E1 receipts (`skel-bst.json`, `skel-colorsys.json`, `skel-mathgen.json`).
- **Target:** `METHODOLOGY.md` §4.2 & Pre-fix reporting semantics.
- **Mechanism:** In pre-fix runs, `compilation_success = false` was recorded alongside `test_pass_rate = 1.0`. The engine ran tests even when the compiler reported errors (or conflated cargo/node test harness failures with clean compilation). This violates the foundational invariant that behavioral parity is only measurable on validly compiled targets.

### M-r1-3: Non-Deterministic Softmax Gating Induces Measurement Instability
- **Severity:** **MINOR**
- **Evidence Tier:** E4 derivation.
- **Target:** `METHODOLOGY.md` §4 Softmax-UCB Temperature.
- **Mechanism:** Introducing a non-zero temperature parameter ($T > 0$) into the MCTS selection policy injects pseudo-randomness into the benchmark harness. Runs with identical seeds produce diverging candidate trajectories, destroying deterministic auditability and making comparative ablation studies statistically ungrounded without large sample sizes ($N \ge 30$).

---

## 3. Evidence & Derivations

### Raw Field Receipts from Ground Truth (`02-evidence-base.md` §1):
```json
{
  "project": "skel-toml",
  "compilation_success": false,
  "test_pass_rate": 1.0,
  "candidate_loc": 18,
  "candidate_bytes": 478,
  "verdict": "FAIL (Under honest criteria)"
}
```
Under the incumbent's formula, had `compilation_success` been true (as occurred on later Node.js versions where `node -c stub.js` succeeds), this run would have been scored $R=1.0$ (PASS). An acceptance oracle that rewards code deletion with a perfect score is defective.

---

## 4. Requests to the Council
1. Enter `M-r1-1` as **FATAL** and `M-r1-2` as **MAJOR** in the master ledger.
2. Require that every alternative methodology mandate an active **Stub-Sensitivity Gate** ($\text{Sensitivity} > 0$) and differential execution testing against legacy ground truth.
