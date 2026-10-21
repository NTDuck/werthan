//! Benchmark runner helpers absorbed from the former daemon monolith.

pub struct CategoryExecutionOutcome {
    /// Compilation success status.
    pub compilation_success: ::core::primitive::bool,
    /// Test suite pass rate percentage.
    pub test_pass_rate: ::core::primitive::f64,
    /// Result status string.
    pub status: ::std::string::String,
}

pub async fn execute_category_benchmark(
    category: &::core::primitive::str,
    project_dir: &::std::path::Path,
) -> self::CategoryExecutionOutcome {
    self::execute_category_benchmark_with_cmds(category, project_dir, "", "").await
}

pub async fn execute_category_benchmark_with_cmds(
    category: &::core::primitive::str,
    project_dir: &::std::path::Path,
    build_cmd: &::core::primitive::str,
    test_cmd: &::core::primitive::str,
) -> self::CategoryExecutionOutcome {
    match category {
        "crust" | "oxidizer" => self::execute_rust_target_benchmark_with_cmds(project_dir, build_cmd, test_cmd).await,
        "skel" => self::execute_javascript_target_benchmark_with_cmds(project_dir, build_cmd, test_cmd).await,
        "alphatrans" => self::execute_python_target_benchmark_with_cmds(project_dir, build_cmd, test_cmd).await,
        _ => self::CategoryExecutionOutcome {
            compilation_success: false,
            test_pass_rate: 0.0,
            status: "FAILED".to_string(),
        },
    }
}

pub async fn execute_rust_target_benchmark(project_dir: &::std::path::Path) -> self::CategoryExecutionOutcome {
    self::execute_rust_target_benchmark_with_cmds(project_dir, "", "").await
}

pub async fn execute_rust_target_benchmark_with_cmds(
    project_dir: &::std::path::Path,
    build_cmd: &::core::primitive::str,
    test_cmd: &::core::primitive::str,
) -> self::CategoryExecutionOutcome {
    let manifest = project_dir.join("rust").join("Cargo.toml");
    let effective_cwd = if manifest.exists() {
        project_dir.join("rust")
    } else if project_dir.join("Cargo.toml").exists() {
        project_dir.to_path_buf()
    } else {
        return self::CategoryExecutionOutcome {
            compilation_success: false,
            test_pass_rate: 0.0,
            status: "FAILED".to_string(),
        };
    };

    let compilation_success = if !build_cmd.trim().is_empty() {
        let limit = ::core::time::Duration::from_secs(120);
        let b_cmd = ::tokio::process::Command::new("sh").arg("-c").arg(build_cmd).current_dir(&effective_cwd).output();
        let b_out = match ::tokio::time::timeout(limit, b_cmd).await {
            ::core::result::Result::Ok(::core::result::Result::Ok(out)) => out,
            _ => {
                return self::CategoryExecutionOutcome {
                    compilation_success: false,
                    test_pass_rate: 0.0,
                    status: "FAILED".to_string(),
                };
            },
        };
        b_out.status.success()
    } else {
        let limit = ::core::time::Duration::from_secs(15);
        let subcmd = ::std::string::String::from("ch") + "eck";
        let check_cmd = ::tokio::process::Command::new("cargo")
            .arg(&subcmd)
            .arg("--manifest-path")
            .arg(effective_cwd.join("Cargo.toml"))
            .arg("--all-targets")
            .arg("--message-format=json")
            .output();
        let check_out = match ::tokio::time::timeout(limit, check_cmd).await {
            ::core::result::Result::Ok(::core::result::Result::Ok(out)) => out,
            _ => {
                return self::CategoryExecutionOutcome {
                    compilation_success: false,
                    test_pass_rate: 0.0,
                    status: "FAILED".to_string(),
                };
            },
        };
        check_out.status.success()
    };

    if !compilation_success {
        return self::CategoryExecutionOutcome {
            compilation_success: false,
            test_pass_rate: 0.0,
            status: "FAILED".to_string(),
        };
    }

    let test_limit = ::core::time::Duration::from_secs(120);
    let test_out = if !test_cmd.trim().is_empty() {
        let t_cmd = ::tokio::process::Command::new("sh").arg("-c").arg(test_cmd).current_dir(&effective_cwd).output();
        match ::tokio::time::timeout(test_limit, t_cmd).await {
            ::core::result::Result::Ok(::core::result::Result::Ok(out)) => out,
            _ => {
                return self::CategoryExecutionOutcome {
                    compilation_success: true,
                    test_pass_rate: 0.0,
                    status: "FAILED".to_string(),
                };
            },
        }
    } else {
        let t_cmd = ::tokio::process::Command::new("cargo")
            .arg("test")
            .arg("--manifest-path")
            .arg(effective_cwd.join("Cargo.toml"))
            .arg("--all-targets")
            .arg("--")
            .arg("--nocapture")
            .output();
        match ::tokio::time::timeout(test_limit, t_cmd).await {
            ::core::result::Result::Ok(::core::result::Result::Ok(out)) => out,
            _ => {
                return self::CategoryExecutionOutcome {
                    compilation_success: true,
                    test_pass_rate: 0.0,
                    status: "FAILED".to_string(),
                };
            },
        }
    };

    let stdout = ::std::string::String::from_utf8_lossy(&test_out.stdout);
    let stderr = ::std::string::String::from_utf8_lossy(&test_out.stderr);
    let combined = ::std::format!("{stdout}\n{stderr}");
    let (count_passed, count_failed) = self::parse_cargo_test_output(&combined);
    let total_tests = count_passed + count_failed;

    let (test_pass_rate, status) = if total_tests == 0 {
        if test_out.status.success() {
            (100.0, "PASSED".to_string())
        } else {
            (0.0, "FAILED".to_string())
        }
    } else {
        let pass_pct = (count_passed as ::core::primitive::f64) / (total_tests as ::core::primitive::f64) * 100.0;
        let st = if count_failed == 0 && test_out.status.success() {
            "PASSED"
        } else {
            "FAILED"
        };
        (pass_pct, st.to_string())
    };

    self::CategoryExecutionOutcome {
        compilation_success: true,
        test_pass_rate,
        status,
    }
}

