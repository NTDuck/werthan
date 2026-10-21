//! Workload sharding and semantic audit batch generation per METHODOLOGY.md Definition 3.

/// One work shard for a phase dispatch round.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub enum PhaseShard {
    /// Single whole-repository shard used by S0 and S1 and S2.
    Whole,
    /// Per-vertex shard used by S3 and S5 and S6.
    Vertex(::std::string::String),
    /// Per-edge shard used by S4.
    Edge(::std::string::String, ::std::string::String),
}

/// Partition the phase workload into deterministic shards.
pub fn partition_shards(
    board: &::global_blackboard::GlobalBlackboard,
    phase: ::global_fsm::MacroState,
    regression_scope: &::core::option::Option<::std::vec::Vec<::std::string::String>>,
) -> ::std::vec::Vec<self::PhaseShard> {
    match phase {
        ::global_fsm::MacroState::S0IntentPreflight
        | ::global_fsm::MacroState::S1Assess
        | ::global_fsm::MacroState::S2MapDag => ::std::vec![self::PhaseShard::Whole],
        ::global_fsm::MacroState::S3ExtractRules => {
            let mut shards = ::std::vec::Vec::new();
            for vertex in self::scoped_vertices(board, regression_scope) {
                shards.push(self::PhaseShard::Vertex(vertex));
            }
            shards
        }
        ::global_fsm::MacroState::S4BriefContracts => {
            let mut shards = ::std::vec::Vec::new();
            for edge_pair in &board.dag.edges {
                shards.push(self::PhaseShard::Edge(edge_pair.0.clone(), edge_pair.1.clone()));
            }
            shards
        }
        ::global_fsm::MacroState::S6SystemVerify => {
            let mut shards = ::std::vec::Vec::new();
            for vertex in &board.dag.vertices {
                shards.push(self::PhaseShard::Vertex(vertex.clone()));
            }
            shards
        }
        ::global_fsm::MacroState::S5IterativeDag | ::global_fsm::MacroState::Terminated => ::std::vec::Vec::new(),
    }
}

/// Build one semantic audit batch per shard for the phase.
pub fn semantic_batches(
    board: &::global_blackboard::GlobalBlackboard,
    phase: ::global_fsm::MacroState,
    regression_scope: &::core::option::Option<::std::vec::Vec<::std::string::String>>,
) -> ::std::vec::Vec<::std::vec::Vec<::std::string::String>> {
    match phase {
        ::global_fsm::MacroState::S0IntentPreflight | ::global_fsm::MacroState::S1Assess => {
            ::std::vec![board.inventory.files.clone()]
        }
        ::global_fsm::MacroState::S2MapDag => ::std::vec![board.dag.vertices.clone()],
        ::global_fsm::MacroState::S3ExtractRules => {
            let mut batches = ::std::vec::Vec::new();
            for vertex in self::scoped_vertices(board, regression_scope) {
                let mut batch = ::std::vec::Vec::new();
                if let ::core::option::Option::Some(rules) = board.rules.get(&vertex) {
                    for card in rules {
                        batch.push(card.rule_id.clone());
                    }
                }
                let batch_empty = batch.is_empty();
                if batch_empty {
                    batch.push(vertex);
                }
                batches.push(batch);
            }
            batches
        }
        ::global_fsm::MacroState::S4BriefContracts => {
            let mut batches = ::std::vec::Vec::new();
            for edge_pair in &board.dag.edges {
                batches.push(::std::vec![::std::format!("{}->{}", edge_pair.0, edge_pair.1)]);
            }
            batches
        }
        ::global_fsm::MacroState::S6SystemVerify => {
            let mut batches = ::std::vec::Vec::new();
            for vertex in &board.dag.vertices {
                batches.push(::std::vec![vertex.clone()]);
            }
            batches
        }
        ::global_fsm::MacroState::S5IterativeDag | ::global_fsm::MacroState::Terminated => ::std::vec::Vec::new(),
    }
}

/// Return the upstream regression target for a phase-level semantic rejection.
pub fn regress_target(phase: ::global_fsm::MacroState) -> ::global_fsm::MacroState {
    match phase {
        ::global_fsm::MacroState::S1Assess => ::global_fsm::MacroState::S0IntentPreflight,
        ::global_fsm::MacroState::S2MapDag => ::global_fsm::MacroState::S1Assess,
        ::global_fsm::MacroState::S3ExtractRules => ::global_fsm::MacroState::S2MapDag,
        ::global_fsm::MacroState::S4BriefContracts => ::global_fsm::MacroState::S3ExtractRules,
        ::global_fsm::MacroState::S6SystemVerify => ::global_fsm::MacroState::S5IterativeDag,
        ::global_fsm::MacroState::S0IntentPreflight
        | ::global_fsm::MacroState::S5IterativeDag
        | ::global_fsm::MacroState::Terminated => phase,
    }
}

/// Return DAG vertices that the active regression scope allows.
fn scoped_vertices(
    board: &::global_blackboard::GlobalBlackboard,
    regression_scope: &::core::option::Option<::std::vec::Vec<::std::string::String>>,
) -> ::std::vec::Vec<::std::string::String> {
    let mut vertices = ::std::vec::Vec::new();
    for vertex in &board.dag.vertices {
        let mut allowed = true;
        if let ::core::option::Option::Some(scope) = regression_scope {
            allowed = scope.contains(vertex);
        }
        if allowed {
            vertices.push(vertex.clone());
        }
    }
    vertices
}
