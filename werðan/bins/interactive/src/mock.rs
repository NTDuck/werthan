//! Mock implementations for invalidated infras and topology crates.

/// Mock agent role in topology.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::cmp::PartialEq, ::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    /// Root orchestrator agent.
    RootOrchestrator,
    /// Legacy analyst specialist.
    LegacyAnalyst,
    /// Binary historian specialist.
    BinaryHistorian,
    /// Business rules extractor specialist.
    BusinessRulesExtractor,
    /// Module lead specialist.
    ModuleLead,
    /// Architecture critic specialist.
    ArchitectureCritic,
    /// Security auditor specialist.
    SecurityAuditor,
    /// Boundary mediator specialist.
    BoundaryMediator,
    /// Test engineer specialist.
    TestEngineer,
    /// Target coder specialist.
    TargetCoder,
}

/// Mock modernization workflow state in deterministic state machine.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::cmp::PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum ModernizationState {
    /// Initial intent and preflight phase.
    S0IntentPreflight,
    /// Shard assessment phase.
    S1Assess,
    /// Dependency DAG construction phase.
    S2MapDag,
    /// Business rules extraction phase.
    S3ExtractRules,
    /// Architecture contracts reimagination phase.
    S4BriefContracts,
    /// Iterative DAG transformation phase.
    S5IterativeDag,
    /// System verification and hardening phase.
    S6SystemVerify,
}

impl ::core::fmt::Display for self::ModernizationState {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::S0IntentPreflight => ::core::write!(f, "S0 - Intent and Preflight."),
            Self::S1Assess => ::core::write!(f, "S1 - Assess."),
            Self::S2MapDag => ::core::write!(f, "S2 - Map DAG."),
            Self::S3ExtractRules => ::core::write!(f, "S3 - Extract Rules."),
            Self::S4BriefContracts => ::core::write!(f, "S4 - Brief Contracts."),
            Self::S5IterativeDag => ::core::write!(f, "S5 - Iterative DAG."),
            Self::S6SystemVerify => ::core::write!(f, "S6 - System Verify."),
        }
    }
}

/// Mock human decision gate guarding critical workflow transitions.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::cmp::PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum HumanGate {
    /// Human gate required after phase S4 before phase S5.
    GateOnReimagine,
    /// Human gate required before rolling back from phase S5 to phase S2.
    GateOnMacroRegression,
    /// Human gate required when an extrinsic parent interface conflict cannot be resolved automatically.
    GateOnExtrinsicConflict,
}

impl ::core::fmt::Display for self::HumanGate {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::GateOnReimagine => ::core::write!(f, "Gate - Reimagine Approval."),
            Self::GateOnMacroRegression => ::core::write!(f, "Gate - Macro Regression Approval."),
            Self::GateOnExtrinsicConflict => ::core::write!(f, "Gate - Extrinsic Conflict Escalation."),
        }
    }
}

/// Mock decision gate approval or rejection record.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub enum GateDecision {
    /// Approved gate decision with operator identifier and notes.
    Approved {
        /// Approver identity string.
        approver: ::std::string::String,
        /// Operator approval notes.
        notes: ::std::string::String,
    },
    /// Rejected gate decision with rejection reason.
    Rejected {
        /// Operator rejection reason.
        reason: ::std::string::String,
    },
}

/// Mock execution mode for modernization campaigns.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::cmp::PartialEq, ::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// Human in the loop mode requiring gate approvals.
    Hitl,
    /// Full auto mode advancing automatically across gates.
    FullAuto,
}

/// Mock workspace paths configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModernizePaths {
    /// Input legacy directory.
    pub input_legacy_dir: ::std::string::String,
    /// Output workspace root.
    pub workspace_root: ::std::string::String,
}

impl ::core::default::Default for self::ModernizePaths {
    fn default() -> Self {
        Self {
            input_legacy_dir: "legacy/source".to_string(),
            workspace_root: "workspace/output".to_string(),
        }
    }
}

/// Mock technology stack configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModernizeStack {
    /// Source legacy language.
    pub source_language: ::std::string::String,
    /// Modernized target language.
    pub target_language: ::std::string::String,
}

impl ::core::default::Default for self::ModernizeStack {
    fn default() -> Self {
        Self {
            source_language: "c".to_string(),
            target_language: "rust".to_string(),
        }
    }
}

/// Mock modernization campaign configuration.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModernizeConfig {
    /// Campaign identifier name.
    pub campaign_id: ::std::string::String,
    /// Execution mode toggle.
    pub execution_mode: self::ExecutionMode,
    /// Modernization paths.
    pub paths: self::ModernizePaths,
    /// Modernization technology stack.
    pub stack: self::ModernizeStack,
}

