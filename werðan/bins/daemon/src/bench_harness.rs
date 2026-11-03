//! Seed a GlobalBlackboard from a campaign configuration and the legacy source tree.
//! Implements the benchmark harness responsibilities: inventory walk, module vertex
//! discovery, edge contracts, workspace layout normalization, and target workspace checks.

/// One discovered legacy source file with its module vertex assignment.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct InventoryEntry {
    /// Legacy-relative path text such as `src/vec.c`.
    pub relative_path: ::std::string::String,
    /// DAG vertex id assigned to this file.
    pub vertex_id: ::std::string::String,
}

/// Count lines in one source text, splitting total and code lines.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default)]
pub struct LineCounts {
    /// Every line including comments and blanks.
    pub total_lines: ::core::primitive::u64,
    /// Non-blank non-comment lines.
    pub code_lines: ::core::primitive::u64,
}

/// Measured size metrics for the legacy tree feeding COCOMO-II.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default)]
pub struct SizeMetrics {
    /// Every line across all discovered sources.
    pub total_lines: ::core::primitive::u64,
    /// Non-blank non-comment lines across all discovered sources.
    pub code_lines: ::core::primitive::u64,
}

/// Output bundle of the board seeding step.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct SeedOutput {
    /// Seeded global blackboard with inventory, DAG, and contracts.
    pub board: ::global_blackboard::GlobalBlackboard,
    /// Measured legacy size metrics.
    pub metrics: self::SizeMetrics,
}

/// Seed the full GlobalBlackboard from the campaign configuration.
pub fn seed_board(
    bench_config: &super::bench_config::CampaignConfig,
) -> ::core::result::Result<self::SeedOutput, ::std::string::String> {
    let repo_root = super::bench_config::repo_root();
    let legacy_dir = repo_root.join(&bench_config.paths.input_legacy_dir);
    let discovered = self::walk_legacy_sources(
        &legacy_dir,
        &bench_config.stack.source_language,
        &bench_config.paths.off_limits_globs,
    )?;
    let entries = self::build_inventory(&legacy_dir, &discovered);
    let inventory_empty = entries.is_empty();
    if inventory_empty {
        return ::core::result::Result::Err(::std::format!(
            "no source files found under {}",
            legacy_dir.display()
        ));
    }
    let dag = self::build_dag(&entries, &bench_config.stack.source_language);
    let mut board = ::global_blackboard::GlobalBlackboard::default();
    // The S0 INTENT gate requires the verified harness configuration on the
    // board. The target commands come from the campaign stack section.
    board.inventory.harness = ::global_blackboard::HarnessConfig {
        build_cmd: ::core::clone::Clone::clone(&bench_config.stack.target_build_cmd),
        test_cmd: ::core::clone::Clone::clone(&bench_config.stack.target_test_cmd),
        legacy_harness_cmd: ::core::clone::Clone::clone(&bench_config.stack.legacy_test_harness_cmd),
    };
    board.inventory.dialect = ::global_blackboard::DialectProfile {
        source_language: ::core::clone::Clone::clone(&bench_config.stack.source_language),
        target_language: ::core::clone::Clone::clone(&bench_config.stack.target_language),
        target_edition: ::core::clone::Clone::clone(&bench_config.stack.target_edition_or_framework),
    };
    // The S2 guard requires every inventory entry to be covered by a DAG
    // vertex, so the board inventory carries module files only. Header and
    // test-tree files still feed the size metrics and the rule prompts.
    for entry in &entries {
        let test_tree = self::is_test_tree_path(&entry.relative_path);
        let is_header = self::is_header_path(&entry.relative_path, &bench_config.stack.source_language);
        if test_tree || is_header {
            continue;
        }
        board.inventory.push(entry.relative_path.clone());
    }
    board.dag = dag;
    for entry in &entries {
        let test_tree = self::is_test_tree_path(&entry.relative_path);
        if test_tree {
            continue;
        }
        let header_file = self::is_header_path(&entry.relative_path, &bench_config.stack.source_language);
        if header_file {
            let header_path = legacy_dir.join(&entry.relative_path);
            let header_text = ::std::fs::read_to_string(&header_path).unwrap_or_default();
            let header_artifact = ::global_blackboard::ModuleArtifact {
                target_code: { header_text },
                test_suite: ::std::vec::Vec::new(),
            };
            board.artifacts.insert(
                entry.vertex_id.clone(),
                ::core::option::Option::Some(header_artifact),
            );
            board.state.module_states.insert(
                entry.vertex_id.clone(),
                ::global_blackboard::ModuleMicroState::M4Hardened,
            );
        } else {
            let source_path = legacy_dir.join(&entry.relative_path);
            if let ::core::result::Result::Ok(content) = ::std::fs::read_to_string(&source_path) {
                let initial_artifact = ::global_blackboard::ModuleArtifact {
                    target_code: { content },
                    test_suite: ::std::vec::Vec::new(),
                };
                board.artifacts.insert(
                    entry.vertex_id.clone(),
                    ::core::option::Option::Some(initial_artifact),
                );
            }
        }
    }
    let workspace_root = repo_root.join(&bench_config.paths.workspace_root);
    let dataset_dir_opt = super::bench_rules::dataset_dir_for_workspace(&workspace_root);
    self::seed_contracts(
        &mut board,
        &legacy_dir,
        dataset_dir_opt.as_deref(),
        &bench_config.stack.target_language,
    );
    let metrics = self::measure_size(&discovered);
    let seed_output = self::SeedOutput {
        board,
        metrics,
    };
    ::core::result::Result::Ok(seed_output)
}

