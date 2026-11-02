# 02 — Artifact Repository Audit: Codebase Architecture & MCP Tooling

## 1. Repository Layout & File Inventory

The ReCodeAgent artifact repository (`benchmarks/assets/ReCodeAgent/`) implements the framework via a hybrid Python/Go system wrapped around the Anthropic Claude Code CLI.

```text
benchmarks/assets/ReCodeAgent/
├── configs/
│   └── prompt_templates.yaml      # 880 lines of Jinja2 prompt templates for all agents
├── docker-env/
│   └── Dockerfile                 # Multi-language build environment (Rust, Go, Python, Java, C)
├── scripts/                       # Shell automation for Docker execution and benchmark evaluation
└── src/
    ├── agents/
    │   ├── baseagent/             # Direct baseline translators (concatenated, condensed)
    │   └── recodeagent/           # Core 4-agent pipeline
    │       ├── agent.py           # Base RecodeAgent class and main orchestration loop
    │       ├── prompt_generator.py # Jinja2 prompt rendering engine
    │       ├── analyzer/          # AnalyzerAgent implementation
    │       ├── planning/          # PlanningAgent implementation
    │       ├── translator/        # TranslatorAgent implementation
    │       └── validator/         # ValidatorAgent implementation
    ├── analysis/                  # Post-run evaluation, test coverage, cost & graphectory scripts
    ├── mcp/                       # Model Context Protocol (MCP) server integrations
    │   ├── mcp_project_analyzer/  # FastMCP + Tree-sitter AST extraction (Python, 2,123 LoC)
    │   └── mcp_language_server/   # Custom LSP client & tool provider written in Go
    ├── utils/
    │   ├── cmd_utils.py           # Subprocess invocation of `claude -p` CLI and stream-JSON parser
    │   └── model_utils.py         # LLM interaction wrapper
    └── run.py                     # CLI entrypoint for running ReCodeAgent
```

---

## 2. Core Orchestrator Mechanics (`src/agents/recodeagent/agent.py`)

The main entrypoint for execution is the `run_recodeagent` async function in `src/agents/recodeagent/agent.py`:

```python
async def run_recodeagent(
    config: Dict[str, Any],
    skip_agents: List[str] = None,
    only_agents: List[str] = None
) -> bool:
    # 1. Analyzer Phase
    analyzer_success, analyzer_results = await analyzer_agent.run(project_details)
    if not analyzer_success: return False

    # 2. Planning Phase
    planning_success, planning_results = await planning_agent.run(project_details)
    if not planning_success: return False

    # 3. Translation-Validation Loop
    max_iterations = config.get("max_translation_validation_iterations", 5)
    iteration = 0
    validation_passed = False

    while iteration < max_iterations and not validation_passed:
        iteration += 1
        translator_success, translator_results = await translator_agent.run(project_details)
        validator_success, validator_results = await validator_agent.run(project_details)
        
        # Check if validation-summary.md exists
        if Path(planning_dir / "validation-summary.md").exists():
            validation_passed = True
            break
```

### Architectural Deficiencies in Orchestration:
1. **Unstructured File-System State Passing:** The orchestrator does not pass structured state vectors in memory. Instead, communication between phases occurs strictly through markdown files in `planning/` (`<source>-project-research.md`, `<target>-overall-design.md`, `name-mapping.json`, `implementation-plan.md`, `validation-report.md`).
2. **Binary Convergence Signal:** Convergence is signaled solely by whether the Validator agent decided to create a file named `validation-summary.md` and delete `validation-report.md`. If the agent hallucinates or crashes, the loop spins blindly until hitting `max_iterations = 5`.
3. **No Rollback Capability:** If an interface mismatch or cyclic dependency is discovered during iteration 3, the orchestrator cannot rollback to the Planning or Analyzer agent. It is locked into a local patch loop.

---

## 3. Subprocess Execution via Claude Code CLI (`src/utils/cmd_utils.py`)

A critical revelation of the artifact audit is that ReCodeAgent does not use direct API calls (e.g., standard Anthropic Python SDK). Instead, it delegates all agent reasoning to the **Claude Code CLI** via an asynchronous subprocess:

```python
cmd = ["claude", "-p", prompt]
if configs.get("mcp_config_file"):
    cmd += ["--mcp-config", configs["mcp_config_file"]]
if configs.get("extra_agent_args"):
    cmd += configs["extra_agent_args"]
```

The function `run_claude_code` intercepts the streamed JSON events (`--stream-json`) emitted by the Claude Code CLI, parsing tool invocations, bash executions, and final markdown results.
- **Consequence:** ReCodeAgent inherits all built-in capabilities and heuristics of Claude Code (file editing tools, bash execution, web fetch, subagent spawning).
- **Consequence for SLMs:** Claude Code CLI was designed specifically for frontier Anthropic models (Claude 3.5 Sonnet / Opus). It expects the underlying model to understand complex tool-calling protocols and tolerate massive multi-turn conversation histories.

---

## 4. MCP Tool Implementations: Language Servers & Tree-Sitter Analyzers

ReCodeAgent exposes two custom MCP tool servers to Claude Code:

### 4.1 FastMCP Project Analyzer (`src/mcp/mcp_project_analyzer/project_analyzer.py`)
Spans 2,123 lines of Python leveraging `fastmcp` and `tree-sitter` for 6 languages (`tree-sitter-python`, `tree-sitter-java`, `tree-sitter-javascript`, `tree-sitter-go`, `tree-sitter-c`, `tree-sitter-rust`).
- `extract_file_skeleton`: Parses ASTs and extracts class declarations, function signatures, docstrings, and imports, stripping method bodies.
- `get_directory_tree`: Traverses project directories, generating a clean file tree.
- `get_file_structure`: Queries specific AST nodes for symbols and variable types.

### 4.2 Go-Based Language Server Bridge (`src/mcp/mcp_language_server/`)
A custom MCP server written in Go that acts as a bridge to native Language Server Protocol (LSP) daemons (`clangd` for C/C++, `rust-analyzer` for Rust, `gopls` for Go, `pyright` for Python, `typescript-language-server` for JS/TS, `jdtls` for Java). Exposes tools:
- `mcp__<lang>-language-server__definition`: Jump to symbol definition.
- `mcp__<lang>-language-server__hover`: Query type information and inline documentation.
- `mcp__<lang>-language-server__references`: Find all references across the workspace.
- `mcp__<lang>-language-server__diagnostics`: Retrieve compiler errors and type warnings.
- `mcp__<lang>-language-server__rename_symbol`: Refactor identifier names across files.

---

## 5. Jinja2 Prompt Templates (`configs/prompt_templates.yaml`)

The prompt engineering of ReCodeAgent is codified in `configs/prompt_templates.yaml`, spanning 880 lines. Each agent receives a colossal prompt template (often 2,000–3,500 tokens before hydration) prescribing rigid multi-step procedures:

1. **Mandatory In-The-Loop Validation Scripts:** The Planning template requires the agent to write a Python script `validate_functions.py`, run it in bash, check its exit code, fix errors, and delete the script before proceeding.
2. **Absolute Identifier Name Preservation:**
   ```yaml
   - PRESERVE SOURCE NAMES EXACTLY - DO NOT RENAME IDENTIFIERS
   - All class, method, function, variable, and constant names MUST remain IDENTICAL to the source.
   - Do NOT convert between naming conventions (e.g., do NOT change snake_case to camelCase or vice versa).
   ```
3. **Hardcoded Benchmark Exceptions:** The prompt templates contain explicit `if CRUST` branches hardcoding behavior for specific benchmark categories, proving that the methodology is not purely language-agnostic.
