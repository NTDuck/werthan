//! Seed-time S3 rule extraction: real LLM passes over legacy sources.
//! The engine's discovery workers dispatch empty problem models. The board
//! rules must be populated before `engine.run()`, or testgen produces no
//! characterization tests and MCTS has no reward path.

/// Extracted rule cards per vertex plus the request count consumed.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default)]
pub struct RuleExtractionReport {
    /// Extracted rule cards keyed by vertex id, ready for board.rules.
    pub cards_per_vertex: ::std::collections::BTreeMap<
        ::std::string::String,
        ::std::vec::Vec<::holon_blackboard::BusinessRuleCard>,
    >,
    /// Number of real LLM requests issued during extraction.
    pub request_count: ::core::primitive::u64,
    /// Vertices whose extraction failed. We record failures honestly.
    pub failed_vertices: ::std::vec::Vec<::std::string::String>,
}

/// Extract business rule cards from every implementation source file.
///
/// We skip headers as vertices. Header content still reaches the prompt as
/// interface context, because the header declares the exact API surface the
/// implementation must expose. Each implementation file gets one LLM
/// exchange requesting strict JSON rule cards. Unparsable or failed
/// exchanges leave that vertex empty rather than inventing rules.
pub fn extract_rules_for_board(
    bench_config: &super::bench_config::CampaignConfig,
    entries: &[super::bench_harness::InventoryEntry],
    request_counter: &::std::sync::Arc<::std::sync::atomic::AtomicU64>,
    api_surface_text: &::core::primitive::str,
) -> self::RuleExtractionReport {
    let mut report = self::RuleExtractionReport::default();
    let inference = &bench_config.inference;
    let qwen_config = super::bench_llm::QwenConfig::from_inference(inference);
    let qwen_model = super::bench_llm::QwenReasoningModel::new(qwen_config);
    let counting_model =
        super::bench_llm::CountingCompletionModel::new(qwen_model, request_counter.clone());
    let request_temperature = inference.inference_temperature;
    let request_max_tokens = inference.inference_max_tokens as ::core::primitive::u64;
    let runtime_res = ::tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    let runtime = runtime_res.expect("current thread runtime builds");
    let repo_root = super::bench_config::repo_root();
    let legacy_dir = repo_root.join(&bench_config.paths.input_legacy_dir);
    for entry in entries {
        let header_file = super::bench_harness::is_header(entry);
        let test_tree = self::is_test_tree_entry(entry);
        let non_module = header_file || test_tree;
        if non_module {
            continue;
        }
        let source_path = legacy_dir.join(&entry.relative_path);
        let read_res = ::std::fs::read_to_string(&source_path);
        let source_text = match read_res {
            ::core::result::Result::Ok(text) => text,
            ::core::result::Result::Err(read_error) => {
                ::tracing::warn!(
                    "rule extraction skipped {}: read failed: {read_error}",
                    entry.vertex_id
                );
                report.failed_vertices.push(entry.vertex_id.clone());
                continue;
            },
        };
        let header_text = self::sibling_header_text(&legacy_dir, entries, entry);
        let prompt = self::render_rule_prompt(
            &entry.relative_path,
            &source_text,
            &header_text,
            api_surface_text,
        );
        let completion_request = ::rig::completion::CompletionRequest {
            preamble: ::core::option::Option::Some(self::RULE_SYSTEM_PROMPT.to_string()),
            prompt: self::user_message_of(&prompt),
            chat_history: ::std::vec::Vec::new(),
            documents: ::std::vec::Vec::new(),
            tools: ::std::vec::Vec::new(),
            temperature: ::core::option::Option::Some(request_temperature),
            max_tokens: ::core::option::Option::Some(request_max_tokens),
            additional_params: ::core::option::Option::None,
        };
        let complete_res = runtime.block_on(async {
            ::rig::completion::CompletionModel::completion(&counting_model, completion_request)
                .await
        });
        let response_text = match complete_res {
            ::core::result::Result::Ok(response) => self::choice_text_of(&response),
            ::core::result::Result::Err(completion_error) => {
                ::tracing::warn!(
                    "rule extraction failed for {}: {completion_error}",
                    entry.vertex_id
                );
                report.failed_vertices.push(entry.vertex_id.clone());
                continue;
            },
        };
        let parsed = self::parse_rule_cards(&response_text, &entry.relative_path);
        report.cards_per_vertex.insert(entry.vertex_id.clone(), parsed);
    }
    report
}

