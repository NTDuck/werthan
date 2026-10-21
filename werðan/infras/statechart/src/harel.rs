//! Harel Statechart Engine implementing XOR macro-states, AND orthogonal regions,
//! Kahn's topological wave scheduling, Deep History (H*), and broadcast calculus.
//! Formalized per §2 Definition 3, §4, §5, and Algorithm 1 in `METHODOLOGY.md`.


/// Top-level sequential XOR Macro-States (S0..S6).
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::hash::Hash,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum HarelMacroState {
    /// S0: Initial intent framing, baseline compilation, preflight verification.
    S0IntentPreflight,
    /// S1: Quantitative repository assessment, file inventory coverage, dialect profiling.
    S1Assess,
    /// S2: Dependency call-graph mapping and Tarjan SCC condensation into DAG G=(V,E).
    S2Map,
    /// S3: Global pre-DAG Given/When/Then business rule extraction.
    S3ExtractBusinessRules,
    /// S4: Target architecture contract stubs and interface brief.
    S4BriefReimagineContracts,
    /// S5: Iterative DAG transformation over orthogonal module regions R(v_i).
    S5IterativeDagTransform,
    /// S6: Final system integration testing and security hardening.
    S6SystemVerifyHarden,
    /// Workflow completed successfully.
    Terminated,
}

impl ::core::fmt::Display for self::HarelMacroState {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let label = match self {
            Self::S0IntentPreflight => "S0: Intent & Preflight",
            Self::S1Assess => "S1: Assess",
            Self::S2Map => "S2: Map (DAG Construction)",
            Self::S3ExtractBusinessRules => "S3: Extract Business Rules",
            Self::S4BriefReimagineContracts => "S4: Brief & Reimagine Contracts",
            Self::S5IterativeDagTransform => "S5: Iterative DAG Transform",
            Self::S6SystemVerifyHarden => "S6: System Verify & Harden",
            Self::Terminated => "Terminated",
        };
        ::core::write!(f, "{label}")
    }
}

/// 6-state Module Micro-FSM states (M0..M5).
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::hash::Hash,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum ModuleMicroState {
    /// M0: Blocked waiting for all upstream parent dependencies to reach M4_Hardened.
    M0Blocked,
    /// M1: Deterministic context hydration from G to B_PM^(v_i).
    M1ContextHydration,
    /// M2: Active intra-module ARIADNE Softmax-UCB MCTS search loop.
    M2AriadneMcts,
    /// M3: Extrinsic upstream contract negotiation (emits ev_ContractConflict).
    M3ContractNegotiate,
    /// M4: Hardened module with 100% compilation and parity (R = 1.0).
    M4Hardened,
    /// M5: Warm-start regression state preserving MCTS search tree and test suite.
    M5Invalidated,
}

impl ::core::fmt::Display for self::ModuleMicroState {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let label = match self {
            Self::M0Blocked => "M0: Blocked",
            Self::M1ContextHydration => "M1: Context Hydration",
            Self::M2AriadneMcts => "M2: ARIADNE MCTS",
            Self::M3ContractNegotiate => "M3: Contract Negotiate",
            Self::M4Hardened => "M4: Hardened",
            Self::M5Invalidated => "M5: Invalidated",
        };
        ::core::write!(f, "{label}")
    }
}

/// Broadcast events popped from event bus Q_events (§2 Definition 3).
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum HarelEvent {
    /// Workflow kick-off event.
    Start,
    /// Shard completion in scatter-gather phase Sm.2.
    ShardDone {
        shard_index: ::core::primitive::usize,
    },
    /// Module candidate reached R = 1.0 and committed artifacts.
    ModuleHardened {
        vertex_id: ::std::string::String,
        reward: ::core::primitive::f64,
    },
    /// Tier-B micro-regression: child proves upstream parent contract is insufficient.
    ContractConflict {
        provider_vertex: ::std::string::String,
        consumer_vertex: ::std::string::String,
        delta_spec: ::std::string::String,
        proof_trace: ::std::string::String,
    },
    /// Tier-C macro-regression: undocumented dependency cycle discovered at runtime.
    CyclicDependencyFound {
        cycle_edges: ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
        proof_trace: ::std::string::String,
    },
}

