//! Deterministic macro-orchestrator engine implementing Algorithm 1 in METHODOLOGY.md.

pub mod batches;
pub mod error;
pub mod guards;
pub mod solver;

pub use ::global_arbiter::{
    Arbiter, CritiqueRecord, CritiqueVerdict, DeterministicArbiter, DeterministicCritic,
    SemanticCritic, trace_references_parties,
};
pub use ::global_fsm::{
    MacroState, MacroStateError, MacroStateMachine, MacroStatechart,
};
pub use self::batches::{PhaseShard, regress_target, semantic_batches};
pub use self::error::MacroEngineError;
pub use self::guards::structural_guard;
pub use self::solver::{ModuleSolver, SolverOutcome, StubModuleSolver};
pub use ::global_workers::{
    AssessmentReport, ContextAssembler, DeterministicWorkers, EdgeStub, PhaseWorkOutput,
    PhaseWorker, RiskMatrix, TaskActivationEnvelope, VerifyVerdict, phase_role_name,
};

/// Deterministic macro-orchestrator engine implementing Algorithm 1 in METHODOLOGY.md.
pub struct MacroEngine {
    board: ::global_blackboard::GlobalBlackboard,
    statechart: ::global_fsm::MacroStatechart,
    solver: ::std::boxed::Box<dyn self::ModuleSolver>,
    critic: ::std::boxed::Box<dyn ::global_arbiter::SemanticCritic>,
    arbiter: ::std::boxed::Box<dyn ::global_arbiter::Arbiter>,
    workers: ::std::boxed::Box<dyn ::global_workers::PhaseWorker>,
    warm_modes: ::std::collections::BTreeMap<::std::string::String, ::holon_fsm::ActivationMode>,
    regression_scope: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
    step_budget: ::core::primitive::usize,
    steps_used: ::core::primitive::usize,
    solver_dispatches: ::core::primitive::usize,
    worker_executions: ::core::primitive::usize,
    structural_retries: ::core::primitive::usize,
    transition_log: ::std::vec::Vec<(::std::string::String, ::global_blackboard::ModuleMicroState)>,
}

impl self::MacroEngine {
    /// Construct an engine over an empty board with deterministic guards and the given solver.
    pub fn new(solver: ::std::boxed::Box<dyn self::ModuleSolver>) -> Self {
        Self::builder().solver(solver).build()
    }
}

#[::bon::bon]
impl self::MacroEngine {
    /// Construct a MacroEngine using a bon builder.
    #[builder]
    pub fn builder(
        board: ::core::option::Option<::global_blackboard::GlobalBlackboard>,
        statechart: ::core::option::Option<::global_fsm::MacroStatechart>,
        solver: ::core::option::Option<::std::boxed::Box<dyn self::ModuleSolver>>,
        critic: ::core::option::Option<::std::boxed::Box<dyn ::global_arbiter::SemanticCritic>>,
        arbiter: ::core::option::Option<::std::boxed::Box<dyn ::global_arbiter::Arbiter>>,
        workers: ::core::option::Option<::std::boxed::Box<dyn ::global_workers::PhaseWorker>>,
        step_budget: ::core::option::Option<::core::primitive::usize>,
    ) -> Self {
        let board = board.unwrap_or_default();
        let statechart = statechart.unwrap_or_default();
        let solver = match solver {
            ::core::option::Option::Some(configured) => configured,
            ::core::option::Option::None => {
                let default_solver = self::StubModuleSolver::new();
                ::std::boxed::Box::new(default_solver)
            }
        };
        let critic = match critic {
            ::core::option::Option::Some(configured) => configured,
            ::core::option::Option::None => {
                let default_critic = ::global_arbiter::DeterministicCritic::new();
                ::std::boxed::Box::new(default_critic)
            }
        };
        let arbiter = match arbiter {
            ::core::option::Option::Some(configured) => configured,
            ::core::option::Option::None => {
                let default_arbiter = ::global_arbiter::DeterministicArbiter::new();
                ::std::boxed::Box::new(default_arbiter)
            }
        };
        let workers = match workers {
            ::core::option::Option::Some(configured) => configured,
            ::core::option::Option::None => {
                let default_workers = ::global_workers::DeterministicWorkers::new();
                ::std::boxed::Box::new(default_workers)
            }
        };
        let step_budget = step_budget.unwrap_or(256);
        Self {
            board,
            statechart,
            solver,
            critic,
            arbiter,
            workers,
            warm_modes: ::std::collections::BTreeMap::new(),
            regression_scope: ::core::option::Option::None,
            step_budget,
            steps_used: 0,
            solver_dispatches: 0,
            worker_executions: 0,
            structural_retries: 0,
            transition_log: ::std::vec::Vec::new(),
        }
    }
    /// Seed the board inventory, DAG, and module vector before running.
    pub fn configure(&mut self, board: ::global_blackboard::GlobalBlackboard) {
        self.board = board;
        self.board.state.module_states.clear();
        for vertex in self.board.dag.vertices.clone() {
            self.board.state.module_states.insert(vertex, ::global_blackboard::ModuleMicroState::M0Blocked);
        }
    }

    /// Borrow the global blackboard.
    pub fn board(&self) -> &::global_blackboard::GlobalBlackboard {
        &self.board
    }

    /// Borrow the live state vector.
    pub fn state_vector(&self) -> &::global_blackboard::GlobalStateVector {
        &self.board.state
    }

    /// Return total solver dispatch count.
    pub fn solver_dispatches(&self) -> ::core::primitive::usize {
        self.solver_dispatches
    }

    /// Return total worker execution count across phase shards.
    pub fn worker_executions(&self) -> ::core::primitive::usize {
        self.worker_executions
    }

