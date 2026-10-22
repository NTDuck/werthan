//! Campaign configurator agent for the interactive `/mkcfg` wizard.
//!
//! Modelled after the omp `ask` workflow: the agent drives a series of typed
//! questions over one draft `CampaignConfig` state, collecting operator
//! answers through the shared `ask` tool contract. The frontend supplies an
//! `AskBridge` implementation (TUI question prompt, CLI stdin loop, or a Rig
//! agent loop) so this crate stays frontend-agnostic.

/// One typed question presented to the operator.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct ConfigQuestion {
    /// Stable question identifier used in answers and traces.
    pub id: ::std::string::String,
    /// Human readable question text.
    pub question: ::std::string::String,
    /// Suggested options. An empty vector means free-form text input.
    pub options: ::std::vec::Vec<::std::string::String>,
    /// Recommended option index, if any.
    pub recommended: ::core::option::Option<::core::primitive::usize>,
}

/// One collected operator answer keyed by question id.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct ConfigAnswer {
    /// Question id the answer belongs to.
    pub id: ::std::string::String,
    /// Selected option text or free-form answer.
    pub value: ::std::string::String,
}

/// Frontend-side bridge that surfaces questions and collects answers.
///
/// The interactive cockpit implements this with a TUI modal. A headless
/// frontend implements it with stdin. Tests implement it with canned
/// answers. The configurator never blocks on any specific UI toolkit.
pub trait AskBridge {
    /// Present one batch of questions and collect one answer per question.
    fn ask(
        &mut self,
        questions: ::std::vec::Vec<self::ConfigQuestion>,
    ) -> ::std::vec::Vec<self::ConfigAnswer>;
}

/// Conversational configurator agent holding the draft campaign state.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::bon::Builder)]
pub struct ConfiguratorAgent {
    /// Draft configuration mutated as answers arrive.
    #[builder(default)]
    pub draft: ::configschema::CampaignConfig,
}

impl self::ConfiguratorAgent {
    /// Drive the full wizard over the provided bridge and return the final
    /// draft configuration.
    ///
    /// The bridge receives the complete script in one call (omp `ask`
    /// semantics: one ask round-trip per batch) and returns answers keyed
    /// by question id. The wizard ignores answers with unknown ids.
    pub fn run_wizard(
        mut self,
        bridge: &mut dyn self::AskBridge,
    ) -> ::configschema::CampaignConfig {
        let script = self.question_script();
        let answers = bridge.ask(script);
        for answer in answers {
            self.apply_answer(&answer.id, &answer.value);
        }
        self.draft
    }
}