/// Outcome of processing an event in the Harel statechart.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::cmp::PartialEq)]
pub enum EventOutcome {
    ModuleDispatched(::std::vec::Vec<::std::string::String>),
    ModuleHardened { vertex_id: ::std::string::String },
    MacroPhaseAdvanced(self::HarelMacroState),
    TierBMicroRegression { parent_invalidated: ::std::string::String, child_blocked: ::std::string::String },
    TierCMacroRegression { target_macro_state: self::HarelMacroState, snapshot_saved: ::core::primitive::usize },
    Ignored,
}

/// Harel Statechart Engine managing XOR macro-phases and AND orthogonal module regions.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct HarelEngine {
    pub macro_state: self::HarelMacroState,
    pub dag: ::blackboard::CondensedDAG,
    pub module_states: ::std::collections::BTreeMap<::std::string::String, self::ModuleMicroState>,
    pub deep_history_snapshot: ::core::option::Option<::std::collections::BTreeMap<::std::string::String, self::ModuleMicroState>>,
    pub event_queue: ::std::collections::VecDeque<self::HarelEvent>,
}

impl self::HarelEngine {
    /// Create new Harel engine initialized in S0.
    pub fn new(dag: ::blackboard::CondensedDAG) -> Self {
        let mut module_states = ::std::collections::BTreeMap::new();
        for v in &dag.vertices {
            module_states.insert(v.clone(), self::ModuleMicroState::M0Blocked);
        }

        let mut event_queue = ::std::collections::VecDeque::new();
        event_queue.push_back(self::HarelEvent::Start);

        Self {
            macro_state: self::HarelMacroState::S0IntentPreflight,
            dag,
            module_states,
            deep_history_snapshot: ::core::option::Option::None,
            event_queue,
        }
    }

    /// Advance macro-phase if two-stage guards (g_struct and g_sem) pass.
    pub fn step_macro_phase(
        &mut self,
        struct_guard_passed: ::core::primitive::bool,
        sem_guard_passed: ::core::primitive::bool,
    ) -> ::core::result::Result<self::HarelMacroState, ::std::string::String> {
        if !struct_guard_passed {
            return ::core::result::Result::Err("Stage-1 Structural Guard (g_struct) failed".to_string());
        }
        if !sem_guard_passed {
            return ::core::result::Result::Err("Stage-2 Adversarial Semantic Guard (g_sem) failed".to_string());
        }

        let next = match self.macro_state {
            self::HarelMacroState::S0IntentPreflight => self::HarelMacroState::S1Assess,
            self::HarelMacroState::S1Assess => self::HarelMacroState::S2Map,
            self::HarelMacroState::S2Map => self::HarelMacroState::S3ExtractBusinessRules,
            self::HarelMacroState::S3ExtractBusinessRules => self::HarelMacroState::S4BriefReimagineContracts,
            self::HarelMacroState::S4BriefReimagineContracts => {
                // If Deep History exists from prior Tier C regression, restore unaffected modules
                if let ::core::option::Option::Some(snapshot) = self.deep_history_snapshot.take() {
                    for (v, prev_state) in snapshot {
                        if prev_state == self::ModuleMicroState::M4Hardened {
                            self.module_states.insert(v, self::ModuleMicroState::M4Hardened);
                        }
                    }
                }
                self::HarelMacroState::S5IterativeDagTransform
            },
            self::HarelMacroState::S5IterativeDagTransform => self::HarelMacroState::S6SystemVerifyHarden,
            self::HarelMacroState::S6SystemVerifyHarden => self::HarelMacroState::Terminated,
            self::HarelMacroState::Terminated => self::HarelMacroState::Terminated,
        };

        self.macro_state = next;
        ::core::result::Result::Ok(next)
    }