impl ::core::default::Default for self::ModernizeConfig {
    fn default() -> Self {
        Self {
            campaign_id: "default-campaign".to_string(),
            execution_mode: self::ExecutionMode::Hitl,
            paths: self::ModernizePaths::default(),
            stack: self::ModernizeStack::default(),
        }
    }
}

/// Mock receipt stamp recorded on workflow milestones.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct ReceiptStamp {
    /// Associated human gate name.
    pub gate: ::std::string::String,
    /// Cryptographic checksum receipt string.
    pub hash: ::std::string::String,
}

/// Mock blackboard streaming event.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub enum BlackboardEvent {
    /// Obligation candidate registered.
    ObligationAdded {
        /// Obligation symbol name.
        name: ::std::string::String,
        /// Obligation category kind.
        kind: ::std::string::String,
        /// Memory or code address.
        address: ::std::string::String,
    },
    /// Clean rule card added.
    RuleCardAdded {
        /// Rule card identifier.
        id: ::std::string::String,
        /// Rule card title.
        title: ::std::string::String,
        /// Academic or technical citation.
        citation: ::std::string::String,
    },
    /// Worktree todo list updated.
    ModuleWorktreeUpdated {
        /// Target module name.
        module_name: ::std::string::String,
        /// Current build status.
        status: ::std::string::String,
        /// Remaining task count.
        todo_count: ::core::primitive::usize,
    },
    /// Taint detection guardrail event.
    TaintDetected {
        /// File path tainted.
        path: ::std::string::String,
        /// Taint rationale.
        reason: ::std::string::String,
    },
}

/// Mock shared blackboard instance for communication across specialists.
#[derive(::core::clone::Clone)]
pub struct Blackboard {
    tx: ::tokio::sync::broadcast::Sender<self::BlackboardEvent>,
}

impl ::core::fmt::Debug for self::Blackboard {
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        formatter.debug_struct("Blackboard").finish_non_exhaustive()
    }
}

impl ::core::default::Default for self::Blackboard {
    fn default() -> Self {
        Self::new()
    }
}

impl self::Blackboard {
    /// Create new blackboard mock channel.
    pub fn new() -> Self {
        let (tx, _) = ::tokio::sync::broadcast::channel(64);
        Self { tx: { tx } }
    }

    /// Subscribe to blackboard broadcast stream.
    pub fn subscribe(&self) -> ::tokio::sync::broadcast::Receiver<self::BlackboardEvent> {
        self.tx.subscribe()
    }

    /// Return mock obligation coverage percentage.
    pub async fn obligation_coverage(&self) -> ::core::primitive::f64 {
        100.0
    }

    /// Return mock taint status.
    pub async fn is_tainted(&self) -> ::core::primitive::bool {
        false
    }

    /// Return mock rule cards inventory.
    pub async fn rule_cards(&self) -> ::std::vec::Vec<::std::string::String> {
        ::std::vec::Vec::new()
    }
}

/// Mock step outcome returned by state machine runner.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub enum StepOutcome {
    /// State machine transitioned from one state to another.
    Advanced {
        /// Previous state.
        from: self::ModernizationState,
        /// Current advanced state.
        to: self::ModernizationState,
    },
    /// State machine requires human decision gate approval.
    GateRequired {
        /// Required gate.
        gate: self::HumanGate,
        /// Prompt context.
        prompt: ::std::string::String,
    },
    /// State machine workflow completed.
    Completed {
        /// Completed receipt stamps.
        receipts: ::std::vec::Vec<self::ReceiptStamp>,
    },
}

/// Run phase zero: S0 Intent Preflight. Awaits the wake transition into assessment.
#[derive(::core::fmt::Debug, ::core::default::Default)]
pub struct RunnerS0IntentPreflight;

/// Run phase one: S1 Assess. Blocks on the reimagine human gate in Hitl mode.
#[derive(::core::fmt::Debug, ::core::default::Default)]
pub struct RunnerS1Assess {
    /// Flag indicating automated execution across the reimagine gate.
    is_auto: ::core::primitive::bool,
}

/// Run phase two: S4 contracts approved. Emits the completion receipt.
#[derive(::core::fmt::Debug, ::core::default::Default)]
pub struct RunnerS4Approved;

/// Shared runtime context carried across mock runner states.
#[derive(::core::fmt::Debug)]
struct RunnerContext {
    /// Active modernization configuration.
    config: self::ModernizeConfig,
    /// Shared blackboard instance retained for future phase observers.
    #[allow(dead_code)]
    blackboard: self::Blackboard,
}

