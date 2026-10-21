//! Shared utility tools and standard omp agent tools implementation.

mod ast_edit;
mod ast_grep;
mod ask;
mod ast_lang;
mod bash;
mod debug;
mod edit;
mod eval;
mod github;
mod glob;
mod grep;
mod lsp;
mod read;
mod write;

pub use self::ast_edit::{AstEditArgs, AstEditOp, AstEditOutput, AstEditTool};
pub use self::ast_grep::{AstGrepArgs, AstGrepOutput, AstGrepTool, AstMatch};
pub use self::ask::{AskAnswer, AskArgs, AskOutput, AskQuestion, AskTool, clear_canned_answers, push_canned_answers};
pub use self::bash::{BashArgs, BashOutput, BashTool};
pub use self::debug::{DebugArgs, DebugOutput, DebugTool};
pub use self::edit::{EditArgs, EditOutput, EditTool};
pub use self::eval::{EvalArgs, EvalOutput, EvalTool};
pub use self::github::{GithubArgs, GithubOutput, GithubTool};
pub use self::glob::{GlobArgs, GlobOutput, GlobTool};
pub use self::grep::{GrepArgs, GrepMatch, GrepOutput, GrepTool};
pub use self::lsp::{LspArgs, LspOutput, LspTool};
pub use self::read::{ReadArgs, ReadOutput, ReadTool};
pub use self::write::{WriteArgs, WriteOutput, WriteTool};
pub use ::rig::tool::Tool;

/// Extension trait converting displayable errors into rig tool errors.
pub(crate) trait DisplayExt: ::core::fmt::Display {
    /// Convert this displayable error into a rig ToolError.
    #[allow(clippy::wrong_self_convention)]
    fn into_tool_error(&self) -> ::rig::tool::ToolError {
        let err_str = self.to_string();
        let io_err = ::std::io::Error::other(err_str);
        let boxed = ::std::boxed::Box::new(io_err);
        ::rig::tool::ToolError::ToolCallError(boxed)
    }
}

impl<T: ::core::fmt::Display + ?::core::marker::Sized> self::DisplayExt for T {}
