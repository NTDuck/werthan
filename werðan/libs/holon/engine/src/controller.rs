//! Deterministic MCTS cluster controller.
//! Governs the intra-module transformation loop and multi-tier regressions.

enum StepResult {
    Hardened {
        target_code: ::std::string::String,
        reward: ::core::primitive::f64,
    },
    ContractConflict {
        parent_id: ::std::string::String,
        delta_spec: ::std::string::String,
        proof_trace: ::std::string::String,
    },
    CycleFound {
        cycle_edges: ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
        proof_trace: ::std::string::String,
    },
}

/// Monte Carlo Tree Search controller managing intra-module synthesis.
pub struct ClusterEngine<
    M: ::rig::completion::CompletionModel,
    H = ::holon_evaluator::DefaultExecutionHarness,
    D = ::holon_evaluator::DefaultDiagnosticDistiller,
> {
    config: crate::types::EngineConfig,
    blackboard: ::holon_blackboard::LocalBlackboard,
    tree: crate::tree::SearchTree,
    evaluator: ::holon_evaluator::TwoStageEvaluator<H, D>,
    agents: crate::agents::SubagentPool<M>,
    rollout_count: ::core::primitive::usize,
    best_candidate_code: ::core::option::Option<::std::string::String>,
    best_reward_score: ::core::primitive::f64,
    last_pass_rate: ::core::option::Option<::core::primitive::f64>,
}

impl<M: ::rig::completion::CompletionModel, H, D> ::core::fmt::Debug for self::ClusterEngine<M, H, D> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ClusterEngine")
            .field("config", &self.config)
            .field("rollout_count", &self.rollout_count)
            .field("best_candidate_code", &self.best_candidate_code)
            .field("best_reward_score", &self.best_reward_score)
            .finish()
    }
}

