---
status: "accepted"
date: 2026-10-15
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
---

# Unified Minimal Agent Interface and Multi-Agent Adapters

## Context and Problem Statement

`werðan` requires a unified, lightweight agent interface capable of driving multiple coding agent backends (Rig, omp RPC, Goose ACP, and Codex CLI app-server) through a single trait. Existing frameworks either couple tightly to a specific transport (stdio RPC, ACP, HTTP) or invent heavy custom message and event abstractions.

How can we provide a minimal, idiomatic Rust agent interface that supports asynchronous input, streaming outputs, OpenAI-compatible model switching, and local tool registration without duplicating structures?

## Decision Drivers

* High Cohesion, Loose Coupling: The core agent contract must remain decoupled from specific agent runtime transports.
* Minimal Surface Area: Reuse existing ecosystem primitives (specifically Rig messaging and streaming types) instead of inventing redundant structs.
* Streaming & Interactivity: Support streaming token deltas, reasoning chunks, and structured events.
* Extensibility: Allow easy registration of host-side tools and runtime switching of OpenAI-compatible models.

## Considered Options

* Bespoke heavy trait hierarchy with custom wire types for every protocol.
* Thin unified `Agent` trait in `werðan/core/agent/` reusing Rig messaging and streaming primitives, with adapter crates in `werðan/infras/agent/**`.

## Decision Outcome

Chosen option: "Thin unified `Agent` trait in `werðan/core/agent/` reusing Rig messaging and streaming primitives, with adapter crates in `werðan/infras/agent/**`", because it achieves maximum cohesion and minimal surface area while interoperating seamlessly across Rig, omp, Goose, and Codex.

### Core Interface Specification

- **Input (`AgentInput`)**: Wraps `rig::completion::Message` with conversions from strings and custom messages.
- **Output (`AgentOutputStream`)**: Streams `Result<AgentOutput, AgentError>` yielding text fragments, reasoning deltas, tool calls/results, native `rig::streaming::StreamEvent`s, and completed `AssistantMessage` turns.
- **Model (`OpenAiModel`)**: Unified representation for OpenAI-compatible models (model id, custom base URL, API key, and temperature), directly convertible into Rig's `OpenAIConfig`.
- **Tool (`LocalTool`)**: Generic local host tool with name, description, JSON Schema parameters, and asynchronous execution handler, convertible to Rig `DynamicTool`, omp `HostTool`, and Goose `ToolListItem`.
- **Adapters**:
  - `werðan/infras/agent/rig`: Adapts `rig-core` SDK with streaming support.
  - `werðan/infras/agent/omp`: Adapts `omp-rpc` client over stdio/IPC.
  - `werðan/infras/agent/goose`: Adapts Goose ACP protocol wire types.
  - `werðan/infras/agent/codex`: Adapts OpenAI Codex app-server protocol types.

### Consequences

* Good, because all agent implementors satisfy a single object-safe `Agent` trait.
* Good, because Rig's rich message and event models are reused directly without translation overhead.
* Good, because consumers can easily switch models and register local tools across all supported agent runtimes.
* Neutral, because each adapter crate maintains its own minimal transport mapping.

### Confirmation

Confirmed via automated unit tests in `werðan/core/agent` and across all four infrastructure crates (`agent-rig`, `agent-omp`, `agent-goose`, `agent-codex`), with full green test passes in `cargo test --workspace`.
