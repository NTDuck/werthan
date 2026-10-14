/// Enforce modern module structure by declaring modules in root or flat files
/// instead of legacy mod.rs files.
///
/// Call this function in the build.rs file of the target crate:
/// ```no_run
/// ::flatmods::check();
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
    let mut warnings = ::std::vec::Vec::new();
    if path.file_name().is_some_and(|n| n == "mod.rs") {
        warnings.push(::std::format!(
            "cargo:warning={}: flatmods rule violation: legacy mod.rs found. Use flat module naming instead.",
            path.display()
        ));
        let stream = match source.parse::<::proc_macro2::TokenStream>() {
            ::core::result::Result::Ok(s) => s,
            ::core::result::Result::Err(_) => return warnings,
        };
        let trees = stream.into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
        scan_submodules(&trees, path, &mut warnings);
    }
    warnings
}

fn scan_submodules(
    trees: &[::proc_macro2::TokenTree],
    path: &::std::path::Path,
    warnings: &mut ::std::vec::Vec<::std::string::String>,
) {
    for w in trees.windows(3) {
        if let [::proc_macro2::TokenTree::Ident(m), ::proc_macro2::TokenTree::Ident(name), ::proc_macro2::TokenTree::Punct(p)] =
            w
        {
            if m == "mod" && p.as_char() == ';' {
                warnings.push(::std::format!(
                    "cargo:warning={}:{}: flatmods rule violation: declare sub-module `{name}` directly in root crate \
                     file or flat module file.",
                    path.display(),
                    m.span().start().line
                ));
            }
        }
    }
    for tree in trees {
        if let ::proc_macro2::TokenTree::Group(g) = tree {
            let inner = g.stream().into_iter().collect::<::std::vec::Vec<::proc_macro2::TokenTree>>();
            scan_submodules(&inner, path, warnings);
        }
    }
}
