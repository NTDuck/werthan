---
status: "accepted"
date: 2026-10-22
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
---

# Environment and Configuration Parameter Plumbing

## Context and Problem Statement

Operational parameters (timeouts, limits, endpoints) risk scattering as literals or ad-hoc `std::env::var` lookups inside library code, where they are undiscoverable, unoverridable, and unauditable. The workspace needs one rule for where a tunable parameter lives and one mechanism for how library code reads it.

Where do operational parameters live, and how do libraries read them without scattering lookups?

## Decision Drivers

* No scattered literals: a tunable value never appears as a magic number or string inside library logic.
* No ad-hoc env lookups: `std::env::var` calls in non-entry-point code are a defect.
* Compile-time reads: libraries read configuration through `env!()` constants backed by the ingestion in [ADR-0007](0007-compile-time-env-ingestion.md).
* Honest key inventory: keys defined in `.env` but not yet consumed are documented as reserved, never claimed as live.

## Considered Options

* Hard-code defaults next to each use site with env lookups sprinkled through libraries.
* A central runtime Config struct passed through every call chain, populated at process start.
* Parameters bubble up to `.env`, ingested at compile time; libraries read via `env!()`; entry points thread values downward.

## Decision Outcome

Chosen option: "operational parameters bubble up to `.env` ingested at compile time; libraries read via `env!()`; no scattered literals or runtime env lookups in library code", because it makes every tunable discoverable in one file and every read a compile-time constant.

Consumer inventory:

| Component | Keys consumed |
| :--- | :--- |
| shared/tools | `TOOL_BASH_TIMEOUT_SECS`, `TOOL_EVAL_TIMEOUT_SECS`, `TOOL_GLOB_LIMIT` |
| daemon (`bench_config.rs`) | `BENCH_CONFIGS_DIR`, `BENCH_RESULTS_DIR` |
| daemon (`bench_llm.rs`) | `BASE_URL`, `MODEL_ID`, `INFERENCE_API_KEY` |

Reserved keys — defined in `.env` / `.env.example`, currently unconsumed, reserved for evaluator and daemon LLM configuration: `BENCH_BUILD_TIMEOUT_SECS`, `BENCH_TEST_TIMEOUT_SECS`, `BENCH_INFERENCE_TIMEOUT_SECS`, `EVALUATOR_BUILD_TIMEOUT_SECS`, `EVALUATOR_TEST_TIMEOUT_SECS`, `LLM_MAX_TOKENS`, `LLM_TEMPERATURE`, `LLM_TIMEOUT_SECS`.

### Consequences

* Good, because `.env` / `.env.example` is the complete, auditable parameter inventory.
* Good, because `env!()` reads are zero-cost and fail the build when a key is missing.
* Bad, because reserved-but-unconsumed keys invite the false assumption that editing them changes behavior; this ADR is the corrective record.
* Neutral, because the compile-time ingestion means parameter changes require a rebuild (per [ADR-0007](0007-compile-time-env-ingestion.md)).

### Confirmation

`.checks` linters and review gates flag `std::env::var` calls outside entry points and magic literals standing in for tunables. `grep -r 'env!' werðan/libs` returns only keys present in `.env.example`. The reserved-key list above matches `.env.example` exactly.

## More Information

Evidence: commits e06febb (`docs(rules): add no-hardcoded-config rule`), 1831b5b, faf5118, 1049964.

The ingestion mechanism is specified in [ADR-0007](0007-compile-time-env-ingestion.md); the daemon's use of `BENCH_CONFIGS_DIR` / `BENCH_RESULTS_DIR` for benchmark artifacts ties to the harness architecture in `docs/specs/benchmark-harness.md`.
