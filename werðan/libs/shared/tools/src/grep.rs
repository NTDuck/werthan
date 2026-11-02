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
    ::core::result::Result::Ok(self::GrepOutput { matches })
}

fn build_regex(
    pattern: &::core::primitive::str,
    case: ::core::option::Option<::core::primitive::bool>,
) -> ::core::result::Result<::regex::Regex, ::rig::tool::ToolError> {
    let is_case_insensitive = !case.unwrap_or(true);
    let mut builder = ::regex::RegexBuilder::new(pattern);
    builder.case_insensitive(is_case_insensitive);
    match builder.build() {
        ::core::result::Result::Ok(compiled_regex) => ::core::result::Result::Ok(compiled_regex),
        ::core::result::Result::Err(regex_error) => {
            let error_message = ::std::format!("Invalid regex pattern: {regex_error}");
            let tool_error = error_message.into_tool_error();
            ::core::result::Result::Err(tool_error)
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
    if !root.exists() {
        let root_display = root.display();
        let error_message = ::std::format!("Path not found: {root_display}");
        let tool_error = error_message.into_tool_error();
        return ::core::result::Result::Err(tool_error);
    }

    if root.is_file() {
        return ::core::result::Result::Ok(::std::vec![root.to_path_buf()]);
    }

    let mut builder = ::ignore::WalkBuilder::new(root);
    builder.hidden(true);
    builder.git_ignore(respect_gitignore);
    builder.git_global(respect_gitignore);
    builder.git_exclude(respect_gitignore);

    let allows_target = root.starts_with("target");
    let allows_node_modules = root.starts_with("node_modules");
    builder.filter_entry(move |entry| {
        let file_name = entry.file_name().to_string_lossy();
        file_name != ".git"
            && (allows_node_modules || file_name != "node_modules")
            && (allows_target || file_name != "target")
    });

    let mut files = builder
        .build()
        .filter_map(::core::result::Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .is_some_and(|file_type| file_type.is_file())
        })
        .map(::ignore::DirEntry::into_path)
        .collect::<::std::vec::Vec<::std::path::PathBuf>>();

    files.sort();
    ::core::result::Result::Ok(files)
}

fn search_files(
    files: &[::std::path::PathBuf],
    regex: &::regex::Regex,
    skip: ::core::primitive::usize,
) -> ::std::vec::Vec<self::GrepMatch> {
    files
        .iter()
        .flat_map(|file_path| {
            self::search_single_file(file_path, regex)
        })
        .skip(skip)
        .take(500)
        .collect::<::std::vec::Vec<self::GrepMatch>>()
}

fn search_single_file(
    path: &::std::path::Path,
    regex: &::regex::Regex,
) -> ::std::vec::Vec<self::GrepMatch> {
    let content = match ::std::fs::read_to_string(path) {
        ::core::result::Result::Ok(content_str) => content_str,
        ::core::result::Result::Err(_) => return ::std::vec::Vec::new(),
    };
    let normalized_path = self::normalize_path_string(path);
    content
        .lines()
        .enumerate()
        .filter_map(|(line_idx, line)| {
            if regex.is_match(line) {
                let path = normalized_path.clone();
                let line_number = line_idx + 1;
                let line = line.to_string();
                ::core::option::Option::Some(self::GrepMatch {
                    path,
                    line_number,
                    line,
                })
            } else {
                ::core::option::Option::None
            }
        })
        .collect::<::std::vec::Vec<self::GrepMatch>>()
}

fn normalize_path_string(path: &::std::path::Path) -> ::std::string::String {
    let forward_slashes = path.to_string_lossy().replace('\\', "/");
    let trimmed = forward_slashes.trim();
    trimmed
        .strip_prefix("./")
        .unwrap_or(trimmed)
        .to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_normalize_path_string() {
        let path_prefixed = ::std::path::Path::new("./foo/bar.rs");
        let normalized = super::normalize_path_string(path_prefixed);
        ::std::assert_eq!(normalized, "foo/bar.rs");

        let path_clean = ::std::path::Path::new("foo/bar.rs");
        let normalized_clean = super::normalize_path_string(path_clean);
        ::std::assert_eq!(normalized_clean, "foo/bar.rs");
    }

    #[test]
    fn test_build_regex() {
        let valid_case = super::build_regex("abc", ::core::option::Option::Some(true));
        ::std::assert!(valid_case.is_ok());

        let case_insensitive = super::build_regex("abc", ::core::option::Option::Some(false));
        ::std::assert!(case_insensitive.is_ok());

        let invalid = super::build_regex("[a-", ::core::option::Option::None);
        ::std::assert!(invalid.is_err());
    }

    #[test]
    fn test_search_files_functional() {
        let temp_dir = ::std::env::temp_dir();
        let unique_timestamp = ::std::time::SystemTime::now()
            .duration_since(::std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let root = temp_dir.join(::std::format!("werthan-grep-test-{unique_timestamp}"));
        ::std::fs::create_dir_all(&root).expect("create test root");

        let file_one = root.join("alpha.txt");
        let file_two = root.join("beta.txt");
        ::std::fs::write(&file_one, "match line 1\nskip me\nmatch line 2\n").expect("write alpha");
        ::std::fs::write(&file_two, "match line 3\nmatch line 4\n").expect("write beta");

        let files = ::std::vec![file_one, file_two];
        let regex = super::build_regex("match", ::core::option::Option::None).expect("valid regex");

        let matches_all = super::search_files(&files, &regex, 0);
        ::std::assert_eq!(matches_all.len(), 4);
        ::std::assert_eq!(matches_all[0].line, "match line 1");
        ::std::assert_eq!(matches_all[0].line_number, 1);
        ::std::assert_eq!(matches_all[1].line, "match line 2");
        ::std::assert_eq!(matches_all[1].line_number, 3);
        ::std::assert_eq!(matches_all[2].line, "match line 3");
        ::std::assert_eq!(matches_all[2].line_number, 1);
        ::std::assert_eq!(matches_all[3].line, "match line 4");
        ::std::assert_eq!(matches_all[3].line_number, 2);

        let matches_skipped = super::search_files(&files, &regex, 2);
        ::std::assert_eq!(matches_skipped.len(), 2);
        ::std::assert_eq!(matches_skipped[0].line, "match line 3");
        ::std::assert_eq!(matches_skipped[1].line, "match line 4");

        let _ignored = ::std::fs::remove_dir_all(&root);
    }
}
