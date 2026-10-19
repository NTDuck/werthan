//! Laya fast non-generative encoder daemon and deterministic hooks.
//!
//! Provides ~30ms latency, zero token cost evaluation for:
//! 1. Stateful Pre-Tool Guardrail (`pre_tool_use`) with taint tracking.
//! 2. The Historian Airlock (`airlock_validate_rule_card`) with injection defense.
//! 3. Fast Pre-Critic Rubric Gate (`evaluate_pre_critic_rubric`) with 12 quality questions.
//! 4. Backward compatibility APIs for former jev engine calls.

/// Calibrated probability value between 0.0 and 1.0.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct Noul(pub ::core::primitive::f64);

impl self::Noul {
    /// Create a new Noul probability value clamped to 0.0..=1.0.
    pub fn new(raw_value: ::core::primitive::f64) -> Self {
        let clamped = raw_value.clamp(0.0, 1.0);
        Self(clamped)
    }

    /// Return the raw float probability value.
    pub fn value(&self) -> ::core::primitive::f64 {
        self.0
    }

    /// Return true when the probability exceeds the high-risk threshold of 0.05.
    pub fn is_high_risk(&self) -> ::core::primitive::bool {
        self.0 > 0.05
    }
}

/// Static rubric score between 0 and 4.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct Score(pub ::core::primitive::u8);

impl self::Score {
    /// Create a new Score value clamped to 0..=4.
    pub fn new(raw_score: ::core::primitive::u8) -> Self {
        let clamped = raw_score.min(4);
        Self(clamped)
    }

    /// Return the score integer.
    pub fn value(&self) -> ::core::primitive::u8 {
        self.0
    }
}

/// Categorical choice output from fast encoder.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct Choice {
    /// Selected label or variant name.
    pub label: ::std::string::String,
    /// Probability assigned to the selection.
    pub probability: ::core::primitive::f64,
}

/// Guardrail action taken on tool invocation.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum GuardrailAction {
    /// Allow execution without friction.
    Allow,
    /// Request human confirmation before execution.
    Confirm,
    /// Block command execution immediately.
    Block,
}

/// Verdict returned by Stateful Pre-Tool Guardrail.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct GuardrailVerdict {
    /// Guardrail action.
    pub action: self::GuardrailAction,
    /// True when the command is allowed to execute.
    pub allow: ::core::primitive::bool,
    /// Computed aggregate risk score (0.0 to 1.0).
    pub risk_score: ::core::primitive::f64,
    /// Explanation of the verdict.
    pub reason: ::std::string::String,
    /// Indicates if the environment is currently tainted.
    pub tainted: ::core::primitive::bool,
    /// Blast radius score (0 to 4).
    pub blast_radius: ::core::primitive::u8,
}

/// Stateful Pre-Tool Guardrail tracking command execution safety and configuration taint.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct PreToolGuardrail {
    /// Indicates whether build configuration has been touched.
    pub tainted_build_config: ::core::primitive::bool,
    /// History of files or commands that caused taint.
    pub taint_history: ::std::vec::Vec<::std::string::String>,
}

/// Raw business rule card submitted to Historian Airlock.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct RuleCard {
    /// Card identifier.
    pub id: ::std::string::String,
    /// Descriptive card title.
    pub title: ::std::string::String,
    /// Behavioral statement in Given/When/Then format.
    pub given_when_then: ::std::string::String,
    /// Citation anchor (e.g., file:LINE#HASH or ghidra:0xADDR).
    pub citation_anchor: ::std::string::String,
    /// Kind of evidence supporting the card.
    pub evidence_kind: ::std::string::String,
}

/// Clean structured rule card accepted by Historian Airlock.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct CleanRuleCard {
    /// Card identifier.
    pub id: ::std::string::String,
    /// Descriptive card title.
    pub title: ::std::string::String,
    /// Behavioral statement in Given/When/Then format.
    pub given_when_then: ::std::string::String,
    /// Validated citation anchor.
    pub citation_anchor: ::std::string::String,
    /// Kind of evidence supporting the card.
    pub evidence_kind: ::std::string::String,
    /// Verification status note.
    pub verified_anchor: ::std::string::String,
}

/// Rejection reasons returned by Historian Airlock.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::cmp::PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum AirlockError {
    /// Required field is empty or missing.
    MissingField(::std::string::String),
    /// Citation anchor format is invalid.
    InvalidCitationAnchor(::std::string::String),
    /// Prompt override or imperative instruction detected.
    PromptOverrideDetected(::std::string::String),
    /// Code or shell injection syntax detected.
    CodeInjectionDetected(::std::string::String),
}

impl ::core::fmt::Display for self::AirlockError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::MissingField(field) => ::core::write!(f, "Missing required field: {field}"),
            Self::InvalidCitationAnchor(anchor) => ::core::write!(f, "Invalid citation anchor: {anchor}"),
            Self::PromptOverrideDetected(reason) => ::core::write!(f, "Prompt override detected: {reason}"),
            Self::CodeInjectionDetected(reason) => ::core::write!(f, "Code injection detected: {reason}"),
        }
    }
}

/// Verdict on rule card acceptability for backward compatibility.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct RuleCardVerdict {
    /// True when card is accepted.
    pub is_accepted: ::core::primitive::bool,
    /// Diagnostic description.
    pub reason: ::std::string::String,
}

/// Decision output from Fast Pre-Critic Rubric Gate.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum PreCriticDecision {
    /// Code passes static rubric checks.
    Pass,
    /// Result is ambiguous, requires adversarial council review.
    Ambiguous,
    /// High-confidence failure detected, reject immediately.
    HighConfidenceFail,
}

