//! Global arbiter validation contracts.
//! Contracts derived from docs/specs/validation-contracts.md. They outrank implementation tests.
//! Covered assertion IDs follow. VAL-ARBITER-01, VAL-ARBITER-02.

use ::global_arbiter::Arbiter;

/// Contract VAL-ARBITER-01: the arbiter adjudicates one interrupt at a time and
/// returns one binary decision. The decision mirrors the proof trace content
/// and the configured policy.
#[test]
fn contract_val_arbiter_01_interrupt_adjudication_binary_decision() {
    let arbiter = ::global_arbiter::DeterministicArbiter::new();
    // A trace with proof content gets acceptance.
    let accepted = arbiter.adjudicate("alpha", "contract conflict trace names alpha and beta");
    ::std::assert!(accepted, "nonempty proof trace gets a true decision");
    // An empty trace carries no proof. The decision is false.
    let rejected = arbiter.adjudicate("alpha", "");
    ::std::assert!(!rejected, "empty proof trace gets a false decision");
    // A configured reject policy overrides trace content.
    let rejecting = ::global_arbiter::DeterministicArbiter::builder().accept_nonempty_trace(false).build();
    let policy_rejected = rejecting.adjudicate("alpha", "proof trace content");
    ::std::assert!(!policy_rejected, "configured reject policy overrides trace content");
    // The engine holds the arbiter behind the trait object boundary.
    let arbiter_inner = ::global_arbiter::DeterministicArbiter::new();
    let boxed = ::std::boxed::Box::new(arbiter_inner);
    let trait_object: ::std::boxed::Box<dyn ::global_arbiter::Arbiter> = boxed;
    ::std::assert!(trait_object.adjudicate("beta", "trace"));
    ::std::assert!(!trait_object.adjudicate("beta", ""));
}

/// Contract VAL-ARBITER-02: adjudication is deterministic. Replaying the same
/// interrupt trace against a fresh arbiter yields the identical decision.
#[test]
fn contract_val_arbiter_02_deterministic_replay() {
    let traces = ::std::vec![
        ("alpha", "cycle between alpha and beta over iface_a"),
        ("beta", "iface_a timeout precondition breached"),
        ("gamma", ""),
    ];
    let first_arbiter = ::global_arbiter::DeterministicArbiter::new();
    let mut first_decisions = ::std::vec::Vec::new();
    for (subject, trace) in &traces {
        first_decisions.push(first_arbiter.adjudicate(subject, trace));
    }
    // A fresh arbiter replays the identical decisions.
    let replay_arbiter = ::global_arbiter::DeterministicArbiter::new();
    for (index, (subject, trace)) in traces.iter().enumerate() {
        let replayed = replay_arbiter.adjudicate(subject, trace);
        ::std::assert_eq!(
            replayed, first_decisions[index],
            "replay must yield the identical decision"
        );
    }
    // Repeating one adjudication on the same arbiter is also stable.
    let repeat = first_arbiter.adjudicate("alpha", "cycle between alpha and beta over iface_a");
    ::std::assert_eq!(repeat, first_decisions[0]);
}

/// A supporting probe for the trace party helper. The regression trace helper
/// names the parent or the module so the engine can gate Tier B escalation.
#[test]
fn contract_val_arbiter_01_supporting_trace_party_detection() {
    let parent_named = ::global_arbiter::trace_references_parties("conflict between a and b", "a", "x");
    ::std::assert!(parent_named);
    let child_named = ::global_arbiter::trace_references_parties("conflict between a and b", "x", "b");
    ::std::assert!(child_named);
    let none_named = ::global_arbiter::trace_references_parties("nothing here", "a", "b");
    ::std::assert!(!none_named);
}
