// Pinned commit: d5cd750e41fede277f9eb933455da92a0738915e
// Reference: https://github.com/can1357/oh-my-pi/blob/d5cd750e41fede277f9eb933455da92a0738915e/packages/coding-agent/src/tools/gh.ts

use crate::DisplayExt;

/// Arguments for GitHub CLI tool operations.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct GithubArgs {
    pub op: ::std::string::String,
    pub repo: ::core::option::Option<::std::string::String>,
    pub branch: ::core::option::Option<::std::string::String>,
    pub path: ::core::option::Option<::std::string::String>,
    pub pr: ::core::option::Option<::std::string::String>,
    pub title: ::core::option::Option<::std::string::String>,
    pub body: ::core::option::Option<::std::string::String>,
    pub query: ::core::option::Option<::std::string::String>,
    pub limit: ::core::option::Option<::core::primitive::usize>,
}

/// Output from GitHub CLI tool operations.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct GithubOutput {
    pub status: ::std::string::String,
    pub output: ::std::string::String,
}

/// GitHub CLI tool interface.
#[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::default::Default)]
pub struct GithubTool;

impl ::rig::tool::Tool for self::GithubTool {
    const NAME: &'static ::core::primitive::str = "github";

    type Error = ::rig::tool::ToolError;
    type Args = self::GithubArgs;
    type Output = self::GithubOutput;

