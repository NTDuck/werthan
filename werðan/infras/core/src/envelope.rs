//! Task Activation Envelope and context hydration schemas for stateless ephemeral agents.
//! Formalized per Definition 4 in `METHODOLOGY.md`.

/// Activation mode selecting the prompt hydration strategy.
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
pub enum ActivationMode {
    /// Initial cold-start execution.
    Initial,
    /// Critic retry driven by Stage-1 validator traceback or Stage-2 critique.
    CriticRetry,
    /// Warm-start regression preserving existing MCTS search trees and test suites.
    WarmStartRegression,
}

impl ::core::fmt::Display for self::ActivationMode {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let label = match self {
            Self::Initial => "INITIAL",
            Self::CriticRetry => "CRITIC_RETRY",
            Self::WarmStartRegression => "WARM_START_REGRESSION",
        };
        ::core::write!(f, "{label}")
    }
}

/// Reasoning thinking budget mapped to deterministic task requirements.
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
pub enum ThinkingBudget {
    /// Fast/low budget for surgical code repair and deterministic map workers.
    Low,
    /// Standard budget for general translation and testing.
    Standard,
    /// High thinking effort for MCTS strategy generation, architecture design, and triage.
    High,
}

/// Distinct agent roles in the 7-phase modernization methodology.
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
pub enum AgentRole {
    // S0
    PreflightProbe,
    // S1
    ShardAssessor,
    AssessmentSynthesizer,
    AssessmentCritic,
    // S2
    ImplicitCouplingScanner,
    TopologyCritic,
    // S3
    RuleMiner,
    CrossModuleRuleLinker,
    RuleAuditor,
    // S4
    MacroArchitect,
    ContractStubber,
    ArchitectureCritic,
    // S5: ARIADNE Cluster
    AriadneStrategy,
    AriadneGenerator,
    AriadneTester,
    AriadnePatcher,
    AriadneEvaluator,
    // S6
    IntegrationVerifier,
    SecurityAuditor,
    ReleaseAuditor,
    // Arbiter
    OrchestratorArbiter,

    // Legacy backwards-compatible variants
    RootOrchestrator,
    LegacyAnalyst,
    BinaryHistorian,
    BusinessRulesExtractor,
    BoundaryMediator,
    ModuleLead,
    TestEngineer,
    TargetCoder,
}

impl self::AgentRole {
    /// Return standard thinking budget for this role.
    pub fn default_thinking_budget(self) -> self::ThinkingBudget {
        match self {
            Self::AriadneStrategy | Self::MacroArchitect | Self::OrchestratorArbiter => {
                self::ThinkingBudget::High
            },
            Self::AriadnePatcher | Self::ShardAssessor | Self::ImplicitCouplingScanner => {
                self::ThinkingBudget::Low
            },
            _ => self::ThinkingBudget::Standard,
        }
    }

    /// Check if this role requires candidate code read-isolation.
    /// In particular, adversarial test generators MUST NOT read candidate code drafts.
    pub fn is_test_generator(self) -> ::core::primitive::bool {
        ::core::matches!(self, Self::AriadneTester | Self::TestEngineer)
    }
}

/// Obligation classification status.
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
pub enum ObligationStatus {
    Discovered,
    Claimed,
    Translating,
    Verified,
    Rejected,
}

/// Reconstruction obligation candidate discovered from legacy sources or binaries.
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
pub struct ObligationCandidate {
    /// Symbol or function name.
    #[builder(into)]
    pub name: ::std::string::String,
    /// Category or kind of symbol.
    #[builder(into)]
    pub kind: ::std::string::String,
    /// Hexadecimal address or offset.
    #[builder(into)]
    pub address_hex: ::std::string::String,
    /// Source file or binary path.
    #[builder(into)]
    pub binary_path: ::std::string::String,
    /// Current obligation status.
    pub status: self::ObligationStatus,
    /// Target language modern symbol implementation.
    #[builder(into)]
    pub target_implementation: ::core::option::Option<::std::string::String>,
    /// Linked characterization test name.
    #[builder(into)]
    pub characterization_test: ::core::option::Option<::std::string::String>,
}

/// Bounded 1-hop subgraph slice relevant to an active translation unit.
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
pub struct GraphSlice {
    /// Target module identifier or vertex ID.
    #[builder(into)]
    pub target_ref: ::std::string::String,
    /// Upstream parent interface contracts: map parent_id -> contract_spec.
    #[serde(default)]
    pub parent_contracts: ::std::collections::BTreeMap<::std::string::String, ::std::string::String>,
    /// Export obligations expected by downstream children.
    #[serde(default)]
    pub export_obligations: ::std::vec::Vec<::std::string::String>,
    /// Verified Given/When/Then business rule cards for this vertex.
    #[serde(default)]
    pub rule_cards: ::std::vec::Vec<::std::string::String>,
    /// Legacy source code snippets or AST extracts.
    #[serde(default)]
    pub legacy_snippets: ::std::vec::Vec<::std::string::String>,
}

/// Failure critique, contract conflict, or compiler diagnostic trace.
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
pub struct DirectiveTrace {
    /// Structured feedback or failure critique from critics.
    #[serde(default)]
    #[builder(into)]
    pub failure_critique: ::core::option::Option<::std::string::String>,
    /// Raw compiler, linter, or test diagnostics.
    #[serde(default)]
    pub compiler_diagnostics: ::std::vec::Vec<::std::string::String>,
    /// Contract conflict delta specification from downstream children.
    #[serde(default)]
    #[builder(into)]
    pub contract_conflict: ::core::option::Option<::std::string::String>,
}

impl ::core::default::Default for self::DirectiveTrace {
    fn default() -> Self {
        Self {
            failure_critique: ::core::option::Option::None,
            compiler_diagnostics: ::std::vec::Vec::new(),
            contract_conflict: ::core::option::Option::None,
        }
    }
}

/// Bounded set of deterministic AST and LSP navigation tools available to the agent.
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
pub struct ToolBindings {
    /// Tool identifiers exposed in the envelope.
    pub allowed_tools: ::std::vec::Vec<::std::string::String>,
}

impl self::ToolBindings {
    /// Read-only discovery tool roster.
    pub fn discovery_roster() -> Self {
        Self {
            allowed_tools: [
                "read_file".to_string(),
                "search_rg".to_string(),
                "tree_sitter_query".to_string(),
                "lsp_find_references".to_string(),
            ]
            .into_iter()
            .collect::<::std::vec::Vec<::std::string::String>>(),
        }
    }

    /// Read-only AST/LSP navigation tool roster for M2 search nodes.
    pub fn mcts_roster() -> Self {
        Self {
            allowed_tools: [
                "read_hashline".to_string(),
                "lsp_typecheck".to_string(),
                "lsp_definitions".to_string(),
                "apply_patch".to_string(),
            ]
            .into_iter()
            .collect::<::std::vec::Vec<::std::string::String>>(),
        }
    }
}

/// Task Activation Envelope (§2 Definition 4 in `METHODOLOGY.md`).
/// Stateless workers receive push-based context hydration through this envelope.
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
pub struct TaskActivationEnvelope {
    /// Activation mode (Initial, CriticRetry, WarmStartRegression).
    pub mode: self::ActivationMode,
    /// Intended agent role.
    pub role: self::AgentRole,
    /// Bounded 1-hop graph slice.
    pub slice: self::GraphSlice,
    /// Directive trace or critique feedback.
    pub directive: self::DirectiveTrace,
    /// Bound deterministic tools.
    pub tools: self::ToolBindings,
    /// Allocated thinking budget for deep reasoning.
    pub thinking_budget: self::ThinkingBudget,
}
