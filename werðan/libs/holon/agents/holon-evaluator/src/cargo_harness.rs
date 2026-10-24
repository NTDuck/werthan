//! Cargo workspace execution harness for the ARIADNE evaluator.
//! Writes candidate code and characterization tests into a real cargo
//! workspace, then derives compilation status and test pass rates from
//! genuine cargo toolchain output.

use crate::ExecutionHarness;
use crate::HarnessExecutionOutput;

/// Explicit toolchain commands for an external (non-cargo) target stack.
/// Field names mirror `global-blackboard::HarnessConfig`.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::cmp::PartialEq, ::core::cmp::Eq)]
pub struct HarnessCommands {
    /// Target build command.
    pub build_cmd: ::std::string::String,
    /// Target test command.
    pub test_cmd: ::std::string::String,
    /// Legacy harness execution command.
    pub legacy_harness_cmd: ::std::string::String,
}

impl ::core::default::Default for self::HarnessCommands {
    /// Default commands reproduce the built-in cargo behavior.
    fn default() -> Self {
        Self {
            build_cmd: ::std::string::String::from("cargo check --all-targets"),
            test_cmd: ::std::string::String::from("cargo test --all-targets"),
            legacy_harness_cmd: ::std::string::String::new(),
        }
    }
}

#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct CargoHarness {
    /// Absolute path of the target cargo workspace root.
    pub workspace_root: ::std::path::PathBuf,
    /// Compilation command timeout in seconds.
    pub build_timeout_secs: ::core::primitive::u64,
    /// Test command timeout in seconds.
    pub test_timeout_secs: ::core::primitive::u64,
    /// Explicit toolchain commands, defaulting to the built-in cargo behavior.
    pub commands: self::HarnessCommands,
}

#[::bon::bon]
impl self::CargoHarness {
    /// Construct a harness over the given workspace with explicit timeouts.
    /// The harness derives the target source file from the executed module
    /// identifier, so one harness instance serves every module vertex.
    pub fn with_timeouts(
        workspace_root: ::std::path::PathBuf,
        _source_file: ::std::path::PathBuf,
        build_timeout_secs: ::core::primitive::u64,
        test_timeout_secs: ::core::primitive::u64,
    ) -> Self {
        Self::builder()
            .workspace_root(workspace_root)
            .build_timeout_secs(build_timeout_secs)
            .test_timeout_secs(test_timeout_secs)
            .build()
    }

    /// Construct a harness over the given workspace with explicit configuration.
    /// The harness derives the target source file from the executed module
    /// identifier, so one harness instance serves every module vertex.
    /// `commands` overrides the build and test commands, defaulting to cargo.
    #[builder]
    pub fn builder(
        workspace_root: ::std::path::PathBuf,
        build_timeout_secs: ::core::primitive::u64,
        test_timeout_secs: ::core::primitive::u64,
        commands: ::core::option::Option<self::HarnessCommands>,
    ) -> Self {
        let commands = commands.unwrap_or_default();
        Self {
            workspace_root,
            build_timeout_secs,
            test_timeout_secs,
            commands,
        }
    }
}

impl self::CargoHarness {
    /// Split one configured command string into program and argument tokens.
    /// Tokens separate on ASCII whitespace, matching shell word splitting
    /// closely enough for toolchain invocation strings.
    pub fn parse_command(
        command_text: &::core::primitive::str,
    ) -> (::std::string::String, ::std::vec::Vec<::std::string::String>) {
        let mut token_iter = command_text.split_whitespace();
        let program_name = token_iter.next().unwrap_or("").to_string();
        let argument_tokens = token_iter
            .map(::std::string::ToString::to_string)
            .collect::<::std::vec::Vec<::std::string::String>>();
        (program_name, argument_tokens)
    }

