//! Line search and LSP diagnostic formatting tools.

/// A line match from in-process search.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct SearchMatch {
    /// File path of the match.
    pub file_path: ::std::string::String,
    /// One-based line number.
    pub line_number: ::core::primitive::usize,
    /// Content of the matched line.
    pub line_content: ::std::string::String,
}

/// Diagnostic report from language server.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct LspDiagnostic {
    /// File path of diagnostic.
    pub file_path: ::std::string::String,
    /// Line number of diagnostic.
    pub line: ::core::primitive::usize,
    /// Diagnostic description.
    pub message: ::std::string::String,
    /// Severity level such as error or warning.
    pub severity: ::std::string::String,
}

/// Search files in directory tree for query string without fork or exec.
pub fn search_files_in_process(
    root_dir: &::std::path::Path,
    query: &::core::primitive::str,
) -> ::core::result::Result<::std::vec::Vec<self::SearchMatch>, ::std::string::String> {
    let mut matches = ::std::vec::Vec::new();
    self::walk_and_search_dir(root_dir, query, &mut matches)?;
    ::core::result::Result::Ok(matches)
}

/// Format edit tool response with appended language server diagnostics.
pub fn format_edit_with_lsp_diagnostics(
    edit_result: &::core::primitive::str,
    diagnostics: &[self::LspDiagnostic],
) -> ::std::string::String {
    if diagnostics.is_empty() {
        return ::std::string::String::from(edit_result);
    }

    let mut output = ::std::string::String::from(edit_result);
    output.push_str("\n\nLSP Diagnostics:\n");
    for diag in diagnostics {
        let line_entry = ::std::format!("[{}] {}:{}: {}\n", diag.severity, diag.file_path, diag.line, diag.message);
        output.push_str(&line_entry);
    }
    output
}

fn walk_and_search_dir(
    dir: &::std::path::Path,
    query: &::core::primitive::str,
    matches: &mut ::std::vec::Vec<self::SearchMatch>,
) -> ::core::result::Result<(), ::std::string::String> {
    let entries = ::std::fs::read_dir(dir).map_err(|err| err.to_string())?;
    for entry_res in entries {
        let entry = entry_res.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            self::walk_and_search_dir(&path, query, matches)?;
        } else if path.is_file() {
            self::search_single_file(&path, query, matches)?;
        }
    }
    ::core::result::Result::Ok(())
}

fn search_single_file(
    file_path: &::std::path::Path,
    query: &::core::primitive::str,
    matches: &mut ::std::vec::Vec<self::SearchMatch>,
) -> ::core::result::Result<(), ::std::string::String> {
    let Ok(content) = ::std::fs::read_to_string(file_path) else {
        return ::core::result::Result::Ok(());
    };

    let path_str = file_path.to_string_lossy().into_owned();
    for (idx, line) in content.lines().enumerate() {
        if line.contains(query) {
            let matched_content = ::std::string::String::from(line);
            matches.push(self::SearchMatch {
                file_path: { path_str.clone() },
                line_number: { idx + 1 },
                line_content: { matched_content },
            });
        }
    }
    ::core::result::Result::Ok(())
}
