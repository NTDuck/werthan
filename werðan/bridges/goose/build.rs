fn main() {
    load_env();
    ::flatmods::check();
    ::pathqual::check();
    ::plumbing::check();
    ::ste100::check();
    ::stepdown::check();
    ::turbofish::check();
}

fn load_env() {
    let manifest_dir = match ::std::env::var("CARGO_MANIFEST_DIR") {
        ::core::result::Result::Ok(d) => d,
        ::core::result::Result::Err(_) => return,
    };
    let root = ::std::path::PathBuf::from(manifest_dir);
    let env_path = root
        .ancestors()
        .flat_map(|dir| [dir.join(".env"), dir.join(".env.example")])
        .find(|candidate| candidate.is_file());

    let path = match env_path {
        ::core::option::Option::Some(p) => p,
        ::core::option::Option::None => return,
    };

    ::std::println!("cargo:rerun-if-changed={}", path.display());
    let content = match ::std::fs::read_to_string(&path) {
        ::core::result::Result::Ok(c) => c,
        ::core::result::Result::Err(_) => return,
    };

    content
        .lines()
        .map(::core::primitive::str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(k, v)| (k.trim(), v.trim().trim_matches('"').trim_matches('\'')))
        .for_each(|(key, val)| {
            ::std::println!("cargo:rerun-if-env-changed={key}");
            let effective = match ::std::env::var(key) {
                ::core::result::Result::Ok(env_val) => env_val,
                ::core::result::Result::Err(_) => val.to_string(),
            };
            ::std::println!("cargo:rustc-env={key}={effective}");
        });
}
