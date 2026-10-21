//! Canonical migration campaign configuration schema.
//!
//! Single source of truth for the `migration.yml` document accepted by every
//! frontend (`bins/daemon --config`, `bins/interactive /start`, and the
//! `/mkcfg` configurator wizard). All previously scattered per-frontend
//! parse structs (`bins/daemon` `BenchConfig`, `bins/interactive` mock
//! `ModernizeConfig`) deserialize into these types.

/// Parsed migration campaign configuration document. Every frontend
/// deserializes its `migration.yml` into this canonical type.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize, ::serde::Serialize)]
#[serde(default)]
pub struct CampaignConfig {
    /// Campaign identifier printed in reports and logs.
    pub campaign_id: ::std::string::String,
    /// Execution mode: `hitl` or `full_auto`.
    pub execution_mode: self::ExecutionMode,
    /// Path section.
    pub paths: self::PathsSection,
    /// Stack section.
    pub stack: self::StackSection,
    /// Inference section.
    pub inference: self::InferenceSection,
    /// Verification section.
    pub verification: self::VerificationSection,
    /// Human-in-the-loop gate section.
    pub hitl: self::HitlSection,
    /// ARIADNE solver tuning section.
    pub ariadne: self::AriadneSection,
}

/// Execution mode of a modernization campaign.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::cmp::PartialEq, ::core::cmp::Eq, ::core::default::Default, ::serde::Deserialize, ::serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// Human in the loop mode pausing on configured gates.
    #[default]
    Hitl,
    /// Full auto mode advancing across gates without operator approval.
    FullAuto,
}

/// Path section of the campaign configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize, ::serde::Serialize)]
#[serde(default)]
pub struct PathsSection {
    /// Directory holding the pristine legacy source tree.
    pub input_legacy_dir: ::std::path::PathBuf,
    /// Scratchpad root under which the translated workspace lives.
    pub workspace_root: ::std::path::PathBuf,
    /// Glob patterns the campaign must not write to.
    pub off_limits_globs: ::std::vec::Vec<::std::string::String>,
}

/// Stack section of the campaign configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize, ::serde::Serialize)]
#[serde(default)]
pub struct StackSection {
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
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize, ::serde::Serialize)]
#[serde(default)]
pub struct InferenceSection {
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
    pub inference_timeout_secs: ::core::primitive::u64,
    /// Target build timeout in seconds.
    pub build_timeout_secs: ::core::primitive::u64,
    /// Target test timeout in seconds.
    pub test_timeout_secs: ::core::primitive::u64,
}

/// Verification section of the campaign configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize, ::serde::Serialize)]
#[serde(default)]
pub struct VerificationSection {
    /// Self-heal retry budget for the solver loop.
    pub max_self_heal_retries: ::core::primitive::u32,
    /// Flag that demands byte-identical oracle output.
    pub require_byte_identical_io: ::core::primitive::bool,
}

/// Escalation policy for unresolved extrinsic contract conflicts.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::cmp::PartialEq, ::core::cmp::Eq, ::core::default::Default, ::serde::Deserialize, ::serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictEscalation {
    /// Resolve conflicts through the automated contract negotiation path.
    #[default]
    AutoNegotiate,
    /// Pause the statechart and ask the human operator.
    AskHuman,
}

/// Human-in-the-loop gate section of the campaign configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize, ::serde::Serialize)]
#[serde(default)]
pub struct HitlSection {
    /// Human signoff required after S4 before S5.
    pub gate_on_reimagine: ::core::primitive::bool,
    /// Human signoff required before rolling S5 back to S2.
    pub gate_on_macro_regression: ::core::primitive::bool,
    /// Policy for unresolved extrinsic contract conflicts.
    pub extrinsic_conflict_escalation: self::ConflictEscalation,
}

