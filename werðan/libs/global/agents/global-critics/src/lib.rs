//! Rig-backed semantic critic crew per METHODOLOGY.md Definition 3.
//! One Rig agent per phase verdict, plus an LLM arbiter for ambiguous
//! adjudication. Each critic shares the blocking prompt helper and parses
//! the verdict label from the model response.

pub mod workers;

/// System prompt for the assessment critic auditing S1 inventory shards.
pub const ASSESSMENT_SYSTEM_PROMPT: &::core::primitive::str = "You are a semantic critic auditing \
inventory shard outputs for a Rust modernization pipeline. Given the batch entries, check for \
missing duplicate inventory entries and uncovered import or export edges. \
Answer with exactly one label on the first line: PASS, REMEDIATE, or REGRESS. \
On the next lines list each violated invariant name, then a one sentence remediation note.";

/// System prompt for the topology critic auditing S2 coupling maps.
pub const TOPOLOGY_SYSTEM_PROMPT: &::core::primitive::str = "You are a semantic critic auditing \
implicit coupling maps for a Rust modernization pipeline. Given the batch entries, check the \
DAG acyclicity and shard boundary coverage invariants. \
Answer with exactly one label on the first line: PASS, REMEDIATE, or REGRESS. \
On the next lines list each violated invariant name, then a one sentence remediation note.";

/// System prompt for the rule auditor auditing S3 mined rule cards.
pub const RULE_AUDITOR_SYSTEM_PROMPT: &::core::primitive::str = "You are a semantic critic auditing \
mined business rule cards for a Rust modernization pipeline. Given the batch entries, check that \
every rule card carries a verifiable Given When Then triple and cites its source vertex. \
Answer with exactly one label on the first line: PASS, REMEDIATE, or REGRESS. \
On the next lines list each violated invariant name, then a one sentence remediation note.";

/// System prompt for the architecture critic auditing S4 contract stubs.
pub const ARCHITECTURE_SYSTEM_PROMPT: &::core::primitive::str = "You are a semantic critic auditing \
interface contract stubs for a Rust modernization pipeline. Given the batch entries, check that \
every cross-shard dependency edge carries a stubbed contract with preconditions. \
Answer with exactly one label on the first line: PASS, REMEDIATE, or REGRESS. \
On the next lines list each violated invariant name, then a one sentence remediation note.";

/// System prompt for the release auditor auditing S6 integration sign-off.
pub const RELEASE_SYSTEM_PROMPT: &::core::primitive::str = "You are a release auditor auditing \
hardened module sign-offs for a Rust modernization pipeline. Given the batch entries, check that \
every module carries a test suite and a compilation verdict before release. \
Answer with exactly one label on the first line: PASS, REMEDIATE, or REGRESS. \
On the next lines list each violated invariant name, then a one sentence remediation note.";

/// System prompt for the LLM arbiter adjudicating ambiguous traces.
pub const ARBITER_SYSTEM_PROMPT: &::core::primitive::str = "You are the arbiter of a modernization \
pipeline. You receive a subject and a proof trace. Judge whether the trace supports the subject \
claim. Answer with exactly one word: ACCEPT or REJECT.";

/// Rig-backed critic crew dispatching one phase critic per audit batch.
///
/// Rig `Agent` is not cloneable, so the host builds one agent per phase
/// with the matching preamble and hands the set to the crew constructor.
pub struct RigCriticCrew<M: ::rig::completion::CompletionModel> {
    /// Phase critic agents keyed by macro phase label.
    phase_agents: ::std::collections::BTreeMap<::std::string::String, ::std::sync::Arc<::rig::agent::Agent<M>>>,
    /// Agent consulted when a phase carries no dedicated critic agent.
    fallback_agent: ::core::option::Option<::std::sync::Arc<::rig::agent::Agent<M>>>,
}

impl<M: ::rig::completion::CompletionModel> ::core::fmt::Debug for self::RigCriticCrew<M> {
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        formatter
            .debug_struct("RigCriticCrew")
            .field("phase_agents", &self.phase_agents.keys().collect::<::std::vec::Vec<_>>())
            .finish()
    }
}

#[::bon::bon]
impl<M: ::rig::completion::CompletionModel + 'static> self::RigCriticCrew<M> {
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

