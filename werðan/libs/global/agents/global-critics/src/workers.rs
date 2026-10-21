//! Rig-backed LLM worker crew implementing the stateless [`crate::PhaseWorker`]
//! contract per METHODOLOGY.md Algorithm 1 substate S_m.2.
//!
//! Each phase dispatch prompts one Rig agent with the hydrated activation
//! envelope content and parses the response into the typed phase output.
//! The engine structural guard remains the verifier: parse failures fall
//! back to the deterministic no-op output so the guard observes the real
//! board gap instead of a fabricated pass.

use crate::prompt_blocking;

/// System prompt family steering worker role responses toward one typed
/// output block. The model answers with a `type:` header followed by
/// key value lines, mirroring the deterministic slice summary format.
pub const WORKER_SYSTEM_PROMPT: &::core::primitive::str = "You are a stateless worker agent in a \
legacy modernization orchestrator. You receive one phase role, one shard subject, and a blackboard \
slice. Propose the next artifact for the shard. Answer with a type header line and key value \
lines. For S1Assess emit type: assessment with file_count and risk note lines. For S2MapDag emit \
type: coupling with parent-child edge lines. For S3ExtractRules emit type: rules with rule lines. \
For S4BriefContracts emit type: contracts with contract stub lines. For other phases emit \
type: none. Emit no prose outside the lines.";

/// Rig-backed worker crew dispatching one phase worker per shard.
pub struct RigWorkerCrew<M: ::rig::completion::CompletionModel> {
    /// Phase worker agents keyed by macro phase label.
    phase_agents: ::std::collections::BTreeMap<
        ::std::string::String,
        ::std::sync::Arc<::rig::agent::Agent<M>>,
    >,
    /// Agent consulted when a phase carries no dedicated worker agent.
    fallback_agent: ::core::option::Option<::std::sync::Arc<::rig::agent::Agent<M>>>,
}

impl<M: ::rig::completion::CompletionModel> ::core::fmt::Debug for self::RigWorkerCrew<M> {
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        formatter
            .debug_struct("RigWorkerCrew")
            .field("phase_agents", &self.phase_agents.keys().collect::<::std::vec::Vec<_>>())
            .finish()
    }
}

#[::bon::bon]
impl<M: ::rig::completion::CompletionModel + 'static> self::RigWorkerCrew<M> {
    /// Construct the crew from prebuilt phase agents.
    #[builder]
    pub fn builder(
        phase_agents: ::std::collections::BTreeMap<
            ::std::string::String,
            ::std::sync::Arc<::rig::agent::Agent<M>>,
        >,
        fallback_agent: ::core::option::Option<::std::sync::Arc<::rig::agent::Agent<M>>>,
    ) -> Self {
        Self {
            phase_agents,
            fallback_agent,
        }
    }
}

