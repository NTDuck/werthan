//! Transactionally persisted state configuration Sigma(G) per METHODOLOGY.md Definition 1.

/// Mirrored micro-state enumeration for macro-level bookkeeping of module regions.
/// This copy avoids a crate cycle between blackboard and fsm. Values match fsm MicroState.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub enum ModuleMicroState {
    /// Blocked on upstream parent hardening.
    M0Blocked,
    /// Deterministic context hydration from the global blackboard.
    M1ContextHydration,
    /// Active ARIADNE cluster synthesis.
    M2AriadneMcts,
    /// Contract negotiation with the upstream parent.
    M3ContractNegotiate,
    /// Hardened with verified code and tests.
    M4Hardened,
    /// Invalidated by downstream conflict. Awaits warm start.
    M5Invalidated,
}

/// Broadcast event calculus on the global event bus Q_events from METHODOLOGY.md Definition 1.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub enum MacroEvent {
    /// Initial start event seeding the event queue per Algorithm 1 step 1.
    Start,
    /// Module hardened with verified code and tests. Tier A completion.
    ModuleHardened {
        /// Hardened module identifier.
        module_id: ::std::string::String,
        /// Verified target code.
        target_code: ::std::string::String,
        /// Characterization test suite.
        test_suite: ::std::vec::Vec<::std::string::String>,
    },
    /// Extrinsic contract conflict on an upstream parent. Tier B micro-regression.
    ContractConflict {
        /// Upstream parent identifier.
        parent_id: ::std::string::String,
        /// Reporting downstream module identifier.
        module_id: ::std::string::String,
        /// Contract delta specification.
        delta_spec: ::std::string::String,
        /// Failure proof trace.
        proof_trace: ::std::string::String,
    },
    /// Undocumented cyclic dependency. Tier C macro-regression with deep history H*.
    CyclicDependencyFound {
        /// Cyclic edge pairs.
        cycle_edges: ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
        /// Failure proof trace.
        proof_trace: ::std::string::String,
    },
    /// Ephemeral agent finished its task. Zero-traffic acknowledgement per the
    /// push-based ephemeral invocation contract (tech-spec 2.6).
    TaskDone {
        /// Identifier of the terminating agent role.
        agent_id: ::std::string::String,
        /// Shard or vertex the agent targeted.
        target_ref: ::std::string::String,
    },
    /// One scatter shard worker completed. Algorithm 1 substate S_m.2 event.
    ShardDone {
        /// Zero-based index of the completed shard.
        shard_index: ::core::primitive::usize,
    },
}

/// Transactionally persisted state configuration Sigma(G) from METHODOLOGY.md Definition 1.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct GlobalStateVector {
    /// Active macro state S_macro.
    pub macro_state: ::global_fsm::MacroState,
    /// Per-module micro-state vector M over all vertices.
    pub module_states: ::std::collections::BTreeMap<::std::string::String, self::ModuleMicroState>,
    /// Deep history snapshot Theta_H* taken on macro-regression out of S5.
    pub deep_history: ::core::option::Option<::std::collections::BTreeMap<::std::string::String, self::ModuleMicroState>>,
    /// FIFO asynchronous event bus Q_events.
    pub event_queue: ::std::collections::VecDeque<self::MacroEvent>,
}

