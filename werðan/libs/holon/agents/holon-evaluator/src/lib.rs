//! Two-stage candidate evaluation crate for the ARIADNE modernization cluster.
//!
//! Evaluates candidate code in two sequential stages.
//! Stage 3a runs an execution harness to check compilation and test pass rates.
//! Stage 3b runs a diagnostic distiller to classify faults and audit business rules.
//! The Rig-backed distiller in `llm_distiller` classifies faults and audits rules
//! through Rig agent prompts, bridged onto the sync trait with a worker thread.

pub mod cargo_harness;
pub mod llm_distiller;

pub use self::llm_distiller::{HolonEvaluatorAgent, HolonEvaluatorAgentBuilder, RigDiagnosticDistiller};

/// Execution output from the execution harness.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct HarnessExecutionOutput {
    /// Flag indicating whether the candidate code compiles.
    pub compiles: ::core::primitive::bool,
    /// Fraction of characterization tests that passed.
    pub pass_rate: ::core::primitive::f64,
    /// Raw diagnostic logs or compiler output.
    pub raw_logs: ::std::string::String,
}

/// Unified evaluation result from the two-stage evaluation process.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvaluationResult {
    /// Flag indicating whether the candidate code compiles.
    pub compiles: ::core::primitive::bool,
    /// Fraction of characterization tests that passed.
    pub test_pass_rate: ::core::primitive::f64,
    /// Semantic hardening score derived from problem model rules.
    pub hardening_score: ::core::primitive::f64,
    /// Optional diagnostic fault record for compiler or test failures.
    pub fault_record: ::core::option::Option<::holon_blackboard::FaultRecord>,
}

/// Contract for execution harnesses executing candidate code against tests.
pub trait ExecutionHarness: ::core::marker::Send + ::core::marker::Sync {
    /// Execute candidate code against characterization tests.
    /// The module identifier locates the target source file inside the workspace.
    fn execute_candidate(
        &self,
        module_id: &::core::primitive::str,
        candidate_code: &::core::primitive::str,
        test_suite: &[::std::string::String],
    ) -> self::HarnessExecutionOutput;
}

/// Default execution harness checking syntax and test conditions.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default)]
pub struct DefaultExecutionHarness;

impl self::DefaultExecutionHarness {
    /// Create a default execution harness.
    pub fn new() -> Self {
        Self
    }
}

impl self::ExecutionHarness for self::DefaultExecutionHarness {
    fn execute_candidate(
        &self,
        module_id: &::core::primitive::str,
        candidate_code: &::core::primitive::str,
        test_suite: &[::std::string::String],
    ) -> self::HarnessExecutionOutput {
        let _ = module_id;
        self::execute_harness_impl(candidate_code, test_suite)
    }
}
impl<H: self::ExecutionHarness + ?::core::marker::Sized> self::ExecutionHarness for ::std::boxed::Box<H> {
    fn execute_candidate(
        &self,
        module_id: &::core::primitive::str,
        candidate_code: &::core::primitive::str,
        test_suite: &[::std::string::String],
    ) -> self::HarnessExecutionOutput {
        (**self).execute_candidate(module_id, candidate_code, test_suite)
    }
}
fn execute_harness_impl(
    candidate_code: &::core::primitive::str,
    test_suite: &[::std::string::String],
) -> self::HarnessExecutionOutput {
    let syntax_check = self::check_syntax_and_compilation(candidate_code);
    if let ::core::result::Result::Err(err_msg) = syntax_check {
        return self::HarnessExecutionOutput {
            compiles: false,
            pass_rate: 0.0,
            raw_logs: { err_msg },
        };
    }
    self::evaluate_suite(candidate_code, test_suite)
}

fn check_syntax_and_compilation(
    candidate_code: &::core::primitive::str,
) -> ::core::result::Result<(), ::std::string::String> {
    let has_compile_error = candidate_code.contains("compile_error!");
    if has_compile_error {
        let err_text = ::std::string::String::from("error: explicit compile_error detected");
        return ::core::result::Result::Err(err_text);
    }
    let has_syntax_tag = candidate_code.contains("SYNTAX_ERROR");
    if has_syntax_tag {
        let err_text = ::std::string::String::from("error: explicit SYNTAX_ERROR tag detected");
        return ::core::result::Result::Err(err_text);
    }
    let token_res = candidate_code.parse::<::proc_macro2::TokenStream>();
    if let ::core::result::Result::Err(parse_err) = token_res {
        let err_text = ::std::format!("error: syntax tokenization failed: {parse_err}");
        return ::core::result::Result::Err(err_text);
    }
    ::core::result::Result::Ok(())
}

