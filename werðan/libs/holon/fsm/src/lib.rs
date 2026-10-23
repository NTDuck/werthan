//! Finite state machine for ARIADNE module clusters.
//! Controls the six micro-states from the methodology specification.
//! Pattern reference: https://hoverbear.org/blog/rust-state-machine-pattern/


/// Serde-compatible tag enumeration for runtime inspection and reflection of micro-states.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub enum MicroStateTag {
    /// Initial synthesis state blocked on upstream contract availability.
    M0Blocked,
    /// Upstream contracts available. Hydrating local problem context.
    M1ContextHydration,
    /// Active intra-module Monte Carlo Tree Search synthesis.
    M2AriadneMcts,
    /// Upstream interface defect identified. Halts search for contract revision.
    M3ContractNegotiate,
    /// Candidate target code verified against unit tests and business rules.
    M4Hardened,
    /// Downstream invalidation detected. Preserves search tree for warm start.
    M5Invalidated,
}

/// Activation mode for MCTS execution.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub enum ActivationMode {
    /// Initial synthesis run from scratch.
    Initial,
    /// Re-dispatch after a failed guard, carrying the validator traceback.
    CriticRetry,
    /// Warm start regression maintaining prior search tree and tests.
    WarmStartRegression,
}

/// Broadcast event emitted to the global statechart queue.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub enum ClusterBroadcastEvent {
    /// Module verified and hardened with target code and test suite.
    ModuleHardened {
        /// Target module identifier.
        module_id: ::std::string::String,
        /// Verified code.
        target_code: ::std::string::String,
        /// Characterization tests.
        test_suite: ::std::vec::Vec<::std::string::String>,
    },
    /// Extrinsic contract conflict on parent module.
    ContractConflict {
        /// Upstream parent identifier.
        parent_id: ::std::string::String,
        /// Target module identifier.
        module_id: ::std::string::String,
        /// Downstream reporting module.
        delta_spec: ::std::string::String,
        /// Failure proof trace.
        proof_trace: ::std::string::String,
    },
    /// Undocumented cyclic dependency requiring macro regression.
    CyclicDependencyFound {
        /// Cyclic edge pairs.
        cycle_edges: ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
        /// Failure proof trace.
        proof_trace: ::std::string::String,
    },
}

/// Transition history metrics and audit trail.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct FsmHistory {
    /// Sequence of visited micro-state reflection tags.
    pub visited_states: ::std::vec::Vec<self::MicroStateTag>,
    /// Number of completed warm start regressions.
    pub warm_start_count: ::core::primitive::usize,
    /// Number of detected contract conflicts.
    pub conflict_count: ::core::primitive::usize,
    /// Last committed target code if hardened.
    pub last_hardened_code: ::core::option::Option<::std::string::String>,
}

/// M0 Blocked typestate waiting for upstream parent contracts.
/// Reference: https://hoverbear.org/blog/rust-state-machine-pattern/
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct M0Blocked;

/// M1 Context Hydration typestate loading problem models and contracts.
/// Reference: https://hoverbear.org/blog/rust-state-machine-pattern/
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct M1ContextHydration;

/// M2 Ariadne MCTS typestate actively executing tree search synthesis.
/// Reference: https://hoverbear.org/blog/rust-state-machine-pattern/
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct M2AriadneMcts;

/// M3 Contract Negotiate typestate escalating contract conflicts.
/// Reference: https://hoverbear.org/blog/rust-state-machine-pattern/
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct M3ContractNegotiate {
    /// Upstream parent identifier.
    pub parent_id: ::std::string::String,
    /// Interface delta specification required.
    pub delta_spec: ::std::string::String,
    /// Failure proof trace.
    pub proof_trace: ::std::string::String,
}

/// M4 Hardened typestate holding verified synthesized code.
/// Reference: https://hoverbear.org/blog/rust-state-machine-pattern/
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct M4Hardened {
    /// Verified target code snippet.
    pub target_code: ::std::string::String,
    /// Characterization test suite items.
    pub test_suite: ::std::vec::Vec<::std::string::String>,
    /// Final reward score.
    pub reward: ::core::primitive::f64,
}

