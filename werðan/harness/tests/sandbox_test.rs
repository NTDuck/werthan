//! Tests for sandbox.

#[test]
fn test_sandbox_enforce_filesystem_policy() {
    let legacy_root = ::std::path::PathBuf::from("/workspace/legacy");
    let modernized_root = ::std::path::PathBuf::from("/workspace/modernized");
    let analysis_root = ::std::path::PathBuf::from("/workspace/analysis");
    let tmp_root = ::std::path::PathBuf::from("/tmp");

    let policy = ::harness::sandbox::SandboxPolicy {
        read_only_paths: { ::std::vec![legacy_root] },
        writable_paths: { ::std::vec![modernized_root, analysis_root, tmp_root] },
    };

    let legacy_file = ::std::path::Path::new("/workspace/legacy/CobolSource.cbl");
    // Read should always succeed
    let read_res = ::harness::sandbox::enforce_filesystem_sandbox(&policy, legacy_file, false);
    ::std::assert!(read_res.is_ok());

    // Write to read-only path should fail
    let write_ro_res = ::harness::sandbox::enforce_filesystem_sandbox(&policy, legacy_file, true);
    ::std::assert!(write_ro_res.is_err());

    let mod_file = ::std::path::Path::new("/workspace/modernized/lib.rs");
    // Write to allowed writable path should succeed
    let write_mod_res = ::harness::sandbox::enforce_filesystem_sandbox(&policy, mod_file, true);
    ::std::assert!(write_mod_res.is_ok());

    let tmp_file = ::std::path::Path::new("/tmp/scratch.txt");
    let write_tmp_res = ::harness::sandbox::enforce_filesystem_sandbox(&policy, tmp_file, true);
    ::std::assert!(write_tmp_res.is_ok());

    let etc_file = ::std::path::Path::new("/etc/passwd");
    // Write outside writable paths should fail
    let write_etc_res = ::harness::sandbox::enforce_filesystem_sandbox(&policy, etc_file, true);
    ::std::assert!(write_etc_res.is_err());
}

#[test]
fn test_sandbox_init_process_sandbox() {
    let res = ::harness::sandbox::init_process_sandbox();
    ::std::assert!(res.is_ok());
}
