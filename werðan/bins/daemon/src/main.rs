//! Thin modernization daemon binary: argument parsing and dispatch into the
//! daemon library crate (bench subsystem modules live in the lib target).

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
        ::daemon::bench_smoke::run_smoke_cli().await;
        return;
    }

    if let ::core::option::Option::Some(path) = &cli.benchmark_path {
        run_benchmark(path).await;
        return;
    }

    if let ::core::option::Option::Some(path) = &cli.benchmark_all_path {
        run_benchmark_all(path).await;
        return;
    }

    if let ::core::option::Option::Some(path) = &cli.config_path {
        run_benchmark(path).await;
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
                let path_buf = ::std::path::PathBuf::from(target_str);
                benchmark_path = ::core::option::Option::Some(path_buf);
            },
            "--benchmark-all" => {
                let default_path = ::std::path::PathBuf::from(::core::env!("BENCH_CONFIGS_DIR"));
                if idx + 1 < args.len() && !args[idx + 1].starts_with("--") {
                    idx += 1;
                    let target_str = &args[idx];
                    let path_buf = ::std::path::PathBuf::from(target_str);
                    benchmark_all_path = ::core::option::Option::Some(path_buf);
                } else {
                    benchmark_all_path = ::core::option::Option::Some(default_path);
                }
            },
            "--config" if idx + 1 < args.len() => {
                idx += 1;
                let target_str = &args[idx];
                let path_buf = ::std::path::PathBuf::from(target_str);
                config_path = ::core::option::Option::Some(path_buf);
            },
            arg if (arg.ends_with(".yml") || arg.ends_with(".yaml")) && !arg.starts_with("--") => {
                let path_buf = ::std::path::PathBuf::from(arg);
                config_path = ::core::option::Option::Some(path_buf);
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

async fn run_benchmark(path: &::std::path::Path) {
    use ::daemon::bench_config::CampaignExt;
    let load_res = ::daemon::bench_config::load_bench_config(path);
    let bench_config = match load_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(load_error) => {
            ::std::println!("[benchmark] Config load failed for {path:?}: {load_error}");
            ::std::process::exit(1);
        },
    };
    ::std::println!(
        "[benchmark] Campaign {} starting.",
        bench_config.campaign_id
    );
    let result = ::daemon::bench_runner::run_campaign(&bench_config).await;
    let result_dir = bench_config.result_dir();
    let create_res = ::std::fs::create_dir_all(&result_dir);
    if let ::core::result::Result::Err(create_error) = create_res {
        ::std::println!("[benchmark] Result dir creation failed: {create_error}");
        ::std::process::exit(1);
    }
    let serialize_res = ::serde_json::to_string_pretty(&result);
    let json_text = match serialize_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(serialize_error) => {
            ::std::println!("[benchmark] Result serialization failed: {serialize_error}");
            ::std::process::exit(1);
        },
    };
    let write_res = ::std::fs::write(bench_config.result_path(), json_text);
    match write_res {
        ::core::result::Result::Ok(()) => {},
        ::core::result::Result::Err(write_error) => {
            ::std::println!("[benchmark] Result write failed: {write_error}");
            ::std::process::exit(1);
        },
    }
    ::std::println!(
        "[benchmark] {}: check={} rate={:.1}% wall={:.1}s unimplemented={} error={}",
        bench_config.project_label(),
        if result.compilation_success { "ok" } else { "failed" },
        result.test_pass_rate * 100.0,
        result.wall_time_ms as ::core::primitive::f64 / 1000.0,
        result.unimplemented_remaining,
        result
            .error
            .clone()
            .unwrap_or_else(|| "none".to_string()),
    );
}

async fn run_benchmark_all(path: &::std::path::Path) {
    let batch_res = ::daemon::bench_batch::run_batch(path).await;
    let summary = match batch_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(batch_error) => {
            ::std::println!("[batch] Batch failed: {batch_error}");
            ::std::process::exit(1);
        },
    };
    let summary_write_res = ::daemon::bench_batch::write_summary(&summary);
    match summary_write_res {
        ::core::result::Result::Ok(summary_path) => {
            ::std::println!(
                "[batch] total={} compilation_rate={:.1}% mean_rate={:.1}% mean_wall={:.1}s total_wall={:.1}s",
                summary.total,
                summary.compilation_rate * 100.0,
                summary.mean_test_pass_rate * 100.0,
                summary.mean_wall_time_ms / 1000.0,
                summary.total_wall_time_ms as ::core::primitive::f64 / 1000.0,
            );
            ::std::println!("[batch] Summary written to {}", summary_path.display());
        },
        ::core::result::Result::Err(write_error) => {
            ::std::println!("[batch] Summary write failed: {write_error}");
            ::std::process::exit(1);
        },
    }
}
