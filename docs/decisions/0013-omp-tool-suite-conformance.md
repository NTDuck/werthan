---
status: "accepted"
date: 2026-10-22
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
---

# omp Tool Suite Conformance

## Context and Problem Statement

Agents invoke host tools through Rig's `rig::tool::Tool` trait. A per-call-site `tool_err` helper converts ad-hoc errors into `rig::tool::ToolError`, but it is a function every tool crate must remember to invoke, and error conversion behavior varies where it is forgotten. The tool suite must also cover the full omp-spec surface so agents never need a capability the suite lacks.

How do all host tools implement the Rig tool contract uniformly, and which tools constitute the suite?

## Decision Drivers

* Uniform error conversion: one blanket mechanism, not per-call-site helper invocations.
* Full omp-spec coverage: thirteen tools so agent prompts can rely on the suite as a fixed contract.
* Single-crate cohesion: one tools crate owns every tool implementation and its shared plumbing.
* Rig-native: tools are `rig::tool::Tool` impls; no adapter layer between werðan tools and Rig agents.

## Considered Options

* Keep the `tool_err` helper and convert errors at each call site.
* Define a per-tool error enum implementing `Into<rig::tool::ToolError>`.
* One blanket `DisplayExt` trait implemented for every tool error type, converting into `rig::tool::ToolError::ToolCallError`.

## Decision Outcome

Chosen option: "13 omp-spec tools implemented as `rig::tool::Tool` impls in one crate, with the `DisplayExt` blanket trait converting errors into `rig::tool::ToolError::ToolCallError`", because a blanket trait makes correct error conversion automatic and uniform, replacing the forgettable `tool_err` helper.

Tool inventory (each at `werðan/libs/shared/tools/src/<name>.rs`, exposing `XTool` / `XArgs` / `XOutput`):

| # | Tool | # | Tool |
| :--- | :--- | :--- | :--- |
| 1 | `read` | 8 | `lsp` |
| 2 | `write` | 9 | `bash` |
| 3 | `edit` | 10 | `eval` |
| 4 | `glob` | 11 | `debug` |
| 5 | `grep` | 12 | `github` |
| 6 | `ast_grep` | 13 | `ask` |
| 7 | `ast_edit` | | |

* The `DisplayExt` blanket trait provides `into_tool_error()` for every error type, yielding `rig::tool::ToolError::ToolCallError`.
* The `tool_err` helper is replaced; call sites convert errors only through the blanket trait.

### Consequences

* Good, because adding a tool means implementing `XTool` / `XArgs` / `XOutput` — error conversion comes from the blanket trait with no extra wiring.
* Good, because the thirteen-tool suite is a fixed contract agents can assume across all frontends.
* Bad, because all tool errors collapse into `ToolCallError`, flattening error discrimination at the Rig boundary.
* Neutral, because tool-level result types (`XOutput`) still carry structured success/failure payloads for agents to inspect.

### Confirmation

`cargo test -p tools` exercises each tool's `rig::tool::Tool` impl and its `into_tool_error()` conversion. A grep gate verifies no `tool_err` call sites remain and that each of the thirteen files implements the trait. Agent harnesses enumerate the registered tools and assert the thirteen-name set.

## More Information

Evidence: commit c1ae9ae (ask tool); session evidence: "Replaced tool_err with a generic DisplayExt blanket implementation".

The tools run inside the agent architecture of [ADR-0010](0010-holon-modular-architecture.md); the tool timeout and limit parameters follow [ADR-0011](0011-env-config-parameters.md).