/// M5 Invalidated typestate awaiting warm-start regression restart.
/// Reference: https://hoverbear.org/blog/rust-state-machine-pattern/
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct M5Invalidated {
    /// Invalidation delta specification.
    pub delta_spec: ::std::string::String,
}

/// Generic typestate machine container with shared context.
/// Reference: https://hoverbear.org/blog/rust-state-machine-pattern/
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct StateMachine<S> {
    /// Unique module identifier.
    pub module_id: ::std::string::String,
    /// Active search activation mode.
    pub activation_mode: self::ActivationMode,
    /// Transition audit trail.
    pub history: self::FsmHistory,
    /// Outbox broadcast events.
    pub outbox: ::std::vec::Vec<self::ClusterBroadcastEvent>,
    /// State-specific payload.
    pub state: S,
}

impl<S> self::StateMachine<S> {
    /// Return reference to module identifier.
    pub fn module_id(&self) -> &::core::primitive::str {
        &self.module_id
    }

    /// Return reference to audit history.
    pub fn history(&self) -> &self::FsmHistory {
        &self.history
    }

    /// Return active search activation mode.
    pub fn activation_mode(&self) -> self::ActivationMode {
        self.activation_mode
    }

    /// Drain pending broadcast events for the global queue.
    pub fn drain_events(&mut self) -> ::std::vec::Vec<self::ClusterBroadcastEvent> {
        ::core::mem::take(&mut self.outbox)
    }
}

impl self::StateMachine<self::M0Blocked> {
    /// Construct new state machine parked in M0 Blocked.
    pub fn new(module_id: impl ::core::convert::Into<::std::string::String>) -> Self {
        let id_string = module_id.into();
        let initial_tag = self::MicroStateTag::M0Blocked;
        let mut history = self::FsmHistory::default();
        history.visited_states.push(initial_tag);

        Self {
            module_id: { id_string },
            activation_mode: self::ActivationMode::Initial,
            history: { history },
            outbox: ::std::vec::Vec::new(),
            state: self::M0Blocked,
        }
    }

    /// Advance from M0 Blocked to M1 Context Hydration.
    pub fn wake_ready(self) -> self::StateMachine<self::M1ContextHydration> {
        self::StateMachine::<self::M1ContextHydration>::from(self)
    }

    /// Return reflection tag.
    pub fn state_tag(&self) -> self::MicroStateTag {
        self::MicroStateTag::M0Blocked
    }

    /// Status predicate.
    pub fn is_blocked(&self) -> ::core::primitive::bool {
        true
    }

    /// Status predicate.
    pub fn is_hardened(&self) -> ::core::primitive::bool {
        false
    }

    /// Status predicate.
    pub fn is_active(&self) -> ::core::primitive::bool {
        false
    }
}

impl self::StateMachine<self::M1ContextHydration> {
    /// Advance from M1 Context Hydration to M2 Ariadne MCTS.
    pub fn hydration_completed(self) -> self::StateMachine<self::M2AriadneMcts> {
        self::StateMachine::<self::M2AriadneMcts>::from(self)
    }

    /// Return reflection tag.
    pub fn state_tag(&self) -> self::MicroStateTag {
        self::MicroStateTag::M1ContextHydration
    }

    /// Status predicate.
    pub fn is_blocked(&self) -> ::core::primitive::bool {
        false
    }

    /// Status predicate.
    pub fn is_hardened(&self) -> ::core::primitive::bool {
        false
    }

    /// Status predicate.
    pub fn is_active(&self) -> ::core::primitive::bool {
        false
    }
}