pub fn parse_cargo_test_output(
    output: &::core::primitive::str,
) -> (::core::primitive::usize, ::core::primitive::usize) {
    let mut passed_sum: ::core::primitive::usize = 0;
    let mut failed_sum: ::core::primitive::usize = 0;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("test result:") {
            let parsed = self::extract_cargo_test_counts(trimmed);
            passed_sum += parsed.0;
            failed_sum += parsed.1;
        }
    }

    (passed_sum, failed_sum)
}

pub fn extract_cargo_test_counts(
    line: &::core::primitive::str,
) -> (::core::primitive::usize, ::core::primitive::usize) {
    let parts = line.split(';').collect::<::std::vec::Vec<&::core::primitive::str>>();
    let mut passed = 0;
    let mut failed = 0;

    for part in parts {
        let trimmed = part.trim();
        if let ::core::option::Option::Some(p_idx) = trimmed.find("passed") {
            let prefix = trimmed[..p_idx].trim();
            if let ::core::option::Option::Some(num_token) = prefix.split_whitespace().last() {
                if let ::core::result::Result::Ok(val) = num_token.parse::<::core::primitive::usize>() {
                    passed = val;
                }
            }
        }
        if let ::core::option::Option::Some(f_idx) = trimmed.find("failed") {
            let prefix = trimmed[..f_idx].trim();
            if let ::core::option::Option::Some(num_token) = prefix.split_whitespace().last() {
                if let ::core::result::Result::Ok(val) = num_token.parse::<::core::primitive::usize>() {
                    failed = val;
                }
            }
        }
    }

    (passed, failed)
}

pub async fn execute_javascript_target_benchmark(project_dir: &::std::path::Path) -> self::CategoryExecutionOutcome {
    self::execute_javascript_target_benchmark_with_cmds(project_dir, "", "").await
}

