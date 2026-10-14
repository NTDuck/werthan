/// Enforce simplified technical English according to ASD-STE100.
///
/// Call this function in the build.rs file of the target crate:
/// ```no_run
/// ::ste100::check();
/// ```
pub fn check() {
    check_manifest();
}

fn check_manifest() {
    let manifest_dir = match ::std::env::var("CARGO_MANIFEST_DIR") {
        ::core::result::Result::Ok(d) => d,
        ::core::result::Result::Err(_) => return,
    };
    let root = ::std::path::PathBuf::from(manifest_dir);
    let mut files = ::std::vec::Vec::new();
    for dir in ["src", "tests", "examples", "benches"] {
        collect_rs_files(&root.join(dir), &mut files);
    }
    files.sort();
    for file in &files {
        check_file(file);
    }
}

fn collect_rs_files(dir: &::std::path::Path, out: &mut ::std::vec::Vec<::std::path::PathBuf>) {
    let entries = match ::std::fs::read_dir(dir) {
        ::core::result::Result::Ok(e) => e,
        ::core::result::Result::Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn check_file(path: &::std::path::Path) {
    ::std::println!("cargo:rerun-if-changed={}", path.display());
    if let ::core::option::Option::Some(script) = find_ste_lint_script() {
        ::std::println!("cargo:rerun-if-changed={}", script.display());
    }
    let source = match ::std::fs::read_to_string(path) {
        ::core::result::Result::Ok(s) => s,
        ::core::result::Result::Err(_) => return,
    };
    for warn in check_source(&source, path) {
        ::std::println!("{warn}");
    }
}

#[derive(::serde::Deserialize)]
struct SteLintViolation {
    line: ::core::primitive::usize,
    col: ::core::primitive::usize,
    rule: ::std::string::String,
    message: ::std::string::String,
}

#[derive(::serde::Deserialize)]
struct SteLintOutput {
    violations: ::std::vec::Vec<self::SteLintViolation>,
}

pub fn check_source(
    source: &::core::primitive::str,
    path: &::std::path::Path,
) -> ::std::vec::Vec<::std::string::String> {
    let script = match find_ste_lint_script() {
        ::core::option::Option::Some(s) => s,
        ::core::option::Option::None => {
            return ::std::vec![::std::format!(
                "cargo:warning={}: ste100 skipped: ste-lint.py not found in submodule",
                path.display()
            )];
        },
    };
    let python = match find_python() {
        ::core::option::Option::Some(p) => p,
        ::core::option::Option::None => {
            return ::std::vec![::std::format!(
                "cargo:warning={}: ste100 skipped: python3 or python interpreter not found on PATH",
                path.display()
            )];
        },
    };

    let masked = mask_source(source);
    let violations = match run_ste_lint(&python, &script, &masked) {
        ::core::result::Result::Ok(v) => v,
        ::core::result::Result::Err(err) => {
            return ::std::vec![::std::format!("cargo:warning={}: ste100 execution error: {err}", path.display())];
        },
    };

    let mut warnings = ::std::vec::Vec::new();
    for v in violations {
        warnings.push(::std::format!(
            "cargo:warning={}:{}:{}: ste100 rule violation ({}): {}",
            path.display(),
            v.line,
            v.col,
            v.rule,
            v.message
        ));
    }
    warnings
}

fn find_ste_lint_script() -> ::core::option::Option<::std::path::PathBuf> {
    let script = ::std::path::Path::new(::core::env!("CARGO_MANIFEST_DIR"))
        .join("references/asd-ste100-skill/scripts/ste-lint.py");
    if script.exists() {
        ::core::option::Option::Some(script)
    } else {
        ::core::option::Option::None
    }
}

fn find_python() -> ::core::option::Option<::std::path::PathBuf> {
    if let ::core::result::Result::Ok(output) = ::std::process::Command::new("python3").arg("--version").output() {
        if output.status.success() {
            return ::core::option::Option::Some(::std::path::PathBuf::from("python3"));
        }
    }
    if let ::core::result::Result::Ok(output) = ::std::process::Command::new("python").arg("--version").output() {
        if output.status.success() {
            return ::core::option::Option::Some(::std::path::PathBuf::from("python"));
        }
    }
    ::core::option::Option::None
}

fn run_ste_lint(
    python: &::std::path::Path,
    script: &::std::path::Path,
    input: &::core::primitive::str,
) -> ::core::result::Result<::std::vec::Vec<self::SteLintViolation>, ::std::string::String> {
    use ::std::io::Write as _;
    let mut child = match ::std::process::Command::new(python)
        .arg(script)
        .arg("--json")
        .stdin(::std::process::Stdio::piped())
        .stdout(::std::process::Stdio::piped())
        .stderr(::std::process::Stdio::piped())
        .spawn()
    {
        ::core::result::Result::Ok(c) => c,
        ::core::result::Result::Err(e) => return ::core::result::Result::Err(e.to_string()),
    };

    if let ::core::option::Option::Some(mut stdin) = child.stdin.take() {
        if let ::core::result::Result::Err(e) = stdin.write_all(input.as_bytes()) {
            return ::core::result::Result::Err(e.to_string());
        }
    }

    let output = match child.wait_with_output() {
        ::core::result::Result::Ok(o) => o,
        ::core::result::Result::Err(e) => return ::core::result::Result::Err(e.to_string()),
    };

    if output.status.code() == ::core::option::Option::Some(2) {
        let stderr = ::std::string::String::from_utf8_lossy(&output.stderr);
        return ::core::result::Result::Err(stderr.to_string());
    }

    let stdout = ::std::string::String::from_utf8_lossy(&output.stdout);
    let parsed: self::SteLintOutput = match ::serde_json::from_str(&stdout) {
        ::core::result::Result::Ok(p) => p,
        ::core::result::Result::Err(e) => return ::core::result::Result::Err(e.to_string()),
    };
    ::core::result::Result::Ok(parsed.violations)
}

fn mask_source(source: &::core::primitive::str) -> ::std::string::String {
    let lines = source.lines().collect::<::std::vec::Vec<&::core::primitive::str>>();
    let mut masked_lines = ::std::vec::Vec::with_capacity(lines.len());
    for _ in 0..lines.len() {
        masked_lines.push(::std::string::String::new());
    }

    let mut in_block_doc = false;
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if in_block_doc {
            if let ::core::option::Option::Some(end_pos) = line.find("*/") {
                in_block_doc = false;
                let content = &line[..end_pos];
                masked_lines[idx] = content.to_string();
            } else {
                masked_lines[idx] = (*line).to_string();
            }
            continue;
        }

        if trimmed.starts_with("/**") || trimmed.starts_with("/*!") {
            let prefix = if trimmed.starts_with("/**") {
                "/**"
            } else {
                "/*!"
            };
            if let ::core::option::Option::Some(pos) = line.find(prefix) {
                let rest = &line[pos + prefix.len()..];
                if let ::core::option::Option::Some(end_pos) = rest.find("*/") {
                    let content = &rest[..end_pos];
                    let mut s = ::std::string::String::new();
                    for _ in 0..(pos + prefix.len()) {
                        s.push(' ');
                    }
                    s.push_str(content);
                    masked_lines[idx] = s;
                } else {
                    in_block_doc = true;
                    let mut s = ::std::string::String::new();
                    for _ in 0..(pos + prefix.len()) {
                        s.push(' ');
                    }
                    s.push_str(rest);
                    masked_lines[idx] = s;
                }
            }
        } else if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            let prefix = if trimmed.starts_with("///") {
                "///"
            } else {
                "//!"
            };
            if let ::core::option::Option::Some(pos) = line.find(prefix) {
                let content = &line[pos + prefix.len()..];
                let mut s = ::std::string::String::new();
                for _ in 0..(pos + prefix.len()) {
                    s.push(' ');
                }
                s.push_str(content);
                masked_lines[idx] = s;
            }
        } else if trimmed.starts_with("//") {
            if let ::core::option::Option::Some(pos) = line.find("//") {
                let content = &line[pos + 2..];
                let mut s = ::std::string::String::new();
                for _ in 0..(pos + 2) {
                    s.push(' ');
                }
                s.push_str(content);
                masked_lines[idx] = s;
            }
        }
    }

    if let ::core::result::Result::Ok(stream) = source.parse::<::proc_macro2::TokenStream>() {
        let trees = stream.into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
        let mut literals = ::std::vec::Vec::new();
        collect_literals(&trees, &mut literals);

        for (start_line, start_col, text) in literals {
            let sub_lines = text.split('\n').collect::<::std::vec::Vec<&::core::primitive::str>>();
            for (offset, sub) in sub_lines.iter().enumerate() {
                let line_num = start_line + offset;
                if line_num > 0 && line_num <= masked_lines.len() {
                    let idx = line_num - 1;
                    if masked_lines[idx].trim().is_empty() {
                        let mut s = ::std::string::String::new();
                        let col = if offset == 0 {
                            start_col
                        } else {
                            0
                        };
                        for _ in 0..col {
                            s.push(' ');
                        }
                        s.push_str(sub);
                        masked_lines[idx] = s;
                    }
                }
            }
        }
    }

    masked_lines.join("\n")
}