impl self::StateMachine<self::M2AriadneMcts> {
    /// Transition from M2 Ariadne MCTS to M4 Hardened.
    pub fn into_hardened(
        mut self,
        target_code: ::std::string::String,
        test_suite: ::std::vec::Vec<::std::string::String>,
        reward: ::core::primitive::f64,
    ) -> self::StateMachine<self::M4Hardened> {
        self.history.visited_states.push(self::MicroStateTag::M4Hardened);
        self.history.last_hardened_code = ::core::option::Option::Some(target_code.clone());
        let broadcast_event = self::ClusterBroadcastEvent::ModuleHardened {
            module_id: { self.module_id.clone() },
            target_code: { target_code.clone() },
            test_suite: { test_suite.clone() },
        };
        self.outbox.push(broadcast_event);
        let hardened_state = self::M4Hardened {
            target_code,
            test_suite,
            reward,
        };
        self::StateMachine {
            module_id: { self.module_id },
            activation_mode: { self.activation_mode },
            history: { self.history },
            outbox: { self.outbox },
            state: { hardened_state },
        }
    }

    /// Transition from M2 Ariadne MCTS to M3 Contract Negotiate.
    pub fn into_contract_negotiate(
        mut self,
        parent_id: ::std::string::String,
        delta_spec: ::std::string::String,
        proof_trace: ::std::string::String,
    ) -> self::StateMachine<self::M3ContractNegotiate> {
        self.history.visited_states.push(self::MicroStateTag::M3ContractNegotiate);
        self.history.conflict_count += 1;
        let broadcast_event = self::ClusterBroadcastEvent::ContractConflict {
            parent_id: { parent_id.clone() },
            module_id: { self.module_id.clone() },
            delta_spec: { delta_spec.clone() },
            proof_trace: { proof_trace.clone() },
        };
        self.outbox.push(broadcast_event);
        let negotiate_state = self::M3ContractNegotiate {
            parent_id,
            delta_spec,
            proof_trace,
        };
        self::StateMachine {
            module_id: { self.module_id },
            activation_mode: { self.activation_mode },
            history: { self.history },
            outbox: { self.outbox },
            state: { negotiate_state },
        }
    }

    /// Emit cycle regression broadcast without state change.
    pub fn emit_cycle_regression(
        &mut self,
        cycle_edges: ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
        proof_trace: ::std::string::String,
    ) {
        let broadcast_event = self::ClusterBroadcastEvent::CyclicDependencyFound {
            cycle_edges,
            proof_trace,
        };
        self.outbox.push(broadcast_event);
    }

    /// Return reflection tag.
    pub fn state_tag(&self) -> self::MicroStateTag {
        self::MicroStateTag::M2AriadneMcts
    }

    /// Status predicate.
    pub fn is_blocked(&self) -> ::core::primitive::bool {
        false
    }

    /// Status predicate.
    pub fn is_hardened(&self) -> ::core::primitive::bool {
        false
    }

    /// Status predicate.
    pub fn is_active(&self) -> ::core::primitive::bool {
        true
    }
}

impl self::StateMachine<self::M3ContractNegotiate> {
    /// Return to M0 Blocked after conflict dispatch.
    pub fn conflict_dispatched(self) -> self::StateMachine<self::M0Blocked> {
        self::StateMachine::<self::M0Blocked>::from(self)
    }

    /// Return reflection tag.
    pub fn state_tag(&self) -> self::MicroStateTag {
        self::MicroStateTag::M3ContractNegotiate
    }

    /// Status predicate.
    pub fn is_blocked(&self) -> ::core::primitive::bool {
        false
    }

    /// Status predicate.
    pub fn is_hardened(&self) -> ::core::primitive::bool {
        false
    }

    /// Status predicate.
    pub fn is_active(&self) -> ::core::primitive::bool {
        false
    }
}

