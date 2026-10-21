//! Deterministic context assembler function.
//! Formalized per §2 Definition 4 in `METHODOLOGY.md`.

use self::super::envelope::ActivationMode;
use self::super::envelope::AgentRole;
use self::super::envelope::DirectiveTrace;
use self::super::envelope::GraphSlice;
use self::super::envelope::TaskActivationEnvelope;
use self::super::envelope::ThinkingBudget;
use self::super::envelope::ToolBindings;

/// Deterministic Context Assembler for hydrating stateless agent envelopes.
pub struct ContextAssembler;

impl self::ContextAssembler {
    /// Hydrate a TaskActivationEnvelope deterministically from role, mode, slice, and directive trace.
    pub fn hydrate(
        role: self::AgentRole,
        mode: self::ActivationMode,
        slice: self::GraphSlice,
        directive: self::DirectiveTrace,
    ) -> self::TaskActivationEnvelope {
        let tools = if role.is_test_generator() {
            // Adversarial test generator has discovery tools but read-isolation from candidate code
            self::ToolBindings::discovery_roster()
        } else {
            match role {
                self::AgentRole::AriadneStrategy
                | self::AgentRole::AriadneGenerator
                | self::AgentRole::AriadnePatcher
                | self::AgentRole::AriadneEvaluator => self::ToolBindings::mcts_roster(),
                _ => self::ToolBindings::discovery_roster(),
            }
        };

        let thinking_budget = match mode {
            self::ActivationMode::CriticRetry => self::ThinkingBudget::High,
            _ => role.default_thinking_budget(),
        };

        self::TaskActivationEnvelope {
            mode,
            role,
            slice,
            directive,
            tools,
            thinking_budget,
        }
    }
}