fn collect_literals(
    trees: &[::proc_macro2::TokenTree],
    out: &mut ::std::vec::Vec<(::core::primitive::usize, ::core::primitive::usize, ::std::string::String)>,
) {
    let mut i = 0;
    while i < trees.len() {
        match &trees[i] {
            ::proc_macro2::TokenTree::Group(g) => {
                let inner = g.stream().into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
                let is_doc = if g.delimiter() == ::proc_macro2::Delimiter::Bracket {
                    let p0 = inner.first();
                    let p1 = inner.get(1);
                    match (p0, p1) {
                        (
                            ::core::option::Option::Some(::proc_macro2::TokenTree::Ident(id)),
                            ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p)),
                        ) => id == "doc" && p.as_char() == '=',
                        _ => false,
                    }
                } else {
                    false
                };
                if !is_doc {
                    collect_literals(&inner, out);
                }
            },
            ::proc_macro2::TokenTree::Literal(lit) => {
                let text = lit.to_string();
                if let ::core::option::Option::Some((content, prefix_len)) = parse_string_literal(&text) {
                    if !content.starts_with("cargo:rerun-if-changed=") {
                        let (content_offset, msg) =
                            if let ::core::option::Option::Some(rest) = content.strip_prefix("cargo:warning=") {
                                ("cargo:warning=".len(), rest)
                            } else {
                                (0, content)
                            };
                        let trimmed = msg.trim();
                        if !trimmed.is_empty() && trimmed != "{}" && trimmed != "{warn}" && trimmed != "{msg}" {
                            let start = lit.span().start();
                            let col = start.column + prefix_len + content_offset;
                            out.push((start.line, col, msg.to_string()));
                        }
                    }
                }
            },
            _ => {},
        }
        i += 1;
    }
}

fn parse_string_literal(
    text: &::core::primitive::str,
) -> ::core::option::Option<(&::core::primitive::str, ::core::primitive::usize)> {
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        ::core::option::Option::Some((&text[1..text.len() - 1], 1))
    } else if let ::core::option::Option::Some(after_r) = text.strip_prefix('r') {
        let hash_count = after_r.chars().take_while(|&c| c == '#').count();
        let rest = &after_r[hash_count..];
        if rest.starts_with('"') && (text.ends_with('"') || text.ends_with('#')) {
            let quote_start = 1 + hash_count + 1;
            let quote_end = text.len() - 1 - hash_count;
            if quote_end >= quote_start {
                ::core::option::Option::Some((&text[quote_start..quote_end], quote_start))
            } else {
                ::core::option::Option::None
            }
        } else {
            ::core::option::Option::None
        }
    } else {
        ::core::option::Option::None
    }
}