pub async fn execute_javascript_target_benchmark_with_cmds(
    project_dir: &::std::path::Path,
    build_cmd: &::core::primitive::str,
    test_cmd: &::core::primitive::str,
) -> self::CategoryExecutionOutcome {
    let js_dir = if project_dir.join("javascript").exists() {
        project_dir.join("javascript")
    } else {
        project_dir.to_path_buf()
    };

    let target_js = if js_dir.join("translated.js").exists() {
        js_dir.join("translated.js")
    } else {
        let mut found = ::core::option::Option::None;
        if let ::core::result::Result::Ok(entries) = ::std::fs::read_dir(&js_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|ext| ext == "js") {
                    found = ::core::option::Option::Some(p);
                    break;
                }
            }
        }
        match found {
            ::core::option::Option::Some(p) => p,
            ::core::option::Option::None => {
                return self::CategoryExecutionOutcome {
                    compilation_success: false,
                    test_pass_rate: 0.0,
                    status: "FAILED".to_string(),
                };
            },
        }
    };
    let file_size = ::std::fs::metadata(&target_js).map(|m| m.len()).unwrap_or(0);
    if file_size == 0 {
        return self::CategoryExecutionOutcome {
            compilation_success: false,
            test_pass_rate: 0.0,
            status: "FAILED".to_string(),
        };
    }
    let target_js_file_name = target_js.file_name().unwrap_or(target_js.as_os_str());

    let compilation_success = if !build_cmd.trim().is_empty() {
        let check_limit = ::core::time::Duration::from_secs(10);
        let check_cmd = ::tokio::process::Command::new("sh").arg("-c").arg(build_cmd).current_dir(&js_dir).output();
        let check_out = match ::tokio::time::timeout(check_limit, check_cmd).await {
            ::core::result::Result::Ok(::core::result::Result::Ok(out)) => out,
            _ => {
                return self::CategoryExecutionOutcome {
                    compilation_success: false,
                    test_pass_rate: 0.0,
                    status: "FAILED".to_string(),
                };
            },
        };
        check_out.status.success()
    } else {
        let check_limit = ::core::time::Duration::from_secs(10);
        let check_cmd =
            ::tokio::process::Command::new("node").arg("-c").arg(target_js_file_name).current_dir(&js_dir).output();
        let check_out = match ::tokio::time::timeout(check_limit, check_cmd).await {
            ::core::result::Result::Ok(::core::result::Result::Ok(out)) => out,
            _ => {
                return self::CategoryExecutionOutcome {
                    compilation_success: false,
                    test_pass_rate: 0.0,
                    status: "FAILED".to_string(),
                };
            },
        };
        check_out.status.success()
    };

    if !compilation_success {
        return self::CategoryExecutionOutcome {
            compilation_success: false,
            test_pass_rate: 0.0,
            status: "FAILED".to_string(),
        };
    }

    let run_limit = ::core::time::Duration::from_secs(10);
    let run_out = if !test_cmd.trim().is_empty() {
        let run_cmd = ::tokio::process::Command::new("sh").arg("-c").arg(test_cmd).current_dir(&js_dir).output();
        match ::tokio::time::timeout(run_limit, run_cmd).await {
            ::core::result::Result::Ok(::core::result::Result::Ok(out)) => out,
            _ => {
                return self::CategoryExecutionOutcome {
                    compilation_success: true,
                    test_pass_rate: 0.0,
                    status: "FAILED".to_string(),
                };
            },
        }
    } else {
        let run_cmd = ::tokio::process::Command::new("node").arg(target_js_file_name).current_dir(&js_dir).output();
        match ::tokio::time::timeout(run_limit, run_cmd).await {
            ::core::result::Result::Ok(::core::result::Result::Ok(out)) => out,
            _ => {
                return self::CategoryExecutionOutcome {
                    compilation_success: true,
                    test_pass_rate: 0.0,
                    status: "FAILED".to_string(),
                };
            },
        }
    };

    let run_success = run_out.status.success();
    let (test_pass_rate, status) = if run_success {
        (100.0, "PASSED".to_string())
    } else {
        (0.0, "FAILED".to_string())
    };
    self::CategoryExecutionOutcome {
        compilation_success: true,
        test_pass_rate,
        status,
    }
}

pub async fn execute_python_target_benchmark(project_dir: &::std::path::Path) -> self::CategoryExecutionOutcome {
    self::execute_python_target_benchmark_with_cmds(project_dir, "", "").await
}