/// Recompute the inventory entries for the configured legacy directory.
pub fn inventory_entries(
    bench_config: &super::bench_config::CampaignConfig,
) -> ::core::result::Result<::std::vec::Vec<self::InventoryEntry>, ::std::string::String> {
    let repo_root = super::bench_config::repo_root();
    let legacy_dir = repo_root.join(&bench_config.paths.input_legacy_dir);
    let walked = self::walk_legacy_sources(
        &legacy_dir,
        &bench_config.stack.source_language,
        &bench_config.paths.off_limits_globs,
    )?;
    let entries = self::build_inventory(&legacy_dir, &walked);
    ::core::result::Result::Ok(entries)
}

/// Walk the legacy directory in deterministic order and collect source file paths.
pub fn walk_legacy_sources(
    legacy_dir: &::std::path::Path,
    source_language: &::core::primitive::str,
    off_limits_globs: &[::std::string::String],
) -> ::core::result::Result<::std::vec::Vec<::std::path::PathBuf>, ::std::string::String> {
    let wanted = self::source_extensions(source_language);
    let walk = ::ignore::WalkBuilder::new(legacy_dir)
        .hidden(false)
        .git_ignore(true)
        .build();
    let compiled_globset = self::compile_off_limits_globset(off_limits_globs);
    let mut discovered = ::std::vec::Vec::new();
    for entry in walk {
        let dir_entry = match entry {
            ::core::result::Result::Ok(value) => value,
            ::core::result::Result::Err(walk_error) => {
                return ::core::result::Result::Err(::std::format!(
                    "legacy walk failed: {walk_error}"
                ));
            },
        };
        let path = dir_entry.path();
        let path_is_file = path.is_file();
        if !path_is_file {
            continue;
        }
        let relative_path = match path.strip_prefix(legacy_dir) {
            ::core::result::Result::Ok(relative) => relative,
            ::core::result::Result::Err(_) => path,
        };
        let off_limits = self::is_off_limits_source(
            relative_path,
            path,
            off_limits_globs,
            compiled_globset.as_ref(),
        );
        if off_limits {
            continue;
        }
        let path_text = path.to_string_lossy();
        for extension in &wanted {
            let extension_matches = path_text.ends_with(extension);
            if extension_matches {
                discovered.push(path.to_path_buf());
            }
        }
    }
    discovered.sort();
    ::core::result::Result::Ok(discovered)
}
/// Return the file extensions that the harness treats as source files.
fn source_extensions(
    source_language: &::core::primitive::str,
) -> ::std::vec::Vec<&'static ::core::primitive::str> {
    match source_language {
        "c" => ::std::vec![".c", ".h"],
        "java" => ::std::vec![".java"],
        "python" => ::std::vec![".py"],
        "py" => ::std::vec![".py"],
        "go" => ::std::vec![".go"],
        _ => ::std::vec![".c", ".h"],
    }
}

/// Compile glob patterns from the off-limits list into a GlobSet.
fn compile_off_limits_globset(
    off_limits_globs: &[::std::string::String],
) -> ::core::option::Option<::globset::GlobSet> {
    if off_limits_globs.is_empty() {
        return ::core::option::Option::None;
    }
    let mut builder = ::globset::GlobSetBuilder::new();
    for pattern in off_limits_globs {
        let mut glob_builder = ::globset::GlobBuilder::new(pattern);
        glob_builder.literal_separator(false);
        if let ::core::result::Result::Ok(built_glob) = glob_builder.build() {
            builder.add(built_glob);
        }
    }
    builder.build().ok()
}

/// Check whether a discovered source file path matches any off-limits glob or pattern.
fn is_off_limits_source(
    relative_path: &::std::path::Path,
    full_path: &::std::path::Path,
    off_limits_globs: &[::std::string::String],
    compiled_globset: ::core::option::Option<&::globset::GlobSet>,
) -> ::core::primitive::bool {
    if off_limits_globs.is_empty() {
        return false;
    }
    if let ::core::option::Option::Some(globset) = compiled_globset {
        let matches_relative = globset.is_match(relative_path);
        let matches_full = globset.is_match(full_path);
        if matches_relative || matches_full {
            return true;
        }
    }
    let relative_str = relative_path.to_string_lossy().replace('\\', "/");
    let full_str = full_path.to_string_lossy().replace('\\', "/");
    for pattern in off_limits_globs {
        let pattern_normalized = pattern.replace('\\', "/");
        let matches_relative_exact = relative_str == pattern_normalized;
        let contains_pattern = relative_str.contains(&pattern_normalized);
        let full_contains_pattern = full_str.contains(&pattern_normalized);
        if matches_relative_exact || contains_pattern || full_contains_pattern {
            return true;
        }
        let trimmed_glob = pattern_normalized.trim_start_matches("**/");
        let matches_trimmed = relative_str == trimmed_glob;
        let suffix = ::std::format!("/{trimmed_glob}");
        let ends_with_suffix = relative_str.ends_with(&suffix);
        let full_ends_with_suffix = full_str.ends_with(&suffix);
        if matches_trimmed || ends_with_suffix || full_ends_with_suffix {
            return true;
        }
    }
    false
}



