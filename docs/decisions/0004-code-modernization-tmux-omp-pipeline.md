---
status: "accepted"
date: 2026-10-17
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
supersedes: ["0002-rig-completion-model-bridges.md", "0003-master-slave-tui-chatbot.md"]
---

# Code Modernization Pipeline via Pure Rust Rig, Ratatui, and OMP Engineering Achievements Architecture

## Context and Problem Statement

`werðan` was previously conceptualized as a runtime traffic and council chatbot leveraging direct Rig completion model bridges (ADR 0002) and a custom Ratatui master/slave TUI (ADR 0003), or alternatively an external tmux multiplexer running Oh-My-Pi RPC sessions with a TypeScript socket extension (Option 1). However, enterprise code modernization requires an offline, batch-oriented, static codebase-to-codebase compiler and verification pipeline with zero runtime RPC overhead, zero brittle PTY/tmux coupling, and deterministic semantic parity.

Modernizing legacy mission-critical systems (e.g. COBOL, PL/SQL, legacy Java/C) involves large structured codebases, deep dependency trees, and non-negotiable semantic parity. The workflow must transition deterministically through distinct workspace stages:
`legacy/<sys>/` (original legacy code and DDL) $\rightarrow$ `analysis/<sys>/` (extracted domain clusters, business rules, architecture models) $\rightarrow$ `modernized/<sys>/` (target modern implementation, equivalence tests, verification suites), adhering to the Anthropic code-modernization methodology.

External process wrappers, tmux pane manipulation, Node/TypeScript IPC extensions (`mas-bridge`), and JSONL RPC streams add unnecessary operational overhead, break terminal portability, and introduce fragility. Instead, we require a pure Rust in-process architecture that captures the best engineering achievements and ideologies pioneered by Oh-My-Pi (OMP)—rebuilt natively in Rust without any RPC daemon or external tool dependencies.

How should the codebase-to-codebase modernization pipeline be orchestrated, displayed, and governed in pure Rust to achieve deterministic migrations while guaranteeing in-process safety and sub-second decision making?

## Decision Drivers

* **Static Codebase-to-Codebase Translation**: Zero reliance on runtime traffic or council chatbots; operate as an offline, staged migration compiler pipeline (`legacy/<sys>/` $\rightarrow$ `analysis/<sys>/` $\rightarrow$ `modernized/<sys>/`).
* **Pure Rust In-Process Architecture (Option 2)**: Eliminate external `tmux`, `omp` RPC daemons, and Node/TypeScript extensions; package the entire pipeline, TUI, and agent infrastructure into cohesive Rust workspace crates.
* **OMP Engineering Achievements Rebuilt in Rust**:
  - **Hashline Editing**: 4-character SHA-256 base62 line anchor content addressing with neighboring context triplet hashing and collision rejection (`HASHLINE_AMBIGUOUS`), eliminating hallucinated line offset drift.
  - **SnapCompact**: HMAC-SHA256 authenticated frame manifests, deterministic immutable cache index stubs (`[snapcompact:sha256=...:lines=...]`), and frame byte verification, preserving LLM KV-cache prefixes across compaction boundaries.
  - **Time-Traveling Stream Rules**: Dormant token stream interceptors scanning chunked streaming output; on forbidden placeholder emission (e.g. `unimplemented!()`, `todo!`), performing mid-token stream abort, prefix rewind to clean newline, and `<system-reminder>` injection to steer the model back into compliance.
  - **Persistent LSP & Diagnostics Loopback**: In-process diagnostic formatting appending compiler errors and type mismatches directly into tool responses before diff acceptance.
  - **In-Process Search**: Sub-millisecond recursive file traversal and line matching without fork/exec overhead or shell dependencies.
  - **Filesystem Sandbox**: In-process path enforcement strictly protecting `legacy/` as read-only and gating writes to `modernized/`, `analysis/`, and `/tmp`.
  - **Magic Directive Extraction**: Extracting `ultrathink`, `orchestrate`, and `workflowz` directives from prompt text.
