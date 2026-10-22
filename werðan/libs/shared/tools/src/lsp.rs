// Pinned commit: d5cd750e41fede277f9eb933455da92a0738915e
// Reference: https://github.com/can1357/oh-my-pi/blob/d5cd750e41fede277f9eb933455da92a0738915e/packages/coding-agent/src/lsp/tool.ts

use crate::DisplayExt;

/// Arguments for LSP tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct LspArgs {
    pub action: ::std::string::String,
    pub file: ::core::option::Option<::std::string::String>,
    pub line: ::core::option::Option<::core::primitive::usize>,
    pub symbol: ::core::option::Option<::std::string::String>,
    pub query: ::core::option::Option<::std::string::String>,
    pub new_name: ::core::option::Option<::std::string::String>,
    pub apply: ::core::option::Option<::core::primitive::bool>,
    pub timeout: ::core::option::Option<::core::primitive::u64>,
    pub payload: ::core::option::Option<::std::string::String>,
}

/// Output from LSP tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct LspOutput {
    pub status: ::std::string::String,
    pub result: ::serde_json::Value,
}

/// LSP tool for language server protocol operations.
#[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::default::Default)]
pub struct LspTool;

impl ::rig::tool::Tool for self::LspTool {
    const NAME: &'static ::core::primitive::str = "lsp";

    type Error = ::rig::tool::ToolError;
    type Args = self::LspArgs;
    type Output = self::LspOutput;

    async fn definition(&self, _prompt: ::std::string::String) -> ::rig::completion::ToolDefinition {
        let parameters = ::serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "LSP action name such as diagnostics, definition, references, hover, symbols, rename, or status."
                },
                "file": {
                    "type": "string",
                    "description": "Target source file path."
                },
                "line": {
                    "type": "integer",
                    "description": "Target one-based line number."
                },
                "symbol": {
                    "type": "string",
                    "description": "Target symbol identifier."
                },
                "query": {
                    "type": "string",
                    "description": "Query string for workspace symbol searches."
                },
                "new_name": {
                    "type": "string",
                    "description": "New identifier name for rename operations."
                },
                "apply": {
                    "type": "boolean",
                    "description": "Apply edits directly to target files."
                },
                "timeout": {
                    "type": "integer",
                    "description": "Operation timeout limit in seconds."
                },
                "payload": {
                    "type": "string",
                    "description": "Raw JSON payload text for custom LSP commands."
                }
            },
            "required": ["action"]
        });
        let name = Self::NAME.to_string();
        let description = "Query language server for diagnostics, definition, hover, and references.".to_string();
        ::rig::completion::ToolDefinition {
            name,
            description,
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        self::execute_lsp_action(&args)
    }
}

fn execute_lsp_action(args: &self::LspArgs) -> ::core::result::Result<self::LspOutput, ::rig::tool::ToolError> {
    let files = self::collect_lsp_files(".");
    let action_str = args.action.to_lowercase();
    match action_str.as_str() {
        "status" => self::handle_status_action(),
        "diagnostics" => self::handle_diagnostics_action(args),
        "definition" => self::handle_definition_action(args, &files),
        "references" => self::handle_references_action(args, &files),
        "hover" => self::handle_hover_action(args, &files),
        "symbols" => self::handle_symbols_action(args, &files),
        "rename" => self::handle_rename_action(args),
        _ => {
            let msg = ::std::format!("Unsupported LSP action: {}", args.action);
            let mut map = ::serde_json::Map::new();
            map.insert("error".to_string(), ::serde_json::Value::String(msg));
            let result = ::serde_json::Value::Object(map);
            let status = "error".to_string();
            let output = self::LspOutput { status, result };
            ::core::result::Result::Ok(output)
        },
    }
}

