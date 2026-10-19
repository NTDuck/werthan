//! Protocol types, wire formats, workflow states, and modernization configurations.

/// Magic directive extraction (absorbed keywords crate).
pub mod directives;

/// Pane role in the static modernization terminal layout.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::hash::Hash,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
#[serde(rename_all = "snake_case")]
pub enum PaneRole {
    /// Left sixty percent pane for user interactions and orchestration.
    Orchestrator,
    /// Right forty percent pane one for legacy codebase analysis.
    LegacyAnalyst,
    /// Right forty percent pane two for business rule extraction.
    BusinessRulesExtractor,
    /// Right forty percent pane three for translation and verification testing.
    TranslatorTest,
    /// Right forty percent pane four for architectural critiques.
    ArchitectureCritic,
    /// Right forty percent pane five for security and vulnerability auditing.
    SecurityAuditor,
}

impl ::core::fmt::Display for self::PaneRole {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let label = match self {
            Self::Orchestrator => "orchestrator",
            Self::LegacyAnalyst => "legacy_analyst",
            Self::BusinessRulesExtractor => "business_rules_extractor",
            Self::TranslatorTest => "translator_test",
            Self::ArchitectureCritic => "architecture_critic",
            Self::SecurityAuditor => "security_auditor",
        };
        ::core::write!(f, "{label}")
    }
}

/// Commands sent from the modernization daemon to an individual pane.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonToPaneCmd {
    /// Dispatch a model prompt to the pane.
    DispatchPrompt {
        /// Task identifier.
        task_id: ::std::string::String,
        /// Model identifier to select.
        model: ::std::string::String,
        /// Prompt body to submit.
        prompt: ::std::string::String,
        /// List of allowed tool names.
        allowed_tools: ::std::vec::Vec<::std::string::String>,
    },
    /// Set pane input mode.
    SetInputMode {
        /// Interruption mode such as immediate or wait.
        interrupt_mode: ::std::string::String,
        /// Steering mode such as all or one-at-a-time.
        steering_mode: ::std::string::String,
    },
    /// Render notification message in pane.
    RenderNotice {
        /// Severity or notice level.
        level: ::std::string::String,
        /// Notification body text.
        text: ::std::string::String,
    },
    /// Abort active model generation turn.
    AbortCurrentTurn,
}

impl self::DaemonToPaneCmd {
    /// Create dispatch prompt command with magic keywords appended to prompt.
    pub fn dispatch_prompt(
        task_id: &::core::primitive::str,
        model: &::core::primitive::str,
        prompt: &::core::primitive::str,
        allowed_tools: ::std::vec::Vec<::std::string::String>,
    ) -> Self {
        let full_prompt = self::append_magic_keywords(prompt);
        Self::DispatchPrompt {
            task_id: { task_id.to_string() },
            model: { model.to_string() },
            prompt: { full_prompt },
            allowed_tools: { allowed_tools },
        }
    }

    /// Ensure prompt contains magic keywords.
    pub fn ensure_magic_keywords(&mut self) {
        if let Self::DispatchPrompt {
            prompt,
            ..
        } = self
        {
            *prompt = self::append_magic_keywords(prompt);
        }
    }
}

/// Magic keywords appended to all agent prompts across modernization system.
pub const MAGIC_KEYWORDS: &::core::primitive::str = "orchestrate ultrathink workflowz";

/// Append magic keywords to prompt string if not already present.
pub fn append_magic_keywords(prompt: &::core::primitive::str) -> ::std::string::String {
    if prompt.contains(self::MAGIC_KEYWORDS) {
        prompt.to_string()
    } else {
        let trimmed = prompt.trim();
        if trimmed.is_empty() {
            self::MAGIC_KEYWORDS.to_string()
        } else {
            ::std::format!("{trimmed}\n\n{}", self::MAGIC_KEYWORDS)
        }
    }
}

/// Events sent from a pane extension back to the central daemon.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PaneToDaemonEvent {
    /// Register pane process and role.
    RegisterPane {
        /// Pane role.
        role: self::PaneRole,
        /// Process identifier of pane.
        pid: ::core::primitive::u32,
    },
    /// User or extension invoked slash command.
    SlashCommandInvoked {
        /// Command name without slash.
        command: ::std::string::String,
        /// Command arguments string.
        args: ::std::string::String,
    },
    /// Model generation turn settled.
    TurnSettled {
        /// Pane role that settled.
        role: self::PaneRole,
        /// Active task identifier.
        task_id: ::std::string::String,
        /// Assistant textual summary.
        assistant_summary: ::std::string::String,
        /// Files modified during turn.
        modified_files: ::std::vec::Vec<::std::string::String>,
        /// Tokens consumed by turn context.
        context_tokens_used: ::core::primitive::u32,
    },
    /// Tool violation intercepted by warden guardrail.
    ToolViolationHeld {
        /// Pane role attempting violation.
        role: self::PaneRole,
        /// Name of restricted tool.
        tool_name: ::std::string::String,
        /// Quoted warden rule explanation.
        warden_rule_quoted: ::std::string::String,
    },
}

