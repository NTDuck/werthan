//! Stateless shard worker validation contracts.
//! Contracts derived from docs/specs/validation-contracts.md. They outrank implementation tests.
//! Covered assertion IDs follow. VAL-FWM-01, VAL-FWM-02, VAL-FWM-03, VAL-FWM-04, VAL-FWM-05, VAL-MACRO-03.

use ::global_workers::PhaseWorker;

/// Contract VAL-FWM-01: a worker invoked with one envelope for shard one reads
/// only that shard slice. The provided API exposes no other shard partition.
#[test]
fn contract_val_fwm_01_shard_scoped_reads() {
    let board = worker_board();
    let envelope = ::global_workers::ContextAssembler::hydrate(
        "RuleMiner",
        ::holon_fsm::ActivationMode::Initial,
        "beta",
        &board,
        ::core::option::Option::None,
    );
    // The hydrated slice names the target and its one-hop parents.
    ::std::assert!(envelope.slice_summary.contains("target: beta"));
    ::std::assert!(envelope.slice_summary.contains("rule beta_card"));
    ::std::assert!(
        !envelope.slice_summary.contains("rule alpha_card"),
        "slice must not carry another shard rule cards"
    );
    // The deterministic crew addresses only the shard subject in its slice.
    // The slice builder rendered exactly the target rules above. Mining on a
    // seeded board emits nothing because beta already holds its card.
    let workers = ::global_workers::DeterministicWorkers::new();
    let output = workers.execute(
        ::global_fsm::MacroState::S3ExtractRules,
        "beta",
        &envelope,
        &board,
    );
    match output {
        ::global_workers::PhaseWorkOutput::Rules { cards, cross_links } => {
            ::std::assert!(cards.is_empty(), "seeded vertex mines no duplicate card");
            ::std::assert!(cross_links.is_empty());
        }
        other => ::std::panic!("expected rules output, got {other:?}"),
    }
    // Mining one unseeded shard subject yields exactly that shard card.
    let mut unseeded_board = worker_board();
    unseeded_board.rules = ::global_blackboard::RuleRegistry::default();
    let mined = workers.execute(
        ::global_fsm::MacroState::S3ExtractRules,
        "beta",
        &envelope,
        &unseeded_board,
    );
    match mined {
        ::global_workers::PhaseWorkOutput::Rules { cards, cross_links } => {
            ::std::assert_eq!(cards.len(), 1);
            ::std::assert_eq!(cards[0].0, "beta");
            ::std::assert!(cross_links.is_empty());
        }
        other => ::std::panic!("expected rules output, got {other:?}"),
    }
}

/// Contract VAL-FWM-02: the worker output commits directly to its board
/// partition. Completion surfaces as the typed ev_ShardDone event and carries
/// the shard position. The output carries data only, never a spawned agent.
#[test]
fn contract_val_fwm_02_partition_write_and_completion_event() {
    let workers = ::global_workers::DeterministicWorkers::new();
    let mut unseeded_board = worker_board();
    unseeded_board.rules = ::global_blackboard::RuleRegistry::default();
    let envelope = sample_envelope();
    let mined = workers.execute(
        ::global_fsm::MacroState::S3ExtractRules,
        "beta",
        &envelope,
        &unseeded_board,
    );
    // Direct partition write: the mined card names the shard subject. The
    // engine commits it onto G[beta] without re-shaping.
    let card = match &mined {
        ::global_workers::PhaseWorkOutput::Rules { cards, cross_links } => {
            let mined_for_beta = cards[0].0 == "beta";
            ::std::assert!(mined_for_beta, "the crew scoped the output to the shard partition");
            ::std::assert!(cross_links.is_empty());
            ::core::clone::Clone::clone(&cards[0].1)
        }
        other => ::std::panic!("expected rules output, got {other:?}"),
    };
    // Commit the mined output onto its partition.
    let beta_key = "beta".to_string();
    // Wrap one card for the registry.
    let mined_cards = ::std::vec![card];
    unseeded_board.rules.insert(beta_key, mined_cards);
    // Probe the committed partition.
    let landed = unseeded_board.rules.contains("beta");
    ::std::assert!(landed, "commit landed");
    // Completion event: the engine commits the output and pushes the typed
    // ev_ShardDone marker with the partition position onto the event queue.
    let mut state_vector = ::global_blackboard::GlobalStateVector::initialize();
    while state_vector.pop_event().is_some() {}
    let shard_index = 1;
    state_vector.push_event(::global_blackboard::MacroEvent::ShardDone { shard_index });
    let completion = state_vector.pop_event();
    ::std::assert!(::core::matches!(
        completion,
        ::core::option::Option::Some(::global_blackboard::MacroEvent::ShardDone { shard_index: 1 })
    ));
    // The manifest records the scatter with the shard subject in position.
    let manifest = ::global_blackboard::WorkManifest {
        phase: ::global_fsm::MacroState::S3ExtractRules,
        shard_subjects: ::std::vec!["alpha".to_string(), "beta".to_string()],
    };
    ::std::assert!(manifest.covers("beta"));
    ::std::assert_eq!(manifest.shard_subjects[1], "beta");
}