/// Mock state machine runner executing modernization campaign lifecycle.
/// State-as-struct Hoverbear pattern. Each run phase is one struct. Transitions
/// consume the machine and produce the next phase via `From` implementations.
/// Reference: https://hoverbear.org/blog/rust-state-machine-pattern/
#[derive(::core::fmt::Debug)]
pub struct StateMachineRunner<S> {
    /// Shared runtime context spanning all run phases.
    context: self::RunnerContext,
    /// Active run phase state payload.
    state: S,
}

impl self::StateMachineRunner<self::RunnerS0IntentPreflight> {
    /// Create new state machine runner mock parked in the intent preflight phase.
    pub fn new(config: self::ModernizeConfig, blackboard: self::Blackboard) -> Self {
        let context = self::RunnerContext {
            config: { config },
            blackboard: { blackboard },
        };
        Self {
            context: { context },
            state: self::RunnerS0IntentPreflight,
        }
    }

    /// Advance from S0 Intent Preflight into S1 Assess.
    pub fn step(self) -> (self::StateMachineRunner<self::RunnerS1Assess>, self::StepOutcome) {
        let from_state = self::ModernizationState::S0IntentPreflight;
        let to_state = self::ModernizationState::S1Assess;
        let outcome = self::StepOutcome::Advanced {
            from: { from_state },
            to: { to_state },
        };
        let machine_m1 = self::StateMachineRunner::<self::RunnerS1Assess>::from(self);
        (machine_m1, outcome)
    }
}

impl ::core::convert::From<self::StateMachineRunner<self::RunnerS0IntentPreflight>>
    for self::StateMachineRunner<self::RunnerS1Assess>
{
    fn from(machine: self::StateMachineRunner<self::RunnerS0IntentPreflight>) -> Self {
        let is_auto = machine.context.config.execution_mode == self::ExecutionMode::FullAuto;
        Self {
            context: { machine.context },
            state: self::RunnerS1Assess { is_auto },
        }
    }
}

impl self::StateMachineRunner<self::RunnerS1Assess> {
    /// Verify whether the campaign runs full-auto across the reimagine gate.
    pub fn is_auto(&self) -> ::core::primitive::bool {
        self.state.is_auto
    }

    /// Full-auto advance past assessment into the approved phase.
    /// Yields the approved machine plus its transition event.
    pub fn advance_auto(self) -> (self::StateMachineRunner<self::RunnerS4Approved>, self::StepOutcome) {
        let from_state = self::ModernizationState::S4BriefContracts;
        let to_state = self::ModernizationState::S5IterativeDag;
        let outcome = self::StepOutcome::Advanced {
            from: { from_state },
            to: { to_state },
        };
        let machine_approved = self::StateMachineRunner::<self::RunnerS4Approved>::from(self);
        (machine_approved, outcome)
    }

    /// Hitl park on the reimagine gate, yielding the gate request payload
    /// while retaining this machine for the gate decision.
    pub fn park_on_gate(&self) -> self::StepOutcome {
        let gate = self::HumanGate::GateOnReimagine;
        let prompt = "Approve contracts reimagination before iterative transformation.".to_string();
        let outcome = self::StepOutcome::GateRequired {
            gate: { gate },
            prompt: { prompt },
        };
        outcome
    }

    /// Record human gate decision, consuming the assessing machine.
    pub fn submit_gate_decision(
        self,
        decision: self::GateDecision,
    ) -> ::core::result::Result<self::StateMachineRunner<self::RunnerS4Approved>, ::std::string::String> {
        match decision {
            self::GateDecision::Approved { .. } => {
                let machine_approved = self::StateMachineRunner::<self::RunnerS4Approved>::from(self);
                ::core::result::Result::Ok(machine_approved)
            }
            self::GateDecision::Rejected { reason } => ::core::result::Result::Err(reason),
        }
    }
}

impl ::core::convert::From<self::StateMachineRunner<self::RunnerS1Assess>>
    for self::StateMachineRunner<self::RunnerS4Approved>
{
    fn from(machine: self::StateMachineRunner<self::RunnerS1Assess>) -> Self {
        Self {
            context: { machine.context },
            state: self::RunnerS4Approved,
        }
    }
}

impl self::StateMachineRunner<self::RunnerS4Approved> {
    /// Complete the campaign with the reimagine receipt.
    pub fn step(self) -> self::StepOutcome {
        let receipt = self::ReceiptStamp {
            gate: "GateOnReimagine".to_string(),
            hash: "mock-receipt-hash".to_string(),
        };
        let receipts_vec = ::std::vec![receipt];
        let outcome = self::StepOutcome::Completed {
            receipts: { receipts_vec },
        };
        outcome
    }
}
