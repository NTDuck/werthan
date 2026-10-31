# BasineTypicality — Round 1 Attack Position (Targets: Incumbent METHODOLOGY.md)

**Target:** `docs/references/METHODOLOGY.md` comparison baselines vs. Valid Empirical SE Baselines.
**Persona:** Baseline & Experimental Integrity Auditor (Preventing strawmen, representative baselines).

---

## 1. Steelman

`THESEUS` justifies its architectural complexity by contrasting against naive single-agent LLM prompting:
1. It demonstrates that naive prompting fails on large repositories due to context limits and lack of dependency tracking.
2. It claims that existing multi-agent systems lack formal statechart regression semantics and robust mathematical search guarantees.

---

## 2. Objections

### B-r1-1: Omission of Published Peer Frameworks as Concrete Experimental Baselines
- **Severity:** **MAJOR**
- **Evidence Tier:** E2 published literature (Ibrahimzada et al., *ReCodeAgent*, ASE 2026; GVS5H, 2025).
- **Target:** `METHODOLOGY.md` §1 Introduction & Architectural Justification.
- **Mechanism:** In scientific software engineering evaluation, an architecture cannot claim superiority solely by critiquing an uninstantiated naive baseline. `METHODOLOGY.md` fails to define formal, executable baselines against:
  1. **Baseline 1 (`B_1`, ReCodeAgent):** The published 4-agent feed-forward DAG baseline from ASE 2026.
  2. **Baseline 2 (`B_2`, CentralLedger / GVS5H):** The centralized manager-worker ledger pattern with sequential dispatch.
  Without executing and measuring these exact baseline topologies under identical model weights and hardware environments, claims of statechart superiority remain scientifically ungrounded.

### B-r1-2: Asymmetric Evaluation Criteria Favoring Complex Orchestration
- **Severity:** **MAJOR**
- **Evidence Tier:** E4 derivation.
- **Target:** `METHODOLOGY.md` §1 & Benchmark Acceptance Rules.
- **Mechanism:** The incumbent credits itself for complex internal state transitions (e.g., successful orthogonal state restorations) rather than purely measuring end-to-end repository pass rates per dollar/GPU-hour. When evaluated on end-to-end task success under fixed wall-clock budgets (e.g., 2 hours per project), simpler architectures often match or exceed the incumbent because they spend zero time navigating statechart transitions or resolving synthetic cycle checks.

---

## 3. Evidence & Derivations

### Minimum Baseline Requirements for Valid Empirical Comparison:
To claim Pareto optimality under the empirical standards of ICSE/FSE:
1. Every candidate must be benchmarked on identical projects (`skel`, `crust`) using identical model weights (`swift-1.5-qwen3.8-27b`).
2. Baselines must include both the naive lower bound (ReCodeAgent feed-forward) and the established centralized industry standard (manager-worker ledger).
3. The evaluation metric must be the efficiency frontier: $\text{Parity Pass Rate} / \text{GPU Wall-Clock Hour}$.

---

## 4. Requests to the Council
1. Enter `B-r1-1` and `B-r1-2` as **MAJOR** findings in the master ledger.
2. Require that the design space (`04-design-space.md`) and candidate evaluation (`05-candidates.md`) explicitly incorporate `B_1` (ReCodeAgent) and `B_2` (CentralLedger) as mandatory baselines.
