# Shared Tools & Configuration Schema

Entry point: [../tech-spec.md](../tech-spec.md). This file documents the 14 agent tools ([ADR-0013](../../decisions/0013-omp-tool-suite-conformance.md)), the canonical configuration schema ([ADR-0009](../../decisions/0009-canonical-campaign-configschema.md)), and the configurator agent.

## 1. Shared Agent Tools Architecture (`werðan/libs/shared/tools`)

The `tools` crate provides 14 self-contained, omp-specification-conforming agent tools exposed as `::rig::tool::Tool` implementations ([ADR-0013](../../decisions/0013-omp-tool-suite-conformance.md)).

### 1.1 Tool Registry & Interface Specifications

| Module | Exported Symbols | Input Arguments (`Tool::Args`) | Output (`Tool::Output`) | Behavioral Contract & Engine |
| :--- | :--- | :--- | :--- | :--- |
| `read` | `ReadTool`, `ReadArgs`, `ReadOutput` | `path: String` | `content: String`, `lines_read: usize` | Asynchronous file content reading with line count derivation. |
| `write` | `WriteTool`, `WriteArgs`, `WriteOutput` | `path: String`, `content: Option<String>` | `path: String`, `bytes_written: usize` | File writing with automatic parent directory creation via `tokio::fs::create_dir_all`. |
| `edit` | `EditTool`, `EditArgs`, `EditOutput` | `input: String` | `summary: String` | Hashline surgical patch parser supporting `ReplaceRange` (`N.=M:`), `InsertBefore` (`<N:`), and `InsertAfter` (`>N:`) with order-preserving array splicing. |
| `glob` | `GlobTool`, `GlobArgs`, `GlobOutput` | `path`, `hidden`, `gitignore`, `limit` (all optional) | `matches: Vec<String>` | Path discovery using `globset::GlobBuilder` and `ignore::WalkBuilder`; respects `.gitignore` and `hidden` flags; prunes `.git`, `node_modules`, `target`. Default limit 100, overridable via `TOOL_GLOB_LIMIT`. |
| `grep` | `GrepTool`, `GrepArgs`, `GrepOutput`, `GrepMatch` | `pattern: String`, optional `path`, `case`, `gitignore`, `skip` | `matches: Vec<GrepMatch>` | Regex content search via `ignore::WalkBuilder` + `regex::RegexBuilder`; prunes `.git`, `node_modules`, `target`; caps at 500 matches. |
| `ast_grep` | `AstGrepTool`, `AstGrepArgs`, `AstGrepOutput`, `AstMatch` | `pat: String`, optional `path`, `lang`, `skip` | `matches: Vec<AstMatch>` | AST pattern discovery via `ast-grep-core` (`Pattern::try_new` + `AstGrep` over `ast_lang` tree-sitter grammars: rust, ts, tsx, js, py, json, toml, md; unsupported `lang` errors); prunes `.git`, `node_modules`, `target`, `.idea`. |
| `ast_edit` | `AstEditTool`, `AstEditArgs`, `AstEditOutput`, `AstEditOp` | `ops: Vec<AstEditOp>`, `paths: Vec<String>` | `files_modified: Vec<String>`, `edits_applied: usize` | AST structural rewrites with native `replace_by` template substitution; language inference from `ast_lang`; byte-range splicing applies non-overlapping replacements. |
| `lsp` | `LspTool`, `LspArgs`, `LspOutput` | `action: String`, optional `file`, `line`, `symbol`, `query`, `new_name`, `apply`, `timeout`, `payload` | `status: String`, `result: serde_json::Value` | Language server operations (`status`, `diagnostics`, `definition`, `references`, `hover`, `symbols`, `rename`) backed by workspace file scanning. |
| `bash` | `BashTool`, `BashArgs`, `BashOutput` | `command: String`, optional `timeout`, `cwd`, `pty`, `async_mode`, `name` | `exit_code: i32`, `stdout: String`, `stderr: String` | Process execution via `tokio::process::Command` under `sh -c` with timeout monitoring (default 60s, overridable via `TOOL_BASH_TIMEOUT_SECS`). |
| `eval` | `EvalTool`, `EvalArgs`, `EvalOutput` | `language: String`, `code: String`, optional `timeout` | `status: String`, `stdout: String`, `stderr: String` | Subprocess execution of `python3` or `node` snippets with exit capture (default 60s, overridable via `TOOL_EVAL_TIMEOUT_SECS`). |
| `debug` | `DebugTool`, `DebugArgs`, `DebugOutput` | `action: String`, optional `session_id`, `payload` | `status: String`, `result: serde_json::Value` | DAP debugger adapter status and control inspector. |
| `github` | `GithubTool`, `GithubArgs`, `GithubOutput` | `op: String`, optional `query`, `repo`, `limit` | `status: String`, `data: serde_json::Value` | Headless GitHub CLI (`gh`) runner querying issues, PRs, and repositories. |
| `ask` | `AskTool`, `AskArgs`, `AskOutput`, `AskQuestion`, `AskAnswer` | `questions: Vec<AskQuestion>` | `answers: Vec<AskAnswer>` | Typed question surfacing (id, text, options, recommended) to the human operator through a process-global canned-answer sink; honest error when no operator answer exists. |
| `partition_write` | `PartitionWriteTool` | `WriteArgs` (reused from `write`: `path: String`, `content: Option<String>`) | `WriteOutput` (reused from `write`: `path: String`, `bytes_written: usize`) | Partition-scoped mutation bounding every write to one declared root, constructed with `PartitionWriteTool::new(partition_root: PathBuf)` (VAL-TOOL-02). Rejects absolute paths and `ParentDir` components up front, and re-canonicalizes the written file to reject a symlink that resolves outside the root; both rejections return `ToolCallError` with a message naming the offending path and root. Filesystem failures convert through `DisplayExt::into_tool_error`. The plain `write` tool stays general. |

