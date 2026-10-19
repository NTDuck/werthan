//! Multi-operative ensemble abstraction and ARIADNE MCTS node.

/// Static topology introspection for UI registration and layout subdivisions.
pub trait StaticTopologyNode {
    /// Return the list of sub-operative identifiers.
    fn sub_operatives(&self) -> &[::operative::OperativeId];
}

/// Abstract cluster interface managing local state and multiple operatives.
pub trait Cluster: self::StaticTopologyNode {
    /// Return the cluster identifier string.
    fn cluster_id(&self) -> &::core::primitive::str;

    /// Return the target module identifier.
    fn module_id(&self) -> &::core::primitive::str;

    /// Return reference to the four-slice local blackboard.
    fn local_blackboard(&self) -> ::std::sync::Arc<::tokio::sync::RwLock<self::LocalBlackboard>>;
}

/// Problem Model slice of the local blackboard containing legacy specifications.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProblemModelSlice {
    /// Extracted business logic rules.
    pub business_rules: ::std::vec::Vec<::std::string::String>,
    /// Upstream dependency interface contracts.
    pub edge_contracts: ::std::vec::Vec<::std::string::String>,
    /// Side-effect and state invariants.
    pub invariants: ::std::vec::Vec<::std::string::String>,
}

/// Strategy Hypotheses slice containing candidate architectural transformation plans.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct StrategyHypothesesSlice {
    /// Candidate modernization hypotheses.
    pub hypotheses: ::std::vec::Vec<::std::string::String>,
    /// Active selected hypothesis index.
    pub active_index: ::core::option::Option<::core::primitive::usize>,
}

/// Characterization Tests slice containing behavioral equivalence test fixtures.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct CharacterizationTestsSlice {
    /// Test case source strings.
    pub test_cases: ::std::vec::Vec<::std::string::String>,
    /// Property-based testing assertions.
    pub property_assertions: ::std::vec::Vec<::std::string::String>,
}

/// Patched Code slice containing candidate implementations and evaluation traces.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct PatchedCodeSlice {
    /// Latest candidate target code draft.
    pub draft_source: ::std::string::String,
    /// Compiler diagnostic messages.
    pub compiler_diagnostics: ::std::vec::Vec<::std::string::String>,
    /// Current scalar reward score.
    pub current_reward: ::core::primitive::f64,
}

/// Four-slice local blackboard store scoped to a single module vertex.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct LocalBlackboard {
    /// Problem model slice.
    pub problem_model: self::ProblemModelSlice,
    /// Strategy hypotheses slice.
    pub strategy_hypotheses: self::StrategyHypothesesSlice,
    /// Characterization tests slice.
    pub characterization_tests: self::CharacterizationTestsSlice,
    /// Patched code and evaluation slice.
    pub patched_code: self::PatchedCodeSlice,
}

/// Scalar reward weight coefficients summing to 1.0.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct RewardWeights {
    /// Weight for compilation status (w1).
    pub compile: ::core::primitive::f64,
    /// Weight for characterization test parity (w2).
    pub parity: ::core::primitive::f64,
    /// Weight for hardening and linter score (w3).
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

/// ARIADNE Monte Carlo Tree Search cluster managing intra-module synthesis.
pub struct AriadneCluster {
    /// Unique cluster identifier.
    pub id: ::std::string::String,
    /// Target module identifier.
    pub module_id: ::std::string::String,
    /// Local four-slice blackboard store.
    pub blackboard: ::std::sync::Arc<::tokio::sync::RwLock<self::LocalBlackboard>>,
    /// Configured reward weights.
    pub reward_weights: self::RewardWeights,
    /// Maximum rollout budget per search invocation.
    pub mcts_budget: ::core::primitive::usize,
    /// Softmax exploration parameter constant.
    pub ucb_exploration_c: ::core::primitive::f64,
    /// Statically configured internal operative identifiers.
    pub sub_operative_ids: ::std::vec::Vec<::operative::OperativeId>,
}

impl self::AriadneCluster {
    /// Construct a new ARIADNE cluster for a specific module.
    pub fn new(module_id: &::core::primitive::str, budget: ::core::primitive::usize) -> Self {
        let sub_operative_ids = ::std::vec![
            ::operative::OperativeId("a_rule".to_string()),
            ::operative::OperativeId("a_str".to_string()),
            ::operative::OperativeId("a_gen".to_string()),
            ::operative::OperativeId("a_test".to_string()),
            ::operative::OperativeId("a_eval".to_string()),
            ::operative::OperativeId("a_patch".to_string()),
        ];

        let default_bb = self::LocalBlackboard::default();
        let lock_bb = ::tokio::sync::RwLock::new(default_bb);
        let arc_bb = ::std::sync::Arc::new(lock_bb);
        let default_weights = self::RewardWeights::default();
        let mod_id_str = module_id.to_string();

        Self {
            id: { ::std::format!("ariadne_{module_id}") },
            module_id: { mod_id_str },
            blackboard: { arc_bb },
            reward_weights: { default_weights },
            mcts_budget: { budget },
            ucb_exploration_c: 1.414,
            sub_operative_ids: { sub_operative_ids },
        }
    }

    /// Execute single search step updating reward.
    pub async fn step(&mut self, compiles: ::core::primitive::bool, pass_rate: ::core::primitive::f64) -> ::core::primitive::f64 {
        let reward = self.calculate_gated_reward(compiles, pass_rate, 1.0);
        let mut bb = self.blackboard.write().await;
        bb.patched_code.current_reward = reward;
        reward
    }

    /// Calculate gated scalar reward for candidate target code.
    pub fn calculate_gated_reward(
        &self,
        compiles: ::core::primitive::bool,
        test_pass_rate: ::core::primitive::f64,
        hardening_score: ::core::primitive::f64,
    ) -> ::core::primitive::f64 {
        if !compiles {
            return 0.0;
        }

        let w1 = self.reward_weights.compile;
        let w2 = self.reward_weights.parity;
        let w3 = self.reward_weights.harden;

        let clamped_pass = test_pass_rate.clamp(0.0, 1.0);
        let clamped_harden = hardening_score.clamp(0.0, 1.0);

        w1 + (w2 * clamped_pass) + (w3 * clamped_harden)
    }
}

impl self::StaticTopologyNode for self::AriadneCluster {
    fn sub_operatives(&self) -> &[::operative::OperativeId] {
        &self.sub_operative_ids
    }
}

impl self::Cluster for self::AriadneCluster {
    fn cluster_id(&self) -> &::core::primitive::str {
        &self.id
    }

    fn module_id(&self) -> &::core::primitive::str {
        &self.module_id
    }

    fn local_blackboard(&self) -> ::std::sync::Arc<::tokio::sync::RwLock<self::LocalBlackboard>> {
        ::std::sync::Arc::clone(&self.blackboard)
    }
}