* **60/40 Ratatui Cockpit with CSI-2026 Synchronized Output**: Native in-process Ratatui TUI dividing the terminal into a 60% Left Orchestrator pane and 5 stacked 40% Right Specialist panes, using CSI-2026 (`\x1b[?2026h` / `\x1b[?2026l`) synchronization to eliminate flicker and tearing.
* **Jev 3-Role Decision Model**: Fast (~250ms) classification middleware acting as route optimizer, Pi-warden safety guardrail evaluator, and gate referee (Given/When/Then rule cards, test coverage theater, done claims).
* **Deterministic 10-State Machine (S0 to S9) with 6 Human Decision Gates**: Manage the end-to-end modernization lifecycle through a formal Finite State Machine (S0 INTENT to S9 COMPLETED) with 6 mandatory human sign-off gates.
* **Quality Governance**: Enforce all 6 workspace check linters (`flatmods`, `pathqual`, `plumbing`, `ste100`, `stepdown`, `turbofish`) at build time.

## Considered Options

* **Option 1: Tmux + Oh-My-Pi with TypeScript Socket Bridge (`extensions/mas-bridge`) & OMP RPC** (Rejected: requires external tmux, bun/node, RPC daemon socket, brittle PTY coordination).
* **Option 2: Pure Rust Rig + Ratatui with OMP Engineering Achievements (In-Process MAS Engine)** (Accepted: zero external process dependencies, native 60/40 Ratatui TUI with CSI-2026, 7 native OMP engineering systems consolidated into 8 deep-domain crates in pure Rust).

## Decision Outcome

Chosen option: **Option 2: Pure Rust Rig + Ratatui with OMP Engineering Achievements (In-Process MAS Engine)**, superseding ADR 0002, ADR 0003, and Option 1, because it eliminates all external tmux, Node.js, and RPC daemon dependencies in favor of a 100% Rust in-process architecture implementing OMP's proven engineering feats: Hashline editing, SnapCompact, Time-Traveling Stream Rules, Persistent LSP, In-process Search, Filesystem Sandboxing, Jev 3-Role Decision Middleware, and 60/40 Ratatui CSI-2026 TUI.

#### 1. Directory & Staging Layout
The modernization compiler operates strictly over three staged workspaces:
- `legacy/<sys>/`: Immutable legacy sources, schemas, copybooks, DDL, and scripts (mounted read-only).
- `analysis/<sys>/`: Intermediate structured artifacts: domain clusters (5–12 domains), tech debt ledger, call graphs, Given/When/Then business rules specification cards (P0/P1/P2), and architectural critiques.
- `modernized/<sys>/`: Target stack implementation, characterization fixtures, cross-stack differential equivalence tests, and security regression audits.

#### 2. Workspace Organization & Target Project Structure
The repository is structured into infrastructure schemas and cohesive, deep-domain Rust crates:

```text
.
├── Cargo.toml                        # Workspace root
├── modernize.schema.json             # JSON Schema for ModernizeConfig
├── pi-warden.template.md             # Guardrail template rendered per run
├── middleware/
│   ├── laya/                         # Laya fast non-generative encoder daemon & 3 deterministic hooks
│   └── tools/REA/                    # REA MCP tools (Headless Ghidra, DAP, Obligation Ledger)
├── blackboard/                       # ARIADNE/LLMAS inspired shared knowledge repository
├── infras/
│   ├── topology/                     # Asymmetric Agent Hierarchy & spawning permissions
│   └── state-machine/                # Exogenous State Machine (S0 to S9) & 6 Human Decision Gates
└── werðan/
    ├── protocol/                     # Wire formats, ModernizeConfig, magic directives
    ├── harness/                      # Tooling: hashline, lsp-tools, snapcompact, stream-rules, sandbox
    ├── verify/                       # Clean-room build/test runner, output differ, receipt signer
    ├── orchestrator/                 # High-level coordinator, benchmark runner, agent factories
    ├── bins/
    │   ├── interactive/              # 60/40 Ratatui TUI cockpit with real-time continuous streaming
    │   └── daemon/                   # CLI daemon entry point (interactive or batch config runner)
```

