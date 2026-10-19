//! Native rig tool implementations over harness primitives.

use ::serde::Deserialize;

/// Tool error type satisfying rig StdError bound.
#[derive(::core::fmt::Debug)]
pub struct ToolFailure(pub ::std::string::String);

impl ::core::fmt::Display for self::ToolFailure {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        ::core::write!(f, "{}", self.0)
    }
}

impl ::std::error::Error for self::ToolFailure {}

fn fail(msg: impl ::core::fmt::Display) -> self::ToolFailure {
    self::ToolFailure(msg.to_string())
}

/// Replace a line range between hashline anchors with replacement text.
#[derive(Deserialize)]
pub struct HashlineReplaceArgs {
    /// Target file path.
    pub path: String,
    /// Four-character start anchor.
    pub start_anchor: String,
    /// Four-character end anchor.
    pub end_anchor: String,
    /// Replacement text.
    pub replacement: String,
}

/// Tool wrapping harness hashline anchor replacement.
#[derive(Default)]
pub struct HashlineReplaceTool;

impl ::rig::tool::Tool for HashlineReplaceTool {
    const NAME: &'static str = "hashline_replace";

    type Error = ToolFailure;
    type Args = HashlineReplaceArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ::rig::completion::ToolDefinition {
        ::rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Replace lines between two 4-char hashline anchors in a file.".to_string(),
            parameters: ::serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "start_anchor": {"type": "string"},
                    "end_anchor": {"type": "string"},
                    "replacement": {"type": "string"}
                },
                "required": ["path", "start_anchor", "end_anchor", "replacement"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        let original = ::std::fs::read_to_string(&args.path).map_err(fail)?;
        let updated =
            crate::hashline::apply_hashline_replace(&original, &args.start_anchor, &args.end_anchor, &args.replacement)
                .map_err(fail)?;
        ::std::fs::write(&args.path, &updated).map_err(fail)?;
        Ok(format!("EDIT_APPLIED {}", args.path))
    }
}

/// Read a hashline-anchored view of a file region.
#[derive(Deserialize)]
pub struct ReadHashlineArgs {
    /// Target file path.
    pub path: String,
    /// Zero-based start line.
    pub start_line: usize,
    /// Maximum lines to render.
    pub max_lines: usize,
}

/// Tool rendering a hashline-anchored file view.
#[derive(Default)]
pub struct ReadHashlineTool;

impl ::rig::tool::Tool for ReadHashlineTool {
    const NAME: &'static str = "read_hashline";

    type Error = ToolFailure;
    type Args = ReadHashlineArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ::rig::completion::ToolDefinition {
        ::rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Render a file region with 4-character hashline anchors.".to_string(),
            parameters: ::serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "start_line": {"type": "integer"},
                    "max_lines": {"type": "integer"}
                },
                "required": ["path", "start_line", "max_lines"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        let source = ::std::fs::read_to_string(&args.path).map_err(fail)?;
        Ok(crate::hashline::render_hashlined_view(&source, args.start_line, args.max_lines))
    }
}

/// Search files under a root for a query string.
#[derive(Deserialize)]
pub struct SearchArgs {
    /// Root directory to search recursively.
    pub root: String,
    /// Query substring to find.
    pub query: String,
}

/// Tool running in-process recursive line search.
#[derive(Default)]
pub struct SearchTool;

impl ::rig::tool::Tool for SearchTool {
    const NAME: &'static str = "search";

    type Error = ToolFailure;
    type Args = SearchArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ::rig::completion::ToolDefinition {
        ::rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Recursively search files under a root for a query substring.".to_string(),
            parameters: ::serde_json::json!({
                "type": "object",
                "properties": {
                    "root": {"type": "string"},
                    "query": {"type": "string"}
                },
                "required": ["root", "query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        let matches =
            crate::lsp_tools::search_files_in_process(::std::path::Path::new(&args.root), &args.query).map_err(fail)?;
        Ok(::serde_json::to_string_pretty(&matches).unwrap_or_else(|_| "[]".to_string()))
    }
}
