//! Unit tests for verify.

#[test]
fn test_verify_junit_xml_and_zero_guard() {
    let empty_xml = "<testsuites tests=\"0\" failures=\"0\"></testsuites>";
    let err = ::mcp::verify::parse_junit_xml(empty_xml).unwrap_err();
    match err {
        ::mcp::verify::VerificationError::ZeroTestsExecuted => {},
        _ => ::core::panic!("Expected ZeroTestsExecuted error"),
    }

    let valid_xml = "<testsuites tests=\"5\" failures=\"1\"><testsuite tests=\"5\" failures=\"1\"/></testsuites>";
    let (total, fails) = ::mcp::verify::parse_junit_xml(valid_xml).unwrap();
    ::std::assert_eq!(total, 5);
    ::std::assert_eq!(fails, 1);
}

#[test]
fn test_verify_diff_similar() {
    let identical = ::mcp::verify::compare_outputs("alpha\nbeta\n", "alpha\nbeta\n");
    ::std::assert_eq!(identical, ::mcp::verify::VerificationReceipt::Proven);

    let divergent = ::mcp::verify::compare_outputs("alpha\nbeta\n", "xyz\n123\n");
    ::std::assert_eq!(divergent, ::mcp::verify::VerificationReceipt::Failed);
}