#[::bon::bon]
impl self::GlobalStateVector {
    /// Construct a global state vector using a bon builder.
    #[builder]
    pub fn builder(
        file_path: ::core::option::Option<::std::path::PathBuf>,
        output_dir: ::core::option::Option<::std::path::PathBuf>,
        #[builder(default = "default".to_string())]
        hash: ::std::string::String,
        macro_state: ::core::option::Option<::global_fsm::MacroState>,
        #[builder(default = false)]
        create_if_missing: ::core::primitive::bool,
    ) -> ::core::result::Result<Self, ::std::io::Error> {
        let resolved_path = crate::path_resolver::resolve_component_path(
            file_path.as_deref(),
            output_dir.as_deref(),
            &hash,
            "state.json",
        );
        let mut instance = if resolved_path.is_file() {
            let file_contents = ::std::fs::read_to_string(&resolved_path)?;
            let parsed_result = ::serde_json::from_str::<Self>(&file_contents);
            match parsed_result {
                ::core::result::Result::Ok(data) => data,
                ::core::result::Result::Err(err) => {
                    let io_error = ::std::io::Error::new(::std::io::ErrorKind::InvalidData, err);
                    return ::core::result::Result::Err(io_error);
                }
            }
        } else {
            Self::initialize()
        };
        if let ::core::option::Option::Some(initial_state) = macro_state {
            instance.macro_state = initial_state;
        }
        if create_if_missing && !resolved_path.is_file() {
            if let ::core::option::Option::Some(parent_dir) = resolved_path.parent() {
                let _ = ::std::fs::create_dir_all(parent_dir);
            }
            let serialized_res = ::serde_json::to_string_pretty(&instance);
            let serialized = match serialized_res {
                ::core::result::Result::Ok(content) => content,
                ::core::result::Result::Err(err) => {
                    let io_error = ::std::io::Error::new(::std::io::ErrorKind::InvalidData, err);
                    return ::core::result::Result::Err(io_error);
                }
            };
            let _ = ::std::fs::write(&resolved_path, serialized);
        }
        ::core::result::Result::Ok(instance)
    }

    /// Save state vector to resolved component path.
    pub fn save(
        &self,
        file_path: ::core::option::Option<&::std::path::Path>,
        output_dir: ::core::option::Option<&::std::path::Path>,
        hash: &::core::primitive::str,
    ) -> ::core::result::Result<::std::path::PathBuf, ::std::io::Error> {
        let target_path = crate::path_resolver::resolve_component_path(
            file_path,
            output_dir,
            hash,
            "state.json",
        );
        if let ::core::option::Option::Some(parent_dir) = target_path.parent() {
            let _ = ::std::fs::create_dir_all(parent_dir);
        }
        let serialized_res = ::serde_json::to_string_pretty(self);
        let serialized = match serialized_res {
            ::core::result::Result::Ok(content) => content,
            ::core::result::Result::Err(err) => {
                let io_error = ::std::io::Error::new(::std::io::ErrorKind::InvalidData, err);
                return ::core::result::Result::Err(io_error);
            }
        };
        ::std::fs::write(&target_path, serialized)?;
        ::core::result::Result::Ok(target_path)
    }
    /// Initialize Sigma(G) per Algorithm 1 step 1 with S0, an empty module vector, and [Start].
    pub fn initialize() -> Self {
        let mut event_queue = ::std::collections::VecDeque::new();
        event_queue.push_back(self::MacroEvent::Start);
        Self {
            macro_state: ::global_fsm::MacroState::S0IntentPreflight,
            module_states: ::std::collections::BTreeMap::new(),
            deep_history: ::core::option::Option::None,
            event_queue,
        }
    }

    /// Push an event to the tail of the event bus queue.
    pub fn push_event(&mut self, event: self::MacroEvent) {
        self.event_queue.push_back(event);
    }

    /// Pop an event from the head of the event bus queue.
    pub fn pop_event(&mut self) -> ::core::option::Option<self::MacroEvent> {
        self.event_queue.pop_front()
    }

    /// Report whether deep history Theta_H* holds a checkpointed state for a vertex.
    pub fn history_state_of(
        &self,
        vertex: &::core::primitive::str,
    ) -> ::core::option::Option<self::ModuleMicroState> {
        let history = self.deep_history.as_ref()?;
        history.get(vertex).copied()
    }

    /// Save the current module states into the deep history snapshot Theta_H*.
    pub fn save_deep_history(&mut self) {
        let snapshot = self.module_states.clone();
        self.deep_history = ::core::option::Option::Some(snapshot);
    }

    /// Restore module states from the deep history snapshot Theta_H*.
    pub fn restore_deep_history(&mut self) {
        if let ::core::option::Option::Some(history) = &self.deep_history {
            for (vertex, state) in history {
                let vertex_key = vertex.clone();
                let state_val = *state;
                self.module_states.insert(vertex_key, state_val);
            }
        }
    }
}

impl ::core::default::Default for self::GlobalStateVector {
    fn default() -> Self {
        Self::initialize()
    }
}
