//! Clean-room test runner, JUnit XML parser, receipt signer, and benchmark runners.

/// Benchmark execution and output parsing helpers (absorbed from daemon).
pub mod runners;

/// Verification result receipt stamp.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum VerificationReceipt {
    /// Full semantic equivalence proven.
    Proven,
    /// Partial equivalence with known deltas.
    PartlyProven,
    /// Verification failed.
    Failed,
}

impl ::core::fmt::Display for self::VerificationReceipt {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let label = match self {
            Self::Proven => "PROVEN",
            Self::PartlyProven => "PARTLY PROVEN",
            Self::Failed => "FAILED",
        };
        ::core::write!(f, "{label}")
    }
}

/// Verification error conditions.
#[derive(::core::fmt::Debug)]
pub enum VerificationError {
    /// Zero tests executed in test harness.
    ZeroTestsExecuted,
    /// Build command failure.
    BuildFailed(::std::string::String),
    /// Test execution failure.
    TestFailed(::std::string::String),
    /// XML parse failure.
    XmlParseError(::std::string::String),
}

impl ::core::fmt::Display for self::VerificationError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::ZeroTestsExecuted => ::core::write!(f, "Verification failed: harness executed zero tests"),
            Self::BuildFailed(msg) => ::core::write!(f, "Build failed: {msg}"),
            Self::TestFailed(msg) => ::core::write!(f, "Test execution failed: {msg}"),
            Self::XmlParseError(msg) => ::core::write!(f, "XML parsing error: {msg}"),
        }
    }
}

impl ::std::error::Error for self::VerificationError {}

/// Parse JUnit XML test report and return total test count and failure count.
pub fn parse_junit_xml(
    xml: &::core::primitive::str,
) -> ::core::result::Result<(::core::primitive::usize, ::core::primitive::usize), self::VerificationError> {
    let mut reader = ::quick_xml::Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = ::std::vec::Vec::new();
    let mut total_tests: ::core::primitive::usize = 0;
    let mut failures: ::core::primitive::usize = 0;

    loop {
        let event = reader.read_event_into(&mut buf);
        match &event {
            ::core::result::Result::Ok(::quick_xml::events::Event::Empty(e))
            | ::core::result::Result::Ok(::quick_xml::events::Event::Start(e)) => {
                let name = e.name();
                if name.as_ref() == b"testsuite" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"tests" {
                            let val_str = ::std::string::String::from_utf8_lossy(&attr.value);
                            if let ::core::result::Result::Ok(n) = val_str.parse::<::core::primitive::usize>() {
                                total_tests += n;
                            }
                        } else if attr.key.as_ref() == b"failures" || attr.key.as_ref() == b"errors" {
                            let val_str = ::std::string::String::from_utf8_lossy(&attr.value);
                            if let ::core::result::Result::Ok(n) = val_str.parse::<::core::primitive::usize>() {
                                failures += n;
                            }
                        }
                    }
                }
            },
            ::core::result::Result::Ok(::quick_xml::events::Event::Eof) => break,
            ::core::result::Result::Err(e) => {
                return ::core::result::Result::Err(self::VerificationError::XmlParseError(e.to_string()));
            },
            _ => {},
        }
        buf.clear();
    }

    if total_tests == 0 {
        return ::core::result::Result::Err(self::VerificationError::ZeroTestsExecuted);
    }

    ::core::result::Result::Ok((total_tests, failures))
}

/// Compare outputs using similar diff and emit verification receipt.
pub fn compare_outputs(
    legacy_output: &::core::primitive::str,
    modern_output: &::core::primitive::str,
) -> self::VerificationReceipt {
    if legacy_output == modern_output {
        return self::VerificationReceipt::Proven;
    }

    let diff = ::similar::TextDiff::from_lines(legacy_output, modern_output);
    let ratio = diff.ratio();

    if ratio > 0.85 {
        self::VerificationReceipt::PartlyProven
    } else {
        self::VerificationReceipt::Failed
    }
}