/// Execution mode for modernization campaigns.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::default::Default,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// Human in the loop mode.
    #[default]
    Hitl,
    /// Fully automated autonomous mode.
    FullAuto,
}

/// Workspace paths configuration.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
pub struct ModernizePaths {
    /// Legacy source directory.
    #[serde(default)]
    #[builder(into)]
    pub input_legacy_dir: ::std::string::String,
    /// Workspace root directory.
    #[serde(default)]
    #[builder(into)]
    pub workspace_root: ::std::string::String,
    /// Off limits file glob patterns.
    #[serde(default)]
    pub off_limits_globs: ::std::vec::Vec<::std::string::String>,
}

impl ::core::default::Default for self::ModernizePaths {
    fn default() -> Self {
        let off_limits_globs = ::std::vec::Vec::new();
        Self {
            input_legacy_dir: { ::std::string::String::new() },
            workspace_root: { ::std::string::String::new() },
            off_limits_globs: { off_limits_globs },
        }
    }
}

/// Technology stack configuration.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
pub struct ModernizeStack {
    /// Source programming language.
    #[serde(default)]
    #[builder(into)]
    pub source_language: ::std::string::String,
    /// Binary artifacts for reverse engineering.
    #[serde(default)]
    pub binary_artifacts: ::std::vec::Vec<::std::string::String>,
    /// Whether to use Ghidra reverse engineering tools.
    #[serde(default)]
    pub use_ghidra_rea: ::core::primitive::bool,
    /// Target programming language.
    #[serde(default)]
    #[builder(into)]
    pub target_language: ::std::string::String,
    /// Target edition or framework version.
    #[serde(default)]
    #[builder(into)]
    pub target_edition_or_framework: ::std::string::String,
    /// Command to build target.
    #[serde(default)]
    #[builder(into)]
    pub target_build_cmd: ::std::string::String,
    /// Command to test target.
    #[serde(default)]
    #[builder(into)]
    pub target_test_cmd: ::std::string::String,
    /// Command to run legacy golden test harness.
    #[serde(default)]
    #[builder(into)]
    pub legacy_test_harness_cmd: ::std::string::String,
}

impl ::core::default::Default for self::ModernizeStack {
    fn default() -> Self {
        let binary_artifacts = ::std::vec::Vec::new();
        Self {
            source_language: { ::std::string::String::new() },
            binary_artifacts: { binary_artifacts },
            use_ghidra_rea: { false },
            target_language: { ::std::string::String::new() },
            target_edition_or_framework: { ::std::string::String::new() },
            target_build_cmd: { ::std::string::String::new() },
            target_test_cmd: { ::std::string::String::new() },
            legacy_test_harness_cmd: { ::std::string::String::new() },
        }
    }
}
/// Source file representation for translation.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct TranslationShard {
    /// Relative path within subsystem.
    pub relative_path: ::std::string::String,
    /// Source file content.
    pub source_content: ::std::string::String,
    /// Source programming language.
    pub source_language: ::std::string::String,
    /// Target programming language.
    pub target_language: ::std::string::String,
}