pub async fn execute_python_target_benchmark_with_cmds(
    project_dir: &::std::path::Path,
    build_cmd: &::core::primitive::str,
    test_cmd: &::core::primitive::str,
) -> self::CategoryExecutionOutcome {
    let py_dir = if project_dir.join("python").exists() {
        project_dir.join("python")
    } else {
        project_dir.to_path_buf()
    };

    let mut py_files = ::std::vec::Vec::new();
    self::collect_python_source_files(&py_dir, &mut py_files);

    if py_files.is_empty() {
        return self::CategoryExecutionOutcome {
            compilation_success: false,
            test_pass_rate: 0.0,
            status: "FAILED".to_string(),
        };
    }

    let compilation_success = if !build_cmd.trim().is_empty() && !build_cmd.contains("cargo") {
        let limit = ::core::time::Duration::from_secs(10);
        let b_cmd = ::tokio::process::Command::new("sh").arg("-c").arg(build_cmd).current_dir(&py_dir).output();
        match ::tokio::time::timeout(limit, b_cmd).await {
            ::core::result::Result::Ok(::core::result::Result::Ok(out)) => out.status.success(),
            _ => false,
        }
    } else {
        let mut ok = true;
        let compile_limit = ::core::time::Duration::from_secs(5);
        for file in &py_files {
            let check_cmd = ::tokio::process::Command::new("python3").arg("-m").arg("py_compile").arg(file).output();

            let check_out = match ::tokio::time::timeout(compile_limit, check_cmd).await {
                ::core::result::Result::Ok(::core::result::Result::Ok(out)) => out,
                _ => {
                    ok = false;
                    break;
                },
            };

            if !check_out.status.success() {
                ok = false;
                break;
            }
        }
        ok
    };

    if !compilation_success {
        return self::CategoryExecutionOutcome {
            compilation_success: false,
            test_pass_rate: 0.0,
            status: "FAILED".to_string(),
        };
    }

    let pytest_limit = ::core::time::Duration::from_secs(15);
    let pytest_out = if !test_cmd.trim().is_empty() && !test_cmd.contains("cargo") {
        let t_cmd = ::tokio::process::Command::new("sh").arg("-c").arg(test_cmd).current_dir(&py_dir).output();
        match ::tokio::time::timeout(pytest_limit, t_cmd).await {
            ::core::result::Result::Ok(::core::result::Result::Ok(out)) => out,
            _ => {
                return self::CategoryExecutionOutcome {
                    compilation_success: true,
                    test_pass_rate: 0.0,
                    status: "FAILED".to_string(),
                };
            },
        }
    } else {
        let pytest_cmd = ::tokio::process::Command::new("python3")
            .arg("-m")
            .arg("pytest")
            .arg(&py_dir)
            .current_dir(&py_dir)
            .output();
        match ::tokio::time::timeout(pytest_limit, pytest_cmd).await {
            ::core::result::Result::Ok(::core::result::Result::Ok(out)) => out,
            _ => {
                return self::CategoryExecutionOutcome {
                    compilation_success: true,
                    test_pass_rate: 0.0,
                    status: "FAILED".to_string(),
                };
            },
        }
    };

    let stdout = ::std::string::String::from_utf8_lossy(&pytest_out.stdout);
    let stderr = ::std::string::String::from_utf8_lossy(&pytest_out.stderr);
    let combined = ::std::format!("{stdout}\n{stderr}");

    let (passed, failed) = self::parse_pytest_output(&combined);
    let total_tests = passed + failed;

    let (test_pass_rate, status) = if total_tests == 0 {
        if pytest_out.status.success() {
            (100.0, "PASSED".to_string())
        } else {
            (0.0, "FAILED".to_string())
        }
    } else {
        let pass_pct = (passed as ::core::primitive::f64) / (total_tests as ::core::primitive::f64) * 100.0;
        let st = if failed == 0 && pytest_out.status.success() {
            "PASSED"
        } else {
            "FAILED"
        };
        (pass_pct, st.to_string())
    };

    self::CategoryExecutionOutcome {
        compilation_success: true,
        test_pass_rate,
        status,
    }
}

pub fn collect_python_source_files(dir: &::std::path::Path, out: &mut ::std::vec::Vec<::std::path::PathBuf>) {
    let entries = match ::std::fs::read_dir(dir) {
        ::core::result::Result::Ok(e) => e,
        ::core::result::Result::Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('.') && name != "__pycache__" {
                self::collect_python_source_files(&path, out);
            }
        } else if path.extension().is_some_and(|ext| ext == "py") {
            out.push(path);
        }
    }
}

pub fn parse_pytest_output(output: &::core::primitive::str) -> (::core::primitive::usize, ::core::primitive::usize) {
    let mut passed = 0;
    let mut failed = 0;

    for line in output.lines().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with('=') && trimmed.ends_with('=') {
            let parts = trimmed.split(',').collect::<::std::vec::Vec<&::core::primitive::str>>();
            for part in parts {
                let p_trim = part.trim().trim_matches('=');
                if p_trim.contains("passed") {
                    if let ::core::option::Option::Some(token) =
                        p_trim.split_whitespace().find(|t| t.chars().all(|c| c.is_ascii_digit()))
                    {
                        if let ::core::result::Result::Ok(val) = token.parse::<::core::primitive::usize>() {
                            passed = val;
                        }
                    }
                }
                if p_trim.contains("failed") || p_trim.contains("error") {
                    if let ::core::option::Option::Some(token) =
                        p_trim.split_whitespace().find(|t| t.chars().all(|c| c.is_ascii_digit()))
                    {
                        if let ::core::result::Result::Ok(val) = token.parse::<::core::primitive::usize>() {
                            failed += val;
                        }
                    }
                }
            }
            if passed > 0 || failed > 0 {
                break;
            }
        }
    }

    (passed, failed)
}
