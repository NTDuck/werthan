//! Shared tools validation contracts.
//! Contracts derived from docs/specs/validation-contracts.md and they outrank implementation tests.
//! Covered assertion IDs follow. VAL-TOOL-01, VAL-TOOL-02, VAL-TOOL-03.

use ::tools::Tool;

/// Contract VAL-TOOL-01: read-only navigation tools resolve legacy symbols
/// without mutating any filesystem or workspace state.
#[::tokio::test]
async fn contract_val_tool_01_readonly_navigation_tools_do_not_mutate() {
    let base = ::std::env::temp_dir();
    let unique = ::std::time::SystemTime::now()
        .duration_since(::std::time::UNIX_EPOCH)
        .map(|span| span.as_nanos())
        .unwrap_or_default();
    let root = base.join(::std::format!("werthan-val-tool-readonly-{unique}"));
    ::std::fs::create_dir_all(&root).expect("test root create");
    let legacy_dir = root.join("legacy");
    ::std::fs::create_dir_all(&legacy_dir).expect("legacy dir create");
    let source_path = legacy_dir.join("legacy_vec.rs");
    ::std::fs::write(
        &source_path,
        "pub fn legacy_count_items() -> usize {\n    7\n}\n",
    )
    .expect("source write");
    let snapshot_before = collect_tree(&root);

    // Definition lookup over the legacy source.
    let lsp = ::tools::LspTool;
    let lsp_args = ::tools::LspArgs {
        action: "definition".to_string(),
        file: ::core::option::Option::Some(source_path.display().to_string()),
        line: ::core::option::Option::None,
        symbol: ::core::option::Option::Some("legacy_count_items".to_string()),
        query: ::core::option::Option::None,
        new_name: ::core::option::Option::None,
        apply: ::core::option::Option::None,
        timeout: ::core::option::Option::None,
        payload: ::core::option::Option::None,
    };
    let lsp_output = lsp.call(lsp_args).await.expect("definition lookup");
    ::std::assert_eq!(lsp_output.status, "ok");
    let definitions = lsp_output
        .result
        .get("definitions")
        .and_then(|value| value.as_array())
        .expect("definitions array");
    ::std::assert_eq!(definitions.len(), 1);

    // Text search over the legacy source.
    let grep = ::tools::GrepTool;
    let grep_args = ::tools::GrepArgs {
        pattern: "legacy_count_items".to_string(),
        path: ::core::option::Option::Some(legacy_dir.display().to_string()),
        case: ::core::option::Option::None,
        gitignore: ::core::option::Option::None,
        skip: ::core::option::Option::None,
    };
    let grep_output = grep.call(grep_args).await.expect("grep search");
    ::std::assert_eq!(grep_output.matches.len(), 1);

    // Glob navigation over the legacy tree.
    let glob = ::tools::GlobTool;
    let glob_args = ::tools::GlobArgs {
        path: ::core::option::Option::Some(legacy_dir.display().to_string()),
        hidden: ::core::option::Option::None,
        gitignore: ::core::option::Option::None,
        limit: ::core::option::Option::None,
    };
    let glob_output = glob.call(glob_args).await.expect("glob search");
    ::std::assert_eq!(glob_output.matches.len(), 1);

    // File read over the legacy source.
    let read = ::tools::ReadTool;
    let read_args = ::tools::ReadArgs {
        path: { source_path.display().to_string() },
    };
    let read_output = read.call(read_args).await.expect("file read");
    ::std::assert!(read_output.content.contains("legacy_count_items"));

    let snapshot_after = collect_tree(&root);
    ::std::assert_eq!(snapshot_before, snapshot_after);
    let _ = ::std::fs::remove_dir_all(&root);
}

