---
status: "accepted"
date: 2026-10-22
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
---

# Compile-Time Environment Ingestion via envload

## Context and Problem Statement

The workspace resolves environment variables at runtime through dotenv consumption, which defers missing-key failures from build time to first daemon invocation and adds per-process file I/O on every launch. Each crate also risks diverging in where and how it reads configuration.

How does the workspace ingest `.env` configuration deterministically, with zero runtime dotenv overhead and identical behavior in every crate?

## Decision Drivers

* Fail-fast: missing or malformed env configuration surfaces at build time, not at daemon runtime.
* Zero runtime overhead: `env!()` reads compile-time constants; no dotenv parsing at process start.
* Uniformity: one ingestion mechanism for all 23 build scripts in the workspace.
* Simplicity: the loader stays generic — it ingests key/value pairs, it never hard-codes a key list.

## Considered Options

* Keep runtime dotenv loading per binary entry point.
* Generate per-crate typed config modules listing known keys.
* Add a dedicated `envload` crate (`.checks/envload`) that every workspace `build.rs` calls first: it walks up from `CARGO_MANIFEST_DIR`, ingests the nearest `.env`, and emits generic `cargo:rustc-env=KEY=VALUE` lines; library code reads with `env!("KEY")`.

## Decision Outcome

Chosen option: "envload crate called from every workspace build.rs, replacing runtime dotenv consumption", because it gives build-time failure on missing configuration, compile-time constant reads via `env!()`, and one uniform mechanism across all crates.

* `werðan/.checks/envload` walks up from `CARGO_MANIFEST_DIR`, ingests the nearest `.env`, emits generic `cargo:rustc-env=KEY=VALUE` per pair.
* Every workspace `build.rs` calls `::envload::load()` first.
* Library code reads configuration exclusively through `env!("KEY")` — zero runtime dotenv overhead.
* The emission is generic: envload holds no key list; consumers declare the keys they read.

### Consequences

* Good, because a missing key breaks the build, not a benchmark campaign mid-run.
* Good, because `.env` edits take effect on the next build with no runtime file probing.
* Bad, because changing `.env` requires a rebuild to affect binaries; runtime reconfiguration goes through `migration.yml` instead (see [ADR-0006](0006-de-qwen-inference-transport.md)).
* Neutral, because nested workspaces resolve to their nearest `.env`, which supports per-directory overrides.

### Confirmation

`cargo build --workspace` succeeds with the `.env` present and fails fast when a consumed key is absent. The 23 build scripts all call `envload::load()` before any other build logic — verifiable by grep across `build.rs` files. Workspace checks and tests remain green ("envload crate wired into all 23 build scripts... workspace checks and tests green").

## More Information

Evidence: commits b4bfb11, 6e20e01, 0121bde, 1831b5b. Session evidence: "envload crate wired into all 23 build scripts... workspace checks and tests green".

The operational parameters routed through this mechanism and their consumers are enumerated in [ADR-0011](0011-env-config-parameters.md); the inference transport config chain that consumes the fallback tier is defined in [ADR-0006](0006-de-qwen-inference-transport.md).