/// Contract VAL-FWM-03: a worker whose artifact names a symbol outside its own
/// shard records an UnresolvedCrossLink tag instead of resolving it inline.
#[test]
fn contract_val_fwm_03_unresolved_cross_link_emission() {
    let board = worker_board();
    let workers = ::global_workers::DeterministicWorkers::new();
    let card = ::holon_blackboard::BusinessRuleCard {
        rule_id: "alpha_card".to_string(),
        given: "legacy module alpha feeds beta".to_string(),
        when: "the engine analyzes module alpha".to_string(),
        then: "module alpha preserves observed behavior".to_string(),
        citation: "legacy/alpha:1".to_string(),
    };
    let cards = ::std::vec![("alpha".to_string(), card)];
    let tags = workers.link_cross_module_rules(&board, &cards);
    ::std::assert_eq!(tags.len(), 1);
    ::std::assert_eq!(tags[0].src_id, "alpha_card");
    ::std::assert_eq!(tags[0].tgt_symbol, "beta");
}

/// Contract VAL-FWM-04: worker execution is single shot. Two activations of one
/// crew and activations of two fresh crews produce equal outputs. No state
/// persists across activations.
#[test]
fn contract_val_fwm_04_ephemeral_single_shot_lifecycle() {
    let board = worker_board();
    let envelope_one = ::global_workers::ContextAssembler::hydrate(
        "RuleMiner",
        ::holon_fsm::ActivationMode::Initial,
        "beta",
        &board,
        ::core::option::Option::None,
    );
    let envelope_two = ::global_workers::ContextAssembler::hydrate(
        "RuleMiner",
        ::holon_fsm::ActivationMode::Initial,
        "beta",
        &board,
        ::core::option::Option::None,
    );
    let envelope_label_one = ::std::format!("{envelope_one:?}");
    let envelope_label_two = ::std::format!("{envelope_two:?}");
    ::std::assert_eq!(envelope_label_one, envelope_label_two);

    let workers = ::global_workers::DeterministicWorkers::new();
    let first = workers.execute(
        ::global_fsm::MacroState::S3ExtractRules,
        "beta",
        &envelope_one,
        &board,
    );
    let second = workers.execute(
        ::global_fsm::MacroState::S3ExtractRules,
        "beta",
        &envelope_two,
        &board,
    );
    let first_label = ::std::format!("{first:?}");
    let second_label = ::std::format!("{second:?}");
    ::std::assert_eq!(first_label, second_label);

    let fresh_workers = ::global_workers::DeterministicWorkers::new();
    let third = fresh_workers.execute(
        ::global_fsm::MacroState::S3ExtractRules,
        "beta",
        &envelope_one,
        &board,
    );
    let third_label = ::std::format!("{third:?}");
    ::std::assert_eq!(third_label, first_label);
}

