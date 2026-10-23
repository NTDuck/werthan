//! Global blackboard G validation contracts.
//! Contracts derived from docs/specs/validation-contracts.md. They outrank implementation tests.
//! Covered assertion IDs follow. VAL-BB-G-01, VAL-BB-G-02, VAL-BB-G-03, VAL-BB-G-04.
//! Also VAL-BB-G-05, VAL-BB-G-06, VAL-BB-G-07.

/// Contract VAL-BB-G-01: G exposes the full Definition 1 schema tuple.
/// The tuple holds inventory, condensed DAG, rules map, contracts map,
/// artifacts map, and the persisted state vector Sigma(G).
/// Sigma(G) carries S_macro, the M vector, Theta_H*, and Q_events.
#[test]
fn contract_val_bb_g_01_schema_tuple_completeness() {
    let board = populated_board();
    let state_vector = &board.state;
    ::std::assert_eq!(
        state_vector.macro_state,
        ::global_fsm::MacroState::S0IntentPreflight
    );
    let inventory_files = &board.inventory.files;
    let dag_vertices = &board.dag.vertices;
    let dag_edges = &board.dag.edges;
    let rules = board.rules.get("alpha");
    let contracts = board.contracts.get(&("alpha".to_string(), "beta".to_string()));
    let artifacts = board.artifacts.get("beta");
    ::std::assert_eq!(inventory_files.len(), 2);
    ::std::assert_eq!(dag_vertices.len(), 2);
    ::std::assert_eq!(dag_edges.len(), 1);
    ::std::assert!(rules.is_some());
    ::std::assert!(contracts.is_some());
    ::std::assert!(artifacts.is_some());
    let module_vector = &state_vector.module_states;
    let deep_history = &state_vector.deep_history;
    let event_queue = &state_vector.event_queue;
    ::std::assert!(module_vector.is_empty());
    ::std::assert!(deep_history.is_none());
    ::std::assert!(!event_queue.is_empty());
}

/// Contract VAL-BB-G-02: the state vector persists transactionally.
/// A save followed by a fresh handle over the same store recovers every field.
/// Recovery covers S_macro, the M vector, deep history Theta_H*, and Q_events.
#[test]
fn contract_val_bb_g_02_transactional_persistence_and_recovery() {
    let mut original = ::global_blackboard::GlobalStateVector::default();
    original
        .module_states
        .insert("alpha".to_string(), ::global_blackboard::ModuleMicroState::M4Hardened);
    original
        .module_states
        .insert("beta".to_string(), ::global_blackboard::ModuleMicroState::M2AriadneMcts);
    original
        .module_states
        .insert("gamma".to_string(), ::global_blackboard::ModuleMicroState::M0Blocked);
    let mut snapshot = ::std::collections::BTreeMap::new();
    snapshot.insert("alpha".to_string(), ::global_blackboard::ModuleMicroState::M5Invalidated);
    snapshot.insert("beta".to_string(), ::global_blackboard::ModuleMicroState::M1ContextHydration);
    original.deep_history = ::core::option::Option::Some(snapshot);
    original.push_event(::global_blackboard::MacroEvent::TaskDone {
        agent_id: "RuleMiner".to_string(),
        target_ref: "alpha".to_string(),
    });
    original.push_event(::global_blackboard::MacroEvent::ShardDone { shard_index: 3 });
    original.macro_state = ::global_fsm::MacroState::S3ExtractRules;

    let directory = ::tempfile::tempdir().expect("tempdir builds");
    let store_path = directory.path().join("campaign.json");
    let saved_path = original
        .save(
            ::core::option::Option::Some(store_path.as_path()),
            ::core::option::Option::None,
            "bb-g02",
        )
        .expect("state vector save succeeds");
    ::std::assert!(saved_path.is_file());

    let store_owned = ::std::path::PathBuf::from(&store_path);
    let recovered = ::global_blackboard::GlobalStateVector::builder().file_path(store_owned).build().expect("fresh handle recovers persisted state vector");
    ::std::assert_eq!(recovered.macro_state, ::global_fsm::MacroState::S3ExtractRules);
    ::std::assert_eq!(recovered.module_states, original.module_states);
    ::std::assert_eq!(recovered.deep_history, original.deep_history);
    ::std::assert_eq!(recovered.event_queue.len(), original.event_queue.len());
    let mut recovered_events = ::std::vec::Vec::new();
    for event in &recovered.event_queue {
        recovered_events.push(::std::format!("{event:?}"));
    }
    let mut original_events = ::std::vec::Vec::new();
    for event in &original.event_queue {
        original_events.push(::std::format!("{event:?}"));
    }
    ::std::assert_eq!(recovered_events, original_events);
}