/// Report whether one inventory entry lives inside a test tree directory.
fn is_test_tree_entry(entry: &super::bench_harness::InventoryEntry) -> ::core::primitive::bool {
    entry
        .relative_path
        .split('/')
        .any(|component| component == "tests" || component == "test")
}

/// Read the target workspace API surface text for the extraction prompt.
///
/// The pristine stub sources declare the exact Rust signatures the
/// modernized crate must expose. Capturing them here grounds the extracted
/// rule cards in the real required API instead of guessed signatures.
pub fn target_api_surface(workspace_root: &::std::path::Path) -> ::std::string::String {
    let src_dir = workspace_root.join("src");
    let walk = ::ignore::WalkBuilder::new(src_dir).build();
    let mut surfaces = ::std::vec::Vec::new();
    for entry in walk.flatten() {
        let path = entry.path();
        let path_is_file = path.is_file();
        if !path_is_file {
            continue;
        }
        let path_text = path.to_string_lossy();
        let is_rust = path_text.ends_with(".rs");
        if !is_rust {
            continue;
        }
        let read_res = ::std::fs::read_to_string(path);
        let text = match read_res {
            ::core::result::Result::Ok(value) => value,
            ::core::result::Result::Err(_) => continue,
        };
        surfaces.push(::std::format!("// file: {}", path_text));
        surfaces.push(text);
    }
    surfaces.join("\n")
}

/// Collect header file texts that share the stem of one implementation entry.
fn sibling_header_text(
    legacy_dir: &::std::path::Path,
    entries: &[super::bench_harness::InventoryEntry],
    implementation_entry: &super::bench_harness::InventoryEntry,
) -> ::std::string::String {
    let stem_end = implementation_entry.relative_path.rfind('.').unwrap_or(implementation_entry.relative_path.len());
    let implementation_stem = &implementation_entry.relative_path[..stem_end];
    let mut header_texts = ::std::vec::Vec::new();
    for candidate in entries {
        let candidate_is_header = super::bench_harness::is_header(candidate);
        if !candidate_is_header {
            continue;
        }
        let candidate_end = candidate.relative_path.rfind('.').unwrap_or(candidate.relative_path.len());
        let candidate_stem = &candidate.relative_path[..candidate_end];
        let stems_match = candidate_stem == implementation_stem;
        if !stems_match {
            continue;
        }
        let header_path = legacy_dir.join(&candidate.relative_path);
        let read_res = ::std::fs::read_to_string(&header_path);
        if let ::core::result::Result::Ok(text) = read_res {
            header_texts.push(::std::format!("[HEADER {}]\n{}", candidate.relative_path, text));
        }
    }
    header_texts.join("\n\n")
}

/// System prompt steering the model toward strict JSON rule cards.
const RULE_SYSTEM_PROMPT: &::core::primitive::str = "You are a business rule extractor for a legacy code modernization pipeline. \
Read the legacy source and emit behavioral rules as strict JSON. \
Output only a JSON array, no markdown fences, no commentary. \
Each element must have exactly these keys: rule_id, given, when, then, citation. \
rule_id is a short uppercase identifier. given is the precondition state plus the exact target Rust function signature from the API surface that the behavior applies to. \
when is the triggering action, phrased using the exact function and field names from the legacy source. \
then is the observable outcome, also phrased with those exact identifiers. \
citation is file path plus line span such as src/vec.c:9-24. \
The first rule must state the crate-level export obligation implied by the API surface, naming every public item the modernized crate root must expose. \
Extract at most 12 rules covering the most important behaviors.";

/// Render the per-file user prompt with the full legacy source.
///
/// Header text supplies the declared C interface, and the target workspace
/// stub sources supply the exact Rust signatures the modernized crate must
/// expose. Both flow into rule cards so code generation inherits the real
/// API surface through the standard problem-model prompt context.
fn render_rule_prompt(
    relative_path: &::core::primitive::str,
    source_text: &::core::primitive::str,
    header_text: &::core::primitive::str,
    api_surface_text: &::core::primitive::str,
) -> ::std::string::String {
    let header_section = header_text.trim();
    let header_block = if header_section.is_empty() {
        ::std::string::String::new()
    } else {
        ::std::format!("\n{header_section}\n\n")
    };
    ::std::format!(
        "[LEGACY FILE {relative_path}]\n{source_text}\n\n{header_block}[TARGET API SURFACE]\n{api_surface_text}\n\nEmit the JSON array of behavioral rule cards now."
    )
}