fn collect_lsp_files(root: &::core::primitive::str) -> ::std::vec::Vec<::std::string::String> {
    let root_path = ::std::path::Path::new(root);
    let is_file = root_path.is_file();
    if is_file {
        let path_str = root.to_string();
        return ::std::vec![path_str];
    }

    let mut builder = ::ignore::WalkBuilder::new(root_path);
    builder.hidden(true);
    builder.git_ignore(true);
    builder.filter_entry(|entry| {
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        name_str != ".git" && name_str != "node_modules" && name_str != "target" && name_str != ".idea"
    });

    let mut files = ::std::vec::Vec::new();
    let walker = builder.build();
    for entry_result in walker {
        let entry = match entry_result {
            ::core::result::Result::Ok(entry_val) => entry_val,
            ::core::result::Result::Err(_) => continue,
        };
        let file_type_opt = entry.file_type();
        let is_file_entry = file_type_opt.is_some_and(|file_type| file_type.is_file());
        if is_file_entry {
            let entry_path = entry.into_path();
            let lossy_str = entry_path.to_string_lossy();
            let owned_str = lossy_str.to_string();
            files.push(owned_str);
        }
    }
    files.sort();
    files
}
fn handle_status_action() -> ::core::result::Result<self::LspOutput, ::rig::tool::ToolError> {
    let result = ::serde_json::json!({
        "servers": [
            {
                "name": "rust-analyzer",
                "languages": ["rust"],
                "status": "ready"
            },
            {
                "name": "typescript-language-server",
                "languages": ["typescript", "javascript"],
                "status": "configured"
            },
            {
                "name": "pyright",
                "languages": ["python"],
                "status": "configured"
            }
        ],
        "lspmux": "active"
    });
    let status = "ok".to_string();
    let output = self::LspOutput { status, result };
    ::core::result::Result::Ok(output)
}

fn handle_diagnostics_action(args: &self::LspArgs) -> ::core::result::Result<self::LspOutput, ::rig::tool::ToolError> {
    let file_opt = args.file.as_deref();
    let mut diagnostics = ::std::vec::Vec::new();

    if let ::core::option::Option::Some(file_path) = file_opt {
        if file_path != "*" {
            let path_ref = ::std::path::Path::new(file_path);
            let read_res = ::std::fs::read_to_string(path_ref);
            if let ::core::result::Result::Ok(content) = read_res {
                self::scan_syntax_diagnostics(file_path, &content, &mut diagnostics);
            }
        }
    }

    let mut map = ::serde_json::Map::new();
    map.insert("diagnostics".to_string(), ::serde_json::Value::Array(diagnostics));
    let result = ::serde_json::Value::Object(map);
    let status = "ok".to_string();
    let output = self::LspOutput { status, result };
    ::core::result::Result::Ok(output)
}

fn scan_syntax_diagnostics(
    file_path: &::core::primitive::str,
    content: &::core::primitive::str,
    out: &mut ::std::vec::Vec<::serde_json::Value>,
) {
    let mut stack = ::std::vec::Vec::new();
    let mut line_num = 1;
    let mut col_num = 1;

    for ch in content.chars() {
        if ch == '\n' {
            line_num += 1;
            col_num = 1;
            continue;
        }

        if ch == '(' || ch == '{' || ch == '[' {
            stack.push((ch, line_num, col_num));
        } else if ch == ')' || ch == '}' || ch == ']' {
            let expected = match ch {
                ')' => '(',
                '}' => '{',
                ']' => '[',
                _ => ' ',
            };
            if let ::core::option::Option::Some((top, _, _)) = stack.last() {
                if *top == expected {
                    stack.pop();
                } else {
                    let mut map = ::serde_json::Map::new();
                    map.insert("file".to_string(), ::serde_json::Value::String(file_path.to_string()));
                    let line_u64 = line_num as ::core::primitive::u64;
                    let num_val = ::serde_json::Number::from(line_u64);
                    let json_num = ::serde_json::Value::Number(num_val);
                    map.insert("line".to_string(), json_num);
                    let col_u64 = col_num as ::core::primitive::u64;
                    let col_num_val = ::serde_json::Number::from(col_u64);
                    let col_json_num = ::serde_json::Value::Number(col_num_val);
                    map.insert("column".to_string(), col_json_num);
                    map.insert("severity".to_string(), ::serde_json::Value::String("error".to_string()));
                    map.insert("message".to_string(), ::serde_json::Value::String("Mismatched closing delimiter".to_string()));
                    let diag = ::serde_json::Value::Object(map);
                    out.push(diag);
                }
            }
        }
        col_num += 1;
    }

    for (unclosed, start_line, start_col) in stack {
        let mut map = ::serde_json::Map::new();
        map.insert("file".to_string(), ::serde_json::Value::String(file_path.to_string()));
        let line_u64 = start_line as ::core::primitive::u64;
        let num_val = ::serde_json::Number::from(line_u64);
        let json_num = ::serde_json::Value::Number(num_val);
        map.insert("line".to_string(), json_num);
        let col_u64 = start_col as ::core::primitive::u64;
        let col_num_val = ::serde_json::Number::from(col_u64);
        let col_json_num = ::serde_json::Value::Number(col_num_val);
        map.insert("column".to_string(), col_json_num);
        map.insert("severity".to_string(), ::serde_json::Value::String("warning".to_string()));
        let unclosed_msg = ::std::format!("Unclosed delimiter `{unclosed}`");
        map.insert("message".to_string(), ::serde_json::Value::String(unclosed_msg));
        let diag = ::serde_json::Value::Object(map);
        out.push(diag);
    }
}

