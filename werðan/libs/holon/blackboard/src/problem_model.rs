//! Problem model slice B_PM per METHODOLOGY.md Definition 2.
//! Holds rule cards, parent contracts, delta constraints, export obligations,
//! and legacy source excerpts hydrated deterministically from the global blackboard G.

/// Extracted business rule card with source citation.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct BusinessRuleCard {
    /// Unique identifier for the business rule.
    pub rule_id: ::std::string::String,
    /// Precondition description in Given clause.
    pub given: ::std::string::String,
    /// Trigger event description in When clause.
    pub when: ::std::string::String,
    /// Expected outcome description in Then clause.
    pub then: ::std::string::String,
    /// Source citation pointing to legacy file and line number.
    pub citation: ::std::string::String,
}

/// Interface contract exported by an upstream dependency edge.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct EdgeContract {
    /// Parent module vertex identifier.
    pub parent_id: ::std::string::String,
    /// Target language type signature or header specification.
    pub interface_spec: ::std::string::String,
    /// Behavioral preconditions required by parent.
    pub preconditions: ::std::vec::Vec<::std::string::String>,
}

/// Problem model slice containing legacy specifications and contract bounds.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProblemModelSlice {
    /// Extracted business logic rule cards.
    pub rules: ::std::vec::Vec<self::BusinessRuleCard>,
    /// Upstream dependency interface contracts.
    pub parent_contracts: ::std::vec::Vec<self::EdgeContract>,
    /// Accumulated contract delta constraints from regressions.
    pub delta_constraints: ::std::vec::Vec<::std::string::String>,
    /// Legacy source code snippets for reference.
    pub legacy_source_slices: ::std::vec::Vec<::std::string::String>,
    /// Downstream export requirements.
    pub export_obligations: ::std::vec::Vec<::std::string::String>,
    /// Target language name the translation must emit, defaulting to rust.
    pub target_language: ::std::string::String,
}

impl self::ProblemModelSlice {
    /// Create empty problem model slice with the rust target default.
    pub fn new() -> Self {
        Self {
            target_language: "rust".to_string(),
            ..Self::default()
        }
    }

    /// Add verified business rule card to slice.
    pub fn add_rule(&mut self, rule: self::BusinessRuleCard) {
        self.rules.push(rule);
    }

    /// Add upstream parent interface contract to slice.
    pub fn add_parent_contract(&mut self, contract: self::EdgeContract) {
        self.parent_contracts.push(contract);
    }

    /// Append regression delta constraint to problem model.
    pub fn add_delta_constraint(&mut self, delta: ::std::string::String) {
        self.delta_constraints.push(delta);
    }

    /// Count extracted rules in slice.
    pub fn rule_count(&self) -> ::core::primitive::usize {
        self.rules.len()
    }

    /// Count parent interface contracts in slice.
    pub fn parent_contract_count(&self) -> ::core::primitive::usize {
        self.parent_contracts.len()
    }

    /// Render the problem model as structured context text for LLM prompts.
    /// Covers rule cards, parent contracts, delta constraints, obligations,
    /// and legacy source excerpts in a deterministic serial order.
    pub fn render_prompt_context(&self) -> ::std::string::String {
        let mut context_lines = ::std::vec::Vec::<::std::string::String>::new();
        context_lines.push("[TARGET LANGUAGE]".to_string());
        context_lines.push(self.target_language.clone());
        context_lines.push("[BUSINESS RULES]".to_string());
        if self.rules.is_empty() {
            context_lines.push("(none extracted)".to_string());
        }
        for rule_entry in &self.rules {
            let rule_line = ::std::format!(
                "- {} | Given: {} | When: {} | Then: {} | Source: {}",
                rule_entry.rule_id,
                rule_entry.given,
                rule_entry.when,
                rule_entry.then,
                rule_entry.citation,
            );
            context_lines.push(rule_line);
        }
        context_lines.push("[PARENT CONTRACTS]".to_string());
        if self.parent_contracts.is_empty() {
            context_lines.push("(none recorded)".to_string());
        }
        for contract_entry in &self.parent_contracts {
            context_lines.push(::std::format!(
                "- {} | Interface: {}",
                contract_entry.parent_id,
                contract_entry.interface_spec,
            ));
            for precondition_line in &contract_entry.preconditions {
                context_lines.push(::std::format!("  precondition: {precondition_line}"));
            }
        }
        context_lines.push("[DELTA CONSTRAINTS]".to_string());
        for delta_line in &self.delta_constraints {
            context_lines.push(::std::format!("- {delta_line}"));
        }
        context_lines.push("[EXPORT OBLIGATIONS]".to_string());
        for obligation_line in &self.export_obligations {
            context_lines.push(::std::format!("- {obligation_line}"));
        }
        context_lines.push("[LEGACY SOURCE EXCERPTS]".to_string());
        for source_excerpt in &self.legacy_source_slices {
            context_lines.push("-----".to_string());
            context_lines.push(source_excerpt.clone());
        }
        context_lines.join("\n")
    }
}

#[cfg(test)]
mod prompt_context_tests {
    use super::*;

    #[test]
    fn test_render_prompt_context_sections() {
        let mut problem_model = self::ProblemModelSlice::new();
        let rule_card = self::BusinessRuleCard {
            rule_id: "RULE_PHRI_001".to_string(),
            given: "empty input".to_string(),
            when: "transform".to_string(),
            then: "identity output".to_string(),
            citation: "legacy/src.rs:42".to_string(),
        };
        problem_model.add_rule(rule_card);
        let rendered = problem_model.render_prompt_context();
        ::core::assert!(rendered.contains("[BUSINESS RULES]"));
        ::core::assert!(rendered.contains("RULE_PHRI_001"));
        ::core::assert!(rendered.contains("[PARENT CONTRACTS]"));
        ::core::assert!(rendered.contains("[LEGACY SOURCE EXCERPTS]"));
    }

    #[test]
    fn test_render_prompt_context_target_language() {
        let mut problem_model = self::ProblemModelSlice::new();
        problem_model.target_language = "javascript".to_string();
        let rendered = problem_model.render_prompt_context();
        ::core::assert!(rendered.contains("[TARGET LANGUAGE]\njavascript"));
    }

    #[test]
    fn test_parent_contract_recording() {
        let mut problem_model = self::ProblemModelSlice::new();
        let contract = self::EdgeContract {
            parent_id: "vertex_parent".to_string(),
            interface_spec: "pub fn parse(input: &str) -> Tree".to_string(),
            preconditions: ::std::vec::Vec::new(),
        };
        problem_model.add_parent_contract(contract);
        ::core::assert_eq!(problem_model.parent_contract_count(), 1);
        ::core::assert_eq!(problem_model.parent_contracts.len(), 1);
    }
}
