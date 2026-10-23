//! Strategy analysis agent for the ARIADNE cluster.
//! Exposes one Rig agent with the strategy preamble wired, plus the domain
//! call that prompts it and parses hypothesis records from the response.

/// System prompt steering the model toward candidate strategy hypotheses.
pub const STRATEGY_SYSTEM_PROMPT: &::core::primitive::str = "You are an architectural strategy analyst for a Rust modernization pipeline. \
Read the problem model and propose competing translation strategies as a JSON array. \
Every element carries exactly the fields strategy_id, description and prior_utility. \
The prior_utility value is a decimal between 0.10 and 0.95. \
Return the JSON array and nothing else. No commentary.";

/// Builder assembling the strategy Rig agent with role defaults wired.
#[derive(::core::fmt::Debug)]
pub struct HolonStrategyAgentBuilder<M: ::rig::completion::CompletionModel> {
    model: ::core::option::Option<M>,
    temperature: ::core::option::Option<::core::primitive::f64>,
    max_tokens_override: ::core::option::Option<::core::primitive::u64>,
}

impl<M: ::rig::completion::CompletionModel> self::HolonStrategyAgentBuilder<M> {
    /// Wire the completion model backing the agent.
    pub fn model(mut self, wired_model: M) -> Self {
        self.model = ::core::option::Option::Some(wired_model);
        self
    }

    /// Override the sampling temperature.
    pub fn temperature(mut self, temperature_value: ::core::primitive::f64) -> Self {
        self.temperature = ::core::option::Option::Some(temperature_value);
        self
    }

    /// Override the completion token budget.
    pub fn max_tokens(mut self, token_budget: ::core::primitive::u64) -> Self {
        self.max_tokens_override = ::core::option::Option::Some(token_budget);
        self
    }

    /// Build the Rig agent with the strategy preamble and token budget wired.
    pub fn build(self) -> ::rig::agent::Agent<M> {
        let wired_model = match self.model {
            ::core::option::Option::Some(found_model) => found_model,
            ::core::option::Option::None => ::core::panic!("strategy model must be wired before build"),
        };
        let mut agent_builder = ::rig::agent::AgentBuilder::new(wired_model)
            .preamble(self::STRATEGY_SYSTEM_PROMPT)
            .max_tokens(4096);
        if let ::core::option::Option::Some(temperature_value) = self.temperature {
            agent_builder = agent_builder.temperature(temperature_value);
        }
        if let ::core::option::Option::Some(token_budget) = self.max_tokens_override {
            agent_builder = agent_builder.max_tokens(token_budget);
        }
        agent_builder.build()
    }
}

/// Error surfaced when strategy inference or parsing produces no usable hypotheses.
/// The ARIADNE contract requires an honest error instead of a silent empty success.
#[derive(::core::fmt::Debug)]
pub struct StrategyAgentError {
    /// Human-readable failure description.
    pub message: ::std::string::String,
}

impl ::core::fmt::Display for self::StrategyAgentError {
    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        ::core::fmt::write(formatter, ::std::format_args!("strategy analysis failed: {}", self.message))
    }
}

impl ::std::error::Error for self::StrategyAgentError {}

/// Namespace for constructing the strategy agent and invoking it.
pub struct HolonStrategyAgent;

impl self::HolonStrategyAgent {
    /// Start building the strategy Rig agent.
    pub fn builder<M: ::rig::completion::CompletionModel>() -> self::HolonStrategyAgentBuilder<M> {
        self::HolonStrategyAgentBuilder {
            model: ::core::option::Option::None,
            temperature: ::core::option::Option::None,
            max_tokens_override: ::core::option::Option::None,
        }
    }

    /// Prompt the wired strategy agent and parse hypothesis records from the response.
    pub fn generate_strategies<M: ::rig::completion::CompletionModel + 'static>(
        agent: &::std::sync::Arc<::rig::agent::Agent<M>>,
        problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::std::vec::Vec<::holon_blackboard::StrategyHypothesis> {
        let user_prompt = self::render_strategy_prompt(problem_model);
        let response_text = self::prompt_blocking(agent, user_prompt);
        self::parse_hypotheses(&response_text)
    }
}

/// Render the user prompt from the problem model slice.
pub fn render_strategy_prompt(
    problem_model: &::holon_blackboard::ProblemModelSlice,
) -> ::std::string::String {
    let context_block = problem_model.render_prompt_context();
    ::std::format!(
        "[PROBLEM CONTEXT]\n{context_block}\n\nPropose the competing modernization strategies as the JSON array now.",
    )
}

