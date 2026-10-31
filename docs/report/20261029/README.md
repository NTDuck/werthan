# 20261029 — Methodology Critique & Pareto-Optimal Alternatives (werðan)

**Mission.** Exhaustively and critically evaluate — and argue against — the current werðan
methodology (`docs/references/METHODOLOGY.md`, ground truth; derivations in
`docs/references/METHODOLOGY-DRV.md`), then derive candidates for pareto-optimal
alternative methodologies for repository-scale code2code modernization under the fixed
deployment regime: a 27B-class SLM (`swift-1.5-qwen3.8-27b`) on a single RTX 3090,
bounded effective context, local serving stack.

**Governance.** `METHODOLOGY.md` remains read-only (ADR-0005). This report *proposes*;
it does not edit ground truth. Any adopted change routes through the ADR ledger.
No code changes, no commits — docs workstream only.

## Report Structure

| File | Content | Status |
|---|---|---|
| [`01-meta-strategy.md`](01-meta-strategy.md) | Research questions (RQ1–RQ8), 9 evaluation axes ($X_1 \dots X_9$), Pareto-optimality definition, evidence tiers (E1–E4), and council deliberation protocol. | **Complete** |
| [`02-evidence-base.md`](02-evidence-base.md) | Ground truth evidence base: failure taxonomy F1–F8, empirical receipts from `20251025`, dataset/oracle metrics, and operational definitions. | **Complete** |
| [`03-incumbent-critique.md`](03-incumbent-critique.md) | Systematic adversarial critique of `METHODOLOGY.md` across RQ1–RQ6: reward gaming, statechart fragility, MCTS token overruns, SLM "Dumb Zone", and unexercised Tier C complexity. | **Complete** |
| [`04-design-space.md`](04-design-space.md) | Bounded 6-axis architectural design space ($\mathcal{T}, \mathcal{S}, \mathcal{M}, \mathcal{O}, \mathcal{R}, \mathcal{H}$) spanning linear pipelines, statecharts, and actor networks. | **Complete** |
| [`05-candidates.md`](05-candidates.md) | Formulation of five candidate architectures ($C_0 \dots C_4$), formal 9-axis scoring rubric, strict Pareto-dominance proofs, and frontier identification. | **Complete** |
| [`06-audit-ledger.md`](06-audit-ledger.md) | Append-only Council of Subagents ledger recording all formal objections, severity ratings, dispositions, and convergence proofs. | **Complete** |
| [`07-bibliography.md`](07-bibliography.md) | Primary literature bibliography covering multi-agent SE, code generation/repair, statecharts, blackboards, and SLM context compaction. | **Complete** |
| [`08-final-verdict.md`](08-final-verdict.md) | Final synthesis, ratifying **`DAEDALUS` ($C_3$)** as the champion Pareto-optimal architecture, with standing falsification tests and reconciliation blueprint. | **Complete** |

## Council Artifacts

Council deliberation records live under `council/`:
- [`council/ledger.md`](council/ledger.md): The official mirror of the master objection and disposition ledger.
- `council/r0/`: Round 0 baseline calibration positions across 7 adversarial lenses.
- `council/r0.5/`: Round 0.5 refined protocol self-attack positions.
- `council/r1/`: Round 1 attack papers against the incumbent methodology (`THESEUS` / `METHODOLOGY.md`).
- `council/r2/`: Round 2 attack papers against alternative candidates (`DAEDALUS`, `KALYPSO`, `CentralLedger`, `FlatPipeline`).
- `council/r3/`: Round 3 re-attack papers verifying repaired candidates and proving convergence under the two-fatal-free-rounds rule.

## Executive Summary & Findings

1. **Refutation of the Incumbent:** The Council unanimously demonstrated that `METHODOLOGY.md` ($C_2$, `THESEUS`) is Pareto-dominated under the 27B SLM regime due to catastrophic MCTS token exhaustion (17+ hours on single modules), lack of context compaction leading to the SLM "Dumb Zone" (>16k tokens), and reward gaming on collapsed stubs.
2. **Champion Selection:** **`DAEDALUS` ($C_3$)** achieved strict Pareto dominance by combining:
   - A **Dependency-DAG Actor Network** over a shared Blackboard.
   - A **Budgeted Portfolio Race ($K=3$)** with zero-token compiler pre-filtering, reducing token costs by ~70% vs. MCTS.
   - A **Frequent Intentional Compaction Protocol** enforcing the $\le 12\text{k}$ token "Smart Zone".
   - A **Layered Hardened Oracle** with mandatory stub-sensitivity and differential execution checks.
## Non-goals

- Re-litigating ADR-0005 or editing `METHODOLOGY.md`.
- Benchmark construction or frontier-model studies; the regime is fixed.
- Code changes: this is a documentation/analysis workstream.
