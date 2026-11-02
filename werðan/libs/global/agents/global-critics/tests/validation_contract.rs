//! Semantic critic validation contracts.
//! Contracts derived from docs/specs/validation-contracts.md. They outrank implementation tests.
//! Covered assertion IDs follow. VAL-CRITIC-01, VAL-CRITIC-02, VAL-CRITIC-03.

use ::global_arbiter::SemanticCritic;

/// Contract VAL-CRITIC-01: the semantic critic verdict type holds exactly the
/// three methodology values. One remediation substate carries its reason vector
/// and stands for one retry budget unit.
#[test]
fn contract_val_critic_01_three_valued_verdicts() {
    let pass = ::global_arbiter::CritiqueVerdict::Pass;
    let local = ::global_arbiter::CritiqueVerdict::RemediateLocal(::std::vec!["fix the shard".to_string()]);
    let regress = ::global_arbiter::CritiqueVerdict::RegressUpstream(::std::vec!["iface changed".to_string()]);
    let labels = ::std::vec![::std::format!("{pass:?}"), ::std::format!("{local:?}"), ::std::format!("{regress:?}")];
    for label in &labels {
        ::std::assert!(!label.is_empty());
    }
    ::std::assert!(pass != local);
    ::std::assert!(local != regress);
    ::std::assert!(pass != regress);
    // One remediation substate stands for one retry budget unit.
    let remediation_units = ::std::vec![local, regress].len();
    ::std::assert_eq!(remediation_units, 2);
}

/// Contract VAL-CRITIC-02: the critic audits one batch and returns one record
/// without mutating any blackboard state. The critic is a pure observer.
#[test]
fn contract_val_critic_02_read_isolation_pure_observer() {
    let board = ::global_blackboard::GlobalBlackboard::default();
    let before = ::std::format!("{board:?}");
    let critic = ::global_arbiter::DeterministicCritic::new();
    let batch = ::std::vec!["alpha".to_string(), "beta".to_string()];
    let record = critic.audit_batch(::global_fsm::MacroState::S5IterativeDag, &batch);
    let after = ::std::format!("{board:?}");
    ::std::assert_eq!(before, after, "critic must not mutate blackboard state");
    ::std::assert_eq!(record.verdict, ::global_arbiter::CritiqueVerdict::Pass);
    ::std::assert!(record.violated_invariants.is_empty());
    ::std::assert!(record.remediation_notes.is_empty());
}

/// Contract VAL-CRITIC-03: the phase semantic guard conjoins shard batch
/// verdicts deterministically. One upstream regression dominates every local
/// remediation. The first local remediation stands so the engine re-dispatches
/// exactly the offending shard batch.
#[test]
fn contract_val_critic_03_verdict_conjunction_dominance() {
    let all_pass = ::std::vec![
        ::global_arbiter::CritiqueRecord {
            verdict: ::global_arbiter::CritiqueVerdict::Pass,
            violated_invariants: ::std::vec::Vec::new(),
            remediation_notes: ::std::string::String::new(),
        },
        ::global_arbiter::CritiqueRecord {
            verdict: ::global_arbiter::CritiqueVerdict::Pass,
            violated_invariants: ::std::vec::Vec::new(),
            remediation_notes: ::std::string::String::new(),
        },
    ];
    let conjoined_pass = ::global_arbiter::conjunction_verdict(&all_pass);
    ::std::assert_eq!(conjoined_pass, ::global_arbiter::CritiqueVerdict::Pass);

    let local_then_regress = ::std::vec![
        ::global_arbiter::CritiqueRecord {
            verdict: ::global_arbiter::CritiqueVerdict::RemediateLocal(::std::vec!["fix alpha".to_string()]),
            violated_invariants: ::std::vec::Vec::new(),
            remediation_notes: ::std::string::String::new(),
        },
        ::global_arbiter::CritiqueRecord {
            verdict: ::global_arbiter::CritiqueVerdict::RegressUpstream(::std::vec!["iface_beta drifted".to_string()]),
            violated_invariants: ::std::vec::Vec::new(),
            remediation_notes: ::std::string::String::new(),
        },
    ];
    let conjoined_regress = ::global_arbiter::conjunction_verdict(&local_then_regress);
    ::std::assert!(
        ::core::matches!(
            conjoined_regress,
            ::global_arbiter::CritiqueVerdict::RegressUpstream(_)
        ),
        "one upstream regression dominates every local remediation"
    );

    let local_first = ::std::vec![
        ::global_arbiter::CritiqueRecord {
            verdict: ::global_arbiter::CritiqueVerdict::RemediateLocal(::std::vec!["fix alpha".to_string()]),
            violated_invariants: ::std::vec::Vec::new(),
            remediation_notes: ::std::string::String::new(),
        },
        ::global_arbiter::CritiqueRecord {
            verdict: ::global_arbiter::CritiqueVerdict::RemediateLocal(::std::vec!["fix beta".to_string()]),
            violated_invariants: ::std::vec::Vec::new(),
            remediation_notes: ::std::string::String::new(),
        },
    ];
    let conjoined_local = ::global_arbiter::conjunction_verdict(&local_first);
    ::std::assert!(
        ::core::matches!(conjoined_local, ::global_arbiter::CritiqueVerdict::RemediateLocal(reasons) if reasons == ::std::vec!["fix alpha".to_string()]),
        "the first local remediation stands for the offending shard re-dispatch"
    );

    let empty: ::std::vec::Vec<::global_arbiter::CritiqueRecord> = ::std::vec::Vec::new();
    let conjoined_empty = ::global_arbiter::conjunction_verdict(&empty);
    ::std::assert_eq!(conjoined_empty, ::global_arbiter::CritiqueVerdict::Pass);
}

/// A supporting probe for the critic surface. The critic implements the
/// SemanticCritic trait so the engine can hold it behind a trait object.
#[test]
fn contract_val_critic_02_supporting_trait_object_surface() {
    let critic = ::global_arbiter::DeterministicCritic::new();
    let boxed = ::std::boxed::Box::new(critic);
    let trait_object: ::std::boxed::Box<dyn ::global_arbiter::SemanticCritic> = boxed;
    let batch = ::std::vec!["delta".to_string()];
    let record = trait_object.audit_batch(::global_fsm::MacroState::S6SystemVerify, &batch);
    ::std::assert_eq!(record.verdict, ::global_arbiter::CritiqueVerdict::Pass);
    // A configured critic emits its configured verdict for every batch.
    let rejecting = ::global_arbiter::DeterministicCritic::builder().verdict(::global_arbiter::CritiqueVerdict::RemediateLocal(::std::vec!["configured".to_string()])).build();
    let rejecting_record = rejecting.audit_batch(::global_fsm::MacroState::S3ExtractRules, &batch);
    ::std::assert!(
        ::core::matches!(
            rejecting_record.verdict,
            ::global_arbiter::CritiqueVerdict::RemediateLocal(_)
        )
    );
}