/// Build the inventory entries, one per discovered source file.
pub fn build_inventory(
    legacy_dir: &::std::path::Path,
    discovered: &[::std::path::PathBuf],
) -> ::std::vec::Vec<self::InventoryEntry> {
    let mut entries = ::std::vec::Vec::new();
    for source_path in discovered {
        let relative_path = self::relative_text(legacy_dir, source_path);
        let entry = self::InventoryEntry {
            vertex_id: { relative_path.clone() },
            relative_path,
        };
        entries.push(entry);
    }
    entries
}

/// Derive the legacy-relative path text for one discovered file.
fn relative_text(
    legacy_dir: &::std::path::Path,
    source_path: &::std::path::Path,
) -> ::std::string::String {
    let strip_res = source_path.strip_prefix(legacy_dir);
    let stripped = match strip_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(_) => source_path,
    };
    stripped.to_string_lossy().to_string()
}

/// Build the condensed DAG vertices from the inventory.
///
/// Vertices are top-level implementation source modules only. Headers carry
/// no portable code and test trees are the harness's verification input, so
/// neither becomes a module. A vertex whose MCTS budget exhausts otherwise
/// maps to a ContractConflict no real parent can satisfy. That wedges the
/// module in M3ContractNegotiate forever. Edges stay empty. Header content
/// reaches codegen through the rule extraction prompt instead.
pub fn build_dag(
    entries: &[self::InventoryEntry],
    source_language: &::core::primitive::str,
) -> ::global_blackboard::CondensedDag {
    let mut dag = ::global_blackboard::CondensedDag::new();
    for entry in entries {
        let test_tree = self::is_test_tree_path(&entry.relative_path);
        let header_file = self::is_header_path(&entry.relative_path, source_language);
        if test_tree || header_file {
            continue;
        }
        dag.vertices.push(entry.vertex_id.clone());
    }
    dag
}
/// Report whether one inventory entry refers to a header file.
pub fn is_header(
    entry: &self::InventoryEntry,
    source_language: &::core::primitive::str,
) -> ::core::primitive::bool {
    self::is_header_path(&entry.relative_path, source_language)
}
/// Report whether one relative path lives inside a test tree directory.
fn is_test_tree_path(relative_path: &::core::primitive::str) -> ::core::primitive::bool {
    relative_path
        .split('/')
        .any(|component| component == "tests" || component == "test")
}
/// Report whether the extension marks a header file.
///
/// Header detection follows the legacy source language. C-family campaigns
/// treat `.h`, `.hpp`, and `.hxx` as headers. Every other language, including
/// rust, python, and java, declares no header extensions, so header-dependent
/// logic falls through to plain implementation-module handling.
fn is_header_path(
    relative_path: &::core::primitive::str,
    source_language: &::core::primitive::str,
) -> ::core::primitive::bool {
    let c_family_selected = ::core::matches!(
        source_language,
        "c" | "cpp" | "c++" | "cxx"
    );
    if !c_family_selected {
        return false;
    }
    relative_path.ends_with(".h")
        || relative_path.ends_with(".hpp")
        || relative_path.ends_with(".hxx")
}