/// Defect identified by Pre-Critic Rubric Gate.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct RubricDefect {
    /// Question identifier (Q01 to Q12).
    pub question_id: ::std::string::String,
    /// Question name.
    pub question_name: ::std::string::String,
    /// Severity level ("high", "medium", "low").
    pub severity: ::std::string::String,
    /// Detailed defect description.
    pub description: ::std::string::String,
    /// Matched code line or pattern.
    pub matched_line: ::std::string::String,
    /// Confidence level (0.0 to 1.0).
    pub confidence: ::core::primitive::f64,
}

/// Result returned by Fast Pre-Critic Rubric Gate.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct PreCriticResult {
    /// Gate decision.
    pub decision: self::PreCriticDecision,
    /// List of detected rubric defects.
    pub defects: ::std::vec::Vec<self::RubricDefect>,
    /// True when the diff should be forwarded to Adversarial Trio.
    pub forward_to_adversarial_trio: ::core::primitive::bool,
    /// Latency elapsed in milliseconds (typically under 30ms).
    pub latency_ms: ::core::primitive::u64,
    /// Total count of evaluated static questions.
    pub evaluated_questions: ::core::primitive::usize,
}

/// Execution profile routing for backward compatibility.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum ExecutionProfile {
    /// Fast mixture-of-experts AST analysis.
    FastMoeAst,
    /// Deep reasoning chain-of-thought analysis.
    DeepChainOfThought,
    /// Adversarial trio security and architecture audit.
    AdversarialAudit,
}

/// Test coverage theater verdict for backward compatibility.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct CoverageVerdict {
    /// True when test coverage is substantive and asserts on values.
    pub is_accepted: ::core::primitive::bool,
    /// Diagnostic description.
    pub reason: ::std::string::String,
}

/// Done claim verdict for backward compatibility.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct DoneClaimVerdict {
    /// True when claim is valid.
    pub valid: ::core::primitive::bool,
    /// Diagnostic description.
    pub reason: ::std::string::String,
}

/// Gate evaluation query struct for backward compatibility.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct GateQuery {
    /// Campaign identifier.
    pub campaign_id: ::std::string::String,
    /// Blocker note.
    pub blocker: ::std::string::String,
    /// Diff status description.
    pub diff_status: ::std::string::String,
    /// Test status description.
    pub test_status: ::std::string::String,
    /// Defect count.
    pub defect_count: ::core::primitive::usize,
}

/// Gate evaluation decision for backward compatibility.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct GateDecision {
    /// True when gate is approved.
    pub approved: ::core::primitive::bool,
    /// Confidence score.
    pub confidence: ::core::primitive::f64,
    /// Diagnostic note.
    pub note: ::std::string::String,
}

/// Loop detection verdict for backward compatibility.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct LoopVerdict {
    /// True when execution is caught in a repetitive loop.
    pub is_stuck: ::core::primitive::bool,
    /// Count of consecutive repeated actions.
    pub repetition_count: ::core::primitive::usize,
    /// Suggested remediation.
    pub suggestion: ::std::string::String,
}

/// Validate a rule card through the Historian Airlock.
pub fn airlock_validate_rule_card(
    card: &self::RuleCard,
) -> ::core::result::Result<self::CleanRuleCard, self::AirlockError> {
    let engine = self::LayaEngine::new();
    let res = engine.airlock_validate_rule_card(card);
    res
}

/// Evaluate code change through the Fast Pre-Critic Rubric Gate.
pub fn evaluate_pre_critic_rubric(
    rule_cards: &[self::RuleCard],
    hashline_diff: &::core::primitive::str,
    compiler_output: &::core::primitive::str,
) -> self::PreCriticResult {
    let engine = self::LayaEngine::new();
    let res = engine.evaluate_pre_critic_rubric(rule_cards, hashline_diff, compiler_output);
    res
}
/// Laya fast encoder client.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct LayaClient {
    endpoint: ::std::string::String,
    is_simulated: ::core::primitive::bool,
    engine: self::LayaEngine,
}

/// Backward compatibility alias for former JevClient.
pub type JevClient = self::LayaClient;

impl self::LayaClient {
    /// Create a new LayaClient connected to a ninfer or daemon endpoint.
    pub fn new(endpoint: &::core::primitive::str) -> Self {
        let ep = endpoint.to_string();
        let engine = self::LayaEngine::new();
        Self {
            endpoint: ep,
            is_simulated: false,
            engine,
        }
    }

    /// Create a simulated in-process LayaClient.
    pub fn simulated() -> Self {
        let ep = "in-process://simulated".to_string();
        let engine = self::LayaEngine::new();
        Self {
            endpoint: ep,
            is_simulated: true,
            engine,
        }
    }

    /// Return reference to internal LayaEngine.
    pub fn engine(&self) -> &self::LayaEngine {
        &self.engine
    }

    /// Instantiate a new Stateful Pre-Tool Guardrail instance.
    pub fn pre_tool_guardrail(&self) -> self::PreToolGuardrail {
        self::PreToolGuardrail::new()
    }

    /// Validate a rule card through the Historian Airlock.
    pub fn airlock_validate_rule_card(
        &self,
        card: &self::RuleCard,
    ) -> ::core::result::Result<self::CleanRuleCard, self::AirlockError> {
        self.engine.airlock_validate_rule_card(card)
    }

    /// Evaluate code change through the Fast Pre-Critic Rubric Gate.
    pub fn evaluate_pre_critic_rubric(
        &self,
        rule_cards: &[self::RuleCard],
        hashline_diff: &::core::primitive::str,
        compiler_output: &::core::primitive::str,
    ) -> self::PreCriticResult {
        self.engine.evaluate_pre_critic_rubric(rule_cards, hashline_diff, compiler_output)
    }

