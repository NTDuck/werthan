fn main() {
    self::load_compile_env();
    ::flatmods::check();
    ::pathqual::check();
    ::plumbing::check();
    ::ste100::check();
    ::stepdown::check();
    ::turbofish::check();
}

/// Load key-value pairs from workspace root .env at compile time.
fn load_compile_env() {
    let manifest_dir = ::std::path::PathBuf::from(::core::env!("CARGO_MANIFEST_DIR"));
    let env_path = manifest_dir.join("../../../.env");
    if env_path.exists() {
        ::std::println!("cargo:rerun-if-changed={}", env_path.display());
        if let ::core::result::Result::Ok(content) = ::std::fs::read_to_string(&env_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let ::core::option::Option::Some((key, value)) = trimmed.split_once('=') {
                    let trimmed_key = key.trim();
                    let trimmed_val = value.trim().trim_matches('"');
                    ::std::println!("cargo:rustc-env={trimmed_key}={trimmed_val}");
                }
            }
        }
    }
}