    /// Resolve the workspace-relative candidate file for one module identifier.
    /// Cargo toolchains keep the flat `src/{stem}.rs` layout. External
    /// toolchains derive the target file from the last build argument that
    /// contains a dot. When no argument names a file, the derivation fails.
    fn candidate_file_for(
        &self,
        module_id: &::core::primitive::str,
        build_program: &::core::primitive::str,
        build_args: &[::std::string::String],
    ) -> ::core::option::Option<::std::path::PathBuf> {
        if build_program == "cargo" {
            return ::core::option::Option::Some(self.source_file_for(module_id));
        }
        build_args
            .iter()
            .rev()
            .find(|token_text| token_text.contains('.'))
            .map(::std::path::PathBuf::from)
    }

    /// Resolve the workspace-relative source file for one module identifier.
    /// Module ids are legacy-relative paths, so the final segment's stem names
    /// the flat target file `src/{stem}.rs`. A `mod` stem names the parent
    /// segment instead, mirroring Rust module directory semantics.
    fn source_file_for(&self, module_id: &::core::primitive::str) -> ::std::path::PathBuf {
        let trimmed_id = module_id.trim_matches('/');
        let segments = trimmed_id
            .split('/')
            .filter(|segment_text| !segment_text.is_empty())
            .collect::<::std::vec::Vec<&::core::primitive::str>>();
        let final_segment = segments
            .last()
            .copied()
            .unwrap_or(trimmed_id);
        let raw_stem = final_segment.split('.').next().unwrap_or(final_segment);
        let stem_is_mod = raw_stem == "mod";
        let has_parent = segments.len() > 1;
        let stem_text = if stem_is_mod && has_parent {
            let parent_segment = segments[segments.len() - 2];
            parent_segment.split('.').next().unwrap_or(parent_segment)
        } else {
            raw_stem
        };
        ::std::path::PathBuf::from(::std::format!("src/{stem_text}.rs"))
    }

    /// Derive the crate package name from the workspace manifest.
    /// Returns an empty string when the manifest lacks a package name.
    fn crate_name(&self) -> ::std::string::String {
        let manifest_path = self.workspace_root.join("Cargo.toml");
        let manifest_text = match ::std::fs::read_to_string(manifest_path) {
            ::core::result::Result::Ok(text) => text,
            ::core::result::Result::Err(_) => return ::std::string::String::new(),
        };
        let mut in_package_section = false;
        for manifest_line in manifest_text.lines() {
            let trimmed_line = manifest_line.trim();
            if trimmed_line.starts_with('[') {
                in_package_section = trimmed_line == "[package]";
                continue;
            }
            if in_package_section {
                if let Some(name_pair) = trimmed_line.strip_prefix("name") {
                    let after_equals = name_pair.trim().trim_start_matches('=').trim();
                    let name_value = after_equals.trim_matches('"');
                    return name_value.to_string();
                }
            }
        }
        ::std::string::String::new()
    }

    /// Run one command inside the workspace with a hard timeout.
    /// Returns command output text or an error description on timeout.
    pub fn run_command(
        &self,
        program_name: &::core::primitive::str,
        program_args: &[&::core::primitive::str],
        timeout_secs: ::core::primitive::u64,
    ) -> ::core::result::Result<::std::process::Output, ::std::string::String> {
        let stdout_target = ::std::process::Stdio::piped();
        let stderr_target = ::std::process::Stdio::piped();
        let child_result = ::std::process::Command::new(program_name)
            .args(program_args)
            .current_dir(&self.workspace_root)
            .stdout(stdout_target)
            .stderr(stderr_target)
            .spawn();
        let mut child = match child_result {
            ::core::result::Result::Ok(spawned) => spawned,
            ::core::result::Result::Err(spawn_error) => {
                return ::core::result::Result::Err(spawn_error.to_string());
            }
        };
        let deadline = ::std::time::Instant::now() + ::std::time::Duration::from_secs(timeout_secs);
        loop {
            match child.try_wait() {
                ::core::result::Result::Ok(::core::option::Option::Some(_)) => break,
                ::core::result::Result::Ok(::core::option::Option::None) => {
                    if ::std::time::Instant::now() > deadline {
                        let _ = child.kill();
                        let _ = child.wait();
                        return ::core::result::Result::Err(::std::format!(
                            "{program_name} exceeded the {timeout_secs} second timeout"
                        ));
                    }
                    let poll_interval = ::std::time::Duration::from_millis(50);
                    ::std::thread::sleep(poll_interval);
                }
                ::core::result::Result::Err(wait_error) => {
                    return ::core::result::Result::Err(wait_error.to_string());
                }
            }
        }
        child.wait_with_output().map_err(|collect_error| collect_error.to_string())
    }