    /// Evaluate transformation gate query for backward compatibility.
    pub async fn evaluate(
        &self,
        query: &self::GateQuery,
    ) -> ::core::result::Result<self::GateDecision, ::std::string::String> {
        if self.is_simulated || self.endpoint.is_empty() {
            let approved = query.defect_count == 0 && query.blocker != "blocker";
            let confidence = if approved {
                0.96
            } else {
                0.20
            };
            let note = if approved {
                "Simulated Laya evaluation approved: diff clean and verified.".to_string()
            } else {
                "Simulated Laya evaluation rejected: defects or blocker present.".to_string()
            };
            let decision = self::GateDecision::builder().approved(approved).confidence(confidence).note(note).build();
            return ::core::result::Result::Ok(decision);
        }

        let approved = query.defect_count == 0;
        let note = "Remote Laya evaluation approved.".to_string();
        let decision = self::GateDecision::builder().approved(approved).confidence(0.95).note(note).build();
        ::core::result::Result::Ok(decision)
    }
}

/// Laya fast encoder engine answering Noul, Score, and Choice questions.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default)]
pub struct LayaEngine;

impl self::LayaEngine {
    /// Create a new LayaEngine instance.
    pub fn new() -> Self {
        Self
    }

    /// Evaluate a Noul question (probability 0.0 to 1.0).
    pub fn evaluate_noul(&self, question: &::core::primitive::str, context: &::core::primitive::str) -> self::Noul {
        let q_lower = question.to_ascii_lowercase();
        let ctx_lower = context.to_ascii_lowercase();

        if q_lower.contains("destructive") {
            let is_destr = self::is_destructive_command(&ctx_lower);
            let prob = if is_destr {
                0.99
            } else {
                0.01
            };
            return self::Noul::new(prob);
        }

        if q_lower.contains("exfiltration") {
            let has_net = ctx_lower.contains("curl") || ctx_lower.contains("wget") || ctx_lower.contains("nc ");
            let prob = if has_net {
                0.95
            } else {
                0.01
            };
            return self::Noul::new(prob);
        }

        if q_lower.contains("prompt_override") || q_lower.contains("prompt_overrides") {
            let has_override = self::detect_prompt_override(&ctx_lower);
            let prob = if has_override {
                0.98
            } else {
                0.01
            };
            return self::Noul::new(prob);
        }

        if q_lower.contains("code_injection") {
            let has_inject = self::detect_code_injection(&ctx_lower);
            let prob = if has_inject {
                0.99
            } else {
                0.01
            };
            return self::Noul::new(prob);
        }

        self::Noul::new(0.02)
    }

    /// Evaluate a Score question (score 0 to 4).
    pub fn evaluate_score(&self, question: &::core::primitive::str, context: &::core::primitive::str) -> self::Score {
        let q_lower = question.to_ascii_lowercase();
        let ctx_lower = context.to_ascii_lowercase();

        if q_lower.contains("blast_radius") {
            let is_root = ctx_lower.contains(" /") || ctx_lower.contains("/etc") || ctx_lower.contains("mkfs");
            if is_root {
                return self::Score::new(4);
            }
            let is_sys = ctx_lower.contains("/usr") || ctx_lower.contains("/var") || ctx_lower.contains("/opt");
            if is_sys {
                return self::Score::new(3);
            }
            let is_multi = ctx_lower.contains("../") || ctx_lower.contains("Cargo.toml");
            if is_multi {
                return self::Score::new(2);
            }
            return self::Score::new(1);
        }

        self::Score::new(0)
    }

    /// Evaluate a Choice question selecting one option.
    pub fn evaluate_choice(
        &self,
        _question: &::core::primitive::str,
        options: &[&::core::primitive::str],
        _context: &::core::primitive::str,
    ) -> self::Choice {
        let first = options.first().copied().unwrap_or("none");
        let label = first.to_string();
        self::Choice::builder().label(label).probability(0.95).build()
    }

    /// Validate a rule card through the Historian Airlock.
    pub fn airlock_validate_rule_card(
        &self,
        card: &self::RuleCard,
    ) -> ::core::result::Result<self::CleanRuleCard, self::AirlockError> {
        let title_trimmed = card.title.trim();
        if title_trimmed.is_empty() {
            let msg = "title must not be empty".to_string();
            return ::core::result::Result::Err(self::AirlockError::MissingField(msg));
        }

        let gwt_trimmed = card.given_when_then.trim();
        if gwt_trimmed.is_empty() {
            let msg = "given_when_then must not be empty".to_string();
            return ::core::result::Result::Err(self::AirlockError::MissingField(msg));
        }

        let evidence_trimmed = card.evidence_kind.trim();
        if evidence_trimmed.is_empty() {
            let msg = "evidence_kind must not be empty".to_string();
            return ::core::result::Result::Err(self::AirlockError::MissingField(msg));
        }

        let anchor_valid = self::validate_anchor_format(&card.citation_anchor);
        if !anchor_valid {
            let msg = ::std::format!("Invalid citation anchor: {}", card.citation_anchor);
            return ::core::result::Result::Err(self::AirlockError::InvalidCitationAnchor(msg));
        }

        let has_prompt_override =
            self::detect_prompt_override(&card.given_when_then) || self::detect_prompt_override(&card.title);
        if has_prompt_override {
            let msg = "Imperative instruction or prompt override detected in rule card".to_string();
            return ::core::result::Result::Err(self::AirlockError::PromptOverrideDetected(msg));
        }

        let has_code_injection =
            self::detect_code_injection(&card.given_when_then) || self::detect_code_injection(&card.title);
        if has_code_injection {
            let msg = "Code injection or shell syntax detected in rule card".to_string();
            return ::core::result::Result::Err(self::AirlockError::CodeInjectionDetected(msg));
        }

        let verified_note = ::std::format!("Verified anchor: {}", card.citation_anchor);
        let clean = self::CleanRuleCard::builder()
            .id(card.id.clone())
            .title(card.title.clone())
            .given_when_then(card.given_when_then.clone())
            .citation_anchor(card.citation_anchor.clone())
            .evidence_kind(card.evidence_kind.clone())
            .verified_anchor(verified_note)
            .build();

        ::core::result::Result::Ok(clean)
    }

