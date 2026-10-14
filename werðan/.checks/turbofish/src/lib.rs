/// Enforce turbofish generics on call expressions instead of type annotations
/// on let bindings.
///
/// Example:
/// Disallowed:
/// ```ignore
/// let items: ::std::vec::Vec<::core::primitive::u32> = iter.collect();
/// ```
/// Required:
/// ```ignore
/// let items = iter.collect::<::std::vec::Vec<::core::primitive::u32>>();
/// ```
///
/// Call this function in the build.rs file of the target crate:
/// ```no_run
/// ::turbofish::check();
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
        let content = match ::std::fs::read_to_string(file) {
            ::core::result::Result::Ok(s) => s,
            ::core::result::Result::Err(_) => continue,
        };
        for warning in check_source(&content, file) {
            ::std::println!("{warning}");
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
    let mut checker = self::Checker::new(path);
    checker.scan_trees(&trees);
    checker.warnings
}

struct Checker<'a> {
    path: &'a ::std::path::Path,
    warnings: ::std::vec::Vec<::std::string::String>,
}

impl<'a> Checker<'a> {
    fn new(path: &'a ::std::path::Path) -> Self {
        let warnings = ::std::vec::Vec::new();
        Self {
            path,
            warnings,
        }
    }

    fn scan_trees(&mut self, trees: &[::proc_macro2::TokenTree]) {
        let mut i = 0;
        while i < trees.len() {
            match &trees[i] {
                ::proc_macro2::TokenTree::Group(g) => {
                    let inner = g.stream().into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
                    self.scan_trees(&inner);
                    i += 1;
                },
                ::proc_macro2::TokenTree::Ident(id) if id == "let" => {
                    let is_if_or_while = if i > 0 {
                        match &trees[i - 1] {
                            ::proc_macro2::TokenTree::Ident(prev) => prev == "if" || prev == "while",
                            _ => false,
                        }
                    } else {
                        false
                    };
                    if is_if_or_while {
                        i += 1;
                        continue;
                    }

                    let start_idx = i;
                    let line = id.span().start().line;
                    i += 1;

                    let mut semi_idx = i;
                    let mut found_semi = false;
                    while semi_idx < trees.len() {
                        if let ::proc_macro2::TokenTree::Punct(p) = &trees[semi_idx] {
                            if p.as_char() == ';' {
                                found_semi = true;
                                break;
                            }
                        }
                        semi_idx += 1;
                    }

                    if !found_semi {
                        continue;
                    }

                    let stmt_tokens = &trees[start_idx..semi_idx];
                    self.check_let_stmt(stmt_tokens, line);

                    for token in stmt_tokens {
                        if let ::proc_macro2::TokenTree::Group(g) = token {
                            let inner = g.stream().into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
                            self.scan_trees(&inner);
                        }
                    }

                    i = semi_idx + 1;
                },
                _ => {
                    i += 1;
                },
            }
        }
    }

    fn check_let_stmt(&mut self, tokens: &[::proc_macro2::TokenTree], line: ::core::primitive::usize) {
        let mut colon_pos = ::core::option::Option::None;
        let mut eq_pos = ::core::option::Option::None;

        for (idx, token) in tokens.iter().enumerate() {
            if let ::proc_macro2::TokenTree::Punct(p) = token {
                if p.as_char() == ':' && colon_pos.is_none() && eq_pos.is_none() {
                    let is_double_colon = if p.spacing() == ::proc_macro2::Spacing::Joint {
                        match tokens.get(idx + 1) {
                            ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p2)) => p2.as_char() == ':',
                            _ => false,
                        }
                    } else if idx > 0 {
                        match tokens.get(idx - 1) {
                            ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p0)) =>
                                p0.as_char() == ':' && p0.spacing() == ::proc_macro2::Spacing::Joint,
                            _ => false,
                        }
                    } else {
                        false
                    };
                    if !is_double_colon {
                        colon_pos = ::core::option::Option::Some(idx);
                    }
                } else if p.as_char() == '=' && eq_pos.is_none() {
                    eq_pos = ::core::option::Option::Some(idx);
                }
            }
        }

        if let ::core::option::Option::Some(c) = colon_pos {
            if let ::core::option::Option::Some(e) = eq_pos {
                if c < e {
                    let rhs = &tokens[e + 1..];
                    if has_call(rhs) {
                        self.warnings.push(::std::format!(
                            "cargo:warning={}:{line}: turbofish rule violation: use turbofish on call expression \
                             instead of type annotation in `let` binding.",
                            self.path.display()
                        ));
                    }
                }
            }
        }
    }
}

fn has_call(tokens: &[::proc_macro2::TokenTree]) -> ::core::primitive::bool {
    for i in 0..tokens.len() {
        match &tokens[i] {
            ::proc_macro2::TokenTree::Group(g) => {
                let inner = g.stream().into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
                if has_call(&inner) {
                    return true;
                }
            },
            ::proc_macro2::TokenTree::Ident(id) => {
                let name = id.to_string();
                if is_call_ident(&name) && has_call_args(tokens, i) {
                    return true;
                }
            },
            _ => {},
        }
    }
    false
}

fn is_call_ident(name: &::core::primitive::str) -> ::core::primitive::bool {
    !::core::matches!(name, "if" | "while" | "for" | "match" | "return" | "loop" | "unsafe")
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