    /// Combine stdout and stderr into one log string.
    fn merge_logs(output: &::std::process::Output) -> ::std::string::String {
        let stdout_text = ::std::string::String::from_utf8_lossy(&output.stdout);
        let stderr_text = ::std::string::String::from_utf8_lossy(&output.stderr);
        let merged = ::std::format!("{}{}", stdout_text, stderr_text);
        merged
    }

    /// Parse cargo test terse output for passed and failed counters.
    fn parse_test_counts(test_log: &::core::primitive::str) -> (::core::primitive::usize, ::core::primitive::usize) {
        let mut passed_total: ::core::primitive::usize = 0;
        let mut failed_total: ::core::primitive::usize = 0;
        for result_line in test_log.lines() {
            let trimmed_line = result_line.trim();
            if trimmed_line.starts_with("test result:") {
                passed_total += Self::extract_count_after(trimmed_line, "passed");
                failed_total += Self::extract_count_after(trimmed_line, "failed");
            }
        }
        (passed_total, failed_total)
    }

    /// Extract the integer immediately preceding a counter word.
    fn extract_count_after(
        summary_line: &::core::primitive::str,
        counter_word: &::core::primitive::str,
    ) -> ::core::primitive::usize {
        let mut cursor = 0;
        while let Some(word_start) = summary_line[cursor..].find(counter_word) {
            let absolute_start = cursor + word_start;
            let before_word = &summary_line[..absolute_start];
            let number_text = before_word
                .split_whitespace()
                .next_back()
                .unwrap_or("");
            if let Ok(parsed_count) = number_text.parse::<::core::primitive::usize>() {
                return parsed_count;
            }
            cursor = absolute_start + counter_word.len();
        }
        0
    }
}