impl self::StateMachine<self::M4Hardened> {
    /// Transition from M4 Hardened to M5 Invalidated.
    pub fn into_invalidated(
        mut self,
        delta_spec: ::std::string::String,
    ) -> self::StateMachine<self::M5Invalidated> {
        self.history.visited_states.push(self::MicroStateTag::M5Invalidated);
        self.activation_mode = self::ActivationMode::WarmStartRegression;
        self.history.warm_start_count += 1;
        let invalidated_state = self::M5Invalidated {
            delta_spec,
        };
        self::StateMachine {
            module_id: { self.module_id },
            activation_mode: { self.activation_mode },
            history: { self.history },
            outbox: { self.outbox },
            state: { invalidated_state },
        }
    }

    /// Return reflection tag.
    pub fn state_tag(&self) -> self::MicroStateTag {
        self::MicroStateTag::M4Hardened
    }

    /// Status predicate.
    pub fn is_blocked(&self) -> ::core::primitive::bool {
        false
    }

    /// Status predicate.
    pub fn is_hardened(&self) -> ::core::primitive::bool {
        true
    }

    /// Status predicate.
    pub fn is_active(&self) -> ::core::primitive::bool {
        false
    }
}

impl self::StateMachine<self::M5Invalidated> {
    /// Re-enter M2 Ariadne MCTS in warm-start regression mode.
    pub fn warm_start_ready(self) -> self::StateMachine<self::M2AriadneMcts> {
        self::StateMachine::<self::M2AriadneMcts>::from(self)
    }

    /// Return reflection tag.
    pub fn state_tag(&self) -> self::MicroStateTag {
        self::MicroStateTag::M5Invalidated
    }

    /// Status predicate.
    pub fn is_blocked(&self) -> ::core::primitive::bool {
        false
    }

    /// Status predicate.
    pub fn is_hardened(&self) -> ::core::primitive::bool {
        false
    }

    /// Status predicate.
    pub fn is_active(&self) -> ::core::primitive::bool {
        false
    }
}

impl ::core::convert::From<self::StateMachine<self::M0Blocked>> for self::StateMachine<self::M1ContextHydration> {
    fn from(machine: self::StateMachine<self::M0Blocked>) -> Self {
        let mut history = machine.history;
        history.visited_states.push(self::MicroStateTag::M1ContextHydration);
        Self {
            module_id: { machine.module_id },
            activation_mode: { machine.activation_mode },
            history: { history },
            outbox: { machine.outbox },
            state: self::M1ContextHydration,
        }
    }
}

impl ::core::convert::From<self::StateMachine<self::M1ContextHydration>> for self::StateMachine<self::M2AriadneMcts> {
    fn from(machine: self::StateMachine<self::M1ContextHydration>) -> Self {
        let mut history = machine.history;
        history.visited_states.push(self::MicroStateTag::M2AriadneMcts);
        Self {
            module_id: { machine.module_id },
            activation_mode: { machine.activation_mode },
            history: { history },
            outbox: { machine.outbox },
            state: self::M2AriadneMcts,
        }
    }
}

impl ::core::convert::From<self::StateMachine<self::M3ContractNegotiate>> for self::StateMachine<self::M0Blocked> {
    fn from(machine: self::StateMachine<self::M3ContractNegotiate>) -> Self {
        let mut history = machine.history;
        history.visited_states.push(self::MicroStateTag::M0Blocked);
        Self {
            module_id: { machine.module_id },
            activation_mode: { machine.activation_mode },
            history: { history },
            outbox: { machine.outbox },
            state: self::M0Blocked,
        }
    }
}

