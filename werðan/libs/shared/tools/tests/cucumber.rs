//! Cucumber test harness for tools behavioral parity.

use ::rig::tool::Tool;

/// World state for cucumber test executions.
#[derive(::core::fmt::Debug, ::core::default::Default, ::cucumber::World)]
pub struct ToolsWorld {
    pub current_target_path: ::std::string::String,
    pub created_files: ::std::vec::Vec<::std::string::String>,
    pub last_read_output: ::core::option::Option<::tools::ReadOutput>,
    pub last_glob_output: ::core::option::Option<::tools::GlobOutput>,
    pub last_grep_output: ::core::option::Option<::tools::GrepOutput>,
    pub last_ast_grep_output: ::core::option::Option<::tools::AstGrepOutput>,
    pub last_lsp_output: ::core::option::Option<::tools::LspOutput>,
    pub last_bash_output: ::core::option::Option<::tools::BashOutput>,
    pub last_eval_output: ::core::option::Option<::tools::EvalOutput>,
    pub last_debug_output: ::core::option::Option<::tools::DebugOutput>,
    pub last_github_output: ::core::option::Option<::tools::GithubOutput>,
    pub last_call_succeeded: ::core::primitive::bool,
}

impl self::ToolsWorld {
    pub fn cleanup_files(&mut self) {
        for path in &self.created_files {
            let _ = ::std::fs::remove_file(path);
        }
        self.created_files.clear();
    }
}

impl ::core::ops::Drop for self::ToolsWorld {
    fn drop(&mut self) {
        self.cleanup_files();
    }
}

#[::cucumber::given(regex = r#"^a temporary file named "([^"]+)" with content:$"#)]
async fn given_temp_file(world: &mut self::ToolsWorld, step: &::cucumber::gherkin::Step, name: ::std::string::String) {
    let path = ::std::format!("tests/tmp_cuke_{name}");
    let doc_content = match &step.docstring {
        ::core::option::Option::Some(text) => text.trim().to_string(),
        ::core::option::Option::None => ::std::string::String::new(),
    };
    let write_res = ::std::fs::write(&path, doc_content.as_bytes());
    let write_ok = write_res.is_ok();
    ::core::assert!(write_ok);

    world.created_files.push(path.clone());
    world.current_target_path = path;
}

#[::cucumber::given(regex = r#"^a target file path "([^"]+)"$"#)]
async fn given_target_path(world: &mut self::ToolsWorld, name: ::std::string::String) {
    let path = ::std::format!("tests/tmp_cuke_{name}");
    world.created_files.push(path.clone());
    world.current_target_path = path;
}

