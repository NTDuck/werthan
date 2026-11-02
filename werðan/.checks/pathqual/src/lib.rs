/// Enforce fully qualified paths with a leading double colon.
///
/// Call this function in the build.rs file of the target crate:
/// ```no_run
/// ::pathqual::check();
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
    let source = match ::std::fs::read_to_string(path) {
        ::core::result::Result::Ok(s) => s,
        ::core::result::Result::Err(_) => return,
    };
    for warn in check_source(&source, path) {
        ::std::println!("{warn}");
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
    let mut warnings = ::std::vec::Vec::new();
    let mut checker = self::Checker::new(path, &mut warnings);
    checker.scan(&trees);
    warnings
}

#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq)]
enum Ctx {
    None,
    Type,
    Generic,
}

struct Checker<'a> {
    path: &'a ::std::path::Path,
    warnings: &'a mut ::std::vec::Vec<::std::string::String>,
    generic_params: ::std::collections::HashSet<::std::string::String>,
}

impl<'a> Checker<'a> {
    fn new(path: &'a ::std::path::Path, warnings: &'a mut ::std::vec::Vec<::std::string::String>) -> Self {
        let generic_params = ::std::collections::HashSet::new();
        Self {
            path,
            warnings,
            generic_params,
        }
    }

    fn scan(&mut self, trees: &[::proc_macro2::TokenTree]) {
        let mut idx = 0;
        let mut ctx = self::Ctx::None;
        let mut in_type_decl = false;
        let mut generic_depth: ::core::primitive::usize = 0;
        while idx < trees.len() {
            if let ::core::option::Option::Some(skip) = self.check_attr(&trees[idx..]) {
                idx += skip;
                continue;
            }

            match &trees[idx] {
                ::proc_macro2::TokenTree::Punct(p) => match p.as_char() {
                    ';' => {
                        in_type_decl = false;
                        ctx = self::Ctx::None;
                    },
                    '=' => {
                        if in_type_decl {
                            ctx = self::Ctx::Type;
                        } else {
                            ctx = self::Ctx::None;
                        }
                    },
                    ',' => {
                        // A comma inside an open `<...>` list still belongs to the
                        // generic declaration (`<H, D>`), so keep registering the
                        // parameters that follow it; reset only at expression level.
                        ctx = if generic_depth > 0 {
                            self::Ctx::Generic
                        } else {
                            self::Ctx::None
                        };
                    },
                    ':' if !is_part_of_colon2(trees, idx) => ctx = self::Ctx::Type,
                    '-' => {
                        let is_arrow = match trees.get(idx + 1) {
                            ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(a)) => a.as_char() == '>',
                            _ => false,
                        };
                        if is_arrow {
                            ctx = self::Ctx::Type;
                            idx += 1;
                        }
                    },
                    '<' => {
                        generic_depth += 1;
                        ctx = self::Ctx::Generic;
                    },
                    '>' => {
                        generic_depth = generic_depth.saturating_sub(1);
                        if generic_depth == 0 {
                            ctx = self::Ctx::None;
                        }
                    },
                    _ => {},
                },
                ::proc_macro2::TokenTree::Group(g) => {
                    let inner = g.stream().into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
                    self.scan(&inner);
                    if g.delimiter() == ::proc_macro2::Delimiter::Brace {
                        ctx = self::Ctx::None;
                        self.generic_params.clear();
                    }
                },
                ::proc_macro2::TokenTree::Ident(id) => {
                    let text = id.to_string();
                    match text.as_str() {
                        "use" | "where" | "impl" | "as" => ctx = self::Ctx::Type,
                        "type" => {
                            in_type_decl = true;
                            ctx = self::Ctx::None;
                        },
                        "fn" | "struct" | "enum" | "mod" | "let" => {
                            in_type_decl = false;
                            ctx = self::Ctx::None;
                        },
                        _ => {
                            if ctx == self::Ctx::Generic {
                                let has_colon_prev = idx >= 2 && is_colon2(trees, idx - 2);
                                if !has_colon_prev {
                                    self.generic_params.insert(text.clone());
                                }
                            }
                            if self.generic_params.contains(&text) {
                                // Generic parameter in scope, do not flag
                            } else if is_macro(trees, idx, &text) {
                                let has_colon_prefix = idx >= 2 && is_colon2(trees, idx - 2);
                                if !has_colon_prefix {
                                    self.warn(id.span(), &text);
                                }
                            } else {
                                let is_type_or_colon = if ctx == self::Ctx::Type || is_primitive(&text) {
                                    true
                                } else {
                                    is_colon2(trees, idx + 1)
                                };
                                if is_type_or_colon && should_flag(trees, idx, &text) {
                                    self.warn(id.span(), &text);
                                }
                            }
                        },
                    }
                },
                _ => {},
            }
            idx += 1;
        }
    }

    fn check_attr(&mut self, trees: &[::proc_macro2::TokenTree]) -> ::core::option::Option<::core::primitive::usize> {
        let hash = match trees.first() {
            ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p)) => p,
            _ => return ::core::option::Option::None,
        };
        if hash.as_char() != '#' {
            return ::core::option::Option::None;
        }
        let is_bang = match trees.get(1) {
            ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p)) => p.as_char() == '!',
            _ => false,
        };
        let skip = if is_bang {
            3
        } else {
            2
        };
        let g = match trees.get(skip - 1) {
            ::core::option::Option::Some(::proc_macro2::TokenTree::Group(grp)) => grp,
            _ => return ::core::option::Option::None,
        };
        if g.delimiter() != ::proc_macro2::Delimiter::Bracket {
            return ::core::option::Option::None;
        }
        let tokens = g.stream().into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
        for (i, tree) in tokens.iter().enumerate() {
            if let ::proc_macro2::TokenTree::Ident(id) = tree {
                let s = id.to_string();
                if s == "derive" {
                    if let ::core::option::Option::Some(::proc_macro2::TokenTree::Group(dg)) = tokens.get(i + 1) {
                        if dg.delimiter() == ::proc_macro2::Delimiter::Parenthesis {
                            let d_tokens =
                                dg.stream().into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
                            for (di, d_tree) in d_tokens.iter().enumerate() {
                                if let ::proc_macro2::TokenTree::Ident(d_id) = d_tree {
                                    let ds = d_id.to_string();
                                    if should_flag(&d_tokens, di, &ds) {
                                        self.warn(d_id.span(), &ds);
                                    }
                                }
                            }
                        }
                    }
                } else if is_colon2(&tokens, i + 1) && should_flag(&tokens, i, &s) {
                    self.warn(id.span(), &s);
                }
            }
        }
        ::core::option::Option::Some(skip)
    }

    fn warn(&mut self, span: ::proc_macro2::Span, text: &::core::primitive::str) {
        self.warnings.push(::std::format!(
            "cargo:warning={}:{}: pathqual rule violation: fully qualify `{text}` with a leading `::`.",
            self.path.display(),
            span.start().line
        ));
    }
}