/// Seed interface contracts per DAG edge and per vertex from companion headers and target interface stubs.
pub fn seed_contracts(
    board: &mut ::global_blackboard::GlobalBlackboard,
    legacy_dir: &::std::path::Path,
    dataset_dir: ::core::option::Option<&::std::path::Path>,
    target_language: &::core::primitive::str,
) {
    let edges = board.dag.edges.clone();
    for edge_pair in edges {
        let parent_id = edge_pair.0.clone();
        let header_path = legacy_dir.join(&parent_id);
        let read_res = ::std::fs::read_to_string(&header_path);
        let header_spec = match read_res {
            ::core::result::Result::Ok(content) => content,
            ::core::result::Result::Err(_) => ::std::format!("parent interface from header {parent_id}"),
        };
        let contract = ::holon_blackboard::EdgeContract {
            parent_id,
            interface_spec: { header_spec },
            preconditions: ::std::vec::Vec::new(),
        };
        board.contracts.insert(edge_pair, contract);
    }

    let effective_dataset_dir = match dataset_dir {
        ::core::option::Option::Some(dataset_path) => ::core::option::Option::Some(dataset_path.to_path_buf()),
        ::core::option::Option::None => {
            let parent_opt = legacy_dir.parent();
            let candidate_opt = parent_opt.map(|parent_path| parent_path.join(target_language));
            candidate_opt.filter(|candidate_path| candidate_path.is_dir())
        }
    };

    let vertices = board.dag.vertices.clone();
    for vertex_id in vertices {
        let vertex_path = ::std::path::Path::new(&vertex_id);
        let stem_opt = vertex_path.file_stem().and_then(|stem_segment| stem_segment.to_str());
        let vertex_stem = stem_opt.unwrap_or("");
        let mut combined_specs = ::std::vec::Vec::new();
        let mut primary_parent = ::std::string::String::new();
        for (registered_id, maybe_artifact) in &board.artifacts.artifacts {
            let artifact_path = ::std::path::Path::new(registered_id);
            let artifact_stem_opt = artifact_path.file_stem().and_then(|stem_segment| stem_segment.to_str());
            let artifact_stem = artifact_stem_opt.unwrap_or("");
            let stems_match = !vertex_stem.is_empty() && vertex_stem == artifact_stem;
            let is_header = registered_id.ends_with(".h")
                || registered_id.ends_with(".hpp")
                || registered_id.ends_with(".hxx");
            if stems_match && is_header {
                if let ::core::option::Option::Some(artifact_entry) = maybe_artifact {
                    let primary_empty = primary_parent.is_empty();
                    if primary_empty {
                        primary_parent = ::core::clone::Clone::clone(registered_id);
                    }
                    let target_code = ::core::clone::Clone::clone(&artifact_entry.target_code);
                    combined_specs.push(target_code);
                }
            }
        }
        if let ::core::option::Option::Some(target_root) = &effective_dataset_dir {
            let target_file_opt = self::find_target_file_for_vertex(
                target_root,
                &vertex_id,
                target_language,
            );
            if let ::core::option::Option::Some(target_file_path) = target_file_opt {
                let target_read_res = ::std::fs::read_to_string(&target_file_path);
                if let ::core::result::Result::Ok(target_content) = target_read_res {
                    let extracted = self::extract_exported_signatures(&target_content, target_language);
                    let has_extracted = !extracted.is_empty();
                    if has_extracted {
                        let target_surface = extracted.join("\n\n");
                        let labeled_surface = ::std::format!("// Target interface export obligations:\n{target_surface}");
                        combined_specs.push(labeled_surface);
                    }
                }
            }
        }
        let has_specs = !combined_specs.is_empty();
        if has_specs {
            let primary_empty = primary_parent.is_empty();
            if primary_empty {
                primary_parent = ::core::clone::Clone::clone(&vertex_id);
            }
            let vertex_first = ::core::clone::Clone::clone(&vertex_id);
            let vertex_second = ::core::clone::Clone::clone(&vertex_id);
            let self_key = (vertex_first, vertex_second);
            let spec_text = combined_specs.join("\n\n");
            let contract = ::holon_blackboard::EdgeContract {
                parent_id: { primary_parent },
                interface_spec: { spec_text },
                preconditions: ::std::vec::Vec::new(),
            };
            board.contracts.insert(self_key, contract);
        }
    }
}

fn find_target_file_for_vertex(
    dataset_dir: &::std::path::Path,
    vertex_id: &::core::primitive::str,
    target_language: &::core::primitive::str,
) -> ::core::option::Option<::std::path::PathBuf> {
    let vertex_path = ::std::path::Path::new(vertex_id);
    let stem_opt = vertex_path.file_stem().and_then(|stem_segment| stem_segment.to_str());
    let vertex_stem = stem_opt?;
    let stem_is_empty = vertex_stem.is_empty();
    if stem_is_empty {
        return ::core::option::Option::None;
    }
    let target_ext = super::bench_rules::target_source_extension(target_language);
    let file_name = ::std::format!("{vertex_stem}{target_ext}");

    let src_dir = dataset_dir.join("src");
    let src_candidate = src_dir.join(&file_name);
    let src_is_file = src_candidate.is_file();
    if src_is_file {
        return ::core::option::Option::Some(src_candidate);
    }

    let root_candidate = dataset_dir.join(&file_name);
    let root_is_file = root_candidate.is_file();
    if root_is_file {
        return ::core::option::Option::Some(root_candidate);
    }

    let search_dir = dataset_dir.join("src");
    let search_is_dir = search_dir.is_dir();
    let effective_dir = if search_is_dir {
        search_dir
    } else {
        dataset_dir.to_path_buf()
    };
    let walk_builder = ::ignore::WalkBuilder::new(effective_dir);
    let walker = walk_builder.build();
    for entry_res in walker.flatten() {
        let entry_path = entry_res.path();
        let entry_is_file = entry_path.is_file();
        if !entry_is_file {
            continue;
        }
        let entry_stem_opt = entry_path.file_stem().and_then(|entry_stem_os| entry_stem_os.to_str());
        let matches_stem = entry_stem_opt == ::core::option::Option::Some(vertex_stem);
        let path_text = entry_path.to_string_lossy();
        let matches_ext = path_text.ends_with(target_ext);
        if matches_stem && matches_ext {
            let path_buf = entry_path.to_path_buf();
            return ::core::option::Option::Some(path_buf);
        }
    }
    ::core::option::Option::None
}

/// Extract exported signatures from target source file content.
pub fn extract_exported_signatures(
    content: &::core::primitive::str,
    target_language: &::core::primitive::str,
) -> ::std::vec::Vec<::std::string::String> {
    let is_python = target_language == "python" || target_language == "py";
    if is_python {
        return self::extract_python_signatures(content);
    }
    let is_js = target_language == "javascript" || target_language == "js";
    if is_js {
        return self::extract_js_signatures(content);
    }
    self::extract_rust_signatures(content)
}