/// Reward weight triple for the gated hierarchical reward function.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize, ::serde::Serialize)]
#[serde(default)]
pub struct RewardWeights {
    /// Weight for clean compilation.
    pub compile: ::core::primitive::f64,
    /// Weight for characterization test parity.
    pub characterization_parity: ::core::primitive::f64,
    /// Weight for semantic hardening score.
    pub hardening_score: ::core::primitive::f64,
}

/// Inference backend coordinates for generative subagents.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize, ::serde::Serialize)]
#[serde(default)]
pub struct LlmBackendSection {
    /// OpenAI-compatible base URL of the local inference server.
    pub base_url: ::std::string::String,
    /// Model identifier served at the endpoint.
    pub model: ::std::string::String,
    /// Completion token budget.
    pub max_tokens: ::core::primitive::u32,
    /// Sampling temperature.
    pub temperature: ::core::primitive::f64,
    /// Request timeout in seconds.
    pub timeout_secs: ::core::primitive::u64,
}

/// Stage 3a harness timeouts.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize, ::serde::Serialize)]
#[serde(default)]
pub struct CargoHarnessSection {
    /// Build (check) timeout in seconds.
    pub build_timeout_secs: ::core::primitive::u64,
    /// Test timeout in seconds.
    pub test_timeout_secs: ::core::primitive::u64,
}

/// ARIADNE solver tuning section of the campaign configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Deserialize, ::serde::Serialize)]
#[serde(default)]
pub struct AriadneSection {
    /// Maximum rollouts (B_max) per module in state M2. The legacy alias
    /// max_rollouts keeps the 118 generated benchmark configs parsing.
    #[serde(alias = "max_rollouts")]
    pub engine_budget_per_module: ::core::primitive::usize,
    /// Exploration constant for Softmax-UCB.
    pub ucb_exploration_c: ::core::primitive::f64,
    /// Softmax temperature for prior sampling.
    pub softmax_temperature: ::core::primitive::f64,
    /// Reward weights for the gated hierarchical reward function.
    pub reward_weights: self::RewardWeights,
    /// Generative agent inference backend coordinates.
    pub llm_backend: self::LlmBackendSection,
    /// Stage 3a real-workspace harness timeouts.
    pub cargo_harness: self::CargoHarnessSection,
}

impl self::CampaignConfig {
    /// Read and parse one campaign YAML file from disk.
    pub fn from_yaml_path(
        path: &::std::path::Path,
    ) -> ::core::result::Result<Self, ::std::string::String> {
        let read_res = ::std::fs::read_to_string(path);
        let text = match read_res {
            ::core::result::Result::Ok(value) => value,
            ::core::result::Result::Err(read_error) => {
                return ::core::result::Result::Err(::std::format!(
                    "config read failed: {read_error}"
                ));
            },
        };
        Self::from_yaml_text(&text)
    }

    /// Parse one campaign YAML document.
    pub fn from_yaml_text(
        yaml_text: &::core::primitive::str,
    ) -> ::core::result::Result<Self, ::std::string::String> {
        let parse_res =
            ::serde_yaml_ng::from_str::<Self>(yaml_text);
        let parsed = match parse_res {
            ::core::result::Result::Ok(value) => value,
            ::core::result::Result::Err(parse_error) => {
                return ::core::result::Result::Err(::std::format!(
                    "config parse failed: {parse_error}"
                ));
            },
        };
        ::core::result::Result::Ok(parsed)
    }

