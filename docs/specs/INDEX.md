# werðan Specifications Index

This directory is the entry point for all werðan specifications. Read this file first; it routes to the right domain spec.

Authority order: [`../references/METHODOLOGY.md`](../references/METHODOLOGY.md) is read-only ground truth and supersedes every file here on any conflict ([ADR-0005](../decisions/0005-methodology-source-of-truth.md)). These specs describe behavior as it is now.

## Spec Map

| File | Scope |
| :--- | :--- |
| [product.md](product.md) | User-facing behavior: frontends, interactive cockpit UX, slash commands, HITL gates, benchmark artifacts, success criteria. |
| [product-config.md](product-config.md) | Operator-facing configuration: `migration.yml` schema, environment variables, CLI invocation, daemon modes, results layout. |
| [tech-spec.md](tech-spec.md) | Technical entry: methodology pinning, crate hierarchy, design rationale §2.1–2.6, pointers into the domain files. |
| [tech-spec/engine-and-blackboard.md](tech-spec/engine-and-blackboard.md) | Global blackboard $\mathcal{G}$ (Definition 1), `GlobalStateVector`, `global-blackboard` module deconstruction, local 4-slice blackboard $\mathcal{B}^{(v_i)}$ (Definition 2). |
| [tech-spec/statecharts-and-ariadne.md](tech-spec/statecharts-and-ariadne.md) | Harel statechart $\mathcal{H}$ (Definition 3), `TaskActivationEnvelope` (Definition 4), S0–S6 pipeline and two-stage guards, micro-FSM M0–M5, Kahn dispatch, ARIADNE MCTS controller and constants, evaluator, reward, regression tiers. |
| [tech-spec/tools-and-configschema.md](tech-spec/tools-and-configschema.md) | 13 shared agent tools, `DisplayExt` error model, `configschema::CampaignConfig` schema, configurator agent. |
| [tech-spec/benchmark-harness.md](tech-spec/benchmark-harness.md) | Daemon `bench_*` modules, workspace reset semantics, honesty gate, `BenchResult` fields, batch resume, LoC tiers, divergence roadmap. |
| [validation-contracts.md](validation-contracts.md) | Implementation-agnostic validation contracts (`VAL-*` catalog) derived from the methodology; outrank implementation tests; replaces the former cucumber suites (superseding [ADR-0014](../decisions/0014-cucumber-bdd-verification.md)). |

Section numbering note: `tech-spec.md` and `tech-spec/engine-and-blackboard.md` both preserve rationale sections `2.1`–`2.6`; the code comment at `werðan/libs/global/blackboard/src/state_vector.rs:54` cites `tech-spec 2.6`, which resolves to the engine-and-blackboard file. Never renumber these headings.

## Reading Order

1. **Operator running a campaign**: [product-config.md](product-config.md) → [product.md](product.md).
2. **Contributing engine/harness code**: [tech-spec.md](tech-spec.md) → the matching `tech-spec/` domain file → [`../references/METHODOLOGY.md`](../references/METHODOLOGY.md).
3. **Evaluating benchmark results**: [tech-spec/benchmark-harness.md](tech-spec/benchmark-harness.md) → `benchmarks/README.md`.

## ADR Map

Decision records live in [`../decisions/`](../decisions/). The ledger is append-only; `ADR-XXXX` inline tags in specs resolve here.

| ADR | Decision |
| :--- | :--- |
| [ADR-0001](../decisions/0001-agent-interface-architecture.md) | Agent interface architecture. Superseded. |
| [ADR-0002](../decisions/0002-rig-completion-model-bridges.md) | Rig completion model bridges. Superseded. |
| [ADR-0003](../decisions/0003-master-slave-tui-chatbot.md) | Master/slave TUI chatbot topology. Superseded. |
| [ADR-0004](../decisions/0004-code-modernization-rig-rust-pipeline.md) | Code modernization Rig/Rust pipeline. Superseded by [ADR-0005](../decisions/0005-methodology-source-of-truth.md). |
| [ADR-0005](../decisions/0005-methodology-source-of-truth.md) | METHODOLOGY.md is the single source of architectural truth. Accepted, canonical. |
| [ADR-0006](../decisions/0006-de-qwen-inference-transport.md) | All LLM transport over Rig's stock OpenAI-compatible provider. Accepted. |
| [ADR-0007](../decisions/0007-compile-time-env-ingestion.md) | Compile-time env ingestion via the `envload` crate in every workspace `build.rs`. Accepted. |
| [ADR-0008](../decisions/0008-loc-tier-inference-scaling.md) | 118 benchmark configs retuned by legacy-codebase LoC tier. Accepted. |
| [ADR-0009](../decisions/0009-canonical-campaign-configschema.md) | Single canonical `configschema::CampaignConfig` parser for all frontends. Accepted. |
| [ADR-0010](../decisions/0010-holon-modular-architecture.md) | Strict crate hierarchy with per-agent holon crates and generics over `Box<dyn>`. Accepted. |
| [ADR-0011](../decisions/0011-env-config-parameters.md) | No scattered literals; operational parameters bubble up to compile-time `.env`. Accepted. |
| [ADR-0012](../decisions/0012-ariadne-constant-calibration.md) | In-tree ARIADNE search constants deliberately deviate from METHODOLOGY τ=0.35/γ=0.15 for the local 27B serving stack. Accepted. |
| [ADR-0013](../decisions/0013-omp-tool-suite-conformance.md) | 13 omp-spec tools with the `DisplayExt` blanket error conversion. Accepted. |
| [ADR-0014](../decisions/0014-cucumber-bdd-verification.md) | Gherkin cucumber suites verify behavior parity with METHODOLOGY. Superseded by [ADR-0015](../decisions/0015-methodology-validation-contracts.md). |
| [ADR-0015](../decisions/0015-methodology-validation-contracts.md) | Methodology validation contracts (`VAL-*`) replace cucumber BDD suites; contracts outrank implementation tests. Accepted. |

## Change Protocol

A behavioral change updates the spec files in the same commit as the code ([`.omp/rules/specs.md`](../../.omp/rules/specs.md)):

- User-facing behavior (UX, keystrokes, slash commands, CLI flags, `migration.yml` semantics, artifact locations, success criteria) → `product.md` and/or `product-config.md`.
- Internal behavior (crate layout, algorithms, schemas, guards, env plumbing, harness internals) → `tech-spec.md` and/or the matching `tech-spec/` domain file.
- Changes crossing the product/tech boundary update both.
- New architectural decisions append a numbered file under `docs/decisions/`; never rewrite an existing ADR body.

Related reading: benchmark procedures live in [`../benchmarks.md`](../benchmarks.md) and `benchmarks/README.md`; foundational derivations in [`../references/METHODOLOGY-DRV.md`](../references/METHODOLOGY-DRV.md).

New ADR tags (`ADR-XXXX`) appearing inline in any spec file MUST be resolvable through the table above; when adding a decision, create the numbered file under `docs/decisions/` and add its map row in the same commit.

Link depth note: files directly under `docs/specs/` link ADRs as `../decisions/...`; files under `docs/specs/tech-spec/` use `../../decisions/...`.
