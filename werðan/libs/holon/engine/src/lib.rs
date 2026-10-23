//! Encapsulated ARIADNE Monte Carlo Tree Search cluster controller.
//! Implements intra-module synthesis, Softmax-UCB tree search, and multi-tier regressions.

pub mod agents;
pub mod controller;
pub mod reward;
pub mod solver;
pub mod tree;
pub mod types;

pub use self::agents::SubagentPool;
pub use self::controller::ClusterEngine;
pub use self::reward::compute_gated_reward;
pub use self::tree::{SearchNode, SearchTree};
pub use self::types::{EngineConfig, EngineTypestateOutcome, RewardWeights};

#[cfg(test)]
pub(crate) mod test_support {

    /// Deterministic completion model double for controller-level tests.
    /// Serves canned synthesis output that pins the rule ids found in the
    /// prompt's business-rule section, mirroring what a compliant model does.
    #[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default)]
    pub struct FakeAriadneModel {
        pub response_text: ::std::string::String,
    }

    impl self::FakeAriadneModel {
        /// Model serving canned codegen output derived from the prompt rules.
        pub fn canned() -> Self {
            Self {
                response_text: ::std::string::String::new(),
            }
        }
    }

    impl ::rig::completion::CompletionModel for self::FakeAriadneModel {
        type Response = ::serde_json::Value;

        async fn completion(
            &self,
            request: ::rig::completion::CompletionRequest,
        ) -> ::core::result::Result<
            ::rig::completion::CompletionResponse<Self::Response>,
            ::rig::completion::CompletionError,
        > {
            let request_preamble = request.preamble.unwrap_or_default();
            let is_codegen = request_preamble.contains("code synthesis engine");
            if is_codegen {
                let mut rule_lines = ::std::vec::Vec::<::std::string::String>::new();
                let prompt_message = request.prompt;
                let prompt_text = match prompt_message {
                    ::rig::message::Message::User { content } => {
                        let text_block = content.first();
                        match text_block {
                            ::rig::message::UserContent::Text(text_value) => text_value.text,
                            _ => ::std::string::String::new(),
                        }
                    }
                    _ => ::std::string::String::new(),
                };
                let section_start = prompt_text.find("[BUSINESS RULES]");
                if let ::core::option::Option::Some(start_index) = section_start {
                    let section_text = &prompt_text[start_index..];
                    let section_end = section_text
                        .find("[PARENT CONTRACTS]")
                        .unwrap_or(section_text.len());
                    for section_line in section_text[..section_end].lines() {
                        let trimmed_line = section_line.trim();
                        if let ::core::option::Option::Some(rest) = trimmed_line.strip_prefix("- ") {
                            if let ::core::option::Option::Some(tag_end) = rest.find(" |") {
                                rule_lines.push(::std::format!("// {}", &rest[..tag_end]));
                            }
                        }
                    }
                }
                rule_lines.push("pub fn execute_modernized() -> bool { true }".to_string());
                let content = ::rig::message::AssistantContent::text(rule_lines.join("\n"));
                let choice = ::rig::one_or_many::OneOrMany::one(content);
                return ::core::result::Result::Ok(::rig::completion::CompletionResponse {
                    choice,
                    raw_response: ::serde_json::Value::Null,
                });
            }
            let content = ::rig::message::AssistantContent::text(self.response_text.clone());
            let choice = ::rig::one_or_many::OneOrMany::one(content);
            ::core::result::Result::Ok(::rig::completion::CompletionResponse {
                choice,
                raw_response: ::serde_json::Value::Null,
            })
        }
    }

