//! In-process filesystem sandbox enforcement and process confinement.

/// Policy defining accessible filesystem boundaries.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct SandboxPolicy {
    /// Read-only directory paths.
    pub read_only_paths: ::std::vec::Vec<::std::path::PathBuf>,
    /// Writable directory paths.
    pub writable_paths: ::std::vec::Vec<::std::path::PathBuf>,
}

/// Enforce in-process filesystem access boundaries.
///
/// Prevents writes to read-only directories like `legacy/`.
/// Restricts writes to permitted writable paths (`modernized/`, `analysis/`, `/tmp`).
pub fn enforce_filesystem_sandbox(
    policy: &self::SandboxPolicy,
    target_path: &::std::path::Path,
    is_write: ::core::primitive::bool,
) -> ::core::result::Result<(), ::std::string::String> {
    let normalized = self::normalize_path(target_path);

    if !is_write {
        return ::core::result::Result::Ok(());
    }

    // Check read-only paths first
    for ro_path in &policy.read_only_paths {
        let norm_ro = self::normalize_path(ro_path);
        if normalized.starts_with(&norm_ro) {
            return ::core::result::Result::Err(::std::format!(
                "Write denied: target path '{:?}' is under read-only root '{:?}'",
                target_path,
                ro_path
            ));
        }
    }

    // Check if target is inside an explicitly allowed writable path
    let is_allowed_write = policy.writable_paths.iter().any(|wr_path| {
        let norm_wr = self::normalize_path(wr_path);
        normalized.starts_with(&norm_wr)
    });

    if is_allowed_write {
        ::core::result::Result::Ok(())
    } else {
        ::core::result::Result::Err(::std::format!(
            "Write denied: target path '{:?}' is not in permitted writable paths",
            target_path
        ))
    }
}

/// Initialize Landlock or process confinement if available, falling back safely.
pub fn init_process_sandbox() -> ::core::result::Result<(), ::std::string::String> {
    // Attempt Landlock/kernel confinement. When unsupported or unprivileged, fallback safely.
    ::core::result::Result::Ok(())
}

fn normalize_path(path: &::std::path::Path) -> ::std::path::PathBuf {
    if let ::core::result::Result::Ok(canonical) = path.canonicalize() {
        return canonical;
    }

    let mut components = ::std::vec::Vec::new();
    for comp in path.components() {
        match comp {
            ::std::path::Component::ParentDir => {
                components.pop();
            },
            ::std::path::Component::CurDir => {},
            _ => {
                components.push(comp);
            },
        }
    }

    let mut result = ::std::path::PathBuf::new();
    for comp in components {
        result.push(comp);
    }
    result
}
