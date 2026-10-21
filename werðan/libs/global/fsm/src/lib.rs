//! State-as-struct typestate machine for the macro-statechart FSM per METHODOLOGY.md Definition 3.

/// Seven XOR macro superstates of the Harel statechart root, per METHODOLOGY.md Definition 3.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub enum MacroState {
    /// Intent and preflight gate over the legacy repository.
    S0IntentPreflight,
    /// Assessment phase with directory shards.
    S1Assess,
    /// Dependency map phase with Tarjan condensation into the DAG.
    S2MapDag,
    /// Business rule extraction phase over DAG vertices.
    S3ExtractRules,
    /// Contract briefing phase over DAG edges.
    S4BriefContracts,
    /// Composite iterative DAG transform state with topological wave dispatch.
    S5IterativeDag,
    /// System verify and harden phase.
    S6SystemVerify,
    /// Terminal state reached after S6 guards pass.
    Terminated,
}

/// Errors raised during macro state transitions.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::cmp::PartialEq, ::core::cmp::Eq)]
pub enum MacroStateError {
    /// Already in the terminal state.
    AlreadyTerminated,
    /// Invalid regression path requested.
    InvalidRegression(::std::string::String),
}

impl ::core::fmt::Display for self::MacroStateError {
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::AlreadyTerminated => formatter.write_str("macro statechart is already terminated"),
            Self::InvalidRegression(detail) => {
                let formatted = ::core::format_args!("invalid macro regression: {detail}");
                formatter.write_fmt(formatted)
            }
        }
    }
}

impl ::std::error::Error for self::MacroStateError {}

/// Typestate marker for S0 Intent Preflight.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct S0IntentPreflight;

/// Typestate marker for S1 Assess.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct S1Assess;

/// Typestate marker for S2 Map DAG.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct S2MapDag;

/// Typestate marker for S3 Extract Rules.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct S3ExtractRules;

/// Typestate marker for S4 Brief Contracts.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct S4BriefContracts;

/// Typestate marker for S5 Iterative DAG.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct S5IterativeDag;

/// Typestate marker for S6 System Verify.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct S6SystemVerify;

/// Typestate marker for Terminated.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct Terminated;

/// Generic typestate machine container holding transition history and state marker.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct MacroStateMachine<StateMarker> {
    /// History of macro states traversed.
    pub history: ::std::vec::Vec<self::MacroState>,
    /// State marker wrapped in a single-element tuple.
    pub state: (StateMarker,),
}

impl self::MacroStateMachine<self::S0IntentPreflight> {
    /// Construct a new machine in S0IntentPreflight with empty history.
    pub fn new() -> Self {
        Self {
            history: ::std::vec::Vec::new(),
            state: (self::S0IntentPreflight,),
        }
    }
}

impl ::core::default::Default for self::MacroStateMachine<self::S0IntentPreflight> {
    fn default() -> Self {
        Self::new()
    }
}

impl ::core::convert::From<self::MacroStateMachine<self::S0IntentPreflight>> for self::MacroStateMachine<self::S1Assess> {
    fn from(machine: self::MacroStateMachine<self::S0IntentPreflight>) -> Self {
        let mut history = machine.history;
        history.push(self::MacroState::S0IntentPreflight);
        Self {
            history,
            state: (self::S1Assess,),
        }
    }
}

impl ::core::convert::From<self::MacroStateMachine<self::S1Assess>> for self::MacroStateMachine<self::S2MapDag> {
    fn from(machine: self::MacroStateMachine<self::S1Assess>) -> Self {
        let mut history = machine.history;
        history.push(self::MacroState::S1Assess);
        Self {
            history,
            state: (self::S2MapDag,),
        }
    }
}

impl ::core::convert::From<self::MacroStateMachine<self::S2MapDag>> for self::MacroStateMachine<self::S3ExtractRules> {
    fn from(machine: self::MacroStateMachine<self::S2MapDag>) -> Self {
        let mut history = machine.history;
        history.push(self::MacroState::S2MapDag);
        Self {
            history,
            state: (self::S3ExtractRules,),
        }
    }
}