### 1.2 Error Propagation & Blanket Trait Implementation

All 14 tools standardize error conversion through the `DisplayExt` blanket trait, which replaced the earlier `tool_err` helper ([ADR-0013](../../decisions/0013-omp-tool-suite-conformance.md)):

```rust
pub(crate) trait DisplayExt: ::core::fmt::Display {
    fn into_tool_error(&self) -> ::rig::tool::ToolError {
        let err_str = self.to_string();
        let io_err = ::std::io::Error::other(err_str);
        let boxed = ::std::boxed::Box::new(io_err);
        ::rig::tool::ToolError::ToolCallError(boxed)
    }
}

impl<Target: ::core::fmt::Display + ?::core::marker::Sized> self::DisplayExt for Target {}
```

## 2. Canonical Configuration Schema (`configschema::CampaignConfig`)

The canonical migration.yml schema is `configschema::CampaignConfig` (crate `werðan/libs/shared/configschema`) with API `from_yaml_text`, `from_yaml_path`, `to_yaml_text` ([ADR-0009](../../decisions/0009-canonical-campaign-configschema.md)). Both frontends deserialize into this single type; per-frontend parse structs do not exist. The YAML field `ariadne.max_rollouts` is a serde alias of `engine_budget_per_module`, keeping the 118 generated benchmark configs parsing.

### 2.1 Schema Shape

| Section | Fields |
| :--- | :--- |
| `campaign_id` | `String` campaign identifier. |
| `execution_mode` | `"hitl"` \| `"full_auto"`. |
| `paths` | `input_legacy_dir`, `workspace_root`, `off_limits_globs`. |
| `stack` | `source_language`, `binary_artifacts`, `use_ghidra_rea`, `target_language`, `target_edition_or_framework`, `target_build_cmd`, `target_test_cmd`, `legacy_test_harness_cmd`. |
| `inference` | `base_url`, `model`, `max_parallel_shards`, `max_shards`, `inference_max_tokens`, `inference_temperature`, `inference_timeout_secs`, `build_timeout_secs`, `test_timeout_secs`. |
| `verification` | `max_self_heal_retries`, `require_byte_identical_io`. |
| `hitl` | `gate_on_reimagine`, `gate_on_macro_regression`, `extrinsic_conflict_escalation` (`"auto_negotiate"` \| `"ask_human"`). |
| `ariadne` | `engine_budget_per_module` (serde alias `max_rollouts`), `ucb_exploration_c`, `softmax_temperature`, `reward_weights` (`compile`, `characterization_parity`, `hardening_score`), `llm_backend` (`base_url`, `model`, `max_tokens`, `temperature`, `timeout_secs`), `cargo_harness` (`build_timeout_secs`, `test_timeout_secs`). |

### 2.2 Defined-but-Not-Consumed Sections

`AriadneSection.llm_backend` and `AriadneSection.cargo_harness` exist in `werðan/libs/shared/configschema/src/lib.rs` for schema completeness. The daemon reads `inference.*` for the OpenAI-compatible model (`bench_llm::OpenAiCompatModel::from_inference`) and `ariadne.engine_budget_per_module` for the rollout budget; it does not read `llm_backend` or `cargo_harness` today. Both sections are schema-complete; consumer wiring is pending.

## 3. Configurator Agent (`werðan/libs/shared/agents/configurator`)

The configurator is frontend-agnostic (crate `configurator`), driving interactive campaign configuration through the `ask` tool bridge — not through Rig completion:

- **`ConfiguratorAgent`**: owns the typed question script and applies answers to a `CampaignConfig` draft.
- **`ConfigQuestion` / `ConfigAnswer`**: typed question-and-answer records surfaced through the `ask` bridge.
- **`AskBridge` trait**: the frontend-supplied transport connecting `ConfigQuestion` sequences to the human operator (interactive TUI inline prompt in the cockpit; test sink in deterministic tests).
- **`question_script`**: the ordered question sequence covering campaign identity, paths, stack, inference, verification, and HITL sections.
- **`apply_answer`**: maps a `ConfigAnswer` onto the config draft; `key=value` arguments take the fast path, bypassing unanswered questions.

The interactive `/init` command runs the wizard: no arguments execute the full `question_script`, `key=value` arguments apply directly through `apply_answer`. The draft persists via `/save-config`. Operator-facing semantics: [../product-config.md](../product-config.md).
