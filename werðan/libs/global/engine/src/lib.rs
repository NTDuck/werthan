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
pub use ::global_blackboard::{UnresolvedCrossLink, WorkManifest};
pub use self::error::MacroEngineError;
pub use self::guards::structural_guard;
pub use self::solver::{ModuleSolver, SolverOutcome, StubModuleSolver};
pub use ::global_workers::{
    AssessmentReport, ContextAssembler, DeterministicWorkers, EdgeStub, PhaseWorkOutput,
    PhaseWorker, RiskMatrix, TaskActivationEnvelope, VerifyVerdict, phase_role_name,
};

/// One human-in-the-loop decision gate kind from the HitlSection contract.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::cmp::PartialEq, ::core::cmp::Eq, ::core::cmp::Ord, ::core::cmp::PartialOrd)]
pub enum HitlGateKind {
    /// Human signoff required after S4 before S5.
    Reimagine,
    /// Human signoff required before rolling S5 back to S2.
    MacroRegression,
    /// Unresolved extrinsic conflict escalated to the human operator.
    ExtrinsicConflict,
}

impl ::core::fmt::Display for self::HitlGateKind {
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let label = match self {
            Self::Reimagine => "reimagine sign-off",
            Self::MacroRegression => "macro regression sign-off",
            Self::ExtrinsicConflict => "extrinsic conflict escalation",
        };
        ::core::write!(formatter, "{label}")
    }
}

/// Human-in-the-loop gate configuration mirrored from configschema HitlSection.
/// `full_auto` runs leave every gate disabled.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default)]
pub struct HitlGates {
    /// Pause before the S4 to S5 transition when true.
    pub gate_on_reimagine: ::core::primitive::bool,
    /// Pause before the S5 to S2 macro regression when true.
    pub gate_on_macro_regression: ::core::primitive::bool,
    /// Escalate unresolved extrinsic conflicts to the human when true.
    pub extrinsic_conflict_ask_human: ::core::primitive::bool,
}