    /// Serialize this configuration back to YAML text.
    pub fn to_yaml_text(&self) -> ::std::string::String {
        ::serde_yaml_ng::to_string(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_full_generated_document() {
        let path_section = ::core::concat!(
            "paths:\n",
            "  input_legacy_dir: benchmarks/assets/ReCodeAgent/data/tool_projects/skel/bst/python\n",
            "  workspace_root: benchmarks/outputs/skel/bst/javascript\n",
            "  off_limits_globs: []\n",
        );
        let stack_section = ::core::concat!(
            "stack:\n",
            "  source_language: python\n",
            "  binary_artifacts: []\n",
            "  use_ghidra_rea: false\n",
            "  target_language: javascript\n",
            "  target_edition_or_framework: '2024'\n",
            "  target_build_cmd: node -c translated.js\n",
            "  target_test_cmd: node translated.js\n",
            "  legacy_test_harness_cmd: pytest\n",
        );
        let inference_section = ::core::concat!(
            "inference:\n",
            "  base_url: ''\n",
            "  model: ashalliant/qwen3.8-27b-abliterated\n",
            "  max_parallel_shards: 2\n",
            "  max_shards: 2\n",
            "  inference_max_tokens: 32768\n",
            "  inference_temperature: 0.2\n",
            "  inference_timeout_secs: 600\n",
            "  build_timeout_secs: 120\n",
            "  test_timeout_secs: 120\n",
        );
        let verification_section = ::core::concat!(
            "verification:\n",
            "  max_self_heal_retries: 3\n",
            "  require_byte_identical_io: false\n",
        );
        let yaml_text = ::std::format!(
            "campaign_id: skel-bst-bench\nexecution_mode: full_auto\n{path_section}{stack_section}{inference_section}{verification_section}"
        );
        let parsed_res = super::CampaignConfig::from_yaml_text(&yaml_text);
        let parsed = parsed_res.expect("full document parses");
        ::core::assert_eq!(parsed.campaign_id, "skel-bst-bench");
        ::core::assert_eq!(parsed.execution_mode, super::ExecutionMode::FullAuto);
        ::core::assert_eq!(parsed.stack.source_language, "python");
        ::core::assert_eq!(parsed.stack.target_language, "javascript");
        ::core::assert_eq!(parsed.inference.inference_max_tokens, 32768);
        ::core::assert!(!parsed.hitl.gate_on_reimagine);
    }

    #[test]
    fn legacy_max_rollouts_alias_parses() {
        let yaml_text = "campaign_id: legacy\nariadne:\n  max_rollouts: 16\n";
        let parsed_res = super::CampaignConfig::from_yaml_text(yaml_text);
        let parsed = parsed_res.expect("legacy ariadne field parses");
        ::core::assert_eq!(parsed.ariadne.engine_budget_per_module, 16);
    }

    #[test]
    fn missing_sections_fall_back_to_defaults() {
        let yaml_text = "campaign_id: minimal\n";
        let parsed_res = super::CampaignConfig::from_yaml_text(yaml_text);
        let parsed = parsed_res.expect("minimal document parses");
        ::core::assert_eq!(parsed.campaign_id, "minimal");
        ::core::assert_eq!(parsed.execution_mode, super::ExecutionMode::Hitl);
        ::core::assert_eq!(parsed.stack.target_language, "");
        ::core::assert_eq!(parsed.ariadne.reward_weights.compile, 0.0);
        ::core::assert_eq!(
            parsed.hitl.extrinsic_conflict_escalation,
            super::ConflictEscalation::AutoNegotiate
        );
    }

    #[test]
    fn round_trips_through_yaml() {
        let yaml_text = "campaign_id: roundtrip\nexecution_mode: hitl\n";
        let parsed_res = super::CampaignConfig::from_yaml_text(yaml_text);
        let parsed = parsed_res.expect("document parses");
        let serialized = parsed.to_yaml_text();
        let reparsed_res = super::CampaignConfig::from_yaml_text(&serialized);
        let reparsed = reparsed_res.expect("round trip parses");
        ::core::assert_eq!(reparsed.campaign_id, "roundtrip");
        ::core::assert_eq!(reparsed.execution_mode, super::ExecutionMode::Hitl);
    }

    #[test]
    fn rejects_malformed_documents() {
        let yaml_text = "execution_mode: [not, a, scalar]\n";
        let parsed_res = super::CampaignConfig::from_yaml_text(yaml_text);
        ::core::assert!(parsed_res.is_err());
    }
}
