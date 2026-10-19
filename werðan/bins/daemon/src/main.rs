//! Thin modernization daemon binary: argument parsing and dispatch.

/// Command line arguments for daemon.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
struct CliArgs {
    /// Run smoke verification and exit.
    is_smoke: ::core::primitive::bool,
    /// Dry run mode.
    is_dry_run: ::core::primitive::bool,
    /// Custom configuration path.
    config_path: ::core::option::Option<::std::path::PathBuf>,
    /// Benchmark configuration path.
    benchmark_path: ::core::option::Option<::std::path::PathBuf>,
    /// Benchmark all ReCodeAgent dataset projects path.
    benchmark_all_path: ::core::option::Option<::std::path::PathBuf>,
}

#[::tokio::main]
async fn main() {
    let args = ::std::env::args().collect::<::std::vec::Vec<::std::string::String>>();
    let cli = parse_args(&args);

    if cli.is_smoke || cli.is_dry_run {
        ::topology::smoke::run_smoke_verification().await;
        return;
    }

    if let ::core::option::Option::Some(path) = &cli.benchmark_path {
        ::topology::run_benchmark(path).await;
        return;
    }

    if let ::core::option::Option::Some(path) = &cli.benchmark_all_path {
        ::topology::run_benchmark_all(path).await;
        return;
    }

    if let ::core::option::Option::Some(path) = &cli.config_path {
        ::topology::run_benchmark(path).await;
        return;
    }

    ::interactive::run_daemon_server().await;
}

fn parse_args(args: &[::std::string::String]) -> self::CliArgs {
    let mut is_smoke = false;
    let mut is_dry_run = false;
    let mut config_path = ::core::option::Option::None;
    let mut benchmark_path = ::core::option::Option::None;
    let mut benchmark_all_path = ::core::option::Option::None;

    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--smoke" => is_smoke = true,
            "--dry-run" => is_dry_run = true,
            "--benchmark" if idx + 1 < args.len() => {
                idx += 1;
                let target_str = &args[idx];
                let p = ::std::path::PathBuf::from(target_str);
                benchmark_path = ::core::option::Option::Some(p);
            },
            "--benchmark-all" => {
                let default_path = "benchmarks/assets/ReCodeAgent/data/tool_projects";
                if idx + 1 < args.len() && !args[idx + 1].starts_with("--") {
                    idx += 1;
                    let target_str = &args[idx];
                    let p = ::std::path::PathBuf::from(target_str);
                    benchmark_all_path = ::core::option::Option::Some(p);
                } else {
                    let p = ::std::path::PathBuf::from(default_path);
                    benchmark_all_path = ::core::option::Option::Some(p);
                }
            },
            "--config" if idx + 1 < args.len() => {
                idx += 1;
                let target_str = &args[idx];
                let p = ::std::path::PathBuf::from(target_str);
                config_path = ::core::option::Option::Some(p);
            },
            arg if (arg.ends_with(".yml") || arg.ends_with(".yaml")) && !arg.starts_with("--") => {
                let p = ::std::path::PathBuf::from(arg);
                config_path = ::core::option::Option::Some(p);
            },
            _ => {},
        }
        idx += 1;
    }

    CliArgs {
        is_smoke: { is_smoke },
        is_dry_run: { is_dry_run },
        config_path: { config_path },
        benchmark_path: { benchmark_path },
        benchmark_all_path: { benchmark_all_path },
    }
}
