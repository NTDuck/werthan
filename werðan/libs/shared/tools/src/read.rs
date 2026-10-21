// Pinned commit: d5cd750e41fede277f9eb933455da92a0738915e
// Reference: https://github.com/can1357/oh-my-pi/blob/d5cd750e41fede277f9eb933455da92a0738915e/packages/coding-agent/src/tools/read.ts

/// Arguments for the read tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct ReadArgs {
    pub path: ::std::string::String,
}

/// Output of the read tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct ReadOutput {
    pub content: ::std::string::String,
    pub lines_read: ::core::primitive::usize,
}

/// Omp read tool inspecting source files and directory listings.
#[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::default::Default)]
pub struct ReadTool;

impl ::rig::tool::Tool for self::ReadTool {
    const NAME: &'static ::core::primitive::str = "read";

    type Error = ::rig::tool::ToolError;
    type Args = self::ReadArgs;
    type Output = self::ReadOutput;

    async fn definition(&self, _prompt: ::std::string::String) -> ::rig::completion::ToolDefinition {
        let parameters = ::serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Local path, internal URI, or URL. Selectors inline."
                }
            },
            "required": ["path"]
        });
        let name = Self::NAME.to_string();
        let description = "Read file contents or directory listings with optional line selectors.".to_string();
        ::rig::completion::ToolDefinition {
            name,
            description,
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        let read_result = ::std::fs::read_to_string(&args.path);
        let content = match read_result {
            ::core::result::Result::Ok(content_str) => content_str,
            ::core::result::Result::Err(io_err) => {
                let boxed_err = ::std::boxed::Box::new(io_err);
                return ::core::result::Result::Err(::rig::tool::ToolError::ToolCallError(boxed_err));
            },
        };
        let lines_read = content.lines().count();
        let out = self::ReadOutput {
            content,
            lines_read,
        };
        ::core::result::Result::Ok(out)
    }
}
