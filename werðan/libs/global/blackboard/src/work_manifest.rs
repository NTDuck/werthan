//! Scatter WorkManifest and declarative cross-link tags on the global blackboard.
//! Implements METHODOLOGY.md Algorithm 1 substates S_m.1 and S_m.2 and S_m.3.
//! Substate S_m.1 writes a `WorkManifest` to G. Workers emit `UnresolvedCrossLink`
//! tags in S_m.2. The Reduce stage linker resolves them in S_m.3.

/// One phase work manifest entry: the shard plus its bounded subjects.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct WorkManifest {
    /// Macro phase the manifest describes.
    pub phase: ::global_fsm::MacroState,
    /// Shard subject identifiers dispatched for the phase, in dispatch order.
    pub shard_subjects: ::std::vec::Vec<::std::string::String>,
}

impl self::WorkManifest {
    /// Report whether this manifest lists one shard subject.
    pub fn covers(&self, subject: &::core::primitive::str) -> ::core::primitive::bool {
        self.shard_subjects.iter().any(|item| item == subject)
    }
}

/// Declarative tag that a shard worker emits when the worker hits a dependency
/// outside its own shard scope (METHODOLOGY-DRV.md S_m.2 marker semantics).
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
)]
pub struct UnresolvedCrossLink {
    /// Identifier of the source rule card or shard subject.
    pub src_id: ::std::string::String,
    /// Symbol name the source depends on across the shard boundary.
    pub tgt_symbol: ::std::string::String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_covers_dispatched_subjects() {
        let manifest = self::WorkManifest {
            phase: ::global_fsm::MacroState::S3ExtractRules,
            shard_subjects: ::std::vec!["a".to_string(), "b".to_string()],
        };
        ::std::assert!(manifest.covers("a"));
        ::std::assert!(!manifest.covers("c"));
    }
}
