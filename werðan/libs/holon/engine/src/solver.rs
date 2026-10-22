//! Adapter implementing the macro engine solver contract over the ARIADNE controller.
//! Maps EngineTypestateOutcome variants onto fsm SolverOutcome
//! per METHODOLOGY.md Algorithm 2 module boundary.

/// Thin wrapper holding one MCTS controller and local blackboard per module vertex.
/// The solver stores the cohesive components themselves. Every per-module
/// controller assembles from clones through the engine bon builder.
pub struct AriadneSolver<
    M: ::rig::completion::CompletionModel,
    H = ::holon_evaluator::DefaultExecutionHarness,
    D = ::holon_evaluator::DefaultDiagnosticDistiller,
> {
    config: crate::types::EngineConfig,
    controllers: ::std::collections::BTreeMap<::std::string::String, crate::ClusterEngine<M, H, D>>,
    agents: crate::agents::SubagentPool<M>,
    harness: H,
    distiller: D,
}

impl<M: ::rig::completion::CompletionModel, H, D> ::core::fmt::Debug for self::AriadneSolver<M, H, D> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("AriadneSolver")
            .field("config", &self.config)
            .finish()
    }
}

#[::bon::bon]
impl<
        M: ::rig::completion::CompletionModel + 'static,
        H: ::holon_evaluator::ExecutionHarness + ::core::clone::Clone,
        D: ::holon_evaluator::DiagnosticDistiller + ::core::clone::Clone,
    > self::AriadneSolver<M, H, D>
{
    /// Construct an adapter over independently built components.
    /// Callers construct the subagent pool, harness, and distiller through
    /// their own bon builders and pass them in directly.
    #[builder]
    pub fn builder(
        config: crate::types::EngineConfig,
        agents: crate::agents::SubagentPool<M>,
        harness: H,
        distiller: D,
    ) -> Self {
        Self {
            config,
            controllers: ::std::collections::BTreeMap::new(),
            agents,
            harness,
            distiller,
        }
    }

    /// Return the per-module controller, creating one with a fresh local blackboard on demand.
    fn controller_entry(
        &mut self,
        module_id: &::core::primitive::str,
        problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> &mut crate::ClusterEngine<M, H, D> {
        let absent = !self.controllers.contains_key(module_id);
        if absent {
            let mut local = ::holon_blackboard::LocalBlackboard::new();
            local.module_id = module_id.to_string();
            let evaluator = ::holon_evaluator::TwoStageEvaluator::builder()
                .harness(self.harness.clone())
                .distiller(self.distiller.clone())
                .build();
            let controller = crate::ClusterEngine::builder()
                .config(self.config.clone())
                .blackboard(local)
                .evaluator(evaluator)
                .agents(self.agents.clone())
                .build();
            self.controllers.insert(module_id.to_string(), controller);
        }
        let controller = self.controllers.get_mut(module_id).expect("controller inserted above");
        controller.blackboard_mut().module_id = module_id.to_string();
        controller.blackboard_mut().problem_model = problem_model.clone();
        controller
    }
}

/// Build an M2 state machine parked at the synthesis entry for one module dispatch.
fn build_m2_machine(
    module_id: &::core::primitive::str,
    mode: ::holon_fsm::ActivationMode,
) -> ::holon_fsm::StateMachine<::holon_fsm::M2AriadneMcts> {
    let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new(module_id);
    let mut machine_m0 = machine_m0;
    machine_m0.activation_mode = mode;
    let machine_m1 = ::holon_fsm::StateMachine::<::holon_fsm::M1ContextHydration>::from(machine_m0);
    let machine_m2 = ::holon_fsm::StateMachine::<::holon_fsm::M2AriadneMcts>::from(machine_m1);
    machine_m2
}

/// Map the strongly-typed controller outcome onto the macro solver outcome.
fn map_typed_outcome(outcome: crate::types::EngineTypestateOutcome) -> ::global_engine::SolverOutcome {
    match outcome {
        crate::types::EngineTypestateOutcome::Hardened(machine) => {
            let payload = machine.state;
            ::global_engine::SolverOutcome::Hardened {
                target_code: { payload.target_code },
                test_suite: { payload.test_suite },
            }
        }
        crate::types::EngineTypestateOutcome::ContractConflict(machine) => {
            let payload = machine.state;
            ::global_engine::SolverOutcome::ContractConflict {
                parent_id: { payload.parent_id },
                delta_spec: { payload.delta_spec },
                proof_trace: { payload.proof_trace },
            }
        }
        crate::types::EngineTypestateOutcome::CycleFound {
            cycle_edges,
            proof_trace,
            ..
        } => ::global_engine::SolverOutcome::CycleFound {
            cycle_edges: { cycle_edges },
            proof_trace: { proof_trace },
        },
        crate::types::EngineTypestateOutcome::BudgetExhausted { machine, .. } => {
            let payload = machine.state;
            ::global_engine::SolverOutcome::ContractConflict {
                parent_id: { payload.parent_id },
                delta_spec: { payload.delta_spec },
                proof_trace: { payload.proof_trace },
            }
        }
    }
}

impl<
        M: ::rig::completion::CompletionModel + 'static,
        H: ::holon_evaluator::ExecutionHarness + ::core::clone::Clone + 'static,
        D: ::holon_evaluator::DiagnosticDistiller + ::core::clone::Clone + 'static,
    > ::global_engine::ModuleSolver for self::AriadneSolver<M, H, D>
{
    fn solve(
        &mut self,
        module_id: &::core::primitive::str,
        problem_model: &::holon_blackboard::ProblemModelSlice,
        mode: ::holon_fsm::ActivationMode,
    ) -> ::global_engine::SolverOutcome {
        let controller = self.controller_entry(module_id, problem_model);
        let machine = self::build_m2_machine(module_id, mode);
        let typed_outcome = controller.solve(machine);
        self::map_typed_outcome(typed_outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic completion double for solver-level tests.
    #[derive(::core::clone::Clone)]
    struct FactoryFakeModel {
        response_text: ::std::string::String,
    }

    impl ::rig::completion::CompletionModel for self::FactoryFakeModel {
        type Response = ::serde_json::Value;

        async fn completion(
            &self,
            _request: ::rig::completion::CompletionRequest,
        ) -> ::core::result::Result<
            ::rig::completion::CompletionResponse<Self::Response>,
            ::rig::completion::CompletionError,
        > {
            let content = ::rig::message::AssistantContent::text(self.response_text.clone());
            let choice = ::rig::one_or_many::OneOrMany::one(content);
            ::core::result::Result::Ok(::rig::completion::CompletionResponse {
                choice,
                raw_response: ::serde_json::Value::Null,
            })
        }
    }


    #[test]
    fn test_trivial_problem_model_hardens() {
        let config = crate::types::EngineConfig::default();
        let fake_model = self::FactoryFakeModel {
            response_text: "pub fn factory_probe() -> u32 {\n    // R001 valid payload\n    1\n}".to_string(),
        };
        let agents = self::build_test_pool(&fake_model);
        let solver = self::AriadneSolver::builder()
            .config(config)
            .agents(agents)
            .harness(::holon_evaluator::DefaultExecutionHarness)
            .distiller(::holon_evaluator::DefaultDiagnosticDistiller)
            .build();
        let mut solver = solver;
        let mut model = ::holon_blackboard::ProblemModelSlice::new();
        let rule = ::holon_blackboard::BusinessRuleCard {
            rule_id: "R001".to_string(),
            given: "valid payload".to_string(),
            when: "payload parsed".to_string(),
            then: "returns syntax tree".to_string(),
            citation: "parser.c:12".to_string(),
        };
        model.add_rule(rule);
        let outcome = ::global_engine::ModuleSolver::solve(
            &mut solver,
            "module_x",
            &model,
            ::holon_fsm::ActivationMode::Initial,
        );
        ::std::assert!(::core::matches!(
            outcome,
            ::global_engine::SolverOutcome::Hardened { .. }
        ));
    }

    #[test]
    fn test_builder_components_harden() {
        let config = crate::types::EngineConfig::default();
        let fake_model = self::FactoryFakeModel {
            response_text: "pub fn factory_probe() -> u32 {\n    // R002 payload parsed returns syntax tree\n    1\n}".to_string(),
        };
        let agents = self::build_test_pool(&fake_model);
        let solver = self::AriadneSolver::builder()
            .config(config)
            .agents(agents)
            .harness(::holon_evaluator::DefaultExecutionHarness)
            .distiller(::holon_evaluator::DefaultDiagnosticDistiller)
            .build();
        let mut solver = solver;
        let mut model = ::holon_blackboard::ProblemModelSlice::new();
        let rule = ::holon_blackboard::BusinessRuleCard {
            rule_id: "R002".to_string(),
            given: "valid payload".to_string(),
            when: "payload parsed".to_string(),
            then: "returns syntax tree".to_string(),
            citation: "parser.c:12".to_string(),
        };
        model.add_rule(rule);
        let outcome = ::global_engine::ModuleSolver::solve(
            &mut solver,
            "module_factory",
            &model,
            ::holon_fsm::ActivationMode::Initial,
        );
        ::std::assert!(::core::matches!(
            outcome,
            ::global_engine::SolverOutcome::Hardened { .. }
        ));
    }
    /// Build a subagent pool from one completion model for solver tests.
    fn build_test_pool(
        model: &self::FactoryFakeModel,
    ) -> crate::agents::SubagentPool<self::FactoryFakeModel> {
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
            .model(model.clone())
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
