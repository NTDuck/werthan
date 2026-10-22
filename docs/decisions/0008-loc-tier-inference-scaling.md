---
status: "accepted"
date: 2026-10-22
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
---

# LoC-Tier Inference Scaling for Benchmark Campaign Configs

## Context and Problem Statement

The 118 ReCodeAgent benchmark campaign configs carry uniform inference settings. Small projects (hundreds of LoC) run with the same rollout budget, shard count, token ceiling, timeouts, and self-heal retries as projects exceeding ten thousand LoC. Uniform settings either starve large projects of search budget or waste serving slots on trivial ones, and the ninfer continuous-batching stack has a fixed slot cap that uniform max-shard settings violate.

How do campaign inference parameters scale predictably with legacy-codebase size while respecting serving-stack limits?

## Decision Drivers

* Size-proportional search budget: MCTS rollout count must grow with module complexity, not stay flat.
* Serving-stack bounds: shard counts respect the ninfer continuous-batching slot cap.
* Context budget: max_tokens stays within 75% of the model's 65536-token context window.
* Config auditability: tier assignment is derivable from the LoC table in `benchmarks/README.md`, not from per-project judgment calls.

## Considered Options

* Uniform settings for all 118 campaigns.
* Per-project hand tuning of each config.
* Four LoC tiers (T1–T4) with scaled parameter bands applied mechanically across all campaigns.

## Decision Outcome

Chosen option: "Retune all 118 benchmark campaign configs by legacy-codebase LoC tier", because mechanical tiering gives size-proportional budgets, respects the slot cap, and stays auditable.

Tier table (legacy codebase LoC → tier):

| Tier | LoC band | Projects | max_rollouts | shards | max_tokens | timeouts (s) | self-heal retries |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| T1 | ≤500 | 24 | 16 | 2 | 32768 | 600 | 3 |
| T4 | >10000 | 5 | 96 | 6 | 49152 | 1800 | 8 |

* T2 and T3 interpolate linearly between the T1 and T4 bounds; `benchmarks/README.md` carries the full per-tier table.
* max_tokens tops out at 49152 — 75% of the 65536-token context window — reserving headroom for prompts and tool output.
* Shard counts respect the ninfer continuous-batching slot cap (maximum 6).
* Served model id: alias `swift-1.5-qwen3.8-27b` for all tiers.

### Consequences

* Good, because T1 campaigns finish in minutes while T4 campaigns get a 6× rollout budget.
* Good, because the tier table is mechanical — a new project's settings follow from its LoC count.
* Bad, because LoC is an imperfect complexity proxy; pathologically dense small modules may still exhaust T1 budgets (raising a tier is a one-line config edit).
* Neutral, because the interpolation bands for T2/T3 are reviewed whenever the serving stack changes.

### Confirmation

`benchmarks/configs/ReCodeAgent/` contains 118 configs whose parameters match the tier table for their project's LoC band. A validation pass cross-checks each config's `ariadne.engine_budget_per_module`, `inference.max_shards`, `inference.inference_max_tokens`, `inference.inference_timeout_secs`, and `verification.max_self_heal_retries` against the band derived from `benchmarks/README.md`'s LoC-tier table.

## More Information

Evidence: commits 45f75f9, 0cca708, ff3f560; full rationale and per-tier table in `benchmarks/README.md`.

The campaign schema these configs populate is the canonical parser defined in [ADR-0009](0009-canonical-campaign-configschema.md); the transport endpoints the configs address follow [ADR-0006](0006-de-qwen-inference-transport.md).