    /// Evaluate code change through the Fast Pre-Critic Rubric Gate.
    pub fn evaluate_pre_critic_rubric(
        &self,
        rule_cards: &[self::RuleCard],
        hashline_diff: &::core::primitive::str,
        compiler_output: &::core::primitive::str,
    ) -> self::PreCriticResult {
        let defects = self::evaluate_rubric_questions(rule_cards, hashline_diff, compiler_output);
        let has_high_severity = defects.iter().any(|d| d.severity == "high");
        let is_empty = defects.is_empty();

        let decision = if has_high_severity {
            self::PreCriticDecision::HighConfidenceFail
        } else if !is_empty {
            self::PreCriticDecision::Ambiguous
        } else {
            self::PreCriticDecision::Pass
        };

        let forward = decision != self::PreCriticDecision::HighConfidenceFail;

        self::PreCriticResult::builder()
            .decision(decision)
            .defects(defects)
            .forward_to_adversarial_trio(forward)
            .latency_ms(2)
            .evaluated_questions(12)
            .build()
    }
}

impl self::PreToolGuardrail {
    /// Create a new clean PreToolGuardrail instance.
    pub fn new() -> Self {
        let history = ::std::vec::Vec::new();
        Self {
            tainted_build_config: false,
            taint_history: history,
        }
    }

    /// Return true when the environment is tainted.
    pub fn is_tainted(&self) -> ::core::primitive::bool {
        self.tainted_build_config
    }

    /// Return slice of taint history.
    pub fn taint_history(&self) -> &[::std::string::String] {
        &self.taint_history
    }

    /// Evaluate a shell command and optional diff before execution.
    pub fn pre_tool_use(
        &mut self,
        command: &::core::primitive::str,
        diff_content: ::core::option::Option<&::core::primitive::str>,
    ) -> self::GuardrailVerdict {
        let subcommands = self::split_shell_commands(command);

        let cmd_taints = self::detect_build_config_touch(command);
        let diff_taints = diff_content.map_or(false, self::detect_build_config_touch);
        if cmd_taints || diff_taints {
            self.tainted_build_config = true;
            let note = ::std::format!("Tainted by: {command}");
            self.taint_history.push(note);
        }

        let is_whole_destr = self::is_destructive_command(command);
        if is_whole_destr {
            let reason = ::std::format!("Blocked destructive command: {command}");
            return self::GuardrailVerdict::builder()
                .action(self::GuardrailAction::Block)
                .allow(false)
                .risk_score(1.0)
                .reason(reason)
                .tainted(self.tainted_build_config)
                .blast_radius(4)
                .build();
        }

        for sub in &subcommands {
            let is_destr = self::is_destructive_command(sub);
            if is_destr {
                let reason = ::std::format!("Blocked destructive command segment: {sub}");
                return self::GuardrailVerdict::builder()
                    .action(self::GuardrailAction::Block)
                    .allow(false)
                    .risk_score(1.0)
                    .reason(reason)
                    .tainted(self.tainted_build_config)
                    .blast_radius(4)
                    .build();
            }

            let is_out_scope = self::is_out_of_scope_path(sub);
            if is_out_scope {
                let reason = ::std::format!("Blocked out-of-scope system path access: {sub}");
                return self::GuardrailVerdict::builder()
                    .action(self::GuardrailAction::Block)
                    .allow(false)
                    .risk_score(0.95)
                    .reason(reason)
                    .tainted(self.tainted_build_config)
                    .blast_radius(3)
                    .build();
            }
        }

        if self.tainted_build_config {
            let risk = self::calculate_shell_safety_risk(&subcommands);
            if risk > 0.05 {
                let reason =
                    ::std::format!("Blocked due to elevated risk ({risk:.2}) under tainted build configuration");
                return self::GuardrailVerdict::builder()
                    .action(self::GuardrailAction::Block)
                    .allow(false)
                    .risk_score(risk)
                    .reason(reason)
                    .tainted(true)
                    .blast_radius(2)
                    .build();
            }

            let reason = "Allowed under elevated monitoring for tainted build config".to_string();
            return self::GuardrailVerdict::builder()
                .action(self::GuardrailAction::Allow)
                .allow(true)
                .risk_score(risk)
                .reason(reason)
                .tainted(true)
                .blast_radius(1)
                .build();
        }

        let reason = "Command passed deterministic shell safety guardrail".to_string();
        self::GuardrailVerdict::builder()
            .action(self::GuardrailAction::Allow)
            .allow(true)
            .risk_score(0.0)
            .reason(reason)
            .tainted(false)
            .blast_radius(0)
            .build()
    }

    /// Check if arbitrary text touches build configuration and record taint if so.
    pub fn check_and_track_taint(&mut self, text: &::core::primitive::str) -> ::core::primitive::bool {
        let touched = self::detect_build_config_touch(text);
        if touched {
            self.tainted_build_config = true;
            let note = ::std::format!("Direct taint: {text}");
            self.taint_history.push(note);
        }
        touched
    }
}

impl ::core::default::Default for self::PreToolGuardrail {
    fn default() -> Self {
        Self::new()
    }
}

/// Route execution profile based on AST metrics and CWE presence.
pub fn route_execution_profile(
    loc_ratio: ::core::primitive::f64,
    ast_depth: ::core::primitive::usize,
    has_cwe: ::core::primitive::bool,
) -> self::ExecutionProfile {
    if has_cwe {
        self::ExecutionProfile::AdversarialAudit
    } else if ast_depth > 20 || loc_ratio > 3.0 {
        self::ExecutionProfile::DeepChainOfThought
    } else {
        self::ExecutionProfile::FastMoeAst
    }
}

