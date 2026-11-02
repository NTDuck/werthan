//! Glob search tool implementation.
//! Reference: https://github.com/can1357/oh-my-pi/blob/d5cd750e41fede277f9eb933455da92a0738915e/packages/coding-agent/src/tools/glob.ts

use crate::DisplayExt;

/// Input arguments for glob tool.
#[derive(::core::fmt::Debug, ::serde::Deserialize)]
pub struct GlobArgs {
    pub path: ::core::option::Option<::std::string::String>,
    pub hidden: ::core::option::Option<::core::primitive::bool>,
    pub gitignore: ::core::option::Option<::core::primitive::bool>,
    pub limit: ::core::option::Option<::core::primitive::usize>,
}

/// Output results for glob tool.
#[derive(::core::fmt::Debug, ::serde::Serialize)]
pub struct GlobOutput {
    pub matches: ::std::vec::Vec<::std::string::String>,
}

/// Tool for finding filesystem paths matching glob patterns.
#[derive(::core::clone::Clone, ::core::default::Default)]
pub struct GlobTool;

impl ::rig::tool::Tool for self::GlobTool {
    const NAME: &'static ::core::primitive::str = "glob";

    type Error = ::rig::tool::ToolError;
    type Args = self::GlobArgs;
    type Output = self::GlobOutput;

    async fn definition(&self, _prompt: ::std::string::String) -> ::rig::completion::ToolDefinition {
        let parameters = ::serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Base directory or glob pattern to search."
                },
                "hidden": {
                    "type": "boolean",
                    "description": "Include hidden files when set to true. Defaults to false."
                },
                "gitignore": {
                    "type": "boolean",
                    "description": "Respect gitignore rules when set to true. Defaults to true."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of paths to return. Defaults to 100."
                }
            }
        });
        let name = Self::NAME.to_string();
        let description = "Find files and directories matching path and glob patterns.".to_string();
        ::rig::completion::ToolDefinition {
            name,
            description,
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        self::execute_glob(args)
    }
}

fn execute_glob(args: self::GlobArgs) -> ::core::result::Result<self::GlobOutput, ::rig::tool::ToolError> {
    let input_path = match args.path {
        ::core::option::Option::Some(path_str) => path_str,
        ::core::option::Option::None => ".".to_string(),
    };
    let include_hidden = args.hidden.unwrap_or(false);
    let respect_gitignore = args.gitignore.unwrap_or(true);
    let limit = args.limit.unwrap_or_else(|| {
        let env_val = ::core::env!("TOOL_GLOB_LIMIT");
        let parsed = env_val
            .parse::<::core::primitive::usize>()
            .expect("TOOL_GLOB_LIMIT must be a valid usize");
        parsed
    });

    let (base_dir, glob_pattern) = self::resolve_search_spec(&input_path);
    let matcher = self::build_glob_matcher(&glob_pattern)?;
    let matches = self::collect_glob_matches(&base_dir, &matcher, include_hidden, respect_gitignore, limit);
    let output = self::GlobOutput { matches };
    ::core::result::Result::Ok(output)
}

fn resolve_search_spec(raw_path: &::core::primitive::str) -> (::std::path::PathBuf, ::std::string::String) {
    let normalized = raw_path.replace('\\', "/");
    let trimmed = normalized.trim();
    if trimmed.is_empty() || trimmed == "." {
        let base_dir = ::std::path::PathBuf::from(".");
        let pattern = "**/*".to_string();
        return (base_dir, pattern);
    }

    let has_wildcard = trimmed.contains('*') || trimmed.contains('?') || trimmed.contains('[');
    if !has_wildcard {
        let base_dir = ::std::path::PathBuf::from(trimmed);
        let pattern = "**/*".to_string();
        return (base_dir, pattern);
    }

    let is_bare_pattern = !trimmed.contains('/');
    if is_bare_pattern {
        let base_dir = ::std::path::PathBuf::from(".");
        let pattern = ::std::format!("**/{trimmed}");
        return (base_dir, pattern);
    }

    let base_dir = ::std::path::PathBuf::from(".");
    let pattern = trimmed.to_string();
    (base_dir, pattern)
}

fn build_glob_matcher(pattern: &::core::primitive::str) -> ::core::result::Result<::globset::GlobMatcher, ::rig::tool::ToolError> {
    let mut builder = ::globset::GlobBuilder::new(pattern);
    builder.literal_separator(false);
    builder.backslash_escape(true);
    builder.empty_alternates(true);
    let glob_res = builder.build();
    let glob = match glob_res {
        ::core::result::Result::Ok(built_glob) => built_glob,
        ::core::result::Result::Err(err) => {
            let err_msg = ::std::format!("Invalid glob pattern: {err}");
            let err_obj = err_msg.into_tool_error();
            return ::core::result::Result::Err(err_obj);
        },
    };
    let matcher = glob.compile_matcher();
    ::core::result::Result::Ok(matcher)
}

