---
description: Maintain and synchronize product specs and tech specs with code changes. Never allow specifications and implementation to drift.
alwaysApply: true
---

# Specifications Synchronization (Specs)

Maintain and continuously synchronize product specifications (`docs/specs/product.md`, `docs/specs/product-config.md`) and technical specifications (`docs/specs/tech-spec.md` plus the `docs/specs/tech-spec/` domain files) with code changes on the fly. Code and specs must remain in lockstep at all times. `docs/specs/INDEX.md` is the entry pointer and defines the file routing.

## 1. Specification Files & Authority

- **Product Specification** (`docs/specs/product.md`, `docs/specs/product-config.md`): Authoritative document for user-facing behavior, capabilities, TUI layout/keystrokes, commands, workflows, tools, and functional requirements (`product.md`), plus operator-facing configuration, environment variables, CLI invocation, and artifact layouts (`product-config.md`).
- **Technical Specification** (`docs/specs/tech-spec.md` entry + `docs/specs/tech-spec/` domain files): Authoritative document for architectural decisions, workspace hierarchy, crate layouts, module contracts, interfaces, data schemas, algorithms, and system invariants. `docs/specs/INDEX.md` routes between them and maps `ADR-XXXX` tags.
- **Architectural Decision Records** (`docs/decisions/`): Architectural decisions follow MADR (Markdown Architectural Decision Records) referencing `references/madr/template/adr-template.md`. The ledger is append-only.

## 2. Inviolable Synchronization Invariants

- **Zero Drift On The Go**: Any task, turn, or commit modifying code behavior, user interfaces, commands, workspace crates, dependencies, or tools MUST update the corresponding product and technical specifications in the same commit.
- **Spec First or Lockstep**: Never postpone specification updates to follow-up work or future PRs. If code changes, specifications must change alongside it immediately.
- **Truth Discipline**: Code that contradicts specification or specification that contradicts code is a defect.
- **Completeness**: When adding or refactoring modules, libraries, or tools (e.g. `werðan/libs/shared/tools/`), both specifications must document their existence, interface contracts, error models, and integration points.
- **Pre-Yield Verification**: Before yielding completion of any task, check `git status` or diff against `docs/specs/` to verify that the specs routed through `docs/specs/INDEX.md` accurately reflect the actual implementation state.

## 3. FSM State Representation Convention

- **State machines use the Hoverbear pattern** (state-as-struct/typestate, reference: https://hoverbear.org/blog/rust-state-machine-pattern/): each state is a struct; transitions consume the machine and yield the next phase via `From` implementations. Existing instances: `global/fsm::MacroStateMachine<S>`, `holon/fsm::StateMachine<S>` (pure typestate; the former dynamic `ClusterFsm` wrapper was removed), `interactive/src/mock.rs::StateMachineRunner<S>`.
- **Unit-variant enums are reflection tags, never state machines**: enums such as `MicroStateTag`, `MacroState`, `ModuleMicroState`, `ModernizationState` exist for serde persistence, UI display, event payloads, and runtime inspection. Do not convert them to typestate and do not drive control flow off a state index counter.
- **New FSMs must follow the pattern**: a new state machine declares one struct per state (with shared context carried separately) rather than a central `enum State` + index/counter dispatch. Dynamic dispatch (`Box<dyn Behavior>`) is acceptable only at a crate boundary that must store heterogeneous phase machines (e.g. `MacroStatechart`).
