// Pinned commit: d5cd750e41fede277f9eb933455da92a0738915e
// Reference: https://github.com/can1357/oh-my-pi/blob/d5cd750e41fede277f9eb933455da92a0738915e/packages/coding-agent/src/tools/ast-grep.ts

use crate::DisplayExt;
use crate::ast_lang::{SupportedLanguage, matches_language, resolve_language};

/// Match record from ast-grep search.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
pub struct AstMatch {
    pub path: ::std::string::String,
    pub line_number: ::core::primitive::usize,
    pub matched_text: ::std::string::String,
}

/// Arguments for ast-grep tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct AstGrepArgs {
    pub pat: ::std::string::String,
    pub path: ::core::option::Option<::std::string::String>,
    pub lang: ::core::option::Option<::std::string::String>,
    pub skip: ::core::option::Option<::core::primitive::usize>,
}

/// Output from ast-grep tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct AstGrepOutput {
    pub matches: ::std::vec::Vec<self::AstMatch>,
}

/// AST grep tool for structural syntax pattern searches.
#[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::default::Default)]
pub struct AstGrepTool;

impl ::rig::tool::Tool for self::AstGrepTool {
    const NAME: &'static ::core::primitive::str = "ast_grep";

    type Error = ::rig::tool::ToolError;
    type Args = self::AstGrepArgs;
    type Output = self::AstGrepOutput;

    async fn definition(&self, _prompt: ::std::string::String) -> ::rig::completion::ToolDefinition {
        let parameters = ::serde_json::json!({
            "type": "object",
            "properties": {
                "pat": {
                    "type": "string",
                    "description": "AST pattern with metavariables to search."
                },
                "path": {
                    "type": "string",
                    "description": "File or directory path to search. Defaults to current directory."
                },
                "lang": {
                    "type": "string",
                    "description": "Language filter identifier such as rust or typescript."
                },
                "skip": {
                    "type": "integer",
                    "description": "Number of matches to skip from results."
                }
            },
            "required": ["pat"]
        });
        let name = Self::NAME.to_string();
        let description = "Search code syntax patterns across files with ast-grep metavariables.".to_string();
        ::rig::completion::ToolDefinition {
            name,
            description,
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        self::execute_ast_grep(&args)
    }
}

fn execute_ast_grep(args: &self::AstGrepArgs) -> ::core::result::Result<self::AstGrepOutput, ::rig::tool::ToolError> {
    let search_path = match &args.path {
        ::core::option::Option::Some(path_str) => path_str.as_str(),
        ::core::option::Option::None => ".",
    };

    let lang_opt = args.lang.as_deref();
    let files = collect_files(search_path, lang_opt);
    let mut all_matches = ::std::vec::Vec::new();
    for file_path in &files {
        let scan_result = self::scan_file_matches(file_path, lang_opt, &args.pat);
        match scan_result {
            ::core::result::Result::Ok(file_matches) => {
                all_matches.extend(file_matches);
            },
            ::core::result::Result::Err(err) => {
                let err_obj = err.into_tool_error();
                return ::core::result::Result::Err(err_obj);
            },
        }
    }

    let skip_count = args.skip.unwrap_or(0);
    let matches = all_matches
        .into_iter()
        .skip(skip_count)
        .collect::<::std::vec::Vec<self::AstMatch>>();

    let output = self::AstGrepOutput {
        matches,
    };
    ::core::result::Result::Ok(output)
}

fn collect_files(
    root: &::core::primitive::str,
    lang_filter: ::core::option::Option<&::core::primitive::str>,
) -> ::std::vec::Vec<::std::string::String> {
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
        if !is_file_entry {
            continue;
        }
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        let matches_lang = matches_language(&name_str, lang_filter);
        if matches_lang {
            let path_buf = entry.into_path();
            let lossy_str = path_buf.to_string_lossy();
            let owned_str = lossy_str.to_string();
            files.push(owned_str);
        }
    }

    files.sort();
    files
}

/// Parses the file with the resolved language and collects structural matches.
/// The scanner skips files it cannot read so mixed trees stay searchable.
fn scan_file_matches(
    file_path: &::core::primitive::str,
    lang_filter: ::core::option::Option<&::core::primitive::str>,
    pattern: &::core::primitive::str,
) -> ::core::result::Result<::std::vec::Vec<self::AstMatch>, ::std::string::String> {
    let language_res = match lang_filter {
        ::core::option::Option::Some(filter_name) => resolve_language(filter_name),
        ::core::option::Option::None => ::core::result::Result::Ok(self::SupportedLanguage::Rust),
    };
    let language = language_res?;
    let empty_vec = ::std::vec::Vec::new();
    let path_ref = ::std::path::Path::new(file_path);
    let read_res = ::std::fs::read_to_string(path_ref);
    let content = match read_res {
        ::core::result::Result::Ok(content_str) => content_str,
        ::core::result::Result::Err(_) => return ::core::result::Result::Ok(empty_vec),
    };

    let pattern_res = ::ast_grep_core::Pattern::try_new(pattern, language);
    let ast_pattern = match pattern_res {
        ::core::result::Result::Ok(pattern_val) => pattern_val,
        ::core::result::Result::Err(pattern_err) => {
            let err_text = ::std::format!("invalid ast-grep pattern: {pattern_err}");
            return ::core::result::Result::Err(err_text);
        },
    };

    let grep_res = ::ast_grep_core::tree_sitter::StrDoc::try_new(content.as_str(), language);
    let doc = match grep_res {
        ::core::result::Result::Ok(doc_val) => doc_val,
        ::core::result::Result::Err(_) => return ::core::result::Result::Ok(empty_vec),
    };
    let grep = ::ast_grep_core::AstGrep::doc(doc);
    let root_node = grep.root();
    let node_matches = root_node.find_all(ast_pattern);

    let mut out = ::std::vec::Vec::new();
    for node_match in node_matches {
        let node_range = node_match.range();
        let start_offset = node_range.start;
        let line_number = self::compute_line_number(content.as_str(), start_offset);
        let matched_slice = node_match.text();
        let trimmed_text = matched_slice.trim();
        let matched_text = trimmed_text.to_string();
        let item = self::AstMatch {
            path: ::std::string::ToString::to_string(file_path),
            line_number,
            matched_text,
        };
        out.push(item);
    }
    ::core::result::Result::Ok(out)
}

fn compute_line_number(text: &::core::primitive::str, offset: ::core::primitive::usize) -> ::core::primitive::usize {
    let slice = &text[..offset];
    let newlines = slice.chars().filter(|character| *character == '\n').count();
    newlines + 1
}
