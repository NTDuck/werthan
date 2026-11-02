// Reference commit d5cd750e41fede277f9eb933455da92a0738915e from https://github.com/can1357/oh-my-pi
// Reference: https://github.com/can1357/oh-my-pi/blob/d5cd750e41fede277f9eb933455da92a0738915e/packages/coding-agent/src/tools/write.ts

use crate::DisplayExt;
/// Write tool.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::default::Default)]
pub struct WriteTool;

/// Write tool arguments.
#[derive(::serde::Deserialize, ::serde::Serialize, ::core::clone::Clone, ::core::fmt::Debug)]
pub struct WriteArgs {
    pub path: ::std::string::String,
    pub content: ::core::option::Option<::std::string::String>,
}

/// Write tool output.
#[derive(::serde::Deserialize, ::serde::Serialize, ::core::clone::Clone, ::core::fmt::Debug)]
pub struct WriteOutput {
    pub path: ::std::string::String,
    pub bytes_written: ::core::primitive::usize,
}

impl ::rig::tool::Tool for self::WriteTool {
    const NAME: &'static ::core::primitive::str = "write";

    type Error = ::rig::tool::ToolError;
    type Args = self::WriteArgs;
    type Output = self::WriteOutput;

    async fn definition(&self, _prompt: ::std::string::String) -> ::rig::completion::ToolDefinition {
        let name = Self::NAME.to_string();
        let description = "Write content to a file at the specified path.".to_string();
        let parameters = ::serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the destination file."
                },
                "content": {
                    "type": "string",
                    "description": "Content to write into the file."
                }
            },
            "required": ["path"]
        });
        ::rig::completion::ToolDefinition {
            name,
            description,
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        self::execute_write(args).await
    }
}

async fn execute_write(args: self::WriteArgs) -> ::core::result::Result<self::WriteOutput, ::rig::tool::ToolError> {
    let target_path = ::std::path::Path::new(&args.path);
    let parent_opt = target_path.parent();
    if let ::core::option::Option::Some(parent_dir) = parent_opt {
        let parent_empty = parent_dir.as_os_str().is_empty();
        if !parent_empty {
            let mkdir_res = ::tokio::fs::create_dir_all(parent_dir).await;
            if let ::core::result::Result::Err(io_err) = mkdir_res {
                let err = io_err.into_tool_error();
                return ::core::result::Result::Err(err);
            }
        }
    }

    let payload = args.content.as_deref().unwrap_or("");

    let payload_bytes = payload.as_bytes();
    let bytes_written = payload_bytes.len();
    let write_res = ::tokio::fs::write(&target_path, payload_bytes).await;
    if let ::core::result::Result::Err(io_err) = write_res {
        let err = io_err.into_tool_error();
        return ::core::result::Result::Err(err);
    }
    let path = args.path;
    let output = self::WriteOutput {
        path,
        bytes_written,
    };
    ::core::result::Result::Ok(output)
}