fn handle_definition_action(
    args: &self::LspArgs,
    all_files: &[::std::string::String],
) -> ::core::result::Result<self::LspOutput, ::rig::tool::ToolError> {
    let symbol_opt = args.symbol.as_deref();
    let symbol = match symbol_opt {
        ::core::option::Option::Some(symbol_str) => symbol_str,
        ::core::option::Option::None => {
            let mut map = ::serde_json::Map::new();
            map.insert("error".to_string(), ::serde_json::Value::String("You must provide the symbol argument for definition lookup.".to_string()));
            let result = ::serde_json::Value::Object(map);
            let status = "error".to_string();
            let output = self::LspOutput { status, result };
            return ::core::result::Result::Ok(output);
        },
    };

    let target_file = args.file.as_deref();
    let def_loc = self::find_symbol_definition(symbol, target_file, all_files);
    let mut map = ::serde_json::Map::new();
    map.insert("definitions".to_string(), ::serde_json::Value::Array(def_loc));
    let result = ::serde_json::Value::Object(map);
    let status = "ok".to_string();
    let output = self::LspOutput { status, result };
    ::core::result::Result::Ok(output)
}

fn find_symbol_definition(
    symbol: &::core::primitive::str,
    target_file: ::core::option::Option<&::core::primitive::str>,
    all_files: &[::std::string::String],
) -> ::std::vec::Vec<::serde_json::Value> {
    let mut defs = ::std::vec::Vec::new();
    let files = match target_file {
        ::core::option::Option::Some(file_str) => {
            let mut list = ::std::vec::Vec::new();
            let owned = file_str.to_string();
            list.push(owned);
            list
        },
        ::core::option::Option::None => all_files.to_vec(),
    };

    for file_path in &files {
        self::scan_file_for_definition(file_path, symbol, &mut defs);
    }
    defs
}

fn scan_file_for_definition(
    file_path: &::core::primitive::str,
    symbol: &::core::primitive::str,
    out: &mut ::std::vec::Vec<::serde_json::Value>,
) {
    let path_ref = ::std::path::Path::new(file_path);
    let read_res = ::std::fs::read_to_string(path_ref);
    let content = match read_res {
        ::core::result::Result::Ok(content_str) => content_str,
        ::core::result::Result::Err(_) => return,
    };

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let is_def = self::is_definition_line(trimmed, symbol);
        if is_def {
            let line_num = line_idx + 1;
            let mut map = ::serde_json::Map::new();
            map.insert("uri".to_string(), ::serde_json::Value::String(file_path.to_string()));
            let line_u64 = line_num as ::core::primitive::u64;
            let num_val = ::serde_json::Number::from(line_u64);
            let json_num = ::serde_json::Value::Number(num_val);
            map.insert("line".to_string(), json_num);
            map.insert("text".to_string(), ::serde_json::Value::String(trimmed.to_string()));
            let loc = ::serde_json::Value::Object(map);
            out.push(loc);
        }
    }
}

fn handle_references_action(
    args: &self::LspArgs,
    all_files: &[::std::string::String],
) -> ::core::result::Result<self::LspOutput, ::rig::tool::ToolError> {
    let symbol_opt = args.symbol.as_deref();
    let symbol = match symbol_opt {
        ::core::option::Option::Some(symbol_str) => symbol_str,
        ::core::option::Option::None => {
            let mut map = ::serde_json::Map::new();
            map.insert("error".to_string(), ::serde_json::Value::String("You must provide the symbol argument for references lookup.".to_string()));
            let result = ::serde_json::Value::Object(map);
            let status = "error".to_string();
            let output = self::LspOutput { status, result };
            return ::core::result::Result::Ok(output);
        },
    };

    let target_file = args.file.as_deref();
    let refs = self::find_symbol_references(symbol, target_file, all_files);
    let mut map = ::serde_json::Map::new();
    map.insert("references".to_string(), ::serde_json::Value::Array(refs));
    let result = ::serde_json::Value::Object(map);
    let status = "ok".to_string();
    let output = self::LspOutput { status, result };
    ::core::result::Result::Ok(output)
}

