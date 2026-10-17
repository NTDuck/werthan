//! Integration tests executing the six repository linters across all project
//! files.

#[test]
fn test_linter_flatmods() {
    let files = target_files();
    for file in &files {
        let source = ::std::fs::read_to_string(file).unwrap();
        let warnings = ::flatmods::check_source(&source, file);
        let count = warnings.len();
        let disp = file.display();
        let msg = ::std::format!("flatmods failed on {disp}: {warnings:?}");
        ::std::assert_eq!(count, 0, "{msg}");
    }
}
#[test]
fn test_linter_pathqual() {
    let files = target_files();
    for file in &files {
        let source = ::std::fs::read_to_string(file).unwrap();
        let warnings = ::pathqual::check_source(&source, file);
        let count = warnings.len();
        let disp = file.display();
        let msg = ::std::format!("pathqual failed on {disp}: {warnings:?}");
        ::std::assert_eq!(count, 0, "{msg}");
    }
}

#[test]
fn test_linter_plumbing() {
    let files = target_files();
    for file in &files {
        let source = ::std::fs::read_to_string(file).unwrap();
        let warnings = ::plumbing::check_source(&source, file);
        let count = warnings.len();
        let disp = file.display();
        let msg = ::std::format!("plumbing failed on {disp}: {warnings:?}");
        ::std::assert_eq!(count, 0, "{msg}");
    }
}

#[test]
fn test_linter_ste100() {
    let files = target_files();
    for file in &files {
        let source = ::std::fs::read_to_string(file).unwrap();
        let warnings = ::ste100::check_source(&source, file);
        let count = warnings.len();
        let disp = file.display();
        let msg = ::std::format!("ste100 failed on {disp}: {warnings:?}");
        ::std::assert_eq!(count, 0, "{msg}");
    }
}

#[test]
fn test_linter_stepdown() {
    let files = target_files();
    for file in &files {
        let source = ::std::fs::read_to_string(file).unwrap();

        let warnings = ::stepdown::check_source(&source, file);
        let count = warnings.len();
        let disp = file.display();
        let msg = ::std::format!("stepdown failed on {disp}: {warnings:?}");
        ::std::assert_eq!(count, 0, "{msg}");
    }
}

#[test]
fn test_linter_turbofish() {
    let files = target_files();
    for file in &files {
        let source = ::std::fs::read_to_string(file).unwrap();
        let warnings = ::turbofish::check_source(&source, file);
        let count = warnings.len();
        let disp = file.display();
        let msg = ::std::format!("turbofish failed on {disp}: {warnings:?}");
        ::std::assert_eq!(count, 0, "{msg}");
    }
}

fn target_files() -> ::std::vec::Vec<::std::path::PathBuf> {
    let manifest_dir = ::std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = ::std::path::PathBuf::from(manifest_dir);
    let workspace_root = root.parent().unwrap().parent().unwrap().parent().unwrap();

    let relative_paths = [
        "werðan/bridges/omp/src/lib.rs",
        "werðan/bridges/goose/src/lib.rs",
        "werðan/bridges/codex/src/lib.rs",
        "werðan/bins/tui/src/main.rs",
        "werðan/bins/tui/src/config.rs",
        "werðan/bins/tui/src/app.rs",
        "werðan/bins/tui/src/tree.rs",
        "werðan/bins/tui/src/ui.rs",
    ];

    relative_paths.iter().map(|rel| workspace_root.join(rel)).collect::<::std::vec::Vec<::std::path::PathBuf>>()
}