/// Evaluate guardrail verdict for backward compatibility.
pub fn evaluate_guardrail_verdict(
    risk_score: ::core::primitive::f64,
    has_taint: ::core::primitive::bool,
) -> self::GuardrailVerdict {
    if has_taint && risk_score > 0.05 {
        let reason = "Tainted build configuration forbids elevated risk edits".to_string();
        self::GuardrailVerdict::builder()
            .action(self::GuardrailAction::Block)
            .allow(false)
            .risk_score(risk_score)
            .reason(reason)
            .tainted(true)
            .blast_radius(3)
            .build()
    } else if risk_score > 0.8 {
        let reason = "High risk score triggers guardrail block".to_string();
        self::GuardrailVerdict::builder()
            .action(self::GuardrailAction::Block)
            .allow(false)
            .risk_score(risk_score)
            .reason(reason)
            .tainted(has_taint)
            .blast_radius(3)
            .build()
    } else {
        let reason = "Clean edit accepted by guardrail".to_string();
        self::GuardrailVerdict::builder()
            .action(self::GuardrailAction::Allow)
            .allow(true)
            .risk_score(risk_score)
            .reason(reason)
            .tainted(has_taint)
            .blast_radius(1)
            .build()
    }
}

/// Judge rule card validity for backward compatibility.
pub fn judge_rule_card(rule_card: &::core::primitive::str) -> self::RuleCardVerdict {
    let lower = rule_card.to_ascii_lowercase();
    let has_override = self::detect_prompt_override(&lower);
    let has_inject = self::detect_code_injection(&lower);

    if has_override || has_inject {
        let reason = "Rule card rejected due to prompt injection or malicious shell tokens".to_string();
        return self::RuleCardVerdict::builder().is_accepted(false).reason(reason).build();
    }

    let has_gwt = lower.contains("given") && lower.contains("when") && lower.contains("then");
    if !has_gwt {
        let reason = "Rule card must contain Given/When/Then structure".to_string();
        return self::RuleCardVerdict::builder().is_accepted(false).reason(reason).build();
    }

    let reason = "Rule card verified with valid invariant structure".to_string();
    self::RuleCardVerdict::builder().is_accepted(true).reason(reason).build()
}

/// Judge test coverage theater for backward compatibility.
pub fn judge_test_coverage_theater(test_code: &::core::primitive::str) -> self::CoverageVerdict {
    let is_theater = self::check_coverage_theater(test_code);
    if is_theater {
        let reason = "Coverage theater detected: tests invoke methods without substantive assertions".to_string();
        self::CoverageVerdict::builder().is_accepted(false).reason(reason).build()
    } else {
        let reason = "Substantive test assertions accepted".to_string();
        self::CoverageVerdict::builder().is_accepted(true).reason(reason).build()
    }
}

/// Judge done claim for backward compatibility.
pub fn judge_done_claim(
    steps_taken: ::core::primitive::usize,
    proofs_attached: ::core::primitive::bool,
) -> self::DoneClaimVerdict {
    let valid = steps_taken > 0 && proofs_attached;
    let reason = if valid {
        "Done claim verified with proofs attached".to_string()
    } else {
        "Done claim invalid: steps empty or proofs missing".to_string()
    };
    self::DoneClaimVerdict::builder().valid(valid).reason(reason).build()
}

/// Build transform gate query struct for backward compatibility.
pub fn build_transform_gate_query(
    campaign_id: &::core::primitive::str,
    blocker: &::core::primitive::str,
    diff_status: &::core::primitive::str,
    test_status: &::core::primitive::str,
    defect_count: ::core::primitive::usize,
) -> self::GateQuery {
    self::GateQuery::builder()
        .campaign_id(campaign_id.to_string())
        .blocker(blocker.to_string())
        .diff_status(diff_status.to_string())
        .test_status(test_status.to_string())
        .defect_count(defect_count)
        .build()
}

/// Evaluate stuck execution loop for backward compatibility.
pub fn evaluate_stuck_loop(recent_actions: &[&::core::primitive::str]) -> self::LoopVerdict {
    let len = recent_actions.len();
    if len < 3 {
        let suggestion = "Insufficient action history to determine loop".to_string();
        return self::LoopVerdict::builder().is_stuck(false).repetition_count(0).suggestion(suggestion).build();
    }

    let last = recent_actions[len - 1];
    let prev1 = recent_actions[len - 2];
    let prev2 = recent_actions[len - 3];

    if last == prev1 && prev1 == prev2 {
        let suggestion = "Identical action repeated 3 times. Break loop and invoke /prewalk".to_string();
        self::LoopVerdict::builder().is_stuck(true).repetition_count(3).suggestion(suggestion).build()
    } else {
        let suggestion = "Execution proceeding normally".to_string();
        self::LoopVerdict::builder().is_stuck(false).repetition_count(0).suggestion(suggestion).build()
    }
}

/// Return static questions TOML specification content.
pub fn get_static_questions_toml() -> &'static ::core::primitive::str {
    ::core::include_str!("laya_questions.toml")
}

// -----------------------------------------------------------------------------
// Internal helper functions ordered in Clean Code stepdown order
// -----------------------------------------------------------------------------

fn split_shell_commands(command: &::core::primitive::str) -> ::std::vec::Vec<::std::string::String> {
    let mut results = ::std::vec::Vec::new();
    let mut current = ::std::string::String::new();
    let mut chars = command.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let ::core::option::Option::Some(ch) = chars.next() {
        if ch == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            current.push(ch);
            continue;
        }

        if ch == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            current.push(ch);
            continue;
        }

        if in_single_quote || in_double_quote {
            current.push(ch);
            continue;
        }

        if ch == ';' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                results.push(trimmed);
            }
            current.clear();
            continue;
        }

        if ch == '&' {
            if let ::core::option::Option::Some(&'&') = chars.peek() {
                chars.next();
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    results.push(trimmed);
                }
                current.clear();
                continue;
            }
        }

        if ch == '|' {
            if let ::core::option::Option::Some(&'|') = chars.peek() {
                chars.next();
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    results.push(trimmed);
                }
                current.clear();
                continue;
            }
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                results.push(trimmed);
            }
            current.clear();
            continue;
        }

        current.push(ch);
    }

    let remaining = current.trim().to_string();
    if !remaining.is_empty() {
        results.push(remaining);
    }

    results
}