impl<M: ::rig::completion::CompletionModel + 'static> self::RigWorkerCrew<M> {
    /// Construct the crew from prebuilt phase agents.
    pub fn new(
        phase_agents: ::std::collections::BTreeMap<
            ::std::string::String,
            ::std::sync::Arc<::rig::agent::Agent<M>>,
        >,
        fallback_agent: ::core::option::Option<::std::sync::Arc<::rig::agent::Agent<M>>>,
    ) -> Self {
        Self::builder()
            .phase_agents(phase_agents)
            .maybe_fallback_agent(fallback_agent)
            .build()
    }

    /// Build one crew agent map from a single model with the worker preamble.
    pub fn from_model(
        model: M,
        temperature: ::core::option::Option<::core::primitive::f64>,
    ) -> Self
    where
        M: ::core::clone::Clone,
    {
        let phase_labels = ::std::vec![
            ::std::string::String::from("S0IntentPreflight"),
            ::std::string::String::from("S1Assess"),
            ::std::string::String::from("S2MapDag"),
            ::std::string::String::from("S3ExtractRules"),
            ::std::string::String::from("S4BriefContracts"),
            ::std::string::String::from("S6SystemVerify"),
        ];
        let mut phase_agents = ::std::collections::BTreeMap::new();
        for phase_label in phase_labels {
            let mut agent_builder = ::rig::agent::AgentBuilder::new(model.clone())
                .preamble(self::WORKER_SYSTEM_PROMPT)
                .max_tokens(2048);
            if let ::core::option::Option::Some(temperature_value) = temperature {
                agent_builder = agent_builder.temperature(temperature_value);
            }
            phase_agents.insert(phase_label, ::std::sync::Arc::new(agent_builder.build()));
        }
        Self {
            phase_agents,
            fallback_agent: ::core::option::Option::None,
        }
    }

    /// Resolve the agent for one phase, falling back to any wired agent.
    fn phase_agent_for(
        &self,
        phase: ::global_fsm::MacroState,
    ) -> ::std::sync::Arc<::rig::agent::Agent<M>> {
        let phase_label = ::std::format!("{phase:?}");
        let resolved = self.phase_agents.get(&phase_label).cloned();
        let resolved_present = resolved.is_some();
        if resolved_present {
            return resolved.expect("phase agent checked above");
        }
        let fallback = self.fallback_agent.clone();
        let fallback_present = fallback.is_some();
        if fallback_present {
            return fallback.expect("fallback agent checked above");
        }
        ::std::panic!("no worker agent wired for phase {phase_label}");
    }

    /// Execute one phase shard through the LLM and parse the typed output.
    pub fn execute_via_model(
        &self,
        phase: ::global_fsm::MacroState,
        shard_subject: &::core::primitive::str,
        envelope: &::global_workers::TaskActivationEnvelope,
        board: &::global_blackboard::GlobalBlackboard,
    ) -> ::global_workers::PhaseWorkOutput {
        let agent_for_call = self.phase_agent_for(phase);
        let user_prompt = self::render_worker_prompt(phase, shard_subject, envelope, board);
        let response_text = self::prompt_blocking(&agent_for_call, user_prompt);
        self::parse_worker_output(phase, &response_text)
    }
}

impl<M: ::rig::completion::CompletionModel + 'static> ::global_workers::PhaseWorker
    for self::RigWorkerCrew<M>
{
    fn execute(
        &self,
        phase: ::global_fsm::MacroState,
        shard_subject: &::core::primitive::str,
        envelope: &::global_workers::TaskActivationEnvelope,
        board: &::global_blackboard::GlobalBlackboard,
    ) -> ::global_workers::PhaseWorkOutput {
        self::RigWorkerCrew::execute_via_model(self, phase, shard_subject, envelope, board)
    }
}

/// Render the worker user prompt from the hydrated envelope content.
fn render_worker_prompt(
    phase: ::global_fsm::MacroState,
    shard_subject: &::core::primitive::str,
    envelope: &::global_workers::TaskActivationEnvelope,
    board: &::global_blackboard::GlobalBlackboard,
) -> ::std::string::String {
    let phase_label = ::std::format!("{phase:?}");
    let mode_label = match envelope.mode {
        ::holon_fsm::ActivationMode::Initial => "initial",
        ::holon_fsm::ActivationMode::WarmStartRegression => "warm start regression",
    };
    let mut prompt_body = ::std::string::String::new();
    prompt_body.push_str(&::std::format!("phase: {phase_label}\n"));
    prompt_body.push_str(&::std::format!("mode: {mode_label}\n"));
    prompt_body.push_str(&::std::format!("shard subject: {shard_subject}\n"));
    prompt_body.push_str(&::std::format!(
        "slice summary:\n{}\n",
        envelope.slice_summary
    ));
    if let ::core::option::Option::Some(delta) = &envelope.delta_directive {
        prompt_body.push_str(&::std::format!("delta directive from failed guard: {delta}\n"));
    }
    let tool_names = envelope.bound_tools.join(", ");
    prompt_body.push_str(&::std::format!("bound read-only tools: {tool_names}\n"));
    let dialect = &board.inventory.dialect;
    prompt_body.push_str(&::std::format!(
        "target dialect: {} edition {}\n",
        dialect.target_language, dialect.target_edition
    ));
    prompt_body
}

