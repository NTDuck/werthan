//! Tests for all tools implementations.

#[::tokio::test]
async fn test_write_and_read_tool() {
    use ::rig::tool::Tool;

    let write_tool = ::tools::WriteTool;
    let path = "target/tmp_test_write.txt".to_string();
    let file_copy = path.clone();
    let content = ::core::option::Option::Some("Line one\nLine two\nLine three\n".to_string());

    let write_args = ::tools::WriteArgs {
        path,
        content,
    };
    let write_res = write_tool.call(write_args).await;
    let write_ok = write_res.is_ok();
    ::std::assert!(write_ok);

    let read_tool = ::tools::ReadTool;
    let path = file_copy.clone();
    let read_args = ::tools::ReadArgs {
        path,
    };
    let read_res = read_tool.call(read_args).await;
    let read_ok = read_res.is_ok();
    ::std::assert!(read_ok);

    let read_out = read_res.unwrap();
    let lines_count = read_out.lines_read;
    ::std::assert_eq!(lines_count, 3);

    let _ = ::std::fs::remove_file(file_copy);
}

#[::tokio::test]
async fn test_edit_tool() {
    use ::rig::tool::Tool;

    let target_file = "target/tmp_test_edit.txt".to_string();
    let initial_content = "alpha\nbeta\ngamma\n";
    let _ = ::std::fs::write(&target_file, initial_content);

    let edit_tool = ::tools::EditTool;
    let input = ::std::format!("[{}]\nPUT 2.=2:\n+beta_updated\n", target_file);
    let edit_args = ::tools::EditArgs {
        input,
    };

    let edit_res = edit_tool.call(edit_args).await;
    let edit_ok = edit_res.is_ok();
    ::std::assert!(edit_ok);

    let content_res = ::std::fs::read_to_string(&target_file);
    let content = content_res.unwrap();
    let contains_updated = content.contains("beta_updated");
    ::std::assert!(contains_updated);

    let _ = ::std::fs::remove_file(target_file);
}

#[::tokio::test]
async fn test_glob_tool() {
    use ::rig::tool::Tool;

    let glob_tool = ::tools::GlobTool;
    let path = ::core::option::Option::Some("src/*.rs".to_string());
    let hidden = ::core::option::Option::Some(false);
    let gitignore = ::core::option::Option::Some(false);
    let limit = ::core::option::Option::Some(10);

    let glob_args = ::tools::GlobArgs {
        path,
        hidden,
        gitignore,
        limit,
    };

    let glob_res = glob_tool.call(glob_args).await;
    let glob_ok = glob_res.is_ok();
    ::std::assert!(glob_ok);

    let glob_out = glob_res.unwrap();
    let matches_count = glob_out.matches.len();
    let has_matches = matches_count > 0;
    ::std::assert!(has_matches);
}

#[::tokio::test]
async fn test_grep_tool() {
    use ::rig::tool::Tool;

    let grep_tool = ::tools::GrepTool;
    let pattern = "Tool for".to_string();
    let path = ::core::option::Option::Some("src/read.rs".to_string());
    let case = ::core::option::Option::Some(true);
    let gitignore = ::core::option::Option::Some(false);
    let skip = ::core::option::Option::None;

    let grep_args = ::tools::GrepArgs {
        pattern,
        path,
        case,
        gitignore,
        skip,
    };

    let grep_res = grep_tool.call(grep_args).await;
    let grep_ok = grep_res.is_ok();
    ::std::assert!(grep_ok);

    let grep_out = grep_res.unwrap();
    let matches_count = grep_out.matches.len();
    let has_matches = matches_count > 0;
    ::std::assert!(has_matches);
}

#[::tokio::test]
async fn test_ast_grep_and_ast_edit_tool() {
    use ::rig::tool::Tool;

    let target_file = "target/tmp_test_ast.rs".to_string();
    let initial_code = "fn compute_total() -> i32 { 42 }\n";
    let _ = ::std::fs::write(&target_file, initial_code);

    let ast_grep_tool = ::tools::AstGrepTool;
    let pat = "fn compute_total".to_string();
    let path = ::core::option::Option::Some(target_file.clone());
    let lang = ::core::option::Option::Some("rust".to_string());
    let skip = ::core::option::Option::None;

    let grep_args = ::tools::AstGrepArgs {
        pat,
        path,
        lang,
        skip,
    };

    let grep_res = ast_grep_tool.call(grep_args).await;
    let grep_ok = grep_res.is_ok();
    ::std::assert!(grep_ok);

    let grep_out = grep_res.unwrap();
    let matches_len = grep_out.matches.len();
    let has_matches = matches_len > 0;
    ::std::assert!(has_matches);

    let ast_edit_tool = ::tools::AstEditTool;
    let pat = "compute_total".to_string();
    let out = "compute_aggregate".to_string();
    let op = ::tools::AstEditOp {
        pat,
        out,
    };
    let ops = ::std::vec![op];
    let paths = ::std::vec![target_file.clone()];
    let edit_args = ::tools::AstEditArgs {
        ops,
        paths,
    };

    let edit_res = ast_edit_tool.call(edit_args).await;
    let edit_ok = edit_res.is_ok();
    ::std::assert!(edit_ok);

    let content_res = ::std::fs::read_to_string(&target_file);
    let content = content_res.unwrap();
    let contains_new_name = content.contains("compute_aggregate");
    ::std::assert!(contains_new_name);

    let _ = ::std::fs::remove_file(target_file);
}