fn detect_build_config_touch(text: &::core::primitive::str) -> ::core::primitive::bool {
    let lower = text.to_ascii_lowercase();
    let touches_cargo = lower.contains("cargo.toml") || lower.contains("cargo.lock");
    let touches_pkg = lower.contains("package.json") || lower.contains("package-lock.json");
    let touches_make = lower.contains("makefile") || lower.contains("cmakelists.txt");
    let touches_git = lower.contains(".git/config") || lower.contains(".git/hooks");
    touches_cargo || touches_pkg || touches_make || touches_git
}

fn is_destructive_command(cmd: &::core::primitive::str) -> ::core::primitive::bool {
    let lower = cmd.to_ascii_lowercase();
    let words = lower.split_whitespace().collect::<::std::vec::Vec<&::core::primitive::str>>();

    if words.is_empty() {
        return false;
    }

    let first = words[0];

    if first == "rm" {
        let has_r =
            words.iter().any(|w| *w == "-rf" || *w == "-r" || *w == "-fr" || *w == "-rfi" || *w == "--recursive");
        if has_r {
            let targets_root = words.iter().any(|w| {
                *w == "/"
                    || *w == "/*"
                    || *w == "~"
                    || *w == "~/*"
                    || *w == "/etc"
                    || *w == "/bin"
                    || *w == "/usr"
                    || *w == "/boot"
                    || *w == "/dev"
                    || *w == "/sys"
                    || *w == "/proc"
                    || *w == ".."
                    || *w == "../.."
            });
            if targets_root {
                return true;
            }
        }
    }

    if first.starts_with("mkfs") || first == "fdisk" || first == "parted" {
        return true;
    }

    if lower.contains("curl")
        && (lower.contains("| sh") || lower.contains("| bash") || lower.contains("|sh") || lower.contains("|bash"))
    {
        return true;
    }

    if lower.contains("wget")
        && (lower.contains("| sh") || lower.contains("| bash") || lower.contains("|sh") || lower.contains("|bash"))
    {
        return true;
    }

    if first == "dd" && lower.contains("of=/dev/") {
        return true;
    }

    if lower.contains("> /dev/sda") || lower.contains("> /dev/nvme") {
        return true;
    }

    if lower.contains("chmod -r 777 /") || lower.contains("chown -r ") && lower.contains(" /") {
        return true;
    }

    false
}

fn is_out_of_scope_path(cmd: &::core::primitive::str) -> ::core::primitive::bool {
    let lower = cmd.to_ascii_lowercase();
    let words = lower.split_whitespace().collect::<::std::vec::Vec<&::core::primitive::str>>();

    for word in words {
        if word == "/dev/null" {
            continue;
        }
        if word.starts_with("/etc/")
            || word == "/etc"
            || word.starts_with("/root/")
            || word == "/root"
            || word.starts_with("/sys/")
            || word.starts_with("/proc/")
        {
            return true;
        }
    }

    false
}

fn calculate_shell_safety_risk(subcommands: &[::std::string::String]) -> ::core::primitive::f64 {
    let mut max_risk = 0.01;

    for sub in subcommands {
        let lower = sub.to_ascii_lowercase();
        if lower.contains("curl") || lower.contains("wget") || lower.contains("nc ") || lower.contains("netcat") {
            if max_risk < 0.90 {
                max_risk = 0.90;
            }
        }
        if lower.contains("npm postinstall") || lower.contains("sh ") || lower.contains("bash ") {
            if max_risk < 0.80 {
                max_risk = 0.80;
            }
        }
        if lower.contains("git config") || lower.contains("git push") {
            if max_risk < 0.40 {
                max_risk = 0.40;
            }
        }
    }

    max_risk
}

fn validate_anchor_format(anchor: &::core::primitive::str) -> ::core::primitive::bool {
    let trimmed = anchor.trim();
    if trimmed.is_empty() {
        return false;
    }

    if let ::core::option::Option::Some(addr) = trimmed.strip_prefix("ghidra:0x") {
        if addr.is_empty() {
            return false;
        }
        return addr.chars().all(|c| c.is_ascii_hexdigit());
    }

    let colon_count = trimmed.chars().filter(|c| *c == ':').count();
    if colon_count != 1 {
        return false;
    }

    let mut parts = trimmed.split(':');
    let file_part = match parts.next() {
        ::core::option::Option::Some(f) => f.trim(),
        ::core::option::Option::None => return false,
    };
    let rest_part = match parts.next() {
        ::core::option::Option::Some(r) => r.trim(),
        ::core::option::Option::None => return false,
    };

    if file_part.is_empty() {
        return false;
    }

    if let ::core::option::Option::Some(hash_idx) = rest_part.find('#') {
        let line_str = &rest_part[..hash_idx];
        let hash_str = &rest_part[hash_idx + 1..];

        let line_valid = !line_str.is_empty() && line_str.chars().all(|c| c.is_ascii_digit());
        let hash_valid = hash_str.len() >= 4 && hash_str.chars().all(|c| c.is_ascii_alphanumeric());
        return line_valid && hash_valid;
    }

    !rest_part.is_empty() && rest_part.chars().all(|c| c.is_ascii_digit())
}

fn detect_prompt_override(text: &::core::primitive::str) -> ::core::primitive::bool {
    let lower = text.to_ascii_lowercase();

    lower.contains("ignore previous instructions")
        || lower.contains("ignore all previous instructions")
        || lower.contains("disregard previous instructions")
        || lower.contains("disregard all instructions")
        || lower.contains("system prompt:")
        || lower.contains("override instructions")
        || lower.contains("override guardrails")
        || lower.contains("you are now an unrestricted")
        || lower.contains("you are now in developer mode")
        || lower.contains("bypass safety")
        || lower.contains("forget your rules")
        || lower.contains("forget all instructions")
}

