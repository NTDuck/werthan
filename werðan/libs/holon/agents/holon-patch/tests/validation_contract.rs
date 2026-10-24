//! Validation contract tests for holon-patch.
//! Covers VAL-APATCH-01, VAL-APATCH-02 and VAL-APATCH-03.
//! Contracts derive from docs/specs/validation-contracts.md and outrank implementation tests.

#[test]
fn contract_val_apatch_01_repair_passthrough() {
    // Given candidate code plus fault records and problem model, the agent
    // returns repaired candidate source. Unchanged code in, unchanged code out.
    let candidate_code = "pub fn repairable_probe() -> u8 { 1 }";
    let fault = ::holon_blackboard::FaultRecord {
        summary: "type mismatch".to_string(),
        diagnostic_trace: "expected u8 found i32".to_string(),
        cause: ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic),
    };
    let problem_model = ::holon_blackboard::ProblemModelSlice::new();
    let response_text = candidate_code.to_string();
    let agent = build_patch_agent(response_text);
    let repaired = ::holon_patch::HolonPatchAgent::repair_code(
        &agent,
        candidate_code,
        &fault,
        &problem_model,
    );
    ::std::assert_eq!(repaired, candidate_code);
}

#[test]
fn contract_val_apatch_02_fault_informed_prompt() {
    // The rendered prompt embeds the fault records so repairs target the
    // distilled diagnostics: fault summary, trace, candidate, problem model.
    let candidate_code = "pub fn decode_frame() -> u8 { 2 }";
    let fault = ::holon_blackboard::FaultRecord {
        summary: "contract precondition violated".to_string(),
        diagnostic_trace: "error: missing export parent_alpha".to_string(),
        cause: ::core::option::Option::Some(::holon_blackboard::FaultCause::ExtrinsicContract),
    };
    let mut problem_model = ::holon_blackboard::ProblemModelSlice::new();
    problem_model.add_rule(::holon_blackboard::BusinessRuleCard {
        rule_id: "RULE_PT_02".to_string(),
        given: "frame bytes".to_string(),
        when: "decoder runs".to_string(),
        then: "frame decodes".to_string(),
        citation: "legacy/decoder.c:3".to_string(),
    });
    let prompt = ::holon_patch::render_repair_prompt(candidate_code, &fault, &problem_model);
    ::std::assert!(prompt.contains("contract precondition violated"));
    ::std::assert!(prompt.contains("error: missing export parent_alpha"));
    ::std::assert!(prompt.contains(candidate_code));
    ::std::assert!(prompt.contains("RULE_PT_02"));
    ::std::assert!(prompt.contains("[FAULT SUMMARY]"));
    ::std::assert!(prompt.contains("[DIAGNOSTIC TRACE]"));
}

#[test]
fn contract_val_apatch_03_honest_failure_sentinel() {
    // Inference failure surfaces a SYNTAX_ERROR sentinel, never a silent
    // success. The sentinel is language neutral and names the repair fault.
    let problem_model = ::holon_blackboard::ProblemModelSlice::new();
    let fault = ::holon_blackboard::FaultRecord::default();
    let agent = build_failing_patch_agent();
    let repaired = ::holon_patch::HolonPatchAgent::repair_code(
        &agent,
        "pub fn broken() -> u8 { 3 }",
        &fault,
        &problem_model,
    );
    ::std::assert!(repaired.contains("SYNTAX_ERROR"));
    ::std::assert!(repaired.contains("ariadne llm repair fault"));
    ::std::assert!(!repaired.contains("pub fn broken"));
}

/// Deterministic completion model returning one constant outcome.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
struct RecordingPatchModel {
    response_outcome: ::core::result::Result<::std::string::String, ::std::string::String>,
}

impl ::rig::completion::CompletionModel for self::RecordingPatchModel {
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
        let mapped = self
            .response_outcome
            .clone()
            .map_err(::rig::completion::CompletionError::ProviderError);
        let response = mapped.map(|raw_text| {
            let content_item = ::rig::message::AssistantContent::text(raw_text);
            let choice_bundle = ::rig::OneOrMany::one(content_item);
            ::rig::completion::CompletionResponse {
                choice: { choice_bundle },
                raw_response: "contract".to_string(),
            }
        });
        ::core::future::ready(response)
    }
}

fn build_patch_agent(
    response_text: ::std::string::String,
) -> ::std::sync::Arc<::rig::agent::Agent<RecordingPatchModel>> {
    let model = RecordingPatchModel {
        response_outcome: ::core::result::Result::Ok(response_text),
    };
    let agent_builder = ::holon_patch::HolonPatchAgent::builder();
    let wired_builder = agent_builder.model(model);
    ::std::sync::Arc::new(wired_builder.build())
}

fn build_failing_patch_agent() -> ::std::sync::Arc<::rig::agent::Agent<RecordingPatchModel>> {
    let model = RecordingPatchModel {
        response_outcome: ::core::result::Result::Err("transport down".to_string()),
    };
    let agent_builder = ::holon_patch::HolonPatchAgent::builder();
    let wired_builder = agent_builder.model(model);
    ::std::sync::Arc::new(wired_builder.build())
}
