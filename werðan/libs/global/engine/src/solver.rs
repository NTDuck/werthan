//! Per-module solver contracts and stubs per Algorithm 2 in METHODOLOGY.md.

/// Per-module solver contract from Algorithm 2 in METHODOLOGY.md.
pub trait ModuleSolver: ::core::fmt::Debug + ::core::marker::Send + ::core::marker::Sync {
    /// Dispatch one module synthesis or regression attempt and return the outcome.
    fn solve(
        &mut self,
        module_id: &::core::primitive::str,
        problem_model: &::holon_blackboard::ProblemModelSlice,
        mode: ::holon_fsm::ActivationMode,
    ) -> self::SolverOutcome;
}

/// Outcome of one module solver dispatch.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub enum SolverOutcome {
    /// Synthesis succeeded with verified code and tests.
    Hardened {
        /// Verified target code.
        target_code: ::std::string::String,
        /// Verified characterization tests.
        test_suite: ::std::vec::Vec<::std::string::String>,
    },
    /// Extrinsic contract conflict on an upstream parent.
    ContractConflict {
        /// Upstream parent identifier.
        parent_id: ::std::string::String,
        /// Interface delta specification.
        delta_spec: ::std::string::String,
        /// Failure proof trace.
        proof_trace: ::std::string::String,
    },
    /// Undocumented cycle found across module boundaries.
    CycleFound {
        /// Cycle edge pairs.
        cycle_edges: ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
        /// Failure proof trace.
        proof_trace: ::std::string::String,
    },
}

/// Deterministic stub solver that emits trivial verified candidates per module.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default)]
pub struct StubModuleSolver {
    invocation_log: ::std::collections::BTreeMap<::std::string::String, ::core::primitive::usize>,
}

#[::bon::bon]
impl self::StubModuleSolver {
    /// Construct a stub module solver using a bon builder.
    #[builder]
    pub fn builder(
        invocation_log: ::core::option::Option<::std::collections::BTreeMap<::std::string::String, ::core::primitive::usize>>,
    ) -> Self {
        let invocation_log = invocation_log.unwrap_or_default();
        Self { invocation_log }
    }

    /// Construct a new empty stub module solver.
    pub fn new() -> Self {
        Self {
            invocation_log: ::std::collections::BTreeMap::new(),
        }
    }

    /// Return the number of solver dispatches recorded for a module.
    pub fn invocation_count(&self, module_id: &::core::primitive::str) -> ::core::primitive::usize {
        *self.invocation_log.get(module_id).unwrap_or(&0)
    }
}

impl self::ModuleSolver for self::StubModuleSolver {
    fn solve(
        &mut self,
        module_id: &::core::primitive::str,
        _problem_model: &::holon_blackboard::ProblemModelSlice,
        mode: ::holon_fsm::ActivationMode,
    ) -> self::SolverOutcome {
        let entry = self.invocation_log.entry(module_id.to_string()).or_insert(0);
        *entry += 1;
        let mode_tag = match mode {
            ::holon_fsm::ActivationMode::Initial => "initial",
            ::holon_fsm::ActivationMode::WarmStartRegression => "warm",
        };
        let target_code = ::std::format!("fn {}_hardened() -> u32 {{ {} }}", module_id, mode_tag);
        let test_suite = ::std::vec![::std::format!("assert {}_hardened() >= 0", module_id)];
        self::SolverOutcome::Hardened {
            target_code,
            test_suite,
        }
    }
}