/// Parse the model response into hypothesis records.
/// Malformed or unparsable responses yield an empty vector so the controller
/// treats inference failure as a spent expansion rather than a fabricated plan.
fn parse_hypotheses(response_text: &::core::primitive::str) -> ::std::vec::Vec<::holon_blackboard::StrategyHypothesis> {
    let trimmed_text = response_text.trim();
    let fenced_start = trimmed_text.find('[');
    let array_text = match fenced_start {
        ::core::option::Option::Some(start_index) => {
            let array_end = trimmed_text.rfind(']');
            match array_end {
                ::core::option::Option::Some(end_index) if end_index > start_index => {
                    &trimmed_text[start_index..=end_index]
                }
                _ => trimmed_text,
            }
        }
        ::core::option::Option::None => trimmed_text,
    };
    let parsed_entries = ::serde_json::from_str::<
        ::std::vec::Vec<self::StrategyHypothesisEntry>,
    >(array_text);
    let entries = match parsed_entries {
        ::core::result::Result::Ok(found_entries) => found_entries,
        ::core::result::Result::Err(_) => return ::std::vec::Vec::new(),
    };
    let mut hypotheses = ::std::vec::Vec::<::holon_blackboard::StrategyHypothesis>::new();
    for entry in entries {
        let hypothesis = ::holon_blackboard::StrategyHypothesis {
            strategy_id: { entry.strategy_id },
            description: { entry.description },
            prior_utility: { entry.prior_utility.clamp(0.10, 0.95) },
            instantiated: false,
        };
        hypotheses.push(hypothesis);
    }
    hypotheses
}

/// Wire format of one hypothesis record inside the model response.
#[derive(::serde::Deserialize)]
struct StrategyHypothesisEntry {
    strategy_id: ::std::string::String,
    description: ::std::string::String,
    prior_utility: ::core::primitive::f64,
}

/// Run one blocking prompt exchange on a dedicated single-thread runtime.
/// The rig prompt API is async while the ARIADNE controller path is synchronous,
/// so the future runs to completion on its own worker thread.
fn prompt_blocking<M: ::rig::completion::CompletionModel + 'static>(
    agent: &::std::sync::Arc<::rig::agent::Agent<M>>,
    user_prompt: ::std::string::String,
) -> ::std::string::String {
    let agent_for_call = ::std::sync::Arc::clone(agent);
    let (sender, receiver) = ::std::sync::mpsc::channel::<
        ::core::result::Result<::std::string::String, ::std::string::String>,
    >();
    let handle_result = ::std::thread::Builder::new()
        .name("ariadne-strategy-call".to_string())
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
            return ::std::format!("strategy inference unavailable: {spawn_error}");
        }
    }
    match receiver.recv() {
        ::core::result::Result::Ok(::core::result::Result::Ok(response_text)) => response_text,
        ::core::result::Result::Ok(::core::result::Result::Err(prompt_error)) => {
            ::std::format!("strategy inference failed: {prompt_error}")
        }
        ::core::result::Result::Err(_) => ::std::string::String::from("strategy thread panicked"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hypotheses_from_plain_array() {
        let response_text = r#"[{"strategy_id":"strat-direct","description":"direct structural port","prior_utility":0.70}]"#;
        let hypotheses = self::parse_hypotheses(response_text);
        ::std::assert_eq!(hypotheses.len(), 1);
        let first = &hypotheses[0];
        ::std::assert_eq!(first.strategy_id, "strat-direct");
        ::std::assert!((first.prior_utility - 0.70).abs() < 1e-6);
        ::std::assert!(!first.instantiated);
    }

    #[test]
    fn test_parse_hypotheses_from_fenced_array() {
        let response_text = "Here is the plan:\n```json\n[{\"strategy_id\":\"s1\",\"description\":\"d1\",\"prior_utility\":0.4}]\n```";
        let hypotheses = self::parse_hypotheses(response_text);
        ::std::assert_eq!(hypotheses.len(), 1);
        ::std::assert_eq!(hypotheses[0].strategy_id, "s1");
    }

    #[test]
    fn test_parse_hypotheses_clamps_utilities() {
        let response_text = r#"[{"strategy_id":"s-low","description":"underfloor","prior_utility":0.01},{"strategy_id":"s-high","description":"overceiling","prior_utility":1.30}]"#;
        let hypotheses = self::parse_hypotheses(response_text);
        ::std::assert_eq!(hypotheses.len(), 2);
        ::std::assert!((hypotheses[0].prior_utility - 0.10).abs() < 1e-6);
        ::std::assert!((hypotheses[1].prior_utility - 0.95).abs() < 1e-6);
    }

    #[test]
    fn test_parse_hypotheses_rejects_prose() {
        let response_text = "I cannot propose strategies today.";
        let hypotheses = self::parse_hypotheses(response_text);
        ::std::assert!(hypotheses.is_empty());
    }
}