impl<M: ::rig::completion::CompletionModel + 'static> self::RigCriticCrew<M> {
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

    /// Build one crew agent map from a single model with default phases.
    ///
    /// Convenience constructor: the host usually wants every phase covered
    /// by the same backend model with the phase preambles wired.
    pub fn from_model(
        model: M,
        temperature: ::core::option::Option<::core::primitive::f64>,
    ) -> Self
    where
        M: ::core::clone::Clone,
    {
        let phase_labels = ::std::vec![
            (::std::string::String::from("S1Assess"), self::ASSESSMENT_SYSTEM_PROMPT),
            (::std::string::String::from("S2MapDag"), self::TOPOLOGY_SYSTEM_PROMPT),
            (::std::string::String::from("S3ExtractRules"), self::RULE_AUDITOR_SYSTEM_PROMPT),
            (::std::string::String::from("S4BriefContracts"), self::ARCHITECTURE_SYSTEM_PROMPT),
            (::std::string::String::from("S6SystemVerify"), self::RELEASE_SYSTEM_PROMPT),
        ];
        let mut phase_agents = ::std::collections::BTreeMap::new();
        for (phase_label, preamble) in phase_labels {
            let mut agent_builder = ::rig::agent::AgentBuilder::new(model.clone())
                .preamble(preamble)
                .max_tokens(2048);
            if let ::core::option::Option::Some(temperature_value) = temperature {
                agent_builder = agent_builder.temperature(temperature_value);
            }
            phase_agents.insert(phase_label, ::std::sync::Arc::new(agent_builder.build()));
        }
        Self {
            phase_agents: { phase_agents },
            fallback_agent: ::core::option::Option::None,
        }
    }

    /// Audit one batch through the phase critic and parse the verdict.
    pub fn audit_batch(
        &self,
        phase: ::global_fsm::MacroState,
        batch: &[::std::string::String],
    ) -> ::global_arbiter::CritiqueRecord {
        let agent_for_call = self.phase_agent_for(phase);
        let user_prompt = self::render_audit_prompt(phase, batch);
        let response_text = self::prompt_blocking(&agent_for_call, user_prompt);
        self::parse_critique(&response_text)
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
        ::std::panic!("no critic agent wired for phase {phase_label}");
    }
}

impl<M: ::rig::completion::CompletionModel + 'static> ::global_arbiter::SemanticCritic
    for self::RigCriticCrew<M>
{
    fn audit_batch(
        &self,
        phase: ::global_fsm::MacroState,
        batch: &[::std::string::String],
    ) -> ::global_arbiter::CritiqueRecord {
        self::RigCriticCrew::audit_batch(self, phase, batch)
    }
}
/// Rig-backed arbiter adjudicating a subject against a proof trace.
pub struct RigArbiter<M: ::rig::completion::CompletionModel> {
    /// Rig agent wired with the arbiter preamble.
    agent: ::std::sync::Arc<::rig::agent::Agent<M>>,
}

impl<M: ::rig::completion::CompletionModel> ::core::fmt::Debug for self::RigArbiter<M> {
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        formatter
            .debug_struct("RigArbiter")
            .finish()
    }
}

#[::bon::bon]
impl<M: ::rig::completion::CompletionModel + 'static> self::RigArbiter<M> {
    /// Construct the arbiter around a Rig agent with the arbiter preamble.
    #[builder]
    pub fn builder(agent: ::rig::agent::Agent<M>) -> Self {
        Self {
            agent: ::std::sync::Arc::new(agent),
        }
    }
}

impl<M: ::rig::completion::CompletionModel + 'static> self::RigArbiter<M> {
    /// Construct the arbiter around a Rig agent with the arbiter preamble.
    pub fn new(agent: ::rig::agent::Agent<M>) -> Self {
        Self::builder().agent(agent).build()
    }

    /// Build the arbiter agent from any completion model.
    pub fn build_agent(
        model: M,
        temperature: ::core::option::Option<::core::primitive::f64>,
    ) -> ::rig::agent::Agent<M> {
        let mut agent_builder = ::rig::agent::AgentBuilder::new(model)
            .preamble(self::ARBITER_SYSTEM_PROMPT)
            .max_tokens(512);
        if let ::core::option::Option::Some(temperature_value) = temperature {
            agent_builder = agent_builder.temperature(temperature_value);
        }
        agent_builder.build()
    }

    /// Adjudicate the subject against the trace via the LLM.
    pub fn adjudicate_via_model(
        &self,
        subject: &::core::primitive::str,
        trace: &::core::primitive::str,
    ) -> ::core::primitive::bool {
        let user_prompt = ::std::format!("subject: {subject}\nproof trace:\n{trace}");
        let response_text = self::prompt_blocking(&self.agent, user_prompt);
        let normalized = response_text.trim().to_ascii_uppercase();
        normalized.starts_with("ACCEPT")
    }
}

impl<M: ::rig::completion::CompletionModel + 'static> ::global_arbiter::Arbiter for self::RigArbiter<M> {
    fn adjudicate(
        &self,
        subject: &::core::primitive::str,
        trace: &::core::primitive::str,
    ) -> ::core::primitive::bool {
        self::RigArbiter::adjudicate_via_model(self, subject, trace)
    }
}