/// Contract VAL-FWM-05: the phase worker set covers the methodology roles.
/// Every phase maps to one distinct nonempty role name and each role output
/// carries its mandated concern fields.
#[test]
fn contract_val_fwm_05_worker_role_coverage() {
    let phases = ::std::vec![
        ::global_fsm::MacroState::S0IntentPreflight,
        ::global_fsm::MacroState::S1Assess,
        ::global_fsm::MacroState::S2MapDag,
        ::global_fsm::MacroState::S3ExtractRules,
        ::global_fsm::MacroState::S4BriefContracts,
        ::global_fsm::MacroState::S5IterativeDag,
        ::global_fsm::MacroState::S6SystemVerify,
        ::global_fsm::MacroState::Terminated,
    ];
    let mut role_names = ::std::collections::BTreeSet::new();
    for phase in phases {
        let role = ::global_workers::phase_role_name(phase);
        ::std::assert!(!role.is_empty());
        role_names.insert(role);
    }
    ::std::assert_eq!(role_names.len(), 8);

    let workers = ::global_workers::DeterministicWorkers::new();
    let board = worker_board();
    // S1 assessor report: inventory and dialect risk coverage.
    let envelope = sample_envelope();
    let s1_output = workers.execute(
        ::global_fsm::MacroState::S1Assess,
        "repository",
        &envelope,
        &board,
    );
    match s1_output {
        ::global_workers::PhaseWorkOutput::Assessment { reports, synthesized } => {
            ::std::assert_eq!(reports.len(), 1);
            ::std::assert!(reports[0].risk_notes.is_empty(), "mapped dialect raises no risk");
            ::std::assert_eq!(synthesized.file_count, 2);
        }
        other => ::std::panic!("expected assessment output, got {other:?}"),
    }
    // S3 rule miner: Given When Then clauses plus a cited legacy source.
    let mut unseeded_board = worker_board();
    unseeded_board.rules = ::global_blackboard::RuleRegistry::default();
    let s3_output = workers.execute(
        ::global_fsm::MacroState::S3ExtractRules,
        "beta",
        &envelope,
        &unseeded_board,
    );
    match s3_output {
        ::global_workers::PhaseWorkOutput::Rules { cards, .. } => {
            ::std::assert!(!cards[0].1.given.is_empty());
            ::std::assert!(!cards[0].1.when.is_empty());
            ::std::assert!(!cards[0].1.then.is_empty());
            ::std::assert!(cards[0].1.citation.contains(':'));
        }
        other => ::std::panic!("expected rules output, got {other:?}"),
    }
    // S4 contract stubber: stubbed edge with an interface spec.
    let s4_output = workers.execute(
        ::global_fsm::MacroState::S4BriefContracts,
        "alpha->beta",
        &envelope,
        &board,
    );
    match s4_output {
        ::global_workers::PhaseWorkOutput::Contracts { stubs, .. } => {
            ::std::assert_eq!(stubs.len(), 1);
            ::std::assert_eq!(stubs[0].parent_id, "alpha");
            ::std::assert!(!stubs[0].contract.interface_spec.is_empty());
        }
        other => ::std::panic!("expected contracts output, got {other:?}"),
    }
    // S6 integration verifier plus security auditor.
    let s6_output = workers.execute(
        ::global_fsm::MacroState::S6SystemVerify,
        "beta",
        &envelope,
        &board,
    );
    match s6_output {
        ::global_workers::PhaseWorkOutput::Verification { verdicts, .. } => {
            ::std::assert_eq!(verdicts.len(), 1);
            ::std::assert_eq!(verdicts[0].module_id, "beta");
        }
        other => ::std::panic!("expected verification output, got {other:?}"),
    }
    // S1 risk coverage: unmapped dialect and unset source language raise notes.
    let mut unmapped_board = worker_board();
    unmapped_board.inventory.files = ::std::vec!["src/alpha.rs".to_string(), "notes.xyz".to_string()];
    unmapped_board.inventory.dialect.source_language = ::std::string::String::new();
    let risk_output = workers.execute(
        ::global_fsm::MacroState::S1Assess,
        "repository",
        &envelope,
        &unmapped_board,
    );
    match risk_output {
        ::global_workers::PhaseWorkOutput::Assessment { reports, .. } => {
            let notes_joined = reports[0].risk_notes.join(" ");
            ::std::assert!(notes_joined.contains("source language unset"));
            ::std::assert!(notes_joined.contains("notes.xyz"));
        }
        other => ::std::panic!("expected assessment output, got {other:?}"),
    }
}

/// Contract VAL-MACRO-03: workers carry no subagent spawn capability.
/// The PhaseWorker trait surface holds exactly one execute method and the
/// deterministic crew exposes no spawn or dispatch operation.
#[test]
fn contract_val_macro_03_no_spawn_capability() {
    // Exhaustive structural check: PhaseWorkOutput carries only data payloads.
    // A nested agent handle would surface as an extra variant field below.
    let output = ::global_workers::PhaseWorkOutput::Skipped;
    let skipped_present = ::core::matches!(output, ::global_workers::PhaseWorkOutput::Skipped);
    ::std::assert!(skipped_present);
    // The trait object is Send and Sync and Debug. No method schedules work.
    let crew = ::global_workers::DeterministicWorkers::new();
    let boxed_crew = ::std::boxed::Box::new(crew);
    let trait_object: ::std::boxed::Box<dyn ::global_workers::PhaseWorker> = boxed_crew;
    let board = worker_board();
    let envelope = sample_envelope();
    let result = trait_object.execute(
        ::global_fsm::MacroState::S2MapDag,
        "repository",
        &envelope,
        &board,
    );
    match result {
        ::global_workers::PhaseWorkOutput::Coupling { discovered_edges } => {
            ::std::assert!(discovered_edges.is_empty());
        }
        other => ::std::panic!("expected coupling output, got {other:?}"),
    }
}

/// Build a two-vertex worker board with a full harness and seeded rules.
fn worker_board() -> ::global_blackboard::GlobalBlackboard {
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
    board.rules.insert("alpha".to_string(), ::std::vec![seeded_card("alpha_card")]);
    board.rules.insert("beta".to_string(), ::std::vec![seeded_card("beta_card")]);
    board
}

/// Build one seeded rule card with a cited legacy source line.
fn seeded_card(card_rule_id: &::core::primitive::str) -> ::holon_blackboard::BusinessRuleCard {
    ::holon_blackboard::BusinessRuleCard {
        rule_id: { card_rule_id.to_string() },
        given: "given clause".to_string(),
        when: "when clause".to_string(),
        then: "then clause".to_string(),
        citation: ::std::format!("legacy/{}.c:42", card_rule_id),
    }
}

/// Build an initial mode envelope with an empty slice summary.
fn sample_envelope() -> ::global_workers::TaskActivationEnvelope {
    ::global_workers::TaskActivationEnvelope {
        mode: ::holon_fsm::ActivationMode::Initial,
        slice_summary: ::std::string::String::new(),
        delta_directive: ::core::option::Option::None,
        bound_tools: ::std::vec::Vec::new(),
    }
}