impl self::ExecutionHarness for self::CargoHarness {
    fn execute_candidate(
        &self,
        module_id: &::core::primitive::str,
        candidate_code: &::core::primitive::str,
        test_suite: &[::std::string::String],
        _target_language: &::core::primitive::str,
    ) -> self::HarnessExecutionOutput {
        let (build_program, build_tokens) = Self::parse_command(&self.commands.build_cmd);
        if build_program != "cargo" {
            return self.execute_external_candidate(
                &build_program,
                &build_tokens,
                candidate_code,
                test_suite,
            );
        }
        let source_relative = self.source_file_for(module_id);
        let source_path = self.workspace_root.join(&source_relative);
        if let ::core::result::Result::Err(write_error) =
            ::std::fs::write(&source_path, candidate_code)
        {
            return self::HarnessExecutionOutput {
                compiles: false,
                pass_rate: 0.0,
                raw_logs: ::std::format!("harness failed to write candidate source: {write_error}"),
            };
        }
        let test_path = self.workspace_root.join("tests/characterization.rs");
        if test_suite.is_empty() {
            let _ = ::std::fs::remove_file(test_path);
        } else {
            let crate_label = self.crate_name();
            let mut test_module_text = ::std::string::String::new();
            if !crate_label.is_empty() {
                let import_line = ::std::format!("use ::{crate_label}::*\x3b");
                test_module_text.push_str(&import_line);
                test_module_text.push_str("\n\n");
            }
            for test_entry in test_suite {
                test_module_text.push_str(test_entry);
                test_module_text.push_str("\n\n");
            }
            let _ = ::std::fs::create_dir_all(self.workspace_root.join("tests"));
            if let ::core::result::Result::Err(write_error) =
                ::std::fs::write(&test_path, test_module_text)
            {
                return self::HarnessExecutionOutput {
                    compiles: false,
                    pass_rate: 0.0,
                    raw_logs: ::std::format!("harness failed to write test file: {write_error}"),
                };
            }
        }
        let build_args = ["check", "--all-targets"];
        let build_output = self.run_command("cargo", &build_args, self.build_timeout_secs);
        let build_log = match build_output {
            ::core::result::Result::Ok(output) => {
                let merged_log = Self::merge_logs(&output);
                if !output.status.success() {
                    return self::HarnessExecutionOutput {
                        compiles: false,
                        pass_rate: 0.0,
                        raw_logs: { merged_log },
                    };
                }
                merged_log
            }
            ::core::result::Result::Err(timeout_text) => {
                return self::HarnessExecutionOutput {
                    compiles: false,
                    pass_rate: 0.0,
                    raw_logs: { timeout_text },
                };
            }
        };
        if test_suite.is_empty() {
            return self::HarnessExecutionOutput {
                compiles: true,
                pass_rate: 0.0,
                raw_logs: { build_log },
            };
        }
        let test_args = ["test", "--all-targets", "--", "--format", "terse"];
        let test_output = self.run_command("cargo", &test_args, self.test_timeout_secs);
        let test_log = match test_output {
            ::core::result::Result::Ok(output) => Self::merge_logs(&output),
            ::core::result::Result::Err(timeout_text) => {
                return self::HarnessExecutionOutput {
                    compiles: true,
                    pass_rate: 0.0,
                    raw_logs: ::std::format!("{build_log}\n{timeout_text}"),
                };
            }
        };
        let (passed_total, failed_total) = Self::parse_test_counts(&test_log);
        let full_log = ::std::format!("{build_log}\n{test_log}");
        if passed_total + failed_total == 0 {
            return self::HarnessExecutionOutput {
                compiles: true,
                pass_rate: 0.0,
                raw_logs: { full_log },
            };
        }
        let denominator_count = passed_total + failed_total;
        let denominator_value = denominator_count as ::core::primitive::f64;
        let numerator_value = passed_total as ::core::primitive::f64;
        let pass_ratio = numerator_value / denominator_value;
        self::HarnessExecutionOutput {
            compiles: true,
            pass_rate: { pass_ratio },
            raw_logs: { full_log },
        }
    }
}

