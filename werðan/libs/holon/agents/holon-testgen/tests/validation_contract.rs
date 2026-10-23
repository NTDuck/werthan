//! Validation contract tests for holon-testgen.
//! Covers VAL-ATEST-01, VAL-ATEST-02 and VAL-ATEST-03.
//! Contracts derive from docs/specs/validation-contracts.md and outrank implementation tests.

#[test]
fn contract_val_atest_01_blind_to_candidate_code() {
    // The test agent inputs carry no candidate code. The prompt renderer only
    // accepts a problem-model slice, so the API itself blocks candidate leakage.
    let mut problem_model = ::holon_blackboard::ProblemModelSlice::new();
    problem_model.add_rule(::holon_blackboard::BusinessRuleCard {
        rule_id: "RULE_TG_01".to_string(),
        given: "empty input".to_string(),
        when: "parser invoked".to_string(),
        then: "parse error surfaced".to_string(),
        citation: "legacy/parser.c:5".to_string(),
    });
    let rendered = ::holon_testgen::render_test_prompt(&problem_model);
    ::std::assert!(rendered.contains("RULE_TG_01"));
    // A candidate marker string never enters the prompt because the renderer
    // signature accepts no candidate parameter.
    let leaked_marker = "CANDIDATE_SECRET_MARKER";
    ::std::assert!(!rendered.contains(leaked_marker));
}

#[test]
fn contract_val_atest_02_self_contained_test_entries() {
    // Given a problem model, the agent returns self-contained test functions
    // and each carries its own test attribute with no shared harness state.
    let problem_model = ::holon_blackboard::ProblemModelSlice::new();
    let response_text = "#[test]\nfn t_roundtrip() {\n    ::core::assert!(true)\n}\n\n#[test]\nfn t_empty_input() {\n    ::core::assert_eq!(1, 1)\n}".to_string();
    let agent = build_testgen_agent(response_text);
    let suite = ::holon_testgen::HolonTestGenAgent::generate_characterization_tests(
        &agent,
        &problem_model,
    );
    ::std::assert_eq!(suite.len(), 2);
    for test_entry in &suite {
        ::std::assert!(test_entry.starts_with("#[test]"));
        ::std::assert!(test_entry.contains("fn t_"));
    }
    // The controller seeds B_CT from these entries.
    let mut characterization = ::holon_blackboard::CharacterizationTestsSlice::new();
    for test_entry in suite {
        characterization.add_test_case(test_entry);
    }
    ::std::assert!(characterization.is_populated());
    ::std::assert_eq!(characterization.test_count(), 2);
}

#[test]
fn contract_val_atest_03_split_semantics_no_fabricated_tests() {
    // Multiple test functions split into individual entries. A response with
    // no test functions yields zero entries, so the agent fabricates none.
    let multi_test = "#[test]\nfn t_first() {\n}\n\n#[test]\nfn t_second() {\n}";
    let split = ::holon_testgen::split_test_entries(multi_test);
    ::std::assert_eq!(split.len(), 2);
    ::std::assert!(split[0].contains("t_first"));
    ::std::assert!(split[1].contains("t_second"));
    // No test functions in the response means zero entries.
    let no_tests = "struct Config {\n    field: u32\n}";
    let empty_split = ::holon_testgen::split_test_entries(no_tests);
    ::std::assert!(empty_split.is_empty());
}

/// Deterministic completion model returning one fixed response.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
struct RecordingTestGenModel {
    response_text: ::std::string::String,
}

impl ::rig::completion::CompletionModel for self::RecordingTestGenModel {
    type Response = ::std::string::String;

    fn completion(
        &self,
        _request: ::rig::completion::CompletionRequest,
    ) -> impl ::core::future::Future<
        Output = ::core::result::Result<
            ::rig::completion::CompletionResponse<Self::Response>,
            ::rig::completion::CompletionError,
        >,
    > + ::core::marker::Send {
        let raw_text = self.response_text.clone();
        let content_item = ::rig::message::AssistantContent::text(raw_text);
        let choice_bundle = ::rig::OneOrMany::one(content_item);
        let outcome = ::rig::completion::CompletionResponse {
            choice: { choice_bundle },
            raw_response: "contract".to_string(),
        };
        ::core::future::ready(::core::result::Result::Ok(outcome))
    }
}

fn build_testgen_agent(
    response_text: ::std::string::String,
) -> ::std::sync::Arc<::rig::agent::Agent<RecordingTestGenModel>> {
    let model = RecordingTestGenModel { response_text };
    let agent_builder = ::holon_testgen::HolonTestGenAgent::builder();
    let wired_builder = agent_builder.model(model);
    ::std::sync::Arc::new(wired_builder.build())
}