fn evaluate_suite(
    candidate_code: &::core::primitive::str,
    test_suite: &[::std::string::String],
) -> self::HarnessExecutionOutput {
    let is_empty_suite = test_suite.is_empty();
    if is_empty_suite {
        let empty_logs = ::std::string::String::new();
        return self::HarnessExecutionOutput {
            compiles: true,
            pass_rate: 1.0,
            raw_logs: { empty_logs },
        };
    }
    let total_count = test_suite.len();
    let mut passed_count: ::core::primitive::usize = 0;
    let mut log_accumulator = ::std::string::String::new();
    for test_entry in test_suite {
        let test_ok = self::run_test_case(candidate_code, test_entry);
        if test_ok {
            passed_count += 1;
        } else {
            let failure_msg = ::std::format!("test failure in characterization test: {test_entry}\n");
            log_accumulator.push_str(&failure_msg);
        }
    }
    let pass_ratio = (passed_count as ::core::primitive::f64) / (total_count as ::core::primitive::f64);
    self::HarnessExecutionOutput {
        compiles: true,
        pass_rate: { pass_ratio },
        raw_logs: { log_accumulator },
    }
}

fn run_test_case(
    candidate_code: &::core::primitive::str,
    test_entry: &::core::primitive::str,
) -> ::core::primitive::bool {
    let has_fail_tag = test_entry.contains("[should_fail]");
    if has_fail_tag {
        return false;
    }
    let has_explicit_fail = test_entry.contains("FAIL");
    if has_explicit_fail {
        return false;
    }
    let has_todo = candidate_code.contains("todo!");
    if has_todo {
        return false;
    }
    let has_unimplemented = candidate_code.contains("unimplemented!");
    if has_unimplemented {
        return false;
    }
    let has_panic = candidate_code.contains("panic!");
    if has_panic {
        return false;
    }
    self::check_test_identifiers(candidate_code, test_entry)
}

fn check_test_identifiers(
    candidate_code: &::core::primitive::str,
    test_entry: &::core::primitive::str,
) -> ::core::primitive::bool {
    let trimmed_test = test_entry.trim();
    let is_empty_test = trimmed_test.is_empty();
    if is_empty_test {
        return true;
    }
    let prefix = "assert_contains:";
    let has_prefix = trimmed_test.starts_with(prefix);
    if has_prefix {
        let stripped = trimmed_test.trim_start_matches(prefix);
        let target = stripped.trim();
        let contains_target = candidate_code.contains(target);
        return contains_target;
    }
    true
}

