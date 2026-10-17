//! Council execution DAG and orchestrator coordination using Rig agents.

/// Node representing a specialist member in the council execution DAG.
#[derive(::bon::Builder, ::core::clone::Clone)]
pub struct CouncilNode {
    /// Member identifier name.
    #[builder(into)]
    pub name: ::std::string::String,

    /// Specialist council domain role.
    #[builder(into)]
    pub role: ::std::string::String,
}

/// Directed acyclic graph coordinating problem framing, council deliberation, and summary synthesis.
#[derive(::bon::Builder, ::core::clone::Clone)]
pub struct CouncilDag {
    /// Root orchestrator agent.
    pub orchestrator: ::rig::agent::Agent,

    /// Council member execution nodes.
    pub members: ::std::vec::Vec<self::CouncilNode>,
}

impl self::CouncilDag {
    /// Frame the user prompt into a focused council consultation mandate for a specific member.
    pub fn frame_member_prompt(
        &self,
        member: &self::CouncilNode,
        problem: &::core::primitive::str,
    ) -> ::std::string::String {
        let name = &member.name;
        let role = &member.role;
        ::std::format!(
            "You are {name}, the {role} council member. \
            Analyze and propose a solution for the following challenge from your specialist perspective: \
            \n\n{problem}"
        )
    }

    /// Formulate the synthesis prompt for the orchestrator to summarize council solutions into a final decision.
    pub fn frame_synthesis_prompt(
        &self,
        problem: &::core::primitive::str,
        deliberations: &[(::std::string::String, ::std::string::String)],
    ) -> ::std::string::String {
        let joined_proposals = deliberations
            .iter()
            .map(|(name, solution)| ::std::format!("### Council Member {name} Proposal:\n{solution}"))
            .collect::<::std::vec::Vec<_>>()
            .join("\n\n");

        ::std::format!(
            "You are the Council Orchestrator. Review the proposals from your council members regarding the problem: \
            \n\nChallenge: {problem}\n\n{joined_proposals}\n\n\
            Synthesize these findings, reconcile trade-offs, and provide a unified, definitive resolution."
        )
    }
}

/// Build the orchestrator agent for council orchestration.
pub fn build_orchestrator(
    orchestrator_model: ::rig::driver::DynModel<::rig::operation::Completion>,
) -> ::rig::agent::Agent {
    ::rig::agent::AgentBuilder::new(orchestrator_model)
        .name("orchestrator")
        .preamble(
            "You are the Council Orchestrator. You coordinate council deliberations, formulate problem briefs, and \
             synthesize council member findings into unified decisions.",
        )
        .max_tokens(1024)
        .build()
}