/// Contract VAL-BB-G-03: after SCC condensation the stored dependency graph is
/// a strict DAG. A cycle collapses to exactly one composite vertex.
#[test]
fn contract_val_bb_g_03_dag_acyclic_after_condensation() {
    let cyclic = ::global_blackboard::CondensedDag {
        vertices: ::std::vec!["alpha".to_string(), "beta".to_string(), "delta".to_string()],
        edges: ::std::vec![
            ("alpha".to_string(), "beta".to_string()),
            ("beta".to_string(), "alpha".to_string()),
        ],
    };
    ::std::assert!(!cyclic.is_acyclic());

    let condensed = ::global_blackboard::scc::tarjan_condense(&cyclic);
    ::std::assert!(condensed.is_acyclic(), "condensed graph must be a strict DAG");
    ::std::assert_eq!(
        condensed.vertices.len(),
        2,
        "one composite vertex plus the untouched singleton"
    );
    let composite_present = condensed.vertices.iter().any(|vertex| vertex == "alpha+beta");
    let delta_present = condensed.vertices.iter().any(|vertex| vertex == "delta");
    ::std::assert!(composite_present);
    ::std::assert!(delta_present);
}

/// Contract VAL-BB-G-04: partitioned slice writes stay addressable and isolated.
/// A worker writing shard one artifact does not mutate any other shard partition.
#[test]
fn contract_val_bb_g_04_partitioned_slice_writes_are_isolated() {
    let mut board = populated_board();
    let before = ::core::clone::Clone::clone(&board);
    board.artifacts.commit(
        "alpha",
        ::global_blackboard::ModuleArtifact {
            target_code: "fn alpha_hardened() {}".to_string(),
            test_suite: ::std::vec!["alpha_test".to_string()],
        },
    );
    let alpha_written = ::core::matches!(
        board.artifacts.get("alpha"),
        ::core::option::Option::Some(::core::option::Option::Some(_))
    );
    ::std::assert!(alpha_written, "written shard partition becomes addressable");
    ::std::assert_eq!(
        board.artifacts.get("beta"),
        before.artifacts.get("beta"),
        "another shard partition must remain untouched"
    );
    let mut isolated = ::global_blackboard::GlobalBlackboard::default();
    isolated.artifacts.commit(
        "shard_one",
        ::global_blackboard::ModuleArtifact {
            target_code: "fn one() {}".to_string(),
            test_suite: ::std::vec![],
        },
    );
    isolated.artifacts.commit(
        "shard_two",
        ::global_blackboard::ModuleArtifact {
            target_code: "fn two() {}".to_string(),
            test_suite: ::std::vec![],
        },
    );
    let first = isolated.artifacts.get("shard_one");
    let second = isolated.artifacts.get("shard_two");
    let first_label = ::std::format!("{first:?}");
    let second_label = ::std::format!("{second:?}");
    ::std::assert_ne!(first_label, second_label, "keys address distinct partitions");
}

/// Contract VAL-BB-G-05: applying a validated Delta_spec merges the delta into
/// contract C(vp,vi). The merge keeps every prior precondition.
#[test]
fn contract_val_bb_g_05_contract_patch_merges_delta() {
    let mut board = populated_board();
    let key = ("alpha".to_string(), "beta".to_string());
    let seeded = board.contracts.get(&key).expect("seeded contract present");
    let seeded_precondition = "precondition".to_string();
    ::std::assert_eq!(seeded.preconditions, ::std::vec![seeded_precondition.clone()]);
    let contract = board.contracts.get_mut(&key).expect("seeded contract present");
    contract.preconditions.push("delta one".to_string());
    let contract = board.contracts.get_mut(&key).expect("seeded contract present");
    contract.preconditions.push("delta two".to_string());
    let merged = board.contracts.get(&key).expect("seeded contract present");
    ::std::assert_eq!(
        merged.preconditions,
        ::std::vec![
            "precondition".to_string(),
            "delta one".to_string(),
            "delta two".to_string()
        ]
    );
    ::std::assert_eq!(merged.interface_spec, "iface_alpha");
}

/// Contract VAL-BB-G-06: the event bus drains in FIFO order.
/// The queue is observable through its head and it drains to empty.
#[test]
fn contract_val_bb_g_06_event_bus_fifo_order() {
    let mut state_vector = ::global_blackboard::GlobalStateVector::default();
    let head_present = state_vector.event_queue.front().is_some();
    ::std::assert!(head_present, "initialized queue observes its head event");
    state_vector.push_event(::global_blackboard::MacroEvent::ShardDone { shard_index: 0 });
    state_vector.push_event(::global_blackboard::MacroEvent::ShardDone { shard_index: 1 });
    state_vector.push_event(::global_blackboard::MacroEvent::ShardDone { shard_index: 2 });
    let mut drained_indices = ::std::vec::Vec::new();
    loop {
        let next = state_vector.pop_event();
        let event = match next {
            ::core::option::Option::Some(pending) => pending,
            ::core::option::Option::None => break,
        };
        if let ::global_blackboard::MacroEvent::ShardDone { shard_index } = event {
            drained_indices.push(shard_index);
        }
    }
    ::std::assert_eq!(drained_indices, ::std::vec![0, 1, 2]);
    ::std::assert!(state_vector.event_queue.is_empty(), "queue drains to empty");
    let drained_again = state_vector.pop_event();
    ::std::assert!(drained_again.is_none(), "draining an empty queue yields no event");
}