    /// Return total structural retry count.
    pub fn structural_retries(&self) -> ::core::primitive::usize {
        self.structural_retries
    }

    /// Borrow the module micro-state transition audit log.
    pub fn transition_log(&self) -> &[(::std::string::String, ::global_blackboard::ModuleMicroState)] {
        &self.transition_log
    }

    /// Execute Algorithm 1 until the Terminated state or an error occurs.
    pub fn run(&mut self) -> ::core::result::Result<(), self::MacroEngineError> {
        self.board.state.event_queue.push_back(::global_blackboard::MacroEvent::Start);
        self.statechart = ::global_fsm::MacroStatechart::new();
        self.board.state.macro_state = self.statechart.state();
        loop {
            self.steps_used += 1;
            ::tracing::info!("MacroEngine::run loop: step {}, state {:?}", self.steps_used, self.statechart.state());
            let budget_spent = self.steps_used > self.step_budget;
            if budget_spent {
                return ::core::result::Result::Err(self::MacroEngineError::StepBudgetExhausted {
                    steps_used: self.steps_used,
                });
            }
            self.process_event_queue()?;
            match self.statechart.state() {
                ::global_fsm::MacroState::Terminated => return ::core::result::Result::Ok(()),
                ::global_fsm::MacroState::S0IntentPreflight
                | ::global_fsm::MacroState::S1Assess
                | ::global_fsm::MacroState::S2MapDag
                | ::global_fsm::MacroState::S3ExtractRules
                | ::global_fsm::MacroState::S4BriefContracts
                | ::global_fsm::MacroState::S6SystemVerify => {
                    let phase = self.statechart.state();
                    self.run_discovery_phase(phase)?;
                }
                ::global_fsm::MacroState::S5IterativeDag => {
                    self.run_transform_phase()?;
                }
            }
        }
    }

    /// Drain and apply the pending events on the global event bus.
    fn process_event_queue(&mut self) -> ::core::result::Result<(), self::MacroEngineError> {
        loop {
            let next = self.board.state.event_queue.pop_front();
            let event = match next {
                ::core::option::Option::Some(pending) => pending,
                ::core::option::Option::None => break,
            };
            match event {
                ::global_blackboard::MacroEvent::Start => {}
                ::global_blackboard::MacroEvent::ModuleHardened {
                    module_id,
                    target_code,
                    test_suite,
                } => {
                    self.handle_module_hardened(module_id, target_code, test_suite)?;
                }
                ::global_blackboard::MacroEvent::ContractConflict {
                    parent_id,
                    module_id,
                    delta_spec,
                    proof_trace,
                } => {
                    self.handle_contract_conflict(parent_id, module_id, delta_spec, proof_trace);
                }
                ::global_blackboard::MacroEvent::CyclicDependencyFound {
                    cycle_edges,
                    proof_trace,
                } => {
                    self.handle_cycle_found(cycle_edges, proof_trace)?;
                }
            }
        }
        ::core::result::Result::Ok(())
    }

    /// Commit a hardened artifact and mark module micro-state as hardened.
    fn handle_module_hardened(
        &mut self,
        module_id: ::std::string::String,
        target_code: ::std::string::String,
        test_suite: ::std::vec::Vec<::std::string::String>,
    ) -> ::core::result::Result<(), self::MacroEngineError> {
        let artifact = ::global_blackboard::ModuleArtifact {
            target_code,
            test_suite,
        };
        self.board.artifacts.insert(module_id.clone(), ::core::option::Option::Some(artifact));
        self.set_module_state(&module_id, ::global_blackboard::ModuleMicroState::M4Hardened);
        ::core::result::Result::Ok(())
    }

    /// Apply the Tier B contract conflict regression on the wave scheduler.
    fn handle_contract_conflict(
        &mut self,
        parent_id: ::std::string::String,
        module_id: ::std::string::String,
        delta_spec: ::std::string::String,
        proof_trace: ::std::string::String,
    ) {
        let structurally_sound =
            ::global_arbiter::trace_references_parties(&proof_trace, &parent_id, &module_id);
        if !structurally_sound {
            return;
        }
        let arbiter = &self.arbiter;
        let accepted = arbiter.adjudicate(&delta_spec, &proof_trace);
        if !accepted {
            return;
        }
        let key = (parent_id.clone(), module_id.clone());
        if let ::core::option::Option::Some(contract) = self.board.contracts.get_mut(&key) {
            contract.preconditions.push(delta_spec.clone());
        } else {
            let edge_contract = ::holon_blackboard::EdgeContract {
                parent_id: ::core::clone::Clone::clone(&parent_id),
                interface_spec: ::std::string::String::new(),
                preconditions: ::std::vec![delta_spec.clone()],
            };
            self.board.contracts.insert(key, edge_contract);
        }
        let parent_is_compilable = self.board.dag.vertices.contains(&parent_id)
            && !parent_id.ends_with(".h");
        if parent_is_compilable && self.is_module_hardened(&parent_id) {
            self.set_module_state(&module_id, ::global_blackboard::ModuleMicroState::M0Blocked);
            self.set_module_state(&parent_id, ::global_blackboard::ModuleMicroState::M5Invalidated);
            self.warm_modes.insert(parent_id.clone(), ::holon_fsm::ActivationMode::WarmStartRegression);
            self.set_module_state(&parent_id, ::global_blackboard::ModuleMicroState::M2AriadneMcts);
        } else {
            self.warm_modes.insert(module_id.clone(), ::holon_fsm::ActivationMode::WarmStartRegression);
            self.set_module_state(&module_id, ::global_blackboard::ModuleMicroState::M2AriadneMcts);
        }
    }

