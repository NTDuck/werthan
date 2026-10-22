//! Grep search tool implementation.
//! Reference: https://github.com/can1357/oh-my-pi/blob/d5cd750e41fede277f9eb933455da92a0738915e/packages/coding-agent/src/tools/grep.ts

use crate::DisplayExt;

/// Match entry discovered by grep tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
pub struct GrepMatch {
    pub path: ::std::string::String,
    pub line_number: ::core::primitive::usize,
    pub line: ::std::string::String,
}

/// Arguments for grep tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct GrepArgs {
    pub pattern: ::std::string::String,
    pub path: ::core::option::Option<::std::string::String>,
    pub case: ::core::option::Option<::core::primitive::bool>,
    pub gitignore: ::core::option::Option<::core::primitive::bool>,
    pub skip: ::core::option::Option<::core::primitive::usize>,
}

/// Output from grep tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct GrepOutput {
    pub matches: ::std::vec::Vec<self::GrepMatch>,
}

/// Grep search tool searching text patterns in files.
#[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::default::Default)]
pub struct GrepTool;

impl ::rig::tool::Tool for self::GrepTool {
    const NAME: &'static ::core::primitive::str = "grep";

    type Error = ::rig::tool::ToolError;
    type Args = self::GrepArgs;
    type Output = self::GrepOutput;

    async fn definition(&self, _prompt: ::std::string::String) -> ::rig::completion::ToolDefinition {
        let parameters = ::serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regular expression pattern to search for."
                },
                "path": {
                    "type": "string",
                    "description": "File path or directory path to search in."
                },
                "case": {
                    "type": "boolean",
                    "description": "Case-sensitive matching when true. Defaults to true."
                },
                "gitignore": {
                    "type": "boolean",
                    "description": "Respect gitignore rules when true. Defaults to true."
                },
                "skip": {
                    "type": "integer",
                    "description": "Number of matches to skip for pagination."
                }
            },
            "required": ["pattern"]
        });
        let name = Self::NAME.to_string();
        let description = "Search regular expression patterns in files and directories.".to_string();
        ::rig::completion::ToolDefinition {
            name,
            description,
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        self::execute_grep(args)
    }
}

fn execute_grep(args: self::GrepArgs) -> ::core::result::Result<self::GrepOutput, ::rig::tool::ToolError> {
    let regex = self::build_regex(&args.pattern, args.case)?;
    let root_path = self::resolve_root_path(&args.path);
    let respect_gitignore = args.gitignore.unwrap_or(true);
    let files = self::collect_search_files(&root_path, respect_gitignore)?;
    let skip_count = args.skip.unwrap_or(0);
    let matches = self::search_files(&files, &regex, skip_count);
    let output = self::GrepOutput { matches };
    ::core::result::Result::Ok(output)
}

fn build_regex(
    pattern: &::core::primitive::str,
    case: ::core::option::Option<::core::primitive::bool>,
) -> ::core::result::Result<::regex::Regex, ::rig::tool::ToolError> {
    let case_sensitive = case.unwrap_or(true);
    let mut builder = ::regex::RegexBuilder::new(pattern);
    let is_case_insensitive = !case_sensitive;
    builder.case_insensitive(is_case_insensitive);
    let build_res = builder.build();
    match build_res {
        ::core::result::Result::Ok(re) => ::core::result::Result::Ok(re),
        ::core::result::Result::Err(err) => {
            let err_msg = ::std::format!("Invalid regex pattern: {err}");
            let err_obj = err_msg.into_tool_error();
            ::core::result::Result::Err(err_obj)
        },
    }
}

fn resolve_root_path(path_opt: &::core::option::Option<::std::string::String>) -> ::std::path::PathBuf {
    match path_opt {
        ::core::option::Option::Some(path_str) => ::std::path::PathBuf::from(path_str),
        ::core::option::Option::None => ::std::path::PathBuf::from("."),
    }
}

fn collect_search_files(
    root: &::std::path::Path,
    respect_gitignore: ::core::primitive::bool,
) -> ::core::result::Result<::std::vec::Vec<::std::path::PathBuf>, ::rig::tool::ToolError> {
    let exists = root.exists();
    if !exists {
        let root_display = root.display();
        let err_msg = ::std::format!("Path not found: {root_display}");
        let err_obj = err_msg.into_tool_error();
        return ::core::result::Result::Err(err_obj);
    }

    let is_file = root.is_file();
    if is_file {
        let single_path = root.to_path_buf();
        let files = ::std::vec![single_path];
        return ::core::result::Result::Ok(files);
    }

    let mut builder = ::ignore::WalkBuilder::new(root);
    builder.hidden(true);
    builder.git_ignore(respect_gitignore);
    builder.git_global(respect_gitignore);
    builder.git_exclude(respect_gitignore);

    let allows_target = root.starts_with("target");
    let allows_node_modules = root.starts_with("node_modules");
    builder.filter_entry(move |entry| {
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        if name_str == ".git" {
            return false;
        }
        if !allows_node_modules && name_str == "node_modules" {
            return false;
        }
        if !allows_target && name_str == "target" {
            return false;
        }
        true
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
            let path_buf = entry.into_path();
            files.push(path_buf);
        }
    }

    files.sort();
    ::core::result::Result::Ok(files)
}

fn search_files(
    files: &[::std::path::PathBuf],
    regex: &::regex::Regex,
    skip: ::core::primitive::usize,
) -> ::std::vec::Vec<self::GrepMatch> {
    let mut matches = ::std::vec::Vec::new();
    let mut skipped = 0;

    for file in files {
        if matches.len() >= 500 {
            break;
        }
        self::search_single_file(file, regex, &mut skipped, skip, &mut matches);
    }

    matches
}

fn search_single_file(
    path: &::std::path::Path,
    regex: &::regex::Regex,
    skipped: &mut ::core::primitive::usize,
    skip: ::core::primitive::usize,
    matches: &mut ::std::vec::Vec<self::GrepMatch>,
) {
    let read_res = ::std::fs::read_to_string(path);
    let content = match read_res {
        ::core::result::Result::Ok(content_str) => content_str,
        ::core::result::Result::Err(_) => return,
    };

    for (idx, line) in content.lines().enumerate() {
        if matches.len() >= 500 {
            break;
        }
        let matches_pattern = regex.is_match(line);
        if matches_pattern {
            if *skipped < skip {
                *skipped += 1;
                continue;
            }
            let line_number = idx + 1;
            let path = self::normalize_path_string(path);
            let line = line.to_string();
            let item = self::GrepMatch {
                path,
                line_number,
                line,
            };
            matches.push(item);
        }
    }
}

fn normalize_path_string(path: &::std::path::Path) -> ::std::string::String {
    let raw = path.to_string_lossy();
    let forward_slashes = raw.replace('\\', "/");
    let trimmed = forward_slashes.trim();
    let stripped = trimmed.strip_prefix("./");
    match stripped {
        ::core::option::Option::Some(unprefixed) => unprefixed.to_string(),
        ::core::option::Option::None => trimmed.to_string(),
    }
}
