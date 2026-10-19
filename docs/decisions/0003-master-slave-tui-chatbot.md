---
status: "superseded by [0004-code-modernization-tmux-omp-pipeline.md](0004-code-modernization-tmux-omp-pipeline.md)"
date: 2026-10-15
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
---

# Master/Slave Coding Agent TUI Chatbot Interface

## Context and Problem Statement

`werðan` provides model bridge implementations for Rig, omp RPC, Goose ACP, and Codex backends. Developers need an interactive terminal user interface to run and orchestrate these coding agent bridges.

How should the TUI binary be configured, structured, and invoked to support deterministic multi-agent orchestration while adhering to strict repository code standards?

## Decision Drivers

* Master/Slave Architecture: The orchestrator master agent on the left relays user prompts deterministically to slave bridge agents stacked vertically on the right.
* CLI Ergonomics: Invokeable via `mise run tui` (debug) and `mise run tui --release` (release).
* UI Design: Clean Ratatui interface with bold window titles and responsive dual-pane layout.
* Quality Governance: Model `werðan/bins/tui/` identically to `@werðan/.examples/mybin/`, enforcing all six build-time check linters (`flatmods`, `pathqual`, `plumbing`, `ste100`, `stepdown`, `turbofish`).

## Decision Outcome

Chosen option: Implement a Ratatui-based TUI in `werðan/bins/tui/` with a horizontal master/slave split (50/50):
- **Master (Left)**: Orchestrator agent displaying message history, input prompt, and dispatch status bar.
- **Workers (Right)**: Three vertically stacked panes displaying outputs from `bridge-omp`, `bridge-goose`, and `bridge-codex`.
- **Deterministic Relay**: Prompt submissions deterministically invoke each bridge's `call()` method without prompt-based translation indirection.
- **Task Runner**: Configured in `mise.toml` as `tui = "cargo run -p tui"` (replacing `debug` and `release`).

### Consequences

* Good, because developers can observe and interact with all bridge models in parallel in real time.
* Good, because `werðan/bins/tui/` adheres 100% to repository linter policies at build time.
* Good, because smoke/headless environments automatically complete a verification turn without hanging.

### Confirmation

Confirmed via `mise run tui`, `mise run tui --release`, and `cargo test --workspace` passing with zero linter warnings.
