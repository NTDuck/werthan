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
    let compat_model_res = super::bench_llm::OpenAiCompatModel::from_inference(inference);
    let compat_model = match compat_model_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(_config_failure) => {
            report.failed_vertices = entries.iter().map(|entry| entry.vertex_id.clone()).collect();
            return report;
        },
    };
    let counting_model =
        super::bench_llm::CountingCompletionModel::new(compat_model, request_counter.clone());
    let request_temperature = inference.inference_temperature;
    let request_max_tokens = inference.inference_max_tokens as ::core::primitive::u64;
    let runtime_res = ::tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    let runtime = runtime_res.expect("current thread runtime builds");
    let repo_root = super::bench_config::repo_root();
    let legacy_dir = repo_root.join(&bench_config.paths.input_legacy_dir);
    let source_language = ::core::clone::Clone::clone(&bench_config.stack.source_language);
    let target_language = ::core::clone::Clone::clone(&bench_config.stack.target_language);
    for entry in entries {
        let header_file = super::bench_harness::is_header(entry, &source_language);
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
        let header_text = self::sibling_header_text(&legacy_dir, entries, entry, &source_language);
        let prompt = self::render_rule_prompt(
            &entry.relative_path,
            &source_text,
            &header_text,
            api_surface_text,
        );
        let system_prompt = self::rule_system_prompt(&target_language);
        let completion_request = ::rig::completion::CompletionRequest {
            preamble: ::core::option::Option::Some(system_prompt),
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
/// The pristine stub sources declare the exact signatures the modernized
/// target must expose. The walk selects files by the target language
/// extension, so rust campaigns read `.rs` sources while other targets read
/// their own source extension. Capturing the surface here grounds the
/// extracted rule cards in the real required API instead of guessed
/// signatures.
/// Capture the target API surface from the pristine dataset directory or workspace.
///
/// When the workspace root is under `benchmarks/outputs/`, this resolves the
/// corresponding pristine `dataset_dir` to capture target signatures before
/// any workspace purging occurs.
pub fn target_api_surface(
    workspace_root: &::std::path::Path,
    target_language: &::core::primitive::str,
) -> ::std::string::String {
    let dataset_dir_opt = self::dataset_dir_for_workspace(workspace_root);
    let effective_root = match &dataset_dir_opt {
        ::core::option::Option::Some(dataset_dir) => dataset_dir.as_path(),
        ::core::option::Option::None => workspace_root,
    };
    let target_extension = self::target_source_extension(target_language);
    let src_dir = effective_root.join("src");
    let walk_builder = ::ignore::WalkBuilder::new(src_dir);
    let walk = walk_builder.build();
    let mut surfaces = ::std::vec::Vec::new();
    for entry_res in walk.flatten() {
        let entry_path = entry_res.path();
        let path_is_file = entry_path.is_file();
        if !path_is_file {
            continue;
        }
        let path_text = entry_path.to_string_lossy();
        let matches_target = path_text.ends_with(target_extension);
        if !matches_target {
            continue;
        }
        let read_res = ::std::fs::read_to_string(entry_path);
        let text = match read_res {
            ::core::result::Result::Ok(value) => value,
            ::core::result::Result::Err(_) => continue,
        };
        let file_header = ::std::format!("// file: {}", path_text);
        surfaces.push(file_header);
        surfaces.push(text);
    }
    surfaces.join("\n")
}

/// Resolve the pristine dataset directory corresponding to a target workspace root.
pub fn dataset_dir_for_workspace(
    workspace_root: &::std::path::Path,
) -> ::core::option::Option<::std::path::PathBuf> {
    let repo_root = super::bench_config::repo_root();
    let root_path_opt = workspace_root.strip_prefix(&repo_root);
    let stripped_root = match root_path_opt {
        ::core::result::Result::Ok(stripped) => stripped,
        ::core::result::Result::Err(_) => workspace_root,
    };
    let root_lossy = stripped_root.to_string_lossy();
    let root_text = root_lossy.to_string();
    let repomod_prefix = "benchmarks/outputs/RepoMod-bench/";
    if let ::core::option::Option::Some(remainder) = root_text.strip_prefix(repomod_prefix) {
        let project_name = remainder
            .split('/')
            .find(|segment| !segment.is_empty())
            .unwrap_or(remainder);
        if !project_name.is_empty() {
            let candidate = repo_root
                .join("benchmarks/assets/RepoMod-bench/dataset")
                .join(project_name)
                .join("dst");
            let candidate_is_dir = candidate.is_dir();
            if candidate_is_dir {
                return ::core::option::Option::Some(candidate);
            }
        }
        return ::core::option::Option::None;
    }
    let outputs_prefix = "benchmarks/outputs/";
    let starts_with_prefix = root_text.starts_with(outputs_prefix);
    if !starts_with_prefix {
        return ::core::option::Option::None;
    }
    let dataset_prefix = "benchmarks/assets/ReCodeAgent/data/tool_projects/";
    let dataset_relative = root_text.replacen(outputs_prefix, dataset_prefix, 1);
    let candidate = repo_root.join(dataset_relative);
    let candidate_is_dir = candidate.is_dir();
    if candidate_is_dir {
        ::core::option::Option::Some(candidate)
    } else {
        ::core::option::Option::None
    }
}

/// Return the source file extension the target language uses.
///
/// Unknown or unset target languages fall back to the rust extension, which
/// preserves the historical default behavior.
pub fn target_source_extension(target_language: &::core::primitive::str) -> &'static ::core::primitive::str {
    match target_language {
        "javascript" | "js" => ".js",
        "python" | "py" => ".py",
        _ => ".rs",
    }
}


/// Collect header file texts that share the stem of one implementation entry.
fn sibling_header_text(
    legacy_dir: &::std::path::Path,
    entries: &[super::bench_harness::InventoryEntry],
    implementation_entry: &super::bench_harness::InventoryEntry,
    source_language: &::core::primitive::str,
) -> ::std::string::String {
    let stem_end = implementation_entry.relative_path.rfind('.').unwrap_or(implementation_entry.relative_path.len());
    let implementation_stem = &implementation_entry.relative_path[..stem_end];
    let mut header_texts = ::std::vec::Vec::new();
    for candidate in entries {
        let candidate_is_header = super::bench_harness::is_header(candidate, source_language);
        if !candidate_is_header {
            continue;
        }
        let candidate_end = candidate.relative_path.rfind('.').unwrap_or(candidate.relative_path.len());
        let candidate_stem = &candidate.relative_path[..candidate_end];
        let candidate_file_stem = ::std::path::Path::new(&candidate.relative_path)
            .file_stem()
            .and_then(|stem_os| stem_os.to_str())
            .unwrap_or("");
        let implementation_file_stem = ::std::path::Path::new(&implementation_entry.relative_path)
            .file_stem()
            .and_then(|stem_os| stem_os.to_str())
            .unwrap_or("");
        let stems_match = candidate_stem == implementation_stem
            || (!candidate_file_stem.is_empty() && candidate_file_stem == implementation_file_stem);
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

/// Render the per-file user prompt with the full legacy source.
///
/// Header text supplies the declared legacy interface, and the target
/// workspace stub sources supply the exact target-language signatures the
/// modernized target must expose. Both flow into rule cards so code
/// generation inherits the real API surface through the standard
/// problem-model prompt context.
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

/// Build the system prompt steering the model toward strict JSON rule cards.
///
/// The target language appears in two guidance fragments so the extractor
/// requests signatures in the target language rather than assuming rust.
fn rule_system_prompt(target_language: &::core::primitive::str) -> ::std::string::String {
    ::std::format!(
        "You are a business rule extractor for a legacy code modernization pipeline. \
Read the legacy source and emit behavioral rules as strict JSON. \
Output only a JSON array, no markdown fences, no commentary. \
Each element must have exactly these keys: rule_id, given, when, then, citation. \
rule_id is a short uppercase identifier. given is the precondition state plus the exact target {target_language} function signature from the API surface that the behavior applies to. \
when is the triggering action, phrased using the exact function and field names from the legacy source. \
then is the observable outcome, also phrased with those exact identifiers. \
citation is file path plus line span such as src/vec.c:9-24. \
The first rule must state the module-level export obligation implied by the API surface, naming every public item the modernized target must expose. \
Extract at most 12 rules covering the most important behaviors."
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

/// Wire format of one raw rule card candidate inside the model response.
#[derive(::serde::Deserialize)]
struct RawRuleCard {
    rule_id: ::core::option::Option<::std::string::String>,
    citation: ::core::option::Option<::std::string::String>,
    given: ::core::option::Option<::std::string::String>,
    when: ::core::option::Option<::std::string::String>,
    then: ::core::option::Option<::std::string::String>,
}

/// Parse one strict JSON array of rule cards from the model response.
///
/// Accepts fenced or bare JSON. We skip malformed entries, never guessing.
/// A card only lands when all five keys parse as strings.
fn parse_rule_cards(
    response_text: &::core::primitive::str,
    relative_path: &::core::primitive::str,
) -> ::std::vec::Vec<::holon_blackboard::BusinessRuleCard> {
    let json_slice = match (response_text.find('['), response_text.rfind(']')) {
        (::core::option::Option::Some(start_idx), ::core::option::Option::Some(end_idx))
            if start_idx < end_idx =>
        {
            &response_text[start_idx..=end_idx]
        }
        _ => return ::std::vec::Vec::new(),
    };
    let raw_cards = match ::serde_json::from_str::<::std::vec::Vec<self::RawRuleCard>>(json_slice) {
        ::core::result::Result::Ok(items) => items,
        ::core::result::Result::Err(_) => return ::std::vec::Vec::new(),
    };
    raw_cards
        .into_iter()
        .enumerate()
        .filter_map(|(rule_index, raw_card)| {
            let rule_id = raw_card.rule_id?.trim().to_string();
            let given_text = raw_card.given?.trim().to_string();
            let when_text = raw_card.when?.trim().to_string();
            let then_text = raw_card.then?.trim().to_string();
            if rule_id.is_empty()
                || given_text.is_empty()
                || when_text.is_empty()
                || then_text.is_empty()
            {
                return ::core::option::Option::None;
            }
            let citation_final = raw_card
                .citation
                .as_deref()
                .map(::core::primitive::str::trim)
                .filter(|trimmed| !trimmed.is_empty())
                .unwrap_or(relative_path)
                .to_string();
            let rule_id_final = if rule_id.starts_with("RULE_") {
                rule_id
            } else {
                ::std::format!("RULE_{rule_index:03}_{rule_id}")
            };
            ::core::option::Option::Some(::holon_blackboard::BusinessRuleCard {
                rule_id: { rule_id_final },
                given: { given_text },
                when: { when_text },
                then: { then_text },
                citation: { citation_final },
            })
        })
        .collect::<::std::vec::Vec<_>>()
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

    #[test]
    fn target_api_surface_resolves_pristine_dataset_for_matrix_multiplication() {
        let repo_root = super::super::bench_config::repo_root();
        let workspace_root = repo_root.join("benchmarks/outputs/crust/matrix_multiplication/rust");
        let surface = super::target_api_surface(&workspace_root, "rust");
        ::std::assert!(surface.contains("pub fn print_array"));
        ::std::assert!(surface.contains("pub fn multiply"));
    }

    #[test]
    fn dataset_dir_for_workspace_resolves_both_repomod_and_recodeagent() {
        let repo_root = super::super::bench_config::repo_root();
        let repomod_workspace = repo_root.join("benchmarks/outputs/RepoMod-bench/jmespath/rust");
        let repomod_dir = super::dataset_dir_for_workspace(&repomod_workspace);
        let expected_repomod_dir = repo_root
            .join("benchmarks/assets/RepoMod-bench/dataset")
            .join("jmespath")
            .join("dst");
        ::std::assert_eq!(
            repomod_dir,
            ::core::option::Option::Some(expected_repomod_dir)
        );

        let recode_workspace = repo_root.join("benchmarks/outputs/crust/matrix_multiplication/rust");
        let recode_dir = super::dataset_dir_for_workspace(&recode_workspace);
        let expected_recode_dir = repo_root
            .join("benchmarks/assets/ReCodeAgent/data/tool_projects/crust/matrix_multiplication/rust");
        ::std::assert_eq!(
            recode_dir,
            ::core::option::Option::Some(expected_recode_dir)
        );
    }

    #[test]
    fn target_api_surface_resolves_pristine_dataset_for_repomod_jmespath() {
        let repo_root = super::super::bench_config::repo_root();
        let workspace_root = repo_root.join("benchmarks/outputs/RepoMod-bench/jmespath/rust");
        let surface = super::target_api_surface(&workspace_root, "rust");
        ::std::assert!(surface.contains("pub fn search"));
    }
}