    /// Apply the Tier C cycle regression with deep history capture and SCC condensation.
    fn handle_cycle_found(
        &mut self,
        cycle_edges: ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
        proof_trace: ::std::string::String,
    ) -> ::core::result::Result<(), self::MacroEngineError> {
        let trace_empty = proof_trace.is_empty();
        if trace_empty {
            return ::core::result::Result::Ok(());
        }
        let joined = cycle_edges
            .iter()
            .map(|edge_pair| ::std::format!("{}->{}", edge_pair.0, edge_pair.1))
            .collect::<::std::vec::Vec<_>>()
            .join(",");
        let subject = joined.as_str();
        let arbiter = &self.arbiter;
        let accepted = arbiter.adjudicate(subject, &proof_trace);
        if !accepted {
            return ::core::result::Result::Ok(());
        }
        // Modules caught mid-synthesis by the macro regression park in M5 so the
        // next wave re-dispatches them instead of deadlocking on M1 occupancy.
        // The conversion precedes the snapshot so Theta_H* holds parked states.
        let stalled = self
            .board
            .state
            .module_states
            .iter()
            .filter(|(_, state)| {
                ::core::matches!(
                    state,
                    ::global_blackboard::ModuleMicroState::M1ContextHydration
                        | ::global_blackboard::ModuleMicroState::M2AriadneMcts
                        | ::global_blackboard::ModuleMicroState::M3ContractNegotiate
                )
            })
            .map(|(vertex, _)| vertex.clone())
            .collect::<::std::vec::Vec<_>>();
        for vertex in stalled {
            self.set_module_state(&vertex, ::global_blackboard::ModuleMicroState::M5Invalidated);
        }
        // Save deep history Theta_H* before leaving the composite S5 state.
        self.board.state.save_deep_history();
        // Union the reported cycle edges into the DAG before condensation.
        for edge_pair in &cycle_edges {
            let known = self.board.dag.edges.contains(edge_pair);
            if !known {
                self.board.dag.edges.push(edge_pair.clone());
            }
        }
        let condensed = ::global_blackboard::scc::tarjan_condense(&self.board.dag);
        self.apply_condensation(condensed);
        let merged_scope = self.merged_scope_vertices();
        self.regression_scope = ::core::option::Option::Some(merged_scope);
        let current_chart = ::core::mem::take(&mut self.statechart);
        let regressed = current_chart.regress_to_s2()?;
        self.statechart = regressed;
        self.board.state.macro_state = self.statechart.state();
        ::core::result::Result::Ok(())
    }







    /// Run one bounded discovery phase with shard dispatch and two-stage guards.
    /// On a structural guard failure the retry dispatch carries the guard detail
    /// as the delta directive per the CRITIC_RETRY protocol.
    fn run_discovery_phase(&mut self, phase: ::global_fsm::MacroState) -> ::core::result::Result<(), self::MacroEngineError> {
        let shards = self::batches::partition_shards(&self.board, phase, &self.regression_scope);
        for shard in shards {
            self.dispatch_phase_worker(shard, ::core::option::Option::None);
        }
        let detail = self::guards::structural_guard(phase, &self.board);
        let guard_failed = detail.is_err();
        if guard_failed {
            // One CRITIC_RETRY re-dispatch round per Algorithm 1 guard protocol.
            self.structural_retries += 1;
            let delta = detail.err();
            let shards = self::batches::partition_shards(&self.board, phase, &self.regression_scope);
            for shard in shards {
                self.dispatch_phase_worker(shard, delta.clone());
            }
            let retried = self::guards::structural_guard(phase, &self.board);
            match retried {
                ::core::result::Result::Ok(()) => {
                    // The retry dispatch repaired the board state. Continue to stage two.
                    self.run_stage_two(phase)?;
                    return ::core::result::Result::Ok(());
                }
                ::core::result::Result::Err(failed) => {
                    return ::core::result::Result::Err(
                        self::MacroEngineError::StructuralGuardFailure {
                            phase,
                            detail: { failed },
                        },
                    );
                }
            }
        }
        self.run_stage_two(phase)?;
        ::core::result::Result::Ok(())
    }

    /// Dispatch one phase shard worker and commit the typed output to the board.
    fn dispatch_phase_worker(
        &mut self,
        shard: self::batches::PhaseShard,
        delta_directive: ::core::option::Option<::std::string::String>,
    ) {
        let subject = match &shard {
            self::batches::PhaseShard::Whole => "repository".to_string(),
            self::batches::PhaseShard::Vertex(vertex) => vertex.clone(),
            self::batches::PhaseShard::Edge(parent_id, child) => ::std::format!("{parent_id}->{child}"),
        };
        let phase = self.statechart.state();
        let role = ::global_workers::phase_role_name(phase);
        let mode = ::holon_fsm::ActivationMode::Initial;
        let envelope = ::global_workers::ContextAssembler::hydrate(
            role,
            mode,
            &subject,
            &self.board,
            delta_directive,
        );
        self.worker_executions += 1;
        let output = self.workers.execute(phase, &subject, &envelope, &self.board);
        self.commit_phase_output(phase, output);
    }