#[::cucumber::when(regex = r#"^I call the read tool for "([^"]+)"$"#)]
async fn when_call_read(world: &mut self::ToolsWorld, name: ::std::string::String) {
    let read_tool = ::tools::ReadTool;
    let path = ::std::format!("tests/tmp_cuke_{name}");
    let read_args = ::tools::ReadArgs {
        path,
    };
    let res = read_tool.call(read_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
    world.last_read_output = res.ok();
}

#[::cucumber::when(regex = r#"^I call the write tool with content:$"#)]
async fn when_call_write(world: &mut self::ToolsWorld, step: &::cucumber::gherkin::Step) {
    let write_tool = ::tools::WriteTool;
    let path = world.current_target_path.clone();
    let text = match &step.docstring {
        ::core::option::Option::Some(text_str) => text_str.clone(),
        ::core::option::Option::None => ::std::string::String::new(),
    };
    let content = ::core::option::Option::Some(text);
    let write_args = ::tools::WriteArgs {
        path,
        content,
    };
    let res = write_tool.call(write_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
}

#[::cucumber::when(regex = r#"^I call the edit tool on "([^"]+)" with patch:$"#)]
async fn when_call_edit(world: &mut self::ToolsWorld, step: &::cucumber::gherkin::Step, name: ::std::string::String) {
    let edit_tool = ::tools::EditTool;
    let target = ::std::format!("tests/tmp_cuke_{name}");
    let patch_body = match &step.docstring {
        ::core::option::Option::Some(text_str) => text_str.trim_matches('\n'),
        ::core::option::Option::None => "",
    };
    let input = ::std::format!("[{target}]\n{patch_body}\n");
    let edit_args = ::tools::EditArgs {
        input,
    };
    world.current_target_path = target;
    let res = edit_tool.call(edit_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
}

#[::cucumber::when(regex = r#"^I call the glob tool with pattern "([^"]+)"$"#)]
async fn when_call_glob(world: &mut self::ToolsWorld, pattern: ::std::string::String) {
    let glob_tool = ::tools::GlobTool;
    let path = ::core::option::Option::Some(pattern);
    let hidden = ::core::option::Option::Some(false);
    let gitignore = ::core::option::Option::Some(false);
    let limit = ::core::option::Option::Some(20);

    let glob_args = ::tools::GlobArgs {
        path,
        hidden,
        gitignore,
        limit,
    };
    let res = glob_tool.call(glob_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
    world.last_glob_output = res.ok();
}

#[::cucumber::when(regex = r#"^I call the grep tool with pattern "([^"]+)" in path "([^"]+)"$"#)]
async fn when_call_grep(world: &mut self::ToolsWorld, pattern: ::std::string::String, path_str: ::std::string::String) {
    let grep_tool = ::tools::GrepTool;
    let path = ::core::option::Option::Some(path_str);
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
    let res = grep_tool.call(grep_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
    world.last_grep_output = res.ok();
}

#[::cucumber::when(regex = r#"^I call the ast grep tool for "([^"]+)" in "([^"]+)"$"#)]
async fn when_call_ast_grep(world: &mut self::ToolsWorld, pat: ::std::string::String, name: ::std::string::String) {
    let ast_grep_tool = ::tools::AstGrepTool;
    let target = ::std::format!("tests/tmp_cuke_{name}");
    let path = ::core::option::Option::Some(target);
    let lang = ::core::option::Option::Some("rust".to_string());
    let skip = ::core::option::Option::None;

    let grep_args = ::tools::AstGrepArgs {
        pat,
        path,
        lang,
        skip,
    };
    let res = ast_grep_tool.call(grep_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
    world.last_ast_grep_output = res.ok();
}

#[::cucumber::when(regex = r#"^I call the ast edit tool on "([^"]+)" to rewrite "([^"]+)" to "([^"]+)"$"#)]
async fn when_call_ast_edit(
    world: &mut self::ToolsWorld,
    name: ::std::string::String,
    pat: ::std::string::String,
    out: ::std::string::String,
) {
    let ast_edit_tool = ::tools::AstEditTool;
    let target = ::std::format!("tests/tmp_cuke_{name}");
    let op = ::tools::AstEditOp {
        pat,
        out,
    };
    let ops = ::std::vec![op];
    let paths = ::std::vec![target];

    let edit_args = ::tools::AstEditArgs {
        ops,
        paths,
    };
    let res = ast_edit_tool.call(edit_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
}

#[::cucumber::when(regex = r#"^I call the lsp tool with action "([^"]+)"$"#)]
async fn when_call_lsp_action(world: &mut self::ToolsWorld, action: ::std::string::String) {
    let lsp_tool = ::tools::LspTool;
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
    let res = lsp_tool.call(status_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
    world.last_lsp_output = res.ok();
}

#[::cucumber::when(regex = r#"^I call the lsp tool for symbols matching "([^"]+)" in "([^"]+)"$"#)]
async fn when_call_lsp_symbols(world: &mut self::ToolsWorld, query_str: ::std::string::String, file_str: ::std::string::String) {
    let lsp_tool = ::tools::LspTool;
    let action = "symbols".to_string();
    let file = ::core::option::Option::Some(file_str);
    let line = ::core::option::Option::None;
    let symbol = ::core::option::Option::None;
    let query = ::core::option::Option::Some(query_str);
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
    let res = lsp_tool.call(symbols_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
    world.last_lsp_output = res.ok();
}

#[::cucumber::when(regex = r#"^I call the lsp tool for diagnostics on "([^"]+)"$"#)]
async fn when_call_lsp_diagnostics(world: &mut self::ToolsWorld, name: ::std::string::String) {
    let lsp_tool = ::tools::LspTool;
    let target = ::std::format!("tests/tmp_cuke_{name}");
    let action = "diagnostics".to_string();
    let file = ::core::option::Option::Some(target);
    let line = ::core::option::Option::None;
    let symbol = ::core::option::Option::None;
    let query = ::core::option::Option::None;
    let new_name = ::core::option::Option::None;
    let apply = ::core::option::Option::None;
    let timeout = ::core::option::Option::None;
    let payload = ::core::option::Option::None;

    let diag_args = ::tools::LspArgs {
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
    let res = lsp_tool.call(diag_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
    world.last_lsp_output = res.ok();
}

#[::cucumber::when(regex = r#"^I call the lsp tool to rename "([^"]+)" to "([^"]+)" in "([^"]+)"$"#)]
async fn when_call_lsp_rename(
    world: &mut self::ToolsWorld,
    sym: ::std::string::String,
    target_new: ::std::string::String,
    name: ::std::string::String,
) {
    let lsp_tool = ::tools::LspTool;
    let target = ::std::format!("tests/tmp_cuke_{name}");
    let action = "rename".to_string();
    let file = ::core::option::Option::Some(target);
    let line = ::core::option::Option::None;
    let symbol = ::core::option::Option::Some(sym);
    let query = ::core::option::Option::None;
    let new_name = ::core::option::Option::Some(target_new);
    let apply = ::core::option::Option::Some(true);
    let timeout = ::core::option::Option::None;
    let payload = ::core::option::Option::None;

    let rename_args = ::tools::LspArgs {
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
    let res = lsp_tool.call(rename_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
    world.last_lsp_output = res.ok();
}

#[::cucumber::when(regex = r#"^I call the bash tool with command "([^"]+)"$"#)]
async fn when_call_bash(world: &mut self::ToolsWorld, command: ::std::string::String) {
    let bash_tool = ::tools::BashTool;
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
    let res = bash_tool.call(bash_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
    world.last_bash_output = res.ok();
}

#[::cucumber::when(regex = r#"^I call the eval tool with python code "([^"]+)"$"#)]
async fn when_call_eval(world: &mut self::ToolsWorld, code: ::std::string::String) {
    let eval_tool = ::tools::EvalTool;
    let language = "python".to_string();
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
    let res = eval_tool.call(eval_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
    world.last_eval_output = res.ok();
}

#[::cucumber::when(regex = r#"^I call the debug tool with action "([^"]+)"$"#)]
async fn when_call_debug(world: &mut self::ToolsWorld, action: ::std::string::String) {
    let debug_tool = ::tools::DebugTool;
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
    let res = debug_tool.call(debug_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
    world.last_debug_output = res.ok();
}

#[::cucumber::when(regex = r#"^I call the github tool with op "([^"]+)" and query "([^"]+)"$"#)]
async fn when_call_github(world: &mut self::ToolsWorld, op: ::std::string::String, query_str: ::std::string::String) {
    let github_tool = ::tools::GithubTool;
    let repo = ::core::option::Option::None;
    let pr = ::core::option::Option::None;
    let path = ::core::option::Option::None;
    let title = ::core::option::Option::None;
    let body = ::core::option::Option::None;
    let branch = ::core::option::Option::None;
    let query = ::core::option::Option::Some(query_str);
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
    let res = github_tool.call(gh_args).await;
    let is_ok = res.is_ok();
    world.last_call_succeeded = is_ok;
    world.last_github_output = res.ok();
}

#[::cucumber::then(regex = r#"^the read tool call succeeds$"#)]
async fn then_read_succeeds(world: &mut self::ToolsWorld) {
    ::core::assert!(world.last_call_succeeded);
}

#[::cucumber::then(regex = r#"^the read content contains "([^"]+)"$"#)]
async fn then_read_contains(world: &mut self::ToolsWorld, expected: ::std::string::String) {
    let read_opt = &world.last_read_output;
    let read_out = read_opt.as_ref().expect("ReadOutput present");
    let contains_str = read_out.content.contains(&expected);
    ::core::assert!(contains_str);
}

#[::cucumber::then(regex = r#"^the read lines count is (\d+)$"#)]
async fn then_read_lines_count(world: &mut self::ToolsWorld, count: ::core::primitive::usize) {
    let read_opt = &world.last_read_output;
    let read_out = read_opt.as_ref().expect("ReadOutput present");
    ::core::assert_eq!(read_out.lines_read, count);
}

#[::cucumber::then(regex = r#"^the write tool call succeeds$"#)]
async fn then_write_succeeds(world: &mut self::ToolsWorld) {
    ::core::assert!(world.last_call_succeeded);
}

#[::cucumber::then(regex = r#"^the target file exists on disk$"#)]
async fn then_target_exists(world: &mut self::ToolsWorld) {
    let path_ref = ::std::path::Path::new(&world.current_target_path);
    let exists = path_ref.exists();
    ::core::assert!(exists);
}

#[::cucumber::then(regex = r#"^the target file content contains "([^"]+)"$"#)]
async fn then_target_content_contains(world: &mut self::ToolsWorld, expected: ::std::string::String) {
    let read_res = ::std::fs::read_to_string(&world.current_target_path);
    let content = read_res.expect("File read succeeded");
    let contains_str = content.contains(&expected);
    ::core::assert!(contains_str);
}

#[::cucumber::then(regex = r#"^the target file content does not contain "([^"]+)"$"#)]
async fn then_target_content_not_contains(world: &mut self::ToolsWorld, unexpected: ::std::string::String) {
    let read_res = ::std::fs::read_to_string(&world.current_target_path);
    let content = read_res.expect("File read succeeded");
    let contains_str = content.contains(&unexpected);
    ::core::assert!(!contains_str);
}

#[::cucumber::then(regex = r#"^the edit tool call succeeds$"#)]
async fn then_edit_succeeds(world: &mut self::ToolsWorld) {
    ::core::assert!(world.last_call_succeeded);
}

#[::cucumber::then(regex = r#"^the glob tool call succeeds$"#)]
async fn then_glob_succeeds(world: &mut self::ToolsWorld) {
    ::core::assert!(world.last_call_succeeded);
}

#[::cucumber::then(regex = r#"^the glob result contains "([^"]+)"$"#)]
async fn then_glob_contains(world: &mut self::ToolsWorld, expected: ::std::string::String) {
    let glob_opt = &world.last_glob_output;
    let glob_out = glob_opt.as_ref().expect("GlobOutput present");
    let found = glob_out.matches.iter().any(|m| m.contains(&expected));
    ::core::assert!(found);
}

#[::cucumber::then(regex = r#"^the grep tool call succeeds$"#)]
async fn then_grep_succeeds(world: &mut self::ToolsWorld) {
    ::core::assert!(world.last_call_succeeded);
}

#[::cucumber::then(regex = r#"^the grep matches count is at least (\d+)$"#)]
async fn then_grep_count(world: &mut self::ToolsWorld, min_count: ::core::primitive::usize) {
    let grep_opt = &world.last_grep_output;
    let grep_out = grep_opt.as_ref().expect("GrepOutput present");
    let len = grep_out.matches.len();
    let is_ge = len >= min_count;
    ::core::assert!(is_ge);
}

#[::cucumber::then(regex = r#"^the ast grep tool call succeeds$"#)]
async fn then_ast_grep_succeeds(world: &mut self::ToolsWorld) {
    ::core::assert!(world.last_call_succeeded);
}

#[::cucumber::then(regex = r#"^the ast grep matches count is at least (\d+)$"#)]
async fn then_ast_grep_count(world: &mut self::ToolsWorld, min_count: ::core::primitive::usize) {
    let ast_grep_opt = &world.last_ast_grep_output;
    let ast_grep_out = ast_grep_opt.as_ref().expect("AstGrepOutput present");
    let len = ast_grep_out.matches.len();
    let is_ge = len >= min_count;
    ::core::assert!(is_ge);
}

#[::cucumber::then(regex = r#"^the ast edit tool call succeeds$"#)]
async fn then_ast_edit_succeeds(world: &mut self::ToolsWorld) {
    ::core::assert!(world.last_call_succeeded);
}

#[::cucumber::then(regex = r#"^the lsp tool call succeeds$"#)]
async fn then_lsp_succeeds(world: &mut self::ToolsWorld) {
    ::core::assert!(world.last_call_succeeded);
}

#[::cucumber::then(regex = r#"^the lsp status is "([^"]+)"$"#)]
async fn then_lsp_status(world: &mut self::ToolsWorld, expected: ::std::string::String) {
    let lsp_opt = &world.last_lsp_output;
    let lsp_out = lsp_opt.as_ref().expect("LspOutput present");
    ::core::assert_eq!(lsp_out.status, expected);
}

#[::cucumber::then(regex = r#"^the lsp output contains server "([^"]+)"$"#)]
async fn then_lsp_contains_server(world: &mut self::ToolsWorld, expected_server: ::std::string::String) {
    let lsp_opt = &world.last_lsp_output;
    let lsp_out = lsp_opt.as_ref().expect("LspOutput present");
    let json_text = lsp_out.result.to_string();
    let contains_server = json_text.contains(&expected_server);
    ::core::assert!(contains_server);
}

#[::cucumber::then(regex = r#"^the lsp output contains diagnostic message "([^"]+)"$"#)]
async fn then_lsp_contains_diag(world: &mut self::ToolsWorld, msg: ::std::string::String) {
    let lsp_opt = &world.last_lsp_output;
    let lsp_out = lsp_opt.as_ref().expect("LspOutput present");
    let json_text = lsp_out.result.to_string();
    let contains_msg = json_text.contains(&msg);
    ::core::assert!(contains_msg);
}

#[::cucumber::then(regex = r#"^the bash tool call succeeds$"#)]
async fn then_bash_succeeds(world: &mut self::ToolsWorld) {
    ::core::assert!(world.last_call_succeeded);
}

#[::cucumber::then(regex = r#"^the bash stdout contains "([^"]+)"$"#)]
async fn then_bash_stdout_contains(world: &mut self::ToolsWorld, expected: ::std::string::String) {
    let bash_opt = &world.last_bash_output;
    let bash_out = bash_opt.as_ref().expect("BashOutput present");
    let contains_str = bash_out.stdout.contains(&expected);
    ::core::assert!(contains_str);
}

#[::cucumber::then(regex = r#"^the eval tool call succeeds$"#)]
async fn then_eval_succeeds(world: &mut self::ToolsWorld) {
    ::core::assert!(world.last_call_succeeded);
}

#[::cucumber::then(regex = r#"^the eval stdout contains "([^"]+)"$"#)]
async fn then_eval_stdout_contains(world: &mut self::ToolsWorld, expected: ::std::string::String) {
    let eval_opt = &world.last_eval_output;
    let eval_out = eval_opt.as_ref().expect("EvalOutput present");
    let contains_str = eval_out.stdout.contains(&expected);
    ::core::assert!(contains_str);
}

#[::cucumber::then(regex = r#"^the debug tool call succeeds$"#)]
async fn then_debug_succeeds(world: &mut self::ToolsWorld) {
    ::core::assert!(world.last_call_succeeded);
}

#[::cucumber::then(regex = r#"^the debug status is "([^"]+)"$"#)]
async fn then_debug_status(world: &mut self::ToolsWorld, expected: ::std::string::String) {
    let debug_opt = &world.last_debug_output;
    let debug_out = debug_opt.as_ref().expect("DebugOutput present");
    ::core::assert_eq!(debug_out.status, expected);
}

#[::cucumber::then(regex = r#"^the github tool call succeeds$"#)]
async fn then_github_succeeds(world: &mut self::ToolsWorld) {
    ::core::assert!(world.last_call_succeeded);
}

#[::cucumber::then(regex = r#"^the github status is "([^"]+)"$"#)]
async fn then_github_status(world: &mut self::ToolsWorld, expected: ::std::string::String) {
    let gh_opt = &world.last_github_output;
    let gh_out = gh_opt.as_ref().expect("GithubOutput present");
    ::core::assert_eq!(gh_out.status, expected);
}

#[::tokio::main]
async fn main() {
    let args = ::std::env::args().collect::<::std::vec::Vec<::std::string::String>>();
    if args.iter().any(|arg| arg == "--list") {
        if !args.iter().any(|arg| arg == "--ignored") {
            ::std::println!("tools::cucumber: test");
        }
        return;
    }

    <self::ToolsWorld as ::cucumber::World>::cucumber()
        .with_default_cli()
        .fail_on_skipped()
        .run_and_exit("tests/features")
        .await;
}