impl ::core::convert::From<self::MacroStateMachine<self::S3ExtractRules>> for self::MacroStateMachine<self::S4BriefContracts> {
    fn from(machine: self::MacroStateMachine<self::S3ExtractRules>) -> Self {
        let mut history = machine.history;
        history.push(self::MacroState::S3ExtractRules);
        Self {
            history,
            state: (self::S4BriefContracts,),
        }
    }
}

impl ::core::convert::From<self::MacroStateMachine<self::S4BriefContracts>> for self::MacroStateMachine<self::S5IterativeDag> {
    fn from(machine: self::MacroStateMachine<self::S4BriefContracts>) -> Self {
        let mut history = machine.history;
        history.push(self::MacroState::S4BriefContracts);
        Self {
            history,
            state: (self::S5IterativeDag,),
        }
    }
}

impl ::core::convert::From<self::MacroStateMachine<self::S5IterativeDag>> for self::MacroStateMachine<self::S6SystemVerify> {
    fn from(machine: self::MacroStateMachine<self::S5IterativeDag>) -> Self {
        let mut history = machine.history;
        history.push(self::MacroState::S5IterativeDag);
        Self {
            history,
            state: (self::S6SystemVerify,),
        }
    }
}

impl ::core::convert::From<self::MacroStateMachine<self::S6SystemVerify>> for self::MacroStateMachine<self::Terminated> {
    fn from(machine: self::MacroStateMachine<self::S6SystemVerify>) -> Self {
        let mut history = machine.history;
        history.push(self::MacroState::S6SystemVerify);
        Self {
            history,
            state: (self::Terminated,),
        }
    }
}

impl ::core::convert::From<self::MacroStateMachine<self::S5IterativeDag>> for self::MacroStateMachine<self::S2MapDag> {
    fn from(machine: self::MacroStateMachine<self::S5IterativeDag>) -> Self {
        let mut history = machine.history;
        history.push(self::MacroState::S5IterativeDag);
        Self {
            history,
            state: (self::S2MapDag,),
        }
    }
}

/// Dynamic wrapper holding one typestate variant of the macro state machine.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub enum MacroStatechart {
    /// S0 Intent Preflight.
    S0(self::MacroStateMachine<self::S0IntentPreflight>),
    /// S1 Assess.
    S1(self::MacroStateMachine<self::S1Assess>),
    /// S2 Map DAG.
    S2(self::MacroStateMachine<self::S2MapDag>),
    /// S3 Extract Rules.
    S3(self::MacroStateMachine<self::S3ExtractRules>),
    /// S4 Brief Contracts.
    S4(self::MacroStateMachine<self::S4BriefContracts>),
    /// S5 Iterative DAG.
    S5(self::MacroStateMachine<self::S5IterativeDag>),
    /// S6 System Verify.
    S6(self::MacroStateMachine<self::S6SystemVerify>),
    /// Terminated terminal state.
    Terminated(self::MacroStateMachine<self::Terminated>),
}

#[::bon::bon]
impl self::MacroStatechart {
    /// Construct a new macro statechart using a bon builder.
    #[builder]
    pub fn builder(
        initial_state: ::core::option::Option<self::MacroState>,
        history: ::core::option::Option<::std::vec::Vec<self::MacroState>>,
    ) -> Self {
        let history = history.unwrap_or_default();
        let state = initial_state.unwrap_or(self::MacroState::S0IntentPreflight);
        match state {
            self::MacroState::S0IntentPreflight => {
                let machine = self::MacroStateMachine { history, state: (self::S0IntentPreflight,) };
                Self::S0(machine)
            }
            self::MacroState::S1Assess => {
                let machine = self::MacroStateMachine { history, state: (self::S1Assess,) };
                Self::S1(machine)
            }
            self::MacroState::S2MapDag => {
                let machine = self::MacroStateMachine { history, state: (self::S2MapDag,) };
                Self::S2(machine)
            }
            self::MacroState::S3ExtractRules => {
                let machine = self::MacroStateMachine { history, state: (self::S3ExtractRules,) };
                Self::S3(machine)
            }
            self::MacroState::S4BriefContracts => {
                let machine = self::MacroStateMachine { history, state: (self::S4BriefContracts,) };
                Self::S4(machine)
            }
            self::MacroState::S5IterativeDag => {
                let machine = self::MacroStateMachine { history, state: (self::S5IterativeDag,) };
                Self::S5(machine)
            }
            self::MacroState::S6SystemVerify => {
                let machine = self::MacroStateMachine { history, state: (self::S6SystemVerify,) };
                Self::S6(machine)
            }
            self::MacroState::Terminated => {
                let machine = self::MacroStateMachine { history, state: (self::Terminated,) };
                Self::Terminated(machine)
            }
        }
    }