fn collect_glob_matches(
    base_dir: &::std::path::Path,
    matcher: &::globset::GlobMatcher,
    include_hidden: ::core::primitive::bool,
    respect_gitignore: ::core::primitive::bool,
    limit: ::core::primitive::usize,
) -> ::std::vec::Vec<::std::string::String> {
    if !base_dir.exists() {
        return ::std::vec::Vec::new();
    }

    if base_dir.is_file() {
        let normalized = self::normalize_entry_path(base_dir);
        let is_match = matcher.is_match(&normalized);
        if is_match {
            return ::std::vec![normalized];
        }
        return ::std::vec::Vec::new();
    }

    let mut builder = ::ignore::WalkBuilder::new(base_dir);
    builder.hidden(!include_hidden);
    builder.git_ignore(respect_gitignore);
    builder.git_global(respect_gitignore);
    builder.git_exclude(respect_gitignore);

    let allows_target = base_dir.starts_with("target");
    let allows_node_modules = base_dir.starts_with("node_modules");
    builder.filter_entry(move |entry| {
        let file_name = entry.file_name().to_string_lossy();
        file_name != ".git"
            && (allows_node_modules || file_name != "node_modules")
            && (allows_target || file_name != "target")
    });

    let mut matches = builder
        .build()
        .filter_map(::core::result::Result::ok)
        .filter(|entry| entry.depth() > 0)
        .map(|entry| {
            let entry_path = entry.path();
            self::normalize_entry_path(entry_path)
        })
        .filter(|normalized_path| matcher.is_match(normalized_path))
        .take(limit)
        .collect::<::std::vec::Vec<::std::string::String>>();

    matches.sort();
    matches.dedup();
    matches
}

fn normalize_entry_path(path: &::std::path::Path) -> ::std::string::String {
    let lossy_str = path.to_string_lossy();
    let forward_slashed = lossy_str.replace('\\', "/");
    let stripped_opt = forward_slashed.strip_prefix("./");
    match stripped_opt {
        ::core::option::Option::Some(unprefixed) => unprefixed.to_string(),
        ::core::option::Option::None => forward_slashed,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_normalize_entry_path() {
        let path_prefixed = ::std::path::Path::new("./foo/bar.rs");
        let normalized = super::normalize_entry_path(path_prefixed);
        ::std::assert_eq!(normalized, "foo/bar.rs");

        let path_clean = ::std::path::Path::new("foo/bar.rs");
        let normalized_clean = super::normalize_entry_path(path_clean);
        ::std::assert_eq!(normalized_clean, "foo/bar.rs");
    }

    #[test]
    fn test_collect_glob_matches_functional() {
        let temp_dir = ::std::env::temp_dir();
        let unique_timestamp = ::std::time::SystemTime::now()
            .duration_since(::std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let root = temp_dir.join(::std::format!("werthan-glob-test-{unique_timestamp}"));
        ::std::fs::create_dir_all(&root).expect("create test root");

        let file_one = root.join("alpha.rs");
        let file_two = root.join("beta.txt");
        let file_three = root.join("gamma.rs");
        ::std::fs::write(&file_one, "fn alpha() {}").expect("write alpha");
        ::std::fs::write(&file_two, "text content").expect("write beta");
        ::std::fs::write(&file_three, "fn gamma() {}").expect("write gamma");

        let matcher = super::build_glob_matcher("**/*.rs").expect("valid matcher");
        let matches_all = super::collect_glob_matches(&root, &matcher, false, true, 100);
        ::std::assert_eq!(matches_all.len(), 2);

        let matches_limited = super::collect_glob_matches(&root, &matcher, false, true, 1);
        ::std::assert_eq!(matches_limited.len(), 1);

        let file_matcher = super::build_glob_matcher("**/alpha.rs").expect("valid file matcher");
        let matches_file = super::collect_glob_matches(&file_one, &file_matcher, false, true, 100);
        ::std::assert_eq!(matches_file.len(), 1);

        let non_existent = root.join("does_not_exist");
        let matches_non_existent = super::collect_glob_matches(&non_existent, &matcher, false, true, 100);
        ::std::assert!(matches_non_existent.is_empty());

        let _ignored = ::std::fs::remove_dir_all(&root);
    }
}
