struct FuncDef {
    name: ::std::string::String,
    body: ::std::vec::Vec<::proc_macro2::TokenTree>,
    calls: ::std::vec::Vec<(::core::primitive::usize, ::proc_macro2::Span)>,
}

impl self::FuncDef {
    fn new(name: ::std::string::String, body: ::std::vec::Vec<::proc_macro2::TokenTree>) -> Self {
        let calls = ::std::vec::Vec::new();
        Self {
            name,
            body,
            calls,
        }
    }
}

/// Enforce Clean Code stepdown rule so that callers appear before callees.
///
/// Call this function in the build.rs file of the target crate:
/// ```no_run
/// ::stepdown::check();
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
    checker.check_scope(&trees);
    warnings
}

struct Checker<'a> {
    path: &'a ::std::path::Path,
    warnings: &'a mut ::std::vec::Vec<::std::string::String>,
}

impl<'a> Checker<'a> {
    fn new(path: &'a ::std::path::Path, warnings: &'a mut ::std::vec::Vec<::std::string::String>) -> Self {
        Self {
            path,
            warnings,
        }
    }

    fn check_scope(&mut self, trees: &[::proc_macro2::TokenTree]) {
        let (funcs, nested) = extract_scope(trees);
        self.check_order(&funcs);
        for inner in &nested {
            self.check_scope(inner);
        }
    }

    fn check_order(&mut self, funcs: &[FuncDef]) {
        for (caller_idx, caller) in funcs.iter().enumerate() {
            for &(callee_idx, span) in &caller.calls {
                if callee_idx < caller_idx {
                    self.warn(
                        span,
                        ::std::format_args!(
                            "`{}` calls `{}`, which is above it. Put callers before callees.",
                            caller.name,
                            funcs[callee_idx].name
                        ),
                    );
                }
            }

            let below = caller
                .calls
                .iter()
                .copied()
                .filter(|(i, _)| *i > caller_idx)
                .collect::<::std::vec::Vec<(::core::primitive::usize, ::proc_macro2::Span)>>();
            for (p, &(c_a, span)) in below.iter().enumerate() {
                for &(c_b, _) in &below[p + 1..] {
                    if c_b < c_a
                        && first_scope_call(funcs, c_b) >= first_scope_call(funcs, c_a)
                        && !reaches(funcs, c_b, c_a)
                    {
                        self.warn(
                            span,
                            ::std::format_args!(
                                "`{}` calls `{}` before `{}`, but `{}` is above `{}`.",
                                caller.name,
                                funcs[c_a].name,
                                funcs[c_b].name,
                                funcs[c_b].name,
                                funcs[c_a].name
                            ),
                        );
                    }
                }
            }
        }
    }

    fn warn(&mut self, span: ::proc_macro2::Span, msg: ::std::fmt::Arguments<'_>) {
        self.warnings.push(::std::format!(
            "cargo:warning={}:{}: stepdown rule violation: {msg}",
            self.path.display(),
            span.start().line
        ));
    }
}

fn extract_scope(
    trees: &[::proc_macro2::TokenTree],
) -> (::std::vec::Vec<FuncDef>, ::std::vec::Vec<::std::vec::Vec<::proc_macro2::TokenTree>>) {
    let mut funcs = ::std::vec::Vec::new();
    let mut nested = ::std::vec::Vec::new();
    let mut idx = 0;
    while idx < trees.len() {
        if let ::core::option::Option::Some((maybe_fn, next)) = try_parse_fn(trees, idx) {
            if let ::core::option::Option::Some(func) = maybe_fn {
                nested.push(func.body.clone());
                funcs.push(func);
            }
            idx = next;
            continue;
        }
        if is_container(trees.get(idx)) {
            if let ::core::option::Option::Some((maybe_body, next)) = find_block_end(trees, idx + 1) {
                if let ::core::option::Option::Some(body) = maybe_body {
                    nested.push(body);
                }
                idx = next;
                continue;
            }
        }
        idx += 1;
    }
    populate_calls(&mut funcs);
    (funcs, nested)
}

fn try_parse_fn(
    trees: &[::proc_macro2::TokenTree],
    start: ::core::primitive::usize,
) -> ::core::option::Option<(::core::option::Option<FuncDef>, ::core::primitive::usize)> {
    let kw = match trees.get(start) {
        ::core::option::Option::Some(::proc_macro2::TokenTree::Ident(id)) => id,
        _ => return ::core::option::Option::None,
    };
    let name_ident = match trees.get(start + 1) {
        ::core::option::Option::Some(::proc_macro2::TokenTree::Ident(id)) => id,
        _ => return ::core::option::Option::None,
    };
    if kw != "fn" {
        return ::core::option::Option::None;
    }
    let mut seen_params = false;
    for (i, tree) in trees[start + 2..].iter().enumerate() {
        match tree {
            ::proc_macro2::TokenTree::Group(g) if g.delimiter() == ::proc_macro2::Delimiter::Parenthesis => {
                seen_params = true;
            },
            ::proc_macro2::TokenTree::Group(g) if g.delimiter() == ::proc_macro2::Delimiter::Brace && seen_params => {
                let body = g.stream().into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
                let name = name_ident.to_string();
                let def = self::FuncDef::new(name, body);
                return ::core::option::Option::Some((::core::option::Option::Some(def), start + 3 + i));
            },
            ::proc_macro2::TokenTree::Punct(p) if p.as_char() == ';' && seen_params => {
                return ::core::option::Option::Some((::core::option::Option::None, start + 3 + i));
            },
            _ => {},
        }
    }
    ::core::option::Option::None
}