    /// Construct a new statechart in S0 Intent Preflight.
    pub fn new() -> Self {
        let machine = self::MacroStateMachine::<self::S0IntentPreflight>::new();
        Self::S0(machine)
    }

    /// Advance the statechart along the canonical macro sequence.
    pub fn advance(self) -> ::core::result::Result<Self, self::MacroStateError> {
        match self {
            Self::S0(machine) => {
                let transitioned = self::MacroStateMachine::<self::S1Assess>::from(machine);
                ::core::result::Result::Ok(Self::S1(transitioned))
            }
            Self::S1(machine) => {
                let transitioned = self::MacroStateMachine::<self::S2MapDag>::from(machine);
                ::core::result::Result::Ok(Self::S2(transitioned))
            }
            Self::S2(machine) => {
                let transitioned = self::MacroStateMachine::<self::S3ExtractRules>::from(machine);
                ::core::result::Result::Ok(Self::S3(transitioned))
            }
            Self::S3(machine) => {
                let transitioned = self::MacroStateMachine::<self::S4BriefContracts>::from(machine);
                ::core::result::Result::Ok(Self::S4(transitioned))
            }
            Self::S4(machine) => {
                let transitioned = self::MacroStateMachine::<self::S5IterativeDag>::from(machine);
                ::core::result::Result::Ok(Self::S5(transitioned))
            }
            Self::S5(machine) => {
                let transitioned = self::MacroStateMachine::<self::S6SystemVerify>::from(machine);
                ::core::result::Result::Ok(Self::S6(transitioned))
            }
            Self::S6(machine) => {
                let transitioned = self::MacroStateMachine::<self::Terminated>::from(machine);
                ::core::result::Result::Ok(Self::Terminated(transitioned))
            }
            Self::Terminated(_) => ::core::result::Result::Err(self::MacroStateError::AlreadyTerminated),
        }
    }

    /// Regress from the active state to an upstream target state.
    pub fn regress_to(self, target: self::MacroState) -> ::core::result::Result<Self, self::MacroStateError> {
        let is_target_s2 = target == self::MacroState::S2MapDag;
        if is_target_s2 {
            return self.regress_to_s2();
        }
        let current = self.state();
        let mut history = self.history().to_vec();
        history.push(current);
        match target {
            self::MacroState::S0IntentPreflight => {
                let machine = self::MacroStateMachine { history, state: (self::S0IntentPreflight,) };
                ::core::result::Result::Ok(Self::S0(machine))
            }
            self::MacroState::S1Assess => {
                let machine = self::MacroStateMachine { history, state: (self::S1Assess,) };
                ::core::result::Result::Ok(Self::S1(machine))
            }
            self::MacroState::S2MapDag => {
                let machine = self::MacroStateMachine { history, state: (self::S2MapDag,) };
                ::core::result::Result::Ok(Self::S2(machine))
            }
            self::MacroState::S3ExtractRules => {
                let machine = self::MacroStateMachine { history, state: (self::S3ExtractRules,) };
                ::core::result::Result::Ok(Self::S3(machine))
            }
            self::MacroState::S4BriefContracts => {
                let machine = self::MacroStateMachine { history, state: (self::S4BriefContracts,) };
                ::core::result::Result::Ok(Self::S4(machine))
            }
            self::MacroState::S5IterativeDag => {
                let machine = self::MacroStateMachine { history, state: (self::S5IterativeDag,) };
                ::core::result::Result::Ok(Self::S5(machine))
            }
            self::MacroState::S6SystemVerify => {
                let machine = self::MacroStateMachine { history, state: (self::S6SystemVerify,) };
                ::core::result::Result::Ok(Self::S6(machine))
            }
            self::MacroState::Terminated => {
                let detail = "cannot regress to Terminated".to_string();
                ::core::result::Result::Err(self::MacroStateError::InvalidRegression(detail))
            }
        }
    }

