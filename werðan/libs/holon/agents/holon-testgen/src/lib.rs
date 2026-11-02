//! Characterization test generation agent for the ARIADNE cluster.
//! Exposes one Rig agent with the test-generation preamble wired, plus the
//! domain call that prompts it and splits the suite into test entries.

/// System prompt steering the model toward self-contained characterization tests.
pub fn testgen_system_prompt(target_language: &::core::primitive::str) -> ::std::string::String {
    ::std::format!(
        "You are a {target_language} characterization test generator for a legacy modernization pipeline. \
Output only complete test functions with no markdown and no commentary. \
Every test follows the idiomatic {target_language} test conventions and every test is self contained. \
Tests reference only public items passed in the prompt and use no external dependencies. Return only tests."
    )
}

/// Builder assembling the test generation Rig agent with role defaults wired.
#[derive(::core::fmt::Debug)]
pub struct HolonTestGenAgentBuilder<M: ::rig::completion::CompletionModel> {
    model: ::core::option::Option<M>,
    temperature: ::core::option::Option<::core::primitive::f64>,
    max_tokens_override: ::core::option::Option<::core::primitive::u64>,
    target_language: ::std::string::String,
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

    /// Set the target language the test-generation preamble advertises.
    pub fn target_language(mut self, language: ::std::string::String) -> Self {
        self.target_language = language;
        self
    }

    /// Build the Rig agent with the test-generation preamble and token budget wired.
    pub fn build(self) -> ::rig::agent::Agent<M> {
        let wired_model = match self.model {
            ::core::option::Option::Some(found_model) => found_model,
            ::core::option::Option::None => ::core::panic!("testgen model must be wired before build"),
        };
        let preamble_text = self::testgen_system_prompt(&self.target_language);
        let mut agent_builder = ::rig::agent::AgentBuilder::new(wired_model)
            .preamble(&preamble_text)
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
            target_language: "rust".to_string(),
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
        self::split_test_entries_for(&response_text, &problem_model.target_language)
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
/// The rust branch splits on the test attribute and yields zero entries when
/// the response carries none so the suite fabricates no non-test content.
/// Other languages split on top-level declaration openers.
pub fn split_test_entries_for(
    response_text: &::core::primitive::str,
    target_language: &::core::primitive::str,
) -> ::std::vec::Vec<::std::string::String> {
    let rust_language_selected = target_language == "rust";
    let opener_prefixes: &[&::core::primitive::str] = if rust_language_selected {
        &["#[test]"]
    } else {
        &["fn ", "function ", "def ", "func "]
    };
    let mut split_entries = ::std::vec::Vec::<::std::string::String>::new();
    let mut current_entry = ::std::string::String::new();
    for source_line in response_text.lines() {
        // The rust branch splits on the attribute wherever it appears after
        // indentation, while other languages split only on column-zero openers.
        let boundary_hit = if rust_language_selected {
            source_line.trim().starts_with("#[test]")
        } else {
            opener_prefixes.iter().any(|opener_prefix| source_line.starts_with(opener_prefix))
        };
        if boundary_hit && !current_entry.trim().is_empty() {
            split_entries.push(current_entry.trim().to_string());
            current_entry = ::std::string::String::new();
        }
        current_entry.push_str(source_line);
        current_entry.push('\n');
    }
    let final_entry_guard = if rust_language_selected {
        current_entry.contains("#[test]")
    } else {
        opener_prefixes.iter().any(|opener_prefix| current_entry.contains(opener_prefix))
    };
    if !current_entry.trim().is_empty() && final_entry_guard {
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
        let suite = self::split_test_entries_for(response_text, "rust");
        ::std::assert_eq!(suite.len(), 2);
        ::std::assert!(suite.first().is_some_and(|entry| entry.contains("t_empty")));
        ::std::assert!(suite.last().is_some_and(|entry| entry.contains("t_basic")));
    }

    #[test]
    fn test_split_entries_preserves_attribute_entry_shape() {
        let suite = self::split_test_entries_for("#[test]\nfn a() {}", "rust");
        ::std::assert_eq!(suite, ::std::vec!["#[test]\nfn a() {}".to_string()]);
    }

    #[test]
    fn test_split_entries_drops_non_test_response() {
        let response_text = "fn helper() {\n    ::core::assert!(true)\n}";
        let suite = self::split_test_entries_for(response_text, "rust");
        ::std::assert!(suite.is_empty());
    }

    #[test]
    fn test_split_entries_yields_empty_for_blank_response() {
        let suite = self::split_test_entries_for("   \n  \n", "rust");
        ::std::assert!(suite.is_empty());
    }

    #[test]
    fn test_split_entries_generic_language_splits_on_declarations() {
        let response_text = "def test_one():\n    assert True\n\ndef test_two():\n    assert 1 + 1 == 2";
        let suite = self::split_test_entries_for(response_text, "python");
        ::std::assert_eq!(suite.len(), 2);
        ::std::assert!(suite.first().is_some_and(|entry| entry.contains("test_one")));
        ::std::assert!(suite.last().is_some_and(|entry| entry.contains("test_two")));
    }

    #[test]
    fn test_split_entries_generic_language_drops_declarative_free_text() {
        let suite = self::split_test_entries_for("Some prose without declarations", "python");
        ::std::assert!(suite.is_empty());
    }
}
