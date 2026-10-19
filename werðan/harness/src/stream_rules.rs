//! Stream interception and dormant rule enforcement.

/// Dormant stream rule specifying pattern and reminder injection.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct DormantStreamRule {
    /// Unique identifier for rule.
    pub id: ::std::string::String,
    /// Trigger pattern text that initiates interception.
    pub trigger_pattern: ::std::string::String,
    /// System reminder injection text.
    pub reminder_injection: ::std::string::String,
}

/// Outcome of stream inspection for incoming chunks.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum StreamInterceptOutcome {
    /// Unmodified stream chunk passed through.
    PassThrough(::std::string::String),
    /// Intercepted stream aborting and rewinding with reminder.
    AbortAndRewind {
        /// Text prefix before triggered violation.
        clean_prefix: ::std::string::String,
        /// Injected system reminder text.
        injected_reminder: ::std::string::String,
        /// Identifier of triggered rule.
        rule_id: ::std::string::String,
    },
}

/// Scan token stream for dormant rule violations.
pub fn scan_token_stream(
    accumulated: &::core::primitive::str,
    incoming_chunk: &::core::primitive::str,
    rules: &[self::DormantStreamRule],
) -> self::StreamInterceptOutcome {
    let combined = self::concatenate_stream_slices(accumulated, incoming_chunk);
    if let ::core::option::Option::Some(match_info) = self::find_earliest_violation(&combined, rules) {
        let (pos, rule) = match_info;
        let clean_prefix = self::extract_clean_prefix(&combined, pos);
        let injected_reminder = self::format_system_reminder(&rule.reminder_injection);
        self::StreamInterceptOutcome::AbortAndRewind {
            clean_prefix: { clean_prefix },
            injected_reminder: { injected_reminder },
            rule_id: { rule.id.clone() },
        }
    } else {
        self::StreamInterceptOutcome::PassThrough(incoming_chunk.to_string())
    }
}

fn concatenate_stream_slices(
    accumulated: &::core::primitive::str,
    incoming: &::core::primitive::str,
) -> ::std::string::String {
    let mut combined = ::std::string::String::with_capacity(accumulated.len() + incoming.len());
    combined.push_str(accumulated);
    combined.push_str(incoming);
    combined
}

fn find_earliest_violation<'a>(
    text: &::core::primitive::str,
    rules: &'a [self::DormantStreamRule],
) -> ::core::option::Option<(::core::primitive::usize, &'a self::DormantStreamRule)> {
    let mut earliest: ::core::option::Option<(::core::primitive::usize, &'a self::DormantStreamRule)> =
        ::core::option::Option::None;
    for rule in rules {
        if let ::core::option::Option::Some(pos) = text.find(&rule.trigger_pattern) {
            match earliest {
                ::core::option::Option::Some((min_pos, _)) if pos < min_pos => {
                    earliest = ::core::option::Option::Some((pos, rule));
                },
                ::core::option::Option::None => {
                    earliest = ::core::option::Option::Some((pos, rule));
                },
                _ => {},
            }
        }
    }
    earliest
}

fn extract_clean_prefix(text: &::core::primitive::str, match_pos: ::core::primitive::usize) -> ::std::string::String {
    let prefix = &text[..match_pos];
    if let ::core::option::Option::Some(last_newline) = prefix.rfind('\n') {
        prefix[..=last_newline].to_string()
    } else {
        prefix.to_string()
    }
}

fn format_system_reminder(reminder: &::core::primitive::str) -> ::std::string::String {
    let formatted = ::std::format!("<system-reminder>\n{reminder}\n</system-reminder>");
    formatted
}