    /// Kahn's topological wave scheduler evaluating unblocked modules in S5 (§4).
    /// ReadySet(G) = { v_i in V | M(v_i) = M0_Blocked and forall (v_p, v_i) in E: M(v_p) = M4_Hardened }
    pub fn compute_ready_set(&self) -> ::std::vec::Vec<::std::string::String> {
        if self.macro_state != self::HarelMacroState::S5IterativeDagTransform {
            return ::std::vec::Vec::new();
        }

        let mut ready = ::std::vec::Vec::new();
        for (v, state) in &self.module_states {
            if *state == self::ModuleMicroState::M0Blocked {
                let parents = self.dag.parents_of(v);
                let all_parents_hardened = parents.iter().all(|p| {
                    self.module_states
                        .get(p)
                        .is_some_and(|s| *s == self::ModuleMicroState::M4Hardened)
                });
                if all_parents_hardened {
                    ready.push(v.clone());
                }
            }
        }
        ready
    }

    /// Dispatch ready modules from M0 -> M1 (context hydration) -> M2 (ARIADNE MCTS).
    pub fn dispatch_ready_modules(&mut self) -> ::std::vec::Vec<::std::string::String> {
        let ready = self.compute_ready_set();
        for v in &ready {
            self.module_states
                .insert(v.clone(), self::ModuleMicroState::M2AriadneMcts);
        }
        ready
    }

    /// Process next broadcast event from event bus Q_events.
    pub fn process_event(&mut self, event: self::HarelEvent) -> ::core::result::Result<self::EventOutcome, ::std::string::String> {
        match event {
            self::HarelEvent::Start => {
                ::core::result::Result::Ok(self::EventOutcome::Ignored)
            },
            self::HarelEvent::ShardDone { .. } => {
                ::core::result::Result::Ok(self::EventOutcome::Ignored)
            },
            self::HarelEvent::ModuleHardened { vertex_id, reward } => {
                if reward < 1.0 {
                    return ::core::result::Result::Err(::std::format!(
                        "Cannot harden module {vertex_id}: reward {reward} < 1.0"
                    ));
                }
                self.module_states.insert(vertex_id.clone(), self::ModuleMicroState::M4Hardened);

                // Check if all vertices are hardened
                let all_hardened = self.module_states.values().all(|s| *s == self::ModuleMicroState::M4Hardened);
                if all_hardened && self.macro_state == self::HarelMacroState::S5IterativeDagTransform {
                    self.macro_state = self::HarelMacroState::S6SystemVerifyHarden;
                    return ::core::result::Result::Ok(self::EventOutcome::MacroPhaseAdvanced(
                        self::HarelMacroState::S6SystemVerifyHarden,
                    ));
                }

                ::core::result::Result::Ok(self::EventOutcome::ModuleHardened { vertex_id })
            },
            self::HarelEvent::ContractConflict {
                provider_vertex,
                consumer_vertex,
                ..
            } => {
                // Tier B Micro-Regression:
                // 1. Park consumer v_i back in M0_Blocked
                self.module_states.insert(consumer_vertex.clone(), self::ModuleMicroState::M0Blocked);

                // 2. Transition provider parent v_p to M5_Invalidated -> M2_AriadneMcts (warm start)
                self.module_states.insert(provider_vertex.clone(), self::ModuleMicroState::M2AriadneMcts);

                ::core::result::Result::Ok(self::EventOutcome::TierBMicroRegression {
                    parent_invalidated: provider_vertex,
                    child_blocked: consumer_vertex,
                })
            },
            self::HarelEvent::CyclicDependencyFound { .. } => {
                // Tier C Macro-Architectural Regression:
                // 1. Snapshot module state vector Theta_H*(S5) <- M
                let snapshot = self.module_states.clone();
                let snapshot_len = snapshot.len();
                self.deep_history_snapshot = ::core::option::Option::Some(snapshot);

                // 2. Transition macro-state S5 -> S2 to condense cycle into SCC via Tarjan's algorithm
                self.macro_state = self::HarelMacroState::S2Map;

                ::core::result::Result::Ok(self::EventOutcome::TierCMacroRegression {
                    target_macro_state: self::HarelMacroState::S2Map,
                    snapshot_saved: snapshot_len,
                })
            },
        }
    }
}
