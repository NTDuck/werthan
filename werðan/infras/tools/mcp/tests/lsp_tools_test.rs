//! Tests for lsp-tools.

#[test]
fn test_search_files_in_process() {
    let temp_dir = ::std::env::temp_dir().join("lsp_tools_test_dir");
    let _ = ::std::fs::remove_dir_all(&temp_dir);
    ::std::fs::create_dir_all(&temp_dir).unwrap();

    let file_a = temp_dir.join("a.rs");
    ::std::fs::write(&file_a, "fn hello() {}\nfn world() {}\n").unwrap();

    let sub_dir = temp_dir.join("sub");
    ::std::fs::create_dir_all(&sub_dir).unwrap();
    let file_b = sub_dir.join("b.txt");
    ::std::fs::write(&file_b, "hello there\ngeneral kenobi\n").unwrap();

    let matches = ::mcp::lsp_tools::search_files_in_process(&temp_dir, "hello").unwrap();
    ::std::assert_eq!(matches.len(), 2);

    let _ = ::std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_format_edit_with_lsp_diagnostics() {
    let edit_res = "Edit applied cleanly";
    let diags = ::std::vec![::mcp::lsp_tools::LspDiagnostic {
        file_path: { ::std::string::String::from("src/main.rs") },
        line: { 42 },
        message: { ::std::string::String::from("type mismatch") },
        severity: { ::std::string::String::from("error") },
    }];

    let formatted = ::mcp::lsp_tools::format_edit_with_lsp_diagnostics(edit_res, &diags);
    ::std::assert!(formatted.contains("Edit applied cleanly"));
    ::std::assert!(formatted.contains("LSP Diagnostics:"));
    ::std::assert!(formatted.contains("[error] src/main.rs:42: type mismatch"));
}