impl self::ConfiguratorAgent {
    /// Return the full question script for a fresh campaign draft.
    ///
    /// Order mirrors the wizard contract: identity, mode, paths, stack,
    /// then inference limits. Each question is self-contained so a
    /// frontend may render them one at a time or as a batch.
    pub fn question_script(&self) -> ::std::vec::Vec<self::ConfigQuestion> {
        ::std::vec![
            self::ConfigQuestion {
                id: "campaign_id".to_string(),
                question: "What campaign identifier should this run carry?"
                    .to_string(),
                options: ::std::vec![],
                recommended: ::core::option::Option::None,
            },
            self::ConfigQuestion {
                id: "execution_mode".to_string(),
                question: "Run in hitl (pause on gates) or full_auto mode?"
                    .to_string(),
                options: ::std::vec![
                    "hitl".to_string(),
                    "full_auto".to_string(),
                ],
                recommended: ::core::option::Option::Some(0),
            },
            self::ConfigQuestion {
                id: "input_legacy_dir".to_string(),
                question: "Where does the pristine legacy source tree live?"
                    .to_string(),
                options: ::std::vec![],
                recommended: ::core::option::Option::None,
            },
            self::ConfigQuestion {
                id: "workspace_root".to_string(),
                question: "Which scratchpad root should hold the translated workspace?"
                    .to_string(),
                options: ::std::vec![],
                recommended: ::core::option::Option::None,
            },
            self::ConfigQuestion {
                id: "source_language".to_string(),
                question: "Which source language does the legacy tree use?"
                    .to_string(),
                options: ::std::vec![
                    "c".to_string(),
                    "cpp".to_string(),
                    "go".to_string(),
                    "java".to_string(),
                    "python".to_string(),
                ],
                recommended: ::core::option::Option::Some(0),
            },
            self::ConfigQuestion {
                id: "target_language".to_string(),
                question: "Which target language should the codebase be translated to?"
                    .to_string(),
                options: ::std::vec![
                    "rust".to_string(),
                    "javascript".to_string(),
                    "python".to_string(),
                ],
                recommended: ::core::option::Option::Some(0),
            },
            self::ConfigQuestion {
                id: "target_build_cmd".to_string(),
                question: "Which command verifies the translated code compiles?"
                    .to_string(),
                options: ::std::vec![
                    "cargo check".to_string(),
                    "node -c translated.js".to_string(),
                    "python3 -m py_compile".to_string(),
                ],
                recommended: ::core::option::Option::Some(0),
            },
            self::ConfigQuestion {
                id: "target_test_cmd".to_string(),
                question: "Which command runs the translated test suite?"
                    .to_string(),
                options: ::std::vec![
                    "cargo test".to_string(),
                    "node translated.js".to_string(),
                    "pytest".to_string(),
                ],
                recommended: ::core::option::Option::Some(0),
            },
            self::ConfigQuestion {
                id: "max_self_heal_retries".to_string(),
                question: "How many self-heal retries should the solver loop get?"
                    .to_string(),
                options: ::std::vec![
                    "1".to_string(),
                    "3".to_string(),
                    "5".to_string(),
                ],
                recommended: ::core::option::Option::Some(1),
            },
            self::ConfigQuestion {
                id: "gate_on_reimagine".to_string(),
                question: "Require human signoff after S4 before S5 (hitl only)?"
                    .to_string(),
                options: ::std::vec![
                    "true".to_string(),
                    "false".to_string(),
                ],
                recommended: ::core::option::Option::Some(1),
            },
        ]
    }