fn extract_python_signatures(content: &::core::primitive::str) -> ::std::vec::Vec<::std::string::String> {
    let mut signatures = ::std::vec::Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let is_def = trimmed.starts_with("def ") && !trimmed.starts_with("def _");
        let is_class = trimmed.starts_with("class ") && !trimmed.starts_with("class _");
        if is_def || is_class {
            let line_string = trimmed.to_string();
            signatures.push(line_string);
        }
    }
    signatures
}

fn extract_js_signatures(content: &::core::primitive::str) -> ::std::vec::Vec<::std::string::String> {
    let mut signatures = ::std::vec::Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let is_export = trimmed.starts_with("export function ")
            || trimmed.starts_with("export class ")
            || trimmed.starts_with("export const ")
            || trimmed.starts_with("export let ");
        if is_export {
            let line_string = trimmed.to_string();
            signatures.push(line_string);
        }
    }
    signatures
}

/// Extract Rust exported signatures from target source content.
pub fn extract_rust_signatures(content: &::core::primitive::str) -> ::std::vec::Vec<::std::string::String> {
    let mut signatures = ::std::vec::Vec::new();
    let lines = content.lines().collect::<::std::vec::Vec<&::core::primitive::str>>();
    let total_lines = lines.len();
    let mut line_idx = 0;
    while line_idx < total_lines {
        let line_text = lines[line_idx].trim();
        let is_pub_fn = self::is_rust_pub_fn_start(line_text);
        let is_pub_block = self::is_rust_pub_block_start(line_text);
        let is_pub_item = self::is_rust_pub_item_start(line_text);

        if is_pub_fn {
            let (signature, next_idx) = self::consume_rust_fn(&lines, line_idx);
            let sig_not_empty = !signature.is_empty();
            if sig_not_empty {
                signatures.push(signature);
            }
            line_idx = next_idx;
        } else if is_pub_block {
            let (block_text, next_idx) = self::consume_rust_block(&lines, line_idx);
            let block_not_empty = !block_text.is_empty();
            if block_not_empty {
                signatures.push(block_text);
            }
            line_idx = next_idx;
        } else if is_pub_item {
            let (item_text, next_idx) = self::consume_rust_item(&lines, line_idx);
            let item_not_empty = !item_text.is_empty();
            if item_not_empty {
                signatures.push(item_text);
            }
            line_idx = next_idx;
        }
        line_idx += 1;
    }
    signatures
}

fn is_rust_pub_fn_start(line_text: &::core::primitive::str) -> ::core::primitive::bool {
    line_text.starts_with("pub fn ")
        || line_text.starts_with("pub const fn ")
        || line_text.starts_with("pub async fn ")
        || line_text.starts_with("pub unsafe fn ")
        || line_text.starts_with("pub extern ")
}

fn is_rust_pub_block_start(line_text: &::core::primitive::str) -> ::core::primitive::bool {
    line_text.starts_with("pub struct ")
        || line_text.starts_with("pub enum ")
        || line_text.starts_with("pub union ")
        || line_text.starts_with("pub trait ")
}

fn is_rust_pub_item_start(line_text: &::core::primitive::str) -> ::core::primitive::bool {
    line_text.starts_with("pub type ")
        || (line_text.starts_with("pub const ") && !line_text.starts_with("pub const fn "))
        || line_text.starts_with("pub static ")
}

fn consume_rust_fn(
    lines: &[&::core::primitive::str],
    start_idx: ::core::primitive::usize,
) -> (::std::string::String, ::core::primitive::usize) {
    let mut collected = ::std::vec::Vec::new();
    let total_lines = lines.len();
    let mut curr_idx = start_idx;
    while curr_idx < total_lines {
        let line = lines[curr_idx];
        if let ::core::option::Option::Some(brace_pos) = line.find('{') {
            let before_slice = &line[..brace_pos];
            let before_brace = before_slice.trim_end();
            let trimmed_before = before_brace.trim();
            let before_not_empty = !trimmed_before.is_empty();
            if before_not_empty {
                collected.push(before_brace);
            }
            break;
        }
        if let ::core::option::Option::Some(semi_pos) = line.find(';') {
            let before_semi = &line[..=semi_pos];
            collected.push(before_semi);
            break;
        }
        collected.push(line);
        curr_idx += 1;
    }
    let raw_sig = collected.join("\n");
    let trimmed_sig = raw_sig.trim();
    let mut signature = trimmed_sig.to_string();
    let sig_not_empty = !signature.is_empty();
    let has_semi = signature.ends_with(';');
    let needs_semi = sig_not_empty && !has_semi;
    if needs_semi {
        signature.push(';');
    }
    (signature, curr_idx)
}

fn consume_rust_block(
    lines: &[&::core::primitive::str],
    start_idx: ::core::primitive::usize,
) -> (::std::string::String, ::core::primitive::usize) {
    let mut collected = ::std::vec::Vec::new();
    let total_lines = lines.len();
    let mut curr_idx = start_idx;
    let mut brace_depth: ::core::primitive::i32 = 0;
    let mut seen_brace = false;
    while curr_idx < total_lines {
        let line = lines[curr_idx];
        collected.push(line);
        for char_val in line.chars() {
            if char_val == '{' {
                brace_depth += 1;
                seen_brace = true;
            } else if char_val == '}' {
                brace_depth -= 1;
            }
        }
        let block_done = seen_brace && brace_depth <= 0;
        if block_done {
            break;
        }
        let semi_done = !seen_brace && line.contains(';');
        if semi_done {
            break;
        }
        curr_idx += 1;
    }
    let raw_block = collected.join("\n");
    let trimmed_block = raw_block.trim();
    let block = trimmed_block.to_string();
    (block, curr_idx)
}