/// Contract VAL-TOOL-02: mutation tools write only within their declared
/// target partition. The tool rejects attempts to touch other partitions.
#[::tokio::test]
async fn contract_val_tool_02_write_tools_reject_out_of_partition() {
    // The plain write tool stays general, so the partition-scoped mutation
    // tool carries the declared target partition boundary.
    let base = ::std::env::temp_dir();
    let unique = ::std::time::SystemTime::now()
        .duration_since(::std::time::UNIX_EPOCH)
        .map(|span| span.as_nanos())
        .unwrap_or_default();
    let partition = base.join(::std::format!("werthan-val-tool-partition-{unique}"));
    ::std::fs::create_dir_all(&partition).expect("partition root create");
    let writer = ::tools::PartitionWriteTool::new(partition.clone());

    // In-partition write succeeds and lands under the partition root.
    let inside_args = ::tools::WriteArgs {
        path: "src/module.rs".to_string(),
        content: ::core::option::Option::Some("pub fn inside() {}".to_string()),
    };
    let inside_output = writer.call(inside_args).await.expect("in-partition write");
    let inside_path = partition.join("src").join("module.rs");
    ::std::assert!(inside_path.is_file());
    ::std::assert!(inside_output.path.contains("module.rs"));

    // Absolute path outside the partition draws a rejection.
    let outside_base = ::std::env::temp_dir();
    let outside_unique = ::std::time::SystemTime::now()
        .duration_since(::std::time::UNIX_EPOCH)
        .map(|span| span.as_nanos())
        .unwrap_or_default();
    let other = outside_base.join(::std::format!("werthan-val-tool-outside-{outside_unique}"));
    ::std::fs::create_dir_all(&other).expect("outside root create");
    let absolute_args = ::tools::WriteArgs {
        path: { other.join("evil.rs").display().to_string() },
        content: ::core::option::Option::Some("pub fn evil() {}".to_string()),
    };
    let absolute_res = writer.call(absolute_args).await;
    ::std::assert!(absolute_res.is_err(), "the tool rejects absolute writes");
    ::std::assert!(!other.join("evil.rs").exists(), "no file landed outside");

    // Parent-directory escape sequence draws a rejection.
    let escape_args = ::tools::WriteArgs {
        path: "../escape.rs".to_string(),
        content: ::core::option::Option::Some("pub fn escape() {}".to_string()),
    };
    let escape_res = writer.call(escape_args).await;
    ::std::assert!(escape_res.is_err(), "the tool rejects escape writes");
    ::std::assert!(!partition.parent().unwrap().join("escape.rs").exists());

    // The rejection text names the declared partition boundary.
    let absolute_args_two = ::tools::WriteArgs {
        path: "/etc/werthan-outside".to_string(),
        content: ::core::option::Option::None,
    };
    let rejected = writer
        .call(absolute_args_two)
        .await
        .expect_err("absolute rejection");
    let rejected_text = rejected.to_string();
    ::std::assert!(rejected_text.contains("outside the declared partition root"));

    let _ = ::std::fs::remove_dir_all(&partition);
    let _ = ::std::fs::remove_dir_all(&other);
}

/// Contract VAL-TOOL-03: each shared tool implements ::rig::tool::Tool with
/// deterministic error types whose Display text stays consistent.
#[test]
fn contract_val_tool_03_rig_tool_trait_compliance() {
    // Tool names match the rig registry convention.
    ::std::assert_eq!(::tools::ReadTool::NAME, "read");
    ::std::assert_eq!(::tools::GrepTool::NAME, "grep");
    ::std::assert_eq!(::tools::GlobTool::NAME, "glob");
    ::std::assert_eq!(::tools::WriteTool::NAME, "write");
    ::std::assert_eq!(::tools::LspTool::NAME, "lsp");
    ::std::assert_eq!(::tools::PartitionWriteTool::NAME, "partition_write");

    // Deterministic error types carry stable Display text.
    let grep = ::tools::GrepTool;
    let runtime = ::tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let invalid_pattern = ::tools::GrepArgs {
        pattern: "([unclosed".to_string(),
        path: ::core::option::Option::None,
        case: ::core::option::Option::None,
        gitignore: ::core::option::Option::None,
        skip: ::core::option::Option::None,
    };
    let invalid_res = runtime.block_on(grep.call(invalid_pattern));
    let invalid_err = invalid_res.expect_err("invalid regex fails the call");
    let invalid_text = invalid_err.to_string();
    ::std::assert!(invalid_text.contains("Invalid regex pattern"));

    // The DisplayExt convention turns displayable errors into ToolCallError.
    let missing_base = ::std::env::temp_dir();
    let missing_unique = ::std::time::SystemTime::now()
        .duration_since(::std::time::UNIX_EPOCH)
        .map(|span| span.as_nanos())
        .unwrap_or_default();
    let missing_root = missing_base.join(::std::format!("werthan-val-tool-missing-{missing_unique}"));
    ::std::fs::create_dir_all(&missing_root).expect("missing root create");
    let grep_args = ::tools::GrepArgs {
        pattern: "anything".to_string(),
        path: ::core::option::Option::Some(missing_root.join("absent").display().to_string()),
        case: ::core::option::Option::None,
        gitignore: ::core::option::Option::None,
        skip: ::core::option::Option::None,
    };
    let missing_res = runtime.block_on(grep.call(grep_args));
    let missing_err = missing_res.expect_err("missing path fails the call");
    ::std::assert!(
        ::core::matches!(missing_err, ::rig::tool::ToolError::ToolCallError(_)),
        "tool errors surface as ToolCallError"
    );
    let _ = ::std::fs::remove_dir_all(&missing_root);
}

/// Collect a sorted snapshot of every file under one root.
fn collect_tree(root: &::std::path::Path) -> ::std::vec::Vec<::std::string::String> {
    let mut entries = ::std::vec::Vec::new();
    collect_tree_inner(root, root, &mut entries);
    entries.sort();
    entries
}

fn collect_tree_inner(
    root: &::std::path::Path,
    current: &::std::path::Path,
    entries: &mut ::std::vec::Vec<::std::string::String>,
) {
    let read_res = ::std::fs::read_dir(current);
    let read_dir = match read_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_tree_inner(root, &path, entries);
        } else {
            let relative = path.strip_prefix(root).unwrap_or(&path);
            entries.push(relative.display().to_string());
        }
    }
}
