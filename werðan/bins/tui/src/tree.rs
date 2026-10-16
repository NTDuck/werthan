//! Hierarchical agent tree construction using Rig models.

/// Build the 3-level agent tree consisting of workers, managers, and root
/// orchestrator.
pub fn build_agent_tree(
    orchestrator_model: ::rig::driver::DynModel<::rig::operation::Completion>,
    workers: &[crate::app::Worker],
) -> ::rig::agent::Agent {
    let mut specialist_tools = ::std::vec::Vec::new();
    for worker in workers {
        if let ::core::result::Result::Ok(tool) = worker.agent.clone().into_tool() {
            specialist_tools.push(tool);
        }
    }

    let discovery_mgr = ::rig::agent::AgentBuilder::new(orchestrator_model.clone())
        .name("discovery_mgr")
        .preamble("You are the Discovery Manager.")
        .build();

    let migration_mgr = ::rig::agent::AgentBuilder::new(orchestrator_model.clone())
        .name("migration_mgr")
        .preamble("You are the Migration Manager.")
        .dynamic_tools(specialist_tools)
        .build();

    let mut manager_tools = ::std::vec::Vec::new();
    if let ::core::result::Result::Ok(disc_tool) = discovery_mgr.into_tool() {
        manager_tools.push(disc_tool);
    }
    if let ::core::result::Result::Ok(migr_tool) = migration_mgr.into_tool() {
        manager_tools.push(migr_tool);
    }

    ::rig::agent::AgentBuilder::new(orchestrator_model)
        .name("root_orchestrator")
        .preamble("You are the Root Orchestrator coordinating all managers and workers.")
        .dynamic_tools(manager_tools)
        .build()
}
