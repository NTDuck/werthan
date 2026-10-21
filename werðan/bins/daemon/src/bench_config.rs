//! Campaign configuration schema parsed from migration YAML files.
//! Mirrors the ReCodeAgent generated configs under benchmarks/configs/generated.

/// Default directory holding per-project result JSON files.
pub fn result_dir_default() -> ::std::path::PathBuf {
    let env_val = ::core::env!("BENCH_RESULTS_DIR");
    ::std::path::PathBuf::from(env_val)
}

/// Resolve the repository root from the daemon manifest location.
pub fn repo_root() -> ::std::path::PathBuf {
    let manifest_dir = ::std::path::PathBuf::from(::core::env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("..").join("..").join("..")
}

/// Read the inference base URL from the environment.
pub fn default_base_url() -> ::std::string::String {
    ::core::env!("BASE_URL").to_string()
}

/// Read the default model identifier from the environment.
pub fn default_model_id() -> ::core::option::Option<::std::string::String> {
    let value = ::core::env!("MODEL_ID");
    if value.is_empty() {
        ::core::option::Option::None
    } else {
        ::core::option::Option::Some(value.to_string())
    }
}

/// Schema default for the cargo check timeout in seconds.
fn default_build_timeout() -> ::core::primitive::u64 {
    ::core::env!("BENCH_BUILD_TIMEOUT_SECS")
        .parse::<::core::primitive::u64>()
        .expect("BENCH_BUILD_TIMEOUT_SECS must be a valid u64")
}

/// Schema default for the cargo test timeout in seconds.
fn default_test_timeout() -> ::core::primitive::u64 {
    ::core::env!("BENCH_TEST_TIMEOUT_SECS")
        .parse::<::core::primitive::u64>()
        .expect("BENCH_TEST_TIMEOUT_SECS must be a valid u64")
}

/// Schema default for the inference request timeout in seconds.
fn default_inference_timeout() -> ::core::primitive::u64 {
    ::core::env!("BENCH_INFERENCE_TIMEOUT_SECS")
        .parse::<::core::primitive::u64>()
        .expect("BENCH_INFERENCE_TIMEOUT_SECS must be a valid u64")
}

/// Path section of the campaign configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize)]
#[serde(default)]
pub struct PathsConfig {
    /// Legacy source directory that the campaign reads.
    pub input_legacy_dir: ::std::path::PathBuf,
    /// Standalone cargo workspace holding the Rust target state.
    pub workspace_root: ::std::path::PathBuf,
    /// Glob patterns the campaign must not write to.
    pub off_limits_globs: ::std::vec::Vec<::std::string::String>,
}

/// Stack section of the campaign configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize)]
#[serde(default)]
pub struct StackConfig {
    /// Legacy source language identifier such as c or go.
    pub source_language: ::std::string::String,
    /// Binary artifacts listed by the campaign.
    pub binary_artifacts: ::std::vec::Vec<::std::string::String>,
    /// Flag that turns on Ghidra REA analysis.
    pub use_ghidra_rea: ::core::primitive::bool,
    /// Target language identifier such as rust.
    pub target_language: ::std::string::String,
    /// Target edition or framework string.
    pub target_edition_or_framework: ::std::string::String,
    /// Build command executed in the target workspace.
    pub target_build_cmd: ::std::string::String,
    /// Test command executed in the target workspace.
    pub target_test_cmd: ::std::string::String,
    /// Legacy test harness command for oracle runs.
    pub legacy_test_harness_cmd: ::std::string::String,
}

/// Inference section of the campaign configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize)]
#[serde(default)]
pub struct InferenceConfig {
    /// Explicit base URL. Empty means the environment or repo default applies.
    pub base_url: ::std::string::String,
    /// Explicit model override. Empty means the environment or server default applies.
    pub model: ::std::string::String,
    /// Upper bound on parallel inference shards.
    pub max_parallel_shards: ::core::primitive::u32,
    /// Upper bound on inference shards.
    pub max_shards: ::core::primitive::u32,
    /// Completion token budget per request.
    pub inference_max_tokens: ::core::primitive::u32,
    /// Sampling temperature for inference requests.
    pub inference_temperature: ::core::primitive::f64,
    /// Inference request timeout in seconds.
    #[serde(default = "self::default_inference_timeout")]
    pub inference_timeout_secs: ::core::primitive::u64,
    /// Cargo check timeout in seconds.
    #[serde(default = "self::default_build_timeout")]
    pub build_timeout_secs: ::core::primitive::u64,
    /// Cargo test timeout in seconds.
    #[serde(default = "self::default_test_timeout")]
    pub test_timeout_secs: ::core::primitive::u64,
}

impl self::InferenceConfig {
    /// Return the base URL with the empty-string fallback applied.
    pub fn effective_base_url(&self) -> ::std::string::String {
        let configured = self.base_url.clone();
        let has_configured = !configured.is_empty();
        if has_configured {
            return configured;
        }
        self::default_base_url()
    }

    /// Return the model override with the environment fallback applied.
    pub fn effective_model(&self) -> ::core::option::Option<::std::string::String> {
        let configured = self.model.clone();
        let has_configured = !configured.is_empty();
        if has_configured {
            return ::core::option::Option::Some(configured);
        }
        self::default_model_id()
    }
}

/// Verification section of the campaign configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize)]
#[serde(default)]
pub struct VerificationConfig {
    /// Self-heal retry budget for the solver loop.
    pub max_self_heal_retries: ::core::primitive::u32,
    /// Flag that demands byte-identical oracle output.
    pub require_byte_identical_io: ::core::primitive::bool,
}

/// Optional ARIADNE tuning section absent from most generated configs.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize)]
#[serde(default)]
pub struct AriadneConfig {
    /// Rollout budget override for the MCTS search.
    pub max_rollouts: ::core::option::Option<::core::primitive::usize>,
}

/// Parsed migration campaign configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize)]
#[serde(default)]
pub struct BenchConfig {
    /// Campaign identifier printed in reports.
    pub campaign_id: ::std::string::String,
    /// Execution mode such as full_auto.
    pub execution_mode: ::std::string::String,
    /// Path section.
    pub paths: self::PathsConfig,
    /// Stack section.
    pub stack: self::StackConfig,
    /// Inference section.
    pub inference: self::InferenceConfig,
    /// Verification section.
    pub verification: self::VerificationConfig,
    /// Optional ARIADNE section.
    pub ariadne: self::AriadneConfig,
}

impl self::BenchConfig {
    /// Return the result JSON path for this campaign.
    pub fn result_path(&self) -> ::std::path::PathBuf {
        let result_dir = self.result_dir();
        let label = self.project_label();
        let file_name = ::std::format!("{label}.json");
        result_dir.join(file_name)
    }

    /// Return the directory that holds per-project result JSON files.
    pub fn result_dir(&self) -> ::std::path::PathBuf {
        self::result_dir_default()
    }

    /// Derive the result label from the workspace root layout outputs/<category>/<project>.
    pub fn project_label(&self) -> ::std::string::String {
        let root_text = self.paths.workspace_root.to_string_lossy();
        let segments = root_text
            .split('/')
            .collect::<::std::vec::Vec<&::core::primitive::str>>();
        let mut label = ::std::string::String::new();
        for (idx, segment) in segments.iter().enumerate() {
            let found = *segment == "outputs";
            if found && idx + 2 < segments.len() {
                label = ::std::format!("{}-{}", segments[idx + 1], segments[idx + 2]);
            }
        }
        let label_empty = label.is_empty();
        if label_empty {
            return self.campaign_id.clone();
        }
        label
    }

}

/// Read and parse one campaign YAML file.
pub fn load_bench_config(
    path: &::std::path::Path,
) -> ::core::result::Result<self::BenchConfig, ::std::string::String> {
    let read_res = ::std::fs::read_to_string(path);
    let text = match read_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(read_error) => {
            return ::core::result::Result::Err(::std::format!(
                "config read failed: {read_error}"
            ));
        },
    };
    let parse_res = ::serde_yaml_ng::from_str::<self::BenchConfig>(&text);
    let bench_config = match parse_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(parse_error) => {
            return ::core::result::Result::Err(::std::format!(
                "config parse failed: {parse_error}"
            ));
        },
    };
    let campaign_empty = bench_config.campaign_id.is_empty();
    if campaign_empty {
        return ::core::result::Result::Err("campaign_id is empty".to_string());
    }
    ::core::result::Result::Ok(bench_config)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_minimal_campaign() {
        let text = "campaign_id: test-campaign\nexecution_mode: full_auto\npaths:\n  input_legacy_dir: legacy\n  workspace_root: workspace\nstack:\n  source_language: c\n  target_language: rust\n  target_build_cmd: cargo check\n  target_test_cmd: cargo test\ninference:\n  model: test-model\nverification:\n  max_self_heal_retries: 3\n";
        let parse_res = ::serde_yaml_ng::from_str::<super::BenchConfig>(text);
        let bench_config = parse_res.expect("minimal config parses");
        ::std::assert_eq!(bench_config.campaign_id, "test-campaign");
        let model = bench_config.inference.effective_model();
        ::std::assert_eq!(model, ::core::option::Option::Some("test-model".to_string()));
        ::std::assert_eq!(bench_config.inference.build_timeout_secs, 120);
        ::std::assert_eq!(bench_config.inference.test_timeout_secs, 120);
        ::std::assert_eq!(bench_config.ariadne.max_rollouts, ::core::option::Option::None);
    }

    #[test]
    fn label_derives_from_outputs_layout() {
        let mut bench_config = super::BenchConfig {
            campaign_id: "fallback-bench".to_string(),
            ..::core::default::Default::default()
        };
        bench_config.paths.workspace_root =
            ::std::path::PathBuf::from("benchmarks/outputs/crust/vec/rust");
        let label = bench_config.project_label();
        ::std::assert_eq!(label, "crust-vec");
        bench_config.paths.workspace_root = ::std::path::PathBuf::from("somewhere/else");
        let fallback_label = bench_config.project_label();
        ::std::assert_eq!(fallback_label, "fallback-bench");
    }
}