fn is_part_of_colon2(trees: &[::proc_macro2::TokenTree], idx: ::core::primitive::usize) -> ::core::primitive::bool {
    if let ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p)) = trees.get(idx) {
        if p.as_char() == ':' && p.spacing() == ::proc_macro2::Spacing::Joint {
            if let ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p2)) = trees.get(idx + 1) {
                if p2.as_char() == ':' {
                    return true;
                }
            }
        }
    }
    if idx > 0 {
        if let ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p_prev)) = trees.get(idx - 1) {
            if p_prev.as_char() == ':' && p_prev.spacing() == ::proc_macro2::Spacing::Joint {
                if let ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p)) = trees.get(idx) {
                    if p.as_char() == ':' {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn is_macro(
    trees: &[::proc_macro2::TokenTree],
    idx: ::core::primitive::usize,
    text: &::core::primitive::str,
) -> ::core::primitive::bool {
    if is_allowed(text) {
        return false;
    }
    let is_bang = match trees.get(idx + 1) {
        ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p)) => {
            p.as_char() == '!' && p.spacing() != ::proc_macro2::Spacing::Joint
        },
        _ => false,
    };
    if !is_bang {
        return false;
    }
    ::core::matches!(trees.get(idx + 2), ::core::option::Option::Some(::proc_macro2::TokenTree::Group(_)))
}

fn should_flag(
    trees: &[::proc_macro2::TokenTree],
    idx: ::core::primitive::usize,
    text: &::core::primitive::str,
) -> ::core::primitive::bool {
    if is_allowed(text) {
        return false;
    }
    if is_dot_access(trees, idx) {
        return false;
    }
    if is_lifetime(trees, idx) {
        return false;
    }
    if idx >= 2 && is_colon2(trees, idx - 2) {
        return false;
    }
    true
}

fn is_primitive(name: &::core::primitive::str) -> ::core::primitive::bool {
    ::core::matches!(
        name,
        "bool"
            | "char"
            | "str"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "f32"
            | "f64"
    )
}

fn is_allowed(name: &::core::primitive::str) -> ::core::primitive::bool {
    ::core::matches!(
        name,
        "self"
            | "super"
            | "crate"
            | "Self"
            | "as"
            | "async"
            | "await"
            | "box"
            | "break"
            | "const"
            | "continue"
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
            | "macro_rules"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "static"
            | "struct"
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

fn is_dot_access(trees: &[::proc_macro2::TokenTree], idx: ::core::primitive::usize) -> ::core::primitive::bool {
    let prev = match trees.get(idx.wrapping_sub(1)) {
        ::core::option::Option::Some(t) => t,
        ::core::option::Option::None => return false,
    };
    match prev {
        ::proc_macro2::TokenTree::Punct(p) => p.as_char() == '.',
        _ => false,
    }
}

fn is_lifetime(trees: &[::proc_macro2::TokenTree], idx: ::core::primitive::usize) -> ::core::primitive::bool {
    let prev = match trees.get(idx.wrapping_sub(1)) {
        ::core::option::Option::Some(t) => t,
        ::core::option::Option::None => return false,
    };
    match prev {
        ::proc_macro2::TokenTree::Punct(p) => p.as_char() == '\'',
        _ => false,
    }
}

fn is_colon2(trees: &[::proc_macro2::TokenTree], idx: ::core::primitive::usize) -> ::core::primitive::bool {
    let first = match trees.get(idx) {
        ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p)) => p,
        _ => return false,
    };
    if first.as_char() != ':' || first.spacing() != ::proc_macro2::Spacing::Joint {
        return false;
    }
    let second = match trees.get(idx + 1) {
        ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p)) => p,
        _ => return false,
    };
    second.as_char() == ':'
}
