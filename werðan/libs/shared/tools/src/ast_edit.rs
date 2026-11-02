// Pinned commit: d5cd750e41fede277f9eb933455da92a0738915e
// Reference: https://github.com/can1357/oh-my-pi/blob/d5cd750e41fede277f9eb933455da92a0738915e/packages/coding-agent/src/tools/ast-edit.ts

use crate::DisplayExt;
use crate::ast_lang::{SupportedLanguage, infer_file_language};

/// Single rewrite operation for AST editing.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
pub struct AstEditOp {
    pub pat: ::std::string::String,
    pub out: ::std::string::String,
}

/// Arguments for ast-edit tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct AstEditArgs {
    pub ops: ::std::vec::Vec<self::AstEditOp>,
    pub paths: ::std::vec::Vec<::std::string::String>,
}

/// Output from ast-edit tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct AstEditOutput {
    pub modified_files: ::std::vec::Vec<::std::string::String>,
}

/// AST edit tool for structural code rewrites.
#[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::default::Default)]
pub struct AstEditTool;

impl ::rig::tool::Tool for self::AstEditTool {
    const NAME: &'static ::core::primitive::str = "ast_edit";

    type Error = ::rig::tool::ToolError;
    type Args = self::AstEditArgs;
    type Output = self::AstEditOutput;

    async fn definition(&self, _prompt: ::std::string::String) -> ::rig::completion::ToolDefinition {
        let parameters = ::serde_json::json!({
            "type": "object",
            "properties": {
                "ops": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "pat": {
                                "type": "string",
                                "description": "AST pattern to match."
                            },
                            "out": {
                                "type": "string",
                                "description": "Replacement template text."
                            }
                        },
                        "required": ["pat", "out"]
                    },
                    "description": "List of structural rewrite operations."
                },
                "paths": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "File or directory paths to rewrite."
                }
            },
            "required": ["ops", "paths"]
        });
        let name = Self::NAME.to_string();
        let description = "Perform structural syntax replacements across files.".to_string();
        ::rig::completion::ToolDefinition {
            name,
            description,
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        self::execute_ast_edit(&args)
    }
}

fn execute_ast_edit(args: &self::AstEditArgs) -> ::core::result::Result<self::AstEditOutput, ::rig::tool::ToolError> {
    let target_files = collect_target_files(&args.paths);

    let mut modified_files = ::std::vec::Vec::new();
    for file_path in &target_files {
        let apply_result = self::apply_ops_to_file(file_path, &args.ops);
        match apply_result {
            ::core::result::Result::Ok(changed) => {
                if changed {
                    let owned_path = file_path.to_string();
                    modified_files.push(owned_path);
                }
            },
            ::core::result::Result::Err(err) => {
                let err_obj = err.into_tool_error();
                return ::core::result::Result::Err(err_obj);
            },
        }
    }

    modified_files.sort();
    let output = self::AstEditOutput { modified_files };
    ::core::result::Result::Ok(output)
}

fn collect_target_files(paths: &[::std::string::String]) -> ::std::vec::Vec<::std::string::String> {
    let mut files = ::std::vec::Vec::new();
    for path_str in paths {
        let path_obj = ::std::path::Path::new(path_str);
        let is_file = path_obj.is_file();
        if is_file {
            let owned_path = path_str.clone();
            files.push(owned_path);
            continue;
        }

        let mut builder = ::ignore::WalkBuilder::new(path_obj);
        builder.hidden(true);
        builder.git_ignore(true);
        builder.filter_entry(|entry| {
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            name_str != ".git" && name_str != "node_modules" && name_str != "target" && name_str != ".idea"
        });
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
    }
    files.sort();
    files.dedup();
    files
}

/// Applies every rewrite operation to one file with real ast-grep matching.
/// The tool splices each match range with the template text, substituting
/// `$NAME` metavariables from the match environment. The function reports
/// whether the file content changed.
fn apply_ops_to_file(
    file_path: &::core::primitive::str,
    ops: &[self::AstEditOp],
) -> ::core::result::Result<::core::primitive::bool, ::std::string::String> {
    let path_ref = ::std::path::Path::new(file_path);
    let file_name = path_ref
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    let language_opt = infer_file_language(file_name.as_str());
    let language = match language_opt {
        ::core::option::Option::Some(lang_value) => lang_value,
        ::core::option::Option::None => return ::core::result::Result::Ok(false),
    };

    let read_res = ::std::fs::read_to_string(path_ref);
    let original = match read_res {
        ::core::result::Result::Ok(content_str) => content_str,
        ::core::result::Result::Err(_) => return ::core::result::Result::Ok(false),
    };

    let mut current = ::core::clone::Clone::clone(&original);
    for op in ops {
        let rewritten = self::apply_single_op(current.as_str(), language, op)?;
        current = rewritten;
    }

    if current != original {
        let write_res = ::std::fs::write(path_ref, current.as_bytes());
        match write_res {
            ::core::result::Result::Ok(()) => ::core::result::Result::Ok(true),
            ::core::result::Result::Err(write_err) => {
                let err_text = write_err.to_string();
                ::core::result::Result::Err(err_text)
            },
        }
    } else {
        ::core::result::Result::Ok(false)
    }
}


/// Compiles the op pattern, collects match ranges with resolved replacement
/// text, and splices them into the source back to front so earlier offsets
/// stay valid.
fn apply_single_op(
    source: &::core::primitive::str,
    language: self::SupportedLanguage,
    op: &self::AstEditOp,
) -> ::core::result::Result<::std::string::String, ::std::string::String> {
    let pattern_res = ::ast_grep_core::Pattern::try_new(op.pat.as_str(), language);
    let ast_pattern = match pattern_res {
        ::core::result::Result::Ok(pattern_val) => pattern_val,
        ::core::result::Result::Err(pattern_err) => {
            let err_text = ::std::format!("invalid ast-grep pattern: {pattern_err}");
            return ::core::result::Result::Err(err_text);
        },
    };

    let grep_res = ::ast_grep_core::tree_sitter::StrDoc::try_new(source, language);
    let doc = match grep_res {
        ::core::result::Result::Ok(doc_val) => doc_val,
        ::core::result::Result::Err(_) => return ::core::result::Result::Ok(source.to_string()),
    };
    let grep = ::ast_grep_core::AstGrep::doc(doc);
    let root_node = grep.root();

    let mut replacements = ::std::vec::Vec::new();
    for node_match in root_node.find_all(ast_pattern) {
        let replacement_pattern = op.out.as_str();
        let edit = node_match.replace_by(replacement_pattern);
        let start_offset = edit.position;
        let end_offset = start_offset + edit.deleted_length;
        let text_bytes = edit.inserted_text;
        let utf8_res = ::std::string::String::from_utf8(text_bytes);
        let text = utf8_res.unwrap_or_default();
        replacements.push((start_offset, end_offset, text));
    }

    if replacements.is_empty() {
        return ::core::result::Result::Ok(source.to_string());
    }
    replacements.sort();
    replacements.dedup_by(|left, right| left.0 == right.0 && left.1 == right.1);

    let mut rewritten = ::std::string::String::with_capacity(source.len());
    let mut last_end = 0;
    for (start_offset, end_offset, text) in replacements {
        if start_offset < last_end {
            continue;
        }
        rewritten.push_str(&source[last_end..start_offset]);
        rewritten.push_str(text.as_str());
        last_end = end_offset;
    }
    rewritten.push_str(&source[last_end..]);
    ::core::result::Result::Ok(rewritten)
}

