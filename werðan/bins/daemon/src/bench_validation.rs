//! Target workspace verification via the cargo toolchain.
//! Extracts test pass rates and check status for migration campaigns.

/// Target workspace outcome after cargo check and cargo test.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct WorkspaceValidation {
    /// Whether cargo check exited zero.
    pub check_ok: ::core::primitive::bool,
    /// Parsed test pass rate from cargo test.
    pub test_pass_rate: ::core::primitive::f64,
}

/// Run cargo check and cargo test in the target workspace and parse outcomes.
pub fn validate_workspace(
    bench_config: &super::bench_config::BenchConfig,
    workspace_root: &::std::path::Path,
) -> self::WorkspaceValidation {
    let build_timeout = bench_config.inference.build_timeout_secs;
    let test_timeout = bench_config.inference.test_timeout_secs;
    ::tracing::info!(
        "campaign {}: build timeout {}s, test timeout {}s, inference timeout {}s",
        bench_config.campaign_id,
        build_timeout,
        test_timeout,
        bench_config.inference.inference_timeout_secs
    );
    let target_source_file = ::std::path::PathBuf::from("src/lib.rs");
    let target_workspace = workspace_root.to_path_buf();
    let harness = ::holon_evaluator::cargo_harness::CargoHarness::with_timeouts(
        target_workspace,
        target_source_file,
        build_timeout,
        test_timeout,
    );
    let check_res = harness.run_command("cargo", &["check"], build_timeout);
    let check_ok = match check_res {
        ::core::result::Result::Ok(output) => {
            let code = output.status.code();
            code == ::core::option::Option::Some(0)
        },
        ::core::result::Result::Err(_) => false,
    };
    let test_res = harness.run_command("cargo", &["test"], test_timeout);
    let test_pass_rate = match test_res {
        ::core::result::Result::Ok(output) => {
            let stdout_text = ::std::string::String::from_utf8_lossy(&output.stdout);
            let stderr_text = ::std::string::String::from_utf8_lossy(&output.stderr);
            let mut combined = stdout_text.into_owned();
            combined.push('\n');
            combined.push_str(&stderr_text);
            let counts = self::parse_test_counts(&combined);
            let passed = counts.0;
            let failed = counts.1;
            self::compute_pass_rate(passed, failed)
        },
        ::core::result::Result::Err(_) => 0.0,
    };
    self::WorkspaceValidation {
        check_ok,
        test_pass_rate,
    }
}

/// Parse the last test result line from cargo test output.
pub fn parse_test_counts(output_text: &::core::primitive::str) -> (::core::primitive::u64, ::core::primitive::u64) {
    let mut passed_total = 0;
    let mut failed_total = 0;
    for line in output_text.lines() {
        let trimmed = line.trim();
        let is_result_line = trimmed.starts_with("test result: ");
        if !is_result_line {
            continue;
        }
        let passed = self::extract_count_after(trimmed, "passed");
        let failed = self::extract_count_after(trimmed, "failed");
        passed_total += passed;
        failed_total += failed;
    }
    (passed_total, failed_total)
}

/// Extract the numeric token preceding one keyword in a cargo test result line.
pub fn extract_count_after(
    line: &::core::primitive::str,
    keyword: &::core::primitive::str,
) -> ::core::primitive::u64 {
    let keyword_token = ::std::format!(" {keyword}");
    let marker = match line.rfind(&keyword_token) {
        ::core::option::Option::Some(found) => found,
        ::core::option::Option::None => return 0,
    };
    let prefix = &line[..marker];
    let number = self::trailing_number(prefix);
    let empty_number = number.is_empty();
    if empty_number {
        return 0;
    }
    number.parse::<::core::primitive::u64>().unwrap_or(0)
}

/// Extract the trailing digit run from one prefix segment.
pub fn trailing_number(prefix: &::core::primitive::str) -> &::core::primitive::str {
    let mut start_idx = prefix.len();
    for (offset, digit_probe) in prefix.char_indices().rev() {
        let is_digit = digit_probe.is_ascii_digit();
        if !is_digit {
            break;
        }
        start_idx = offset;
    }
    &prefix[start_idx..]
}

/// Compute the pass rate from parsed cargo test counts.
pub fn compute_pass_rate(passed: ::core::primitive::u64, failed: ::core::primitive::u64) -> ::core::primitive::f64 {
    let denominator = passed + failed;
    let no_tests = denominator == 0;
    if no_tests {
        return 0.0;
    }
    (passed as ::core::primitive::f64) / (denominator as ::core::primitive::f64)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_counts_from_mixed_suites() {
        let output = "test result: ok. 0 passed\x3B 0 failed\x3B 0 ignored\n\
                      test result: FAILED. 20 passed\x3B 2 failed\x3B 0 ignored\n";
        let counts = super::parse_test_counts(output);
        let passed = counts.0;
        let failed = counts.1;
        ::std::assert_eq!(passed, 20);
        ::std::assert_eq!(failed, 2);
        let rate = super::compute_pass_rate(passed, failed);
        let diff = (rate - 20.0 / 22.0).abs();
        ::std::assert!(diff < 1e-9, "rate was {rate}");
    }

    #[test]
    fn zero_tests_yield_zero_rate() {
        let rate = super::compute_pass_rate(0, 0);
        ::std::assert_eq!(rate, 0.0);
    }
}
