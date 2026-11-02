//! Patch and fault records slice B_PR per METHODOLOGY.md Definition 2.
//! Holds distilled compiler and test fault localizations indexing failed
//! MCTS rollouts for root-cause triage.

/// Classification of diagnostic failure causes.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::cmp::PartialEq, ::core::cmp::Eq, ::serde::Serialize, ::serde::Deserialize)]
pub enum FaultCause {
    /// Local implementation fault repairable via internal code patch.
    Intrinsic,
    /// Upstream interface contract insufficiency requiring parent re-negotiation.
    ExtrinsicContract,
    /// Undocumented runtime dependency cycle requiring graph re-condensation.
    ExtrinsicCycle,
}

/// Localized diagnostic fault record from compiler or test execution.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct FaultRecord {
    /// Concise summary of diagnostic failure.
    pub summary: ::std::string::String,
    /// Raw diagnostic logs or compiler stack trace.
    pub diagnostic_trace: ::std::string::String,
    /// Cause classification.
    pub cause: ::core::option::Option<self::FaultCause>,
}

/// Patch and fault records slice indexing compiler and test failures.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct PatchRecordsSlice {
    /// Diagnostic failure records from failed rollouts.
    pub records: ::std::vec::Vec<self::FaultRecord>,
}

impl self::PatchRecordsSlice {
    /// Create empty patch records slice.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record localized diagnostic fault card.
    pub fn record_fault(&mut self, fault: self::FaultRecord) {
        self.records.push(fault);
    }

    /// Retrieve latest diagnostic fault record if available.
    pub fn latest_fault(&self) -> ::core::option::Option<&self::FaultRecord> {
        self.records.last()
    }

    /// Count recorded faults in slice.
    pub fn fault_count(&self) -> ::core::primitive::usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fault_recording_semantics() {
        let mut slice = self::PatchRecordsSlice::new();
        let first = self::FaultRecord {
            summary: "type mismatch".to_string(),
            diagnostic_trace: "expected u32 found i32".to_string(),
            cause: ::core::option::Option::Some(self::FaultCause::Intrinsic),
        };
        slice.record_fault(first);
        let second = self::FaultRecord {
            summary: "contract conflict".to_string(),
            diagnostic_trace: "missing parent precondition".to_string(),
            cause: ::core::option::Option::Some(self::FaultCause::ExtrinsicContract),
        };
        slice.record_fault(second);
        ::std::assert_eq!(slice.fault_count(), 2);
        let latest = slice.latest_fault();
        ::std::assert!(latest.is_some());
        ::std::assert_eq!(latest.map(|f| f.summary.clone()), ::core::option::Option::Some("contract conflict".to_string()));
        ::std::assert_eq!(slice.records[0].cause, ::core::option::Option::Some(self::FaultCause::Intrinsic));
    }
}