/// Inference model configuration.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
pub struct ModernizeInference {
    /// Base URL for inference service.
    #[serde(default)]
    #[builder(into)]
    pub ninfer_base_url: ::std::string::String,
    /// Model identifier for deep reasoning and transformations.
    #[serde(default)]
    #[builder(into)]
    pub deep_model: ::std::string::String,
    /// Model identifier for fast interactions and shards.
    #[serde(default)]
    #[builder(into)]
    pub fast_model: ::std::string::String,
    /// Maximum concurrent worker shards.
    #[serde(default = "default_max_parallel_shards")]
    pub max_parallel_shards: ::core::primitive::u32,
    /// User input coordination mode.
    #[serde(default = "default_user_input_mode")]
    #[builder(into)]
    pub user_input_mode: ::std::string::String,
    /// Max concurrent worker shards (overrides WERTHAN_MAX_SHARDS env).
    #[serde(default = "default_max_shards")]
    pub max_shards: ::core::primitive::u32,
    /// Inference base URL (overrides WERTHAN_BASE_URL env).
    #[serde(default)]
    #[builder(into)]
    pub base_url: ::std::string::String,
    /// Model identifier (overrides WERTHAN_MODEL env).
    #[serde(default)]
    #[builder(into)]
    pub model: ::std::string::String,
    /// Max tokens per inference response.
    #[serde(default = "default_inference_max_tokens")]
    pub inference_max_tokens: ::core::primitive::u32,
    /// Sampling temperature for inference.
    #[serde(default = "default_inference_temperature")]
    pub inference_temperature: ::core::primitive::f64,
    /// Inference HTTP timeout in seconds.
    #[serde(default = "default_inference_timeout_secs")]
    pub inference_timeout_secs: ::core::primitive::u64,
    /// Build command timeout in seconds.
    #[serde(default = "default_build_timeout_secs")]
    pub build_timeout_secs: ::core::primitive::u64,
    /// Test command timeout in seconds.
    #[serde(default = "default_test_timeout_secs")]
    pub test_timeout_secs: ::core::primitive::u64,
}

fn default_max_parallel_shards() -> ::core::primitive::u32 {
    2
}

fn default_user_input_mode() -> ::std::string::String {
    "wait".to_string()
}
fn default_max_shards() -> ::core::primitive::u32 { 2 }
fn default_inference_max_tokens() -> ::core::primitive::u32 { 32768 }
fn default_inference_temperature() -> ::core::primitive::f64 { 0.2 }
fn default_inference_timeout_secs() -> ::core::primitive::u64 { 600 }
fn default_build_timeout_secs() -> ::core::primitive::u64 { 120 }
fn default_test_timeout_secs() -> ::core::primitive::u64 { 120 }

impl ::core::default::Default for self::ModernizeInference {
    fn default() -> Self {
        Self {
            ninfer_base_url: { ::std::string::String::new() },
            deep_model: { "ashalliant/qwen3.8-27b-abliterated".to_string() },
            fast_model: { "ashalliant/qwen3.8-27b-abliterated".to_string() },
            max_parallel_shards: { self::default_max_parallel_shards() },
            user_input_mode: { self::default_user_input_mode() },
            max_shards: { self::default_max_shards() },
            base_url: { ::std::string::String::new() },
            model: { "ashalliant/qwen3.8-27b-abliterated".to_string() },
            inference_max_tokens: { self::default_inference_max_tokens() },
            inference_temperature: { self::default_inference_temperature() },
            inference_timeout_secs: { self::default_inference_timeout_secs() },
            build_timeout_secs: { self::default_build_timeout_secs() },
            test_timeout_secs: { self::default_test_timeout_secs() },
        }
    }
}

/// Verification parameters configuration.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
pub struct ModernizeVerification {
    /// Maximum self heal retry attempts.
    #[serde(default = "default_max_retries")]
    pub max_self_heal_retries: ::core::primitive::u32,
    /// Whether clean room verification requires byte identical input output.
    #[serde(default)]
    pub require_byte_identical_io: ::core::primitive::bool,
}

fn default_max_retries() -> ::core::primitive::u32 {
    3
}

impl ::core::default::Default for self::ModernizeVerification {
    fn default() -> Self {
        Self {
            max_self_heal_retries: { self::default_max_retries() },
            require_byte_identical_io: { false },
        }
    }
}

/// Root configuration schema for modernization campaigns.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
pub struct ModernizeConfig {
    /// Campaign identifier.
    #[builder(into)]
    pub campaign_id: ::std::string::String,
    /// Execution mode.
    #[serde(default)]
    pub execution_mode: self::ExecutionMode,
    /// Workspace paths.
    #[serde(default)]
    pub paths: self::ModernizePaths,
    /// Stack definitions.
    #[serde(default)]
    pub stack: self::ModernizeStack,
    /// Inference parameters.
    #[serde(default)]
    pub inference: self::ModernizeInference,
    /// Verification rules.
    #[serde(default)]
    pub verification: self::ModernizeVerification,
}

impl ::core::default::Default for self::ModernizeConfig {
    fn default() -> Self {
        let paths = self::ModernizePaths::default();
        let stack = self::ModernizeStack::default();
        let inference = self::ModernizeInference::default();
        let verification = self::ModernizeVerification::default();
        Self {
            campaign_id: { "default-campaign".to_string() },
            execution_mode: { self::ExecutionMode::Hitl },
            paths: { paths },
            stack: { stack },
            inference: { inference },
            verification: { verification },
        }
    }
}

