---
status: "superseded by [0004-code-modernization-rig-rust-pipeline.md](0004-code-modernization-rig-rust-pipeline.md)"
date: 2026-10-15
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
---

# Direct Rig CompletionModel Bridges and bon Builder Integration

## Context and Problem Statement

`werðan` previously defined a custom `Agent` trait and wrapper types in `werðan/core/agent/` with adapter implementations under `werðan/infras/agent/`. This added an unnecessary indirection layer when interacting with Rig workflows.

How can we make Rig, omp RPC, Goose ACP, and Codex backends available directly as `CompletionModel` in Rig while enforcing clean code rules, zero unnecessary intermediate crates, and ergonomic construction via `bon` builders?

## Decision Drivers

* High Cohesion, Loose Coupling: Remove intermediate abstract agent traits; wire each bridge directly into Rig's native driver architecture (`Wire` + `Transport` + `Model<W, T>` + `DynModel<Completion>`).
* Builder Ergonomics: Construct completion models using the modern `bon` builder crate across workspace bridges.
* Quality Governance: Model each bridge identically to `@werðan/.examples/mylib/` by executing all six check linters (`flatmods`, `pathqual`, `plumbing`, `ste100`, `stepdown`, `turbofish`) in `build.rs`.

## Considered Options

* Keep `core/agent` with adapters in `infras/agent`.
* Introduce a shared intermediate bridge trait crate (`bridge-common`).
* Remove `core/agent`, rename `infras` to `bridges`, wire directly to Rig's driver model primitives without intermediary crates, and expose `Model<W, T>` and `DynModel<Completion>` conversions.

## Decision Outcome

Chosen option: "Remove `core/agent`, rename `infras` to `bridges`, and wire each bridge directly to Rig's native Driver architecture (`Wire` + `Transport` + `DynModel<Completion>`) constructible with `bon` builder", because it aligns directly with Rig's completion model architecture and eliminates intermediate abstraction overhead.

### Bridge Specification

- **`bridge-omp` (`werðan/bridges/omp`)**: Omp RPC completion model with native `OmpWire` (`Wire`) and `OmpTransport` (`Transport`), returning `Model<OmpWire, OmpTransport>` and `From<&CompletionModel> for DynModel<Completion>`.
- **`bridge-goose` (`werðan/bridges/goose`)**: Goose ACP completion model with native `GooseWire` (`Wire`) and `GooseTransport` (`Transport`), returning `Model<GooseWire, GooseTransport>` and `From<&CompletionModel> for DynModel<Completion>`.
- **`bridge-codex` (`werðan/bridges/codex`)**: Codex protocol completion model with native `CodexWire` (`Wire`) and `CodexTransport` (`Transport`), returning `Model<CodexWire, CodexTransport>` and `From<&CompletionModel> for DynModel<Completion>`.

All bridges execute the six workspace checks in `build.rs` and derive `::bon::Builder`.

### Consequences

* Good, because consumers directly instantiate and call completion models using `bon` builder methods and pass them directly into Rig's `AgentBuilder::new()`.
* Good, because dead weight in `core/agent` and intermediate shim crates is eliminated.
* Good, because 100% compliance with `flatmods`, `pathqual`, `plumbing`, `ste100`, `stepdown`, and `turbofish` is verified at build time.

### Confirmation
Confirmed via automated unit tests in `bridge-omp`, `bridge-goose`, and `bridge-codex`, passing in `cargo test --workspace`.
