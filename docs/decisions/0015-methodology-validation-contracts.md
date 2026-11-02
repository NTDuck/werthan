# ADR-0015: Methodology Validation Contracts over BDD Suites

- Status: Accepted (supersedes [ADR-0014](0014-cucumber-bdd-verification.md))
- Date: 2026-10-23

## Context

[ADR-0014](0014-cucumber-bdd-verification.md) introduced Gherkin/cucumber suites in the holon and shared-tools crates to verify behavior parity with `docs/references/METHODOLOGY.md`. Experience showed the Gherkin layer adds a heavyweight harness (feature files + step glue + non-standard test binary) while the actual assertions remain implementation-derived: steps were written by reading what the code does, not what the methodology mandates. The methodology itself evolved to a finalized THESEUS form (7-phase statechart, scatter-gather, ARIADNE clusters), and contract coverage drifted.

## Decision

Replace cucumber BDD suites with **methodology validation contracts**:

- The authoritative behavior catalog lives in [`docs/specs/validation-contracts.md`](../specs/validation-contracts.md) as `VAL-<COMPONENT>-<NN>` assertions derived exclusively from the methodology documents.
- Each crate exposes its contracts as plain Rust integration tests in `tests/validation_contract.rs` (`cargo test -p <crate>`), named `contract_<id>_<slug>`.
- Contracts are implementation-agnostic: they pin observable behavior (formulas, state transitions, event emissions, persistence), never internal symbols. On conflict, the contract wins and the implementation changes.
- Contracts outrank all implementation tests. Cucumber suites and their dependencies are deleted and MUST NOT be reintroduced.

## Consequences

- New methodology-driven behavior lands contract-first: catalog entry, then implementation.
- Bug discovery shifts left: a failing contract means implementation deviation from the methodology, and the fix belongs in src/.
- No external BDD tooling; verification reduces to the standard `cargo test` surface plus the linter-gated build.