/// Typed terminal outcome of one MacroEngine run invocation.
#[derive(::core::fmt::Debug)]
pub enum RunStop {
    /// The statechart reached Terminated.
    Terminated,
    /// The engine parked on a human-in-the-loop gate.
    /// The caller resolves the gate through approve_pending_gate and re-invokes run.
    HitlGatePending {
        /// Gate awaiting the operator decision.
        gate: self::HitlGateKind,
    },
}

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
    /// Batch keys already re-dispatched in the active stage-two remediation round.
    stage_two_remediations: ::core::option::Option<::std::collections::BTreeSet<::std::string::String>>,
    /// Human-in-the-loop gate configuration.
    hitl: self::HitlGates,
    /// Gate currently parked pending operator approval.
    pending_gate: ::core::option::Option<self::HitlGateKind>,
    /// Gate kinds already approved by the operator for this campaign.
    approved_gates: ::std::collections::BTreeSet<self::HitlGateKind>,
    /// Contract deltas awaiting hydration into the owning parent problem model.
    pending_parent_deltas:
        ::std::collections::BTreeMap<::std::string::String, ::std::vec::Vec<::std::string::String>>,
    /// Futile dispatch count per module (park back into the pre-dispatch state).
    futile_dispatches: ::std::collections::BTreeMap<::std::string::String, ::core::primitive::usize>,
    /// Cap on consecutive futile dispatches per module before termination.
    futile_dispatch_cap: ::core::primitive::usize,
    /// S0 preflight intent gate failure deferred until after the structural guard retry round.
    intent_gate_error: ::core::option::Option<self::MacroEngineError>,
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
        hitl: ::core::option::Option<self::HitlGates>,
        step_budget: ::core::option::Option<::core::primitive::usize>,
        futile_dispatch_cap: ::core::option::Option<::core::primitive::usize>,
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
        let futile_dispatch_cap = futile_dispatch_cap.unwrap_or(3);
        let hitl = hitl.unwrap_or_default();
        Self {
            board,
            statechart,
            solver,
            critic,
            arbiter,
            workers,
            warm_modes: ::std::collections::BTreeMap::new(),
            regression_scope: ::core::option::Option::None,
            stage_two_remediations: ::core::option::Option::None,
            hitl,
            pending_gate: ::core::option::Option::None,
            approved_gates: ::std::collections::BTreeSet::new(),
            pending_parent_deltas: ::std::collections::BTreeMap::new(),
            futile_dispatches: ::std::collections::BTreeMap::new(),
            futile_dispatch_cap,
            intent_gate_error: ::core::option::Option::None,
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
        self.futile_dispatches.clear();
    }

    /// Borrow the global blackboard.
    pub fn board(&self) -> &::global_blackboard::GlobalBlackboard {
        &self.board
    }

    /// Borrow the live state vector.
    pub fn state_vector(&self) -> &::global_blackboard::GlobalStateVector {
        &self.board.state
    }

    /// Return the macro statechart traversal history (VAL-MACRO-08/11/12).
    /// The history is the observable record of transitions, including
    /// regression revisits produced by Tier B/C remediation loops.
    pub fn macro_history(&self) -> &[::global_fsm::MacroState] {
        self.statechart.history()
    }

    /// Return total solver dispatch count.
    pub fn solver_dispatches(&self) -> ::core::primitive::usize {
        self.solver_dispatches
    }

    /// Return the macro loop step count consumed by the latest run.
    pub fn steps_used(&self) -> ::core::primitive::usize {
        self.steps_used
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

    /// Execute Algorithm 1 until Terminated, a human gate pause, or an error.
    /// Seeding happens only when no campaign is in flight (steps_used == 0).
    /// Callers resume a gate-paused run by approving the gate and calling run again.
    pub fn run(&mut self) -> ::core::result::Result<self::RunStop, self::MacroEngineError> {
        let campaign_fresh = self.steps_used == 0;
        if campaign_fresh {
            self.board.state.event_queue.push_back(::global_blackboard::MacroEvent::Start);
            self.statechart = ::global_fsm::MacroStatechart::new();
            self.board.state.macro_state = self.statechart.state();
        }
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
            let parked = self.pending_gate.clone();
            if let ::core::option::Option::Some(gate) = parked {
                return ::core::result::Result::Ok(self::RunStop::HitlGatePending { gate });
            }
            match self.statechart.state() {
                ::global_fsm::MacroState::Terminated => {
                    return ::core::result::Result::Ok(self::RunStop::Terminated);
                }
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

    /// Record operator approval for the pending human gate so the next run
    /// invocation resumes past it.
    pub fn approve_pending_gate(&mut self) {
        if let ::core::option::Option::Some(gate) = self.pending_gate.take() {
            self.approved_gates.insert(gate);
        }
    }

    /// Borrow the human gate parked pending operator approval.
    pub fn pending_gate(&self) -> ::core::option::Option<&self::HitlGateKind> {
        self.pending_gate.as_ref()
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
                    module_id,
                    cycle_edges,
                    proof_trace,
                } => {
                    self.handle_cycle_found(::core::option::Option::Some(module_id), cycle_edges, proof_trace)?;
                }
                ::global_blackboard::MacroEvent::TaskDone { .. } => {
                    // Zero-traffic acknowledgement: the worker committed its
                    // artifact before pushing this event (tech-spec 2.6).
                }
                ::global_blackboard::MacroEvent::ShardDone { .. } => {
                    // Scatter shard completion marker. The engine drains these
                    // without further action because shard outputs commit
                    // synchronously through dispatch_phase_worker.
                }
            }
            // A parked HITL gate must suspend the drain: the requeued event
            // sits at the queue front and reprocessing it without approval
            // would loop forever.
            let gate_parked = self.pending_gate.is_some();
            if gate_parked {
                break;
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
        // Real progress clears the futile-dispatch counter.
        self.futile_dispatches.remove(&module_id);
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
            // An unusable regression report must not orphan the child in an
            // M1/M2/M3 micro-state. compute_ready_set excludes those states,
            // so an early return here would deadlock the wave scheduler.
            // Park in M0_Blocked so the scheduler can re-ready the module.
            self.set_module_state(&module_id, ::global_blackboard::ModuleMicroState::M0Blocked);
            return;
        }
        let arbiter = &self.arbiter;
        let accepted = arbiter.adjudicate(&delta_spec, &proof_trace);
        if !accepted {
            // Unresolved conflict with ask-human escalation parks on the
            // extrinsic conflict gate. The operator resolves the contract
            // externally and approves. Subsequent reprocessing takes the
            // M0 parking path below without renegotiating.
            let escalation = self.hitl.extrinsic_conflict_ask_human;
            let already_approved = self
                .approved_gates
                .contains(&self::HitlGateKind::ExtrinsicConflict);
            if escalation && !already_approved {
                let conflict_event = ::global_blackboard::MacroEvent::ContractConflict {
                    parent_id,
                    module_id,
                    delta_spec,
                    proof_trace,
                };
                self.board.state.event_queue.push_front(conflict_event);
                self.pending_gate = ::core::option::Option::Some(self::HitlGateKind::ExtrinsicConflict);
            } else {
                // Rejected conflict without an ask-human gate: park the child
                // in M0_Blocked. Leaving it in M1/M2/M3 would deadlock the
                // wave scheduler because compute_ready_set skips those states.
                self.set_module_state(&module_id, ::global_blackboard::ModuleMicroState::M0Blocked);
            }
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
        // Canonical Algorithm 1 Tier B path: park the conflicting child in M0_Blocked.
        // Invalidate the parent's hardened state. Warm-start the parent's cluster with
        // the updated contract re-hydrated into its problem model.
        let parent_holds_artifact = self.is_module_hardened(&parent_id);
        let parent_known = self.board.dag.vertices.contains(&parent_id);
        if parent_known && parent_holds_artifact {
            self.set_module_state(&module_id, ::global_blackboard::ModuleMicroState::M0Blocked);
            self.set_module_state(&parent_id, ::global_blackboard::ModuleMicroState::M5Invalidated);
            self.warm_modes.insert(parent_id.clone(), ::holon_fsm::ActivationMode::WarmStartRegression);
            self.set_module_state(&parent_id, ::global_blackboard::ModuleMicroState::M2AriadneMcts);
            // Warm-start patch (METHODOLOGY-DRV.md Tier B step 3): append the
            // reported delta to the parent problem model's delta constraints.
            let parent_artifact_absent = !self.is_module_hardened(&parent_id);
            if parent_artifact_absent {
                let delta_text =
                    ::std::format!("parent {parent_id} contract delta from {module_id}: {delta_spec}");
                self.pending_parent_deltas
                    .entry(parent_id)
                    .or_default()
                    .push(delta_text);
            }
        } else {
            // No hardened parent artifact exists yet. The child simply waits in M0 for
            // the wave scheduler to re-ready it once the parent dispatches.
            self.set_module_state(&module_id, ::global_blackboard::ModuleMicroState::M0Blocked);
        }
    }

    /// Apply the Tier C cycle regression with deep history capture and SCC condensation.
    /// An invalid report must not orphan the reporting module in M1/M2/M3.
    /// Invalid means empty trace, non-traversable edges, or arbiter rejection.
    /// compute_ready_set skips M1/M2/M3 states. An orphaned reporter would
    /// therefore deadlock the wave scheduler. The reporter parks in M0_Blocked
    /// so the scheduler can re-ready it.
    fn handle_cycle_found(
        &mut self,
        module_id: ::core::option::Option<::std::string::String>,
        cycle_edges: ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
        proof_trace: ::std::string::String,
    ) -> ::core::result::Result<(), self::MacroEngineError> {
        // Legacy persisted events carry no reporter. Fall back to the sole
        // stalled-module inference used before the field existed.
        let reporter = module_id.filter(|id| self.board.dag.vertices.contains(id));
        let trace_empty = proof_trace.is_empty();
        if trace_empty {
            self.park_cycle_reporter(&reporter);
            return ::core::result::Result::Ok(());
        }
        let joined = cycle_edges
            .iter()
            .map(|edge_pair| ::std::format!("{}->{}", edge_pair.0, edge_pair.1))
            .collect::<::std::vec::Vec<_>>()
            .join(",");
        let subject = joined.as_str();
        // g_struct(E_cycle): every reported edge endpoint must be a known vertex, and
        // every edge must traverse the current DAG before the engine accepts the regression.
        let edges_traversable = cycle_edges.iter().all(|edge_pair| {
            let start_known = self.board.dag.vertices.contains(&edge_pair.0);
            let end_known = self.board.dag.vertices.contains(&edge_pair.1);
            start_known && end_known
        });
        if !edges_traversable {
            self.park_cycle_reporter(&reporter);
            return ::core::result::Result::Ok(());
        }
        let arbiter = &self.arbiter;
        let accepted = arbiter.adjudicate(subject, &proof_trace);
        if !accepted {
            self.park_cycle_reporter(&reporter);
            return ::core::result::Result::Ok(());
        }
        // Human-in-the-loop gate before rolling S5 back to S2. The cycle event
        // returns to the queue front so an approval reprocesses it.
        let gate_enabled = self.hitl.gate_on_macro_regression;
        let gate_approved = self
            .approved_gates
            .contains(&self::HitlGateKind::MacroRegression);
        if gate_enabled && !gate_approved {
            let cycle_event = ::global_blackboard::MacroEvent::CyclicDependencyFound {
                module_id: { reporter.unwrap_or_default() },
                cycle_edges,
                proof_trace,
            };
            self.board.state.event_queue.push_front(cycle_event);
            self.pending_gate = ::core::option::Option::Some(self::HitlGateKind::MacroRegression);
            return ::core::result::Result::Ok(());
        }
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

    /// Park the cycle reporter in M0_Blocked so the wave scheduler can re-ready
    /// it after an invalid/unaccepted regression report. Without this the
    /// reporter stalls in M1/M2/M3, which compute_ready_set never re-selects
    /// (pre-fd48337 WaveDeadlock root cause).
    ///
    /// A park is the futile-dispatch signal: the dispatch burned a full solver
    /// rollout budget and produced no progress. The counter increments here and
    /// resets on hardening. run_transform_phase terminates the campaign once a
    /// module exhausts the cap.
    fn park_cycle_reporter(&mut self, reporter: &::core::option::Option<::std::string::String>) {
        if let ::core::option::Option::Some(vertex_id) = reporter {
            let target_state = ::global_blackboard::ModuleMicroState::M0Blocked;
            self.set_module_state(vertex_id, target_state);
            let counter = self.futile_dispatches.entry(vertex_id.clone()).or_insert(0);
            *counter += 1;
        }
    }







    /// Run one bounded discovery phase with shard dispatch and two-stage guards.
    /// Structural failures re-dispatch only the offending shards under
    /// ActivationMode::CriticRetry until the guard passes (Algorithm 1 stage one).
    /// After one fruitless retry round the phase fails with StructuralGuardFailure.
    fn run_discovery_phase(&mut self, phase: ::global_fsm::MacroState) -> ::core::result::Result<(), self::MacroEngineError> {
        const MAX_GUARD_RETRIES: ::core::primitive::usize = 1;
        let shards = self::batches::partition_shards(&self.board, phase, &self.regression_scope);
        let shard_clones = shards.to_vec();
        for (shard_index, shard) in shard_clones.into_iter().enumerate() {
            self.dispatch_phase_worker(shard, shard_index, ::core::option::Option::None);
        }
        let mut retry_round = 0;
        loop {
            let failing = self::batches::guard_failing_shards(&self.board, phase, &shards);
            let guard_passed = failing.is_empty();
            if guard_passed {
                break;
            }
            if retry_round >= MAX_GUARD_RETRIES {
                let detail = self::guards::structural_guard(phase, &self.board)
                    .expect_err("guard reported failing shards without a detail");
                return ::core::result::Result::Err(
                    self::MacroEngineError::StructuralGuardFailure { phase, detail },
                );
            }
            retry_round += 1;
            self.structural_retries += 1;
            let delta = self::guards::structural_guard(phase, &self.board).err();
            for shard in failing {
                let mut shard_index = ::core::primitive::usize::default();
                for (probe, listed) in shards.iter().enumerate() {
                    if self::shards_match_subject(listed, &shard) {
                        shard_index = probe;
                        break;
                    }
                }
                self.dispatch_phase_worker(shard, shard_index, delta.clone());
            }
        }
        // A probe-reported INTENT violation binds after the guard round: the
        // deterministic guard cannot repair a missing harness or inventory.
        if let ::core::option::Option::Some(gate_failure) = self.intent_gate_error.take() {
            return ::core::result::Result::Err(gate_failure);
        }
        self.run_stage_two(phase)?;
        self.run_reduce_stage(phase);
        ::core::result::Result::Ok(())
    }


    /// Run the stage-two semantic critic batches and either remediate, regress, or advance.
    /// RemediateLocal verdicts re-dispatch only the offending batch's owning worker with
    /// the CritiqueRecord notes as the delta directive (Algorithm 1 stage two).
    fn run_stage_two(&mut self, phase: ::global_fsm::MacroState) -> ::core::result::Result<(), self::MacroEngineError> {
        // Scatter (Algorithm 1 S_m.1): persist the phase WorkManifest on G.
        let shards = self::batches::partition_shards(&self.board, phase, &self.regression_scope);
        self.commit_work_manifest(phase, &shards);
        let batches = self::batches::semantic_batches(&self.board, phase, &self.regression_scope);
        let mut remediated = self.stage_two_remediations.take().unwrap_or_default();
        let mut remediation_rounds = ::std::vec::Vec::new();
        for batch in batches {
            let record = self.critic.audit_batch(phase, &batch);
            match record.verdict {
                ::global_arbiter::CritiqueVerdict::RegressUpstream(_) => {
                    remediated.clear();
                    self.stage_two_remediations = ::core::option::Option::None;
                    let target = self::batches::regress_target(phase);
                    let current_chart = ::core::mem::take(&mut self.statechart);
                    let regressed = current_chart.regress_to(target)?;
                    self.statechart = regressed;
                    self.board.state.macro_state = self.statechart.state();
                    return ::core::result::Result::Ok(());
                }

                ::global_arbiter::CritiqueVerdict::RemediateLocal(_) => {
                    let batch_key = batch.join("\u{1f}");
                    let already = remediated.contains(&batch_key);
                    if already {
                        // The remediation dispatch did not flip the critic verdict.
                        // Surface the rejection instead of looping forever.
                        remediated.clear();
                        self.stage_two_remediations = ::core::option::Option::None;
                        return ::core::result::Result::Err(
                            self::MacroEngineError::SemanticRemediationFailed {
                                phase,
                                detail: ::core::clone::Clone::clone(&record.remediation_notes),
                            },
                        );
                    }
                    remediated.insert(batch_key);
                    self.structural_retries += 1;
                    remediation_rounds.push((batch, record.remediation_notes));
                }
                ::global_arbiter::CritiqueVerdict::Pass => {}
            }
        }
        for (batch, directive) in remediation_rounds {
            self.dispatch_shard_batch(phase, &batch, ::core::option::Option::Some(directive));
        }
        remediated.clear();
        self.stage_two_remediations = ::core::option::Option::None;
        self.advance_phase(phase)
    }

    /// Persist the Scatter WorkManifest for one phase onto the board.
    fn commit_work_manifest(
        &mut self,
        phase: ::global_fsm::MacroState,
        shards: &[self::batches::PhaseShard],
    ) {
        let shard_subjects = shards
            .iter()
            .map(|shard| match shard {
                self::batches::PhaseShard::Whole => "repository".to_string(),
                self::batches::PhaseShard::Vertex(vertex) => ::core::clone::Clone::clone(vertex),
                self::batches::PhaseShard::Edge(parent_id, child_id) => {
                    ::std::format!("{parent_id}->{child_id}")
                }
            })
            .collect::<::std::vec::Vec<_>>();
        self.board.work_manifests.push(self::WorkManifest {
            phase,
            shard_subjects,
        });
    }

    /// Re-dispatch the workers whose shard subjects appear in one semantic batch.
    fn dispatch_shard_batch(
        &mut self,
        phase: ::global_fsm::MacroState,
        batch: &[::std::string::String],
        directive: ::core::option::Option<::std::string::String>,
    ) {
        let shards = self::batches::partition_shards(&self.board, phase, &self.regression_scope);
        for (shard_index, shard) in shards.into_iter().enumerate() {
            let subject_owned = match &shard {
                self::batches::PhaseShard::Whole => "repository".to_string(),
                self::batches::PhaseShard::Vertex(vertex) => vertex.clone(),
                self::batches::PhaseShard::Edge(parent_id, child_id) => ::std::format!("{parent_id}->{child_id}"),
            };
            let batch_hit = batch
                .iter()
                .any(|item| subject_owned.contains(item.as_str()) || item.contains(&subject_owned));
            if batch_hit {
                self.dispatch_phase_worker(shard, shard_index, directive.clone());
            }
        }
    }

    /// Dispatch one phase shard worker and commit the typed output to the board.
    /// `shard_index` is the partition position of `shard` for the phase. The
    /// engine carries it on the `ev_ShardDone` completion event.
    fn dispatch_phase_worker(
        &mut self,
        shard: self::batches::PhaseShard,
        shard_index: ::core::primitive::usize,
        delta_directive: ::core::option::Option<::std::string::String>,
    ) {
        let subject = match &shard {
            self::batches::PhaseShard::Whole => "repository".to_string(),
            self::batches::PhaseShard::Vertex(vertex) => vertex.clone(),
            self::batches::PhaseShard::Edge(parent_id, child) => ::std::format!("{parent_id}->{child}"),
        };
        let phase = self.statechart.state();
        let role = ::global_workers::phase_role_name(phase);
        let retry_dispatch = delta_directive.is_some();
        let mode = if retry_dispatch {
            ::holon_fsm::ActivationMode::CriticRetry
        } else {
            ::holon_fsm::ActivationMode::Initial
        };
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
        // Completion signaling (VAL-FWM-02): the engine pushes the typed
        // ev_ShardDone event onto the event queue. No polling surface exists.
        // Agents drain events FIFO via pop_event.
        self.board
            .state
            .event_queue
            .push_back(::global_blackboard::MacroEvent::ShardDone { shard_index });
    }

    /// Run the Reduce stage (Algorithm 1 S_m.3) after one phase completes.
    /// After S3 the linker resolves every UnresolvedCrossLink tag by copying
    /// the referenced source rule card to the vertex whose rules name the
    /// target symbol. Resolved tags drop off the board.
    fn run_reduce_stage(&mut self, phase: ::global_fsm::MacroState) {
        let s3_complete = phase == ::global_fsm::MacroState::S3ExtractRules;
        if !s3_complete {
            return;
        }
        let tags = ::core::clone::Clone::clone(&self.board.cross_links);
        let mut remaining = ::std::vec::Vec::new();
        for tag in tags {
            let resolved = self.resolve_cross_link(&tag);
            if !resolved {
                remaining.push(tag);
            }
        }
        self.board.cross_links = remaining;
    }

    /// Resolve one cross-link tag by linking the referenced rule card.
    /// The tag resolves when a rule card whose clauses name the target symbol
    /// exists on a vertex. The card copies to every vertex the tag names.
    fn resolve_cross_link(&mut self, tag: &::global_blackboard::UnresolvedCrossLink) -> ::core::primitive::bool {
        let mut source_found = ::core::option::Option::None;
        for vertex in &self.board.dag.vertices {
            let cards_absent = !self.board.rules.contains(vertex);
            if cards_absent {
                continue;
            }
            let cards = match self.board.rules.get(vertex) {
                ::core::option::Option::Some(entries) => ::core::clone::Clone::clone(entries),
                ::core::option::Option::None => ::std::vec::Vec::new(),
            };
            let references_source = cards
                .iter()
                .any(|card| card.rule_id == tag.src_id);
            if references_source {
                source_found = ::core::option::Option::Some(vertex.clone());
                break;
            }
        }
        let source_vertex = match source_found {
            ::core::option::Option::Some(name) => name,
            ::core::option::Option::None => return false,
        };
        let source_cards = match self.board.rules.get(&source_vertex) {
            ::core::option::Option::Some(entries) => ::core::clone::Clone::clone(entries),
            ::core::option::Option::None => ::std::vec::Vec::new(),
        };
        let matching_card = source_cards
            .iter()
            .find(|card| card.rule_id == tag.src_id)
            .cloned();
        let card = match matching_card {
            ::core::option::Option::Some(entry) => entry,
            ::core::option::Option::None => return false,
        };
        // Copy the card to the vertex the tag names by its identifier.
        let mut linked_any = false;
        for vertex in &self.board.dag.vertices {
            let target_match = vertex == &tag.tgt_symbol;
            if !target_match {
                continue;
            }
            let self_link = *vertex == source_vertex;
            if self_link {
                continue;
            }
            let target_cards = match self.board.rules.get(vertex) {
                ::core::option::Option::Some(entries) => ::core::clone::Clone::clone(entries),
                ::core::option::Option::None => ::std::vec::Vec::new(),
            };
            let already_linked = target_cards.iter().any(|existing| existing.rule_id == card.rule_id);
            if already_linked {
                continue;
            }
            let card_copy = ::core::clone::Clone::clone(&card);
            let vertex_name = ::core::clone::Clone::clone(vertex);
            let mut merged = target_cards;
            merged.push(card_copy);
            self.board.rules.insert(vertex_name, merged);
            linked_any = true;
        }
        linked_any
    }

    /// Commit one typed worker output onto the global blackboard.
    fn commit_phase_output(&mut self, phase: ::global_fsm::MacroState, output: ::global_workers::PhaseWorkOutput) {
        match (phase, output) {
            (
                ::global_fsm::MacroState::S0IntentPreflight,
                ::global_workers::PhaseWorkOutput::Preflight {
                    intent_valid,
                    harness,
                },
            ) => {
                // INTENT structural gate (METHODOLOGY.md S0 guard): the probe
                // verdict binds. An invalid intent fails the phase outright.
                if !intent_valid {
                    let build_present = !harness.build_cmd.is_empty();
                    let test_present = !harness.test_cmd.is_empty();
                    let inventory_present = !self.board.inventory.is_empty();
                    let detail = if !inventory_present {
                        "intent: repository inventory is empty".to_string()
                    } else if !build_present && !test_present {
                        "intent: target build and test commands missing".to_string()
                    } else if !build_present {
                        "intent: target build command missing".to_string()
                    } else if !test_present {
                        "intent: target test command missing".to_string()
                    } else {
                        "intent: harness commands present but probe rejected intent".to_string()
                    };
                    self.intent_gate_error = ::core::option::Option::Some(
                        self::MacroEngineError::PreflightGateFailed { detail },
                    );
                }
            }
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
                ::global_workers::PhaseWorkOutput::Rules { cards, cross_links },
            ) => {
                for (vertex, card) in cards {
                    let mut merged = match self.board.rules.get(&vertex) {
                        ::core::option::Option::Some(existing) => ::core::clone::Clone::clone(existing),
                        ::core::option::Option::None => ::std::vec::Vec::new(),
                    };
                    merged.push(card);
                    self.board.rules.insert(vertex, merged);
                }
                for tag in cross_links {
                    let known = self
                        .board
                        .cross_links
                        .iter()
                        .any(|existing| existing.src_id == tag.src_id && existing.tgt_symbol == tag.tgt_symbol);
                    if known {
                        continue;
                    }
                    self.board.cross_links.push(tag);
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

    /// Advance to the next macro phase and run the S4 to S5 deep history restore.
    fn advance_phase(&mut self, _phase: ::global_fsm::MacroState) -> ::core::result::Result<(), self::MacroEngineError> {
        // Human-in-the-loop gate after S4 before entering S5 (reimagine sign-off).
        let leaving_s4 = self.statechart.state() == ::global_fsm::MacroState::S4BriefContracts;
        let gate_enabled = self.hitl.gate_on_reimagine;
        let gate_approved = self
            .approved_gates
            .contains(&self::HitlGateKind::Reimagine);
        if leaving_s4 && gate_enabled && !gate_approved {
            self.pending_gate = ::core::option::Option::Some(self::HitlGateKind::Reimagine);
            return ::core::result::Result::Ok(());
        }
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
        // Futile dispatch cap: a module parked back into the same unhardened
        // state it held before its last dispatch made no progress. Re-dispatch
        // burns a full solver rollout budget, so terminate once a module
        // exhausts the cap instead of cycling park, re-ready, re-dispatch.
        for vertex in &ready {
            let exhausted = self.futile_dispatches.get(vertex).copied().unwrap_or(0);
            if exhausted >= self.futile_dispatch_cap {
                return ::core::result::Result::Err(self::MacroEngineError::ModuleRetryExhausted {
                    module_id: { vertex.clone() },
                    attempts: { exhausted },
                });
            }
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

    /// Record one module micro-state change in the state vector and audit log.
    fn set_module_state(&mut self, module_id: &::core::primitive::str, next_state: ::global_blackboard::ModuleMicroState) {
        self.board.state.module_states.insert(module_id.to_string(), next_state);
        self.transition_log.push((module_id.to_string(), next_state));
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
            ::holon_fsm::ActivationMode::CriticRetry | ::holon_fsm::ActivationMode::WarmStartRegression => {
                ::global_blackboard::ModuleMicroState::M2AriadneMcts
            }
        };
        let model = self.hydrate_problem_model(module_id);
        self.board.state.module_states.insert(module_id.to_string(), next_state);
        self.transition_log.push((module_id.to_string(), next_state));

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
                self.board
                    .state
                    .module_states
                    .insert(module_id.to_string(), ::global_blackboard::ModuleMicroState::M3ContractNegotiate);
                self.transition_log.push((module_id.to_string(), ::global_blackboard::ModuleMicroState::M3ContractNegotiate));
                // A conflict dispatch burned a full solver rollout budget and
                // parks the dispatcher back into a re-dispatchable state. It
                // therefore counts toward the futile cap exactly like a parked
                // cycle reporter. Hardening clears the counter.
                let counter = self.futile_dispatches.entry(module_id.to_string()).or_insert(0);
                *counter += 1;
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
                // The engine invokes the solver per dispatch. The dispatcher's
                // module id therefore names the cycle reporter. The engine parks
                // that vertex when the report proves invalid, preventing a wave
                // scheduler deadlock.
                let cycle_event = ::global_blackboard::MacroEvent::CyclicDependencyFound {
                    module_id: { module_id.to_string() },
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
        if let ::core::option::Option::Some(deltas) = self.pending_parent_deltas.get(module_id) {
            for delta in deltas {
                model.add_delta_constraint(delta.clone());
            }
        }
        if let ::core::option::Option::Some(::core::option::Option::Some(artifact)) =
            self.board.artifacts.get(module_id)
        {
            model.legacy_source_slices.push(artifact.target_code.clone());
        }
        // The dialect profile seeds empty strings when the campaign sets no
        // language, so the rust default in the slice must survive.
        let dialect_language_empty = self.board.inventory.dialect.target_language.is_empty();
        if !dialect_language_empty {
            model.target_language =
                ::core::clone::Clone::clone(&self.board.inventory.dialect.target_language);
        }
        model
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

/// Report whether two shards address the same partition subject. Shard
/// variants do not derive equality because subjects may repeat across
/// partition regenerations. Subject identity is the partition key.
fn shards_match_subject(
    listed: &self::batches::PhaseShard,
    candidate: &self::batches::PhaseShard,
) -> ::core::primitive::bool {
    let listed_subject = match listed {
        self::batches::PhaseShard::Whole => "repository".to_string(),
        self::batches::PhaseShard::Vertex(vertex) => ::core::clone::Clone::clone(vertex),
        self::batches::PhaseShard::Edge(parent_id, child_id) => {
            ::std::format!("{parent_id}->{child_id}")
        }
    };
    let candidate_subject = match candidate {
        self::batches::PhaseShard::Whole => "repository".to_string(),
        self::batches::PhaseShard::Vertex(vertex) => ::core::clone::Clone::clone(vertex),
        self::batches::PhaseShard::Edge(parent_id, child_id) => {
            ::std::format!("{parent_id}->{child_id}")
        }
    };
    listed_subject == candidate_subject
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
    fn test_reimagine_gate_pauses_and_resume_completes() {
        let solver = self::StubModuleSolver::default();
        let boxed = ::std::boxed::Box::new(solver);
        let hitl = self::HitlGates {
            gate_on_reimagine: true,
            gate_on_macro_regression: false,
            extrinsic_conflict_ask_human: false,
        };
        let mut engine = self::MacroEngine::builder()
            .solver(boxed)
            .hitl(hitl)
            .build();
        let board = self::chain_board();
        engine.configure(board);
        let first = engine.run();
        match first {
            ::core::result::Result::Ok(self::RunStop::HitlGatePending { gate }) => {
                ::std::assert_eq!(gate, self::HitlGateKind::Reimagine);
            }
            other => ::std::panic!("expected reimagine gate pause, got {other:?}"),
        }
        ::std::assert!(engine.pending_gate().is_some());
        engine.approve_pending_gate();
        ::std::assert!(engine.pending_gate().is_none());
        let second = engine.run();
        ::std::assert!(::core::matches!(
            second,
            ::core::result::Result::Ok(self::RunStop::Terminated)
        ));
    }

    #[test]
    fn test_macro_regression_gate_pauses_and_reprocesses_cycle() {
        let solver = self::CycleThenHardenSolver {
            cycle_fired: false,
        };
        let boxed = ::std::boxed::Box::new(solver);
        let hitl = self::HitlGates {
            gate_on_reimagine: false,
            gate_on_macro_regression: true,
            extrinsic_conflict_ask_human: false,
        };
        let mut engine = self::MacroEngine::builder()
            .solver(boxed)
            .hitl(hitl)
            .build();
        let mut board = ::global_blackboard::GlobalBlackboard {
            inventory: ::std::vec!["a".to_string(), "b".to_string(), "d".to_string()].into(),
            ..::core::default::Default::default()
        };
        board.inventory.harness = ::global_blackboard::HarnessConfig {
            build_cmd: "cargo check".to_string(),
            test_cmd: "cargo test".to_string(),
            legacy_harness_cmd: "legacy test".to_string(),
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
        engine.configure(board);
        let first = engine.run();
        match first {
            ::core::result::Result::Ok(self::RunStop::HitlGatePending { gate }) => {
                ::std::assert_eq!(gate, self::HitlGateKind::MacroRegression);
            }
            other => ::std::panic!("expected macro regression gate pause, got {other:?}"),
        }
        // The cycle event returned to the queue so approval reprocesses it.
        let queued_cycle = engine
            .state_vector()
            .event_queue
            .iter()
            .any(|event| ::core::matches!(event, ::global_blackboard::MacroEvent::CyclicDependencyFound { .. }));
        ::std::assert!(queued_cycle);
        engine.approve_pending_gate();
        let second = engine.run();
        ::std::assert!(::core::matches!(
            second,
            ::core::result::Result::Ok(self::RunStop::Terminated)
        ));
        // The regression still ran after approval: the DAG condensed the cycle.
        let vertices = &engine.board().dag.vertices;
        ::std::assert!(vertices.contains(&"a+b".to_string()));
    }

    #[test]
    fn test_scatter_writes_work_manifest_per_phase() {
        let solver = self::StubModuleSolver::default();
        let boxed = ::std::boxed::Box::new(solver);
        let mut engine = self::chain_engine(boxed);
        let outcome = engine.run();
        ::std::assert!(outcome.is_ok());
        let manifests = &engine.board().work_manifests;
        ::std::assert!(manifests.len() >= 6);
        let s3_covered = manifests
            .iter()
            .any(|manifest| manifest.phase == ::global_fsm::MacroState::S3ExtractRules
                && manifest.covers("a")
                && manifest.covers("c"));
        ::std::assert!(s3_covered);
    }

    #[test]
    fn test_task_done_and_shard_done_events_drain_without_effect() {
        let solver = self::StubModuleSolver::default();
        let boxed = ::std::boxed::Box::new(solver);
        let mut engine = self::chain_engine(boxed);
        engine
            .board
            .state
            .push_event(::global_blackboard::MacroEvent::TaskDone {
                agent_id: "RuleMiner".to_string(),
                target_ref: "a".to_string(),
            });
        engine
            .board
            .state
            .push_event(::global_blackboard::MacroEvent::ShardDone { shard_index: 0 });
        let outcome = engine.run();
        ::std::assert!(outcome.is_ok());
        ::std::assert!(engine.state_vector().event_queue.is_empty());
        ::std::assert_eq!(
            engine.state_vector().macro_state,
            ::global_fsm::MacroState::Terminated
        );
    }

    #[test]
    fn test_cross_link_tag_resolves_by_copying_rule_card() {
        let solver = self::StubModuleSolver::default();
        let boxed = ::std::boxed::Box::new(solver);
        let mut engine = self::chain_engine(boxed);
        // A rule card on vertex a whose clauses name module b models the
        // cross-shard reference the tag reports.
        engine.board.rules.insert(
            "a",
            ::std::vec![::holon_blackboard::BusinessRuleCard {
                rule_id: "RA".to_string(),
                given: "module b exposes a ledger".to_string(),
                when: "module b commits an entry".to_string(),
                then: "module b preserves the entry order".to_string(),
                citation: "legacy/a.c:42".to_string(),
            }],
        );
        engine.board.cross_links.push(::global_blackboard::UnresolvedCrossLink {
            src_id: "RA".to_string(),
            tgt_symbol: "b".to_string(),
        });
        let outcome = engine.run();
        ::std::assert!(outcome.is_ok());
        // The linker copied RA onto the vertex whose rules name the target symbol.
        let linked = engine.board().rules.get("b").map(|cards| {
            cards
                .iter()
                .any(|card| card.rule_id == "RA")
        });
        ::std::assert_eq!(linked, ::core::option::Option::Some(true));
        // Resolved tags drop off the board.
        ::std::assert!(engine.board().cross_links.is_empty());
    }

    #[test]
    fn test_empty_harness_fails_preflight_intent_gate() {
        let solver = self::StubModuleSolver::default();
        let boxed = ::std::boxed::Box::new(solver);
        let mut engine = self::chain_engine(boxed);
        engine.board.inventory.harness = ::global_blackboard::HarnessConfig::default();
        let outcome = engine.run();
        let error = outcome.unwrap_err();
        ::std::assert!(::core::matches!(
            error,
            self::MacroEngineError::PreflightGateFailed { .. }
        ));
    }

    #[test]
    fn test_tier_b_delta_appends_to_parent_problem_model() {
        let solver = self::ConflictThenHardenSolver::new();
        let boxed = ::std::boxed::Box::new(solver);
        let mut engine = self::MacroEngine::new(boxed);
        let mut board = ::global_blackboard::GlobalBlackboard {
            inventory: ::std::vec!["a".to_string(), "b".to_string()].into(),
            ..::core::default::Default::default()
        };
        board.inventory.harness = ::global_blackboard::HarnessConfig {
            build_cmd: "cargo check".to_string(),
            test_cmd: "cargo test".to_string(),
            legacy_harness_cmd: "legacy test".to_string(),
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
        engine.configure(board);
        let outcome = engine.run();
        ::std::assert!(outcome.is_ok());
        // The parent warm-start re-dispatch hydrated the reported delta.
        let pending = engine.pending_parent_deltas.get("a");
        ::std::assert!(pending.is_some());
        let deltas = pending.unwrap();
        ::std::assert!(deltas
            .iter()
            .any(|delta| delta.contains("delta_from_b")));
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
        board.inventory.harness = ::global_blackboard::HarnessConfig {
            build_cmd: "cargo check".to_string(),
            test_cmd: "cargo test".to_string(),
            legacy_harness_cmd: "legacy test".to_string(),
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
        board.inventory.harness = ::global_blackboard::HarnessConfig {
            build_cmd: "cargo check".to_string(),
            test_cmd: "cargo test".to_string(),
            legacy_harness_cmd: "legacy test".to_string(),
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
            detail: "repository: inventory is empty".to_string(),
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
        board.inventory.harness = ::global_blackboard::HarnessConfig {
            build_cmd: "cargo check".to_string(),
            test_cmd: "cargo test".to_string(),
            legacy_harness_cmd: "legacy test".to_string(),
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