fn populate_calls(funcs: &mut [FuncDef]) {
    let names = funcs.iter().map(|f| f.name.clone()).collect::<::std::vec::Vec<::std::string::String>>();
    for (caller_idx, func) in funcs.iter_mut().enumerate() {
        let mut raw = ::std::vec::Vec::new();
        collect_body_calls(&func.body, &names, &mut raw);
        for (callee_idx, span) in raw {
            if callee_idx != caller_idx && !func.calls.iter().any(|(i, _)| *i == callee_idx) {
                func.calls.push((callee_idx, span));
            }
        }
    }
}

fn collect_body_calls(
    trees: &[::proc_macro2::TokenTree],
    names: &[::std::string::String],
    out: &mut ::std::vec::Vec<(::core::primitive::usize, ::proc_macro2::Span)>,
) {
    let mut idx = 0;
    while idx < trees.len() {
        if let ::core::option::Option::Some(next) = skip_nested_item(trees, idx) {
            idx = next;
            continue;
        }
        if let ::proc_macro2::TokenTree::Ident(ident) = &trees[idx] {
            let text = ident.to_string();
            if let ::core::option::Option::Some(callee_idx) = names.iter().position(|n| n == &text) {
                if is_valid_call(trees, idx) {
                    out.push((callee_idx, ident.span()));
                }
            }
        }
        if let ::proc_macro2::TokenTree::Group(g) = &trees[idx] {
            let inner = g.stream().into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
            collect_body_calls(&inner, names, out);
        }
        idx += 1;
    }
}

fn skip_nested_item(
    trees: &[::proc_macro2::TokenTree],
    start: ::core::primitive::usize,
) -> ::core::option::Option<::core::primitive::usize> {
    let first = trees.get(start);
    let second = trees.get(start + 1);
    let is_fn = match (first, second) {
        (
            ::core::option::Option::Some(::proc_macro2::TokenTree::Ident(k)),
            ::core::option::Option::Some(::proc_macro2::TokenTree::Ident(_)),
        ) => k == "fn",
        _ => false,
    };
    if !is_fn && !is_container(first) {
        return ::core::option::Option::None;
    }
    find_block_end(trees, start + 1).map(|(_, next)| next)
}

fn is_container(tree: ::core::option::Option<&::proc_macro2::TokenTree>) -> ::core::primitive::bool {
    let id = match tree {
        ::core::option::Option::Some(::proc_macro2::TokenTree::Ident(id)) => id,
        _ => return false,
    };
    let s = id.to_string();
    ::core::matches!(s.as_str(), "mod" | "impl" | "trait")
}

fn find_block_end(
    trees: &[::proc_macro2::TokenTree],
    start: ::core::primitive::usize,
) -> ::core::option::Option<(::core::option::Option<::std::vec::Vec<::proc_macro2::TokenTree>>, ::core::primitive::usize)>
{
    for (offset, tree) in trees[start..].iter().enumerate() {
        match tree {
            ::proc_macro2::TokenTree::Group(g) if g.delimiter() == ::proc_macro2::Delimiter::Brace => {
                let inner = g.stream().into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
                return ::core::option::Option::Some((::core::option::Option::Some(inner), start + offset + 1));
            },
            ::proc_macro2::TokenTree::Punct(p) if p.as_char() == ';' => {
                return ::core::option::Option::Some((::core::option::Option::None, start + offset + 1));
            },
            _ => {},
        }
    }
    ::core::option::Option::None
}

fn is_valid_call(trees: &[::proc_macro2::TokenTree], idx: ::core::primitive::usize) -> ::core::primitive::bool {
    let prefix_ok = match trees.get(idx.wrapping_sub(1)) {
        ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p)) if p.as_char() == '.' => {
            match trees.get(idx.wrapping_sub(2)) {
                ::core::option::Option::Some(::proc_macro2::TokenTree::Ident(w)) => w == "self",
                _ => false,
            }
        },
        ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(p)) if p.as_char() == ':' => {
            let has_colon = match trees.get(idx.wrapping_sub(2)) {
                ::core::option::Option::Some(::proc_macro2::TokenTree::Punct(c)) => c.as_char() == ':',
                _ => false,
            };
            let has_self = match trees.get(idx.wrapping_sub(3)) {
                ::core::option::Option::Some(::proc_macro2::TokenTree::Ident(w)) => w == "Self" || w == "self",
                _ => false,
            };
            has_colon && has_self
        },
        ::core::option::Option::Some(::proc_macro2::TokenTree::Ident(w)) if w == "fn" => false,
        _ => true,
    };
    prefix_ok && has_call_args(trees, idx)
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

fn first_scope_call(funcs: &[FuncDef], target: ::core::primitive::usize) -> ::core::primitive::usize {
    funcs.iter().flat_map(|f| &f.calls).position(|(i, _)| *i == target).unwrap_or(::core::primitive::usize::MAX)
}

fn reaches(
    funcs: &[FuncDef],
    start: ::core::primitive::usize,
    goal: ::core::primitive::usize,
) -> ::core::primitive::bool {
    let mut visited = ::std::vec::Vec::new();
    let mut stack = ::std::vec![start];
    while let ::core::option::Option::Some(curr) = stack.pop() {
        if visited.contains(&curr) {
            continue;
        }
        visited.push(curr);
        for &(next, _) in &funcs[curr].calls {
            if next == goal {
                return true;
            }
            stack.push(next);
        }
    }
    false
}