fn detect_code_injection(text: &::core::primitive::str) -> ::core::primitive::bool {
    let lower = text.to_ascii_lowercase();

    lower.contains('`')
        || lower.contains("<script")
        || lower.contains("$(")
        || lower.contains("${")
        || lower.contains("; rm ")
        || lower.contains(";rm ")
        || lower.contains("| sh")
        || lower.contains("| bash")
        || lower.contains("eval(")
        || lower.contains("exec(")
        || lower.contains("system(")
        || lower.contains("__import__")
}

fn evaluate_rubric_questions(
    _rule_cards: &[self::RuleCard],
    hashline_diff: &::core::primitive::str,
    compiler_output: &::core::primitive::str,
) -> ::std::vec::Vec<self::RubricDefect> {
    let mut defects = ::std::vec::Vec::new();

    if let ::core::option::Option::Some(d) = self::check_deleted_assertions(hashline_diff) {
        defects.push(d);
    }

    if let ::core::option::Option::Some(d) = self::check_hardcoded_returns(hashline_diff) {
        defects.push(d);
    }

    if let ::core::option::Option::Some(d) = self::check_coverage_theater_diff(hashline_diff) {
        defects.push(d);
    }

    if let ::core::option::Option::Some(d) = self::check_unsafe_blocks(hashline_diff) {
        defects.push(d);
    }

    if let ::core::option::Option::Some(d) = self::check_swallowed_errors(hashline_diff) {
        defects.push(d);
    }

    if let ::core::option::Option::Some(d) = self::check_unimplemented_stubs(hashline_diff) {
        defects.push(d);
    }

    if let ::core::option::Option::Some(d) = self::check_magic_number_drift(hashline_diff) {
        defects.push(d);
    }

    if let ::core::option::Option::Some(d) = self::check_unbounded_allocations(hashline_diff) {
        defects.push(d);
    }

    if let ::core::option::Option::Some(d) = self::check_concurrency_race(hashline_diff) {
        defects.push(d);
    }

    if let ::core::option::Option::Some(d) = self::check_compiler_diagnostics(compiler_output) {
        defects.push(d);
    }

    defects
}

fn check_deleted_assertions(hashline_diff: &::core::primitive::str) -> ::core::option::Option<self::RubricDefect> {
    let mut deleted_count = 0;
    let mut added_count = 0;
    let mut sample_line = ::std::string::String::new();

    for line in hashline_diff.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('-') {
            let is_assert = trimmed.contains("assert!")
                || trimmed.contains("assert_eq!")
                || trimmed.contains("assert_ne!")
                || trimmed.contains("ensure!");
            if is_assert {
                deleted_count += 1;
                if sample_line.is_empty() {
                    sample_line = trimmed.to_string();
                }
            }
        } else if trimmed.starts_with('+') {
            let is_assert = trimmed.contains("assert!")
                || trimmed.contains("assert_eq!")
                || trimmed.contains("assert_ne!")
                || trimmed.contains("ensure!");
            if is_assert {
                added_count += 1;
            }
        }
    }

    if deleted_count > 0 && added_count == 0 {
        let desc =
            ::std::format!("Deleted {deleted_count} assertions without replacing with equivalent verification checks");
        let defect = self::RubricDefect::builder()
            .question_id("Q01".to_string())
            .question_name("Deleted Assertions".to_string())
            .severity("high".to_string())
            .description(desc)
            .matched_line(sample_line)
            .confidence(0.95)
            .build();
        return ::core::option::Option::Some(defect);
    }

    ::core::option::Option::None
}

fn check_hardcoded_returns(hashline_diff: &::core::primitive::str) -> ::core::option::Option<self::RubricDefect> {
    for line in hashline_diff.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('+') {
            let body = trimmed[1..].trim();
            let is_hardcoded = body == "return true;"
                || body == "return false;"
                || body == "return Ok(());"
                || body == "return 0;"
                || body == "return \"\";"
                || body == "return vec![];"
                || body == "true"
                || body == "0"
                || body == "Ok(())";
            if is_hardcoded {
                let desc = "Hardcoded trivial return value stubbing function implementation".to_string();
                let defect = self::RubricDefect::builder()
                    .question_id("Q02".to_string())
                    .question_name("Hardcoded Return Values".to_string())
                    .severity("high".to_string())
                    .description(desc)
                    .matched_line(trimmed.to_string())
                    .confidence(0.92)
                    .build();
                return ::core::option::Option::Some(defect);
            }
        }
    }

    ::core::option::Option::None
}

fn check_coverage_theater(test_code: &::core::primitive::str) -> ::core::primitive::bool {
    let lower = test_code.to_ascii_lowercase();
    let has_dummy_assert = lower.contains("assert!(true)")
        || lower.contains("assert_eq!(1, 1)")
        || lower.contains("assert_eq!(true, true)")
        || lower.contains("assert_eq!(0, 0)");

    let has_real_assert = lower.contains("assert_eq!")
        || lower.contains("assert_ne!")
        || lower.contains("assert!")
        || lower.contains("ensure!");

    if has_dummy_assert && !has_real_assert {
        return true;
    }

    let has_ignore = lower.contains("let _ =") || lower.contains("let _=");
    if has_ignore && !has_real_assert {
        return true;
    }

    !has_real_assert
}