##### Crate Responsibilities
- **`infras/topology`**: Encapsulates the Asymmetric Agent Hierarchy. Root Orchestrator (Depth 0) is the session owner with spawning permissions; Depth 1 has global read-only leaves and the Module-Lead (sub-orchestrator) with Depth 2 spawning authority; Depth 2 has module execution leaves. Defines role authorities, spawning rules, and tool scopes.
- **`infras/state-machine`**: Owns the deterministic lifecycle (S0: INTENT through S9: COMPLETED) and the 6 Human Decision Gates, executing outside prompts in host code.
- **`blackboard`**: Central shared knowledge repository inspired by ARIADNE/LLMAS with structured partitions for Obligation Ledger, Business Rules, Domain Clusters, Module Worktrees, Taint Ledger, and Audit Trails.
- **`middleware/laya`**: Lightweight non-generative encoder daemon (~30ms latency, $0 token cost) executing 3 deterministic hooks: Stateful Pre-Tool Guardrail (`pre_tool_use` with taint tracking), Historian Airlock (S4 & S6b schema and injection verification), and Fast Pre-Critic Rubric Gate (S6c 12-question quality rubric).
- **`middleware/tools/REA`**: Reverse Engineer Anything tools wrapping Headless Ghidra, scc metrics, and DAP memory snapshots. Strictly role-isolated to binary-historian and business-rules-extractor, seeding and verifying the Obligation Ledger.
- **`werðan/protocol`**: Defines wire-format IPC structs, ModernizeConfig schema, and magic directives (`ultrathink`, `orchestrate`, `workflowz`).
- **`werðan/harness`**: Core tools: hashline anchor editing, snapcompact KV-cache preservation, stream rules, sandbox filesystem enforcement.
- **`werðan/verify`**: Deterministic clean-room build/test runner, JUnit XML parsing, and benchmark runners.
- **`werðan/orchestrator`**: Coordinates FSM execution, benchmark loops, and rig AgentBuilder factories.
- **`werðan/bins/interactive`**: 60/40 Ratatui cockpit: Left pane (60%) conversational agent with slash commands (`/init`, `/start`, `/approve`); Right pane (40%) vertically stacked windows streaming independent continuous specialist logs.
- **`werðan/bins/daemon`**: CLI entry point supporting both interactive TUI launch and batch `--config <path>` runs for automated benchmarks.

#### 3. In-Process Ratatui 60/40 Cockpit & OMP Engineering Feats in Pure Rust
Option 2 executes the complete modernization engine in-process in pure Rust:

1. **Native Ratatui 60/40 Cockpit with CSI-2026 Synchronization**:
   - Terminal area is divided into a **60% Left Orchestrator pane** and **5 vertically stacked 40% Right Specialist panes** (`compute_layout`).
   - Terminal updates are wrapped with CSI-2026 synchronization tokens (`\x1b[?2026h` start, `\x1b[?2026l` end), guaranteeing tear-free atomic frame updates across all panes.
   - Operator keyboard input remains locked to the left orchestrator pane (`is_input_locked_to_left`), while right specialist panes stream real-time worker progress.
   - Magic keywords (`ultrathink`, `orchestrate`, `workflowz`) are highlighted directly in prompt buffers (`highlight_magic_keywords`).

2. **Hashline Content-Addressed Editing**:
   - Replaces fragile line numbers with 4-character base62 SHA-256 anchors computed over $(line_{i-1}, line_i, line_{i+1})$ triplets.
   - Any line shift or ambiguous anchor duplicate triggers instant `HASHLINE_AMBIGUOUS` rejection, preventing silent edit corruption.

3. **SnapCompact Frame Archiving & KV Cache Preservation**:
   - Frame byte streams are hashed and verified with SHA-256 (`verify_frame_bytes`), then signed via HMAC-SHA256 (`sign_manifest`).
   - Compacted text is stubbed with deterministic pointers (`[snapcompact:sha256=...:lines=...]`), preserving KV-cache prefixes across LLM context compaction cycles.

4. **Time-Traveling Stream Rules**:
   - Active stream inspection halts generation mid-stream when a forbidden token (e.g. `unimplemented!()` or `todo!()`) appears.
   - The output stream rewinds to the clean prefix preceding the violation, and injects a `<system-reminder>` tag to guide the model without exhausting full turn limits.

5. **Persistent LSP Compiler Loopback & Zero-Fork Search**:
   - LSP diagnostics (`LspDiagnostic`) are appended directly to edit tool results, creating an immediate compiler feedback loop.
   - In-process file search (`search_files_in_process`) walks directory trees directly, eliminating subshell fork storms.

#### 4. The Agent Hierarchy (Asymmetric Tree)
The hierarchy is intentionally unbalanced (Depth 1 for analysis/audit, Depth 2 only for module synthesis):