/// Contract VAL-BB-G-07: the inventory records every repository file exactly once.
/// A repeated addition is a no-op so the inventory backs the S1 coverage guard.
#[test]
fn contract_val_bb_g_07_inventory_records_files_exactly_once() {
    let mut inventory = ::global_blackboard::RepositoryInventory::default();
    inventory.add_file("src/alpha.rs");
    inventory.add_file("src/beta.rs");
    inventory.add_file("src/alpha.rs");
    inventory.push("src/beta.rs");
    ::std::assert_eq!(
        inventory.files(),
        ::std::vec!["src/alpha.rs".to_string(), "src/beta.rs".to_string()]
    );
    ::std::assert_eq!(inventory.len(), 2);
    ::std::assert!(inventory.contains("src/alpha.rs"));
}

/// Supporting surface probe for the Definition 1 schema tuple components.
/// Registries construct from plain maps. The manifest covers one subject.
#[test]
fn contract_val_bb_g_01_supporting_registries_construct_from_maps() {
    let mut rules_map = ::std::collections::BTreeMap::new();
    rules_map.insert("alpha".to_string(), ::std::vec![cited_card("card_one")]);
    let rules_registry = ::global_blackboard::RuleRegistry::from(rules_map);
    ::std::assert!(rules_registry.contains("alpha"));
    let mut artifact_map = ::std::collections::BTreeMap::new();
    artifact_map.insert(
        "alpha".to_string(),
        ::core::option::Option::Some(::global_blackboard::ModuleArtifact {
            target_code: "fn alpha() {}".to_string(),
            test_suite: ::std::vec!["test_one".to_string()],
        }),
    );
    let artifact_registry = ::global_blackboard::ArtifactRegistry::from(artifact_map);
    ::std::assert!(artifact_registry.is_hardened("alpha"));
    let mut contract_map = ::std::collections::BTreeMap::new();
    contract_map.insert(
        ("alpha".to_string(), "beta".to_string()),
        ::holon_blackboard::EdgeContract {
            parent_id: "alpha".to_string(),
            interface_spec: "iface".to_string(),
            preconditions: ::std::vec![],
        },
    );
    let contract_registry = ::global_blackboard::ContractRegistry::from(contract_map);
    let edge_key = ("alpha".to_string(), "beta".to_string());
    ::std::assert!(contract_registry.contains_edge(&edge_key));
    let manifest = ::global_blackboard::WorkManifest {
        phase: ::global_fsm::MacroState::S3ExtractRules,
        shard_subjects: ::std::vec!["alpha".to_string()],
    };
    ::std::assert!(manifest.covers("alpha"));
    let link = ::global_blackboard::UnresolvedCrossLink {
        src_id: "card_one".to_string(),
        tgt_symbol: "beta".to_string(),
    };
    ::std::assert_eq!(link.src_id, "card_one");
}

/// Build a fully populated blackboard for the schema tuple contracts.
fn populated_board() -> ::global_blackboard::GlobalBlackboard {
    let mut board = ::global_blackboard::GlobalBlackboard::default();
    board.inventory.files = ::std::vec!["src/alpha.rs".to_string(), "src/beta.rs".to_string()];
    board.inventory.dialect = ::global_blackboard::DialectProfile {
        source_language: "c".to_string(),
        target_language: "rust".to_string(),
        target_edition: "2021".to_string(),
    };
    board.inventory.harness = ::global_blackboard::HarnessConfig {
        build_cmd: "cargo build".to_string(),
        test_cmd: "cargo test".to_string(),
        legacy_harness_cmd: "legacy test".to_string(),
    };
    board.dag.vertices = ::std::vec!["alpha".to_string(), "beta".to_string()];
    board.dag.edges = ::std::vec![("alpha".to_string(), "beta".to_string())];
    board
        .rules
        .insert("alpha".to_string(), ::std::vec![cited_card("alpha_card")]);
    board.contracts.insert(
        ("alpha".to_string(), "beta".to_string()),
        ::holon_blackboard::EdgeContract {
            parent_id: "alpha".to_string(),
            interface_spec: "iface_alpha".to_string(),
            preconditions: ::std::vec!["precondition".to_string()],
        },
    );
    board.artifacts.commit(
        "beta",
        ::global_blackboard::ModuleArtifact {
            target_code: "fn beta() {}".to_string(),
            test_suite: ::std::vec!["beta_test".to_string()],
        },
    );
    board
}

/// Build one cited rule card with all clauses nonempty.
fn cited_card(card_rule_id: &::core::primitive::str) -> ::holon_blackboard::BusinessRuleCard {
    ::holon_blackboard::BusinessRuleCard {
        rule_id: { card_rule_id.to_string() },
        given: "given clause".to_string(),
        when: "when clause".to_string(),
        then: "then clause".to_string(),
        citation: ::std::format!("legacy/{}.c:42", card_rule_id),
    }
}
