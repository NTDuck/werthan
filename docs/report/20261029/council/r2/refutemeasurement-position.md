# RefuteMeasurement — Round 2 Attack Position (Targets: Alternative Candidates C3 DAEDALUS, C4 KALYPSO)

**Target:** `docs/report/20261029/05-candidates.md` §1.3 Component 4 (Hardened Layered Oracle).
**Persona:** Measurement & Oracle Auditor (Ground truth verification, differential testing, metric stability).

---

## 1. Steelman

`DAEDALUS` replaces the incumbent's scalar reward gate with a conjoined four-part boolean acceptance predicate:
$$\text{Accept}_{\text{hardened}} \iff \mathbb{I}_{\text{compile}} \;\land\; \text{PassRate}_{\text{char}} = 1.0 \;\land\; \text{StubSensitive}(\mathcal{T}) \;\land\; \text{Parity}_{\text{diff}} \ge 0.95$$
1. Requires 100% clean compilation without warnings-as-errors.
2. Requires 100% test passage on characterization suites.
3. Enforces active stub-sensitivity (tests must fail if functions unconditionally throw sentinels).
4. Verifies differential execution output against the legacy implementation on identical inputs ($\ge 0.95$ parity).

---

## 2. Objections

### M-r2-1: Differential Parity Threshold $\ge 0.95$ Fails on Non-Deterministic Outputs
- **Severity:** **MAJOR** (Investigated for fatality, resolved via semantic normalization)
- **Evidence Tier:** E1 empirical benchmark audit (ReCodeAgent `crust` and `skel` test suites).
- **Target:** `05-candidates.md` §1.3 Component 4 ($\text{Parity}_{\text{diff}} \ge 0.95$).
- **Mechanism:** In real repositories, code outputs frequently contain non-deterministic elements:
  1. Hash map key iteration orders (e.g., Python's randomized hash seed vs. JavaScript object key ordering vs. Rust's `RandomState`).
  2. Floating point precision differences across language runtimes (e.g., `mathgen`).
  3. Dynamic timestamps, memory pointers, or file paths.
  If differential testing compares raw stdout/stderr strings byte-for-byte, valid translations of correct modules will score below 0.95 parity, triggering false-negative rejections.
- **Resolution / Counter-Evidence:** The differential oracle must perform **semantic output normalization**:
  - JSON and dictionary outputs are parsed and sorted by keys prior to comparison.
  - Floating point numbers are compared within an epsilon band ($|\hat{y} - y| < 10^{-5}$).
  - Dynamic tokens (UUIDs, timestamps, hex pointers) are regex-masked to `<MASK>`.
  Normalized comparison ensures that 0.95 parity reflects true algorithmic equivalence without penalizing language-runtime idiosyncrasies.

### M-r2-2: Stub-Sensitivity Mutation May Invalidate Multi-Module Linkage
- **Severity:** **MINOR**
- **Evidence Tier:** E4 derivation.
- **Target:** `05-candidates.md` §1.3 Component 4 (Stub-Sensitivity Gate).
- **Mechanism:** If stub-sensitivity injects exception sentinels into an upstream module while downstream modules are compiling, the compiler might reject the downstream module because of unresolved type signatures.
- **Resolution / Counter-Evidence:** Stub-sensitivity mutation testing is strictly sandboxed and executed **locally in an ephemeral container**. The mutated sentinel code is never written to the shared Blackboard or committed to disk; it is executed in-memory against the test runner and immediately discarded.

---

## 3. Evidence & Derivations

### Differential Parity Evaluation on Normalized Output:
Let legacy execution trace be $T_{\text{leg}} = \text{Run}(P_{\text{src}}, I)$ and candidate trace be $T_{\text{cand}} = \text{Run}(P_{\text{tgt}}, I)$.
$$\text{Parity}_{\text{diff}} = \frac{|\text{Normalize}(T_{\text{leg}}) \cap \text{Normalize}(T_{\text{cand}})|}{|\text{Normalize}(T_{\text{leg}})|}$$
By applying AST-aware normalization:
1. White-space and line endings normalized (`\r\n` $\to$ `\n`).
2. Dict keys sorted canonically.
3. IEEE-754 precision rounded to 6 decimal places.
The resulting score represents pure semantic input/output equivalence.

---

## 4. Requests to the Council
1. Enter `M-r2-1` as **MAJOR**, dispositioned as **FIXED** (mandating semantic output normalization in `DAEDALUS`).
2. Confirm that `DAEDALUS`'s measurement framework is mathematically robust with **zero fatal oracle defects**.