fn consume_rust_item(
    lines: &[&::core::primitive::str],
    start_idx: ::core::primitive::usize,
) -> (::std::string::String, ::core::primitive::usize) {
    let mut collected = ::std::vec::Vec::new();
    let total_lines = lines.len();
    let mut curr_idx = start_idx;
    while curr_idx < total_lines {
        let line = lines[curr_idx];
        collected.push(line);
        let has_semi = line.contains(';');
        if has_semi {
            break;
        }
        curr_idx += 1;
    }
    let raw_item = collected.join("\n");
    let trimmed_item = raw_item.trim();
    let item = trimmed_item.to_string();
    (item, curr_idx)
}


/// Measure the legacy tree size metrics from the discovered source files.
pub fn measure_size(discovered: &[::std::path::PathBuf]) -> self::SizeMetrics {
    let mut metrics = self::SizeMetrics::default();
    for source_path in discovered {
        let counts = self::count_lines(source_path);
        metrics.total_lines += counts.total_lines;
        metrics.code_lines += counts.code_lines;
    }
    metrics
}

/// Count total and code lines of one text file.
fn count_lines(source_path: &::std::path::Path) -> self::LineCounts {
    let read_res = ::std::fs::read_to_string(source_path);
    let text = match read_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(_) => return self::LineCounts::default(),
    };
    let mut counts = self::LineCounts::default();
    for line in text.lines() {
        counts.total_lines += 1;
        let trimmed = line.trim();
        let blank = trimmed.is_empty();
        if blank {
            continue;
        }
        let comment_line = trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with("*")
            || trimmed.starts_with('#');
        if comment_line {
            continue;
        }
        counts.code_lines += 1;
    }
    counts
}

/// Compute the COCOMO-II organic-mode effort in person-months from measured KLOC.
pub fn cocomo_effort_pm(code_lines: ::core::primitive::u64) -> ::core::primitive::f64 {
    let kloc = (code_lines as ::core::primitive::f64) / 1000.0;
    2.4 * kloc.powf(1.05)
}

/// Compute the COCOMO-II organic-mode duration in months from effort person-months.
pub fn cocomo_duration_months(effort_pm: ::core::primitive::f64) -> ::core::primitive::f64 {
    2.5 * effort_pm.powf(0.38)
}

/// Count the remaining `unimplemented!()` markers under the workspace src tree.
///
/// The stub-count metric is a Rust-target concept by definition: it measures
/// how many `unimplemented!()` markers the modernized crate still carries.
/// Other target languages define no such marker convention, so this reports
/// zero immediately instead of scanning for a Rust-only token.
pub fn count_unimplemented(
    workspace_root: &::std::path::Path,
    target_language: &::core::primitive::str,
) -> ::core::primitive::usize {
    let rust_target_selected = target_language.eq_ignore_ascii_case("rust");
    if !rust_target_selected {
        return 0;
    }
    let src_dir = workspace_root.join("src");
    let walk = ::ignore::WalkBuilder::new(src_dir).build();
    let mut marker_count = 0;
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
        let file_markers = text.matches("unimplemented!()").count();
        marker_count += file_markers;
    }
    marker_count
}

