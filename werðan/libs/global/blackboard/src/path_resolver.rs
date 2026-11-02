//! Platform path resolution for blackboard persistence.

/// Resolve a component target file path from optional path, output directory, hash, and file name.
pub fn resolve_component_path(
    file_path: ::core::option::Option<&::std::path::Path>,
    output_dir: ::core::option::Option<&::std::path::Path>,
    hash: &::core::primitive::str,
    default_file_name: &::core::primitive::str,
) -> ::std::path::PathBuf {
    if let ::core::option::Option::Some(given_path) = file_path {
        if given_path.is_absolute() {
            given_path.to_path_buf()
        } else if let ::core::option::Option::Some(out_dir) = output_dir {
            out_dir.join(given_path)
        } else {
            self::default_blackboard_dir(hash).join(given_path)
        }
    } else if let ::core::option::Option::Some(out_dir) = output_dir {
        out_dir.join(default_file_name)
    } else {
        self::default_blackboard_dir(hash).join(default_file_name)
    }
}

/// Compute the default root directory for blackboard persistence.
pub fn default_blackboard_dir(hash: &::core::primitive::str) -> ::std::path::PathBuf {
    if ::core::cfg!(windows) {
        let fallback_path = ::std::path::PathBuf::from("AppData");
        let base_dir = match ::dirs::data_dir() {
            ::core::option::Option::Some(directory) => directory,
            ::core::option::Option::None => fallback_path,
        };
        base_dir.join("werðan").join(hash)
    } else {
        let fallback_path = ::std::path::PathBuf::from("/home");
        let base_dir = match ::dirs::home_dir() {
            ::core::option::Option::Some(directory) => directory,
            ::core::option::Option::None => fallback_path,
        };
        base_dir.join(".werðan").join(hash)
    }
}
