//! Validation contract tests for holon-evaluator.
//! Covers VAL-AEVAL-01, VAL-AEVAL-02, VAL-AEVAL-03, VAL-AEVAL-04 and VAL-AEVAL-05.
//! Contracts derive from docs/specs/validation-contracts.md and outrank implementation tests.

/// Recording harness counting invocations to pin stage order.
#[derive(::core::fmt::Debug)]
struct RecordingHarness {
    call_count: ::std::sync::atomic::AtomicUsize,
    output: self::HarnessOutputChoice,
}

#[derive(::core::clone::Clone, ::core::fmt::Debug)]
enum HarnessOutputChoice {
    Compiling,
    Failing,
}

impl self::RecordingHarness {
    fn new(output: self::HarnessOutputChoice) -> Self {
        let harness_call_count = ::std::sync::atomic::AtomicUsize::new(0);
        Self { call_count: { harness_call_count }, output }
    }
}

impl ::holon_evaluator::ExecutionHarness for self::RecordingHarness {
    fn execute_candidate(
        &self,
        _module_id: &::core::primitive::str,
        _candidate_code: &::core::primitive::str,
        _test_suite: &[::std::string::String],
        _target_language: &::core::primitive::str,
    ) -> ::holon_evaluator::HarnessExecutionOutput {
        self.call_count.fetch_add(1, ::std::sync::atomic::Ordering::SeqCst);
        match self.output {
            self::HarnessOutputChoice::Compiling => {
                let full_log = ::std::string::String::from("test result: ok. 2 passed");
                ::holon_evaluator::HarnessExecutionOutput {
                    compiles: true,
                    pass_rate: 1.0,
                    raw_logs: { full_log },
                }
            }
            self::HarnessOutputChoice::Failing => {
                let failing_log = ::std::string::String::from("test result: FAILED. 0 passed, 1 failed");
                ::holon_evaluator::HarnessExecutionOutput {
                    compiles: true,
                    pass_rate: 0.0,
                    raw_logs: { failing_log },
                }
            }
        }
    }
}

/// Recording distiller counting invocations to pin stage order.
#[derive(::core::fmt::Debug)]
struct RecordingDistiller {
    call_count: ::std::sync::atomic::AtomicUsize,
    audit_score: ::core::primitive::f64,
}

impl ::holon_evaluator::DiagnosticDistiller for self::RecordingDistiller {
    fn distill_failure(
        &self,
        raw_logs: &::core::primitive::str,
        _candidate_code: &::core::primitive::str,
        _problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::holon_blackboard::FaultRecord {
        self.call_count.fetch_add(1, ::std::sync::atomic::Ordering::SeqCst);
        ::holon_blackboard::FaultRecord {
            summary: "distilled".to_string(),
            diagnostic_trace: { raw_logs.to_string() },
            cause: ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic),
        }
    }