/// Normalize the stub workspace layout into one self-contained `src/lib.rs`.
///
/// The dataset stub pattern declares `pub mod <name>;` in `lib.rs` with the
/// stub bodies in sibling module files. The evaluation harness overwrites
/// `src/lib.rs` wholesale with each candidate. The sibling stub files would
/// stay on disk with their `unimplemented!()` markers yet leave the compiled
/// crate. The honest stub-count metric would then stay permanently nonzero.
/// Inlining each module as an inline `pub mod <name>` block preserves the
/// module path that consumer targets rely on. For example,
/// `src/bin/test_vec.rs` calls `vec::vec_insert`. The pristine workspace also
/// keeps compiling. Every compiled artifact lands in the one file the harness
/// replaces. Returns the inlined module names. Test-only today. The campaign
/// pipeline wires this in when the workspace-overwrite path lands.
///
/// The `pub mod` declaration inlining follows a Rust workspace convention, so
/// the normalization only runs for rust targets. Non-rust targets define no
/// `src/lib.rs` module layout to collapse, and the normalization leaves their
/// workspace unchanged.
#[cfg(test)]
pub fn normalize_workspace_layout(
    workspace_root: &::std::path::Path,
    target_language: &::core::primitive::str,
) -> ::std::vec::Vec<::std::string::String> {
    let rust_target_selected = target_language.eq_ignore_ascii_case("rust");
    if !rust_target_selected {
        return ::std::vec::Vec::new();
    }
    let lib_path = workspace_root.join("src/lib.rs");
    let read_res = ::std::fs::read_to_string(&lib_path);
    let lib_text = match read_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(_) => return ::std::vec::Vec::new(),
    };
    let mut inlined_declarations = ::std::vec::Vec::new();
    let mut rewritten_lines = ::std::vec::Vec::new();
    let mut layout_changed = false;
    for line in lib_text.lines() {
        let trimmed = line.trim();
        let module_name = trimmed
            .strip_prefix("pub mod ")
            .and_then(|remainder| remainder.strip_suffix(';'))
            .map(|name_text| name_text.trim().to_string());
        let module_name = match module_name {
            ::core::option::Option::Some(value) => value,
            ::core::option::Option::None => {
                rewritten_lines.push(line.to_string());
                continue;
            },
        };
        let module_file = workspace_root.join("src").join(::std::format!("{module_name}.rs"));
        let module_res = ::std::fs::read_to_string(&module_file);
        let module_text = match module_res {
            ::core::result::Result::Ok(value) => value,
            ::core::result::Result::Err(_) => {
                // Not a file-module stub declaration, so keep the line verbatim.
                rewritten_lines.push(line.to_string());
                continue;
            },
        };
        rewritten_lines.push(::std::format!("pub mod {module_name} {{"));
        rewritten_lines.push(module_text);
        rewritten_lines.push("}".to_string());
        let blank_line = ::std::string::String::new();
        rewritten_lines.push(blank_line);
        inlined_declarations.push(module_name);
        layout_changed = true;
    }
    let layout_unchanged = !layout_changed;
    if layout_unchanged {
        return ::std::vec::Vec::new();
    }
    let mut normalized_text = rewritten_lines.join("\n");
    normalized_text.push('\n');
    let write_res = ::std::fs::write(&lib_path, normalized_text);
    if write_res.is_err() {
        return ::std::vec::Vec::new();
    }
    for module_name in &inlined_declarations {
        let module_file = workspace_root.join("src").join(::std::format!("{module_name}.rs"));
        let _ = ::std::fs::remove_file(&module_file);
    }
    inlined_declarations
}

#[cfg(test)]
mod tests {
    #[test]
    fn cocomo_matches_reference_values() {
        // Standard COCOMO-II organic formula: 2.4 * kloc^1.05.
        // 998 code lines equal 0.998 kloc and yield 2.3950 person-months.
        let effort = super::cocomo_effort_pm(998);
        ::std::assert!((effort - 2.395).abs() < 0.01, "effort was {effort}");
        let duration = super::cocomo_duration_months(effort);
        let expected_duration = 2.5 * effort.powf(0.38);
        ::std::assert!((duration - expected_duration).abs() < 1e-9, "duration was {duration}");
        // 630 code lines equal 0.63 kloc and yield 1.4783 person-months.
        let totp_effort = super::cocomo_effort_pm(630);
        ::std::assert!((totp_effort - 1.478).abs() < 0.01, "effort was {totp_effort}");
    }

    #[test]
    fn normalization_inlines_stub_modules_and_removes_files() {
        let pid_text = ::std::process::id();
        let suffix = ::std::format!("daemon-normalize-test-{pid_text}");
        let workspace = ::std::env::temp_dir().join(suffix);
        let src_dir = workspace.join("src");
        ::std::fs::create_dir_all(&src_dir).expect("temp src dir creates");
        let lib_path = src_dir.join("lib.rs");
        let module_path = src_dir.join("widget.rs");
        let stub_declaration = "pub mod widget\x3b\n";
        ::std::fs::write(&lib_path, stub_declaration).expect("stub lib writes");
        let module_source = "pub fn spin() -> u8 {\n    unimplemented!()\n}\n";
        ::std::fs::write(&module_path, module_source).expect("stub module writes");
        let removed =
            super::normalize_workspace_layout(&workspace, "rust");
        ::std::assert_eq!(removed, ::std::vec!["widget".to_string()]);
        let normalized = ::std::fs::read_to_string(&lib_path).expect("normalized lib reads");
        ::std::assert!(normalized.contains("pub fn spin"));
        let declaration = "pub mod widget {";
        ::std::assert!(normalized.contains(declaration));
        let stale_declaration = "pub mod widget\x3b";
        ::std::assert!(!normalized.contains(stale_declaration));
        ::std::assert!(!module_path.exists(), "inlined stub module file must go");
        let marker_count = super::count_unimplemented(&workspace, "rust");
        ::std::assert_eq!(marker_count, 1, "markers survive inside lib.rs");
        let _ = ::std::fs::remove_dir_all(&workspace);
    }

    #[test]
    fn normalization_leaves_non_stub_layouts_untouched() {
        let pid_text = ::std::process::id();
        let suffix = ::std::format!("daemon-normalize-keep-{pid_text}");
        let workspace = ::std::env::temp_dir().join(suffix);
        let src_dir = workspace.join("src");
        ::std::fs::create_dir_all(&src_dir).expect("temp src dir creates");
        let lib_path = src_dir.join("lib.rs");
        let original = "pub fn ready() -> bool {\n    true\n}\n";
        ::std::fs::write(&lib_path, original).expect("plain lib writes");
        let removed = super::normalize_workspace_layout(&workspace, "rust");
        ::std::assert!(removed.is_empty());
        let after = ::std::fs::read_to_string(&lib_path).expect("plain lib rereads");
        ::std::assert_eq!(after, original);
        let _ = ::std::fs::remove_dir_all(&workspace);
    }