impl self::CargoHarness {
    /// Execute one candidate with an external (non-cargo) toolchain.
    /// The candidate lands in the file named by the build command args.
    /// When no build argument names a file, execution fails without writing
    /// or running anything. A successful build command gates compilation.
    /// The test command exit code decides the pass rate.
    /// External stacks lack cargo test counts, so the harness writes no test file.
    fn execute_external_candidate(
        &self,
        build_program: &::core::primitive::str,
        build_tokens: &[::std::string::String],
        candidate_code: &::core::primitive::str,
        test_suite: &[::std::string::String],
    ) -> self::HarnessExecutionOutput {
        let target_relative = match self.candidate_file_for("", build_program, build_tokens) {
            ::core::option::Option::Some(derived_path) => derived_path,
            ::core::option::Option::None => {
                return self::HarnessExecutionOutput {
                    compiles: false,
                    pass_rate: 0.0,
                    raw_logs: ::std::string::String::from(
                        "harness could not derive the candidate file from the configured build command args. Add the target file path to the build command",
                    ),
                };
            }
        };
        let target_path = self.workspace_root.join(&target_relative);
        if let ::core::result::Result::Err(write_error) =
            ::std::fs::write(&target_path, candidate_code)
        {
            return self::HarnessExecutionOutput {
                compiles: false,
                pass_rate: 0.0,
                raw_logs: ::std::format!("harness failed to write candidate source: {write_error}"),
            };
        }
        let build_arg_refs = build_tokens
            .iter()
            .map(::std::string::String::as_str)
            .collect::<::std::vec::Vec<&::core::primitive::str>>();
        let build_output = self.run_command(build_program, &build_arg_refs, self.build_timeout_secs);
        let build_log = match build_output {
            ::core::result::Result::Ok(output) => {
                let merged_log = Self::merge_logs(&output);
                if !output.status.success() {
                    return self::HarnessExecutionOutput {
                        compiles: false,
                        pass_rate: 0.0,
                        raw_logs: { merged_log },
                    };
                }
                merged_log
            }
            ::core::result::Result::Err(timeout_text) => {
                return self::HarnessExecutionOutput {
                    compiles: false,
                    pass_rate: 0.0,
                    raw_logs: { timeout_text },
                };
            }
        };
        if test_suite.is_empty() {
            return self::HarnessExecutionOutput {
                compiles: true,
                pass_rate: 0.0,
                raw_logs: { build_log },
            };
        }
        let (test_program, test_tokens) = Self::parse_command(&self.commands.test_cmd);
        let test_arg_refs = test_tokens
            .iter()
            .map(::std::string::String::as_str)
            .collect::<::std::vec::Vec<&::core::primitive::str>>();
        let test_output = self.run_command(&test_program, &test_arg_refs, self.test_timeout_secs);
        let test_log = match test_output {
            ::core::result::Result::Ok(output) => {
                let merged_test_log = Self::merge_logs(&output);
                if output.status.success() {
                    let full_log = ::std::format!("{build_log}\n{merged_test_log}");
                    return self::HarnessExecutionOutput {
                        compiles: true,
                        pass_rate: 1.0,
                        raw_logs: { full_log },
                    };
                }
                merged_test_log
            }
            ::core::result::Result::Err(timeout_text) => {
                return self::HarnessExecutionOutput {
                    compiles: true,
                    pass_rate: 0.0,
                    raw_logs: ::std::format!("{build_log}\n{timeout_text}"),
                };
            }
        };
        self::HarnessExecutionOutput {
            compiles: true,
            pass_rate: 0.0,
            raw_logs: ::std::format!("{build_log}\n{test_log}"),
        }
    }
}

#[cfg(test)]
mod cargo_harness_tests {
    use super::*;
    use crate::ExecutionHarness;

    #[test]
    fn test_cargo_harness_passing_suite() {
        let workspace_root = self::make_temp_workspace("pass");
        let harness = self::CargoHarness::builder()
            .workspace_root(workspace_root.clone())
            .build_timeout_secs(60)
            .test_timeout_secs(60)
            .build();
        let candidate_code = "pub fn probe_add(value_x: i32, value_y: i32) -> i32 {\n    value_x + value_y\n}";
        let test_entry = "#[test]\nfn t_probe_add() {\n    ::core::assert_eq!(probe_add(2, 2), 4)\n}";
        let test_suite = ::std::vec![test_entry.to_string()];
        let outcome = harness.execute_candidate("src/lib.rs", candidate_code, &test_suite, "rust");
        ::core::assert!(outcome.compiles, "logs: {}", outcome.raw_logs);
        ::core::assert!(outcome.pass_rate > 0.99, "logs: {}", outcome.raw_logs);
        ::core::assert!(outcome.raw_logs.contains("test result:"));
        let _ = ::std::fs::remove_dir_all(workspace_root);
    }

