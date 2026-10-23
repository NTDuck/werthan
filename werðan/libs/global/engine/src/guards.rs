//! Structural guard validation per METHODOLOGY.md section 3 guard table.

/// Route the structural guard for a phase to its per-phase pure function.
/// Doc: METHODOLOGY.md section 3 guard table.
pub fn structural_guard(
    phase: ::global_fsm::MacroState,
    board: &::global_blackboard::GlobalBlackboard,
) -> ::core::result::Result<(), ::std::string::String> {
    match phase {
        ::global_fsm::MacroState::S0IntentPreflight => self::guard_s0(board),
        ::global_fsm::MacroState::S1Assess => self::guard_s1(board),
        ::global_fsm::MacroState::S2MapDag => self::guard_s2(board),
        ::global_fsm::MacroState::S3ExtractRules => self::guard_s3(board),
        ::global_fsm::MacroState::S4BriefContracts => self::guard_s4(board),
        ::global_fsm::MacroState::S6SystemVerify => self::guard_s6(board),
        ::global_fsm::MacroState::S5IterativeDag => ::core::result::Result::Ok(()),
        ::global_fsm::MacroState::Terminated => ::core::result::Result::Ok(()),
    }
}

/// S0 guard. Requires a nonempty inventory with nonempty entries.
fn guard_s0(board: &::global_blackboard::GlobalBlackboard) -> ::core::result::Result<(), ::std::string::String> {
    let inventory_empty = board.inventory.is_empty();
    if inventory_empty {
        return ::core::result::Result::Err("repository: inventory is empty".to_string());
    }
    for entry in &board.inventory {
        let entry_empty = entry.is_empty();
        if entry_empty {
            return ::core::result::Result::Err(::std::format!("{entry}: inventory holds an empty entry"));
        }
    }
    ::core::result::Result::Ok(())
}

/// S1 guard. Requires unique inventory entries that cover every module.
fn guard_s1(board: &::global_blackboard::GlobalBlackboard) -> ::core::result::Result<(), ::std::string::String> {
    let mut seen = ::std::collections::BTreeSet::new();
    for entry in &board.inventory {
        let duplicate = seen.contains(entry);
        if duplicate {
            return ::core::result::Result::Err(::std::format!("{entry}: inventory holds a duplicate entry"));
        }
        seen.insert(entry.clone());
    }
    ::core::result::Result::Ok(())
}

/// S2 guard. Requires an acyclic DAG and inventory coverage of every file.
fn guard_s2(board: &::global_blackboard::GlobalBlackboard) -> ::core::result::Result<(), ::std::string::String> {
    let cyclic = !board.dag.is_acyclic();
    if cyclic {
        return ::core::result::Result::Err("repository: dag holds a cycle".to_string());
    }
    for entry in &board.inventory {
        let mut covered = false;
        for vertex in &board.dag.vertices {
            let split = vertex.split('+');
            for part in split {
                let part_matches = part == entry;
                if part_matches {
                    covered = true;
                }
            }
        }
        let uncovered = !covered;
        if uncovered {
            return ::core::result::Result::Err(::std::format!("{entry}: inventory entry not covered by a vertex"));
        }
    }
    ::core::result::Result::Ok(())
}

/// S3 guard. Requires every rule card to hold nonempty clauses and a cited source.
fn guard_s3(board: &::global_blackboard::GlobalBlackboard) -> ::core::result::Result<(), ::std::string::String> {
    for (vertex, rules) in board.rules.iter() {
        for card in rules {
            let clauses_empty = card.given.is_empty() || card.when.is_empty() || card.then.is_empty();
            if clauses_empty {
                return ::core::result::Result::Err(::std::format!("{vertex}: rule card holds an empty clause"));
            }
            let citation_valid = card.citation.contains(':');
            if !citation_valid {
                return ::core::result::Result::Err(::std::format!("{vertex}: rule card citation lacks a line marker"));
            }
        }
    }
    ::core::result::Result::Ok(())
}

/// S4 guard. Requires every DAG edge to hold a contract entry.
fn guard_s4(board: &::global_blackboard::GlobalBlackboard) -> ::core::result::Result<(), ::std::string::String> {
    for edge in &board.dag.edges {
        let has_contract = board.contracts.contains_key(edge);
        if !has_contract {
            return ::core::result::Result::Err(::std::format!(
                "{}->{}: dag edge lacks a contract entry",
                edge.0, edge.1
            ));
        }
    }
    ::core::result::Result::Ok(())
}

/// S6 guard. Requires every vertex to hold a hardened artifact.
fn guard_s6(board: &::global_blackboard::GlobalBlackboard) -> ::core::result::Result<(), ::std::string::String> {
    for vertex in &board.dag.vertices {
        let hardened = ::core::matches!(board.artifacts.get(vertex), ::core::option::Option::Some(::core::option::Option::Some(_)));
        if !hardened {
            return ::core::result::Result::Err(::std::format!("{vertex}: vertex lacks a hardened artifact"));
        }
    }
    ::core::result::Result::Ok(())
}