#[::bon::bon]
impl<
        M: ::rig::completion::CompletionModel + ::core::clone::Clone + 'static,
        H: ::holon_evaluator::ExecutionHarness,
        D: ::holon_evaluator::DiagnosticDistiller,
    > self::ClusterEngine<M, H, D>
{
    /// Construct a controller from explicit cohesive components.
    /// The search tree, rollout counters, and best-candidate slots initialize
    /// to their empty per-instance state.
    #[builder]
    pub fn builder(
        config: crate::types::EngineConfig,
        blackboard: ::holon_blackboard::LocalBlackboard,
        evaluator: ::holon_evaluator::TwoStageEvaluator<H, D>,
        agents: crate::agents::SubagentPool<M>,
    ) -> Self {
        let tree = crate::tree::SearchTree::new(::core::option::Option::None);
        Self {
            config,
            blackboard,
            tree,
            evaluator,
            agents,
            rollout_count: 0,
            last_pass_rate: ::core::option::Option::None,
            best_candidate_code: ::core::option::Option::None,
            best_reward_score: 0.0,
        }
    }

    /// Return reference to the four-slice local blackboard.
    pub fn blackboard(&self) -> &::holon_blackboard::LocalBlackboard {
        &self.blackboard
    }

    /// Return mutable reference to the local blackboard.
    pub fn blackboard_mut(&mut self) -> &mut ::holon_blackboard::LocalBlackboard {
        &mut self.blackboard
    }

    /// Return reference to the MCTS search tree.
    pub fn search_tree(&self) -> &crate::tree::SearchTree {
        &self.tree
    }

    /// Return number of completed search rollouts.
    pub fn rollout_count(&self) -> ::core::primitive::usize {
        self.rollout_count
    }

    /// Execute MCTS synthesis directly over strongly-typed StateMachine in M2 AriadneMcts state.
    /// Enforces compile-time typestate progression as specified in METHODOLOGY.md.
    pub fn solve(
        &mut self,
        mut machine: ::holon_fsm::StateMachine<::holon_fsm::M2AriadneMcts>,
    ) -> crate::types::EngineTypestateOutcome {
        let mode = machine.activation_mode;
        self.initialize(mode);

        let budget = self.config.max_rollouts;
        for _ in 0..budget {
            let iter_res = self.step_core();
            match iter_res {
                ::core::option::Option::Some(self::StepResult::Hardened { target_code, reward }) => {
                    let test_suite = self.blackboard.characterization_tests.test_cases.clone();
                    let hardened_machine = machine.into_hardened(target_code, test_suite, reward);
                    return crate::types::EngineTypestateOutcome::Hardened(hardened_machine);
                }
                ::core::option::Option::Some(self::StepResult::ContractConflict {
                    parent_id,
                    delta_spec,
                    proof_trace,
                }) => {
                    let conflict_machine = machine.into_contract_negotiate(parent_id, delta_spec, proof_trace);
                    return crate::types::EngineTypestateOutcome::ContractConflict(conflict_machine);
                }
                ::core::option::Option::Some(self::StepResult::CycleFound { cycle_edges, proof_trace }) => {
                    machine.emit_cycle_regression(cycle_edges.clone(), proof_trace.clone());
                    return crate::types::EngineTypestateOutcome::CycleFound {
                        machine,
                        cycle_edges,
                        proof_trace,
                    };
                }
                ::core::option::Option::None => {}
            }
        }

        let parent_id = self.resolve_parent_id();
        let blocker = "Rollout budget exhausted without achieving unit reward".to_string();
        let trace = ::std::format!(
            "Diagnostic fault threshold reached on module {} with parent {}",
            machine.module_id(),
            parent_id,
        );
        let conflict_machine = machine.into_contract_negotiate(parent_id, blocker.clone(), trace);

        crate::types::EngineTypestateOutcome::BudgetExhausted {
            machine: { conflict_machine },
            best_candidate: self.best_candidate_code.clone(),
            best_reward: self.best_reward_score,
            blocker_trace: { blocker },
        }
    }

    /// Initialize controller state based on activation mode.
    pub fn initialize(&mut self, mode: ::holon_fsm::ActivationMode) {
        match mode {
            ::holon_fsm::ActivationMode::Initial => {
                if !self.blackboard.characterization_tests.is_populated() {
                    let tests = ::holon_testgen::HolonTestGenAgent::generate_characterization_tests(
                        &self.agents.test_gen_agent,
                        &self.blackboard.problem_model,
                    );
                    for test in tests {
                        self.blackboard.characterization_tests.add_test_case(test);
                    }
                }
                self.rollout_count = 0;
            }
            ::holon_fsm::ActivationMode::WarmStartRegression => {
                let best_id = self.tree.best_node();
                self.tree.set_warm_root(best_id);
            }
            ::holon_fsm::ActivationMode::CriticRetry => {
                // Re-entry after a validator or critic rejection. Keep the prior
                // search tree and the populated characterization tests, and continue
                // expanding from the existing frontier.
                let best_id = self.tree.best_node();
                self.tree.set_warm_root(best_id);
            }
        }
    }
    fn step_core(&mut self) -> ::core::option::Option<self::StepResult> {
        self.rollout_count += 1;

        // VAL-M2-03(d): demonstrated low parity invokes the adversarial test
        // agent to expand coverage. The agent stays blind to candidate code.
        // INITIAL-mode seeding already covers the empty-suite case.
        let coverage_low = ::core::matches!(
            self.last_pass_rate,
            ::core::option::Option::Some(rate_value) if rate_value < 0.999_999
        );
        if coverage_low {
            let seeded_tests = ::holon_testgen::HolonTestGenAgent::generate_characterization_tests(
                &self.agents.test_gen_agent,
                &self.blackboard.problem_model,
            );
            for test_entry in seeded_tests {
                self.blackboard.characterization_tests.add_test_case(test_entry);
            }
        }

        let expandable_id = self.tree.select_expandable(
            self.config.ucb_c_exp,
            self.config.gamma_prior,
            self.config.softmax_temperature,
        );

        let expansion_res = self.expand_node(expandable_id);
        let child_id = expansion_res.0;
        let candidate_code = expansion_res.1;

        let eval_res = self.evaluator.evaluate(
            &self.blackboard.module_id,
            &candidate_code,
            &self.blackboard.characterization_tests.test_cases,
            &self.blackboard.problem_model,
        );

        self.last_pass_rate = ::core::option::Option::Some(eval_res.test_pass_rate);
        let reward = crate::reward::compute_gated_reward(
            &self.config.reward_weights,
            eval_res.compiles,
            eval_res.test_pass_rate,
            eval_res.hardening_score,
        );
        self.tree.backpropagate(child_id, reward);

        if reward > self.best_reward_score {
            self.best_reward_score = reward;
            self.best_candidate_code = ::core::option::Option::Some(candidate_code.clone());
        }

        if let ::core::option::Option::Some(fault) = &eval_res.fault_record {
            self.blackboard.patch_records.record_fault(fault.clone());

            if let ::core::option::Option::Some(::holon_blackboard::FaultCause::ExtrinsicContract) = fault.cause {
                let parent_id = self.resolve_parent_id();
                let delta_spec = fault.summary.clone();
                let proof_trace = fault.diagnostic_trace.clone();
                let res = self::StepResult::ContractConflict {
                    parent_id,
                    delta_spec,
                    proof_trace,
                };
                return ::core::option::Option::Some(res);
            }

            if let ::core::option::Option::Some(::holon_blackboard::FaultCause::ExtrinsicCycle) = fault.cause {
                let parent_id = self.resolve_parent_id();
                let edge = (parent_id, "module".to_string());
                let cycle_edges = ::std::vec![edge];
                let proof_trace = fault.diagnostic_trace.clone();
                let res = self::StepResult::CycleFound {
                    cycle_edges,
                    proof_trace,
                };
                return ::core::option::Option::Some(res);
            }
        }

        if (reward - 1.0).abs() < 1e-6 {
            let res = self::StepResult::Hardened {
                target_code: { candidate_code },
                reward,
            };
            return ::core::option::Option::Some(res);
        }

        ::core::option::Option::None
    }

    fn expand_node(
        &mut self,
        selected_id: ::core::primitive::usize,
    ) -> (::core::primitive::usize, ::std::string::String) {
        if !self.blackboard.strategy_hypotheses.has_uninstantiated() {
            let new_strats = ::holon_strategy::HolonStrategyAgent::generate_strategies(
                &self.agents.strategy_agent,
                &self.blackboard.problem_model,
            );
            for strat in new_strats {
                self.blackboard.strategy_hypotheses.add_hypothesis(strat);
            }
        }

        let selected_code_opt = match self.tree.get(selected_id) {
            ::core::option::Option::Some(node) => node.candidate_code.clone(),
            ::core::option::Option::None => ::core::option::Option::None,
        };

        match selected_code_opt {
            ::core::option::Option::None => {
                let hypothesis = match self.blackboard.strategy_hypotheses.next_uninstantiated() {
                    ::core::option::Option::Some(h) => {
                        h.instantiated = true;
                        h.clone()
                    }
                    ::core::option::Option::None => ::holon_blackboard::StrategyHypothesis {
                        strategy_id: "default_strategy".to_string(),
                        description: "Direct transformation strategy".to_string(),
                        prior_utility: 0.50,
                        instantiated: true,
                    },
                };

                let new_code = ::holon_codegen::HolonCodeGenAgent::generate_candidate(
                    &self.agents.code_gen_agent,
                    &self.blackboard.problem_model,
                    &hypothesis,
                );
                let child_id = self.tree.add_child(
                    selected_id,
                    ::core::option::Option::Some(new_code.clone()),
                    ::core::option::Option::Some(hypothesis.strategy_id),
                    hypothesis.prior_utility,
                );
                (child_id, new_code)
            }
            ::core::option::Option::Some(existing_code) => {
                let fault = match self.blackboard.patch_records.latest_fault() {
                    ::core::option::Option::Some(f) => f.clone(),
                    ::core::option::Option::None => ::holon_blackboard::FaultRecord {
                        summary: "Suboptimal parity score".to_string(),
                        cause: ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic),
                        diagnostic_trace: ::std::string::String::new(),
                    },
                };

                let repaired_code = ::holon_patch::HolonPatchAgent::repair_code(
                    &self.agents.patch_agent,
                    &existing_code,
                    &fault,
                    &self.blackboard.problem_model,
                );

                let child_id = self.tree.add_child(
                    selected_id,
                    ::core::option::Option::Some(repaired_code.clone()),
                    ::core::option::Option::None,
                    0.50,
                );
                (child_id, repaired_code)
            }
        }
    }

    fn resolve_parent_id(&self) -> ::std::string::String {
        if let ::core::option::Option::Some(contract) = self.blackboard.problem_model.parent_contracts.first() {
            return contract.parent_id.clone();
        }
        "parent_upstream".to_string()
    }
}
