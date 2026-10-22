//! Specialist agent subagent pool for MCTS cluster controller.
//! Each generative role is one Rig agent built by its holon crate builder.

/// Consolidated specialist subagent pool over one completion model.
pub struct SubagentPool<M: ::rig::completion::CompletionModel> {
    /// Strategy analysis agent.
    pub strategy_agent: ::std::sync::Arc<::rig::agent::Agent<M>>,
    /// Code generation agent.
    pub code_gen_agent: ::std::sync::Arc<::rig::agent::Agent<M>>,
    /// Test generation agent.
    pub test_gen_agent: ::std::sync::Arc<::rig::agent::Agent<M>>,
    /// Surgical repair agent.
    pub patch_agent: ::std::sync::Arc<::rig::agent::Agent<M>>,
}

impl<M: ::rig::completion::CompletionModel> ::core::clone::Clone for self::SubagentPool<M> {
    fn clone(&self) -> Self {
        Self {
            strategy_agent: ::std::sync::Arc::clone(&self.strategy_agent),
            code_gen_agent: ::std::sync::Arc::clone(&self.code_gen_agent),
            test_gen_agent: ::std::sync::Arc::clone(&self.test_gen_agent),
            patch_agent: ::std::sync::Arc::clone(&self.patch_agent),
        }
    }
}

impl<M: ::rig::completion::CompletionModel> ::core::fmt::Debug for self::SubagentPool<M> {
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        formatter
            .debug_struct("SubagentPool")
            .field("roles", &["strategy", "codegen", "testgen", "patch"])
            .finish()
    }
}

#[::bon::bon]
impl<M: ::rig::completion::CompletionModel + 'static> self::SubagentPool<M> {
    /// Build the specialist pool from four independently constructed Rig agents.
    /// Every role agent assembles through its holon crate builder at the call
    /// site and enters the pool as a shared handle.
    #[builder]
    pub fn builder(
        strategy_agent: ::std::sync::Arc<::rig::agent::Agent<M>>,
        code_gen_agent: ::std::sync::Arc<::rig::agent::Agent<M>>,
        test_gen_agent: ::std::sync::Arc<::rig::agent::Agent<M>>,
        patch_agent: ::std::sync::Arc<::rig::agent::Agent<M>>,
    ) -> Self {
        Self {
            strategy_agent,
            code_gen_agent,
            test_gen_agent,
            patch_agent,
        }
    }
}