    async fn definition(&self, _prompt: ::std::string::String) -> ::rig::completion::ToolDefinition {
        let parameters = ::serde_json::json!({
            "type": "object",
            "properties": {
                "op": {
                    "type": "string",
                    "description": "GitHub operation name."
                },
                "repo": {
                    "type": "string",
                    "description": "Target repository in owner and name format."
                },
                "branch": {
                    "type": "string",
                    "description": "Git branch name."
                },
                "path": {
                    "type": "string",
                    "description": "Repository file path."
                },
                "pr": {
                    "type": "string",
                    "description": "Pull request number or branch."
                },
                "title": {
                    "type": "string",
                    "description": "Pull request or issue title."
                },
                "body": {
                    "type": "string",
                    "description": "Pull request or issue body description."
                },
                "query": {
                    "type": "string",
                    "description": "Search query text."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of items to return."
                }
            },
            "required": ["op"]
        });
        let name = Self::NAME.to_string();
        let description = "Execute GitHub CLI operations for repositories and pull requests.".to_string();
        ::rig::completion::ToolDefinition {
            name,
            description,
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        self::execute_github(&args).await
    }
}

async fn execute_github(args: &self::GithubArgs) -> ::core::result::Result<self::GithubOutput, ::rig::tool::ToolError> {
    let mut cmd = ::tokio::process::Command::new("gh");
    cmd.kill_on_drop(true);
    self::build_gh_args(&mut cmd, args);

    let output_future = cmd.output();
    let duration = ::tokio::time::Duration::from_secs(60);
    let timed_result = ::tokio::time::timeout(duration, output_future).await;

    let output = match timed_result {
        ::core::result::Result::Ok(process_result) => match process_result {
            ::core::result::Result::Ok(out) => out,
            ::core::result::Result::Err(err) => {
                let err_msg = ::std::format!("Failed to execute gh CLI: {err}");
                let err_obj = err_msg.into_tool_error();
                return ::core::result::Result::Err(err_obj);
            },
        },
        ::core::result::Result::Err(_) => {
            let err_msg = "GitHub CLI command timed out after 60 seconds.".to_string();
            let err_obj = err_msg.into_tool_error();
            return ::core::result::Result::Err(err_obj);
        },
    };

    let stdout_cow = ::std::string::String::from_utf8_lossy(&output.stdout);
    let stdout = stdout_cow.into_owned();
    let stderr_cow = ::std::string::String::from_utf8_lossy(&output.stderr);
    let stderr = stderr_cow.into_owned();

    let is_success = output.status.success();
    let status = if is_success {
        "success".to_string()
    } else {
        "failed".to_string()
    };

    let output = if stdout.is_empty() {
        stderr
    } else if stderr.is_empty() {
        stdout
    } else {
        ::std::format!("{stdout}\n{stderr}")
    };

    let gh_output = self::GithubOutput {
        status,
        output,
    };
    ::core::result::Result::Ok(gh_output)
}

fn build_gh_args(cmd: &mut ::tokio::process::Command, args: &self::GithubArgs) {
    match args.op.as_str() {
        "repo_view" | "repo" => {
            cmd.arg("repo");
            cmd.arg("view");
            if let ::core::option::Option::Some(repo) = &args.repo {
                cmd.arg(repo);
            }
        },
        "file_read" | "file" => {
            cmd.arg("api");
            let repo = args.repo.as_deref().unwrap_or("");
            let path = args.path.as_deref().unwrap_or("");
            let endpoint = ::std::format!("repos/{repo}/contents/{path}");
            cmd.arg(&endpoint);
            if let ::core::option::Option::Some(branch) = &args.branch {
                cmd.arg("-f");
                let ref_param = ::std::format!("ref={branch}");
                cmd.arg(&ref_param);
            }
        },
        "pr_create" => {
            cmd.arg("pr");
            cmd.arg("create");
            if let ::core::option::Option::Some(title) = &args.title {
                cmd.arg("--title");
                cmd.arg(title);
            }
            if let ::core::option::Option::Some(body) = &args.body {
                cmd.arg("--body");
                cmd.arg(body);
            }
            if let ::core::option::Option::Some(repo) = &args.repo {
                cmd.arg("--repo");
                cmd.arg(repo);
            }
            if let ::core::option::Option::Some(branch) = &args.branch {
                cmd.arg("--head");
                cmd.arg(branch);
            }
        },
        "pr_view" => {
            cmd.arg("pr");
            cmd.arg("view");
            if let ::core::option::Option::Some(pr) = &args.pr {
                cmd.arg(pr);
            }
            if let ::core::option::Option::Some(repo) = &args.repo {
                cmd.arg("--repo");
                cmd.arg(repo);
            }
        },
        "pr_list" => {
            cmd.arg("pr");
            cmd.arg("list");
            if let ::core::option::Option::Some(repo) = &args.repo {
                cmd.arg("--repo");
                cmd.arg(repo);
            }
            if let ::core::option::Option::Some(limit) = args.limit {
                cmd.arg("--limit");
                let limit_str = limit.to_string();
                cmd.arg(&limit_str);
            }
        },
        "issue_view" => {
            cmd.arg("issue");
            cmd.arg("view");
            if let ::core::option::Option::Some(pr) = &args.pr {
                cmd.arg(pr);
            }
            if let ::core::option::Option::Some(repo) = &args.repo {
                cmd.arg("--repo");
                cmd.arg(repo);
            }
        },
        "issue_list" => {
            cmd.arg("issue");
            cmd.arg("list");
            if let ::core::option::Option::Some(repo) = &args.repo {
                cmd.arg("--repo");
                cmd.arg(repo);
            }
            if let ::core::option::Option::Some(limit) = args.limit {
                cmd.arg("--limit");
                let limit_str = limit.to_string();
                cmd.arg(&limit_str);
            }
        },
        "search_issues" => {
            cmd.arg("search");
            cmd.arg("issues");
            if let ::core::option::Option::Some(query) = &args.query {
                cmd.arg(query);
            }
            if let ::core::option::Option::Some(limit) = args.limit {
                cmd.arg("--limit");
                let limit_str = limit.to_string();
                cmd.arg(&limit_str);
            }
        },
        "search_repos" => {
            cmd.arg("search");
            cmd.arg("repos");
            if let ::core::option::Option::Some(query) = &args.query {
                cmd.arg(query);
            }
            if let ::core::option::Option::Some(limit) = args.limit {
                cmd.arg("--limit");
                let limit_str = limit.to_string();
                cmd.arg(&limit_str);
            }
        },
        other => {
            let parts = other.split_whitespace().collect::<::std::vec::Vec<&::core::primitive::str>>();
            for part in parts {
                cmd.arg(part);
            }
            if let ::core::option::Option::Some(repo) = &args.repo {
                cmd.arg("--repo");
                cmd.arg(repo);
            }
        },
    }
}