    fn audit_rules(
        &self,
        _candidate_code: &::core::primitive::str,
        _problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::core::primitive::f64 {
        self.audit_score
    }
}

#[test]
fn contract_val_aeval_01_stage_order_fixed() {
    // Stage 3a (harness) runs first and stage 3b (distiller) runs only on
    // failure or for harden scoring on success.
    let evaluator = build_recording_evaluator(self::HarnessOutputChoice::Compiling, 0.5);
    let problem_model = ::holon_blackboard::ProblemModelSlice::new();
    let empty_suite = ::std::vec::Vec::<::std::string::String>::new();
    let result = evaluator.evaluate("module_stage", "pub fn ok() -> u8 { 1 }", &empty_suite, &problem_model);
    ::std::assert_eq!(evaluator.harness.call_count.load(::std::sync::atomic::Ordering::SeqCst), 1);
    ::std::assert_eq!(evaluator.distiller.call_count.load(::std::sync::atomic::Ordering::SeqCst), 0);
    ::std::assert!(result.fault_record.is_none());
    // On harness failure the distiller runs for fault classification.
    let failing_evaluator = build_recording_evaluator(self::HarnessOutputChoice::Failing, 0.0);
    let failing_result = failing_evaluator.evaluate(
        "module_stage",
        "pub fn broken() -> u8 { 1 }",
        &empty_suite,
        &problem_model,
    );
    ::std::assert_eq!(
        failing_evaluator.harness.call_count.load(::std::sync::atomic::Ordering::SeqCst),
        1
    );
    ::std::assert_eq!(
        failing_evaluator.distiller.call_count.load(::std::sync::atomic::Ordering::SeqCst),
        1
    );
    ::std::assert!(failing_result.fault_record.is_some());
}

#[test]
fn contract_val_aeval_05_unified_result() {
    // The evaluator composes harness and distiller output into one result
    // carrying compile indicator, pass rate, harden score and fault records.
    let evaluator = build_recording_evaluator(self::HarnessOutputChoice::Compiling, 0.75);
    let mut problem_model = ::holon_blackboard::ProblemModelSlice::new();
    problem_model.add_rule(::holon_blackboard::BusinessRuleCard {
        rule_id: "RULE_EV_05".to_string(),
        given: "input".to_string(),
        when: "processing".to_string(),
        then: "output".to_string(),
        citation: "legacy/core.c:1".to_string(),
    });
    let suite = ::std::vec![::std::string::String::from("assert: value")];
    let result = evaluator.evaluate("module_unified", "pub fn value() -> u8 { 7 }", &suite, &problem_model);
    ::std::assert!(result.compiles);
    ::std::assert_eq!(result.test_pass_rate, 1.0);
    ::std::assert_eq!(result.hardening_score, 0.75);
    ::std::assert!(result.fault_record.is_none());
}

fn build_recording_evaluator(
    output_choice: self::HarnessOutputChoice,
    audit_score: ::core::primitive::f64,
) -> ::holon_evaluator::TwoStageEvaluator<self::RecordingHarness, self::RecordingDistiller> {
    let harness = self::RecordingHarness::new(output_choice);
    let distiller = RecordingDistiller { call_count: ::std::sync::atomic::AtomicUsize::new(0), audit_score };
    ::holon_evaluator::TwoStageEvaluator::builder()
        .harness(harness)
        .distiller(distiller)
        .build()
}

#[test]
fn contract_val_aeval_02_harness_reports_toolchain_honestly() {
    // The default harness compiles and runs tests through a real cargo
    // workspace and reports failure honestly when the toolchain is absent.
    let workspace_root = ::std::path::PathBuf::from("/tmp/evaluator-contract-ws-missing");
    let source_file = ::std::path::PathBuf::from("src/lib.rs");
    let configured = ::holon_evaluator::CargoHarness::with_timeouts(
        workspace_root,
        source_file,
        60,
        30,
    );
    let syntax_broken = ::std::string::String::from("pub fn broken( { unclosed");
    let suite = ::std::vec![::std::string::String::from("assert: broken")];
    let output = ::holon_evaluator::ExecutionHarness::execute_candidate(&configured, "module_eval", &syntax_broken, &suite, "rust");
    // A missing workspace cannot fabricate a pass.
    ::std::assert!(!output.compiles);
    ::std::assert_eq!(output.pass_rate, 0.0);
}

#[test]
fn contract_val_aeval_03_fault_taxonomy() {
    // Every failure classifies into exactly one taxonomy cause and
    // unrecognized labels classify as intrinsic.
    let distiller = ::holon_evaluator::DefaultDiagnosticDistiller::new();
    let problem_model = ::holon_blackboard::ProblemModelSlice::new();
    // Intrinsic: plain compile defect.
    let intrinsic_logs = "error[E0308]: mismatched types";
    let intrinsic_fault = ::holon_evaluator::DiagnosticDistiller::distill_failure(&distiller, intrinsic_logs, "fn demo() {}", &problem_model);
    ::std::assert_eq!(
        intrinsic_fault.cause,
        ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic)
    );
    // Extrinsic contract: parent contract insufficiency signal.
    let contract_logs = "error: contract conflict detected against parent export";
    let contract_fault = ::holon_evaluator::DiagnosticDistiller::distill_failure(&distiller, contract_logs, "fn demo() {}", &problem_model);
    ::std::assert_eq!(
        contract_fault.cause,
        ::core::option::Option::Some(::holon_blackboard::FaultCause::ExtrinsicContract)
    );
    // Extrinsic cycle: undocumented dependency cycle signal.
    let cycle_logs = "error: cyclic dependency graph loop detected";
    let cycle_fault = ::holon_evaluator::DiagnosticDistiller::distill_failure(&distiller, cycle_logs, "fn demo() {}", &problem_model);
    ::std::assert_eq!(
        cycle_fault.cause,
        ::core::option::Option::Some(::holon_blackboard::FaultCause::ExtrinsicCycle)
    );
    // Unrecognized label falls back to intrinsic.
    let unknown_logs = "some totally unrecognized diagnostic text";
    let unknown_fault = ::holon_evaluator::DiagnosticDistiller::distill_failure(&distiller, unknown_logs, "fn demo() {}", &problem_model);
    ::std::assert_eq!(
        unknown_fault.cause,
        ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic)
    );
}

#[test]
fn contract_val_aeval_04_rule_audit_score_bounds() {
    // The parity critic yields the fraction of Given/When/Then rules
    // satisfied. The score is 1.0 when all terms appear. The score is 0.0
    // for empty rule sets. Partial coverage stays within the unit interval.
    let distiller = ::holon_evaluator::DefaultDiagnosticDistiller::new();
    let empty_model = ::holon_blackboard::ProblemModelSlice::new();
    let empty_score = ::holon_evaluator::DiagnosticDistiller::audit_rules(&distiller, "fn demo() {}", &empty_model);
    ::std::assert_eq!(empty_score, 0.0);
    let mut rule_model = ::holon_blackboard::ProblemModelSlice::new();
    rule_model.add_rule(::holon_blackboard::BusinessRuleCard {
        rule_id: "RULE_EV_01".to_string(),
        given: "frame bytes".to_string(),
        when: "decoder invoked".to_string(),
        then: "frame decodes".to_string(),
        citation: "legacy/decoder.c:2".to_string(),
    });
    let full_code = "fn decoder_invoked(frame_bytes) { frame_decodes(frame_bytes) }";
    let full_score = ::holon_evaluator::DiagnosticDistiller::audit_rules(&distiller, full_code, &rule_model);
    ::std::assert_eq!(full_score, 1.0);
    let empty_code = "fn unrelated() {}";
    let partial_score = ::holon_evaluator::DiagnosticDistiller::audit_rules(&distiller, empty_code, &rule_model);
    ::std::assert!((0.0..=1.0).contains(&partial_score));
    ::std::assert!(partial_score < 1.0);
}

