//! Macro engine error types.

/// Error raised when an invariant of Algorithm 1 breaks at runtime.
#[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
pub enum MacroEngineError {
    /// A structural guard still fails after one retry dispatch.
    StructuralGuardFailure {
        /// Phase whose guard failed.
        phase: ::global_fsm::MacroState,
        /// Guard failure detail.
        detail: ::std::string::String,
    },
    /// Step budget exhausted before reaching Terminated.
    StepBudgetExhausted {
        /// Consumed step count.
        steps_used: ::core::primitive::usize,
    },
    /// A RemediateLocal critique persisted after its remediation re-dispatch.
    SemanticRemediationFailed {
        /// Phase whose critic rejected the remediated batch.
        phase: ::global_fsm::MacroState,
        /// Critic remediation notes describing the unresolved rejection.
        detail: ::std::string::String,
    },
    /// The S0 preflight intent gate rejected the campaign intent (METHODOLOGY.md
    /// S0 structural guard: harness configuration and INTENT validity).
    PreflightGateFailed {
        /// Gate detail describing the invalid intent element.
        detail: ::std::string::String,
    },
    /// Wave dispatch found no ready vertex while modules remain unhardened.
    WaveDeadlock {
        /// Detail about the stuck module set.
        detail: ::std::string::String,
    },
    /// One module re-dispatched without observable progress up to the futile
    /// dispatch cap. Each futile dispatch burns a full solver rollout budget,
    /// so the engine terminates instead of cycling park, re-ready, re-dispatch.
    ModuleRetryExhausted {
        /// Module whose dispatches proved futile.
        module_id: ::std::string::String,
        /// Futile dispatch count at the cap.
        attempts: ::core::primitive::usize,
    },
    /// Macro state transition error.
    MacroStateError(::global_fsm::MacroStateError),
}

impl ::core::convert::From<::global_fsm::MacroStateError> for self::MacroEngineError {
    fn from(error: ::global_fsm::MacroStateError) -> Self {
        Self::MacroStateError(error)
    }
}

impl ::core::fmt::Display for self::MacroEngineError {
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::StructuralGuardFailure { phase, detail } => {
                ::core::write!(formatter, "structural guard failure in phase {phase:?}: {detail}")
            }
            Self::StepBudgetExhausted { steps_used } => {
                ::core::write!(formatter, "step budget exhausted after {steps_used} steps")
            }
            Self::SemanticRemediationFailed { phase, detail } => {
                ::core::write!(formatter, "semantic remediation failed in phase {phase:?}: {detail}")
            }
            Self::PreflightGateFailed { detail } => {
                ::core::write!(formatter, "preflight intent gate failed: {detail}")
            }
            Self::WaveDeadlock { detail } => {
                ::core::write!(formatter, "wave dispatch deadlock: {detail}")
            }
            Self::ModuleRetryExhausted { module_id, attempts } => {
                ::core::write!(
                    formatter,
                    "module {module_id} exhausted the futile dispatch cap after {attempts} attempts"
                )
            }
            Self::MacroStateError(error) => {
                let formatted = ::core::format_args!("macro statechart transition error: {error}");
                formatter.write_fmt(formatted)
            }
        }
    }
}

impl ::std::error::Error for self::MacroEngineError {}
