//! Batch benchmark mode: iterate every generated campaign YAML sequentially,
//! resume from existing result JSON files, and write an aggregate summary.

/// Aggregate metrics over one batch run.
pub async fn run_batch(
    config_dir: &::std::path::Path,
) -> ::core::result::Result<self::BatchSummary, ::std::string::String> {
    use super::bench_config::CampaignExt;
    let config_paths = self::list_config_paths(config_dir)?;
    let total_count = config_paths.len();
    let mut results = ::std::vec::Vec::new();
    for (idx, config_path) in config_paths.iter().enumerate() {
        let load_res = super::bench_config::load_bench_config(config_path);
        let bench_config = match load_res {
            ::core::result::Result::Ok(value) => value,
            ::core::result::Result::Err(load_error) => {
                ::std::println!(
                    "[batch {}/{total_count}] {}: config load failed: {load_error}",
                    idx + 1,
                    config_path.display()
                );
                continue;
            },
        };
        let label = bench_config.project_label();
        let resume_path = bench_config.result_path();
        let already_done = resume_path.exists();
        let result = if already_done {
            let cached = self::load_result(&resume_path);
            match cached {
                ::core::option::Option::Some(value) => {
                    ::std::println!("[batch {}/{total_count}] {label}: resumed from disk", idx + 1);
                    value
                },
                ::core::option::Option::None => {
                    let fresh = super::bench_runner::run_campaign(&bench_config).await;
                    let write_res = self::write_result(&bench_config, &fresh);
                    if let ::core::result::Result::Err(write_error) = write_res {
                        ::std::println!("[batch] {label}: result write failed: {write_error}");
                    }
                    fresh
                },
            }
        } else {
            let fresh = super::bench_runner::run_campaign(&bench_config).await;
            let write_res = self::write_result(&bench_config, &fresh);
            if let ::core::result::Result::Err(write_error) = write_res {
                ::std::println!("[batch] {label}: result write failed: {write_error}");
            }
            fresh
        };
        let error_text = self::format_option_error(&result.error);
        let check_state = if result.compilation_success { "ok" } else { "failed" };
        ::std::println!(
            "[batch {}/{total_count}] {label}: check={check_state} rate={:.1}% wall={:.1}s error={error_text}",
            idx + 1,
            result.test_pass_rate * 100.0,
            result.wall_time_ms as ::core::primitive::f64 / 1000.0,
        );
        results.push(result);
    }
    let summary = self::summarize(&results);
    ::core::result::Result::Ok(summary)
}

/// Render the optional error field for progress lines.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize)]
pub struct BatchSummary {
    /// Number of campaigns processed or skipped in the batch.
    pub total: ::core::primitive::usize,
    /// Share of campaigns whose result reports compilation success.
    pub compilation_rate: ::core::primitive::f64,
    /// Mean test pass rate over campaigns with executed tests.
    pub mean_test_pass_rate: ::core::primitive::f64,
    /// Mean engine wall-clock time in milliseconds.
    pub mean_wall_time_ms: ::core::primitive::f64,
    /// Sum of engine wall-clock times in milliseconds.
    pub total_wall_time_ms: ::core::primitive::u64,
    /// Per-category counts of successful compilations.
    pub per_category: ::std::collections::BTreeMap<::std::string::String, self::CategorySummary>,
}

/// Per-category breakdown entry.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize)]
pub struct CategorySummary {
    /// Campaign count in the category.
    pub count: ::core::primitive::usize,
    /// Campaign count whose result reports compilation success.
    pub compilation_successes: ::core::primitive::usize,
}

/// Category label derived from a result file stem such as `crust-vec`.
fn list_config_paths(dir: &::std::path::Path) -> ::core::result::Result<::std::vec::Vec<::std::path::PathBuf>, ::std::string::String> {
    let read_res = ::std::fs::read_dir(dir);
    let dir_entries = match read_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(read_error) => {
            return ::core::result::Result::Err(::std::format!(
                "config dir read failed: {read_error}"
            ));
        },
    };
    let mut config_paths = ::std::vec::Vec::new();
    for entry in dir_entries.flatten() {
        let path = entry.path();
        let path_text = path.to_string_lossy();
        let is_yaml = path_text.ends_with(".yml") || path_text.ends_with(".yaml");
        if is_yaml {
            config_paths.push(path);
        }
    }
    config_paths.sort();
    ::core::result::Result::Ok(config_paths)
}