/// Project parameters in migration configuration.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
pub struct ProjectConfig {
    pub input_dir: ::std::path::PathBuf,
    pub output_dir: ::std::path::PathBuf,
    #[serde(default = "default_source_lang")]
    pub source_language: ::std::string::String,
    #[serde(default = "default_target_lang")]
    pub output_language: ::std::string::String,
    #[serde(default = "default_target_edition")]
    pub target_edition: ::std::string::String,
}

impl ::core::default::Default for self::ProjectConfig {
    fn default() -> Self {
        Self {
            input_dir: ::std::path::PathBuf::from("./legacy"),
            output_dir: ::std::path::PathBuf::from("./modernized"),
            source_language: self::default_source_lang(),
            output_language: self::default_target_lang(),
            target_edition: self::default_target_edition(),
        }
    }
}

fn default_source_lang() -> ::std::string::String { "cpp14".to_string() }
fn default_target_lang() -> ::std::string::String { "rust".to_string() }
fn default_target_edition() -> ::std::string::String { "2024".to_string() }

/// Human-in-the-loop configuration parameters.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::default::Default,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
pub struct HitlConfig {
    #[serde(default)]
    pub gate_on_reimagine: ::core::primitive::bool,
    #[serde(default)]
    pub gate_on_macro_regression: ::core::primitive::bool,
    #[serde(default = "default_conflict_escalation")]
    pub extrinsic_conflict_escalation: ::std::string::String,
}

fn default_conflict_escalation() -> ::std::string::String { "auto_negotiate".to_string() }

/// ARIADNE MCTS search configuration parameters.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
pub struct AriadneConfig {
    #[serde(default = "default_mcts_budget")]
    pub mcts_budget_per_module: ::core::primitive::usize,
    #[serde(default = "default_ucb_c")]
    pub ucb_exploration_c: ::core::primitive::f64,
    #[serde(default = "default_softmax_temp")]
    pub softmax_temperature: ::core::primitive::f64,
}

impl ::core::default::Default for self::AriadneConfig {
    fn default() -> Self {
        Self {
            mcts_budget_per_module: self::default_mcts_budget(),
            ucb_exploration_c: self::default_ucb_c(),
            softmax_temperature: self::default_softmax_temp(),
        }
    }
}

fn default_mcts_budget() -> ::core::primitive::usize { 24 }
fn default_ucb_c() -> ::core::primitive::f64 { 1.414 }
fn default_softmax_temp() -> ::core::primitive::f64 { 0.35 }

/// Infras integration preferences.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
pub struct InfrasConfig {
    #[serde(default = "default_true")]
    pub enable_rea: ::core::primitive::bool,
    #[serde(default = "default_guardrail_mode")]
    pub laya_guardrail_mode: ::std::string::String,
}

impl ::core::default::Default for self::InfrasConfig {
    fn default() -> Self {
        Self {
            enable_rea: self::default_true(),
            laya_guardrail_mode: self::default_guardrail_mode(),
        }
    }
}

fn default_true() -> ::core::primitive::bool { true }
fn default_guardrail_mode() -> ::std::string::String { "strict".to_string() }

/// Full migration specification loaded from migration.yml.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
pub struct MigrationConfig {
    #[serde(default = "default_version")]
    pub version: ::std::string::String,
    pub project: self::ProjectConfig,
    #[serde(default)]
    pub hitl: self::HitlConfig,
    #[serde(default)]
    pub ariadne: self::AriadneConfig,
    #[serde(default)]
    pub infras: self::InfrasConfig,
}

fn default_version() -> ::std::string::String { "1.0".to_string() }

impl self::MigrationConfig {
    /// Parse migration config from YAML formatted string.
    pub fn from_yaml_str(yaml: &::core::primitive::str) -> ::core::result::Result<Self, ::serde_yaml_ng::Error> {
        ::serde_yaml_ng::from_str::<Self>(yaml)
    }

    /// Parse migration config from YAML file path.
    pub fn from_yaml_file(path: &::std::path::Path) -> ::core::result::Result<Self, ::std::string::String> {
        let content = match ::std::fs::read_to_string(path) {
            ::core::result::Result::Ok(c) => c,
            ::core::result::Result::Err(e) => return ::core::result::Result::Err(e.to_string()),
        };
        match self::MigrationConfig::from_yaml_str(&content) {
            ::core::result::Result::Ok(cfg) => ::core::result::Result::Ok(cfg),
            ::core::result::Result::Err(e) => ::core::result::Result::Err(e.to_string()),
        }
    }