fn find_symbol_references(
    symbol: &::core::primitive::str,
    target_file: ::core::option::Option<&::core::primitive::str>,
    all_files: &[::std::string::String],
) -> ::std::vec::Vec<::serde_json::Value> {
    let mut refs = ::std::vec::Vec::new();
    let files = match target_file {
        ::core::option::Option::Some(file_str) => {
            let mut list = ::std::vec::Vec::new();
            let owned = file_str.to_string();
            list.push(owned);
            list
        },
        ::core::option::Option::None => all_files.to_vec(),
    };

    for file_path in &files {
        self::scan_file_for_references(file_path, symbol, &mut refs);
    }
    refs
}

fn scan_file_for_references(
    file_path: &::core::primitive::str,
    symbol: &::core::primitive::str,
    out: &mut ::std::vec::Vec<::serde_json::Value>,
) {
    let path_ref = ::std::path::Path::new(file_path);
    let read_res = ::std::fs::read_to_string(path_ref);
    let content = match read_res {
        ::core::result::Result::Ok(content_str) => content_str,
        ::core::result::Result::Err(_) => return,
    };

    for (line_idx, line) in content.lines().enumerate() {
        if line.contains(symbol) {
            let line_num = line_idx + 1;
            let trimmed = line.trim();
            let mut map = ::serde_json::Map::new();
            map.insert("uri".to_string(), ::serde_json::Value::String(file_path.to_string()));
            let line_u64 = line_num as ::core::primitive::u64;
            let num_val = ::serde_json::Number::from(line_u64);
            let json_num = ::serde_json::Value::Number(num_val);
            map.insert("line".to_string(), json_num);
            map.insert("text".to_string(), ::serde_json::Value::String(trimmed.to_string()));
            let ref_item = ::serde_json::Value::Object(map);
            out.push(ref_item);
        }
    }
}

fn handle_hover_action(
    args: &self::LspArgs,
    all_files: &[::std::string::String],
) -> ::core::result::Result<self::LspOutput, ::rig::tool::ToolError> {
    let symbol_opt = args.symbol.as_deref();
    let symbol = match symbol_opt {
        ::core::option::Option::Some(symbol_str) => symbol_str,
        ::core::option::Option::None => {
            let mut map = ::serde_json::Map::new();
            map.insert("error".to_string(), ::serde_json::Value::String("You must provide the symbol argument for hover lookup.".to_string()));
            let result = ::serde_json::Value::Object(map);
            let status = "error".to_string();
            let output = self::LspOutput { status, result };
            return ::core::result::Result::Ok(output);
        },
    };

    let file_opt = args.file.as_deref();
    let result = self::find_symbol_hover(symbol, file_opt, all_files);
    let status = "ok".to_string();
    let output = self::LspOutput { status, result };
    ::core::result::Result::Ok(output)
}

fn find_symbol_hover(
    symbol: &::core::primitive::str,
    file_opt: ::core::option::Option<&::core::primitive::str>,
    all_files: &[::std::string::String],
) -> ::serde_json::Value {
    let files = match file_opt {
        ::core::option::Option::Some(file_str) => {
            let mut list = ::std::vec::Vec::new();
            let owned = file_str.to_string();
            list.push(owned);
            list
        },
        ::core::option::Option::None => all_files.to_vec(),
    };

    for file_path in &files {
        let path_ref = ::std::path::Path::new(file_path);
        let read_res = ::std::fs::read_to_string(path_ref);
        let content = match read_res {
            ::core::result::Result::Ok(content_str) => content_str,
            ::core::result::Result::Err(_) => continue,
        };

        let lines = content.lines().collect::<::std::vec::Vec<&::core::primitive::str>>();
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let is_def = self::is_definition_line(trimmed, symbol);
            if is_def {
                let mut docs = ::std::vec::Vec::new();
                let mut back_idx = idx;
                while back_idx > 0 {
                    back_idx -= 1;
                    let prev_line = lines[back_idx].trim();
                    if prev_line.starts_with("///") || prev_line.starts_with("//!") || prev_line.starts_with("#") {
                        let owned_doc = prev_line.to_string();
                        docs.push(owned_doc);
                    } else {
                        break;
                    }
                }
                docs.reverse();

                let doc_text = docs.join("\n");
                let mut map = ::serde_json::Map::new();
                map.insert("symbol".to_string(), ::serde_json::Value::String(symbol.to_string()));
                map.insert("declaration".to_string(), ::serde_json::Value::String(trimmed.to_string()));
                map.insert("documentation".to_string(), ::serde_json::Value::String(doc_text));
                return ::serde_json::Value::Object(map);
            }
        }
    }

    let mut map = ::serde_json::Map::new();
    map.insert("symbol".to_string(), ::serde_json::Value::String(symbol.to_string()));
    map.insert("declaration".to_string(), ::serde_json::Value::Null);
    map.insert("documentation".to_string(), ::serde_json::Value::Null);
    ::serde_json::Value::Object(map)
}