/// Run the batch over every campaign YAML under one directory.
fn load_result(path: &::std::path::Path) -> ::core::option::Option<super::bench_runner::BenchResult> {
    let read_res = ::std::fs::read_to_string(path);
    let text = match read_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(_) => return ::core::option::Option::None,
    };
    let parse_res = ::serde_json::from_str::<super::bench_runner::BenchResult>(&text);
    parse_res.ok()
}

/// Write one result JSON to the daemon results directory.
fn write_result(
    bench_config: &super::bench_config::CampaignConfig,
    result: &super::bench_runner::BenchResult,
) -> ::core::result::Result<(), ::std::string::String> {
    use super::bench_config::CampaignExt;
    let result_dir = bench_config.result_dir();
    let dir_create = ::std::fs::create_dir_all(&result_dir);
    if let ::core::result::Result::Err(create_error) = dir_create {
        return ::core::result::Result::Err(::std::format!(
            "result dir creation failed: {create_error}"
        ));
    }
    let serialize_res = ::serde_json::to_string_pretty(result);
    let json_text = match serialize_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(serialize_error) => {
            return ::core::result::Result::Err(::std::format!(
                "result serialization failed: {serialize_error}"
            ));
        },
    };
    let write_res = ::std::fs::write(bench_config.result_path(), json_text);
    match write_res {
        ::core::result::Result::Ok(()) => ::core::result::Result::Ok(()),
        ::core::result::Result::Err(write_error) => {
            ::core::result::Result::Err(::std::format!("result write failed: {write_error}"))
        },
    }
}

/// List every campaign YAML under one directory in deterministic order.
fn format_option_error(
    error: &::core::option::Option<::std::string::String>,
) -> ::std::string::String {
    match error {
        ::core::option::Option::Some(text) => text.clone(),
        ::core::option::Option::None => "none".to_string(),
    }
}

/// Aggregate batch metrics over collected results.
fn summarize(results: &[super::bench_runner::BenchResult]) -> self::BatchSummary {
    let total = results.len();
    let mut compilation_successes = 0;
    let mut pass_rate_sum = 0.0;
    let mut wall_time_sum_ms = 0;
    let mut per_category = ::std::collections::BTreeMap::new();
    for result in results {
        let success = result.compilation_success;
        if success {
            compilation_successes += 1;
        }
        pass_rate_sum += result.test_pass_rate;
        wall_time_sum_ms += result.wall_time_ms;
        let label = category_of(&result.campaign_id);
        let entry = per_category.entry(label).or_insert(self::CategorySummary {
            count: 0,
            compilation_successes: 0,
        });
        entry.count += 1;
        if success {
            entry.compilation_successes += 1;
        }
    }
    let count_float = total as ::core::primitive::f64;
    let compilation_rate = (compilation_successes as ::core::primitive::f64) / count_float;
    let mean_test_pass_rate = pass_rate_sum / count_float;
    let mean_wall_time_ms = (wall_time_sum_ms as ::core::primitive::f64) / count_float;
    let summary = self::BatchSummary {
        total,
        compilation_rate,
        mean_test_pass_rate,
        mean_wall_time_ms,
        total_wall_time_ms: { wall_time_sum_ms },
        per_category,
    };
    summary
}

/// Write the aggregate summary JSON next to the per-project results.
fn category_of(stem: &::core::primitive::str) -> ::std::string::String {
    let split_res = stem.split_once('-');
    let head = match split_res {
        ::core::option::Option::Some(parts) => parts.0,
        ::core::option::Option::None => stem,
    };
    head.to_string()
}

/// Load an existing per-project result JSON file.
pub fn write_summary(
    summary: &self::BatchSummary,
) -> ::core::result::Result<::std::path::PathBuf, ::std::string::String> {
    let result_dir = super::bench_config::result_dir_default();
    let create_res = ::std::fs::create_dir_all(&result_dir);
    if let ::core::result::Result::Err(create_error) = create_res {
        return ::core::result::Result::Err(::std::format!(
            "summary dir creation failed: {create_error}"
        ));
    }
    let serialize_res = ::serde_json::to_string_pretty(summary);
    let json_text = match serialize_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(serialize_error) => {
            return ::core::result::Result::Err(::std::format!(
                "summary serialization failed: {serialize_error}"
            ));
        },
    };
    let summary_path = result_dir.join("summary.json");
    let write_res = ::std::fs::write(&summary_path, json_text);
    match write_res {
        ::core::result::Result::Ok(()) => ::core::result::Result::Ok(summary_path),
        ::core::result::Result::Err(write_error) => {
            ::core::result::Result::Err(::std::format!("summary write failed: {write_error}"))
        },
    }
}