    #[test]
    fn test_cargo_harness_failing_test_scores_zero() {
        let workspace_root = self::make_temp_workspace("fail");
        let harness = self::CargoHarness::builder()
            .workspace_root(workspace_root.clone())
            .build_timeout_secs(60)
            .test_timeout_secs(60)
            .build();
        let candidate_code = "pub fn probe_add(value_x: i32, value_y: i32) -> i32 {\n    value_x + value_y\n}";
        let test_entry = "#[test]\nfn t_probe_add() {\n    ::core::assert_eq!(probe_add(2, 2), 5)\n}";
        let test_suite = ::std::vec![test_entry.to_string()];
        let outcome = harness.execute_candidate("src/lib.rs", candidate_code, &test_suite, "rust");
        ::core::assert!(outcome.compiles);
        ::core::assert!(outcome.pass_rate < 0.01, "logs: {}", outcome.raw_logs);
        let _ = ::std::fs::remove_dir_all(workspace_root);
    }

    #[test]
    fn test_cargo_harness_broken_candidate_fails_build() {
        let workspace_root = self::make_temp_workspace("broken");
        let harness = self::CargoHarness::builder()
            .workspace_root(workspace_root.clone())
            .build_timeout_secs(60)
            .test_timeout_secs(60)
            .build();
        let candidate_code = "::core::compile_error!(\"sentinel\")";
        let empty_suite = ::std::vec::Vec::<::std::string::String>::new();
        let outcome = harness.execute_candidate("src/lib.rs", candidate_code, &empty_suite, "rust");
        ::core::assert!(!outcome.compiles);
        let _ = ::std::fs::remove_dir_all(workspace_root);
    }
    #[test]
    fn test_cargo_harness_builder_fields() {
        let workspace_root = ::std::path::PathBuf::from("/tmp/ariadne_probe");
        let harness = self::CargoHarness::builder()
            .workspace_root(workspace_root.clone())
            .build_timeout_secs(45)
            .test_timeout_secs(90)
            .build();
        ::core::assert_eq!(harness.workspace_root, workspace_root);
        ::core::assert_eq!(harness.build_timeout_secs, 45);
        ::core::assert_eq!(harness.test_timeout_secs, 90);
    }

    #[test]
    fn test_cargo_harness_module_targeting() {
        let workspace_root = ::std::path::PathBuf::from("/tmp/ariadne_probe");
        let harness = self::CargoHarness::builder()
            .workspace_root(workspace_root.clone())
            .build_timeout_secs(45)
            .test_timeout_secs(90)
            .build();
        let flat_module = ::std::path::PathBuf::from("src/lib.rs");
        ::core::assert_eq!(harness.source_file_for("src/lib.rs"), flat_module);
        let legacy_path_module = ::std::path::PathBuf::from("src/lexer.rs");
        ::core::assert_eq!(harness.source_file_for("parser/lexer.rs"), legacy_path_module);
        let mod_file_module = ::std::path::PathBuf::from("src/lexer.rs");
        ::core::assert_eq!(harness.source_file_for("parser/lexer/mod.rs"), mod_file_module);
        let bare_stem_module = ::std::path::PathBuf::from("src/lexer.rs");
        ::core::assert_eq!(harness.source_file_for("lexer.rs"), bare_stem_module);
    }

    #[test]
    fn test_external_js_toolchain_valid_candidate_compiles() {
        if !self::node_is_available() {
            return;
        }
        let workspace_root = self::make_temp_js_workspace("js-valid");
        let harness = self::CargoHarness::builder()
            .workspace_root(workspace_root.clone())
            .build_timeout_secs(30)
            .test_timeout_secs(30)
            .commands(self::HarnessCommands {
                build_cmd: ::std::string::String::from("node -c translated.js"),
                test_cmd: ::std::string::String::from("node translated.js"),
                legacy_harness_cmd: ::std::string::String::new(),
            })
            .build();
        let candidate_code = "function probe_add(value_x, value_y) {\n    return value_x + value_y\n}";
        let empty_suite = ::std::vec::Vec::<::std::string::String>::new();
        let outcome = harness.execute_candidate("source.py", candidate_code, &empty_suite, "javascript");
        ::core::assert!(outcome.compiles, "logs: {}", outcome.raw_logs);
        ::core::assert!(workspace_root.join("translated.js").exists());
        let _ = ::std::fs::remove_dir_all(workspace_root);
    }