impl ::core::convert::From<self::StateMachine<self::M5Invalidated>> for self::StateMachine<self::M2AriadneMcts> {
    fn from(machine: self::StateMachine<self::M5Invalidated>) -> Self {
        let mut history = machine.history;
        history.visited_states.push(self::MicroStateTag::M2AriadneMcts);
        Self {
            module_id: { machine.module_id },
            activation_mode: self::ActivationMode::WarmStartRegression,
            history: { history },
            outbox: { machine.outbox },
            state: self::M2AriadneMcts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typestate_normal_lifecycle() {
        let machine_m0 = self::StateMachine::<self::M0Blocked>::new("module_alpha");
        ::std::assert!(machine_m0.is_blocked());

        let machine_m1 = machine_m0.wake_ready();
        ::std::assert_eq!(machine_m1.state_tag(), self::MicroStateTag::M1ContextHydration);

        let machine_m2 = machine_m1.hydration_completed();
        ::std::assert!(machine_m2.is_active());

        let target_code = "pub fn execute() -> i32 { 42 }".to_string();
        let test_suite = ::std::vec!["assert_eq!(execute(), 42)".to_string()];
        let mut machine_m4 = machine_m2.into_hardened(target_code, test_suite, 1.0);
        ::std::assert!(machine_m4.is_hardened());

        let events = machine_m4.drain_events();
        ::std::assert_eq!(events.len(), 1);
        let first_event = &events[0];
        let is_hardened_broadcast =
            ::core::matches!(first_event, self::ClusterBroadcastEvent::ModuleHardened { .. });
        ::std::assert!(is_hardened_broadcast);
    }

    #[test]
    fn test_typestate_warm_start_regression() {
        let machine_m0 = self::StateMachine::<self::M0Blocked>::new("module_beta");
        let hydrated = machine_m0.wake_ready().hydration_completed();
        let empty_suite = ::std::vec::Vec::<::std::string::String>::new();
        let hardened = hydrated.into_hardened("fn test() {}".to_string(), empty_suite, 1.0);
        let machine_m5 = hardened.into_invalidated("requires extra parameter".to_string());
        ::std::assert_eq!(machine_m5.state_tag(), self::MicroStateTag::M5Invalidated);
        let mode_m5 = machine_m5.activation_mode();
        ::std::assert_eq!(mode_m5, self::ActivationMode::WarmStartRegression);

        let machine_m2_warm = machine_m5.warm_start_ready();
        ::std::assert_eq!(machine_m2_warm.state_tag(), self::MicroStateTag::M2AriadneMcts);
        let mode_warm = machine_m2_warm.activation_mode();
        ::std::assert_eq!(mode_warm, self::ActivationMode::WarmStartRegression);
        let warm_count = machine_m2_warm.history().warm_start_count;
        ::std::assert_eq!(warm_count, 1);
    }

    #[test]
    fn test_typestate_extrinsic_contract_conflict() {
        let machine_m0 = self::StateMachine::<self::M0Blocked>::new("module_gamma");
        let machine_m2 = machine_m0.wake_ready().hydration_completed();
        let mut machine_m3 = machine_m2.into_contract_negotiate(
            "parent_alpha".to_string(),
            "new signature".to_string(),
            "diagnostic trace".to_string(),
        );
        ::std::assert_eq!(machine_m3.state_tag(), self::MicroStateTag::M3ContractNegotiate);

        let events = machine_m3.drain_events();
        ::std::assert_eq!(events.len(), 1);
        let is_conflict_broadcast =
            ::core::matches!(events[0], self::ClusterBroadcastEvent::ContractConflict { .. });
        ::std::assert!(is_conflict_broadcast);

        let machine_m0_back = machine_m3.conflict_dispatched();
        ::std::assert_eq!(machine_m0_back.state_tag(), self::MicroStateTag::M0Blocked);
    }

    #[test]
    fn test_typestate_cycle_regression_broadcast() {
        let machine_m0 = self::StateMachine::<self::M0Blocked>::new("module_delta");
        let mut machine_m2 = machine_m0.wake_ready().hydration_completed();
        machine_m2.emit_cycle_regression(
            ::std::vec![("parent_a".to_string(), "module_delta".to_string())],
            "cyclic dependency graph loop".to_string(),
        );
        ::std::assert_eq!(machine_m2.state_tag(), self::MicroStateTag::M2AriadneMcts);
        let events = machine_m2.drain_events();
        ::std::assert_eq!(events.len(), 1);
        let is_cycle_broadcast =
            ::core::matches!(events[0], self::ClusterBroadcastEvent::CyclicDependencyFound { .. });
        ::std::assert!(is_cycle_broadcast);
    }
}