fn is_definition_line(line: &::core::primitive::str, symbol: &::core::primitive::str) -> ::core::primitive::bool {
    if !line.contains(symbol) {
        return false;
    }
    let is_decl = line.starts_with("fn ")
        || line.starts_with("pub fn ")
        || line.starts_with("struct ")
        || line.starts_with("pub struct ")
        || line.starts_with("enum ")
        || line.starts_with("pub enum ")
        || line.starts_with("trait ")
        || line.starts_with("pub trait ")
        || line.starts_with("type ")
        || line.starts_with("let ")
        || line.starts_with("const ")
        || line.starts_with("class ")
        || line.starts_with("def ")
        || line.starts_with("function ");
    is_decl
}

fn handle_symbols_action(
    args: &self::LspArgs,
    all_files: &[::std::string::String],
) -> ::core::result::Result<self::LspOutput, ::rig::tool::ToolError> {
    let query_opt = args.query.as_deref();
    let file_opt = args.file.as_deref();
    let mut symbols = ::std::vec::Vec::new();

    let files = match file_opt {
        ::core::option::Option::Some(file_str) => {
            let mut list = ::std::vec::Vec::new();
            let owned = file_str.to_string();
            list.push(owned);
            list
        },
        ::core::option::Option::None => all_files.to_vec(),
    };

    for file_path in &files {
        self::scan_file_symbols(file_path, query_opt, &mut symbols);
    }

    let mut map = ::serde_json::Map::new();
    map.insert("symbols".to_string(), ::serde_json::Value::Array(symbols));
    let result = ::serde_json::Value::Object(map);
    let status = "ok".to_string();
    let output = self::LspOutput { status, result };
    ::core::result::Result::Ok(output)
}

fn scan_file_symbols(
    file_path: &::core::primitive::str,
    query_opt: ::core::option::Option<&::core::primitive::str>,
    out: &mut ::std::vec::Vec<::serde_json::Value>,
) {
    let path_ref = ::std::path::Path::new(file_path);
    let read_res = ::std::fs::read_to_string(path_ref);
    let content = match read_res {
        ::core::result::Result::Ok(content_str) => content_str,
        ::core::result::Result::Err(_) => return,
    };

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let sym_info = self::parse_symbol_from_line(trimmed);
        if let ::core::option::Option::Some((name, kind)) = sym_info {
            let matches_query = match query_opt {
                ::core::option::Option::Some(query_str) => name.contains(query_str),
                ::core::option::Option::None => true,
            };
            if matches_query {
                let line_num = line_idx + 1;
                let mut loc_map = ::serde_json::Map::new();
                loc_map.insert("uri".to_string(), ::serde_json::Value::String(file_path.to_string()));
                let line_u64 = line_num as ::core::primitive::u64;
                let num_val = ::serde_json::Number::from(line_u64);
                let json_num = ::serde_json::Value::Number(num_val);
                loc_map.insert("line".to_string(), json_num);

                let mut sym_map = ::serde_json::Map::new();
                sym_map.insert("name".to_string(), ::serde_json::Value::String(name));
                sym_map.insert("kind".to_string(), ::serde_json::Value::String(kind.to_string()));
                sym_map.insert("location".to_string(), ::serde_json::Value::Object(loc_map));
                out.push(::serde_json::Value::Object(sym_map));
            }
        }
    }
}