    /// Apply one wizard answer onto the draft configuration.
    pub fn apply_answer(
        &mut self,
        answer_id: &::core::primitive::str,
        answer_value: &::core::primitive::str,
    ) {
        let value = answer_value.trim().to_string();
        let value_empty = value.is_empty();
        if value_empty {
            return;
        }
        match answer_id {
            "campaign_id" => self.draft.campaign_id = value,
            "execution_mode" => {
                let mode_res = ::serde_json::from_value::<
                    ::configschema::ExecutionMode,
                >(::serde_json::Value::String(value.clone()));
                if let ::core::result::Result::Ok(mode) = mode_res {
                    self.draft.execution_mode = mode;
                }
            },
            "input_legacy_dir" => {
                self.draft.paths.input_legacy_dir =
                    ::std::path::PathBuf::from(value);
            },
            "workspace_root" => {
                self.draft.paths.workspace_root =
                    ::std::path::PathBuf::from(value);
            },
            "source_language" => self.draft.stack.source_language = value,
            "target_language" => self.draft.stack.target_language = value,
            "target_build_cmd" => self.draft.stack.target_build_cmd = value,
            "target_test_cmd" => self.draft.stack.target_test_cmd = value,
            "max_self_heal_retries" => {
                let parsed_res = value.parse::<::core::primitive::u32>();
                if let ::core::result::Result::Ok(parsed) = parsed_res {
                    self.draft.verification.max_self_heal_retries = parsed;
                }
            },
            "gate_on_reimagine" => {
                let parsed_res = value.parse::<::core::primitive::bool>();
                if let ::core::result::Result::Ok(parsed) = parsed_res {
                    self.draft.hitl.gate_on_reimagine = parsed;
                }
            },
            _ => {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AskBridge;

    /// Canned-answer bridge returning fixed values for every question.
    #[derive(::core::default::Default)]
    struct CannedBridge {
        answers: ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
    }

    impl self::AskBridge for self::CannedBridge {
        fn ask(
            &mut self,
            questions: ::std::vec::Vec<super::ConfigQuestion>,
        ) -> ::std::vec::Vec<super::ConfigAnswer> {
            questions
                .into_iter()
                .map(|question: super::ConfigQuestion| {
                    let canned = self
                        .answers
                        .iter()
                        .find(|(canned_id, _)| canned_id == &question.id);
                    let value = match canned {
                        ::core::option::Option::Some((_, canned_value)) => {
                            canned_value.clone()
                        },
                        ::core::option::Option::None => {
                            question
                                .options
                                .first()
                                .cloned()
                                .unwrap_or_default()
                        },
                    };
                    let id = question.id;
                    super::ConfigAnswer {
                        id,
                        value,
                    }
                })
                .collect()
        }
    }

    #[test]
    fn script_covers_core_sections() {
        let agent = super::ConfiguratorAgent::builder().build();
        let script = agent.question_script();
        let ids = script
            .iter()
            .map(|question| -> ::std::string::String { question.id.clone() })
            .collect::<::std::vec::Vec<::std::string::String>>();
        ::core::assert!(ids.iter().any(|id_text| id_text == "campaign_id"));
        ::core::assert!(ids.iter().any(|id_text| id_text == "execution_mode"));
        ::core::assert!(ids.iter().any(|id_text| id_text == "input_legacy_dir"));
        ::core::assert!(ids.iter().any(|id_text| id_text == "target_language"));
        ::core::assert!(ids.iter().any(|id_text| id_text == "gate_on_reimagine"));
    }

    #[test]
    fn wizard_applies_answers_onto_draft() {
        let agent = super::ConfiguratorAgent::builder().build();
        let mut bridge = self::CannedBridge {
            answers: ::std::vec![
                (
                    "campaign_id".to_string(),
                    "my-campaign".to_string()
                ),
                (
                    "execution_mode".to_string(),
                    "full_auto".to_string()
                ),
                (
                    "input_legacy_dir".to_string(),
                    "legacy/src".to_string()
                ),
                (
                    "workspace_root".to_string(),
                    "workspace/out".to_string()
                ),
                (
                    "source_language".to_string(),
                    "python".to_string()
                ),
                (
                    "target_language".to_string(),
                    "javascript".to_string()
                ),
                (
                    "max_self_heal_retries".to_string(),
                    "5".to_string()
                ),
                (
                    "gate_on_reimagine".to_string(),
                    "true".to_string()
                ),
            ],
        };
        let draft = agent.run_wizard(&mut bridge);
        ::core::assert_eq!(draft.campaign_id, "my-campaign");
        ::core::assert_eq!(
            draft.execution_mode,
            ::configschema::ExecutionMode::FullAuto
        );
        let legacy_path = ::std::path::PathBuf::from("legacy/src");
        ::core::assert_eq!(draft.paths.input_legacy_dir, legacy_path);
        ::core::assert_eq!(draft.stack.source_language, "python");
        ::core::assert_eq!(draft.stack.target_language, "javascript");
        ::core::assert_eq!(draft.verification.max_self_heal_retries, 5);
        ::core::assert!(draft.hitl.gate_on_reimagine);
    }

    #[test]
    fn empty_and_unknown_answers_are_ignored() {
        let agent = super::ConfiguratorAgent::builder().build();
        let mut bridge = self::CannedBridge {
            answers: ::std::vec![
                ("unknown_field".to_string(), "zzz".to_string()),
                ("campaign_id".to_string(), "   ".to_string()),
            ],
        };
        let draft = agent.run_wizard(&mut bridge);
        let empty_draft = ::configschema::CampaignConfig::default();
        ::core::assert_eq!(draft.campaign_id, empty_draft.campaign_id);
    }

    #[test]
    fn serialized_draft_round_trips_to_yaml() {
        let agent = super::ConfiguratorAgent::builder().build();
        let mut bridge = self::CannedBridge {
            answers: ::std::vec![
                (
                    "campaign_id".to_string(),
                    "yaml-campaign".to_string()
                ),
                ("source_language".to_string(), "c".to_string()),
            ],
        };
        let draft = agent.run_wizard(&mut bridge);
        let yaml_text = draft.to_yaml_text();
        let reparsed_res = ::configschema::CampaignConfig::from_yaml_text(&yaml_text);
        let reparsed = reparsed_res.expect("yaml round trip parses");
        ::core::assert_eq!(reparsed.campaign_id, "yaml-campaign");
        ::core::assert_eq!(reparsed.stack.source_language, "c");
    }
}
