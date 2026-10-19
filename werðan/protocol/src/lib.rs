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

/// Inference model configuration.
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
}

fn default_max_parallel_shards() -> ::core::primitive::u32 {
    2
}

fn default_user_input_mode() -> ::std::string::String {
    "wait".to_string()
}

impl ::core::default::Default for self::ModernizeInference {
    fn default() -> Self {
        Self {
            ninfer_base_url: { ::std::string::String::new() },
            deep_model: { ::std::string::String::new() },
            fast_model: { ::std::string::String::new() },
            max_parallel_shards: { self::default_max_parallel_shards() },
            user_input_mode: { self::default_user_input_mode() },
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
    ::core::cmp::Eq,
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
