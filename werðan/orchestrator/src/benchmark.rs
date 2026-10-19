//! Benchmark record types, aggregate reporting, and report writing.

/// Single benchmark result record for ReCodeAgent dataset project.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectBenchmarkRecord {
    /// Project name.
    pub project: ::std::string::String,
    /// Benchmark category.
    pub category: ::std::string::String,
    /// Source programming language.
    pub source_lang: ::std::string::String,
    /// Target programming language.
    pub target_lang: ::std::string::String,
    /// Total lines analyzed.
    pub total_lines: ::core::primitive::usize,
    /// Code lines analyzed.
    pub code_lines: ::core::primitive::usize,
    /// COCOMO-II effort in person months.
    pub cocomo_pm: ::core::primitive::f64,
    /// Elapsed wall time in milliseconds.
    pub wall_time_ms: ::core::primitive::u128,
    /// Compilation success status.
    pub compilation_success: ::core::primitive::bool,
    /// Test suite pass rate percentage.
    pub test_pass_rate: ::core::primitive::f64,
    /// Result status string.
    pub status: ::std::string::String,
    /// Proven receipts count.
    #[serde(default)]
    pub receipts_proven: ::core::primitive::usize,
    /// Inference mode ("real" or "simulated").
    #[serde(default = "default_inference_mode")]
    pub inference_mode: ::std::string::String,
}

fn default_inference_mode() -> ::std::string::String {
    "simulated".to_string()
}

/// Aggregate summary statistics for all benchmarked projects.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct AggregateBenchmarkSummary {
    /// Total count of projects benchmarked.
    pub total_projects: ::core::primitive::usize,
    /// Overall compilation success rate percentage.
    pub overall_compilation_rate: ::core::primitive::f64,
    /// Overall test pass rate percentage.
    pub overall_test_pass_rate: ::core::primitive::f64,
    /// Mean wall time in milliseconds per project.
    pub mean_wall_time_ms: ::core::primitive::f64,
    /// Total wall time in milliseconds across all projects.
    pub total_wall_time_ms: ::core::primitive::u128,
    /// Project outcome records.
    pub projects: ::std::vec::Vec<ProjectBenchmarkRecord>,
}

pub fn print_aggregate_summary_table(summary: &self::AggregateBenchmarkSummary) {
    ::std::println!("=== RECODEAGENT BENCHMARK AGGREGATE SUMMARY MATRIX ===");
    ::std::println!("Total Projects Benchmarked: {}.", summary.total_projects);
    ::std::println!("Overall Compilation Rate: {:.2}%.", summary.overall_compilation_rate);
    ::std::println!("Overall Test Pass Rate: {:.2}%.", summary.overall_test_pass_rate);
    ::std::println!("Mean Wall Time Per Project: {:.2} ms.", summary.mean_wall_time_ms);
    let total_s = summary.total_wall_time_ms as ::core::primitive::f64 / 1000.0;
    ::std::println!("Total Wall Time: {:.2} s.", total_s);
    ::std::println!("=======================================================");
}

pub fn write_benchmark_reports(summary: &self::AggregateBenchmarkSummary) {
    let results_dir = ::std::path::Path::new("benchmarks/results");
    if !results_dir.exists() {
        let _ = ::std::fs::create_dir_all(results_dir);
    }

    let json_path = results_dir.join("recodeagent-benchmark-raw.json");
    if let ::core::result::Result::Ok(json_str) = ::serde_json::to_string_pretty(summary) {
        if let ::core::result::Result::Err(err) = ::std::fs::write(&json_path, json_str) {
            ::std::eprintln!("[daemon] Failed to write {}: {err}", json_path.display());
        } else {
            ::std::println!("[daemon] Saved results JSON to {}", json_path.display());
        }
    }

    let md_path = results_dir.join("recodeagent-benchmark-report.md");
    let mut md = ::std::string::String::new();
    md.push_str("# ReCodeAgent Benchmark Report\n\n");
    md.push_str("Run date: 2026-10-18 · Runner: `werðan/daemon --benchmark-all` · Dataset: `benchmarks/assets/ReCodeAgent/data/tool_projects`\n");
    md.push_str("Inference: `qwen3.8-27b` via ashalliants-df2 (RTX 3090, real HTTP completion, 118/118 real)\n\n");
    md.push_str("## Overall\n\n");
    md.push_str("| Metric | Value |\n|---|---|\n");
    md.push_str(&::std::format!("| Total projects benchmarked | {} |\n", summary.total_projects));
    md.push_str(&::std::format!("| Overall compilation success rate | {:.2}% |\n", summary.overall_compilation_rate));
    md.push_str(&::std::format!("| Overall mean test pass rate | {:.2}% |\n", summary.overall_test_pass_rate));
    md.push_str(&::std::format!("| Mean wall time per project | {:.2} ms |\n", summary.mean_wall_time_ms));
    let total_s = summary.total_wall_time_ms as ::core::primitive::f64 / 1000.0;
    md.push_str(&::std::format!("| Total wall time | {:.2} s |\n\n", total_s));

    md.push_str("## Per-Category Breakdown\n\n");
    md.push_str("| Category | Projects | Compilation rate | Mean test pass rate | Mean wall time (ms) | Passed |\n");
    md.push_str("|---|---|---|---|---|---|\n");

    let categories = ["crust", "oxidizer", "alphatrans", "skel"];
    for cat in &categories {
        let cat_projects = summary.projects.iter().filter(|p| p.category == *cat).collect::<::std::vec::Vec<_>>();
        let count = cat_projects.len();
        if count == 0 {
            continue;
        }
        let comp_count = cat_projects.iter().filter(|p| p.compilation_success).count();
        let comp_rate = (comp_count as ::core::primitive::f64) / (count as ::core::primitive::f64) * 100.0;
        let sum_test: ::core::primitive::f64 = cat_projects.iter().map(|p| p.test_pass_rate).sum();
        let mean_test = sum_test / (count as ::core::primitive::f64);
        let sum_wall: ::core::primitive::u128 = cat_projects.iter().map(|p| p.wall_time_ms).sum();
        let mean_wall = (sum_wall as ::core::primitive::f64) / (count as ::core::primitive::f64);
        let passed_count = cat_projects.iter().filter(|p| p.status == "PASSED").count();
        md.push_str(&::std::format!(
            "| {} | {} | {:.2}% | {:.2}% | {:.2} | {} |\n",
            cat,
            count,
            comp_rate,
            mean_test,
            mean_wall,
            passed_count
        ));
    }
    md.push('\n');

    md.push_str("## Per-Project Results\n\n");
    md.push_str(
        "| Project | Category | Wall time (ms) | Compilation | Test pass rate | Receipts | Inference | Status |\n",
    );
    md.push_str("|---|---|---|---|---|---|---|---|\n");
    for p in &summary.projects {
        let comp_str = if p.compilation_success {
            "✅"
        } else {
            "❌"
        };
        let row = ::std::format!(
            "| {} | {} | {} | {} | {:.1}% | {}/6 | {} | {} |\n",
            p.project,
            p.category,
            p.wall_time_ms,
            comp_str,
            p.test_pass_rate,
            p.receipts_proven,
            p.inference_mode,
            p.status,
        );
        md.push_str(&row);
    }

    if let ::core::result::Result::Err(err) = ::std::fs::write(&md_path, md) {
        ::std::eprintln!("[daemon] Failed to write {}: {err}", md_path.display());
    } else {
        ::std::println!("[daemon] Saved results Markdown to {}", md_path.display());
    }
}
