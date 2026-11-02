//! Four-slice local blackboard store for an isolated module vertex.
//! Follows the ARIADNE intra-module solver specification in METHODOLOGY.md Section 2.

pub mod characterization_tests;
pub mod patch_records;
pub mod problem_model;
pub mod strategy_hypotheses;

pub use self::characterization_tests::CharacterizationTestsSlice;
pub use self::patch_records::{FaultCause, FaultRecord, PatchRecordsSlice};
pub use self::problem_model::{BusinessRuleCard, EdgeContract, ProblemModelSlice};
pub use self::strategy_hypotheses::{StrategyHypothesesSlice, StrategyHypothesis};

/// Four-slice local blackboard store scoped to a single module vertex.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct LocalBlackboard {
    /// Target module vertex identifier.
    pub module_id: ::std::string::String,
    /// Problem model slice with specifications and contracts.
    pub problem_model: self::problem_model::ProblemModelSlice,
    /// Strategy hypotheses slice with refactoring plans.
    pub strategy_hypotheses: self::strategy_hypotheses::StrategyHypothesesSlice,
    /// Characterization tests slice with adversarial fixtures.
    pub characterization_tests: self::characterization_tests::CharacterizationTestsSlice,
    /// Patch records slice with distilled diagnostic traces.
    pub patch_records: self::patch_records::PatchRecordsSlice,
}

impl self::LocalBlackboard {
    /// Initialize fresh four-slice local blackboard.
    /// The problem model gets the rust target default: a blank target language
    /// routes the test splitter to the non-rust opener branch and fragments
    /// rust suites.
    pub fn new() -> Self {
        Self {
            problem_model: self::problem_model::ProblemModelSlice::new(),
            ..Self::default()
        }
    }

    /// Check if blackboard problem model contains required domain context.
    pub fn is_hydrated(&self) -> ::core::primitive::bool {
        let has_rules = !self.problem_model.rules.is_empty();
        let has_contracts = !self.problem_model.parent_contracts.is_empty();
        has_rules || has_contracts
    }

    /// Clear transient rollout artifacts during warm start regression while retaining tree knowledge.
    pub fn reset_transient_records(&mut self) {
        self.patch_records.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blackboard_initialization() {
        let bb = self::LocalBlackboard::new();
        ::std::assert!(!bb.is_hydrated());
        ::std::assert_eq!(bb.problem_model.rule_count(), 0);
        ::std::assert_eq!(bb.strategy_hypotheses.uninstantiated_hypotheses().len(), 0);
        ::std::assert_eq!(bb.characterization_tests.test_count(), 0);
        ::std::assert_eq!(bb.patch_records.fault_count(), 0);
    }

    #[test]
    fn test_problem_model_hydration() {
        let mut bb = self::LocalBlackboard::new();
        let rule = self::BusinessRuleCard {
            rule_id: "RULE_001".to_string(),
            given: "valid payload".to_string(),
            when: "parsed".to_string(),
            then: "returns syntax tree".to_string(),
            citation: "parser.c:12".to_string(),
        };
        bb.problem_model.add_rule(rule);
        ::std::assert!(bb.is_hydrated());
        ::std::assert_eq!(bb.problem_model.rule_count(), 1);

        let delta = "requires timeout limit".to_string();
        bb.problem_model.add_delta_constraint(delta);
        ::std::assert_eq!(bb.problem_model.delta_constraints.len(), 1);
    }

    #[test]
    fn test_strategy_lifecycle() {
        let mut bb = self::LocalBlackboard::new();
        let hyp = self::StrategyHypothesis {
            strategy_id: "STRAT_001".to_string(),
            description: "port to idiomatic rust iterator".to_string(),
            prior_utility: 0.85,
            instantiated: false,
        };
        bb.strategy_hypotheses.add_hypothesis(hyp);
        let uninst = bb.strategy_hypotheses.uninstantiated_hypotheses();
        ::std::assert_eq!(uninst.len(), 1);

        bb.strategy_hypotheses.mark_instantiated("STRAT_001");
        let remaining = bb.strategy_hypotheses.uninstantiated_hypotheses();
        ::std::assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_patch_fault_recording() {
        let mut bb = self::LocalBlackboard::new();
        let fault = self::FaultRecord {
            summary: "type mismatch".to_string(),
            diagnostic_trace: "expected u32 found i32".to_string(),
            cause: ::core::option::Option::Some(self::FaultCause::Intrinsic),
        };
        bb.patch_records.record_fault(fault);
        ::std::assert_eq!(bb.patch_records.fault_count(), 1);
        let latest = bb.patch_records.latest_fault();
        ::std::assert!(latest.is_some());
    }

    #[test]
    fn test_reset_transient_records() {
        let mut bb = self::LocalBlackboard::new();
        let fault = self::FaultRecord::default();
        bb.patch_records.record_fault(fault);
        ::std::assert_eq!(bb.patch_records.fault_count(), 1);
        bb.reset_transient_records();
        ::std::assert_eq!(bb.patch_records.fault_count(), 0);
    }
}
