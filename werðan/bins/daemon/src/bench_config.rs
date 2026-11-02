//! Campaign configuration facade over the canonical configschema crate.
//! The daemon keeps its historical entry points (`load_bench_config`, the
//! result-path helpers, and the inference fallbacks) while delegating the
//! document schema to `::configschema::CampaignConfig`.

pub use ::configschema::CampaignConfig;

/// Effective defaults for the canonical inference section.
pub trait InferenceExt {
    /// Return the base URL with the empty-string fallback applied.
    fn effective_base_url(&self) -> ::std::string::String;

    /// Return the model override with the environment fallback applied.
    fn effective_model(&self) -> ::core::option::Option<::std::string::String>;
}

impl self::InferenceExt for ::configschema::InferenceSection {
    /// Return the base URL with the empty-string fallback applied.
    fn effective_base_url(&self) -> ::std::string::String {
        let configured = self.base_url.clone();
        let has_configured = !configured.is_empty();
        if has_configured {
            return configured;
        }
        self::default_base_url()
    }

    /// Return the model override with the environment fallback applied.
    fn effective_model(&self) -> ::core::option::Option<::std::string::String> {
        let configured = self.model.clone();
        let has_configured = !configured.is_empty();
        if has_configured {
            return ::core::option::Option::Some(configured);
        }
        self::default_model_id()
    }
}

/// Effective result-path behavior for the canonical campaign document.
pub trait CampaignExt {
    /// Return the result JSON path for this campaign.
    fn result_path(&self) -> ::std::path::PathBuf;

    /// Return the directory that holds per-project result JSON files.
    fn result_dir(&self) -> ::std::path::PathBuf;

    /// Derive the result label from the workspace root layout outputs/<category>/<project>.
    fn project_label(&self) -> ::std::string::String;
}

impl self::CampaignExt for ::configschema::CampaignConfig {
    /// Return the result JSON path for this campaign.
    fn result_path(&self) -> ::std::path::PathBuf {
        let result_dir = self.result_dir();
        let label = self.project_label();
        let file_name = ::std::format!("{label}.json");
        result_dir.join(file_name)
    }

    /// Return the directory that holds per-project result JSON files.
    fn result_dir(&self) -> ::std::path::PathBuf {
        self::result_dir_default()
    }

    /// Derive the result label from the workspace root layout outputs/<category>/<project>.
    fn project_label(&self) -> ::std::string::String {
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

/// Read and parse one campaign YAML file, rejecting empty campaign ids.
pub fn load_bench_config(
    path: &::std::path::Path,
) -> ::core::result::Result<self::CampaignConfig, ::std::string::String> {
    let parse_res = ::configschema::CampaignConfig::from_yaml_path(path);
    let bench_config = match parse_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(parse_error) => {
            return ::core::result::Result::Err(parse_error);
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
    use super::CampaignExt;
    use super::InferenceExt;

    #[test]
    fn parse_minimal_campaign() {
        let text = "campaign_id: test-campaign\nexecution_mode: full_auto\npaths:\n  input_legacy_dir: legacy\n  workspace_root: workspace\nstack:\n  source_language: c\n  target_language: rust\n  target_build_cmd: cargo check\n  target_test_cmd: cargo test\ninference:\n  model: test-model\n  build_timeout_secs: 120\n  test_timeout_secs: 120\nverification:\n  max_self_heal_retries: 3\n";
        let parse_res = ::serde_yaml_ng::from_str::<super::CampaignConfig>(text);
        let bench_config = parse_res.expect("minimal config parses");
        ::std::assert_eq!(bench_config.campaign_id, "test-campaign");
        let model = bench_config.inference.effective_model();
        ::std::assert_eq!(model, ::core::option::Option::Some("test-model".to_string()));
        ::std::assert_eq!(bench_config.inference.build_timeout_secs, 120);
        ::std::assert_eq!(bench_config.inference.test_timeout_secs, 120);
        ::std::assert_eq!(bench_config.ariadne.engine_budget_per_module, 0);
    }

    #[test]
    fn label_derives_from_outputs_layout() {
        let mut bench_config = super::CampaignConfig {
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
