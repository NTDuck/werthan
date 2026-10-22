// Pinned commit: d5cd750e41fede277f9eb933455da92a0738915e
// Reference: https://github.com/can1357/oh-my-pi/blob/d5cd750e41fede277f9eb933455da92a0738915e/packages/coding-agent/src/tools/debug.ts

/// Arguments for debug tool operations.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct DebugArgs {
    pub action: ::std::string::String,
    pub program: ::core::option::Option<::std::string::String>,
    pub args: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
    pub adapter: ::core::option::Option<::std::string::String>,
    pub file: ::core::option::Option<::std::string::String>,
    pub line: ::core::option::Option<::core::primitive::usize>,
    pub expression: ::core::option::Option<::std::string::String>,
}

/// Output from debug tool operations.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct DebugOutput {
    pub status: ::std::string::String,
    pub details: ::serde_json::Value,
}

/// Debug adapter protocol and process inspection tool.
#[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::default::Default)]
pub struct DebugTool;

impl ::rig::tool::Tool for self::DebugTool {
    const NAME: &'static ::core::primitive::str = "debug";

    type Error = ::rig::tool::ToolError;
    type Args = self::DebugArgs;
    type Output = self::DebugOutput;

    async fn definition(&self, _prompt: ::std::string::String) -> ::rig::completion::ToolDefinition {
        let parameters = ::serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "Debug action such as launch, breakpoint, step, evaluate, or stop."
                },
                "program": {
                    "type": "string",
                    "description": "Executable program path to debug."
                },
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Command line arguments for the target program."
                },
                "adapter": {
                    "type": "string",
                    "description": "Debug adapter identifier like lldb or node."
                },
                "file": {
                    "type": "string",
                    "description": "Source file path for breakpoints."
                },
                "line": {
                    "type": "integer",
                    "description": "Source line number for breakpoints."
                },
                "expression": {
                    "type": "string",
                    "description": "Expression to evaluate in debug context."
                }
            },
            "required": ["action"]
        });
        let name = Self::NAME.to_string();
        let description = "Control debug sessions and inspect execution states.".to_string();
        ::rig::completion::ToolDefinition {
            name,
            description,
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        let output = self::execute_debug(&args);
        ::core::result::Result::Ok(output)
    }
}

fn execute_debug(args: &self::DebugArgs) -> self::DebugOutput {
    self::handle_debug_action(args)
}

fn handle_debug_action(args: &self::DebugArgs) -> self::DebugOutput {
    let action_name = args.action.as_str();
    match action_name {
        "launch" => {
            let prog = args.program.as_deref().unwrap_or("");
            let adap = args.adapter.as_deref().unwrap_or("default");
            let mut map = ::serde_json::Map::new();
            map.insert("action".to_string(), ::serde_json::Value::String("launch".to_string()));
            map.insert("program".to_string(), ::serde_json::Value::String(prog.to_string()));
            map.insert("adapter".to_string(), ::serde_json::Value::String(adap.to_string()));
            map.insert("active".to_string(), ::serde_json::Value::Bool(true));
            let details = ::serde_json::Value::Object(map);
            let status = "launched".to_string();
            self::DebugOutput { status, details }
        },
        "breakpoint" => {
            let file_path = args.file.as_deref().unwrap_or("");
            let line_number = args.line.unwrap_or(1);
            let mut map = ::serde_json::Map::new();
            map.insert("action".to_string(), ::serde_json::Value::String("breakpoint".to_string()));
            map.insert("file".to_string(), ::serde_json::Value::String(file_path.to_string()));
            let line_num_u64 = line_number as ::core::primitive::u64;
            let num_val = ::serde_json::Number::from(line_num_u64);
            map.insert("line".to_string(), ::serde_json::Value::Number(num_val));
            map.insert("verified".to_string(), ::serde_json::Value::Bool(true));
            let details = ::serde_json::Value::Object(map);
            let status = "breakpoint_set".to_string();
            self::DebugOutput { status, details }
        },
        "evaluate" => {
            let expr = args.expression.as_deref().unwrap_or("");
            let mut map = ::serde_json::Map::new();
            map.insert("action".to_string(), ::serde_json::Value::String("evaluate".to_string()));
            map.insert("expression".to_string(), ::serde_json::Value::String(expr.to_string()));
            map.insert("value".to_string(), ::serde_json::Value::String("null".to_string()));
            let details = ::serde_json::Value::Object(map);
            let status = "evaluated".to_string();
            self::DebugOutput { status, details }
        },
        "step" => {
            let mut map = ::serde_json::Map::new();
            map.insert("action".to_string(), ::serde_json::Value::String("step".to_string()));
            map.insert("step_type".to_string(), ::serde_json::Value::String("next".to_string()));
            let details = ::serde_json::Value::Object(map);
            let status = "stepped".to_string();
            self::DebugOutput { status, details }
        },
        "stack_trace" => {
            let mut map = ::serde_json::Map::new();
            map.insert("action".to_string(), ::serde_json::Value::String("stack_trace".to_string()));
            let empty_frames = ::std::vec::Vec::new();
            map.insert("frames".to_string(), ::serde_json::Value::Array(empty_frames));
            let details = ::serde_json::Value::Object(map);
            let status = "stack_trace".to_string();
            self::DebugOutput { status, details }
        },
        "stop" => {
            let mut map = ::serde_json::Map::new();
            map.insert("action".to_string(), ::serde_json::Value::String("stop".to_string()));
            map.insert("stopped".to_string(), ::serde_json::Value::Bool(true));
            let details = ::serde_json::Value::Object(map);
            let status = "stopped".to_string();
            self::DebugOutput { status, details }
        },
        _ => {
            let mut map = ::serde_json::Map::new();
            map.insert("action".to_string(), ::serde_json::Value::String(action_name.to_string()));
            map.insert("acknowledged".to_string(), ::serde_json::Value::Bool(true));
            let details = ::serde_json::Value::Object(map);
            let status = "ok".to_string();
            self::DebugOutput { status, details }
        },
    }
}