    /// Commit one typed worker output onto the global blackboard.
    fn commit_phase_output(&mut self, phase: ::global_fsm::MacroState, output: ::global_workers::PhaseWorkOutput) {
        match (phase, output) {
            (
                ::global_fsm::MacroState::S1Assess,
                ::global_workers::PhaseWorkOutput::Assessment { synthesized, .. },
            ) => {
                let _ = synthesized;
            }
            (
                ::global_fsm::MacroState::S2MapDag,
                ::global_workers::PhaseWorkOutput::Coupling { discovered_edges },
            ) => {
                for edge_pair in &discovered_edges {
                    let known = self.board.dag.edges.contains(edge_pair);
                    if !known {
                        self.board.dag.edges.push(edge_pair.clone());
                    }
                }
            }
            (
                ::global_fsm::MacroState::S3ExtractRules,
                ::global_workers::PhaseWorkOutput::Rules { cards, .. },
            ) => {
                for (vertex, card) in cards {
                    let mut merged = match self.board.rules.get(&vertex) {
                        ::core::option::Option::Some(existing) => ::core::clone::Clone::clone(existing),
                        ::core::option::Option::None => ::std::vec::Vec::new(),
                    };
                    merged.push(card);
                    self.board.rules.insert(vertex, merged);
                }
            }
            (
                ::global_fsm::MacroState::S4BriefContracts,
                ::global_workers::PhaseWorkOutput::Contracts { idioms, stubs },
            ) => {
                let _ = idioms;
                for stub in stubs {
                    let parent_id = ::core::clone::Clone::clone(&stub.parent_id);
                    let child_id = ::core::clone::Clone::clone(&stub.child_id);
                    let edge_key = (parent_id, child_id);
                    let absent = !self.board.contracts.contains_edge(&edge_key);
                    if absent {
                        self.board.contracts.insert(edge_key, stub.contract);
                    }
                }
            }
            (
                ::global_fsm::MacroState::S6SystemVerify,
                ::global_workers::PhaseWorkOutput::Verification { verdicts, .. },
            ) => {
                for verdict in verdicts {
                    let verdict_green = verdict.tests_pass && verdict.security_findings.is_empty();
                    let already_hardened = ::core::matches!(
                        self.board.state.module_states.get(&verdict.module_id),
                        ::core::option::Option::Some(::global_blackboard::ModuleMicroState::M4Hardened)
                    );
                    if verdict_green && !already_hardened {
                        self.set_module_state(
                            &verdict.module_id,
                            ::global_blackboard::ModuleMicroState::M4Hardened,
                        );
                    }
                }
            }
            _ => {}
        }
    }

    /// Run the stage-two semantic critic batches and either regress or advance the phase.
    fn run_stage_two(&mut self, phase: ::global_fsm::MacroState) -> ::core::result::Result<(), self::MacroEngineError> {
        let batches = self::batches::semantic_batches(&self.board, phase, &self.regression_scope);
        for batch in batches {
            let record = self.critic.audit_batch(phase, &batch);
            if let ::global_arbiter::CritiqueVerdict::RegressUpstream(_) = record.verdict {
                let target = self::batches::regress_target(phase);
                let current_chart = ::core::mem::take(&mut self.statechart);
                let regressed = current_chart.regress_to(target)?;
                self.statechart = regressed;
                self.board.state.macro_state = self.statechart.state();
                return ::core::result::Result::Ok(());
            }
        }
        self.advance_phase(phase)
    }

    /// Advance to the next macro phase and run the S4 to S5 deep history restore.
    fn advance_phase(&mut self, _phase: ::global_fsm::MacroState) -> ::core::result::Result<(), self::MacroEngineError> {
        let current_chart = ::core::mem::take(&mut self.statechart);
        let advanced = current_chart.advance()?;
        self.statechart = advanced;
        self.board.state.macro_state = self.statechart.state();
        let entered_s5 = self.board.state.macro_state == ::global_fsm::MacroState::S5IterativeDag;
        if entered_s5 {
            self.restore_history_into_module_states();
            self.regression_scope = ::core::option::Option::None;
        }
        ::core::result::Result::Ok(())
    }

    /// Restore module vector entries from Theta_H* when entering S5 per Algorithm 1 step 6.
    fn restore_history_into_module_states(&mut self) {
        for vertex in self.board.dag.vertices.clone() {
            let restored = self.board.state.history_state_of(&vertex);
            let next_state = match restored {
                ::core::option::Option::Some(checkpoint) => checkpoint,
                ::core::option::Option::None => ::global_blackboard::ModuleMicroState::M0Blocked,
            };
            self.board.state.module_states.insert(vertex.clone(), next_state);
            self.transition_log
                .push((vertex, next_state));
        }
    }

    /// Run the S5 composite transform state with Kahn topological wave dispatch.
    fn run_transform_phase(&mut self) -> ::core::result::Result<(), self::MacroEngineError> {
        if self.all_vertices_hardened() {
            let current_chart = ::core::mem::take(&mut self.statechart);
            let advanced = current_chart.advance()?;
            self.statechart = advanced;
            self.board.state.macro_state = self.statechart.state();
            return ::core::result::Result::Ok(());
        }
        let ready = self.compute_ready_set();
        let ready_empty = ready.is_empty();
        if ready_empty {
            let detail = "no ready module while unhardened modules remain".to_string();
            return ::core::result::Result::Err(self::MacroEngineError::WaveDeadlock { detail });
        }
        for vertex in ready {
            self.dispatch_module(&vertex);
        }
        ::core::result::Result::Ok(())
    }

    /// Report whether every DAG vertex holds a hardened artifact and module state.
    fn all_vertices_hardened(&self) -> ::core::primitive::bool {
        for vertex in &self.board.dag.vertices {
            let artifact_present = ::core::matches!(
                self.board.artifacts.get(vertex),
                ::core::option::Option::Some(::core::option::Option::Some(_))
            );
            let state_present = ::core::matches!(
                self.board.state.module_states.get(vertex),
                ::core::option::Option::Some(::global_blackboard::ModuleMicroState::M4Hardened)
            );
            let hardened = artifact_present && state_present;
            if !hardened {
                return false;
            }
        }
        true
    }