    /// Serialize to YAML string.
    pub fn to_yaml_string(&self) -> ::core::result::Result<::std::string::String, ::serde_yaml_ng::Error> {
        ::serde_yaml_ng::to_string(self)
    }

    /// Convert to legacy ModernizeConfig.
    pub fn to_modernize_config(&self) -> self::ModernizeConfig {
        let mut mc = self::ModernizeConfig::default();
        mc.paths.input_legacy_dir = self.project.input_dir.to_string_lossy().to_string();
        mc.paths.workspace_root = self.project.output_dir.to_string_lossy().to_string();
        mc.stack.source_language = self.project.source_language.clone();
        mc.stack.target_language = self.project.output_language.clone();
        mc
    }
}

/// Modernization workflow state in deterministic state machine.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
pub enum WorkflowState {
    /// S0: Initial intent framing.
    S0Intent,
    /// S1: Preflight discovery and dependency analysis.
    S1Preflight,
    /// S2: Quantitative assessment and code metrics.
    S2Assess,
    /// S3: Domain clustering and system mapping.
    S3Map,
    /// S4: Business rule extraction into cards.
    S4ExtractRules,
    /// S5: Target architecture brief preparation.
    S5Brief,
    /// S6: Codebase translation and transformation.
    S6Transform,
    /// S7: Characterization and verification testing.
    S7Verify,
    /// S8: Security hardening and vulnerability audit.
    S8Harden,
    /// S9: Workflow completed state.
    S9Completed,
}

impl ::core::fmt::Display for self::WorkflowState {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let label = match self {
            Self::S0Intent => "S0 INTENT",
            Self::S1Preflight => "S1 PREFLIGHT",
            Self::S2Assess => "S2 ASSESS",
            Self::S3Map => "S3 MAP",
            Self::S4ExtractRules => "S4 EXTRACT_RULES",
            Self::S5Brief => "S5 BRIEF",
            Self::S6Transform => "S6 TRANSFORM",
            Self::S7Verify => "S7 VERIFY",
            Self::S8Harden => "S8 HARDEN",
            Self::S9Completed => "S9 COMPLETED",
        };
        ::core::write!(f, "{label}")
    }
}

/// Human decision gate guarding critical workflow transitions.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
pub enum HumanGate {
    /// Gate 1: Preflight questions and clarifications.
    Gate1Preflight,
    /// Gate 2: Business rule review and approval.
    Gate2RuleReview,
    /// Gate 3: Architecture brief sign-off.
    Gate3BriefApproval,
    /// Gate 4: Code transformation diff acceptance.
    Gate4DiffAcceptance,
    /// Gate 5: Test proof and verification receipt.
    Gate5ProofReceipt,
    /// Gate 6: Security audit and patch sign-off.
    Gate6PatchSign,
}

impl ::core::fmt::Display for self::HumanGate {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let label = match self {
            Self::Gate1Preflight => "Gate 1: Preflight Q&As",
            Self::Gate2RuleReview => "Gate 2: Rule Review",
            Self::Gate3BriefApproval => "Gate 3: Brief Approval",
            Self::Gate4DiffAcceptance => "Gate 4: Diff Acceptance",
            Self::Gate5ProofReceipt => "Gate 5: Proof Receipt",
            Self::Gate6PatchSign => "Gate 6: Patch Sign",
        };
        ::core::write!(f, "{label}")
    }
}

/// Decision gate approval status.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
pub enum GateStatus {
    /// Gate decision is pending.
    Pending,
    /// Operator approved the gate.
    Approved,
    /// Operator rejected the gate.
    Rejected,
}

/// Receipt stamp recorded on human gate decision.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema
)]
pub struct ReceiptStamp {
    /// Receipt identifier.
    #[builder(into)]
    pub id: ::std::string::String,
    /// Associated human gate.
    pub gate: self::HumanGate,
    /// Decision status.
    pub status: self::GateStatus,
    /// Approver name or role.
    #[builder(into)]
    pub approver: ::std::string::String,
    /// Notes or reason.
    #[builder(into)]
    pub notes: ::std::string::String,
    /// Cryptographic or digest signature.
    #[builder(into)]
    pub signature_hash: ::std::string::String,
}