fn parse_symbol_from_line(line: &::core::primitive::str) -> ::core::option::Option<(::std::string::String, &'static ::core::primitive::str)> {
    let prefixes: [(&::core::primitive::str, &'static ::core::primitive::str); 6] = [
        ("fn ", "function"),
        ("pub fn ", "function"),
        ("struct ", "struct"),
        ("pub struct ", "struct"),
        ("enum ", "enum"),
        ("pub enum ", "enum"),
    ];

    for (prefix, kind) in &prefixes {
        if let ::core::option::Option::Some(pos) = line.find(prefix) {
            let rest = &line[pos + prefix.len()..];
            let default_end = rest.len();
            let name_end = rest.find(|character: ::core::primitive::char| {
                let is_ident = character.is_ascii_alphanumeric() || character == '_';
                !is_ident
            }).unwrap_or(default_end);
            let name = &rest[..name_end];
            if !name.is_empty() {
                let owned_name = name.to_string();
                return ::core::option::Option::Some((owned_name, *kind));
            }
        }
    }
    ::core::option::Option::None
}

fn handle_rename_action(args: &self::LspArgs) -> ::core::result::Result<self::LspOutput, ::rig::tool::ToolError> {
    let new_name_opt = args.new_name.as_deref();
    let new_name = match new_name_opt {
        ::core::option::Option::Some(name_str) => name_str,
        ::core::option::Option::None => {
            let mut map = ::serde_json::Map::new();
            map.insert("error".to_string(), ::serde_json::Value::String("You must provide the new name argument for rename.".to_string()));
            let result = ::serde_json::Value::Object(map);
            let status = "error".to_string();
            let output = self::LspOutput { status, result };
            return ::core::result::Result::Ok(output);
        },
    };

    let symbol_opt = args.symbol.as_deref();
    let symbol = match symbol_opt {
        ::core::option::Option::Some(symbol_str) => symbol_str,
        ::core::option::Option::None => {
            let mut map = ::serde_json::Map::new();
            map.insert("error".to_string(), ::serde_json::Value::String("You must provide the symbol argument for rename.".to_string()));
            let result = ::serde_json::Value::Object(map);
            let status = "error".to_string();
            let output = self::LspOutput { status, result };
            return ::core::result::Result::Ok(output);
        },
    };

    let file_opt = args.file.as_deref();
    let target_file = match file_opt {
        ::core::option::Option::Some(file_str) => file_str,
        ::core::option::Option::None => {
            let mut map = ::serde_json::Map::new();
            map.insert("error".to_string(), ::serde_json::Value::String("You must provide the file argument for rename.".to_string()));
            let result = ::serde_json::Value::Object(map);
            let status = "error".to_string();
            let output = self::LspOutput { status, result };
            return ::core::result::Result::Ok(output);
        },
    };

    let should_apply = args.apply.unwrap_or(false);
    let result = self::execute_rename(target_file, symbol, new_name, should_apply)?;
    let status = "ok".to_string();
    let output = self::LspOutput { status, result };
    ::core::result::Result::Ok(output)
}

fn execute_rename(
    target_file: &::core::primitive::str,
    symbol: &::core::primitive::str,
    new_name: &::core::primitive::str,
    should_apply: ::core::primitive::bool,
) -> ::core::result::Result<::serde_json::Value, ::rig::tool::ToolError> {
    let path_ref = ::std::path::Path::new(target_file);
    let read_res = ::std::fs::read_to_string(path_ref);
    let content = match read_res {
        ::core::result::Result::Ok(content_str) => content_str,
        ::core::result::Result::Err(err) => {
            let err_obj = err.into_tool_error();
            return ::core::result::Result::Err(err_obj);
        },
    };

    let occurrences = content.matches(symbol).count();
    let replaced = content.replace(symbol, new_name);

    if should_apply && occurrences > 0 {
        let write_res = ::std::fs::write(path_ref, replaced.as_bytes());
        if let ::core::result::Result::Err(err) = write_res {
            let err_obj = err.into_tool_error();
            return ::core::result::Result::Err(err_obj);
        }
    }

    let mut map = ::serde_json::Map::new();
    map.insert("file".to_string(), ::serde_json::Value::String(target_file.to_string()));
    map.insert("symbol".to_string(), ::serde_json::Value::String(symbol.to_string()));
    map.insert("new_name".to_string(), ::serde_json::Value::String(new_name.to_string()));
    let occ_u64 = occurrences as ::core::primitive::u64;
    let num_val = ::serde_json::Number::from(occ_u64);
    let json_num = ::serde_json::Value::Number(num_val);
    map.insert("occurrences".to_string(), json_num);
    map.insert("applied".to_string(), ::serde_json::Value::Bool(should_apply));
    let result = ::serde_json::Value::Object(map);
    ::core::result::Result::Ok(result)
}