    /// Compute the Kahn ready set of modules with all parents hardened.
    fn compute_ready_set(&self) -> ::std::vec::Vec<::std::string::String> {
        let mut ready = ::std::vec::Vec::new();
        for vertex in &self.board.dag.vertices {
            let now = *self.board.state.module_states.get(vertex).unwrap_or(&::global_blackboard::ModuleMicroState::M0Blocked);
            // Warm-start modules in M2 re-enter dispatch without a parent gate because
            // they hardened once before the downstream conflict invalidated them.
            let warm_pending = self.warm_modes.contains_key(vertex);
            let parked_eligible = now == ::global_blackboard::ModuleMicroState::M0Blocked
                || now == ::global_blackboard::ModuleMicroState::M5Invalidated;
            if warm_pending {
                ready.push(vertex.clone());
                continue;
            }
            if !parked_eligible {
                continue;
            }
            let mut parents_ready = true;
            for parent in self.board.dag.parents_of(vertex) {
                let parent_ok = self.is_module_hardened(&parent);
                if !parent_ok {
                    parents_ready = false;
                }
            }
            if parents_ready {
                ready.push(vertex.clone());
            }
        }
        ready
    }

    /// Report whether one module sits in the hardened micro-state.
    fn is_module_hardened(&self, module_id: &::core::primitive::str) -> ::core::primitive::bool {
        ::core::matches!(
            self.board.state.module_states.get(module_id),
            ::core::option::Option::Some(::global_blackboard::ModuleMicroState::M4Hardened)
        )
    }

    /// Dispatch the solver for one ready module and drive its micro-state transitions.
    fn dispatch_module(&mut self, module_id: &::core::primitive::str) {
        let mode = match self.warm_modes.remove(module_id) {
            ::core::option::Option::Some(saved) => saved,
            ::core::option::Option::None => ::holon_fsm::ActivationMode::Initial,
        };
        let next_state = match mode {
            ::holon_fsm::ActivationMode::Initial => ::global_blackboard::ModuleMicroState::M1ContextHydration,
            ::holon_fsm::ActivationMode::WarmStartRegression => ::global_blackboard::ModuleMicroState::M2AriadneMcts,
        };
        let model = self.hydrate_problem_model(module_id);
        self.set_module_state(module_id, next_state);

        self.solver_dispatches += 1;
        let outcome = self.solver.solve(module_id, &model, mode);
        match outcome {
            self::solver::SolverOutcome::Hardened {
                target_code,
                test_suite,
            } => {
                let module_event = ::global_blackboard::MacroEvent::ModuleHardened {
                    module_id: { module_id.to_string() },
                    target_code: { target_code },
                    test_suite: { test_suite },
                };
                self.board.state.event_queue.push_back(module_event);
            }
            self::solver::SolverOutcome::ContractConflict {
                parent_id,
                delta_spec,
                proof_trace,
            } => {
                self.set_module_state(module_id, ::global_blackboard::ModuleMicroState::M3ContractNegotiate);
                let conflict_event = ::global_blackboard::MacroEvent::ContractConflict {
                    parent_id,
                    module_id: { module_id.to_string() },
                    delta_spec,
                    proof_trace,
                };
                self.board.state.event_queue.push_back(conflict_event);
            }
            self::solver::SolverOutcome::CycleFound {
                cycle_edges,
                proof_trace,
            } => {
                let cycle_event = ::global_blackboard::MacroEvent::CyclicDependencyFound {
                    cycle_edges,
                    proof_trace,
                };
                self.board.state.event_queue.push_back(cycle_event);
            }
        }
    }

    /// Hydrate one module problem model from rules and parent contracts on the board.
    fn hydrate_problem_model(&self, module_id: &::core::primitive::str) -> ::holon_blackboard::ProblemModelSlice {
        let mut model = ::holon_blackboard::ProblemModelSlice::new();
        if let ::core::option::Option::Some(rules) = self.board.rules.get(module_id) {
            for card in rules {
                model.add_rule(card.clone());
            }
        }
        for parent in self.board.dag.parents_of(module_id) {
            let key = (parent.clone(), module_id.to_string());
            if let ::core::option::Option::Some(contract) = self.board.contracts.get(&key) {
                model.add_parent_contract(contract.clone());
            }
        }
        if let ::core::option::Option::Some(::core::option::Option::Some(artifact)) =
            self.board.artifacts.get(module_id)
        {
            model.legacy_source_slices.push(artifact.target_code.clone());
        }
        model
    }

    /// Record one module micro-state change in the state vector and audit log.
    fn set_module_state(&mut self, module_id: &::core::primitive::str, next_state: ::global_blackboard::ModuleMicroState) {
        self.board.state.module_states.insert(module_id.to_string(), next_state);
        self.transition_log.push((module_id.to_string(), next_state));
    }

