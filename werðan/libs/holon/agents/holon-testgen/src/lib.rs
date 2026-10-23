//! Characterization test generation agent for the ARIADNE cluster.
//! Exposes one Rig agent with the test-generation preamble wired, plus the
//! domain call that prompts it and splits the suite into test entries.

/// System prompt steering the model toward self-contained characterization tests.
pub const TESTGEN_SYSTEM_PROMPT: &::core::primitive::str = "You are a Rust characterization test generator for a legacy modernization pipeline. \
Output only complete test functions with no markdown and no commentary. \
Every function carries its own test attribute and every function is self contained. \
Tests reference only public items passed in the prompt and use no external crates beyond ::core and ::std. Return only tests.";

/// Builder assembling the test generation Rig agent with role defaults wired.
#[derive(::core::fmt::Debug)]
pub struct HolonTestGenAgentBuilder<M: ::rig::completion::CompletionModel> {
    model: ::core::option::Option<M>,
    temperature: ::core::option::Option<::core::primitive::f64>,
    max_tokens_override: ::core::option::Option<::core::primitive::u64>,
}

impl<M: ::rig::completion::CompletionModel> self::HolonTestGenAgentBuilder<M> {
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

    /// Build the Rig agent with the test-generation preamble and token budget wired.
    pub fn build(self) -> ::rig::agent::Agent<M> {
        let wired_model = match self.model {
            ::core::option::Option::Some(found_model) => found_model,
            ::core::option::Option::None => ::core::panic!("testgen model must be wired before build"),
        };
        let mut agent_builder = ::rig::agent::AgentBuilder::new(wired_model)
            .preamble(self::TESTGEN_SYSTEM_PROMPT)
            .max_tokens(8192);
        if let ::core::option::Option::Some(temperature_value) = self.temperature {
            agent_builder = agent_builder.temperature(temperature_value);
        }
        if let ::core::option::Option::Some(token_budget) = self.max_tokens_override {
            agent_builder = agent_builder.max_tokens(token_budget);
        }
        agent_builder.build()
    }
}

/// Namespace for constructing the test generation agent and invoking it.
pub struct HolonTestGenAgent;

impl self::HolonTestGenAgent {
    /// Start building the test generation Rig agent.
    pub fn builder<M: ::rig::completion::CompletionModel>() -> self::HolonTestGenAgentBuilder<M> {
        self::HolonTestGenAgentBuilder {
            model: ::core::option::Option::None,
            temperature: ::core::option::Option::None,
            max_tokens_override: ::core::option::Option::None,
        }
    }

    /// Prompt the wired agent and split the response into test entries.
    /// On a prompt error the suite is empty. An empty suite records as a
    /// zero pass rate and never fabricates tests.
    pub fn generate_characterization_tests<M: ::rig::completion::CompletionModel + 'static>(
        agent: &::std::sync::Arc<::rig::agent::Agent<M>>,
        problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::std::vec::Vec<::std::string::String> {
        let user_prompt = self::render_test_prompt(problem_model);
        let response_text = self::prompt_blocking(agent, user_prompt);
        if response_text.contains("testgen inference unavailable")
            || response_text.contains("testgen inference failed")
            || response_text.contains("testgen thread panicked")
        {
            return ::std::vec::Vec::new();
        }
        self::split_test_entries(&response_text)
    }
}

/// Render the test generation prompt from the problem model.
pub fn render_test_prompt(
    problem_model: &::holon_blackboard::ProblemModelSlice,
) -> ::std::string::String {
    let context_block = problem_model.render_prompt_context();
    ::std::format!(
        "[PROBLEM CONTEXT]\n{context_block}\n\nWrite characterization test functions that pin the documented behavior now.",
    )
}

/// Split a model response into individual test function entries.
/// A response carrying no test attribute yields zero entries so the
/// suite fabricates no non-test content.
pub fn split_test_entries(response_text: &::core::primitive::str) -> ::std::vec::Vec<::std::string::String> {
    let mut split_entries = ::std::vec::Vec::<::std::string::String>::new();
    let mut current_entry = ::std::string::String::new();
    for source_line in response_text.lines() {
        let trimmed_line = source_line.trim();
        if trimmed_line.starts_with("#[test]") && !current_entry.trim().is_empty() {
            split_entries.push(current_entry.trim().to_string());
            current_entry = ::std::string::String::new();
        }
        current_entry.push_str(source_line);
        current_entry.push('\n');
    }
    if !current_entry.trim().is_empty() && current_entry.contains("#[test]") {
        split_entries.push(current_entry.trim().to_string());
    }
    split_entries
}

/// Run one blocking prompt exchange on a dedicated single-thread runtime.
/// The rig prompt API is async while the ARIADNE controller path is synchronous,
/// so the future is driven to completion on its own worker thread.
fn prompt_blocking<M: ::rig::completion::CompletionModel + 'static>(
    agent: &::std::sync::Arc<::rig::agent::Agent<M>>,
    user_prompt: ::std::string::String,
) -> ::std::string::String {
    let agent_for_call = ::std::sync::Arc::clone(agent);
    let (sender, receiver) = ::std::sync::mpsc::channel::<
        ::core::result::Result<::std::string::String, ::std::string::String>,
    >();
    let handle_result = ::std::thread::Builder::new()
        .name("ariadne-testgen-call".to_string())
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
            return ::std::format!("testgen inference unavailable: {spawn_error}");
        }
    }
    match receiver.recv() {
        ::core::result::Result::Ok(::core::result::Result::Ok(response_text)) => response_text,
        ::core::result::Result::Ok(::core::result::Result::Err(prompt_error)) => {
            ::std::format!("testgen inference failed: {prompt_error}")
        }
        ::core::result::Result::Err(_) => ::std::string::String::from("testgen thread panicked"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_entries_separates_test_functions() {
        let response_text = "#[test]\nfn t_empty() {\n    ::core::assert!(true)\n}\n\n#[test]\nfn t_basic() {\n    ::core::assert_eq!(2 + 2, 4)\n}";
        let suite = self::split_test_entries(response_text);
        ::std::assert_eq!(suite.len(), 2);
        ::std::assert!(suite.first().is_some_and(|entry| entry.contains("t_empty")));
        ::std::assert!(suite.last().is_some_and(|entry| entry.contains("t_basic")));
    }

    #[test]
    fn test_split_entries_drops_non_test_response() {
        let response_text = "fn helper() {\n    ::core::assert!(true)\n}";
        let suite = self::split_test_entries(response_text);
        ::std::assert!(suite.is_empty());
    }

    #[test]
    fn test_split_entries_yields_empty_for_blank_response() {
        let suite = self::split_test_entries("   \n  \n");
        ::std::assert!(suite.is_empty());
    }
}
