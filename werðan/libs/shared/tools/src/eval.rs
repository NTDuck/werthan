// Pinned commit: d5cd750e41fede277f9eb933455da92a0738915e
// Reference: https://github.com/can1357/oh-my-pi/blob/d5cd750e41fede277f9eb933455da92a0738915e/packages/coding-agent/src/tools/eval.ts

use crate::DisplayExt;

/// Arguments for code evaluation tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct EvalArgs {
    pub language: ::std::string::String,
    pub code: ::std::string::String,
    pub title: ::core::option::Option<::std::string::String>,
    pub timeout: ::core::option::Option<::core::primitive::u64>,
    pub reset: ::core::option::Option<::core::primitive::bool>,
}

/// Output from code evaluation tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct EvalOutput {
    pub stdout: ::std::string::String,
    pub stderr: ::std::string::String,
    pub result: ::core::option::Option<::std::string::String>,
}

/// Code evaluation tool for Python and JavaScript.
#[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::default::Default)]
pub struct EvalTool;

impl ::rig::tool::Tool for self::EvalTool {
    const NAME: &'static ::core::primitive::str = "eval";

    type Error = ::rig::tool::ToolError;
    type Args = self::EvalArgs;
    type Output = self::EvalOutput;

    async fn definition(&self, _prompt: ::std::string::String) -> ::rig::completion::ToolDefinition {
        let parameters = ::serde_json::json!({
            "type": "object",
            "properties": {
                "language": {
                    "type": "string",
                    "description": "Programming language identifier like py or js."
                },
                "code": {
                    "type": "string",
                    "description": "Source code snippet to execute."
                },
                "title": {
                    "type": "string",
                    "description": "Optional title for the execution context."
                },
                "timeout": {
                    "type": "integer",
                    "description": "Execution timeout in seconds."
                },
                "reset": {
                    "type": "boolean",
                    "description": "Reset session state before execution."
                }
            },
            "required": ["language", "code"]
        });
        let name = Self::NAME.to_string();
        let description = "Execute code snippets in Python or JavaScript runtimes.".to_string();
        ::rig::completion::ToolDefinition {
            name,
            description,
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        self::execute_eval(&args).await
    }
}

async fn execute_eval(args: &self::EvalArgs) -> ::core::result::Result<self::EvalOutput, ::rig::tool::ToolError> {
    let runtime_pair = self::resolve_runtime(&args.language)?;
    let program = runtime_pair.0;
    let flag = runtime_pair.1;

    let mut cmd = ::tokio::process::Command::new(program);
    cmd.arg(flag);
    cmd.arg(&args.code);
    cmd.kill_on_drop(true);

    let timeout_secs = args.timeout.unwrap_or_else(|| {
        let env_val = ::core::env!("TOOL_EVAL_TIMEOUT_SECS");
        let parsed = env_val
            .parse::<::core::primitive::u64>()
            .expect("TOOL_EVAL_TIMEOUT_SECS must be a valid u64");
        parsed
    });
    let duration = ::tokio::time::Duration::from_secs(timeout_secs);
    let output_future = cmd.output();
    let timed_result = ::tokio::time::timeout(duration, output_future).await;

    let output = match timed_result {
        ::core::result::Result::Ok(process_result) => match process_result {
            ::core::result::Result::Ok(out) => out,
            ::core::result::Result::Err(err) => {
                let err_msg = ::std::format!("Failed to start {program} runtime: {err}");
                let err_obj = err_msg.into_tool_error();
                return ::core::result::Result::Err(err_obj);
            },
        },
        ::core::result::Result::Err(_) => {
            let err_msg = ::std::format!("Evaluation timed out after {timeout_secs} seconds.");
            let err_obj = err_msg.into_tool_error();
            return ::core::result::Result::Err(err_obj);
        },
    };

    let stdout_cow = ::std::string::String::from_utf8_lossy(&output.stdout);
    let stdout = stdout_cow.into_owned();
    let stderr_cow = ::std::string::String::from_utf8_lossy(&output.stderr);
    let stderr = stderr_cow.into_owned();

    let trimmed_stdout = stdout.trim();
    let result = if trimmed_stdout.is_empty() {
        ::core::option::Option::None
    } else {
        let res_str = trimmed_stdout.to_string();
        ::core::option::Option::Some(res_str)
    };

    let eval_output = self::EvalOutput {
        stdout,
        stderr,
        result,
    };
    ::core::result::Result::Ok(eval_output)
}

fn resolve_runtime(
    lang: &::core::primitive::str,
) -> ::core::result::Result<(&'static ::core::primitive::str, &'static ::core::primitive::str), ::rig::tool::ToolError> {
    let normalized = lang.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "py" | "python" | "python3" => {
            ::core::result::Result::Ok(("python3", "-c"))
        },
        "js" | "javascript" | "node" => {
            ::core::result::Result::Ok(("node", "-e"))
        },
        _ => {
            let err_msg = ::std::format!("Unsupported evaluation language: {normalized}");
            let err_obj = err_msg.into_tool_error();
            ::core::result::Result::Err(err_obj)
        },
    }
}