    /// Re-key rules, contracts, and artifacts onto the condensed DAG.
    fn apply_condensation(&mut self, condensed: ::global_blackboard::CondensedDag) {
        let mut new_rules = ::std::collections::BTreeMap::new();
        let mut new_artifacts = ::std::collections::BTreeMap::new();
        for vertex in &condensed.vertices {
            let mut union_rules = ::std::vec::Vec::new();
            for member in vertex.split('+') {
                if let ::core::option::Option::Some(member_rules) = self.board.rules.get(member) {
                    union_rules.extend(member_rules.clone());
                }
            }
            new_rules.insert(vertex.clone(), union_rules);
            let mut merged_artifact = ::core::option::Option::None;
            for member in vertex.split('+') {
                if let ::core::option::Option::Some(::core::option::Option::Some(existing)) =
                    self.board.artifacts.get(member)
                {
                    merged_artifact = ::core::option::Option::Some(existing.clone());
                }
            }
            new_artifacts.insert(vertex.clone(), merged_artifact);
        }
        self.board.rules = ::global_blackboard::RuleRegistry::from(new_rules);
        self.board.artifacts = ::global_blackboard::ArtifactRegistry::from(new_artifacts);
        // Drop contracts internal to a merged vertex and re-key cross edges.
        let mut new_contracts = ::std::collections::BTreeMap::new();
        for (key, contract) in self.board.contracts.contracts.clone() {
            let owner_parent = self.merged_owner(&condensed, &key.0);
            let owner_child = self.merged_owner(&condensed, &key.1);
            let owner_match = owner_parent == owner_child;
            if owner_match {
                continue;
            }
            let parent = match owner_parent {
                ::core::option::Option::Some(name) => name,
                ::core::option::Option::None => key.0.clone(),
            };
            let child = match owner_child {
                ::core::option::Option::Some(name) => name,
                ::core::option::Option::None => key.1.clone(),
            };
            new_contracts.insert((parent, child), contract);
        }
        self.board.contracts = ::global_blackboard::ContractRegistry::from(new_contracts);
        self.board.dag = condensed;
        // Rebuild module vector: unknown merged vertices park in M0, known states carry over.
        let mut new_states = ::std::collections::BTreeMap::new();
        for vertex in &self.board.dag.vertices {
            let mut carried = ::core::option::Option::None;
            for member in vertex.split('+') {
                if let ::core::option::Option::Some(existing) = self.board.state.module_states.get(member) {
                    carried = ::core::option::Option::Some(*existing);
                }
            }
            let next_state = match carried {
                ::core::option::Option::Some(existing) => existing,
                ::core::option::Option::None => ::global_blackboard::ModuleMicroState::M0Blocked,
            };
            new_states.insert(vertex.clone(), next_state);
        }
        self.board.state.module_states = new_states;
    }

    /// Return the merged vertex owning a pre-condensation id when one exists.
    fn merged_owner(&self, condensed: &::global_blackboard::CondensedDag, member: &::core::primitive::str) -> ::core::option::Option<::std::string::String> {
        for vertex in &condensed.vertices {
            let parts = vertex.split('+');
            for part in parts {
                let member_match = part == member;
                if member_match {
                    return ::core::option::Option::Some(vertex.clone());
                }
            }
        }
        ::core::option::Option::None
    }

