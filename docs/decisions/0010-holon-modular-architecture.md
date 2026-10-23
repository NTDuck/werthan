---
status: "accepted"
date: 2026-10-22
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
---

# Holon Modular Architecture: Strict Crate Hierarchy with Typestate FSMs

## Context and Problem Statement

Flat crate layouts and dynamic state-machine wrappers blur module boundaries: a dynamic `ClusterFsm` layer wraps state enums in runtime dispatch that the type system cannot verify, and agent crates mix construction, prompting, and transport concerns. The workspace needs crate boundaries that a linter can enforce and a compiler can check.

How do the crate hierarchy, agent construction, and state-machine dispatch enforce strict modularity?

## Decision Drivers

* Strict hierarchy: crate paths mirror architectural tiers — no flat orphans.
* Uniform agent construction: every holon agent crate exposes the same builder pattern yielding a Rig agent.
* Compile-time state verification: state machines are typestate, not runtime index dispatch.
* Static dispatch in hot paths: generics over trait bounds, not `Box<dyn>` vtables, in engine and solver cores.

## Considered Options

* Retain flat crate layout with the dynamic `ClusterFsm` wrapper and `Box<dyn Behavior>` dispatch.
* Consolidate all agents into one mega-crate with internal modules.
* Strict hierarchical crates (`global/`, `holon/`, `shared/`), builder-constructed agents, engine generics over completion model / harness / distiller, and pure-typestate FSMs.

## Decision Outcome

Chosen option: "strict crate hierarchy with builder-constructed agents, generic engine cores, and pure-typestate FSMs", because it makes every architectural boundary compiler-checkable and linter-enforceable.

Crate hierarchy:

```text
werðan/
├── bins/{interactive,daemon}
├── libs/global/{blackboard,engine,fsm,agents/{arbiter,workers,global-critics}}
├── libs/shared/{configschema,tools,agents/configurator}
├── libs/holon/{blackboard,engine,fsm,agents/holon-{strategy,codegen,testgen,patch,evaluator}}
├── .checks/{envload,flatmods,pathqual,plumbing,ste100,stepdown,turbofish}
└── .experiments/{laya-guardrails,laya-as-a-judge,laya-decision-model}
```

* Every holon agent crate exposes `Holon{name}Agent::builder() → rig::agent::Agent`.
* Engine cores are generic, not dynamically dispatched: `ClusterEngine<M, H, D>` and `AriadneSolver<M, H, D>` take the completion model, harness, and distiller as type parameters.
* Per-agent max token budgets: strategy 4096, codegen 16384, testgen 8192, patch 16384, evaluator roles 512.
* The dynamic `ClusterFsm` layer is removed in favor of pure typestate (state-as-struct, Hoverbear pattern) in `global/fsm` and `holon/fsm`.

### Consequences

* Good, because the crate tree reads as the architecture diagram; `.checks` linters (`flatmods`, `pathqual`) enforce it at build time.
* Good, because engine dispatch is zero-cost static dispatch; `Box<dyn>` survives only at crate boundaries that must store heterogeneous phase machines.
* Good, because invalid FSM transitions fail to compile.
* Bad, because generic engine signatures grow monomorphized instantiations per (model, harness, distiller) tuple.
* Neutral, because `.experiments/laya-*` crates stay outside the `libs/` tiers; promoting one into the hierarchy is a future decision.

### Confirmation

`.checks/flatmods` and `.checks/pathqual` verify the hierarchy at build time — no orphan crates, no cross-tier shortcuts. `cargo test --workspace` compiles the generic engine instantiations and the typestate transitions; transition attempts that skip states fail compilation. The FSM crate layout is asserted by the validation contracts in [ADR-0015](0015-methodology-validation-contracts.md).

## More Information

Evidence: commits 4c8d267, 760db85, c8dec2d. Session evidence: "HOLON typestate refactor is complete: fsm dynamic wrapper removed, engine/controller migrated to typed solve" and "ARIAD: Create crate holon/fsm... holon/mcts (removed as noop)".

The search constants flowing through `AriadneSolver` are calibrated in [ADR-0012](0012-ariadne-constant-calibration.md); the tool suite the agents consume conforms to [ADR-0013](0013-omp-tool-suite-conformance.md); FSM behavior parity is verified per [ADR-0015](0015-methodology-validation-contracts.md).