```text
[Depth 0] Root Orchestrator (Session Owner — hooks/state.ts) [CAN SPAWN SUBAGENTS]
 │
 ├── [Depth 1: Global Read-Only & Audit Leaves] [CANNOT SPAWN]
 │    ├── legacy-analyst            (Read-only source AST/LSP exploration)
 │    ├── binary-historian          (Read-only REA / Headless Ghidra exploration)
 │    ├── business-rules-extractor  (Read-only shard worker; fanned out 50–200x)
 │    ├── architecture-critic       (Read-only structural & contract reviewer)
 │    ├── security-auditor          (Read-only CWE scanner & secret masker)
 │    └── boundary-mediator         (Scoped R/W adapter fixer across 2 module boundaries)
 │
 └── [Depth 1: Module Sub-Orchestrator] [CAN SPAWN SUBAGENTS]
      └── module-lead               (1 per module in an isolated omp CoW worktree)
           │
           └── [Depth 2: Module Execution Leaves] [CANNOT SPAWN]
                ├── test-engineer     (R/W strictly to modernized/<mod>/tests/**)
                ├── target-coder      (R/W strictly to modernized/<mod>/src/** via /prewalk)
                ├── legacy-analyst    (On-demand source oracle, gated by Laya Airlock)
                └── binary-historian  (On-demand REA binary oracle, gated by Laya Airlock)
```

#### 5. The Exogenous State Machine (S0 → S9) & 6 Human Decision Gates
The state machine lives in deterministic host code (`infras/state-machine`) and is owned exclusively by the `root-orchestrator`.

1. **`S0: INTENT`**: Initialize session ledger and `.dem` replay log. Gate 1: `INTENT.md` written + Human Gate 1.
2. **`S1: PREFLIGHT`**: Run `scc`/`cloc`, AST call-graph, and REA binary inventory to seed the Obligation Ledger.
3. **`S2: ASSESS`**: Mask detected credentials (`Pr0d****`), domain clustering, tech-debt ranking, CWE baseline.
4. **`S3: MAP`**: Compute topological DAG waves across modules. Gate 2: `MAP.md` verified + Human Gate 2.
5. **`S4: EXTRACT_RULES`**: `make_shards.py` splits source + REA decompiled views; Laya Airlock validates citations. Gate 3: `BUSINESS_RULES.md` + Human Gate 3.
6. **`S5: BRIEF`**: Generate stubbed target skeleton directories. Gate 4: `BRIEF.md` & target skeletons approved + Human Gate 4.
7. **`S6: TRANSFORM`**: CoW worktree per module; `module-lead` drives S6a Pin (`test-engineer`), S6b Code (`target-coder` via `/prewalk`), S6c Audit (Laya Rubric in 30ms -> Adversarial Trio: `test-engineer`, `architecture-critic`, `security-auditor`).
8. **`S7: VERIFY`**: Clean-room test runner parses XML/JSON; REA Obligation Ledger 100% linked. Gate 5: Proof Receipt + Human Gate 5.
9. **`S8: HARDEN`**: Cross-module static analysis & CVSS calculator; zero P0/P1 CWE regressions.
10. **`S9: COMPLETED`**: Final HTML topology & SHA-256 hash-chained audit receipt. Gate 6: Final Release Sign-Off.

#### 6. Integration: Laya, REA, and Blackboard
- **Laya Middleware**: Runs locally as a lightweight non-generative encoder daemon (~30ms latency, $0 token cost) answering pre-compiled `Noul` (probability 0–1), `Score` (0–4), and `Choice` questions. Powers 3 deterministic hooks: Pre-Tool Guardrail with taint tracking, Historian Airlock for business rule cards, and Fast Pre-Critic Rubric Gate.
- **REA Middleware**: Reverse Engineer Anything tools wrapping Headless Ghidra and debuggers. Strictly mounted only on `binary-historian` (and read-only `rea_query_retained_view` on `business-rules-extractor`), seeding and verifying the Obligation Ledger.
- **Blackboard**: Shared knowledge repository inspired by ARIADNE/LLMAS with structured partitions for Obligation Ledger, Business Rules, Domain Clusters, Module Worktrees, Taint Ledger, and Audit Trails.
#### 7. Rig Ecosystem Integration

The modernization engine integrates the Rig LLM ecosystem (`rig-core 0.9`) natively, with no external bridge processes:

- **Native rig Tools (`werðan/harness::tools`)**: Three `rig::tool::Tool` implementations wrap harness primitives behind the `rig` cargo feature:
  - `HashlineReplaceTool` (`path`, `start_anchor`, `end_anchor`, `replacement`) → calls `harness::hashline::apply_hashline_replace` and returns an edit status string.
  - `ReadHashlineTool` (`path`, `start_line`, `max_lines`) → calls `render_hashlined_view` to render anchor-addressed file regions.
  - `SearchTool` (`root`, `query`) → calls `harness::lsp_tools::search_files_in_process`.
- **Specialist Agent Factories (`werðan/orchestrator::agents`)**: Six `rig::agent::AgentBuilder` factories mirroring the specialist pane council — `orchestrator`, `legacy_analyst`, `rules_extractor`, `translator_test` (equipped with `HashlineReplaceTool` plus the verify runners), `architecture_critic`, and `security_auditor` (read-only tools). Each factory injects a role preamble kept under 700 tokens, distilled from the deleted `.omp/agents/` manifests' mandates.
- **Completion Model Provider**: Wired through `rig::providers::openai::Client::from_url`, with the base URL overridable via `WERTHAN_BASE_URL` (e.g. a local ninfer OpenAI-compatible endpoint) and the model selectable via `WERTHAN_MODEL`. When `OPENAI_API_KEY` is unset, the factories return `None` and callers fall back to a simulated completion path (`orchestrator::agents::simulated_reply`), keeping the whole workspace compiling and testable without any provider credentials.

#### 8. Quality & Environment Governance
- All Rust workspace crates enforce the repository's 6 quality check linters at build time: `flatmods`, `pathqual`, `plumbing`, `ste100`, `stepdown`, and `turbofish`.
- Nix environment parity: `flake.nix` devShell packages explicitly provide `pkgs.scc` (source code line/complexity counting) for analysis.

### Consequences

* **Good**: 100% pure Rust architecture with zero external runtime dependencies (no process wrappers, RPC daemons, or Node.js bridges).
* **Good**: Native in-process 60/40 Ratatui terminal cockpit with CSI-2026 synchronization (`\x1b[?2026h`), delivering zero-flicker, sub-millisecond redraws directly from `werðan/daemon`.
* **Good**: Native Rust implementation of all OMP engineering achievements: Hashline content-addressed editing (`werðan/harness::hashline`), SnapCompact frame signing & cache stubs (`werðan/harness::snapcompact`), Time-Traveling Stream Rules (`werðan/harness::stream_rules`), Persistent LSP diagnostics loopback (`werðan/harness::lsp_tools`), In-process search (`werðan/harness::lsp_tools`), In-process filesystem sandbox (`werðan/harness::sandbox`), and Magic directive extraction (`werðan/protocol::directives`).
* **Good**: High cohesion and loose coupling across 8 dedicated deep-domain workspace crates (`protocol`, `harness`, `jev`, `rea`, `verify`, `orchestrator`, `tui`, `daemon`), each consolidating former shallow crates along domain boundaries (e.g. `harness` absorbs `hashline`, `lsp-tools`, `snapcompact`, `stream-rules`, `sandbox`, `config`, and `worker-pool`; `protocol` absorbs `keywords` as `protocol::directives`).
* **Good**: Fast Jev 3-role middleware (~250ms latency) evaluating profile routing, Pi-warden safety guardrails, and human gate verification proofs without cloud roundtrips.
* **Good**: 6 human decision gates guarantee developer oversight and prevent runaway autonomous agent drift on mission-critical legacy rewrites.
* **Good**: Interactive execution enabled by default via `mise run tui` alongside headless `--smoke` / `--dry-run` modes for CI and testing.

### Confirmation

- Verified workspace organization across all 8 deep-domain pure Rust crates in `werðan/*` (`cargo check --workspace --all-targets` clean: 0 errors, 0 warnings).
- Verified all 7 OMP engineering systems wired and asserted in `orchestrator::smoke` (`cargo run -p daemon -- --smoke` passes).
- Verified native 60/40 Ratatui TUI running directly in `run_daemon_server()` with CSI-2026 synchronization.
- Verified end-to-end smoke execution passing via `cargo run -p daemon -- --smoke` and `mise run tui --smoke`.
- Verified all workspace tests passing via `cargo test --workspace`.
- All workspace checks verified against repository linter rules (`pathqual`, `stepdown`, `ste100`, `flatmods`, `plumbing`, `turbofish`).
