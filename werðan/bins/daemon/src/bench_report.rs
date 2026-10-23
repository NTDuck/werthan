//! Campaign report schema for the THESEUS campaign runner (VAL-DAEMON-01/03/04).
//! The report carries campaign identity, per-module outcomes, the FIFO event
//! log, hardened/failed totals, and an honest backend configuration echo.


/// One module outcome in the campaign report.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize, ::core::cmp::PartialEq)]
pub struct ModuleOutcome {
    /// DAG vertex identifier of the module.
    pub module_id: ::std::string::String,
    /// Final per-module micro-state reached by the campaign.
    pub final_micro_state: ::global_blackboard::ModuleMicroState,
    /// Reference to the hardened artifact for this module, when hardened.
    pub artifact_reference: ::core::option::Option<::std::string::String>,
    /// Blocker description when the module failed to harden.
    pub blocker: ::core::option::Option<::std::string::String>,
}

/// Honest echo of the backend configuration used by the campaign.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize, ::core::cmp::PartialEq)]
pub struct BackendEcho {
    /// Configured inference base URL, or the stub marker when no URL exists.
    pub backend_url: ::std::string::String,
    /// Configured model identifier, when present.
    pub model: ::core::option::Option<::std::string::String>,
    /// True when the campaign ran against the deterministic stub solver.
    pub stub_backend: ::core::primitive::bool,
}

/// Campaign runner report: the observable result of one THESEUS campaign.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize,
)]
pub struct CampaignReport {
    /// Campaign identifier from the configuration.
    pub campaign_id: ::std::string::String,
    /// One outcome per DAG vertex.
    pub module_outcomes: ::std::vec::Vec<self::ModuleOutcome>,
    /// FIFO event log: one entry per recorded transition in dispatch order.
    pub event_log: ::std::vec::Vec<::std::string::String>,
    /// Count of modules that reached the hardened micro-state.
    pub hardened_count: ::core::primitive::usize,
    /// Count of modules that failed to harden.
    pub failed_count: ::core::primitive::usize,
    /// Echo of the backend configuration actually used.
    pub backend: self::BackendEcho,
    /// Solver dispatch count reported by the engine.
    pub solver_dispatches: ::core::primitive::usize,
    /// Failure description or none on success.
    pub error: ::core::option::Option<::std::string::String>,
}

/// Run the deterministic stub campaign over the seeded board.
///
/// Event log entries derive from the engine transition log, which records
/// one entry per recorded transition in dispatch order.
pub fn run_stub_campaign(
    campaign_id: &::core::primitive::str,
    board: ::global_blackboard::GlobalBlackboard,
) -> self::CampaignReport {
    let solver = ::global_engine::StubModuleSolver::new();
    let boxed_solver = ::std::boxed::Box::new(solver);
    let backend = self::BackendEcho {
        backend_url: "stub://local".to_string(),
        model: ::core::option::Option::None,
        stub_backend: true,
    };
    let mut engine = ::global_engine::MacroEngine::new(boxed_solver);
    engine.configure(board);
    let run_res = engine.run();
    match run_res {
        ::core::result::Result::Ok(stop) => {
            let report = self::report_from_engine(campaign_id, &engine, backend);
            let finished = ::core::matches!(stop, ::global_engine::RunStop::Terminated);
            if finished {
                report
            } else {
                let parked = "campaign parked on human gate before termination".to_string();
                let mut parked_report = report;
                parked_report.error = ::core::option::Option::Some(parked);
                parked_report
            }
        },
        ::core::result::Result::Err(engine_error) => {
            self::failed_report_from_error(campaign_id, &engine, backend, &engine_error)
        },
    }
}

pub fn failed_report_from_error(
    campaign_id: &::core::primitive::str,
    engine: &::global_engine::MacroEngine,
    backend: self::BackendEcho,
    engine_error: &::global_engine::MacroEngineError,
) -> self::CampaignReport {
    let mut report = self::report_from_engine(campaign_id, engine, backend);
    for outcome in report.module_outcomes.iter_mut() {
        let not_hardened = !::core::matches!(outcome.final_micro_state, ::global_blackboard::ModuleMicroState::M4Hardened);
        if not_hardened {
            outcome.blocker = ::core::option::Option::Some(::std::format!("{engine_error}"));
        }
    }
    report.error = ::core::option::Option::Some(::std::format!("{engine_error}"));
    report.failed_count = report
        .module_outcomes
        .iter()
        .filter(|outcome| !::core::matches!(outcome.final_micro_state, ::global_blackboard::ModuleMicroState::M4Hardened))
        .count();
    report.hardened_count = report.module_outcomes.len() - report.failed_count;
    report
}

pub fn report_from_engine(
    campaign_id: &::core::primitive::str,
    engine: &::global_engine::MacroEngine,
    backend: self::BackendEcho,
) -> self::CampaignReport {
    let board = engine.board();
    let state_vector = engine.state_vector();
    let mut module_ids = state_vector
        .module_states
        .keys()
        .cloned()
        .collect::<::std::vec::Vec<::std::string::String>>();
    module_ids.sort();
    let mut module_outcomes = ::std::vec::Vec::new();
    let mut hardened_count = 0;
    let mut failed_count = 0;
    for module_id in module_ids {
        let fallback_micro = ::global_blackboard::ModuleMicroState::M0Blocked;
        let observed_state = state_vector
            .module_states
            .get(&module_id)
            .copied()
            .unwrap_or(fallback_micro);
        let hardened = ::core::matches!(observed_state, ::global_blackboard::ModuleMicroState::M4Hardened);
        let artifact_reference = if hardened {
            board.artifacts.get(&module_id).and_then(|slot| slot.as_ref()).map(|_| module_id.clone())
        } else {
            ::core::option::Option::None
        };
        let blocker = if hardened {
            ::core::option::Option::None
        } else {
            ::core::option::Option::Some("module did not reach the hardened micro-state".to_string())
        };
        if hardened {
            hardened_count += 1;
        } else {
            failed_count += 1;
        }
        let outcome_state = observed_state;
        module_outcomes.push(self::ModuleOutcome {
            module_id,
            final_micro_state: ::core::clone::Clone::clone(&outcome_state),
            artifact_reference,
            blocker,
        });
    }
    let mut event_log = ::std::vec::Vec::new();
    for (logged_module, logged_state) in engine.transition_log() {
        let entry = ::std::format!("{logged_module}: {logged_state:?}");
        event_log.push(entry);
    }
    let solver_dispatches = engine.solver_dispatches();
    self::CampaignReport {
        campaign_id: { campaign_id.to_string() },
        module_outcomes,
        event_log,
        hardened_count,
        failed_count,
        backend,
        solver_dispatches,
        error: ::core::option::Option::None,
    }
}
