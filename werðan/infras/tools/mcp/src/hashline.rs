//! Hashline content addressing and anchor replacement utilities.

const BASE62_ALPHABET: &[::core::primitive::u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Replace line range between anchors with replacement text.
pub fn apply_hashline_replace(
    original: &::core::primitive::str,
    start_anchor: &::core::primitive::str,
    end_anchor: &::core::primitive::str,
    replacement: &::core::primitive::str,
) -> ::core::result::Result<::std::string::String, ::std::string::String> {
    let (lines, has_trailing_newline) = self::split_lines(original);
    let start_indices = self::find_matching_anchor_indices(&lines, start_anchor);
    let start_idx = self::validate_anchor_matches(&start_indices)?;
    let end_indices = self::find_matching_anchor_indices(&lines, end_anchor);
    let end_idx = self::validate_anchor_matches(&end_indices)?;
    if start_idx > end_idx {
        let err = "HASHLINE_STALE_ANCHOR".to_string();
        return ::core::result::Result::Err(err);
    }
    let res = self::assemble_replacement(&lines, start_idx, end_idx, replacement, has_trailing_newline);
    ::core::result::Result::Ok(res)
}

/// Render line content prefixed by computed anchors.
pub fn render_hashlined_view(
    content: &::core::primitive::str,
    start_line: ::core::primitive::usize,
    max_lines: ::core::primitive::usize,
) -> ::std::string::String {
    let (lines, _) = self::split_lines(content);
    let mut rendered = ::std::string::String::new();
    let total = lines.len();
    if start_line >= total || max_lines == 0 {
        return rendered;
    }
    let end_line = ::core::cmp::min(total, start_line + max_lines);
    for idx in start_line..end_line {
        let anchor = self::compute_line_anchor(&lines, idx);
        rendered.push_str(&anchor);
        rendered.push('\u{2502}');
        rendered.push(' ');
        rendered.push_str(lines[idx]);
        rendered.push('\n');
    }
    rendered
}

fn split_lines(
    content: &::core::primitive::str,
) -> (::std::vec::Vec<&::core::primitive::str>, ::core::primitive::bool) {
    if content.is_empty() {
        let empty_vec = ::std::vec::Vec::new();
        return (empty_vec, false);
    }
    let has_trailing = content.ends_with('\n');
    let mut lines = content.split('\n').collect::<::std::vec::Vec<&::core::primitive::str>>();
    if has_trailing && lines.last() == ::core::option::Option::Some(&"") {
        lines.pop();
    }
    (lines, has_trailing)
}

fn find_matching_anchor_indices(
    lines: &[&::core::primitive::str],
    target_anchor: &::core::primitive::str,
) -> ::std::vec::Vec<::core::primitive::usize> {
    let mut matches = ::std::vec::Vec::new();
    for idx in 0..lines.len() {
        let anchor = self::compute_line_anchor(lines, idx);
        if anchor == target_anchor {
            matches.push(idx);
        }
    }
    matches
}

/// Calculate line anchor incorporating neighbor lines.
pub fn compute_line_anchor(lines: &[&::core::primitive::str], idx: ::core::primitive::usize) -> ::std::string::String {
    let prev = if idx > 0 {
        lines[idx - 1].trim()
    } else {
        ""
    };
    let curr = lines[idx];
    let next = if idx + 1 < lines.len() {
        lines[idx + 1].trim()
    } else {
        ""
    };
    self::digest_neighbor_triplet(prev, curr, next)
}

fn validate_anchor_matches(
    matches: &[::core::primitive::usize],
) -> ::core::result::Result<::core::primitive::usize, ::std::string::String> {
    if matches.is_empty() {
        let err = "HASHLINE_STALE_ANCHOR".to_string();
        ::core::result::Result::Err(err)
    } else if matches.len() > 1 {
        let err = "HASHLINE_AMBIGUOUS".to_string();
        ::core::result::Result::Err(err)
    } else {
        ::core::result::Result::Ok(matches[0])
    }
}

fn assemble_replacement(
    lines: &[&::core::primitive::str],
    start_idx: ::core::primitive::usize,
    end_idx: ::core::primitive::usize,
    replacement: &::core::primitive::str,
    has_trailing_newline: ::core::primitive::bool,
) -> ::std::string::String {
    let mut output = ::std::string::String::new();
    for idx in 0..start_idx {
        output.push_str(lines[idx]);
        output.push('\n');
    }
    if !replacement.is_empty() {
        output.push_str(replacement);
        if !replacement.ends_with('\n') {
            output.push('\n');
        }
    }
    for idx in (end_idx + 1)..lines.len() {
        output.push_str(lines[idx]);
        output.push('\n');
    }
    if !has_trailing_newline && output.ends_with('\n') {
        output.pop();
    }
    output
}

fn digest_neighbor_triplet(
    prev: &::core::primitive::str,
    curr: &::core::primitive::str,
    next: &::core::primitive::str,
) -> ::std::string::String {
    use ::sha2::Digest;
    let mut hasher = ::sha2::Sha256::new();
    hasher.update(prev.as_bytes());
    hasher.update(b"\0");
    hasher.update(curr.as_bytes());
    hasher.update(b"\0");
    hasher.update(next.as_bytes());
    let hash = hasher.finalize();
    self::encode_base62_prefix(&hash)
}

fn encode_base62_prefix(hash: &[::core::primitive::u8]) -> ::std::string::String {
    let b0 = hash[0] as ::core::primitive::u64;
    let b1 = hash[1] as ::core::primitive::u64;
    let b2 = hash[2] as ::core::primitive::u64;
    let b3 = hash[3] as ::core::primitive::u64;
    let mut num = (b0 << 24) | (b1 << 16) | (b2 << 8) | b3;
    let mut chars = ['0'; 4];
    for pos in (0..4).rev() {
        let rem = (num % 62) as ::core::primitive::usize;
        num /= 62;
        chars[pos] = self::BASE62_ALPHABET[rem] as ::core::primitive::char;
    }
    let s = chars.iter().collect::<::std::string::String>();
    s
}