#[::tokio::test]
async fn test_lsp_tool() {
    use ::rig::tool::Tool;

    let lsp_tool = ::tools::LspTool;
    let action = "status".to_string();
    let file = ::core::option::Option::None;
    let line = ::core::option::Option::None;
    let symbol = ::core::option::Option::None;
    let query = ::core::option::Option::None;
    let new_name = ::core::option::Option::None;
    let apply = ::core::option::Option::None;
    let timeout = ::core::option::Option::None;
    let payload = ::core::option::Option::None;

    let status_args = ::tools::LspArgs {
        action,
        file,
        line,
        symbol,
        query,
        new_name,
        apply,
        timeout,
        payload,
    };

    let status_res = lsp_tool.call(status_args).await;
    let status_ok = status_res.is_ok();
    ::std::assert!(status_ok);

    let status_out = status_res.unwrap();
    ::std::assert_eq!(status_out.status, "ok");

    let action = "symbols".to_string();
    let file = ::core::option::Option::Some("src/lib.rs".to_string());
    let line = ::core::option::Option::None;
    let symbol = ::core::option::Option::None;
    let query = ::core::option::Option::Some("DisplayExt".to_string());
    let new_name = ::core::option::Option::None;
    let apply = ::core::option::Option::None;
    let timeout = ::core::option::Option::None;
    let payload = ::core::option::Option::None;

    let symbols_args = ::tools::LspArgs {
        action,
        file,
        line,
        symbol,
        query,
        new_name,
        apply,
        timeout,
        payload,
    };

    let symbols_res = lsp_tool.call(symbols_args).await;
    let symbols_ok = symbols_res.is_ok();
    ::std::assert!(symbols_ok);

    let symbols_out = symbols_res.unwrap();
    ::std::assert_eq!(symbols_out.status, "ok");
}

#[::tokio::test]
async fn test_bash_tool() {
    use ::rig::tool::Tool;

    let bash_tool = ::tools::BashTool;
    let command = "echo test_bash_output".to_string();
    let timeout = ::core::option::Option::Some(5);
    let cwd = ::core::option::Option::None;
    let pty = ::core::option::Option::None;
    let async_mode = ::core::option::Option::None;
    let name = ::core::option::Option::None;

    let bash_args = ::tools::BashArgs {
        command,
        timeout,
        cwd,
        pty,
        async_mode,
        name,
    };

    let bash_res = bash_tool.call(bash_args).await;
    let bash_ok = bash_res.is_ok();
    ::std::assert!(bash_ok);

    let bash_out = bash_res.unwrap();
    let contains_output = bash_out.stdout.contains("test_bash_output");
    ::std::assert!(contains_output);
}

#[::tokio::test]
async fn test_github_and_debug_and_eval_tools() {
    use ::rig::tool::Tool;

    let debug_tool = ::tools::DebugTool;
    let action = "status".to_string();
    let program = ::core::option::Option::None;
    let args = ::core::option::Option::None;
    let adapter = ::core::option::Option::None;
    let file = ::core::option::Option::None;
    let line = ::core::option::Option::None;
    let expression = ::core::option::Option::None;

    let debug_args = ::tools::DebugArgs {
        action,
        program,
        args,
        adapter,
        file,
        line,
        expression,
    };

    let debug_res = debug_tool.call(debug_args).await;
    let debug_ok = debug_res.is_ok();
    ::std::assert!(debug_ok);

    let github_tool = ::tools::GithubTool;
    let op = "search_repos".to_string();
    let repo = ::core::option::Option::None;
    let pr = ::core::option::Option::None;
    let path = ::core::option::Option::None;
    let title = ::core::option::Option::None;
    let body = ::core::option::Option::None;
    let branch = ::core::option::Option::None;
    let query = ::core::option::Option::Some("werthan".to_string());
    let limit = ::core::option::Option::Some(5);

    let gh_args = ::tools::GithubArgs {
        op,
        repo,
        pr,
        path,
        title,
        body,
        branch,
        query,
        limit,
    };

    let gh_res = github_tool.call(gh_args).await;
    let gh_ok = gh_res.is_ok();
    ::std::assert!(gh_ok);

    let eval_tool = ::tools::EvalTool;
    let language = "python".to_string();
    let code = "print(1 + 1)".to_string();
    let title = ::core::option::Option::None;
    let timeout = ::core::option::Option::Some(5);
    let reset = ::core::option::Option::None;

    let eval_args = ::tools::EvalArgs {
        language,
        code,
        title,
        timeout,
        reset,
    };

    let eval_res = eval_tool.call(eval_args).await;
    let eval_ok = eval_res.is_ok();
    ::std::assert!(eval_ok);
}
