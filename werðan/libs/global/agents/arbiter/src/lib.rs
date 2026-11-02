//! Global arbiter and semantic critic agents per METHODOLOGY.md Definition 3.

/// Verdict emitted by the semantic critic for an audit batch.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub enum CritiqueVerdict {
    /// Batch accepted with no findings.
    Pass,
    /// Local remediation required inside the current phase.
    RemediateLocal(::std::vec::Vec<::std::string::String>),
    /// Regression required back to an upstream phase.
    RegressUpstream(::std::vec::Vec<::std::string::String>),
}

/// Audit record produced by the semantic critic over a shard batch.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct CritiqueRecord {
    /// Stage-two verdict for the batch.
    pub verdict: self::CritiqueVerdict,
    /// Invariant names violated by the batch.
    pub violated_invariants: ::std::vec::Vec<::std::string::String>,
    /// Human-readable or LLM remediation notes.
    pub remediation_notes: ::std::string::String,
}

/// Semantic critic contract from METHODOLOGY.md section 3 stage-two guard column.
pub trait SemanticCritic: ::core::fmt::Debug + ::core::marker::Send + ::core::marker::Sync {
    /// Audit one batch of artifact identifiers for the given phase.
    fn audit_batch(&self, phase: ::global_fsm::MacroState, batch: &[::std::string::String]) -> self::CritiqueRecord;
}

/// Default deterministic critic that accepts every batch or emits a configured verdict.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct DeterministicCritic {
    verdict: self::CritiqueVerdict,
}

#[::bon::bon]
impl self::DeterministicCritic {
    /// Construct a deterministic critic using a bon builder.
    #[builder]
    pub fn builder(
        verdict: ::core::option::Option<self::CritiqueVerdict>,
    ) -> Self {
        let verdict = verdict.unwrap_or(self::CritiqueVerdict::Pass);
        Self { verdict }
    }

    /// Construct a deterministic critic that passes all batches.
    pub fn new() -> Self {
        Self {
            verdict: self::CritiqueVerdict::Pass,
        }
    }
}

impl ::core::default::Default for self::DeterministicCritic {
    fn default() -> Self {
        Self::new()
    }
}

impl self::SemanticCritic for self::DeterministicCritic {
    fn audit_batch(&self, _phase: ::global_fsm::MacroState, _batch: &[::std::string::String]) -> self::CritiqueRecord {
        self::CritiqueRecord {
            verdict: self.verdict.clone(),
            violated_invariants: ::std::vec::Vec::new(),
            remediation_notes: ::std::string::String::new(),
        }
    }
}

/// LLM arbiter interface that adjudicates an ambiguous subject against a trace.
pub trait Arbiter: ::core::fmt::Debug + ::core::marker::Send + ::core::marker::Sync {
    /// Return a decision for the subject given the proof trace.
    fn adjudicate(&self, subject: &::core::primitive::str, trace: &::core::primitive::str) -> ::core::primitive::bool;
}

/// Default deterministic arbiter that verifies trace content is nonempty.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct DeterministicArbiter {
    accept_nonempty_trace: ::core::primitive::bool,
}

#[::bon::bon]
impl self::DeterministicArbiter {
    /// Construct a deterministic arbiter using a bon builder.
    #[builder]
    pub fn builder(
        accept_nonempty_trace: ::core::option::Option<::core::primitive::bool>,
    ) -> Self {
        let accept_nonempty_trace = accept_nonempty_trace.unwrap_or(true);
        Self { accept_nonempty_trace }
    }

    /// Construct a default deterministic arbiter.
    pub fn new() -> Self {
        Self {
            accept_nonempty_trace: true,
        }
    }
}

impl ::core::default::Default for self::DeterministicArbiter {
    fn default() -> Self {
        Self::new()
    }
}

impl self::Arbiter for self::DeterministicArbiter {
    fn adjudicate(&self, _subject: &::core::primitive::str, trace: &::core::primitive::str) -> ::core::primitive::bool {
        if self.accept_nonempty_trace {
            !trace.is_empty()
        } else {
            false
        }
    }
}

/// Report whether a proof trace names either party of a regression.
pub fn trace_references_parties(
    trace: &::core::primitive::str,
    parent_id: &::core::primitive::str,
    module_id: &::core::primitive::str,
) -> ::core::primitive::bool {
    trace.contains(parent_id) || trace.contains(module_id)
}

/// Conjoin the shard batch verdicts of one phase's stage-two audit round into
/// the phase semantic guard outcome (VAL-CRITIC-03). The phase passes only when
/// every shard batch verdict is PASS. One REGRESS_UPSTREAM dominates all local
/// remediations. Otherwise the first REJECT_LOCAL verdict stands so the engine
/// re-dispatches exactly the offending shard batch.
pub fn conjunction_verdict(records: &[self::CritiqueRecord]) -> self::CritiqueVerdict {
    let mut first_local = ::core::option::Option::None;
    for record in records {
        match &record.verdict {
            self::CritiqueVerdict::RegressUpstream(reasons) => {
                let cloned_reasons = ::core::clone::Clone::clone(reasons);
                return self::CritiqueVerdict::RegressUpstream(cloned_reasons);
            }
            self::CritiqueVerdict::RemediateLocal(reasons) => {
                if first_local.is_none() {
                    let cloned_reasons = ::core::clone::Clone::clone(reasons);
                    first_local = ::core::option::Option::Some(self::CritiqueVerdict::RemediateLocal(cloned_reasons));
                }
            }
            self::CritiqueVerdict::Pass => {}
        }
    }
    first_local.unwrap_or(self::CritiqueVerdict::Pass)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_critic_builder() {
        let critic = self::DeterministicCritic::builder().build();
        let items = ::std::vec!["foo.rs".to_string()];
        let record = critic.audit_batch(::global_fsm::MacroState::S1Assess, &items);
        ::std::assert_eq!(record.verdict, self::CritiqueVerdict::Pass);
    }

    #[test]
    fn test_deterministic_arbiter_builder() {
        let arbiter = self::DeterministicArbiter::builder()
            .accept_nonempty_trace(true)
            .build();
        ::std::assert!(arbiter.adjudicate("cycle", "trace data"));
        ::std::assert!(!arbiter.adjudicate("cycle", ""));
    }

    #[test]
    fn test_trace_references_parties() {
        let parent_matches = self::trace_references_parties("parent->child conflict", "parent", "other");
        ::std::assert!(parent_matches);
        let child_matches = self::trace_references_parties("parent->child conflict", "other", "child");
        ::std::assert!(child_matches);
        let none_matches = self::trace_references_parties("unrelated", "parent", "child");
        ::std::assert!(!none_matches);
    }
}
