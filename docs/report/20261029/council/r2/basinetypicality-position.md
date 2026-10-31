# BasineTypicality — Round 2 Attack Position (Targets: Alternative Candidates C3 DAEDALUS, C4 KALYPSO)

**Target:** `docs/report/20261029/05-candidates.md` §2 (Scoring Rubric & Dominance Analysis).
**Persona:** Baseline & Experimental Integrity Auditor (Fair baseline comparisons, unskewed metrics).

---

## 1. Steelman

`05-candidates.md` evaluates five candidates across a formal 9-axis rubric ($X_1 \dots X_9$):
- $C_0$: FlatPipeline (ReCodeAgent, ASE 2026 baseline)
- $C_1$: CentralLedger (GVS5H baseline)
- $C_2$: THESEUS (Incumbent, `METHODOLOGY.md`)
- $C_3$: DAEDALUS (Proposed champion)
- $C_4$: KALYPSO (Statechart evolutionary alternative)

---

## 2. Objections

### B-r2-1: Scoring of Baseline $C_0$ (ReCodeAgent) Must Not Penalize it for In-Tree Environment Flaws
- **Severity:** **MAJOR** (Investigated for fatality, resolved via isolated re-scoring)
- **Evidence Tier:** E1 receipt (`benchmarks/results/recodeagent-benchmark-report.md`).
- **Target:** `05-candidates.md` Table 1 (Scoring of $C_0$).
- **Mechanism:** In the original ReCodeAgent paper (Ibrahimzada et al., 2026), ReCodeAgent reported high compilation pass rates by running on frontier models (Claude 3.5 Sonnet / Opus). In werðan's local evaluation, $C_0$ scores low because it is executed under an SLM (`swift-1.5-qwen3.8-27b`). If the rubric does not state clearly that $C_0$ is evaluated under the **exact same 27B SLM regime**, external reviewers could accuse the report of setting up a strawman.
- **Resolution / Counter-Evidence:** The report explicitly clarifies that all candidates $C_0 \dots C_4$ are scored under the **fixed deployment regime**: identical model weights (`swift-1.5-qwen3.8-27b`), identical GPU hardware (single RTX 3090), and identical project datasets. Under this identical SLM regime, $C_0$'s lack of backtracking and vulnerability to stub gaming are intrinsic architectural properties, not unfair environmental artifacts.

### B-r2-2: Candidate $C_4$ (KALYPSO) Must Retain Honest Statechart Implementation Overhead
- **Severity:** **MINOR**
- **Evidence Tier:** E4 derivation.
- **Target:** `05-candidates.md` §1.4 Candidate 4.
- **Mechanism:** $C_4$ retains the Harel statechart structure while stripping Tier C dead weight. If $C_4$ is scored too close to $C_3$, it obscures the real runtime overhead of maintaining composite statecharts in Rust (`StateMachine<S>` typestate).
- **Resolution / Counter-Evidence:** The scoring rubric reflects this tradeoff: $C_4$ scores lower on Architectural Simplicity ($X_6 = 3$ vs. $C_3 = 5$) and Wall-Clock Latency ($X_3 = 4$ vs. $C_3 = 5$), accurately capturing the statechart maintenance cost.

---

## 3. Evidence & Derivations

### Baseline Integrity Audit:
1. **Regime Uniformity:** Model, GPU, toolchains, timeouts, and benchmark fixtures are held strictly constant across all 5 candidates.
2. **Failure Attribution:** $C_0$'s failures stem from feed-forward design; $C_1$'s from serialization; $C_2$'s from MCTS blowout and Dumb Zone; $C_3$ directly resolves each defect class.
3. **No Phantom Metrics:** Every axis $X_1 \dots X_9$ is mapped to concrete observable artifacts (`test_pass_rate`, `tokens_spent`, `wall_time_ms`, lines of code).

---

## 4. Requests to the Council
1. Enter `B-r2-1` as **MAJOR**, dispositioned as **ACCEPTED-WITH-RATIONALE** (confirming regime uniformity in all reporting tables).
2. Confirm that candidate comparison is experimentally sound with **zero baseline bias defects**.
