//! Magic directive extraction from prompt text (absorbed keywords crate).
//! Exposed as protocol::directives.

/// Directives extracted from prompt text.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct MagicDirectives {
    /// Flag indicating ultrathink reasoning mode.
    pub ultrathink: ::core::primitive::bool,
    /// Flag indicating orchestrate execution mode.
    pub orchestrate: ::core::primitive::bool,
    /// Flag indicating workflowz execution mode.
    pub workflowz: ::core::primitive::bool,
    /// Cleaned prompt text without directive tokens.
    pub cleaned_prompt: ::std::string::String,
}

/// Extract magic directives from raw prompt text.
pub fn extract_magic_directives(raw_prompt: &::core::primitive::str) -> self::MagicDirectives {
    let mut ultrathink = false;
    let mut orchestrate = false;
    let mut workflowz = false;
    let mut cleaned_lines = ::std::vec::Vec::new();

    for line in raw_prompt.lines() {
        let (kept_line, has_u, has_o, has_w) = self::process_prompt_line(line);
        if has_u {
            ultrathink = true;
        }
        if has_o {
            orchestrate = true;
        }
        if has_w {
            workflowz = true;
        }
        if let ::core::option::Option::Some(line_text) = kept_line {
            cleaned_lines.push(line_text);
        }
    }

    let cleaned_prompt = self::join_cleaned_lines(&cleaned_lines, raw_prompt.ends_with('\n'));
    MagicDirectives {
        ultrathink: { ultrathink },
        orchestrate: { orchestrate },
        workflowz: { workflowz },
        cleaned_prompt: { cleaned_prompt },
    }
}

fn is_punctuation_char(ch: ::core::primitive::char) -> ::core::primitive::bool {
    !ch.is_alphanumeric()
}

fn process_prompt_line(
    line: &::core::primitive::str,
) -> (
    ::core::option::Option<::std::string::String>,
    ::core::primitive::bool,
    ::core::primitive::bool,
    ::core::primitive::bool,
) {
    let mut has_u = false;
    let mut has_o = false;
    let mut has_w = false;
    let mut remaining_words = ::std::vec::Vec::new();

    for word in line.split_whitespace() {
        let normalized = word.trim_matches(self::is_punctuation_char);
        if normalized.eq_ignore_ascii_case("ultrathink") {
            has_u = true;
        } else if normalized.eq_ignore_ascii_case("orchestrate") {
            has_o = true;
        } else if normalized.eq_ignore_ascii_case("workflowz") {
            has_w = true;
        } else {
            remaining_words.push(word);
        }
    }

    if remaining_words.is_empty() && (has_u || has_o || has_w) {
        (::core::option::Option::None, has_u, has_o, has_w)
    } else if has_u || has_o || has_w {
        let rebuilt = remaining_words.join(" ");
        (::core::option::Option::Some(rebuilt), has_u, has_o, has_w)
    } else {
        let kept = line.to_string();
        (::core::option::Option::Some(kept), false, false, false)
    }
}

fn join_cleaned_lines(
    lines: &[::std::string::String],
    trailing_newline: ::core::primitive::bool,
) -> ::std::string::String {
    let mut out = lines.join("\n");
    if trailing_newline && !out.is_empty() {
        out.push('\n');
    }
    out
}