    #[test]
    fn test_external_toolchain_without_dotted_arg_fails_honestly() {
        let workspace_root = self::make_temp_js_workspace("no-dotted-arg");
        let harness = self::CargoHarness::builder()
            .workspace_root(workspace_root.clone())
            .build_timeout_secs(30)
            .test_timeout_secs(30)
            .commands(self::HarnessCommands {
                build_cmd: ::std::string::String::from("make build"),
                test_cmd: ::std::string::String::from("make test"),
                legacy_harness_cmd: ::std::string::String::new(),
            })
            .build();
        let candidate_code = "function probe_add(value_x, value_y) {\n    return value_x + value_y\n}";
        let empty_suite = ::std::vec::Vec::<::std::string::String>::new();
        let outcome = harness.execute_candidate("source.py", candidate_code, &empty_suite, "javascript");
        ::core::assert!(!outcome.compiles);
        ::core::assert_eq!(outcome.pass_rate, 0.0);
        ::core::assert!(
            outcome.raw_logs.contains("could not derive"),
            "logs: {}",
            outcome.raw_logs
        );
        ::core::assert!(!workspace_root.join("make").exists());
        let _ = ::std::fs::remove_dir_all(workspace_root);
    }

    #[test]
    fn test_external_js_toolchain_invalid_candidate_fails_build() {
        if !self::node_is_available() {
            return;
        }
        let workspace_root = self::make_temp_js_workspace("js-broken");
        let harness = self::CargoHarness::builder()
            .workspace_root(workspace_root.clone())
            .build_timeout_secs(30)
            .test_timeout_secs(30)
            .commands(self::HarnessCommands {
                build_cmd: ::std::string::String::from("node -c translated.js"),
                test_cmd: ::std::string::String::from("node translated.js"),
                legacy_harness_cmd: ::std::string::String::new(),
            })
            .build();
        let candidate_code = "function broken( {\n    return";
        let empty_suite = ::std::vec::Vec::<::std::string::String>::new();
        let outcome = harness.execute_candidate("source.py", candidate_code, &empty_suite, "javascript");
        ::core::assert!(!outcome.compiles);
        ::core::assert_eq!(outcome.pass_rate, 0.0);
        let _ = ::std::fs::remove_dir_all(workspace_root);
    }

    /// Skip guard for machines without a node runtime on PATH.
    fn node_is_available() -> ::core::primitive::bool {
        ::std::process::Command::new("node")
            .arg("--version")
            .output()
            .map(|version_output| version_output.status.success())
            .unwrap_or(false)
    }

    /// Build a plain throwaway directory (no Cargo.toml) for JS runs.
    fn make_temp_js_workspace(tag_text: &::core::primitive::str) -> ::std::path::PathBuf {
        let unique_suffix = ::std::time::SystemTime::now()
            .duration_since(::std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.subsec_nanos())
            .unwrap_or(0);
        let root_dir = ::std::env::temp_dir().join(::std::format!("ariadne-js-harness-{tag_text}-{unique_suffix}"));
        let _ = ::std::fs::create_dir_all(&root_dir);
        root_dir
    }

    /// Build a tiny throwaway cargo workspace under the system temp dir.
    fn make_temp_workspace(tag_text: &::core::primitive::str) -> ::std::path::PathBuf {
        let unique_suffix = ::std::time::SystemTime::now()
            .duration_since(::std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.subsec_nanos())
            .unwrap_or(0);
        let root_dir = ::std::env::temp_dir().join(::std::format!("ariadne-harness-{tag_text}-{unique_suffix}"));
        let _ = ::std::fs::create_dir_all(root_dir.join("src"));
        let manifest_text = "[package]\nname = \"ariadne_harness_probe\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
        ::std::fs::write(root_dir.join("Cargo.toml"), manifest_text).expect("manifest writes");
        ::std::fs::write(root_dir.join("src/lib.rs"), "").expect("stub lib writes");
        root_dir
    }
}
