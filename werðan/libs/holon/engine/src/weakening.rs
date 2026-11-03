//! Test weakening detection preventing assertion regression or deletion during repair.
//!
//! Ported from MAgHARCM validator architecture (NEW-PRIM-13 / GAP-01).

static ASSERTION_PATTERN: ::std::sync::LazyLock<::regex::Regex> =
    ::std::sync::LazyLock::new(|| {
        ::regex::Regex::new(r"(?i)\b(?:assert|debug_assert|expect|should|require)\w*\s*[\(!]")
            .expect("valid assertion regex")
    });

/// Detect test suite weakening across successive repair iterations.
///
/// Verifies that test cases are not deleted and assertion counts do not regress.
pub fn detect_test_weakening(
    prev_tests: &[::std::string::String],
    current_tests: &[::std::string::String],
) -> ::core::result::Result<(), ::std::string::String> {
    if prev_tests.is_empty() {
        return ::core::result::Result::Ok(());
    }

    if current_tests.len() < prev_tests.len() {
        return ::core::result::Result::Err(::std::format!(
            "test cases deleted: suite size reduced from {} to {}",
            prev_tests.len(),
            current_tests.len(),
        ));
    }

    let mut prev_assertions: ::core::primitive::usize = 0;
    for prev_test_case in prev_tests {
        let code_slice = prev_test_case.as_str();
        let assertion_count = self::count_assertions(code_slice);
        prev_assertions += assertion_count;
    }

    let mut curr_assertions: ::core::primitive::usize = 0;
    for curr_test_case in current_tests {
        let code_slice = curr_test_case.as_str();
        let assertion_count = self::count_assertions(code_slice);
        curr_assertions += assertion_count;
    }

    if curr_assertions < prev_assertions {
        return ::core::result::Result::Err(::std::format!(
            "assertion count regressed: suite assertions reduced from {} to {}",
            prev_assertions,
            curr_assertions,
        ));
    }

    let mut current_named_map = ::std::collections::BTreeMap::new();
    for current_case in current_tests {
        let current_case_str = current_case.as_str();
        if let ::core::option::Option::Some(test_name) = self::extract_fn_name(current_case_str) {
            current_named_map.insert(test_name, current_case_str);
        }
    }

    for prev_case in prev_tests {
        let prev_case_str = prev_case.as_str();
        if let ::core::option::Option::Some(test_name) = self::extract_fn_name(prev_case_str) {
            match current_named_map.get(test_name.as_str()) {
                ::core::option::Option::None => {
                    return ::core::result::Result::Err(::std::format!(
                        "suite deleted test case `{}`",
                        test_name,
                    ));
                }
                ::core::option::Option::Some(matched_current_code) => {
                    let prev_count = self::count_assertions(prev_case_str);
                    let curr_count = self::count_assertions(matched_current_code);
                    if curr_count < prev_count {
                        return ::core::result::Result::Err(::std::format!(
                            "test case `{}` assertions reduced from {} to {}",
                            test_name,
                            prev_count,
                            curr_count,
                        ));
                    }
                }
            }
        }
    }

    ::core::result::Result::Ok(())
}

/// Count assertion calls in test source code.
///
/// Matches assertions such as `assert!`, `assert_eq!`, `assert_ne!`,
/// `debug_assert!`, `debug_assert_eq!`, `expect!`, `expect(`, and `require!`.
pub fn count_assertions(code: &::core::primitive::str) -> ::core::primitive::usize {
    ASSERTION_PATTERN.find_iter(code).count()
}