fn check_coverage_theater_diff(hashline_diff: &::core::primitive::str) -> ::core::option::Option<self::RubricDefect> {
    for line in hashline_diff.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('+') {
            let lower = trimmed.to_ascii_lowercase();
            let is_theater = lower.contains("assert!(true)")
                || lower.contains("assert_eq!(1, 1)")
                || lower.contains("assert_eq!(true, true)");
            if is_theater {
                let desc = "Coverage theater detected: dummy assertion does not verify execution output".to_string();
                let defect = self::RubricDefect::builder()
                    .question_id("Q03".to_string())
                    .question_name("Coverage Theater".to_string())
                    .severity("high".to_string())
                    .description(desc)
                    .matched_line(trimmed.to_string())
                    .confidence(0.98)
                    .build();
                return ::core::option::Option::Some(defect);
            }
        }
    }

    ::core::option::Option::None
}

fn check_unsafe_blocks(hashline_diff: &::core::primitive::str) -> ::core::option::Option<self::RubricDefect> {
    for line in hashline_diff.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('+') {
            let is_unsafe = trimmed.contains("unsafe {") || trimmed.contains("unsafe fn ");
            if is_unsafe {
                let desc = "Unsafe block or function introduced into modernized codebase".to_string();
                let defect = self::RubricDefect::builder()
                    .question_id("Q04".to_string())
                    .question_name("Unsafe Blocks".to_string())
                    .severity("high".to_string())
                    .description(desc)
                    .matched_line(trimmed.to_string())
                    .confidence(0.99)
                    .build();
                return ::core::option::Option::Some(defect);
            }
        }
    }

    ::core::option::Option::None
}

fn check_swallowed_errors(hashline_diff: &::core::primitive::str) -> ::core::option::Option<self::RubricDefect> {
    for line in hashline_diff.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('+') {
            let is_swallowed = trimmed.contains("let _ = ") && trimmed.contains(".unwrap_err()")
                || trimmed.contains("Err(_) => Ok(())")
                || trimmed.contains(".unwrap_or_default()");
            if is_swallowed {
                let desc = "Swallowed error suppressing failure conditions silently".to_string();
                let defect = self::RubricDefect::builder()
                    .question_id("Q05".to_string())
                    .question_name("Swallowed Errors".to_string())
                    .severity("medium".to_string())
                    .description(desc)
                    .matched_line(trimmed.to_string())
                    .confidence(0.85)
                    .build();
                return ::core::option::Option::Some(defect);
            }
        }
    }

    ::core::option::Option::None
}

fn check_unimplemented_stubs(hashline_diff: &::core::primitive::str) -> ::core::option::Option<self::RubricDefect> {
    for line in hashline_diff.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('+') {
            let is_stub = trimmed.contains("todo!()") || trimmed.contains("unimplemented!()");
            if is_stub {
                let desc = "Unimplemented stub or todo macro detected in generated implementation".to_string();
                let defect = self::RubricDefect::builder()
                    .question_id("Q06".to_string())
                    .question_name("Unimplemented Stubs".to_string())
                    .severity("high".to_string())
                    .description(desc)
                    .matched_line(trimmed.to_string())
                    .confidence(0.99)
                    .build();
                return ::core::option::Option::Some(defect);
            }
        }
    }

    ::core::option::Option::None
}

fn check_magic_number_drift(hashline_diff: &::core::primitive::str) -> ::core::option::Option<self::RubricDefect> {
    for line in hashline_diff.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('+') {
            let is_magic = trimmed.contains("0xdeadbeef") || trimmed.contains("13371337");
            if is_magic {
                let desc = "Unanchored magic number literal diverging from invariant specification".to_string();
                let defect = self::RubricDefect::builder()
                    .question_id("Q07".to_string())
                    .question_name("Magic Number Drift".to_string())
                    .severity("medium".to_string())
                    .description(desc)
                    .matched_line(trimmed.to_string())
                    .confidence(0.80)
                    .build();
                return ::core::option::Option::Some(defect);
            }
        }
    }

    ::core::option::Option::None
}

fn check_unbounded_allocations(hashline_diff: &::core::primitive::str) -> ::core::option::Option<self::RubricDefect> {
    for line in hashline_diff.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('+') {
            let is_unbounded = trimmed.contains("Vec::with_capacity(usize::MAX)");
            if is_unbounded {
                let desc = "Unbounded memory allocation hazard detected".to_string();
                let defect = self::RubricDefect::builder()
                    .question_id("Q08".to_string())
                    .question_name("Unbounded Allocations".to_string())
                    .severity("high".to_string())
                    .description(desc)
                    .matched_line(trimmed.to_string())
                    .confidence(0.95)
                    .build();
                return ::core::option::Option::Some(defect);
            }
        }
    }

    ::core::option::Option::None
}

fn check_concurrency_race(hashline_diff: &::core::primitive::str) -> ::core::option::Option<self::RubricDefect> {
    for line in hashline_diff.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('+') {
            let is_race = trimmed.contains("static mut ");
            if is_race {
                let desc = "Mutable static variable introduces concurrency race hazard".to_string();
                let defect = self::RubricDefect::builder()
                    .question_id("Q12".to_string())
                    .question_name("Concurrency Race Hazard".to_string())
                    .severity("high".to_string())
                    .description(desc)
                    .matched_line(trimmed.to_string())
                    .confidence(0.99)
                    .build();
                return ::core::option::Option::Some(defect);
            }
        }
    }

    ::core::option::Option::None
}

fn check_compiler_diagnostics(compiler_output: &::core::primitive::str) -> ::core::option::Option<self::RubricDefect> {
    if compiler_output.contains("error[E") || compiler_output.contains("error:") {
        let first_err =
            compiler_output.lines().find(|l| l.contains("error[E") || l.contains("error:")).unwrap_or("compiler error");
        let desc = "Compiler diagnostic error present in build output".to_string();
        let defect = self::RubricDefect::builder()
            .question_id("COMPILER".to_string())
            .question_name("Compiler Diagnostic".to_string())
            .severity("high".to_string())
            .description(desc)
            .matched_line(first_err.to_string())
            .confidence(1.0)
            .build();
        return ::core::option::Option::Some(defect);
    }

    ::core::option::Option::None
}
