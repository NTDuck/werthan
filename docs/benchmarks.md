# Benchmarks

Benchmark procedures and dataset layout live in [`benchmarks/README.md`](../benchmarks/README.md).
The benchmark harness architecture (daemon `bench_*` modules, honesty gate, result artifacts,
divergence roadmap) is specified in [`docs/specs/tech-spec/benchmark-harness.md`](specs/tech-spec/benchmark-harness.md).

## Campaign configs

The 118 ReCodeAgent campaign configs under `benchmarks/assets/configs/ReCodeAgent/`
are retuned by legacy-codebase LoC tier per [ADR-0008](decisions/0008-loc-tier-inference-scaling.md).
The canonical config schema is `configschema::CampaignConfig` per [ADR-0009](decisions/0009-canonical-campaign-configschema.md).

## Historical note

The 2026-10-17 benchmark report (preserved in git history as this file's prior content) was produced
under the superseded 10-state FSM model (S0-S9) that [ADR-0005](decisions/0005-methodology-source-of-truth.md)
replaced with the canonical 7-phase methodology. Its pipeline description (10-state FSM, 6 mandatory
gates, Pi-Warden tool guardrails, Jev System-1) does not describe the current system. Current run
procedures, config semantics, and result artifact behavior are defined in the two targets above, not here.