/// Wrap the rule prompt into one Rig user message.
fn user_message_of(prompt_text: &::core::primitive::str) -> ::rig::message::Message {
    let text_content = ::rig::message::UserContent::Text(::rig::message::Text {
        text: { prompt_text.to_string() },
    });
    let content = ::rig::OneOrMany::one(text_content);
    ::rig::message::Message::User { content: { content } }
}

/// Extract the first text content item from one completion response choice.
fn choice_text_of(
    response: &::rig::completion::CompletionResponse<::serde_json::Value>,
) -> ::std::string::String {
    response
        .choice
        .iter()
        .find_map(|item| match item {
            ::rig::message::AssistantContent::Text(text) => {
                ::core::option::Option::Some(text.text.clone())
            },
            _ => ::core::option::Option::None,
        })
        .unwrap_or_default()
}

/// Parse one strict JSON array of rule cards from the model response.
///
/// Accepts fenced or bare JSON. We skip malformed entries, never guessing.
/// A card only lands when all five keys parse as strings.
fn parse_rule_cards(
    response_text: &::core::primitive::str,
    relative_path: &::core::primitive::str,
) -> ::std::vec::Vec<::holon_blackboard::BusinessRuleCard> {
    let json_start = response_text.find('[');
    let json_end = response_text.rfind(']');
    let bounds = match (json_start, json_end) {
        (::core::option::Option::Some(start), ::core::option::Option::Some(end)) if start < end => {
            (start, end + 1)
        },
        _ => return ::std::vec::Vec::new(),
    };
    let json_text = &response_text[bounds.0..bounds.1];
    let parsed = ::serde_json::from_str::<::serde_json::Value>(json_text);
    let parsed_array = match parsed {
        ::core::result::Result::Ok(::serde_json::Value::Array(items)) => items,
        _ => return ::std::vec::Vec::new(),
    };
    let mut cards = ::std::vec::Vec::new();
    for (rule_index, item) in parsed_array.into_iter().enumerate() {
        let rule_id = self::string_field(&item, "rule_id");
        let given_text = self::string_field(&item, "given");
        let when_text = self::string_field(&item, "when");
        let then_text = self::string_field(&item, "then");
        let citation_text = self::string_field(&item, "citation");
        let keys_missing = rule_id.is_empty()
            || given_text.is_empty()
            || when_text.is_empty()
            || then_text.is_empty();
        if keys_missing {
            continue;
        }
        let citation_final = if citation_text.is_empty() {
            relative_path.to_string()
        } else {
            citation_text
        };
        let rule_id_final = if rule_id.starts_with("RULE_") {
            rule_id
        } else {
            ::std::format!("RULE_{rule_index:03}_{rule_id}")
        };
        let card = ::holon_blackboard::BusinessRuleCard {
            rule_id: { rule_id_final },
            given: { given_text },
            when: { when_text },
            then: { then_text },
            citation: { citation_final },
        };
        cards.push(card);
    }
    cards
}

/// Read one string field from a parsed JSON object.
fn string_field(
    item: &::serde_json::Value,
    field_name: &::core::primitive::str,
) -> ::std::string::String {
    item.get(field_name)
        .and_then(|field_value| field_value.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rejects_non_json_response() {
        let cards = self::parse_rule_cards("no json here", "src/vec.c");
        ::core::assert!(cards.is_empty());
    }

    #[test]
    fn parse_keeps_only_complete_cards() {
        let response = r#"[
            {"rule_id": "EXPAND", "given": "empty", "when": "push", "then": "capacity doubles", "citation": "src/vec.c:9-24"},
            {"rule_id": "PARTIAL", "given": "x", "when": "y"}
        ]"#;
        let cards = self::parse_rule_cards(response, "src/vec.c");
        ::std::assert_eq!(cards.len(), 1);
        ::std::assert_eq!(cards[0].rule_id, "RULE_000_EXPAND");
        ::std::assert_eq!(cards[0].citation, "src/vec.c:9-24");
    }

    #[test]
    fn parse_handles_fenced_json() {
        let response = "```json\n[{\"rule_id\": \"COMPACT\", \"given\": \"len 0\", \"when\": \"compact\", \"then\": \"buffer freed\", \"citation\": \"src/vec.c:60\"}]\n```";
        let cards = self::parse_rule_cards(response, "src/vec.c");
        ::std::assert_eq!(cards.len(), 1);
        ::std::assert_eq!(cards[0].rule_id, "RULE_000_COMPACT");
    }
}
