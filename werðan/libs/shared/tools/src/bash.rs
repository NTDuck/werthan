// Pinned commit: d5cd750e41fede277f9eb933455da92a0738915e
// Reference: https://github.com/can1357/oh-my-pi/blob/d5cd750e41fede277f9eb933455da92a0738915e/packages/coding-agent/src/tools/bash.ts

use crate::DisplayExt;

/// Arguments for bash command execution.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct BashArgs {
    pub command: ::std::string::String,
    pub timeout: ::core::option::Option<::core::primitive::u64>,
    pub cwd: ::core::option::Option<::std::string::String>,
    pub pty: ::core::option::Option<::core::primitive::bool>,
    pub async_mode: ::core::option::Option<::core::primitive::bool>,
    pub name: ::core::option::Option<::std::string::String>,
}

/// Output from bash command execution.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct BashOutput {
    pub exit_code: ::core::primitive::i32,
    pub stdout: ::std::string::String,
    pub stderr: ::std::string::String,
}

/// Bash command execution tool.
#[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::default::Default)]
pub struct BashTool;

impl ::rig::tool::Tool for self::BashTool {
    const NAME: &'static ::core::primitive::str = "bash";

    type Error = ::rig::tool::ToolError;
    type Args = self::BashArgs;
    type Output = self::BashOutput;

    async fn definition(&self, _prompt: ::std::string::String) -> ::rig::completion::ToolDefinition {
        let parameters = ::serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell command to execute."
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds."
                },
                "cwd": {
                    "type": "string",
                    "description": "Working directory for the process."
                },
                "pty": {
                    "type": "boolean",
                    "description": "Run in pseudo-terminal mode."
                },
                "async_mode": {
                    "type": "boolean",
                    "description": "Run in background asynchronously."
                },
                "name": {
                    "type": "string",
                    "description": "Identifier name for session."
                }
            },
            "required": ["command"]
        });
        let name = Self::NAME.to_string();
        let description = "Execute a shell command and capture status and output.".to_string();
        ::rig::completion::ToolDefinition {
            name,
            description,
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        self::execute_bash(&args).await
    }
}

async fn execute_bash(args: &self::BashArgs) -> ::core::result::Result<self::BashOutput, ::rig::tool::ToolError> {
    let mut cmd = ::tokio::process::Command::new("sh");
    cmd.arg("-c");
    cmd.arg(&args.command);
    cmd.kill_on_drop(true);

    if let ::core::option::Option::Some(dir) = &args.cwd {
        cmd.current_dir(dir);
    }

    let timeout_secs = args.timeout.unwrap_or_else(|| {
        ::std::env::var("TOOL_BASH_TIMEOUT_SECS")
            .expect("TOOL_BASH_TIMEOUT_SECS must be set in environment")
            .parse::<::core::primitive::u64>()
            .expect("TOOL_BASH_TIMEOUT_SECS must be a valid u64")
    });
    let duration = ::tokio::time::Duration::from_secs(timeout_secs);
    let output_future = cmd.output();
    let timed_result = ::tokio::time::timeout(duration, output_future).await;

    let output = match timed_result {
        ::core::result::Result::Ok(process_result) => match process_result {
            ::core::result::Result::Ok(out) => out,
            ::core::result::Result::Err(err) => {
                let err_msg = ::std::format!("Process failed to start: {err}");
                let err_obj = err_msg.into_tool_error();
                return ::core::result::Result::Err(err_obj);
            },
        },
        ::core::result::Result::Err(_) => {
            let err_msg = ::std::format!("Command timed out after {timeout_secs} seconds.");
            let err_obj = err_msg.into_tool_error();
            return ::core::result::Result::Err(err_obj);
        },
    };

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout_cow = ::std::string::String::from_utf8_lossy(&output.stdout);
    let stdout = stdout_cow.into_owned();
    let stderr_cow = ::std::string::String::from_utf8_lossy(&output.stderr);
    let stderr = stderr_cow.into_owned();

    let bash_output = self::BashOutput {
        exit_code,
        stdout,
        stderr,
    };
    ::core::result::Result::Ok(bash_output)
}