/// Extract test function identifier from source text if available.
fn extract_fn_name(
    source_text: &::core::primitive::str,
) -> ::core::option::Option<::std::string::String> {
    for line_text in source_text.lines() {
        let trimmed_line = line_text.trim();
        if let ::core::option::Option::Some(signature_tail) = trimmed_line.strip_prefix("fn ") {
            if let ::core::option::Option::Some(raw_name) = signature_tail.split('(').next() {
                let function_name = raw_name.trim();
                if !function_name.is_empty() {
                    return ::core::option::Option::Some(function_name.to_string());
                }
            }
        }
    }
    ::core::option::Option::None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_assertions_various_macros() {
        let code = ::core::concat!(
            "fn test_example() {\n",
            "    assert!(1 == 1)\n",
            "    assert_eq!(foo, bar)\n",
            "    assert_ne!(baz, qux)\n",
            "    debug_assert!(is_valid)\n",
            "    debug_assert_eq!(count, 10)\n",
            "    expect(result).to_equal(10)\n",
            "    require!(status.is_ok())\n",
            "}\n"
        );
        let counted_assertions = self::count_assertions(code);
        ::std::assert_eq!(counted_assertions, 7);
    }

    #[test]
    fn test_count_assertions_no_assertions() {
        let code = ::core::concat!(
            "fn helper_function() {\n",
            "    let assertion_result = true\n",
            "    let unasserted = 42\n",
            "}\n"
        );
        let counted_assertions = self::count_assertions(code);
        ::std::assert_eq!(counted_assertions, 0);
    }

    #[test]
    fn test_detect_weakening_empty_prev_tests_ok() {
        let prev = ::std::vec::Vec::<::std::string::String>::new();
        let curr = ::std::vec![::std::string::String::from(
            "fn test_one() { assert!(true) }"
        )];
        let check_outcome = self::detect_test_weakening(&prev, &curr);
        ::std::assert!(check_outcome.is_ok());
    }

    #[test]
    fn test_detect_weakening_test_case_deleted() {
        let prev = ::std::vec![
            ::std::string::String::from("fn test_one() { assert!(true) }"),
            ::std::string::String::from("fn test_two() { assert!(true) }"),
        ];
        let curr = ::std::vec![::std::string::String::from(
            "fn test_one() { assert!(true) assert!(true) }"
        )];
        let check_outcome = self::detect_test_weakening(&prev, &curr);
        ::std::assert!(check_outcome.is_err());
        let error_message = check_outcome.unwrap_err();
        ::std::assert!(error_message.contains("test cases deleted"));
    }

    #[test]
    fn test_detect_weakening_total_assertions_regressed() {
        let prev = ::std::vec![
            ::std::string::String::from("assert!(true) assert!(true)"),
        ];
        let curr = ::std::vec![
            ::std::string::String::from("assert!(true)"),
        ];
        let check_outcome = self::detect_test_weakening(&prev, &curr);
        ::std::assert!(check_outcome.is_err());
        let error_message = check_outcome.unwrap_err();
        ::std::assert!(error_message.contains("assertion count regressed"));
    }

    #[test]
    fn test_detect_weakening_named_test_deleted() {
        let prev = ::std::vec![
            ::std::string::String::from("fn test_alpha() { assert!(true) }"),
        ];
        let curr = ::std::vec![
            ::std::string::String::from("fn test_beta() { assert!(true) }"),
        ];
        let check_outcome = self::detect_test_weakening(&prev, &curr);
        ::std::assert!(check_outcome.is_err());
        let error_message = check_outcome.unwrap_err();
        ::std::assert!(error_message.contains("suite deleted test case `test_alpha`"));
    }

    #[test]
    fn test_detect_weakening_named_test_assertions_reduced() {
        let prev = ::std::vec![
            ::std::string::String::from("fn test_alpha() { assert!(true) assert_eq!(1, 1) }"),
            ::std::string::String::from("fn test_beta() { assert!(true) }"),
        ];
        let curr = ::std::vec![
            ::std::string::String::from("fn test_alpha() { assert!(true) }"),
            ::std::string::String::from("fn test_beta() { assert!(true) assert!(true) }"),
        ];
        let check_outcome = self::detect_test_weakening(&prev, &curr);
        ::std::assert!(check_outcome.is_err());
        let error_message = check_outcome.unwrap_err();
        ::std::assert!(error_message.contains("test case `test_alpha` assertions reduced"));
    }

    #[test]
    fn test_detect_weakening_valid_progression_ok() {
        let prev = ::std::vec![
            ::std::string::String::from("fn test_alpha() { assert!(true) }"),
        ];
        let curr = ::std::vec![
            ::std::string::String::from("fn test_alpha() { assert!(true) assert_eq!(2, 2) }"),
            ::std::string::String::from("fn test_beta() { assert!(true) }"),
        ];
        let check_outcome = self::detect_test_weakening(&prev, &curr);
        ::std::assert!(check_outcome.is_ok());
    }
}