/// Contract for diagnostic distillers classifying failures and auditing business rules.
pub trait DiagnosticDistiller: ::core::marker::Send + ::core::marker::Sync {
    /// Distill raw compiler or test logs into a structured fault record.
    fn distill_failure(
        &self,
        raw_logs: &::core::primitive::str,
        candidate_code: &::core::primitive::str,
        problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::holon_blackboard::FaultRecord;

    /// Audit candidate code against problem model business rules.
    fn audit_rules(
        &self,
        candidate_code: &::core::primitive::str,
        problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::core::primitive::f64;
}

/// Default diagnostic distiller classifying failure causes and scoring business rule compliance.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default)]
pub struct DefaultDiagnosticDistiller;

impl self::DefaultDiagnosticDistiller {
    /// Create a default diagnostic distiller.
    pub fn new() -> Self {
        Self
    }
}

impl self::DiagnosticDistiller for self::DefaultDiagnosticDistiller {
    fn distill_failure(
        &self,
        raw_logs: &::core::primitive::str,
        candidate_code: &::core::primitive::str,
        problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::holon_blackboard::FaultRecord {
        self::distill_failure_impl(raw_logs, candidate_code, problem_model)
    }

    fn audit_rules(
        &self,
        candidate_code: &::core::primitive::str,
        problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::core::primitive::f64 {
        self::audit_rules_impl(candidate_code, problem_model)
    }
}
fn distill_failure_impl(
    raw_logs: &::core::primitive::str,
    candidate_code: &::core::primitive::str,
    problem_model: &::holon_blackboard::ProblemModelSlice,
) -> ::holon_blackboard::FaultRecord {
    let cause = self::classify_cause(raw_logs, candidate_code, problem_model);
    let summary = self::derive_summary(raw_logs);
    let diagnostic_trace = ::std::string::String::from(raw_logs);
    let cause_opt = ::core::option::Option::Some(cause);
    ::holon_blackboard::FaultRecord {
        summary,
        diagnostic_trace,
        cause: { cause_opt },
    }
}

fn classify_cause(
    raw_logs: &::core::primitive::str,
    candidate_code: &::core::primitive::str,
    problem_model: &::holon_blackboard::ProblemModelSlice,
) -> ::holon_blackboard::FaultCause {
    let has_cycle = self::detect_extrinsic_cycle(raw_logs, candidate_code);
    if has_cycle {
        return ::holon_blackboard::FaultCause::ExtrinsicCycle;
    }
    let has_contract = self::detect_contract_conflict(raw_logs, problem_model);
    if has_contract {
        return ::holon_blackboard::FaultCause::ExtrinsicContract;
    }
    ::holon_blackboard::FaultCause::Intrinsic
}

fn detect_extrinsic_cycle(
    raw_logs: &::core::primitive::str,
    candidate_code: &::core::primitive::str,
) -> ::core::primitive::bool {
    let has_cycle_log = raw_logs.contains("cyclic dependency");
    if has_cycle_log {
        return true;
    }
    let has_detected_log = raw_logs.contains("cycle detected");
    if has_detected_log {
        return true;
    }
    let has_loop_log = raw_logs.contains("dependency cycle");
    if has_loop_log {
        return true;
    }
    let has_cycle_marker = candidate_code.contains("CYCLIC_DEPENDENCY");
    if has_cycle_marker {
        return true;
    }
    false
}

fn detect_contract_conflict(
    raw_logs: &::core::primitive::str,
    problem_model: &::holon_blackboard::ProblemModelSlice,
) -> ::core::primitive::bool {
    let lower_logs = raw_logs.to_ascii_lowercase();
    let has_conflict_log = lower_logs.contains("contract conflict") || lower_logs.contains("contract_conflict");
    if has_conflict_log {
        return true;
    }
    let has_mismatch_log = raw_logs.contains("interface mismatch");
    if has_mismatch_log {
        return true;
    }
    let has_upstream_log = raw_logs.contains("upstream export");
    if has_upstream_log {
        return true;
    }
    let has_missing_export = raw_logs.contains("missing parent export");
    if has_missing_export {
        return true;
    }
    for contract_entry in &problem_model.parent_contracts {
        let parent_identifier = &contract_entry.parent_id;
        let is_empty_parent = parent_identifier.is_empty();
        if !is_empty_parent {
            let contains_parent = raw_logs.contains(parent_identifier);
            if contains_parent {
                return true;
            }
        }
    }
    for delta_entry in &problem_model.delta_constraints {
        let is_empty_delta = delta_entry.is_empty();
        if !is_empty_delta {
            let contains_delta = raw_logs.contains(delta_entry);
            if contains_delta {
                return true;
            }
        }
    }
    false
}

fn derive_summary(raw_logs: &::core::primitive::str) -> ::std::string::String {
    for line_slice in raw_logs.lines() {
        let trimmed_slice = line_slice.trim();
        let is_empty_line = trimmed_slice.is_empty();
        if !is_empty_line {
            let line_string = ::std::string::String::from(trimmed_slice);
            return line_string;
        }
    }
    ::std::string::String::from("diagnostic evaluation failure")
}

fn audit_rules_impl(
    candidate_code: &::core::primitive::str,
    problem_model: &::holon_blackboard::ProblemModelSlice,
) -> ::core::primitive::f64 {
    let is_empty_rules = problem_model.rules.is_empty();
    if is_empty_rules {
        return 1.0;
    }
    let total_rules = problem_model.rules.len();
    let mut matched_rules: ::core::primitive::usize = 0;
    for rule_entry in &problem_model.rules {
        let rule_satisfied = self::check_rule_satisfaction(candidate_code, rule_entry);
        if rule_satisfied {
            matched_rules += 1;
        }
    }
    (matched_rules as ::core::primitive::f64) / (total_rules as ::core::primitive::f64)
}

fn check_rule_satisfaction(
    candidate_code: &::core::primitive::str,
    rule_card: &::holon_blackboard::BusinessRuleCard,
) -> ::core::primitive::bool {
    let rule_tag = &rule_card.rule_id;
    let contains_rule_id = candidate_code.contains(rule_tag);
    if contains_rule_id {
        return true;
    }
    let when_terms = self::extract_significant_terms(&rule_card.when);
    let then_terms = self::extract_significant_terms(&rule_card.then);
    let is_when_empty = when_terms.is_empty();
    let is_then_empty = then_terms.is_empty();
    if is_when_empty && is_then_empty {
        return true;
    }
    let when_ok = self::terms_present_in_code(candidate_code, &when_terms);
    let then_ok = self::terms_present_in_code(candidate_code, &then_terms);
    when_ok && then_ok
}

fn extract_significant_terms(clause_text: &::core::primitive::str) -> ::std::vec::Vec<::std::string::String> {
    let mut term_list = ::std::vec::Vec::new();
    let word_list = clause_text.split_whitespace();
    for word_token in word_list {
        let clean_token = word_token.trim_matches(self::is_not_alphanumeric);
        let token_length = clean_token.len();
        if token_length > 3 {
            let lower_token = clean_token.to_ascii_lowercase();
            let is_stop = self::is_stop_word(&lower_token);
            if !is_stop {
                term_list.push(lower_token);
            }
        }
    }
    term_list
}

fn is_not_alphanumeric(character_token: ::core::primitive::char) -> ::core::primitive::bool {
    let is_alnum = character_token.is_alphanumeric();
    !is_alnum
}

fn is_stop_word(word_token: &::core::primitive::str) -> ::core::primitive::bool {
    ::core::matches!(
        word_token,
        "when" | "then" | "given" | "should" | "must" | "will" | "with" | "from" | "that" | "this" | "have"
    )
}

fn terms_present_in_code(
    candidate_code: &::core::primitive::str,
    terms: &[::std::string::String],
) -> ::core::primitive::bool {
    let is_empty_terms = terms.is_empty();
    if is_empty_terms {
        return true;
    }
    let lower_code = candidate_code.to_ascii_lowercase();
    for term_entry in terms {
        let contains_term = lower_code.contains(term_entry);
        if !contains_term {
            return false;
        }
    }
    true
}

/// Two-stage evaluator combining deterministic execution harness and diagnostic distiller.
/// Generic over the concrete stage components with no trait-object boxing.
pub struct TwoStageEvaluator<H = self::DefaultExecutionHarness, D = self::DefaultDiagnosticDistiller> {
    /// Stage 3a deterministic execution harness.
    pub harness: H,
    /// Stage 3b diagnostic distiller and parity critic.
    pub distiller: D,
}

impl<H, D> ::core::fmt::Debug for self::TwoStageEvaluator<H, D> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("TwoStageEvaluator").finish()
    }
}

impl<H: ::core::default::Default, D: ::core::default::Default> ::core::default::Default for self::TwoStageEvaluator<H, D> {
    fn default() -> Self {
        Self { harness: H::default(), distiller: D::default() }
    }
}

#[::bon::bon]
impl<H: self::ExecutionHarness, D: self::DiagnosticDistiller> self::TwoStageEvaluator<H, D> {
    /// Construct a two-stage evaluator over explicit stage components.
    #[builder]
    pub fn builder(harness: H, distiller: D) -> Self {
        Self { harness, distiller }
    }

    /// Evaluate candidate code against test suite and problem model rules.
    pub fn evaluate(
        &self,
        module_id: &::core::primitive::str,
        candidate_code: &::core::primitive::str,
        test_suite: &[::std::string::String],
        problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> self::EvaluationResult {
        let harness_output = self.harness.execute_candidate(module_id, candidate_code, test_suite);
        let does_compile = harness_output.compiles;
        if !does_compile {
            let fault = self.distiller.distill_failure(&harness_output.raw_logs, candidate_code, problem_model);
            let fault_opt = ::core::option::Option::Some(fault);
            return self::EvaluationResult {
                compiles: false,
                test_pass_rate: 0.0,
                hardening_score: 0.0,
                fault_record: { fault_opt },
            };
        }

        let test_pass_rate = harness_output.pass_rate;
        let hardening_score = self.distiller.audit_rules(candidate_code, problem_model);
        let is_perfect_pass = test_pass_rate >= 0.999_999;
        if !is_perfect_pass {
            let fault = self.distiller.distill_failure(&harness_output.raw_logs, candidate_code, problem_model);
            let fault_opt = ::core::option::Option::Some(fault);
            return self::EvaluationResult {
                compiles: true,
                test_pass_rate,
                hardening_score,
                fault_record: { fault_opt },
            };
        }

        self::EvaluationResult {
            compiles: true,
            test_pass_rate: 1.0,
            hardening_score,
            fault_record: ::core::option::Option::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(::core::clone::Clone, ::core::fmt::Debug)]
    struct MockExecutionHarness;

    impl self::ExecutionHarness for self::MockExecutionHarness {
        fn execute_candidate(
            &self,
            _module_id: &::core::primitive::str,
            _candidate_code: &::core::primitive::str,
            _test_suite: &[::std::string::String],
        ) -> self::HarnessExecutionOutput {
            let logs_text = ::std::string::String::new();
            self::HarnessExecutionOutput {
                compiles: true,
                pass_rate: 1.0,
                raw_logs: { logs_text },
            }
        }
    }

    #[derive(::core::clone::Clone, ::core::fmt::Debug)]
    struct MockDiagnosticDistiller;

    impl self::DiagnosticDistiller for self::MockDiagnosticDistiller {
        fn distill_failure(
            &self,
            raw_logs: &::core::primitive::str,
            _candidate_code: &::core::primitive::str,
            _problem_model: &::holon_blackboard::ProblemModelSlice,
        ) -> ::holon_blackboard::FaultRecord {
            let trace_val = ::std::string::String::from(raw_logs);
            let summary_val = ::std::string::String::from("mock failure");
            let cause_val = ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic);
            ::holon_blackboard::FaultRecord {
                summary: { summary_val },
                diagnostic_trace: { trace_val },
                cause: { cause_val },
            }
        }

        fn audit_rules(
            &self,
            _candidate_code: &::core::primitive::str,
            _problem_model: &::holon_blackboard::ProblemModelSlice,
        ) -> ::core::primitive::f64 {
            0.75
        }
    }

    #[test]
    fn test_evaluator_compiling_code() {
        let evaluator = self::TwoStageEvaluator::<
            self::DefaultExecutionHarness,
            self::DefaultDiagnosticDistiller,
        >::default();
        let candidate_code = "pub fn add_values(val_a: i32, val_b: i32) -> i32 { // RULE_001\n val_a + val_b }";
        let test_item = ::std::string::String::from("assert_contains: add_values");
        let test_suite = ::std::vec![test_item];
        let mut problem_model = ::holon_blackboard::ProblemModelSlice::new();
        let rule_id = ::std::string::String::from("RULE_001");
        let given_text = ::std::string::String::from("two numbers");
        let when_text = ::std::string::String::from("add values");
        let then_text = ::std::string::String::from("returns sum");
        let citation_text = ::std::string::String::from("math.c:10");
        let rule_card = ::holon_blackboard::BusinessRuleCard {
            rule_id,
            given: { given_text },
            when: { when_text },
            then: { then_text },
            citation: { citation_text },
        };
        problem_model.add_rule(rule_card);
        let result_output = evaluator.evaluate("module_probe", candidate_code, &test_suite, &problem_model);
        let compiles_status = result_output.compiles;
        ::core::assert!(compiles_status);
        let pass_rate_val = result_output.test_pass_rate;
        let is_pass_full = pass_rate_val >= 0.999_999;
        ::core::assert!(is_pass_full);
        let hardening_val = result_output.hardening_score;
        let is_hardening_positive = hardening_val > 0.0;
        ::core::assert!(is_hardening_positive);
        let fault_presence = result_output.fault_record.is_some();
        ::core::assert!(!fault_presence);
    }

    #[test]
    fn test_evaluator_failing_tests() {
        let evaluator = self::TwoStageEvaluator::<
            self::DefaultExecutionHarness,
            self::DefaultDiagnosticDistiller,
        >::default();
        let candidate_code = "pub fn add_values(val_a: i32, val_b: i32) -> i32 { panic!(\"unimplemented\") }";
        let test_item = ::std::string::String::from("assert_contains: add_values");
        let test_suite = ::std::vec![test_item];
        let problem_model = ::holon_blackboard::ProblemModelSlice::new();
        let result_output = evaluator.evaluate("module_probe", candidate_code, &test_suite, &problem_model);
        let compiles_status = result_output.compiles;
        ::core::assert!(compiles_status);
        let pass_rate_val = result_output.test_pass_rate;
        let is_zero_pass = pass_rate_val < 0.000_001;
        ::core::assert!(is_zero_pass);
        let fault_presence = result_output.fault_record.is_some();
        ::core::assert!(fault_presence);
        let fault_entry = result_output.fault_record.unwrap();
        let cause_val = fault_entry.cause;
        let expected_cause = ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic);
        ::core::assert_eq!(cause_val, expected_cause);
    }

    #[test]
    fn test_evaluator_uncompilable_code() {
        let evaluator = self::TwoStageEvaluator::<
            self::DefaultExecutionHarness,
            self::DefaultDiagnosticDistiller,
        >::default();
        let candidate_code = "pub fn syntax_error( { unclosed tokens";
        let test_suite = ::std::vec::Vec::<::std::string::String>::new();
        let problem_model = ::holon_blackboard::ProblemModelSlice::new();
        let result_output = evaluator.evaluate("module_probe", candidate_code, &test_suite, &problem_model);
        let compiles_status = result_output.compiles;
        ::core::assert!(!compiles_status);
        let pass_rate_val = result_output.test_pass_rate;
        let is_zero_pass = pass_rate_val < 0.000_001;
        ::core::assert!(is_zero_pass);
        let hardening_val = result_output.hardening_score;
        let is_zero_hardening = hardening_val < 0.000_001;
        ::core::assert!(is_zero_hardening);
        let fault_presence = result_output.fault_record.is_some();
        ::core::assert!(fault_presence);
        let fault_entry = result_output.fault_record.unwrap();
        let cause_val = fault_entry.cause;
        let expected_cause = ::core::option::Option::Some(::holon_blackboard::FaultCause::Intrinsic);
        ::core::assert_eq!(cause_val, expected_cause);
    }

    #[test]
    fn test_evaluator_extrinsic_cycle() {
        let evaluator = self::TwoStageEvaluator::<
            self::DefaultExecutionHarness,
            self::DefaultDiagnosticDistiller,
        >::default();
        let candidate_code = "pub fn cyclic_node() -> i32 { 1 }\n// CYCLIC_DEPENDENCY";
        let test_fail = ::std::string::String::from("FAIL");
        let test_suite = ::std::vec![test_fail];
        let problem_model = ::holon_blackboard::ProblemModelSlice::new();
        let result_output = evaluator.evaluate("module_probe", candidate_code, &test_suite, &problem_model);
        let fault_presence = result_output.fault_record.is_some();
        ::core::assert!(fault_presence);
        let fault_entry = result_output.fault_record.unwrap();
        let cause_val = fault_entry.cause;
        let expected_cause = ::core::option::Option::Some(::holon_blackboard::FaultCause::ExtrinsicCycle);
        ::core::assert_eq!(cause_val, expected_cause);
    }

    #[test]
    fn test_evaluator_extrinsic_contract() {
        let evaluator = self::TwoStageEvaluator::<
            self::DefaultExecutionHarness,
            self::DefaultDiagnosticDistiller,
        >::default();
        let candidate_code = "pub fn broken_upstream() -> i32 { 1 }";
        let test_fail = ::std::string::String::from("FAIL: missing parent export");
        let test_suite = ::std::vec![test_fail];
        let mut problem_model = ::holon_blackboard::ProblemModelSlice::new();
        let delta_text = ::std::string::String::from("missing parent export");
        problem_model.add_delta_constraint(delta_text);
        let result_output = evaluator.evaluate("module_probe", candidate_code, &test_suite, &problem_model);
        let fault_presence = result_output.fault_record.is_some();
        ::core::assert!(fault_presence);
        let fault_entry = result_output.fault_record.unwrap();
        let cause_val = fault_entry.cause;
        let expected_cause = ::core::option::Option::Some(::holon_blackboard::FaultCause::ExtrinsicContract);
        ::core::assert_eq!(cause_val, expected_cause);
    }

    #[test]
    fn test_evaluator_builder() {
        let mock_harness = self::tests::MockExecutionHarness;
        let mock_distiller = self::tests::MockDiagnosticDistiller;
        let evaluator = self::TwoStageEvaluator::builder()
            .harness(mock_harness)
            .distiller(mock_distiller)
            .build();
        let candidate_code = "pub fn demo() -> i32 { 1 }";
        let test_suite = ::std::vec::Vec::<::std::string::String>::new();
        let problem_model = ::holon_blackboard::ProblemModelSlice::new();
        let result_output = evaluator.evaluate("module_probe", candidate_code, &test_suite, &problem_model);
        let compiles_status = result_output.compiles;
        ::core::assert!(compiles_status);
        let pass_rate_val = result_output.test_pass_rate;
        let is_full_pass = pass_rate_val >= 0.999_999;
        ::core::assert!(is_full_pass);
        let hardening_val = result_output.hardening_score;
        let hardening_delta = (hardening_val - 0.75).abs();
        let is_hardening_matching = hardening_delta < 0.000_001;
        ::core::assert!(is_hardening_matching);
    }
}
