---
status: "accepted"
date: 2026-10-22
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
---

# Canonical Campaign Configuration Schema

## Context and Problem Statement

Each frontend parses `migration.yml` with its own ad-hoc structs, so field names, defaults, and serde behavior drift between the interactive TUI and the daemon. The 118 benchmark configs must parse identically everywhere, and the interactive configurator needs a stable schema to drive its question script against.

Where does the single definition of the campaign configuration schema live, and who parses `migration.yml`?

## Decision Drivers

* Single source of truth: one crate owns the schema; zero duplicate parse structs.
* Round-trip fidelity: `from_yaml_path` → `to_yaml_text` preserves operator comments-worthy structure and keeps the 118 existing benchmark configs parsing unchanged.
* Backward compatibility with existing files: the 118 configs use `ariadne.max_rollouts`; renaming fields must not break them.
* Frontend-agnostic configurator: the `/init` wizard writes through the same schema it reads.

## Considered Options

* Per-frontend parse structs (current drift source).
* A shared serde-only struct dump with no load/save API.
* One canonical `configschema::CampaignConfig` crate exposing `from_yaml_text` / `from_yaml_path` / `to_yaml_text`, with a serde alias preserving the legacy `ariadne.max_rollouts` key.

## Decision Outcome

Chosen option: "single canonical `configschema::CampaignConfig` crate as the only `migration.yml` parser", because it eliminates drift at the type level and keeps every existing config file loadable without migration.

* `werðan/libs/shared/configschema` exposes `CampaignConfig` with `from_yaml_text`, `from_yaml_path`, `to_yaml_text`.
* Per-frontend parse structs in `werðan/bins/interactive` and `werðan/bins/daemon` are deleted; both frontends consume the shared crate.
* Serde alias `ariadne.max_rollouts` → `engine_budget_per_module` keeps the 118 benchmark configs parsing unchanged.
* The interactive `/init` wizard drives `configurator::ConfiguratorAgent` (with `ConfigQuestion` / `ConfigAnswer`, the `AskBridge` trait, `apply_answer`, `question_script`) through the ask bridge; the wizard is frontend-agnostic and writes the same schema it reads.

### Consequences

* Good, because schema changes compile-break every consumer instead of silently diverging per frontend.
* Good, because all 118 benchmark configs keep parsing with zero edits, courtesy of the serde alias.
* Bad, because every schema addition updates one crate that both frontends depend on — a change bottleneck by design.
* Neutral, because `HitlSection` gates (`gate_on_reimagine`, `gate_on_macro_regression`, `extrinsic_conflict_escalation`) resolve via `/approve` / `/reject` in the interactive frontend; the daemon runs `full_auto` and skips them.

### Confirmation

`cargo test -p configschema` covers round-trip parsing (`from_yaml_text` ∘ `to_yaml_text` is identity on the 118 fixture configs, including the `max_rollouts` alias). Frontend builds fail if any local parse struct re-appears — grep gate for duplicate `migration.yml` struct definitions outside the crate.

## More Information

Evidence: commits 54306ed, a3ab9ac, cb5d91c, ea82257.

The 118 configs' per-tier parameter bands are defined in [ADR-0008](0008-loc-tier-inference-scaling.md); the `inference.*` keys this schema carries feed the transport chain in [ADR-0006](0006-de-qwen-inference-transport.md). The `ariadne.*` override keys interact with the constant calibration in [ADR-0012](0012-ariadne-constant-calibration.md).
