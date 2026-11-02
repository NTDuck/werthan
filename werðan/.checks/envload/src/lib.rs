//! Compile-time environment ingestion for every workspace build script.
//!
//! Call `::envload::load()` as the first statement in `build.rs`. The helper
//! walks up from `CARGO_MANIFEST_DIR` to the filesystem root, reads the first
//! `.env` it finds, and emits one generic `cargo:rustc-env=KEY=VALUE` line per
//! parsed key-value pair. No key list lives here: adding a pair to `.env`
//! reaches `env!("KEY")` call sites without any build-script change.

/// Ingest the nearest ancestor `.env` as compile-time environment variables.
///
/// Emits `cargo:rerun-if-changed` for the resolved file so edits invalidate
/// the build cache, then one `cargo:rustc-env` emission per parsed pair.
/// Missing files and malformed lines stay silent: build scripts must not
/// fail the tree over optional environment defaults.
pub fn load() {
    let env_path = match find_env_file() {
        ::core::option::Option::Some(found_path) => found_path,
        ::core::option::Option::None => return,
    };
    ::std::println!("cargo:rerun-if-changed={}", env_path.display());
    let read_res = ::std::fs::read_to_string(&env_path);
    let content = match read_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(_) => return,
    };
    for pair in parse_pairs(&content) {
        ::std::println!("cargo:rustc-env={}", pair);
    }
}

/// Walk up from `CARGO_MANIFEST_DIR` to the root and return the first `.env`.
fn find_env_file() -> ::core::option::Option<::std::path::PathBuf> {
    let manifest_dir = ::std::env::var("CARGO_MANIFEST_DIR").ok()?;
    let mut current = ::std::path::PathBuf::from(manifest_dir);
    loop {
        let candidate = current.join(".env");
        if candidate.is_file() {
            return ::core::option::Option::Some(candidate);
        }
        if !current.pop() {
            return ::core::option::Option::None;
        }
    }
}

/// Reduce `.env` text to one `KEY=VALUE` string per valid line.
///
/// Blank lines and `#` comments are skipped. Values lose one layer of
/// surrounding double quotes; everything else passes through verbatim.
fn parse_pairs(content: &::core::primitive::str) -> ::std::vec::Vec<::std::string::String> {
    let mut pairs = ::std::vec::Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let line_empty = trimmed.is_empty();
        let line_comment = trimmed.starts_with('#');
        if line_empty || line_comment {
            continue;
        }
        let split_res = trimmed.split_once('=');
        if let ::core::option::Option::Some((key, value)) = split_res {
            let trimmed_key = key.trim();
            let trimmed_val = value.trim().trim_matches('"');
            pairs.push(::std::format!("{trimmed_key}={trimmed_val}"));
        }
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pairs_skips_comments_and_blanks() {
        let content = "# header comment\n\nBASE_URL=\"http://127.0.0.1:8092/v1\"\nTOOL_GLOB_LIMIT=100\n";
        let pairs = self::parse_pairs(content);
        ::std::assert_eq!(pairs.len(), 2);
        ::std::assert!(pairs[0].starts_with("BASE_URL=http://127.0.0.1:8092/v1"));
        ::std::assert_eq!(pairs[1], "TOOL_GLOB_LIMIT=100");
    }

    #[test]
    fn parse_pairs_keeps_inline_text_verbatim() {
        let content = "MODEL_ID=swift-1.5-qwen3.8-27b\n";
        let pairs = self::parse_pairs(content);
        ::std::assert_eq!(pairs[0], "MODEL_ID=swift-1.5-qwen3.8-27b");
    }
}
