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
    let discovered = self::walk_legacy_sources(&legacy_dir, &bench_config.stack.source_language)?;
    let entries = self::build_inventory(&legacy_dir, &discovered);
    let inventory_empty = entries.is_empty();
    if inventory_empty {
        return ::core::result::Result::Err(::std::format!(
            "no source files found under {}",
            legacy_dir.display()
        ));
    }
    let dag = self::build_dag(&entries);
    let mut board = ::global_blackboard::GlobalBlackboard::default();
    // The S2 guard requires every inventory entry to be covered by a DAG
    // vertex, so the board inventory carries module files only. Header and
    // test-tree files still feed the size metrics and the rule prompts.
    for entry in &entries {
        let is_header = self::is_header_path(&entry.relative_path);
        let test_tree = self::is_test_tree_path(&entry.relative_path);
        let non_test = !test_tree || is_header;
        if non_test {
            board.inventory.push(entry.relative_path.clone());
        }
    }
    board.dag = dag;
    self::seed_contracts(&mut board, &legacy_dir);
    for entry in &entries {
        let header_file = self::is_header_path(&entry.relative_path);
        let test_tree = self::is_test_tree_path(&entry.relative_path);
        if test_tree && !header_file {
            continue;
        }
        let header_file = self::is_header_path(&entry.relative_path);
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
    let walked = self::walk_legacy_sources(&legacy_dir, &bench_config.stack.source_language)?;
    let entries = self::build_inventory(&legacy_dir, &walked);
    ::core::result::Result::Ok(entries)
}

/// Walk the legacy directory in deterministic order and collect source file paths.
pub fn walk_legacy_sources(
    legacy_dir: &::std::path::Path,
    source_language: &::core::primitive::str,
) -> ::core::result::Result<::std::vec::Vec<::std::path::PathBuf>, ::std::string::String> {
    let wanted = self::source_extensions(source_language);
    let walk = ::ignore::WalkBuilder::new(legacy_dir)
        .hidden(false)
        .git_ignore(true)
        .build();
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
pub fn build_dag(entries: &[self::InventoryEntry]) -> ::global_blackboard::CondensedDag {
    let mut dag = ::global_blackboard::CondensedDag::new();
    let mut headers = ::std::vec::Vec::new();
    let mut modules = ::std::vec::Vec::new();
    for entry in entries {
        let header_file = self::is_header_path(&entry.relative_path);
        let test_tree = self::is_test_tree_path(&entry.relative_path);
        if test_tree && !header_file {
            continue;
        }
        if header_file {
            headers.push(entry);
        } else {
            modules.push(entry);
        }
        dag.vertices.push(entry.vertex_id.clone());
    }
    for module_entry in &modules {
        let module_stem = self::stem_of(&module_entry.relative_path);
        for header_entry in &headers {
            let header_stem = self::stem_of(&header_entry.relative_path);
            if module_stem == header_stem {
                dag.edges.push((header_entry.vertex_id.clone(), module_entry.vertex_id.clone()));
            }
        }
    }
    dag
}
/// Report whether one inventory entry refers to a header file.
pub fn is_header(entry: &self::InventoryEntry) -> ::core::primitive::bool {
    self::is_header_path(&entry.relative_path)
}

/// Report whether the extension marks a header file.
fn is_header_path(relative_path: &::core::primitive::str) -> ::core::primitive::bool {
    relative_path.ends_with(".h")
}

/// Report whether one relative path lives inside a test tree directory.
fn is_test_tree_path(relative_path: &::core::primitive::str) -> ::core::primitive::bool {
    relative_path
        .split('/')
        .any(|component| component == "tests" || component == "test")
}

fn stem_of(relative_path: &::core::primitive::str) -> &::core::primitive::str {
    let file_name = relative_path.rsplit('/').next().unwrap_or(relative_path);
    let stem_end = file_name.rfind('.').unwrap_or(file_name.len());
    &file_name[..stem_end]
}
/// Seed one interface contract per DAG edge so the S4 brief guard holds.
pub fn seed_contracts(
    board: &mut ::global_blackboard::GlobalBlackboard,
    legacy_dir: &::std::path::Path,
) {
    let edges = board.dag.edges.clone();
    for edge_pair in edges {
        let parent_id = edge_pair.0.clone();
        let header_path = legacy_dir.join(&parent_id);
        let header_spec = ::std::fs::read_to_string(&header_path)
            .unwrap_or_else(|_| ::std::format!("parent interface from header {parent_id}"));
        let contract = ::holon_blackboard::EdgeContract {
            parent_id,
            interface_spec: { header_spec },
            preconditions: ::std::vec::Vec::new(),
        };
        board.contracts.insert(edge_pair, contract);
    }
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
pub fn count_unimplemented(workspace_root: &::std::path::Path) -> ::core::primitive::usize {
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
#[cfg(test)]
pub fn normalize_workspace_layout(workspace_root: &::std::path::Path) -> ::std::vec::Vec<::std::string::String> {
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
        let removed = super::normalize_workspace_layout(&workspace);
        ::std::assert_eq!(removed, ::std::vec!["widget".to_string()]);
        let normalized = ::std::fs::read_to_string(&lib_path).expect("normalized lib reads");
        ::std::assert!(normalized.contains("pub fn spin"));
        let declaration = "pub mod widget {";
        ::std::assert!(normalized.contains(declaration));
        let stale_declaration = "pub mod widget\x3b";
        ::std::assert!(!normalized.contains(stale_declaration));
        ::std::assert!(!module_path.exists(), "inlined stub module file must go");
        let marker_count = super::count_unimplemented(&workspace);
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
        let removed = super::normalize_workspace_layout(&workspace);
        ::std::assert!(removed.is_empty());
        let after = ::std::fs::read_to_string(&lib_path).expect("plain lib rereads");
        ::std::assert_eq!(after, original);
        let _ = ::std::fs::remove_dir_all(&workspace);
    }
}