    /// Return the condensed vertices that still require a rework dispatch.
    fn merged_scope_vertices(&self) -> ::std::vec::Vec<::std::string::String> {
        let mut scope = ::std::vec::Vec::new();
        for vertex in &self.board.dag.vertices {
            let merged = vertex.contains('+');
            if merged {
                scope.push(vertex.clone());
            }
        }
        scope
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_engine_builder() {
        let engine = self::MacroEngine::builder().build();
        ::std::assert_eq!(engine.step_budget, 256);
        ::std::assert_eq!(engine.solver_dispatches(), 0);
    }

    #[test]
    fn test_happy_path_s0_through_s6() {
        let solver = self::StubModuleSolver::default();
        let boxed = ::std::boxed::Box::new(solver);
        let mut engine = self::chain_engine(boxed);
        let outcome = engine.run();
        ::std::assert!(outcome.is_ok());
        ::std::assert_eq!(
            engine.state_vector().macro_state,
            ::global_fsm::MacroState::Terminated
        );
        for vertex in ["a", "b", "c"] {
            let artifact = engine.board().artifacts.get(vertex);
            ::std::assert!(::core::matches!(artifact, ::core::option::Option::Some(::core::option::Option::Some(_))));
            let module_state = engine.state_vector().module_states.get(vertex);
            ::std::assert_eq!(
                module_state,
                ::core::option::Option::Some(&::global_blackboard::ModuleMicroState::M4Hardened)
            );
        }
    }

    /// Counting worker crew delegating to the deterministic defaults.
    #[derive(::core::fmt::Debug)]
    struct CountingWorkers {
        inner: ::global_workers::DeterministicWorkers,
        executions: ::std::sync::Arc<::std::sync::atomic::AtomicUsize>,
    }

    impl ::global_workers::PhaseWorker for self::CountingWorkers {
        fn execute(
            &self,
            phase: ::global_fsm::MacroState,
            shard_subject: &::core::primitive::str,
            envelope: &::global_workers::TaskActivationEnvelope,
            board: &::global_blackboard::GlobalBlackboard,
        ) -> ::global_workers::PhaseWorkOutput {
            self.executions
                .fetch_add(1, ::std::sync::atomic::Ordering::SeqCst);
            self.inner.execute(phase, shard_subject, envelope, board)
        }
    }

    #[test]
    fn test_worker_executions_counter_tracks_dispatches() {
        let zero = ::std::sync::atomic::AtomicUsize::new(0);
        let counter = ::std::sync::Arc::new(zero);
        let inner = ::global_workers::DeterministicWorkers::new();
        let executions = ::core::clone::Clone::clone(&counter);
        let workers = CountingWorkers { inner, executions };
        let crew = ::std::boxed::Box::new(workers);
        let stub_solver = self::StubModuleSolver::default();
        let solver = ::std::boxed::Box::new(stub_solver);
        let mut engine = self::MacroEngine::builder()
            .solver(solver)
            .workers(crew)
            .build();
        let board = self::chain_board();
        engine.configure(board);
        let outcome = engine.run();
        ::std::assert!(outcome.is_ok());
        let counted = counter.load(::std::sync::atomic::Ordering::SeqCst);
        ::std::assert!(counted > 0);
        ::std::assert_eq!(engine.worker_executions(), counted);
    }

    #[test]
    fn test_s4_workers_stub_fill_missing_contracts() {
        let stub_solver = self::StubModuleSolver::default();
        let solver = ::std::boxed::Box::new(stub_solver);
        let mut engine = self::MacroEngine::new(solver);
        let mut board = self::chain_board();
        board.contracts = ::global_blackboard::ContractRegistry::new();
        engine.configure(board);
        let outcome = engine.run();
        ::std::assert!(outcome.is_ok());
        for edge_pair in engine.board().dag.edges.clone() {
            let contract = engine.board().contracts.get(&edge_pair);
            ::std::assert!(
                ::core::matches!(contract, ::core::option::Option::Some(_)),
                "edge {edge_pair:?} lacks a worker-stubbed contract"
            );
        }
    }

    /// Stub solver that fails one contract conflict on first dispatch of module b.
    #[derive(::core::fmt::Debug)]
    struct ConflictThenHardenSolver {
        conflicts_fired: ::core::primitive::usize,
    }

    impl self::ConflictThenHardenSolver {
        fn new() -> Self {
            Self {
                conflicts_fired: 0,
            }
        }
    }

    impl self::ModuleSolver for self::ConflictThenHardenSolver {
        fn solve(
            &mut self,
            module_id: &::core::primitive::str,
            problem_model: &::holon_blackboard::ProblemModelSlice,
            mode: ::holon_fsm::ActivationMode,
        ) -> self::SolverOutcome {
            let module_is_b = module_id == "b";
            let warm_mode = ::core::matches!(mode, ::holon_fsm::ActivationMode::WarmStartRegression);
            let hydrated = !problem_model.parent_contracts.is_empty();
            let fires = module_is_b && !warm_mode && hydrated && self.conflicts_fired == 0;
            if fires {
                self.conflicts_fired += 1;
                return self::SolverOutcome::ContractConflict {
                    parent_id: "a".to_string(),
                    delta_spec: "delta_from_b".to_string(),
                    proof_trace: "trace names b and parent a".to_string(),
                };
            }
            let mut fallback = self::StubModuleSolver::default();
            fallback.solve(module_id, problem_model, mode)
        }
    }

    #[test]
    fn test_tier_b_contract_conflict_regression() {
        // Two vertices keep the trace small. Vertex b reports a conflict on parent a.
        let mut board = ::global_blackboard::GlobalBlackboard {
            inventory: ::std::vec!["a".to_string(), "b".to_string()].into(),
            ..::core::default::Default::default()
        };
        board.dag.vertices = ::std::vec!["a".to_string(), "b".to_string()];
        board.dag.edges = ::std::vec![("a".to_string(), "b".to_string())];
        board.contracts.insert(
            ("a".to_string(), "b".to_string()),
            ::holon_blackboard::EdgeContract {
                parent_id: "a".to_string(),
                interface_spec: "iface".to_string(),
                preconditions: ::std::vec::Vec::new(),
            },
        );
        let solver = self::ConflictThenHardenSolver::new();
        let boxed = ::std::boxed::Box::new(solver);
        let mut engine = self::MacroEngine::new(boxed);
        engine.configure(board);
        let outcome = engine.run();
        ::std::assert!(outcome.is_ok());
        ::std::assert_eq!(
            engine.state_vector().macro_state,
            ::global_fsm::MacroState::Terminated
        );
        // The parent passed through M5 Invalidated and back to M2 for warm start.
        let log = engine.transition_log();
        let invalidation = log.iter().position(|entry| entry.0 == "a" && entry.1 == ::global_blackboard::ModuleMicroState::M5Invalidated);
        let warm = log.iter().position(|entry| entry.0 == "a" && entry.1 == ::global_blackboard::ModuleMicroState::M2AriadneMcts);
        ::std::assert!(invalidation.is_some());
        ::std::assert!(warm.is_some());
        ::std::assert!(warm.unwrap() > invalidation.unwrap());
        // The contract on the board gained the reported delta precondition.
        let contract = engine.board().contracts.get(&("a".to_string(), "b".to_string()));
        let has_delta = match contract {
            ::core::option::Option::Some(entry) => entry.preconditions.contains(&"delta_from_b".to_string()),
            ::core::option::Option::None => false,
        };
        ::std::assert!(has_delta);
        // Both vertices hardened despite the regression.
        ::std::assert_eq!(
            engine.state_vector().module_states.get("a"),
            ::core::option::Option::Some(&::global_blackboard::ModuleMicroState::M4Hardened)
        );
        ::std::assert_eq!(
            engine.state_vector().module_states.get("b"),
            ::core::option::Option::Some(&::global_blackboard::ModuleMicroState::M4Hardened)
        );
    }

    /// Stub solver that reports one undocumented cycle from module b.
    #[derive(::core::fmt::Debug)]
    struct CycleThenHardenSolver {
        cycle_fired: ::core::primitive::bool,
    }

    impl self::ModuleSolver for self::CycleThenHardenSolver {
        fn solve(
            &mut self,
            module_id: &::core::primitive::str,
            problem_model: &::holon_blackboard::ProblemModelSlice,
            mode: ::holon_fsm::ActivationMode,
        ) -> self::SolverOutcome {
            let module_is_b = module_id == "b";
            let hydrated = !problem_model.parent_contracts.is_empty();
            if module_is_b && hydrated && !self.cycle_fired {
                self.cycle_fired = true;
                return self::SolverOutcome::CycleFound {
                    cycle_edges: ::std::vec![("b".to_string(), "a".to_string())],
                    proof_trace: "undocumented cycle between a and b".to_string(),
                };
            }
            let mut engine_solver = self::StubModuleSolver::default();
            engine_solver.solve(module_id, problem_model, mode)
        }
    }

    #[test]
    fn test_tier_c_cycle_regression_with_deep_history() {
        let mut board = ::global_blackboard::GlobalBlackboard {
            inventory: ::std::vec!["a".to_string(), "b".to_string(), "d".to_string()].into(),
            ..::core::default::Default::default()
        };
        board.dag.vertices = ::std::vec!["a".to_string(), "b".to_string(), "d".to_string()];
        board.dag.edges = ::std::vec![("a".to_string(), "b".to_string())];
        board.contracts.insert(
            ("a".to_string(), "b".to_string()),
            ::holon_blackboard::EdgeContract {
                parent_id: "a".to_string(),
                interface_spec: "iface".to_string(),
                preconditions: ::std::vec::Vec::new(),
            },
        );
        let solver = self::CycleThenHardenSolver {
            cycle_fired: false,
        };
        let boxed = ::std::boxed::Box::new(solver);
        let mut engine = self::MacroEngine::new(boxed);
        engine.configure(board);
        let outcome = engine.run();
        ::std::assert!(outcome.is_ok());
        // Deep history captured at the regression boundary.
        ::std::assert!(engine.state_vector().deep_history.is_some());
        let history = engine.state_vector().deep_history.as_ref().unwrap();
        ::std::assert_eq!(
            history.get("d"),
            ::core::option::Option::Some(&::global_blackboard::ModuleMicroState::M4Hardened)
        );
        // The DAG condensed the two-cycle into one joined vertex.
        let vertices = &engine.board().dag.vertices;
        ::std::assert!(vertices.contains(&"a+b".to_string()));
        ::std::assert!(vertices.contains(&"d".to_string()));
        ::std::assert!(engine.board().dag.is_acyclic());
        // The unaffected hardened module kept its checkpointed state without re-solve churn.
        ::std::assert_eq!(
            engine.state_vector().module_states.get("d"),
            ::core::option::Option::Some(&::global_blackboard::ModuleMicroState::M4Hardened)
        );
        ::std::assert_eq!(
            engine.state_vector().macro_state,
            ::global_fsm::MacroState::Terminated
        );
    }

    #[test]
    fn test_structural_guard_failure_after_single_retry() {
        // An empty inventory fails the S0 guard even after the retry dispatch.
        let solver = self::StubModuleSolver::default();
        let boxed = ::std::boxed::Box::new(solver);
        let mut engine = self::MacroEngine::new(boxed);
        let empty_board = ::global_blackboard::GlobalBlackboard::default();
        engine.configure(empty_board);
        let outcome = engine.run();
        let error = outcome.unwrap_err();
        let expected = self::MacroEngineError::StructuralGuardFailure {
            phase: ::global_fsm::MacroState::S0IntentPreflight,
            detail: "inventory is empty".to_string(),
        };
        ::std::assert_eq!(error, expected);
        ::std::assert_eq!(engine.structural_retries(), 1);
    }

    #[test]
    fn test_step_budget_guards_against_livelock() {
        // A solver that always reports a cycle forces repeated Tier C regressions.
        let solver = self::AlwaysCycleSolver;
        let boxed = ::std::boxed::Box::new(solver);
        let mut engine = self::chain_engine(boxed);
        let outcome = engine.run();
        let error = outcome.unwrap_err();
        ::std::assert!(::core::matches!(error, self::MacroEngineError::StepBudgetExhausted { .. }));
    }

    /// Stub solver that always reports an undocumented cycle for every dispatch.
    #[derive(::core::fmt::Debug)]
    struct AlwaysCycleSolver;

    impl self::ModuleSolver for self::AlwaysCycleSolver {
        fn solve(
            &mut self,
            _module_id: &::core::primitive::str,
            _problem_model: &::holon_blackboard::ProblemModelSlice,
            _mode: ::holon_fsm::ActivationMode,
        ) -> self::SolverOutcome {
            self::SolverOutcome::CycleFound {
                cycle_edges: ::std::vec![("a".to_string(), "b".to_string())],
                proof_trace: "cycle".to_string(),
            }
        }
    }

    /// Build a deterministic stub solver engine with a three vertex chain.
    /// Vertices a, b, c with edges a to b and b to c. Only c depends on b, b on a.
    fn chain_engine(solver: ::std::boxed::Box<dyn ModuleSolver>) -> self::MacroEngine {
        let board = self::chain_board();
        let mut engine = self::MacroEngine::new(solver);
        engine.configure(board);
        engine
    }

    /// Build the three vertex chain board shared by engine tests.
    fn chain_board() -> ::global_blackboard::GlobalBlackboard {
        let mut board = ::global_blackboard::GlobalBlackboard {
            inventory: ::std::vec!["a".to_string(), "b".to_string(), "c".to_string()].into(),
            ..::core::default::Default::default()
        };
        board.dag.vertices = ::std::vec!["a".to_string(), "b".to_string(), "c".to_string()];
        board.dag.edges = ::std::vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
        ];
        board.rules.insert("a".to_string(), ::std::vec![rule_card("RA")]);
        board.rules.insert("b".to_string(), ::std::vec![rule_card("RB")]);
        board.rules.insert("c".to_string(), ::std::vec![rule_card("RC")]);
        board.contracts.insert(
            ("a".to_string(), "b".to_string()),
            ::holon_blackboard::EdgeContract {
                parent_id: "a".to_string(),
                interface_spec: "iface_a".to_string(),
                preconditions: ::std::vec::Vec::new(),
            },
        );
        board.contracts.insert(
            ("b".to_string(), "c".to_string()),
            ::holon_blackboard::EdgeContract {
                parent_id: "b".to_string(),
                interface_spec: "iface_b".to_string(),
                preconditions: ::std::vec::Vec::new(),
            },
        );
        board
    }

    /// Build a rule card with a cited legacy source line.
    fn rule_card(rule_id: &::core::primitive::str) -> ::holon_blackboard::BusinessRuleCard {
        ::holon_blackboard::BusinessRuleCard {
            rule_id: { rule_id.to_string() },
            given: "given clause".to_string(),
            when: "when clause".to_string(),
            then: "then clause".to_string(),
            citation: ::std::format!("legacy/{}.c:42", rule_id),
        }
    }
}