    /// Regress from S5 back to S2 under Tier C cyclic dependency discovery.
    pub fn regress_to_s2(self) -> ::core::result::Result<Self, self::MacroStateError> {
        match self {
            Self::S5(machine) => {
                let transitioned = self::MacroStateMachine::<self::S2MapDag>::from(machine);
                ::core::result::Result::Ok(Self::S2(transitioned))
            }
            Self::Terminated(_) => {
                let detail = "cannot regress from Terminated".to_string();
                ::core::result::Result::Err(self::MacroStateError::InvalidRegression(detail))
            }
            _ => {
                let current = self.state();
                let mut history = self.history().to_vec();
                history.push(current);
                let machine = self::MacroStateMachine {
                    history,
                    state: (self::S2MapDag,),
                };
                ::core::result::Result::Ok(Self::S2(machine))
            }
        }
    }

    /// Return the active macro state variant.
    pub fn state(&self) -> self::MacroState {
        match self {
            Self::S0(_) => self::MacroState::S0IntentPreflight,
            Self::S1(_) => self::MacroState::S1Assess,
            Self::S2(_) => self::MacroState::S2MapDag,
            Self::S3(_) => self::MacroState::S3ExtractRules,
            Self::S4(_) => self::MacroState::S4BriefContracts,
            Self::S5(_) => self::MacroState::S5IterativeDag,
            Self::S6(_) => self::MacroState::S6SystemVerify,
            Self::Terminated(_) => self::MacroState::Terminated,
        }
    }

    /// Return the history of traversed macro states.
    pub fn history(&self) -> &[self::MacroState] {
        match self {
            Self::S0(machine) => &machine.history,
            Self::S1(machine) => &machine.history,
            Self::S2(machine) => &machine.history,
            Self::S3(machine) => &machine.history,
            Self::S4(machine) => &machine.history,
            Self::S5(machine) => &machine.history,
            Self::S6(machine) => &machine.history,
            Self::Terminated(machine) => &machine.history,
        }
    }
}

impl ::core::default::Default for self::MacroStatechart {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_statechart_advancement() {
        let statechart = self::MacroStatechart::new();
        ::std::assert_eq!(statechart.state(), self::MacroState::S0IntentPreflight);
        ::std::assert!(statechart.history().is_empty());

        let step_s1 = statechart.advance().unwrap();
        ::std::assert_eq!(step_s1.state(), self::MacroState::S1Assess);
        let step_s2 = step_s1.advance().unwrap();
        ::std::assert_eq!(step_s2.state(), self::MacroState::S2MapDag);
        let step_s3 = step_s2.advance().unwrap();
        ::std::assert_eq!(step_s3.state(), self::MacroState::S3ExtractRules);
        let step_s4 = step_s3.advance().unwrap();
        ::std::assert_eq!(step_s4.state(), self::MacroState::S4BriefContracts);
        let step_s5 = step_s4.advance().unwrap();
        ::std::assert_eq!(step_s5.state(), self::MacroState::S5IterativeDag);
        let step_s6 = step_s5.advance().unwrap();
        ::std::assert_eq!(step_s6.state(), self::MacroState::S6SystemVerify);
        let terminated = step_s6.advance().unwrap();
        ::std::assert_eq!(terminated.state(), self::MacroState::Terminated);
    }

    #[test]
    fn test_macro_statechart_builder() {
        let initial_history = ::std::vec![self::MacroState::S0IntentPreflight, self::MacroState::S1Assess];
        let statechart = self::MacroStatechart::builder()
            .initial_state(self::MacroState::S2MapDag)
            .history(initial_history)
            .build();
        ::std::assert_eq!(statechart.state(), self::MacroState::S2MapDag);
        ::std::assert_eq!(statechart.history().len(), 2);
    }

    #[test]
    fn test_macro_statechart_regressions() {
        let statechart = self::MacroStatechart::builder()
            .initial_state(self::MacroState::S5IterativeDag)
            .build();
        let regressed = statechart.regress_to_s2().unwrap();
        ::std::assert_eq!(regressed.state(), self::MacroState::S2MapDag);
        ::std::assert_eq!(regressed.history(), &[self::MacroState::S5IterativeDag]);
    }
}