    /// Build a specialist pool from one fake completion model for controller tests.
    /// Each role agent assembles through its holon crate builder over the shared model.
    pub fn build_fake_pool(
        model: self::FakeAriadneModel,
    ) -> crate::agents::SubagentPool<self::FakeAriadneModel> {
        let strategy_agent = ::holon_strategy::HolonStrategyAgent::builder()
            .model(model.clone())
            .temperature(0.2)
            .build();
        let codegen_agent = ::holon_codegen::HolonCodeGenAgent::builder()
            .model(model.clone())
            .temperature(0.2)
            .build();
        let testgen_agent = ::holon_testgen::HolonTestGenAgent::builder()
            .model(model.clone())
            .temperature(0.2)
            .build();
        let patch_agent = ::holon_patch::HolonPatchAgent::builder()
            .model(model)
            .temperature(0.2)
            .build();
        let strategy_handle = ::std::sync::Arc::new(strategy_agent);
        let codegen_handle = ::std::sync::Arc::new(codegen_agent);
        let testgen_handle = ::std::sync::Arc::new(testgen_agent);
        let patch_handle = ::std::sync::Arc::new(patch_agent);
        crate::agents::SubagentPool::builder()
            .strategy_agent(strategy_handle)
            .code_gen_agent(codegen_handle)
            .test_gen_agent(testgen_handle)
            .patch_agent(patch_handle)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_successful_synthesis_flow() {
        let mut bb = ::holon_blackboard::LocalBlackboard::new();
        let rule = ::holon_blackboard::BusinessRuleCard {
            rule_id: "RULE_PARITY_001".to_string(),
            given: "input vector".to_string(),
            when: "transformed".to_string(),
            then: "matches legacy behavior".to_string(),
            citation: "legacy.c:42".to_string(),
        };
        bb.problem_model.add_rule(rule);

        let config = self::EngineConfig {
            max_rollouts: 5,
            ..::core::default::Default::default()
        };

        let fake_model = self::test_support::FakeAriadneModel::canned();
        let default_evaluator = ::holon_evaluator::TwoStageEvaluator::<
            ::holon_evaluator::DefaultExecutionHarness,
            ::holon_evaluator::DefaultDiagnosticDistiller,
        >::default();
        let fake_agents = self::test_support::build_fake_pool(fake_model);
        let mut controller = self::ClusterEngine::builder()
            .config(config)
            .blackboard(bb)
            .evaluator(default_evaluator)
            .agents(fake_agents)
            .build();
        let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("module_omega");
        let machine_m2 = machine_m0.wake_ready().hydration_completed();

        let outcome = controller.solve(machine_m2);
        match outcome {
            self::EngineTypestateOutcome::Hardened(hardened_machine) => {
                let reward = hardened_machine.state.reward;
                ::std::assert!((reward - 1.0).abs() < 1e-6);
                let hardened_code = hardened_machine.state.target_code;
                ::std::assert!(hardened_code.contains("pub fn execute_modernized"));
            }
            _ => ::std::panic!("expected hardened outcome"),
        }
    }

    #[test]
    fn test_strongly_typed_solve_flow() {
        let mut bb = ::holon_blackboard::LocalBlackboard::new();
        let rule = ::holon_blackboard::BusinessRuleCard {
            rule_id: "RULE_PARITY_002".to_string(),
            given: "input vector".to_string(),
            when: "transformed".to_string(),
            then: "matches legacy behavior".to_string(),
            citation: "legacy.c:99".to_string(),
        };
        bb.problem_model.add_rule(rule);

        let config = self::EngineConfig {
            max_rollouts: 5,
            ..::core::default::Default::default()
        };

        let fake_model = self::test_support::FakeAriadneModel::canned();
        let default_evaluator = ::holon_evaluator::TwoStageEvaluator::<
            ::holon_evaluator::DefaultExecutionHarness,
            ::holon_evaluator::DefaultDiagnosticDistiller,
        >::default();
        let fake_agents = self::test_support::build_fake_pool(fake_model);
        let mut controller = self::ClusterEngine::builder()
            .config(config)
            .blackboard(bb)
            .evaluator(default_evaluator)
            .agents(fake_agents)
            .build();
        let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("module_typestate");
        let machine_m1 = ::holon_fsm::StateMachine::<::holon_fsm::M1ContextHydration>::from(machine_m0);
        let machine_m2 = ::holon_fsm::StateMachine::<::holon_fsm::M2AriadneMcts>::from(machine_m1);

        let outcome = controller.solve(machine_m2);
        match outcome {
            self::EngineTypestateOutcome::Hardened(hardened_machine) => {
                let reward = hardened_machine.state.reward;
                ::std::assert!((reward - 1.0).abs() < 1e-6);
                ::std::assert_eq!(hardened_machine.history.visited_states.last(), ::core::option::Option::Some(&::holon_fsm::MicroStateTag::M4Hardened));
            }
            _ => ::std::panic!("expected hardened typestate outcome"),
        }
    }

    struct ConflictHarness;

    impl ::holon_evaluator::ExecutionHarness for self::ConflictHarness {
        fn execute_candidate(
            &self,
            _module_id: &::core::primitive::str,
            _candidate_code: &::core::primitive::str,
            _test_suite: &[::std::string::String],
        ) -> ::holon_evaluator::HarnessExecutionOutput {
            ::holon_evaluator::HarnessExecutionOutput {
                compiles: false,
                pass_rate: 0.0,
                raw_logs: "error[E0425]: parent_alpha CONTRACT_CONFLICT missing symbol".to_string(),
            }
        }
    }

    struct CycleHarness;

    impl ::holon_evaluator::ExecutionHarness for self::CycleHarness {
        fn execute_candidate(
            &self,
            _module_id: &::core::primitive::str,
            _candidate_code: &::core::primitive::str,
            _test_suite: &[::std::string::String],
        ) -> ::holon_evaluator::HarnessExecutionOutput {
            ::holon_evaluator::HarnessExecutionOutput {
                compiles: false,
                pass_rate: 0.0,
                raw_logs: "fatal error: cyclic dependency between modules detected".to_string(),
            }
        }
    }

    #[test]
    fn test_contract_conflict_escalation() {
        let mut bb = ::holon_blackboard::LocalBlackboard::new();
        let contract = ::holon_blackboard::EdgeContract {
            parent_id: "parent_alpha".to_string(),
            interface_spec: "pub fn compute(x: i32) -> i32".to_string(),
            preconditions: ::std::vec![],
        };
        bb.problem_model.add_parent_contract(contract);

        let harness = self::ConflictHarness;
        let distiller = ::holon_evaluator::DefaultDiagnosticDistiller;
        let evaluator = ::holon_evaluator::TwoStageEvaluator::builder()
            .harness(harness)
            .distiller(distiller)
            .build();
        let fake_model = self::test_support::FakeAriadneModel::canned();
        let agents = self::test_support::build_fake_pool(fake_model);

        let config = self::EngineConfig {
            max_rollouts: 3,
            ..::core::default::Default::default()
        };

        let mut controller = self::ClusterEngine::builder()
            .config(config)
            .blackboard(bb)
            .evaluator(evaluator)
            .agents(agents)
            .build();
        let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("module_child");
        let machine_m2 = machine_m0.wake_ready().hydration_completed();

        let outcome = controller.solve(machine_m2);
        match outcome {
            self::EngineTypestateOutcome::ContractConflict(conflict_machine) => {
                let conflict_parent = conflict_machine.state.parent_id;
                ::std::assert_eq!(conflict_parent, "parent_alpha");
            }
            _ => ::std::panic!("expected contract conflict outcome"),
        }
    }

    #[test]
    fn test_cycle_detection_escalation() {
        let mut bb = ::holon_blackboard::LocalBlackboard::new();
        let contract = ::holon_blackboard::EdgeContract {
            parent_id: "parent_cycle".to_string(),
            interface_spec: "stub".to_string(),
            preconditions: ::std::vec![],
        };
        bb.problem_model.add_parent_contract(contract);

        let harness = self::CycleHarness;
        let distiller = ::holon_evaluator::DefaultDiagnosticDistiller;
        let evaluator = ::holon_evaluator::TwoStageEvaluator::builder()
            .harness(harness)
            .distiller(distiller)
            .build();
        let fake_model = self::test_support::FakeAriadneModel::canned();
        let agents = self::test_support::build_fake_pool(fake_model);

        let config = self::EngineConfig {
            max_rollouts: 3,
            ..::core::default::Default::default()
        };

        let mut controller = self::ClusterEngine::builder()
            .config(config)
            .blackboard(bb)
            .evaluator(evaluator)
            .agents(agents)
            .build();
        let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("module_cyclic");
        let machine_m2 = machine_m0.wake_ready().hydration_completed();

        let outcome = controller.solve(machine_m2);
        match outcome {
            self::EngineTypestateOutcome::CycleFound { cycle_edges, .. } => {
                ::std::assert_eq!(cycle_edges.len(), 1);
            }
            _ => ::std::panic!("expected cycle found outcome"),
        }
    }

    #[test]
    fn test_warm_start_preserves_tree() {
        // VAL-AEVAL-04: an empty rule set yields a zero harden score. The
        // hardening path needs one rule card whose tag the canned model echoes.
        let mut bb = ::holon_blackboard::LocalBlackboard::new();
        let warm_rule = ::holon_blackboard::BusinessRuleCard {
            rule_id: "RULE_WARM_001".to_string(),
            given: "warm module".to_string(),
            when: "warm start invoked".to_string(),
            then: "tree retained".to_string(),
            citation: "legacy/warm.c:7".to_string(),
        };
        bb.problem_model.add_rule(warm_rule);
        let config = self::EngineConfig::default();
        let fake_model = self::test_support::FakeAriadneModel::canned();
        let default_evaluator = ::holon_evaluator::TwoStageEvaluator::<
            ::holon_evaluator::DefaultExecutionHarness,
            ::holon_evaluator::DefaultDiagnosticDistiller,
        >::default();
        let fake_agents = self::test_support::build_fake_pool(fake_model);
        let mut controller = self::ClusterEngine::builder()
            .config(config)
            .blackboard(bb)
            .evaluator(default_evaluator)
            .agents(fake_agents)
            .build();
        let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("module_warm");
        let machine_m2_first = machine_m0.wake_ready().hydration_completed();

        let first_outcome = controller.solve(machine_m2_first);
        ::std::assert!(::core::matches!(first_outcome, self::EngineTypestateOutcome::Hardened(_)));

        let nodes_before = controller.search_tree().len();
        ::std::assert!(nodes_before > 1);

        let machine_m0_warm = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("module_warm");
        let machine_m2_warm = machine_m0_warm.wake_ready().hydration_completed();
        controller.initialize(::holon_fsm::ActivationMode::WarmStartRegression);
        let warm_outcome = controller.solve(machine_m2_warm);
        // The warm start retains the tree. The canned model serves an empty
        // patch response. The second pass exhausts its budget honestly
        // instead of hardening on a fabricated candidate.
        ::std::assert!(::core::matches!(
            warm_outcome,
            self::EngineTypestateOutcome::Hardened(_)
                | self::EngineTypestateOutcome::BudgetExhausted { .. }
        ));

        let nodes_after = controller.search_tree().len();
        ::std::assert!(nodes_after >= nodes_before);
    }
}