/// Render the audit user prompt for one batch.
fn render_audit_prompt(
    phase: ::global_fsm::MacroState,
    batch: &[::std::string::String],
) -> ::std::string::String {
    let phase_label = ::std::format!("{phase:?}");
    let mut batch_body = ::std::string::String::new();
    for entry in batch {
        batch_body.push_str("- ");
        batch_body.push_str(entry);
        batch_body.push('\n');
    }
    let batch_empty = batch.is_empty();
    if batch_empty {
        batch_body.push_str("- (empty batch)\n");
    }
    ::std::format!("phase: {phase_label}\nbatch entries:\n{batch_body}")
}

/// Parse one model response into a critique record.
///
/// The label maps PASS to Pass, REMEDIATE to RemediateLocal, and any other
/// first token to RegressUpstream so malformed output stays conservative.
fn parse_critique(response_text: &::core::primitive::str) -> ::global_arbiter::CritiqueRecord {
    let mut lines = response_text.lines();
    let first_line = lines.next().unwrap_or("").trim().to_ascii_uppercase();
    let mut violated_invariants = ::std::vec::Vec::new();
    for line in lines {
        let trimmed = line.trim();
        let line_present = !trimmed.is_empty();
        if line_present {
            violated_invariants.push(trimmed.to_string());
        }
    }
    let verdict = if first_line.starts_with("PASS") {
        ::global_arbiter::CritiqueVerdict::Pass
    } else if first_line.starts_with("REMEDIATE") {
        ::global_arbiter::CritiqueVerdict::RemediateLocal(violated_invariants.clone())
    } else {
        ::global_arbiter::CritiqueVerdict::RegressUpstream(violated_invariants.clone())
    };
    let remediation_notes = violated_invariants
        .last()
        .cloned()
        .unwrap_or_default();
    ::global_arbiter::CritiqueRecord {
        verdict: { verdict },
        violated_invariants: { violated_invariants },
        remediation_notes: { remediation_notes },
    }
}

/// Prompt one Rig agent on a dedicated blocking runtime thread.
pub(crate) fn prompt_blocking<M: ::rig::completion::CompletionModel + 'static>(
    agent: &::std::sync::Arc<::rig::agent::Agent<M>>,
    user_prompt: ::std::string::String,
) -> ::std::string::String {
    let agent_for_call = ::std::sync::Arc::clone(agent);
    let (sender, receiver) = ::std::sync::mpsc::channel::<
        ::core::result::Result<::std::string::String, ::std::string::String>,
    >();
    let handle_result = ::std::thread::Builder::new()
        .name("critic-prompt-call".to_string())
        .spawn(move || {
            let runtime = ::tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime builds");
            let outcome = runtime.block_on(async {
                ::rig::completion::Prompt::prompt(&*agent_for_call, user_prompt)
                    .await
                    .map_err(|prompt_error| prompt_error.to_string())
            });
            let _ = sender.send(outcome);
        });
    match handle_result {
        ::core::result::Result::Ok(handle) => {
            let _ = handle.join();
        }
        ::core::result::Result::Err(spawn_error) => {
            return ::std::format!("REGRESS\ncritic thread failed: {spawn_error}");
        }
    }
    match receiver.recv() {
        ::core::result::Result::Ok(::core::result::Result::Ok(response_text)) => response_text,
        ::core::result::Result::Ok(::core::result::Result::Err(prompt_error)) => {
            ::std::format!("REGRESS\ncritic inference failed: {prompt_error}")
        }
        ::core::result::Result::Err(_) => ::std::string::String::from("REGRESS\ncritic thread panicked"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_critique_maps_pass_label() {
        let record = self::parse_critique("PASS\nall invariants hold");
        ::std::assert!(matches!(
            record.verdict,
            ::global_arbiter::CritiqueVerdict::Pass
        ));
    }

    #[test]
    fn parse_critique_maps_remediate_label_with_invariants() {
        let record = self::parse_critique("REMEDIATE\ndag_acyclicity\nshard_coverage");
        match record.verdict {
            ::global_arbiter::CritiqueVerdict::RemediateLocal(found) => {
                ::std::assert_eq!(found.len(), 2);
            }
            _ => ::std::panic!("expected remediate verdict"),
        }
        ::std::assert_eq!(record.remediation_notes, "shard_coverage");
    }

    #[test]
    fn parse_critique_defaults_to_regress_on_malformed_output() {
        let record = self::parse_critique("I cannot decide");
        ::std::assert!(matches!(
            record.verdict,
            ::global_arbiter::CritiqueVerdict::RegressUpstream(_)
        ));
    }

    #[test]
    fn render_audit_prompt_carries_phase_and_entries() {
        let batch = ::std::vec![::std::string::String::from("vertex_a")];
        let prompt = self::render_audit_prompt(::global_fsm::MacroState::S2MapDag, &batch);
        ::std::assert!(prompt.contains("S2MapDag"));
        ::std::assert!(prompt.contains("vertex_a"));
    }
}
