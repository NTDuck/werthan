---
status: "accepted"
date: 2026-10-22
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
---

# ARIADNE Search Constant Calibration for the Local Serving Stack

## Context and Problem Statement

`docs/references/METHODOLOGY.md` — the read-only ground truth pinned by [ADR-0005](0005-methodology-source-of-truth.md) — specifies Softmax-UCB temperature τ = 0.35 and gamma prior γ = 0.15. The in-tree defaults in `werðan/libs/holon/engine/src/types.rs` (`EngineConfig::default()`) carry different values, tuned for the local 27B SLM serving stack. Code and methodology must not silently disagree: the divergence needs an explicit, bounded rationale.

Which constants govern the ARIADNE search in-tree, and how does the in-tree configuration relate to the read-only methodology?

## Decision Drivers

* Methodology primacy: METHODOLOGY.md stays untouched; the divergence is resolved on the code side of the ledger, with a recorded rationale.
* Serving-stack realism: constants are calibrated for a local 27B model, not for the larger models the methodology's values assume.
* Operator override: campaign configs can adjust search behavior without recompiling.
* Bounded daemon coupling: the daemon overrides only what its benchmark harness needs; the rest flows from in-tree defaults.

## Considered Options

* Change METHODOLOGY.md to match the code.
* Change the in-tree defaults to match METHODOLOGY.md exactly.
* Keep METHODOLOGY.md read-only; calibrate in-tree constants for the 27B stack; expose overrides through `migration.yml`.

## Decision Outcome

Chosen option: "in-tree ARIADNE constants intentionally deviate from METHODOLOGY.md's τ = 0.35 / γ = 0.15 for the local 27B serving stack; METHODOLOGY.md stays untouched; `migration.yml` exposes overrides; the daemon currently overrides `max_rollouts` only", because it preserves methodology primacy while letting the search perform on the hardware actually serving it.

In-tree constants (`werðan/libs/holon/engine/src/types.rs:37-53`, `EngineConfig::default()`):

| Constant | In-tree value |
| :--- | :--- |
| `max_rollouts` | 16 |
| `ucb_c_exp` | 1.414 ($\sqrt{2}$) |
| `gamma_prior` | 0.50 |
| `softmax_temperature` | 0.50 |
| `generative_temperature` | 0.20 |
| `RewardWeights` (compile / parity / harden) | 0.20 / 0.60 / 0.20 |

* The divergence from METHODOLOGY.md τ = 0.35 / γ = 0.15 is deliberate, scaled for the local 27B SLM serving stack.
* `migration.yml`'s `ariadne` section exposes overrides (`engine_budget_per_module`, `ucb_exploration_c`, `softmax_temperature`, `reward_weights`); `engine_budget_per_module` carries the serde alias `max_rollouts` for the 118 existing configs.
* The daemon currently overrides `max_rollouts` only (`ariadne.engine_budget_per_module`, `bench_runner.rs:329-334`); all other constants flow from `EngineConfig::default()`.

### Consequences

* Good, because search behavior is tuned to the serving stack without editing the read-only methodology.
* Good, because reward tests pin the weighted sum (0.20 + 0.60 × 0.5), locking the weight semantics.
* Bad, because two sources of constant values coexist; this ADR is the reconciliation record — the spec documents in-tree values and cites this ADR for the deviation rationale.
* Neutral, because widening daemon overrides beyond `max_rollouts` is a configschema change per [ADR-0009](0009-canonical-campaign-configschema.md), not a code edit.

### Confirmation

`cargo test -p holon-engine` asserts `EngineConfig::default()` field-by-field and the reward computation 0.20 + (0.60 × 0.5). A benchmark run with `ariadne.engine_budget_per_module` set in `migration.yml` shows the daemon overriding `max_rollouts` while other constants remain at in-tree defaults.

## More Information

Evidence: `werðan/libs/holon/engine/src/types.rs:37-53`; reward tests expecting 0.20 + (0.60 × 0.5).

The methodology pinning that makes METHODOLOGY.md read-only is [ADR-0005](0005-methodology-source-of-truth.md); the constants execute inside the architecture defined in [ADR-0010](0010-holon-modular-architecture.md).
