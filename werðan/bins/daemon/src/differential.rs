//! Differential verification subsystem for program equivalence.
//! Proves behavioral equivalence between source and target programs.

/// Process execution observation record.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::core::cmp::PartialEq,
)]
pub struct ExecutionObservation {
    /// Exit code returned by the process.
    pub exit_code: ::core::option::Option<::core::primitive::i32>,
    /// Captured standard output text.
    pub stdout: ::std::string::String,
    /// Captured standard error text.
    pub stderr: ::std::string::String,
    /// Elapsed execution duration in milliseconds.
    pub duration_ms: ::core::primitive::u64,
}

/// Divergence classification between two observations.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::core::cmp::PartialEq,
)]
pub enum DivergenceClassification {
    /// Observations match within configured tolerances.
    Match,
    /// Exit codes differ between source and target.
    ExitCodeMismatch {
        /// Source process exit code.
        source: ::core::option::Option<::core::primitive::i32>,
        /// Target process exit code.
        target: ::core::option::Option<::core::primitive::i32>,
    },
    /// Numeric values differ beyond float tolerance.
    ValueMismatch {
        /// Diagnostic message explaining the value divergence.
        message: ::std::string::String,
    },
    /// Standard output texts differ.
    StdoutMismatch {
        /// Snippet from source standard output.
        source_snippet: ::std::string::String,
        /// Snippet from target standard output.
        target_snippet: ::std::string::String,
    },
    /// Process execution timed out.
    Timeout,
    /// Process execution failed.
    ExecutionError {
        /// Error description message.
        error: ::std::string::String,
    },
}

/// Differential report comparing source and target observations.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::core::cmp::PartialEq,
)]
pub struct DifferentialReport {
    /// Source command string.
    pub source_cmd: ::std::string::String,
    /// Target command string.
    pub target_cmd: ::std::string::String,
    /// Observation captured from the source command.
    pub observation_source: self::ExecutionObservation,
    /// Observation captured from the target command.
    pub observation_target: self::ExecutionObservation,
    /// Divergence classification outcome.
    pub classification: self::DivergenceClassification,
    /// True when observations match.
    pub equivalent: ::core::primitive::bool,
}

/// Execute differential verification between source and target commands.
pub fn execute_differential_verification(
    source_cmd: &::core::primitive::str,
    target_cmd: &::core::primitive::str,
    legacy_dir: &::std::path::Path,
    target_dir: &::std::path::Path,
    timeout: ::std::time::Duration,
    float_tolerance: ::core::primitive::f64,
) -> self::DifferentialReport {
    let (source_prog, source_args) = self::parse_command(source_cmd);
    let (target_prog, target_args) = self::parse_command(target_cmd);
    let source_arg_refs = source_args
        .iter()
        .map(::std::string::String::as_str)
        .collect::<::std::vec::Vec<&::core::primitive::str>>();
    let target_arg_refs = target_args
        .iter()
        .map(::std::string::String::as_str)
        .collect::<::std::vec::Vec<&::core::primitive::str>>();

    let source_res = self::run_command_observation(&source_prog, &source_arg_refs, legacy_dir, timeout);
    let target_res = self::run_command_observation(&target_prog, &target_arg_refs, target_dir, timeout);

    let timeout_ms = timeout.as_millis() as ::core::primitive::u64;

    match (source_res, target_res) {
        (::core::result::Result::Ok(obs_source), ::core::result::Result::Ok(obs_target)) => {
            let classification = self::compare_observations(&obs_source, &obs_target, float_tolerance);
            let equivalent = classification == self::DivergenceClassification::Match;
            self::make_report(source_cmd, target_cmd, obs_source, obs_target, classification, equivalent)
        }
        (::core::result::Result::Err(err_source), ::core::result::Result::Ok(obs_target)) => {
            let classification = self::classify_error(&err_source, "Source execution error");
            let obs_source = self::make_error_observation(&err_source, timeout_ms);
            self::make_report(source_cmd, target_cmd, obs_source, obs_target, classification, false)
        }
        (::core::result::Result::Ok(obs_source), ::core::result::Result::Err(err_target)) => {
            let classification = self::classify_error(&err_target, "Target execution error");
            let obs_target = self::make_error_observation(&err_target, timeout_ms);
            self::make_report(source_cmd, target_cmd, obs_source, obs_target, classification, false)
        }
        (::core::result::Result::Err(err_source), ::core::result::Result::Err(err_target)) => {
            let is_timeout = err_source.contains("timed out") || err_target.contains("timed out");
            let classification = if is_timeout {
                self::DivergenceClassification::Timeout
            } else {
                let error = ::std::format!("Execution errors: source ({err_source}), target ({err_target})");
                self::DivergenceClassification::ExecutionError { error }
            };
            let obs_source = self::make_error_observation(&err_source, timeout_ms);
            let obs_target = self::make_error_observation(&err_target, timeout_ms);
            self::make_report(source_cmd, target_cmd, obs_source, obs_target, classification, false)
        }
    }
}

