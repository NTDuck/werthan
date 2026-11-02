//! Common types and configuration for MCTS cluster controller.
//! Follows the ARIADNE intra-module solver specification.

/// Reward weight coefficients summing to 1.0.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct RewardWeights {
    /// Weight for compilation status.
    pub compile: ::core::primitive::f64,
    /// Weight for characterization test pass rate.
    pub parity: ::core::primitive::f64,
    /// Weight for rule parity and hardening score.
    pub harden: ::core::primitive::f64,
}

impl ::core::default::Default for self::RewardWeights {
    fn default() -> Self {
        Self {
            compile: 0.20,
            parity: 0.60,
            harden: 0.20,
        }
    }
}

/// Configuration parameters for MCTS search controller.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct EngineConfig {
    /// Maximum rollout budget per search invocation.
    pub max_rollouts: ::core::primitive::usize,
    /// Upper Confidence Bound exploration constant.
    pub ucb_c_exp: ::core::primitive::f64,
    /// Strategy prior weight constant.
    pub gamma_prior: ::core::primitive::f64,
    /// Temperature for softmax child selection.
    pub softmax_temperature: ::core::primitive::f64,
    /// Sampling temperature for Rig-backed generative agent calls.
    pub generative_temperature: ::core::primitive::f64,
    /// Gated reward weight coefficients.
    pub reward_weights: self::RewardWeights,
}

impl ::core::default::Default for self::EngineConfig {
    fn default() -> Self {
        let weights = self::RewardWeights::default();
        Self {
            max_rollouts: 16,
            ucb_c_exp: 1.414,
            gamma_prior: 0.15,
            softmax_temperature: 0.35,
            generative_temperature: 0.20,
            reward_weights: { weights },
        }
    }
}

/// Strongly-typed outcome of compile-time enforced typestate MCTS search.
/// Consumes `StateMachine<M2AriadneMcts>` and transitions into the target typestate.
#[derive(::core::fmt::Debug)]
pub enum EngineTypestateOutcome {
    /// Synthesis succeeded, advancing machine into M4 Hardened.
    Hardened(::holon_fsm::StateMachine<::holon_fsm::M4Hardened>),
    /// Extrinsic contract defect identified, advancing machine into M3 ContractNegotiate.
    ContractConflict(::holon_fsm::StateMachine<::holon_fsm::M3ContractNegotiate>),
    /// Cycle detected, emitting regression broadcast while retaining M2 AriadneMcts state.
    CycleFound {
        /// Retained state machine in M2 AriadneMcts.
        machine: ::holon_fsm::StateMachine<::holon_fsm::M2AriadneMcts>,
        /// Detected cycle edge pairs.
        cycle_edges: ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
        /// Failure proof trace.
        proof_trace: ::std::string::String,
    },
    /// Rollout budget exhausted without hardening. The machine stays in M2
    /// AriadneMcts: no Tier B broadcast fires here. When the module blocks on
    /// a real parent edge, the solver bridge converts this into a Tier B
    /// contract conflict. A parentless module parks plainly (Tier A exhaustion).
    BudgetExhausted {
        /// Retained state machine in M2 AriadneMcts.
        machine: ::holon_fsm::StateMachine<::holon_fsm::M2AriadneMcts>,
        /// Parent identifier the module blocks on. Empty when the module has
        /// no parent contracts.
        blocked_on_parent: ::std::string::String,
        /// Best candidate code identified during rollouts.
        best_candidate: ::core::option::Option<::std::string::String>,
        /// Highest reward achieved.
        best_reward: ::core::primitive::f64,
        /// Blocker trace from highest reward candidate.
        blocker_trace: ::std::string::String,
    },
}