    #[test]
    fn self_contract_seeded_for_module_matching_header() {
        let repo_root = super::super::bench_config::repo_root();
        let config_path = repo_root.join("benchmarks/assets/configs/ReCodeAgent/crust/libwecan.yml");
        let config_text = ::std::fs::read_to_string(&config_path)
            .expect("libwecan config reads");
        let config = ::serde_yaml_ng::from_str::<super::super::bench_config::CampaignConfig>(&config_text)
            .expect("config parses");
        let seeded = super::seed_board(&config).expect("seed board succeeds");
        let self_key = ("src/libwecan.c".to_string(), "src/libwecan.c".to_string());
        let has_self_contract = seeded.board.contracts.contracts.contains_key(&self_key);
        ::std::assert!(has_self_contract, "self-contract for libwecan must be seeded");
        let contract = &seeded.board.contracts.contracts[&self_key];
        ::std::assert!(contract.interface_spec.contains("extract"));
        ::std::assert!(contract.interface_spec.contains("insert"));
    }
    #[test]
    fn walk_legacy_sources_respects_off_limits_globs() {
        let pid_text = ::std::process::id();
        let suffix = ::std::format!("daemon-walk-offlimits-{pid_text}");
        let legacy_dir = ::std::env::temp_dir().join(suffix);
        ::std::fs::create_dir_all(&legacy_dir).expect("temp legacy dir creates");
        let main_c = legacy_dir.join("main.c");
        let matrix_c = legacy_dir.join("matrix.c");
        let utils_c = legacy_dir.join("utils.c");
        ::std::fs::write(&main_c, "int main() { return 0\x3b }\n").expect("main.c writes");
        ::std::fs::write(&matrix_c, "void mat() {}\n").expect("matrix.c writes");
        ::std::fs::write(&utils_c, "void util() {}\n").expect("utils.c writes");

        let off_limits = ::std::vec!["**/main.c".to_string()];
        let discovered = super::walk_legacy_sources(&legacy_dir, "c", &off_limits)
            .expect("walk legacy sources succeeds");

        let discovered_names = discovered
            .iter()
            .map(|path_entry| {
                path_entry
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .collect::<::std::vec::Vec<::std::string::String>>();

        ::std::assert!(!discovered_names.contains(&"main.c".to_string()));
        ::std::assert!(discovered_names.contains(&"matrix.c".to_string()));
        ::std::assert!(discovered_names.contains(&"utils.c".to_string()));

        let _ = ::std::fs::remove_dir_all(&legacy_dir);
    }

    #[test]
    fn seed_board_for_matrix_multiplication_excludes_main_c() {
        let repo_root = super::super::bench_config::repo_root();
        let config_path = repo_root.join("benchmarks/assets/configs/ReCodeAgent/crust/matrix_multiplication.yml");
        let config_text = ::std::fs::read_to_string(&config_path)
            .expect("matrix_multiplication config reads");
        let config = ::serde_yaml_ng::from_str::<super::super::bench_config::CampaignConfig>(&config_text)
            .expect("config parses");
        let seeded = super::seed_board(&config).expect("seed board succeeds");
        let has_main_vertex = seeded.board.dag.vertices.contains(&"main.c".to_string());
        ::std::assert!(!has_main_vertex, "main.c must not be in DAG vertices");
        let has_main_inventory = seeded.board.inventory.contains("main.c");
        ::std::assert!(!has_main_inventory, "main.c must not be in inventory");
    }

    #[test]
    fn seed_board_for_matrix_multiplication_seeds_target_signatures() {
        let repo_root = super::super::bench_config::repo_root();
        let config_path = repo_root.join("benchmarks/assets/configs/ReCodeAgent/crust/matrix_multiplication.yml");
        let config_text = ::std::fs::read_to_string(&config_path)
            .expect("matrix_multiplication config reads");
        let config = ::serde_yaml_ng::from_str::<super::super::bench_config::CampaignConfig>(&config_text)
            .expect("config parses");
        let seeded = super::seed_board(&config).expect("seed board succeeds");
        let utils_key = ("utils.c".to_string(), "utils.c".to_string());
        let has_utils_contract = seeded.board.contracts.contracts.contains_key(&utils_key);
        ::std::assert!(has_utils_contract, "utils.c must have a self-contract");
        let utils_contract = &seeded.board.contracts.contracts[&utils_key];
        ::std::assert!(
            utils_contract.interface_spec.contains("print_array(matrix: &[Vec<f32>], name: &str, typ: &str)"),
            "utils self-contract must contain 3-argument print_array signature: {}",
            utils_contract.interface_spec
        );
        let matrix_key = ("matrix.c".to_string(), "matrix.c".to_string());
        let has_matrix_contract = seeded.board.contracts.contracts.contains_key(&matrix_key);
        ::std::assert!(has_matrix_contract, "matrix.c must have a self-contract");
        let matrix_contract = &seeded.board.contracts.contracts[&matrix_key];
        ::std::assert!(
            matrix_contract.interface_spec.contains("pub fn generate_matrix"),
            "matrix self-contract must contain generate_matrix"
        );
    }
}