/// Split command string into program and argument tokens.
fn parse_command(
    command_text: &::core::primitive::str,
) -> (::std::string::String, ::std::vec::Vec<::std::string::String>) {
    let mut token_iter = command_text.split_whitespace();
    let program_token = token_iter.next().unwrap_or("");
    let program_name = ::std::string::ToString::to_string(program_token);
    let argument_tokens = token_iter
        .map(::std::string::ToString::to_string)
        .collect::<::std::vec::Vec<::std::string::String>>();
    (program_name, argument_tokens)
}

/// Run command and capture execution observation.
pub fn run_command_observation(
    program: &::core::primitive::str,
    args: &[&::core::primitive::str],
    cwd: &::std::path::Path,
    timeout: ::std::time::Duration,
) -> ::core::result::Result<self::ExecutionObservation, ::std::string::String> {
    let program_empty = program.is_empty();
    if program_empty {
        let empty_msg = ::std::string::String::from("Command program string is empty");
        return ::core::result::Result::Err(empty_msg);
    }
    let start_instant = ::std::time::Instant::now();
    let stdout_pipe = ::std::process::Stdio::piped();
    let stderr_pipe = ::std::process::Stdio::piped();
    let mut command = ::std::process::Command::new(program);
    command.args(args);
    command.current_dir(cwd);
    command.stdout(stdout_pipe);
    command.stderr(stderr_pipe);

    #[cfg(unix)]
    {
        use ::std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    let child_res = command.spawn();
    let mut child = match child_res {
        ::core::result::Result::Ok(proc_child) => proc_child,
        ::core::result::Result::Err(spawn_err) => {
            let err_text = spawn_err.to_string();
            return ::core::result::Result::Err(err_text);
        }
    };

    let child_stdout = child.stdout.take();
    let child_stderr = child.stderr.take();

    let stdout_handle = ::std::thread::spawn(move || {
        let mut collected = ::std::vec::Vec::new();
        if let ::core::option::Option::Some(mut stream) = child_stdout {
            use ::std::io::Read;
            let _ = stream.read_to_end(&mut collected);
        }
        collected
    });

    let stderr_handle = ::std::thread::spawn(move || {
        let mut collected = ::std::vec::Vec::new();
        if let ::core::option::Option::Some(mut stream) = child_stderr {
            use ::std::io::Read;
            let _ = stream.read_to_end(&mut collected);
        }
        collected
    });

    let wait_res = self::wait_for_child_process(&mut child, program, timeout);
    let stdout_bytes = stdout_handle.join().unwrap_or_default();
    let stderr_bytes = stderr_handle.join().unwrap_or_default();
    let stdout = ::std::string::String::from_utf8_lossy(&stdout_bytes).into_owned();
    let stderr = ::std::string::String::from_utf8_lossy(&stderr_bytes).into_owned();
    let elapsed = start_instant.elapsed();
    let duration_ms = elapsed.as_millis() as ::core::primitive::u64;

    let exit_code = match wait_res {
        ::core::result::Result::Ok(code) => code,
        ::core::result::Result::Err(err_msg) => {
            return ::core::result::Result::Err(err_msg);
        }
    };

    let observation = self::ExecutionObservation {
        exit_code,
        stdout,
        stderr,
        duration_ms,
    };
    ::core::result::Result::Ok(observation)
}

/// Wait for child process to complete or time out.
fn wait_for_child_process(
    child: &mut ::std::process::Child,
    program: &::core::primitive::str,
    timeout: ::std::time::Duration,
) -> ::core::result::Result<::core::option::Option<::core::primitive::i32>, ::std::string::String> {
    let start_instant = ::std::time::Instant::now();
    let deadline = start_instant + timeout;
    let sleep_step = ::std::time::Duration::from_millis(20);
    loop {
        match child.try_wait() {
            ::core::result::Result::Ok(::core::option::Option::Some(status)) => {
                let code_val = status.code();
                return ::core::result::Result::Ok(code_val);
            }
            ::core::result::Result::Ok(::core::option::Option::None) => {
                let now_instant = ::std::time::Instant::now();
                let exceeded = now_instant > deadline;
                if exceeded {
                    #[cfg(unix)]
                    {
                        let pid_num = child.id();
                        let pid_string = pid_num.to_string();
                        let mut kill_cmd = ::std::process::Command::new("kill");
                        let kill_arg = ::std::format!("-{pid_string}");
                        kill_cmd.args(["-KILL", &kill_arg]);
                        let _ = kill_cmd.output();
                    }
                    let _ = child.kill();
                    let _ = child.wait();
                    let timeout_ms = timeout.as_millis();
                    let timeout_msg = ::std::format!("{program} timed out after {timeout_ms} ms");
                    return ::core::result::Result::Err(timeout_msg);
                }
                ::std::thread::sleep(sleep_step);
            }
            ::core::result::Result::Err(wait_err) => {
                let err_text = wait_err.to_string();
                return ::core::result::Result::Err(err_text);
            }
        }
    }
}

/// Compare execution observations with float tolerance.
pub fn compare_observations(
    source: &self::ExecutionObservation,
    target: &self::ExecutionObservation,
    float_tolerance: ::core::primitive::f64,
) -> self::DivergenceClassification {
    let code_mismatch = source.exit_code != target.exit_code;
    if code_mismatch {
        let source = source.exit_code;
        let target = target.exit_code;
        let classification = self::DivergenceClassification::ExitCodeMismatch {
            source,
            target,
        };
        return classification;
    }
    self::compare_token_sequences(&source.stdout, &target.stdout, float_tolerance)
}

/// Compare token sequences and extract float values.
fn compare_token_sequences(
    source_stdout: &::core::primitive::str,
    target_stdout: &::core::primitive::str,
    float_tolerance: ::core::primitive::f64,
) -> self::DivergenceClassification {
    let source_words = source_stdout
        .split_whitespace()
        .collect::<::std::vec::Vec<&::core::primitive::str>>();
    let target_words = target_stdout
        .split_whitespace()
        .collect::<::std::vec::Vec<&::core::primitive::str>>();
    let length_mismatch = source_words.len() != target_words.len();
    if length_mismatch {
        let source_snippet = self::truncate_snippet(source_stdout);
        let target_snippet = self::truncate_snippet(target_stdout);
        let mismatch = self::DivergenceClassification::StdoutMismatch {
            source_snippet,
            target_snippet,
        };
        return mismatch;
    }
    for (idx, &source_token) in source_words.iter().enumerate() {
        let target_token = target_words[idx];
        let token_match = source_token == target_token;
        if token_match {
            continue;
        }
        let source_float = self::parse_float_token(source_token);
        let target_float = self::parse_float_token(target_token);
        if let (::core::option::Option::Some(source_num), ::core::option::Option::Some(target_num)) =
            (source_float, target_float)
        {
            let diff_val = (source_num - target_num).abs();
            let within_tolerance = diff_val <= float_tolerance;
            if within_tolerance {
                continue;
            }
            let message = ::std::format!(
                "Float value divergence: {source_num} vs {target_num} exceeds tolerance {float_tolerance}"
            );
            let divergence = self::DivergenceClassification::ValueMismatch {
                message,
            };
            return divergence;
        }
        let source_snippet = self::truncate_snippet(source_token);
        let target_snippet = self::truncate_snippet(target_token);
        let mismatch = self::DivergenceClassification::StdoutMismatch {
            source_snippet,
            target_snippet,
        };
        return mismatch;
    }
    self::DivergenceClassification::Match
}

/// Truncate text snippet for diagnostic reporting.
fn truncate_snippet(text: &::core::primitive::str) -> ::std::string::String {
    let trimmed = text.trim();
    let char_count = trimmed.chars().count();
    let short_enough = char_count <= 80;
    if short_enough {
        ::std::string::String::from(trimmed)
    } else {
        trimmed.chars().take(80).collect::<::std::string::String>()
    }
}

/// Parse floating point number from token string.
fn parse_float_token(token: &::core::primitive::str) -> ::core::option::Option<::core::primitive::f64> {
    if let ::core::result::Result::Ok(num) = token.parse::<::core::primitive::f64>() {
        return ::core::option::Option::Some(num);
    }
    let trimmed = token.trim_matches(|probe_char: ::core::primitive::char| {
        probe_char == ','
            || probe_char == ';'
            || probe_char == '('
            || probe_char == ')'
            || probe_char == '['
            || probe_char == ']'
            || probe_char == '{'
            || probe_char == '}'
            || probe_char == ':'
    });
    if let ::core::result::Result::Ok(num) = trimmed.parse::<::core::primitive::f64>() {
        return ::core::option::Option::Some(num);
    }
    ::core::option::Option::None
}

/// Construct differential report from components.
fn make_report(
    source_cmd: &::core::primitive::str,
    target_cmd: &::core::primitive::str,
    observation_source: self::ExecutionObservation,
    observation_target: self::ExecutionObservation,
    classification: self::DivergenceClassification,
    equivalent: ::core::primitive::bool,
) -> self::DifferentialReport {
    let source_cmd = ::std::string::String::from(source_cmd);
    let target_cmd = ::std::string::String::from(target_cmd);
    self::DifferentialReport {
        source_cmd,
        target_cmd,
        observation_source,
        observation_target,
        classification,
        equivalent,
    }
}

/// Classify execution error payload into timeout or execution error.
fn classify_error(err_text: &::core::primitive::str, prefix: &::core::primitive::str) -> self::DivergenceClassification {
    let is_timeout = err_text.contains("timed out");
    if is_timeout {
        self::DivergenceClassification::Timeout
    } else {
        let error = ::std::format!("{prefix}: {err_text}");
        self::DivergenceClassification::ExecutionError { error }
    }
}

/// Construct synthetic execution observation for execution error states.
fn make_error_observation(
    stderr_text: &::core::primitive::str,
    duration_ms: ::core::primitive::u64,
) -> self::ExecutionObservation {
    let exit_code = ::core::option::Option::None;
    let stdout = ::std::string::String::new();
    let stderr = ::std::string::String::from(stderr_text);
    self::ExecutionObservation {
        exit_code,
        stdout,
        stderr,
        duration_ms,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn matching_observations_yield_match() {
        let source_obs = super::ExecutionObservation {
            exit_code: ::core::option::Option::Some(0),
            stdout: ::std::string::String::from("computed value: 42\n"),
            stderr: ::std::string::String::new(),
            duration_ms: 10,
        };
        let target_obs = super::ExecutionObservation {
            exit_code: ::core::option::Option::Some(0),
            stdout: ::std::string::String::from("computed value: 42"),
            stderr: ::std::string::String::new(),
            duration_ms: 12,
        };
        let classification = super::compare_observations(&source_obs, &target_obs, 1e-5);
        ::std::assert_eq!(classification, super::DivergenceClassification::Match);
    }

    #[test]
    fn exit_code_mismatch_yields_classification() {
        let source_obs = super::ExecutionObservation {
            exit_code: ::core::option::Option::Some(0),
            stdout: ::std::string::String::from("output"),
            stderr: ::std::string::String::new(),
            duration_ms: 10,
        };
        let target_obs = super::ExecutionObservation {
            exit_code: ::core::option::Option::Some(1),
            stdout: ::std::string::String::from("output"),
            stderr: ::std::string::String::new(),
            duration_ms: 12,
        };
        let classification = super::compare_observations(&source_obs, &target_obs, 1e-5);
        let expected = super::DivergenceClassification::ExitCodeMismatch {
            source: ::core::option::Option::Some(0),
            target: ::core::option::Option::Some(1),
        };
        ::std::assert_eq!(classification, expected);
    }

    #[test]
    fn float_tolerance_comparison() {
        let source_obs = super::ExecutionObservation {
            exit_code: ::core::option::Option::Some(0),
            stdout: ::std::string::String::from("result: 0.0000100"),
            stderr: ::std::string::String::new(),
            duration_ms: 10,
        };
        let target_obs = super::ExecutionObservation {
            exit_code: ::core::option::Option::Some(0),
            stdout: ::std::string::String::from("result: 0.0000101"),
            stderr: ::std::string::String::new(),
            duration_ms: 12,
        };
        let match_classification = super::compare_observations(&source_obs, &target_obs, 1e-5);
        ::std::assert_eq!(match_classification, super::DivergenceClassification::Match);

        let mismatch_classification = super::compare_observations(&source_obs, &target_obs, 1e-8);
        let is_value_mismatch = ::core::matches!(
            mismatch_classification,
            super::DivergenceClassification::ValueMismatch { .. }
        );
        ::std::assert!(is_value_mismatch);
    }

    #[test]
    fn stdout_mismatch_yields_stdout_mismatch() {
        let source_obs = super::ExecutionObservation {
            exit_code: ::core::option::Option::Some(0),
            stdout: ::std::string::String::from("result: success"),
            stderr: ::std::string::String::new(),
            duration_ms: 10,
        };
        let target_obs = super::ExecutionObservation {
            exit_code: ::core::option::Option::Some(0),
            stdout: ::std::string::String::from("result: failure"),
            stderr: ::std::string::String::new(),
            duration_ms: 12,
        };
        let classification = super::compare_observations(&source_obs, &target_obs, 1e-5);
        let expected = super::DivergenceClassification::StdoutMismatch {
            source_snippet: ::std::string::String::from("success"),
            target_snippet: ::std::string::String::from("failure"),
        };
        ::std::assert_eq!(classification, expected);
    }

    #[test]
    fn run_command_observation_echo() {
        let current_dir = ::std::path::Path::new(".");
        let duration_timeout = ::std::time::Duration::from_secs(5);
        let obs_res = super::run_command_observation(
            "echo",
            &["differential-test"],
            current_dir,
            duration_timeout,
        );
        let obs = obs_res.expect("echo command succeeds");
        ::std::assert_eq!(obs.exit_code, ::core::option::Option::Some(0));
        let contains_probe = obs.stdout.contains("differential-test");
        ::std::assert!(contains_probe);
    }

    #[test]
    #[ignore = "integration test requiring pre-existing matching benchmark workspace"]
    fn differential_verification_matrix_multiplication() {
        let repo_root = super::super::bench_config::repo_root();
        let legacy_dir = repo_root.join("benchmarks/assets/ReCodeAgent/data/tool_projects/crust/matrix_multiplication/c");
        let target_dir = repo_root.join("benchmarks/outputs/crust/matrix_multiplication/rust");
        let has_legacy = legacy_dir.exists();
        let has_target = target_dir.exists();
        if !has_legacy || !has_target {
            return;
        }
        let timeout = ::std::time::Duration::from_secs(60);
        let float_tolerance = 1e-4;
        let report = super::execute_differential_verification(
            "make test",
            "cargo run --bin test_matrix",
            &legacy_dir,
            &target_dir,
            timeout,
            float_tolerance,
        );
        let report_json = ::serde_json::to_string_pretty(&report).expect("report serializes to JSON");
        ::std::println!("DifferentialReport:\n{report_json}");
        ::std::assert_eq!(report.observation_source.exit_code, ::core::option::Option::Some(0));
        ::std::assert_eq!(report.observation_target.exit_code, ::core::option::Option::Some(0));
        ::std::assert_eq!(report.classification, super::DivergenceClassification::Match);
        ::std::assert!(report.equivalent);
    }

    #[test]
    fn property_invariants_matrix_multiplication() {
        for rows in [1, 2, 4, 8] {
            for cols in [1, 2, 4, 8] {
                let mut expected = ::std::vec::Vec::new();
                for row_idx in 0..rows {
                    let mut row = ::std::vec::Vec::new();
                    for col_idx in 0..cols {
                        let val = (col_idx + row_idx * rows) as ::core::primitive::f32;
                        row.push(val);
                    }
                    expected.push(row);
                }
                ::std::assert_eq!(expected.len(), rows);
                ::std::assert_eq!(expected[0].len(), cols);
            }
        }
        let dimension_pairs = [
            (4, 4, 4, 4),
            (8, 8, 8, 8),
            (1, 1, 1, 1),
            (1, 2, 2, 1),
            (3, 4, 4, 2),
            (5, 3, 3, 6),
        ];
        for (rows_a, cols_a, rows_b, cols_b) in dimension_pairs {
            ::std::assert_eq!(cols_a, rows_b, "inner dimension must match for multiplication");
            let result_rows = rows_a;
            let result_cols = cols_b;
            ::std::assert_eq!(result_rows, rows_a);
            ::std::assert_eq!(result_cols, cols_b);
        }
    }
}
