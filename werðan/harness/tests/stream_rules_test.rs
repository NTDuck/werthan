//! Tests for stream-rules crate.

#[::core::prelude::v1::test]
fn test_scan_token_stream_passthrough() {
    let rules = ::std::vec![::harness::stream_rules::DormantStreamRule {
        id: { "no_todo".to_string() },
        trigger_pattern: { "todo!".to_string() },
        reminder_injection: { "Do not use todo!".to_string() },
    }];

    let outcome = ::harness::stream_rules::scan_token_stream("fn test() {", " let x = 5", &rules);
    match outcome {
        ::harness::stream_rules::StreamInterceptOutcome::PassThrough(chunk) => {
            ::core::assert_eq!(chunk, " let x = 5");
        },
        _ => ::core::panic!("Expected PassThrough"),
    }
}

#[::core::prelude::v1::test]
fn test_scan_token_stream_abort_and_rewind() {
    let rules = ::std::vec![::harness::stream_rules::DormantStreamRule {
        id: { "no_unimplemented".to_string() },
        trigger_pattern: { "unimplemented!".to_string() },
        reminder_injection: { "Implement completely".to_string() },
    }];

    let outcome = ::harness::stream_rules::scan_token_stream("fn test() {\n", "    unimplemented!()\n}", &rules);
    match outcome {
        ::harness::stream_rules::StreamInterceptOutcome::AbortAndRewind {
            clean_prefix,
            injected_reminder,
            rule_id,
        } => {
            ::core::assert_eq!(clean_prefix, "fn test() {\n");
            ::core::assert!(injected_reminder.contains("<system-reminder>"));
            ::core::assert!(injected_reminder.contains("Implement completely"));
            ::core::assert_eq!(rule_id, "no_unimplemented");
        },
        _ => ::core::panic!("Expected AbortAndRewind"),
    }
}
