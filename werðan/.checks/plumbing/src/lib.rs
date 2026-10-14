/// Enforce chaining nested function calls with tap or pipe constructs across
/// multiple lines.
///
/// Call this function in the build.rs file of the target crate:
/// ```no_run
/// ::plumbing::check();
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
        ::std::println!("cargo:rerun-if-changed={}", file.display());
        let source = match ::std::fs::read_to_string(file) {
            ::core::result::Result::Ok(s) => s,
            ::core::result::Result::Err(_) => continue,
        };
        for warn in check_source(&source, file) {
            ::std::println!("{warn}");
        }
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

pub fn check_source(
    source: &::core::primitive::str,
    path: &::std::path::Path,
) -> ::std::vec::Vec<::std::string::String> {
    let stream = match source.parse::<::proc_macro2::TokenStream>() {
        ::core::result::Result::Ok(s) => s,
        ::core::result::Result::Err(_) => return ::std::vec::Vec::new(),
    };
    let trees = stream.into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
    let mut calls = ::std::vec::Vec::new();
    scan(&trees, 0, &mut calls);
    calls
        .into_iter()
        .map(|(line, name)| {
            ::std::format!(
                "cargo:warning={}:{}: plumbing rule violation: chain nested call `{name}(...)` with tap or pipe \
                 constructs across multiple lines.",
                path.display(),
                line
            )
        })
        .collect::<::std::vec::Vec<::std::string::String>>()
}

fn scan(
    trees: &[::proc_macro2::TokenTree],
    depth: ::core::primitive::usize,
    out: &mut ::std::vec::Vec<(::core::primitive::usize, ::std::string::String)>,
) {
    for (idx, tree) in trees.iter().enumerate() {
        match tree {
            ::proc_macro2::TokenTree::Group(g) => {
                let inner = g.stream().into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
                let next_depth = if g.delimiter() == ::proc_macro2::Delimiter::Parenthesis {
                    depth + 1
                } else {
                    0
                };
                scan(&inner, next_depth, out);
            },
            ::proc_macro2::TokenTree::Ident(id) if depth > 0 => {
                let is_call = has_call_args(trees, idx);
                let is_method = if idx > 0 {
                    match trees.get(idx - 1) {
                        ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p)) => p.as_char() == '.',
                        _ => false,
                    }
                } else {
                    false
                };
                let name = id.to_string();
                if is_call && !is_method && is_function_call(&name) {
                    out.push((id.span().start().line, name));
                }
            },
            _ => {},
        }
    }
}

fn is_function_call(name: &::core::primitive::str) -> ::core::primitive::bool {
    if is_keyword(name) {
        return false;
    }
    match name.chars().next() {
        ::core::option::Option::Some(c) => c.is_ascii_lowercase() || c == '_',
        _ => false,
    }
}
fn has_call_args(trees: &[::proc_macro2::TokenTree], idx: ::core::primitive::usize) -> ::core::primitive::bool {
    if let ::core::option::Option::Some(::proc_macro2::TokenTree::Group(g)) = trees.get(idx + 1) {
        if g.delimiter() == ::proc_macro2::Delimiter::Parenthesis {
            return true;
        }
    }
    let p1 = trees.get(idx + 1);
    let p2 = trees.get(idx + 2);
    let p3 = trees.get(idx + 3);
    let is_turbofish = match (p1, p2, p3) {
        (
            ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(a)),
            ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(b)),
            ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(c)),
        ) => a.as_char() == ':' && b.as_char() == ':' && c.as_char() == '<',
        _ => false,
    };
    if !is_turbofish {
        return false;
    }
    let mut depth: ::core::primitive::usize = 0;
    for (offset, tree) in trees[idx + 3..].iter().enumerate() {
        if let ::proc_macro2::TokenTree::Punct(p) = tree {
            match p.as_char() {
                '<' => depth += 1,
                '>' => {
                    depth -= 1;
                    if depth == 0 {
                        return match trees.get(idx + 4 + offset) {
                            ::core::option::Option::Some(::proc_macro2::TokenTree::Group(g)) =>
                                g.delimiter() == ::proc_macro2::Delimiter::Parenthesis,
                            _ => false,
                        };
                    }
                },
                _ => {},
            }
        }
    }
    false
}

fn is_keyword(s: &::core::primitive::str) -> ::core::primitive::bool {
    ::core::matches!(
        s,
        "as" | "async"
            | "await"
            | "box"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "macro"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "try"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "yield"
    )
}