/// Parse one model response into the typed phase output.
///
/// Unrecognized or malformed responses map to the deterministic no-op
/// output for the phase so the structural guard observes the real gap.
fn parse_worker_output(
    phase: ::global_fsm::MacroState,
    response_text: &::core::primitive::str,
) -> ::global_workers::PhaseWorkOutput {
    let mut output_kind = ::std::string::String::new();
    let mut file_count = ::core::primitive::usize::MIN;
    let mut risk_notes = ::std::vec::Vec::new();
    let mut discovered_edges = ::std::vec::Vec::new();
    for line in response_text.lines() {
        let trimmed = line.trim();
        let line_present = !trimmed.is_empty();
        if !line_present {
            continue;
        }
        if let ::core::option::Option::Some(kind) = trimmed.strip_prefix("type:") {
            output_kind = kind.trim().to_ascii_lowercase();
            continue;
        }
        if let ::core::option::Option::Some(count_text) = trimmed.strip_prefix("file_count:") {
            let parsed = count_text.trim().parse::<::core::primitive::usize>();
            if let ::core::result::Result::Ok(count_value) = parsed {
                file_count = count_value;
            }
            continue;
        }
        if let ::core::option::Option::Some(note) = trimmed.strip_prefix("risk:") {
            risk_notes.push(note.trim().to_string());
            continue;
        }
        if let ::core::option::Option::Some(edge_text) = trimmed.strip_prefix("edge:") {
            let halves = edge_text.split("->").collect::<::std::vec::Vec<_>>();
            let edge_wellformed = halves.len() == 2;
            if edge_wellformed {
                let parent_id = halves[0].trim().to_string();
                let child_id = halves[1].trim().to_string();
                discovered_edges.push((parent_id, child_id));
            }
            continue;
        }
    }
    let assessment_kind = output_kind == "assessment";
    let coupling_kind = output_kind == "coupling";
    if assessment_kind {
        let report = ::global_workers::AssessmentReport {
            shard_subject: ::std::string::String::new(),
            file_count,
            risk_notes: ::core::clone::Clone::clone(&risk_notes),
        };
        let reports = ::std::vec![report];
        let synthesized = ::global_workers::RiskMatrix {
            file_count,
            risk_notes: ::core::clone::Clone::clone(&risk_notes),
        };
        return ::global_workers::PhaseWorkOutput::Assessment {
            reports,
            synthesized,
        };
    }
    if coupling_kind {
        return ::global_workers::PhaseWorkOutput::Coupling { discovered_edges };
    }
    match phase {
        ::global_fsm::MacroState::S1Assess => {
            let report = ::global_workers::AssessmentReport::default();
            let reports = ::std::vec![report];
            let synthesized = ::global_workers::RiskMatrix::default();
            ::global_workers::PhaseWorkOutput::Assessment {
                reports,
                synthesized,
            }
        }
        ::global_fsm::MacroState::S2MapDag => ::global_workers::PhaseWorkOutput::Coupling {
            discovered_edges: ::std::vec::Vec::new(),
        },
        _ => ::global_workers::PhaseWorkOutput::Skipped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_worker_output_coupling_edges() {
        let response = "type: coupling\nedge: alpha->beta\nedge: beta->gamma\n";
        let output = self::parse_worker_output(::global_fsm::MacroState::S2MapDag, response);
        let edge_pairs = match output {
            ::global_workers::PhaseWorkOutput::Coupling { discovered_edges } => discovered_edges,
            other => {
                let detail = ::std::format!("unexpected output {other:?}");
                ::std::panic!("{detail}");
            }
        };
        ::std::assert_eq!(edge_pairs.len(), 2);
        ::std::assert_eq!(edge_pairs[0], ("alpha".to_string(), "beta".to_string()));
    }

    #[test]
    fn test_parse_worker_output_malformed_falls_back_noop() {
        let response = "I propose refactoring the module thoroughly.";
        let output = self::parse_worker_output(::global_fsm::MacroState::S3ExtractRules, response);
        ::std::assert!(::core::matches!(
            output,
            ::global_workers::PhaseWorkOutput::Skipped
        ));
    }

    #[test]
    fn test_parse_worker_output_assessment_counts_and_notes() {
        let response = "type: assessment\nfile_count: 12\nrisk: unmapped dialect\n";
        let output = self::parse_worker_output(::global_fsm::MacroState::S1Assess, response);
        let matrix = match output {
            ::global_workers::PhaseWorkOutput::Assessment { synthesized, .. } => synthesized,
            other => {
                let detail = ::std::format!("unexpected output {other:?}");
                ::std::panic!("{detail}");
            }
        };
        ::std::assert_eq!(matrix.file_count, 12);
        ::std::assert_eq!(matrix.risk_notes.len(), 1);
    }
}
