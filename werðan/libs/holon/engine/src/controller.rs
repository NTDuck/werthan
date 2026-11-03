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

/// Target reward score signifying complete candidate hardening (compilation + 100% test pass rate + rules).
const HARDENED_REWARD_TARGET: ::core::primitive::f64 = 1.0;

/// Floating-point tolerance for verifying exact match with the hardened reward target.
const HARDENED_REWARD_EPSILON: ::core::primitive::f64 = 1e-6;

/// Threshold below which test pass rate triggers adversarial test expansion.
const ADVERSARIAL_PARITY_THRESHOLD: ::core::primitive::f64 = 0.999_999;

/// Maximum number of characterization tests allowed before suppressing further adversarial test expansion.
const MAX_EXPANDED_CHARACTERIZATION_TESTS: ::core::primitive::usize = 10;

/// Default prior utility assigned to fallback strategy hypotheses and repair nodes in the search tree.
const DEFAULT_PRIOR_UTILITY: ::core::primitive::f64 = 0.50;

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
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        formatter
            .debug_struct("ClusterEngine")
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
        let mut flat_count: ::core::primitive::usize = 0;
        let mut previous_best_reward = self.best_reward_score;
        let mut plateau_detector = crate::plateau::PlateauDetector::new();

        for _rollout_index in 0..budget {
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

            let total_tests = self.blackboard.characterization_tests.test_count();
            let last_pass_rate = self.last_pass_rate.unwrap_or(0.0);
            let passed_tests = (last_pass_rate * (total_tests as ::core::primitive::f64)).round()
                as ::core::primitive::usize;
            let passed_tests = passed_tests.min(total_tests);
            let uncovered_count = total_tests.saturating_sub(passed_tests);
            let coverage_sample = crate::plateau::CoverageSample {
                rollout: self.rollout_count,
                total_tests,
                passed_tests,
                uncovered_count,
            };
            plateau_detector.record(coverage_sample);

            let plateau_should_break = plateau_detector.is_plateau()
                && self.best_candidate_code.is_some()
                && self.best_reward_score >= 0.95;
            if plateau_should_break {
                ::tracing::warn!(
                    target: "holon_engine",
                    module_id = %self.blackboard.module_id,
                    summary = %plateau_detector.summary(),
                    "coverage plateau detected, breaking rollout loop early"
                );
                break;
            }

            if (self.best_reward_score - previous_best_reward).abs() < 1e-6 {
                flat_count += 1;
                let should_break = flat_count >= 16
                    && self.best_candidate_code.is_some()
                    && self.best_reward_score >= 0.95;
                if should_break {
                    ::tracing::warn!(
                        target: "holon_engine",
                        module_id = %self.blackboard.module_id,
                        flat_count,
                        "stagnation detected, breaking rollout loop early"
                    );
                    break;
                }
            } else {
                flat_count = 0;
                previous_best_reward = self.best_reward_score;
            }
        }

        let parented = !self.blackboard.problem_model.parent_contracts.is_empty();
        let empty_parent_id = ::std::string::String::new();
        let (blocked_on, trace) = if parented {
            let parent_id = self.resolve_parent_id();
            (
                parent_id.to_string(),
                ::std::format!(
                    "Rollout budget exhausted on module {} blocked on parent {}",
                    machine.module_id(),
                    parent_id,
                ),
            )
        } else {
            (
                empty_parent_id,
                ::std::format!(
                    "Rollout budget exhausted on parentless module {} with no contract counterparty",
                    machine.module_id(),
                ),
            )
        };
        crate::types::EngineTypestateOutcome::BudgetExhausted {
            machine,
            blocked_on_parent: { blocked_on },
            best_candidate: self.best_candidate_code.clone(),
            best_reward: self.best_reward_score,
            blocker_trace: { trace },
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
        ::tracing::info!(
            target: "holon_engine",
            "module {} rollout {}/{}: tests={}, last_pass_rate={:?}",
            self.blackboard.module_id,
            self.rollout_count,
            self.config.max_rollouts,
            self.blackboard.characterization_tests.test_count(),
            self.last_pass_rate,
        );

        // VAL-M2-03(d): demonstrated low parity invokes the adversarial test
        // agent to expand coverage. The agent stays blind to candidate code.
        // INITIAL-mode seeding already covers the empty-suite case.
        // Bound test expansion to prevent runaway growth during repeated repairs
        // (ADR-0018 intentional test suite compaction).
        let coverage_low = ::core::matches!(
            self.last_pass_rate,
            ::core::option::Option::Some(rate_value) if rate_value < ADVERSARIAL_PARITY_THRESHOLD
        ) && self.blackboard.characterization_tests.test_count() < MAX_EXPANDED_CHARACTERIZATION_TESTS;
        if coverage_low {
            let seeded_tests = ::holon_testgen::HolonTestGenAgent::generate_characterization_tests(
                &self.agents.test_gen_agent,
                &self.blackboard.problem_model,
            );
            for test_entry in seeded_tests {
                self.blackboard.characterization_tests.add_test_case(test_entry);
            }
        }

        // METHODOLOGY.md §4: Select the most promising expandable node in the
        // search tree using the UCB1 (Upper Confidence Bound 1) exploration policy.
        // UCB1 balances exploitation of high-reward candidate paths with exploration
        // of less-visited hypotheses based on prior utility and visit count.
        let expandable_id = self.tree.select_expandable(
            self.config.ucb_c_exp,
            self.config.gamma_prior,
            self.config.softmax_temperature,
        );
        ::tracing::info!(
            target: "holon_engine",
            module_id = %self.blackboard.module_id,
            node_id = expandable_id,
            "selected expandable MCTS node"
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
        ::tracing::info!(
            target: "holon_engine",
            "module {} rollout {} eval: compiles={}, pass_rate={}, reward={}",
            self.blackboard.module_id,
            self.rollout_count,
            eval_res.compiles,
            eval_res.test_pass_rate,
            reward,
        );
        if !eval_res.compiles {
            let diagnostic_snippet = eval_res
                .fault_record
                .as_ref()
                .map(|fault| fault.summary.as_str())
                .unwrap_or("no fault record");
            ::tracing::warn!(
                target: "holon_engine",
                "module {} rollout {} compilation failed: {}",
                self.blackboard.module_id,
                self.rollout_count,
                diagnostic_snippet,
            );
        }
        // Backpropagation loop: propagate the scalar evaluation reward back along
        // the search path from the newly expanded child node to the root.
        // This updates visit counts and cumulative reward estimates, steering subsequent
        // UCB1 selections toward the most promising transformation branches.
        self.tree.backpropagate(child_id, reward);
        if reward > self.best_reward_score || self.best_candidate_code.is_none() {
            self.best_reward_score = reward;
            self.best_candidate_code = ::core::option::Option::Some(candidate_code.clone());
        }

        if let ::core::option::Option::Some(fault) = &eval_res.fault_record {
            self.blackboard.patch_records.record_fault(fault.clone());

            // Extrinsic classifications are only actionable when a real upstream
            // counterparty exists. On a parentless module (single-file campaign)
            // resolve_parent_id() fabricates "parent_upstream". No parent can
            // resolve such a conflict: the module parks, the futile-dispatch
            // counter exhausts, and the campaign dies with
            // ModuleRetryExhausted. Treat such faults as intrinsic so the
            // rollout loop and repair agent keep running within this controller.
            let parentless_module = self.blackboard.problem_model.parent_contracts.is_empty();
            let extrinsic_contract = ::core::matches!(
                fault.cause,
                ::core::option::Option::Some(::holon_blackboard::FaultCause::ExtrinsicContract)
            );
            if extrinsic_contract && !parentless_module {
                let parent_id = self.resolve_parent_id().to_string();
                let delta_spec = fault.summary.clone();
                let proof_trace = fault.diagnostic_trace.clone();
                let res = self::StepResult::ContractConflict {
                    parent_id,
                    delta_spec,
                    proof_trace,
                };
                return ::core::option::Option::Some(res);
            }

            // A parentless module cannot prove an extrinsic cycle either: the
            // fabricated edge (parent_upstream, self) has no real parent vertex.
            let extrinsic_cycle = ::core::matches!(
                fault.cause,
                ::core::option::Option::Some(::holon_blackboard::FaultCause::ExtrinsicCycle)
            );
            if extrinsic_cycle && !parentless_module {
                let parent_id = self.resolve_parent_id().to_string();
                let reporter_id = self.blackboard.module_id.clone();
                let edge = (parent_id, reporter_id);
                let cycle_edges = ::std::vec![edge];
                let proof_trace = fault.diagnostic_trace.clone();
                let res = self::StepResult::CycleFound {
                    cycle_edges,
                    proof_trace,
                };
                return ::core::option::Option::Some(res);
            }
        }

        // Terminate early when reward reaches 1.0.
        // The candidate compiled and passed all tests.
        // Continued rollouts waste compute budget.
        if (reward - HARDENED_REWARD_TARGET).abs() < HARDENED_REWARD_EPSILON {
            ::tracing::info!(
                target: "holon_engine",
                module_id = %self.blackboard.module_id,
                rollout = self.rollout_count,
                reward,
                "candidate passed Gate 5 proof receipt - module hardened"
            );
            let res = self::StepResult::Hardened {
                target_code: { candidate_code },
                reward,
            };
            return ::core::option::Option::Some(res);
        }

        ::core::option::Option::None
    }

    /// Expand an MCTS search node by generating or repairing candidate code.
    ///
    /// For uninstantiated frontier nodes, the controller instantiates a strategy
    /// hypothesis and generates candidate code. For existing nodes, it repairs
    /// candidate code using the latest diagnostic fault record.
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

        let selected_code_opt = self
            .tree
            .get(selected_id)
            .and_then(|node| node.candidate_code.as_deref());
        ::tracing::info!(
            target: "holon_engine",
            module_id = %self.blackboard.module_id,
            node_id = selected_id,
            is_repair = selected_code_opt.is_some(),
            "expanding MCTS search node"
        );

        match selected_code_opt {
            ::core::option::Option::None => {
                // Frontier root expansion: instantiate an unvisited strategy hypothesis
                // and synthesize an initial candidate implementation from problem requirements.
                let hypothesis = match self.blackboard.strategy_hypotheses.next_uninstantiated() {
                    ::core::option::Option::Some(uninstantiated_hypothesis) => {
                        uninstantiated_hypothesis.instantiated = true;
                        uninstantiated_hypothesis.clone()
                    }
                    ::core::option::Option::None => ::holon_blackboard::StrategyHypothesis {
                        strategy_id: "default_strategy".to_string(),
                        description: "Direct transformation strategy".to_string(),
                        prior_utility: self::DEFAULT_PRIOR_UTILITY,
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
                // Targeted repair expansion: invoke the patch agent on the existing candidate
                // guided by the latest distilled diagnostic fault.
                let default_fault;
                let fault = match self.blackboard.patch_records.latest_fault() {
                    ::core::option::Option::Some(recorded_fault) => recorded_fault,
                    ::core::option::Option::None => {
                        default_fault = ::holon_blackboard::FaultRecord {
                            summary: "Suboptimal parity score".to_string(),
                            cause: ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic),
                            diagnostic_trace: ::std::string::String::new(),
                        };
                        &default_fault
                    }
                };

                let repaired_code = ::holon_patch::HolonPatchAgent::repair_code(
                    &self.agents.patch_agent,
                    existing_code,
                    fault,
                    &self.blackboard.problem_model,
                );

                let child_id = self.tree.add_child(
                    selected_id,
                    ::core::option::Option::Some(repaired_code.clone()),
                    ::core::option::Option::None,
                    DEFAULT_PRIOR_UTILITY,
                );
                (child_id, repaired_code)
            }
        }
    }

    fn resolve_parent_id(&self) -> &::core::primitive::str {
        self.blackboard
            .problem_model
            .parent_contracts
            .first()
            .map(|contract| contract.parent_id.as_str())
            .unwrap_or("parent_upstream")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Completion agent sampling temperature used across deterministic unit tests.
    const TEST_AGENT_TEMPERATURE: ::core::primitive::f64 = 0.2;

    /// Deterministic completion double for controller-level tests.
    #[derive(::core::clone::Clone)]
    struct ControllerFakeModel {
        response_text: ::std::string::String,
    }

    impl ::rig::completion::CompletionModel for self::ControllerFakeModel {
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

    /// Harness double that always reports a compile failure carrying the
    /// "interface mismatch" marker the default distiller maps to
    /// FaultCause::ExtrinsicContract.
    #[derive(::core::clone::Clone, ::core::fmt::Debug)]
    struct ContractConflictHarness;

    impl ::holon_evaluator::ExecutionHarness for self::ContractConflictHarness {
        fn execute_candidate(
            &self,
            _module_id: &::core::primitive::str,
            _candidate_code: &::core::primitive::str,
            _test_suite: &[::std::string::String],
            _target_language: &::core::primitive::str,
        ) -> ::holon_evaluator::HarnessExecutionOutput {
            let mismatch_logs = "error: interface mismatch against upstream export".to_string();
            ::holon_evaluator::HarnessExecutionOutput {
                compiles: false,
                pass_rate: 0.0,
                raw_logs: { mismatch_logs },
            }
        }
    }

    /// Distiller double emitting a constant ExtrinsicContract fault regardless
    /// of log content, isolating the controller gate from classifier heuristics.
    #[derive(::core::clone::Clone, ::core::fmt::Debug)]
    struct FixedContractDistiller;

    impl ::holon_evaluator::DiagnosticDistiller for self::FixedContractDistiller {
        fn distill_failure(
            &self,
            raw_logs: &::core::primitive::str,
            _candidate_code: &::core::primitive::str,
            _problem_model: &::holon_blackboard::ProblemModelSlice,
        ) -> ::holon_blackboard::FaultRecord {
            let cause_opt = ::core::option::Option::Some(::holon_blackboard::FaultCause::ExtrinsicContract);
            ::holon_blackboard::FaultRecord {
                summary: "contract conflict".to_string(),
                diagnostic_trace: ::std::string::ToString::to_string(raw_logs),
                cause: { cause_opt },
            }
        }

        fn audit_rules(
            &self,
            _candidate_code: &::core::primitive::str,
            _problem_model: &::holon_blackboard::ProblemModelSlice,
        ) -> ::core::primitive::f64 {
            0.0
        }
    }

    /// A fault classified as ExtrinsicContract on a parentless problem model
    /// (empty parent_contracts) must NOT yield StepResult::ContractConflict.
    /// The fabricated "parent_upstream" counterparty could never resolve it.
    /// The engine would park the module until dispatches exhaust. The step
    /// instead falls through to the intrinsic reward path (None).
    #[test]
    fn test_parentless_extrinsic_contract_fault_falls_through() {
        let fake_model = self::ControllerFakeModel {
            response_text: "pub fn factory_probe() -> u32 {\n    1\n}".to_string(),
        };
        let agents = self::build_test_pool(&fake_model);
        let empty_model = ::holon_blackboard::ProblemModelSlice::new();
        let mut controller = self::build_controller(agents, empty_model);
        ::std::assert!(controller.blackboard.problem_model.parent_contracts.is_empty());

        let step_res = controller.step_core();
        ::std::assert!(
            !::core::matches!(step_res, ::core::option::Option::Some(self::StepResult::ContractConflict { .. })),
            "parentless module must not produce StepResult::ContractConflict",
        );
    }

    /// With at least one real parent contract the existing behavior holds:
    /// the same ExtrinsicContract fault surfaces as StepResult::ContractConflict.
    #[test]
    fn test_parented_extrinsic_contract_fault_still_conflicts() {
        let fake_model = self::ControllerFakeModel {
            response_text: "pub fn factory_probe() -> u32 {\n    1\n}".to_string(),
        };
        let agents = self::build_test_pool(&fake_model);
        let mut model = ::holon_blackboard::ProblemModelSlice::new();
        let contract = ::holon_blackboard::EdgeContract {
            parent_id: "module_upstream".to_string(),
            interface_spec: "fn upstream_entry() -> u32".to_string(),
            preconditions: ::std::vec::Vec::new(),
        };
        model.add_parent_contract(contract);
        let mut controller = self::build_controller(agents, model);

        let step_res = controller.step_core();
        ::std::assert!(
            ::core::matches!(step_res, ::core::option::Option::Some(self::StepResult::ContractConflict { .. })),
            "parented module keeps the extrinsic contract conflict path",
        );
    }

    /// Distiller double emitting an Intrinsic fault, isolating the controller
    /// stagnation gate from extrinsic contract conflict handling on parented modules.
    #[derive(::core::clone::Clone, ::core::fmt::Debug)]
    struct FixedIntrinsicDistiller;

    impl ::holon_evaluator::DiagnosticDistiller for self::FixedIntrinsicDistiller {
        fn distill_failure(
            &self,
            raw_logs: &::core::primitive::str,
            _candidate_code: &::core::primitive::str,
            _problem_model: &::holon_blackboard::ProblemModelSlice,
        ) -> ::holon_blackboard::FaultRecord {
            let cause_opt = ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic);
            ::holon_blackboard::FaultRecord {
                summary: "intrinsic fault".to_string(),
                diagnostic_trace: ::std::string::ToString::to_string(raw_logs),
                cause: { cause_opt },
            }
        }

        fn audit_rules(
            &self,
            _candidate_code: &::core::primitive::str,
            _problem_model: &::holon_blackboard::ProblemModelSlice,
        ) -> ::core::primitive::f64 {
            0.0
        }
    }

    /// Suboptimal reward (< 0.95) must not exit rollouts early on stagnation.
    /// Parented modules with zero reward must exhaust the full rollout budget (16).
    #[test]
    fn test_parented_module_zero_reward_does_not_stagnate_early() {
        let fake_model = self::ControllerFakeModel {
            response_text: "pub fn factory_probe() -> u32 {\n    1\n}".to_string(),
        };
        let agents = self::build_test_pool(&fake_model);
        let mut model = ::holon_blackboard::ProblemModelSlice::new();
        let contract = ::holon_blackboard::EdgeContract {
            parent_id: "module_upstream".to_string(),
            interface_spec: "fn upstream_entry() -> u32".to_string(),
            preconditions: ::std::vec::Vec::new(),
        };
        model.add_parent_contract(contract);
        let mut controller = self::build_controller_with_distiller(agents, model, self::FixedIntrinsicDistiller);
        let blocked = ::holon_fsm::StateMachine::new("test_module".to_string());
        let machine = blocked.wake_ready().hydration_completed();

        let outcome = controller.solve(machine);
        ::std::assert!(
            ::core::matches!(outcome, crate::types::EngineTypestateOutcome::BudgetExhausted { .. }),
            "flat rollouts must yield BudgetExhausted outcome",
        );
        ::std::assert_eq!(
            controller.rollout_count, 16,
            "suboptimal reward must not exit early: budget must be exhausted (16), actual was {}",
            controller.rollout_count,
        );
        ::std::assert!(
            controller.best_candidate_code.is_some(),
            "best candidate code must be captured on initial rollout even at zero reward",
        );
    }

    /// Parentless modules with zero reward must not exit rollouts early.
    /// They must exhaust the full rollout budget (16).
    #[test]
    fn test_parentless_module_zero_reward_does_not_stagnate_early() {
        let fake_model = self::ControllerFakeModel {
            response_text: "pub fn factory_probe() -> u32 {\n    1\n}".to_string(),
        };
        let agents = self::build_test_pool(&fake_model);
        let empty_model = ::holon_blackboard::ProblemModelSlice::new();
        let mut controller = self::build_controller(agents, empty_model);
        let blocked = ::holon_fsm::StateMachine::new("test_module".to_string());
        let machine = blocked.wake_ready().hydration_completed();

        let outcome = controller.solve(machine);
        ::std::assert!(
            ::core::matches!(outcome, crate::types::EngineTypestateOutcome::BudgetExhausted { .. }),
            "flat rollouts must yield BudgetExhausted outcome",
        );
        ::std::assert_eq!(
            controller.rollout_count, 16,
            "parentless module with zero reward must exhaust full budget (16), actual was {}",
            controller.rollout_count,
        );
        ::std::assert!(
            controller.best_candidate_code.is_some(),
            "best candidate code must be captured on initial rollout even at zero reward",
        );
    }

    #[derive(::core::clone::Clone, ::core::fmt::Debug)]
    struct HighRewardHarness;

    impl ::holon_evaluator::ExecutionHarness for self::HighRewardHarness {
        fn execute_candidate(
            &self,
            _module_id: &::core::primitive::str,
            _candidate_code: &::core::primitive::str,
            _test_suite: &[::std::string::String],
            _target_language: &::core::primitive::str,
        ) -> ::holon_evaluator::HarnessExecutionOutput {
            ::holon_evaluator::HarnessExecutionOutput {
                compiles: true,
                pass_rate: 1.0,
                raw_logs: ::std::string::String::new(),
            }
        }
    }

    #[derive(::core::clone::Clone, ::core::fmt::Debug)]
    struct HighRewardDistiller;

    impl ::holon_evaluator::DiagnosticDistiller for self::HighRewardDistiller {
        fn distill_failure(
            &self,
            _raw_logs: &::core::primitive::str,
            _candidate_code: &::core::primitive::str,
            _problem_model: &::holon_blackboard::ProblemModelSlice,
        ) -> ::holon_blackboard::FaultRecord {
            ::holon_blackboard::FaultRecord {
                summary: "none".to_string(),
                diagnostic_trace: ::std::string::String::new(),
                cause: ::core::option::Option::None,
            }
        }

        fn audit_rules(
            &self,
            _candidate_code: &::core::primitive::str,
            _problem_model: &::holon_blackboard::ProblemModelSlice,
        ) -> ::core::primitive::f64 {
            // Reward: 0.20 (compile) + 0.60 * 1.0 (parity) + 0.20 * 0.8 (harden) = 0.96
            0.8
        }
    }

    /// High reward (>= 0.95) allows early exit when plateau occurs.
    #[test]
    fn test_early_exit_triggers_at_high_reward_plateau() {
        let fake_model = self::ControllerFakeModel {
            response_text: "pub fn factory_probe() -> u32 {\n    1\n}".to_string(),
        };
        let agents = self::build_test_pool(&fake_model);
        let model = ::holon_blackboard::ProblemModelSlice::new();
        let config = crate::types::EngineConfig {
            max_rollouts: 32,
            ..Default::default()
        };
        let evaluator = ::holon_evaluator::TwoStageEvaluator::builder()
            .harness(self::HighRewardHarness)
            .distiller(self::HighRewardDistiller)
            .build();
        let local_board = ::holon_blackboard::LocalBlackboard::new();
        let mut controller = self::ClusterEngine::builder()
            .config(config)
            .blackboard(local_board)
            .evaluator(evaluator)
            .agents(agents)
            .build();
        controller.blackboard_mut().problem_model = model;
        let blocked = ::holon_fsm::StateMachine::new("test_module".to_string());
        let machine = blocked.wake_ready().hydration_completed();

        let outcome = controller.solve(machine);
        ::std::assert!(
            ::core::matches!(outcome, crate::types::EngineTypestateOutcome::BudgetExhausted { .. }),
            "plateau rollouts must yield BudgetExhausted outcome",
        );
        ::std::assert_eq!(
            controller.rollout_count, 6,
            "plateau early-exit must terminate after 6 rollouts (window 5 + 1), actual was {}",
            controller.rollout_count,
        );
        ::std::assert!(controller.best_reward_score >= 0.95);
    }


    /// Build a subagent pool from one completion model for controller tests.
    fn build_test_pool(
        model: &self::ControllerFakeModel,
    ) -> crate::agents::SubagentPool<self::ControllerFakeModel> {
        let strategy_agent = ::holon_strategy::HolonStrategyAgent::builder()
            .model(model.clone())
            .temperature(TEST_AGENT_TEMPERATURE)
            .target_language("rust".to_string())
            .build();
        let codegen_agent = ::holon_codegen::HolonCodeGenAgent::builder()
            .model(model.clone())
            .temperature(TEST_AGENT_TEMPERATURE)
            .target_language("rust".to_string())
            .build();
        let testgen_agent = ::holon_testgen::HolonTestGenAgent::builder()
            .model(model.clone())
            .temperature(TEST_AGENT_TEMPERATURE)
            .target_language("rust".to_string())
            .build();
        let patch_agent = ::holon_patch::HolonPatchAgent::builder()
            .model(model.clone())
            .temperature(TEST_AGENT_TEMPERATURE)
            .target_language("rust".to_string())
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

    fn build_controller(
        agents: crate::agents::SubagentPool<self::ControllerFakeModel>,
        problem_model: ::holon_blackboard::ProblemModelSlice,
    ) -> self::ClusterEngine<
        self::ControllerFakeModel,
        self::ContractConflictHarness,
        self::FixedContractDistiller,
    > {
        self::build_controller_with_distiller(agents, problem_model, self::FixedContractDistiller)
    }

    fn build_controller_with_distiller<D>(
        agents: crate::agents::SubagentPool<self::ControllerFakeModel>,
        problem_model: ::holon_blackboard::ProblemModelSlice,
        distiller: D,
    ) -> self::ClusterEngine<
        self::ControllerFakeModel,
        self::ContractConflictHarness,
        D,
    >
    where
        D: ::holon_evaluator::DiagnosticDistiller + 'static,
    {
        let harness_value = self::ContractConflictHarness;
        let evaluator = ::holon_evaluator::TwoStageEvaluator::builder()
            .harness(harness_value)
            .distiller(distiller)
            .build();
        let engine_config = crate::types::EngineConfig::default();
        let local_board = ::holon_blackboard::LocalBlackboard::new();
        let mut controller = self::ClusterEngine::builder()
            .config(engine_config)
            .blackboard(local_board)
            .evaluator(evaluator)
            .agents(agents)
            .build();
        controller.blackboard_mut().problem_model = problem_model;
        controller
    }
}
