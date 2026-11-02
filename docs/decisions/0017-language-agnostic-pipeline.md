# 0017. Language-Agnostic Pipeline Mandate

Date: 2026-10-24

## Status

Accepted

## Context and Problem Statement

How does the werðan modernization pipeline stay language-agnostic end to end while shipping with a batteries-included Rust default?

Live benchmark campaigns surfaced language assumptions leaking beyond the sanctioned harness/config boundary: codegen, strategy, testgen, patch, distiller, auditor, and all five global-critic preambles hardcoded "Rust"; the evaluator's syntax gate ran `proc_macro2` Rust tokenization on every candidate; the fault sentinel encoded failures as a Rust macro (`::core::compile_error!`); the harness guessed `translated.js`/`src/lib.rs` candidate paths from toolchain names; daemon workspace post-processing (header detection, `unimplemented!()` stub counting, `pub mod` normalization, API-surface extraction, cargo-test terse parsing) and the workers' `unsafe`-token scan assumed Rust targets unconditionally; the interactive wizard recognized only "rust".

## Decision Drivers

* The methodology defines the stack as config (`stack.source_language`, `stack.target_language`, `stack.target_build_cmd`, `stack.target_test_cmd`); behavior must derive from configuration, not embedded language identity.
* The board already carries `DialectProfile` (source/target language) — the plumbing existed but nobody consumed it outside the harness.
* Honest failure beats silent guessing: an underivable candidate path or unrunnable check must report failure, not fabricate a Rust-shaped answer.

## Considered Options

1. Keep Rust hardcoding; document the pipeline as Rust-only.
2. Parameterize everything on `target_language` with `rust` as the sanctioned default, gate language-specific post-processing on the dialect, and encode inference failures with a language-neutral marker.

## Decision Outcome

Chosen option 2: full language parameterization.

* **Prompts**: `codegen_system_prompt`, `strategy_system_prompt`, `testgen_system_prompt`, `patch_system_prompt`, `distill_system_prompt`, `audit_system_prompt`, the five critic preambles, and `rule_system_prompt` are `fn(target_language) -> String` composed from builder `.target_language(String)` setters (default `rust`), fed from `bench_config.stack.target_language` → `board.inventory.dialect` → `ProblemModelSlice.target_language`.
* **Sentinels**: inference failures emit `SYNTAX_ERROR: ariadne llm {synthesis|repair} fault: <reason>` (language-neutral marker channel); the evaluator retains legacy `compile_error!` recognition for old artifacts.
* **Syntax gate**: `proc_macro2` tokenization runs only for rust (or unset) targets; other targets rely on the marker channels plus the configured toolchain as the honest syntax check.
* **Candidate path**: derived from the build command's own arguments (last arg containing `.`); cargo keeps the `src/{stem}.rs` module convention; an underivable path fails with `compiles=false` and an explanatory log — no `node`/`translated.js`/`src/lib.rs` magic.
* **Daemon post-processing**: header detection follows `source_language` (C-family extensions); stub counting, `pub mod` normalization, and the workers' `unsafe` scan run only for rust targets (zero/no-op for others, documented as awaiting language-agnostic equivalents); API-surface extraction and the testgen splitter generalize per target language (`split_test_entries_for`); `bench_validation` uses cargo terse counting only when the test command is cargo, else exit-code pass/fail.
* **Interactive**: the config wizard recognizes a known-language token list (`KNOWN_LANGUAGE_TOKENS`) with word-boundary matching instead of a rust-only substring check.

### Consequences

* Good, because any stack expressible in the YAML config flows through the full pipeline without code changes.
* Good, because failures for unsupported stacks are honest harness logs, not fabricated artifacts.
* Bad, because non-rust targets currently get zero-valued stub metrics and skip the `unsafe` security scan until language-agnostic critic equivalents exist.
* Neutral, because the `rust` default preserves byte-identical behavior for every pre-existing Rust campaign and test.

### Confirmation

Workspace gates: `cargo check/test/clippy --tests` clean across `holon-{codegen,strategy,testgen,patch,evaluator}`, `global-{critics,workers}`, `interactive`, `daemon`. Dedicated tests: JS-toolchain harness tests, generic test-entry splitting, external-toolchain exit-code pass rates, non-rust unsafe-scan skip, underivable-candidate-path honest failure, language-token wizard matching.

## More Information

* Spec routing: `docs/specs/tech-spec/statecharts-and-ariadne.md` (sentinel + parameterization contracts), `docs/specs/tech-spec/benchmark-harness.md` (dialect-gated post-processing).
* Predecessor decision for the harness command override: [ADR-0016](0016-daemon-owned-transport-deadlines.md) (transport), plus the `HarnessCommands` parameterization it enabled.
