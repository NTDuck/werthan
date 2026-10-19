//! Integration and validation tests for agent topology.

#[test]
fn test_role_depths() {
    let depth_root = ::topology::role_depth(::topology::AgentRole::RootOrchestrator);
    ::std::assert_eq!(depth_root, ::topology::TreeDepth::Depth0);

    let depth_analyst = ::topology::role_depth(::topology::AgentRole::LegacyAnalyst);
    ::std::assert_eq!(depth_analyst, ::topology::TreeDepth::Depth1);

    let depth_historian = ::topology::role_depth(::topology::AgentRole::BinaryHistorian);
    ::std::assert_eq!(depth_historian, ::topology::TreeDepth::Depth1);

    let depth_rules = ::topology::role_depth(::topology::AgentRole::BusinessRulesExtractor);
    ::std::assert_eq!(depth_rules, ::topology::TreeDepth::Depth1);

    let depth_critic = ::topology::role_depth(::topology::AgentRole::ArchitectureCritic);
    ::std::assert_eq!(depth_critic, ::topology::TreeDepth::Depth1);

    let depth_auditor = ::topology::role_depth(::topology::AgentRole::SecurityAuditor);
    ::std::assert_eq!(depth_auditor, ::topology::TreeDepth::Depth1);

    let depth_mediator = ::topology::role_depth(::topology::AgentRole::BoundaryMediator);
    ::std::assert_eq!(depth_mediator, ::topology::TreeDepth::Depth1);

    let depth_lead = ::topology::role_depth(::topology::AgentRole::ModuleLead);
    ::std::assert_eq!(depth_lead, ::topology::TreeDepth::Depth1);

    let depth_test = ::topology::role_depth(::topology::AgentRole::TestEngineer);
    ::std::assert_eq!(depth_test, ::topology::TreeDepth::Depth2);

    let depth_coder = ::topology::role_depth(::topology::AgentRole::TargetCoder);
    ::std::assert_eq!(depth_coder, ::topology::TreeDepth::Depth2);
}

#[test]
fn test_spawning_rules() {
    let can_root_spawn = ::topology::can_spawn(::topology::AgentRole::RootOrchestrator);
    ::std::assert!(can_root_spawn);

    let can_lead_spawn = ::topology::can_spawn(::topology::AgentRole::ModuleLead);
    ::std::assert!(can_lead_spawn);

    let leaves = [
        ::topology::AgentRole::LegacyAnalyst,
        ::topology::AgentRole::BinaryHistorian,
        ::topology::AgentRole::BusinessRulesExtractor,
        ::topology::AgentRole::ArchitectureCritic,
        ::topology::AgentRole::SecurityAuditor,
        ::topology::AgentRole::BoundaryMediator,
        ::topology::AgentRole::TestEngineer,
        ::topology::AgentRole::TargetCoder,
    ];

    for leaf in leaves {
        let can_leaf_spawn = ::topology::can_spawn(leaf);
        ::std::assert!(!can_leaf_spawn);

        for candidate in leaves {
            let can_spawn_leaf_child = ::topology::can_spawn_child(leaf, candidate);
            ::std::assert!(!can_spawn_leaf_child);
        }
    }

    let depth1_targets = [
        ::topology::AgentRole::LegacyAnalyst,
        ::topology::AgentRole::BinaryHistorian,
        ::topology::AgentRole::BusinessRulesExtractor,
        ::topology::AgentRole::ArchitectureCritic,
        ::topology::AgentRole::SecurityAuditor,
        ::topology::AgentRole::BoundaryMediator,
        ::topology::AgentRole::ModuleLead,
    ];

    for child in depth1_targets {
        let root_spawn_d1 = ::topology::can_spawn_child(::topology::AgentRole::RootOrchestrator, child);
        ::std::assert!(root_spawn_d1);
    }

    let root_cannot_spawn_root =
        ::topology::can_spawn_child(::topology::AgentRole::RootOrchestrator, ::topology::AgentRole::RootOrchestrator);
    ::std::assert!(!root_cannot_spawn_root);

    let root_cannot_spawn_coder =
        ::topology::can_spawn_child(::topology::AgentRole::RootOrchestrator, ::topology::AgentRole::TargetCoder);
    ::std::assert!(!root_cannot_spawn_coder);

    let root_cannot_spawn_test =
        ::topology::can_spawn_child(::topology::AgentRole::RootOrchestrator, ::topology::AgentRole::TestEngineer);
    ::std::assert!(!root_cannot_spawn_test);

    let lead_can_spawn_coder =
        ::topology::can_spawn_child(::topology::AgentRole::ModuleLead, ::topology::AgentRole::TargetCoder);
    ::std::assert!(lead_can_spawn_coder);

    let lead_can_spawn_test =
        ::topology::can_spawn_child(::topology::AgentRole::ModuleLead, ::topology::AgentRole::TestEngineer);
    ::std::assert!(lead_can_spawn_test);

    let lead_can_spawn_analyst =
        ::topology::can_spawn_child(::topology::AgentRole::ModuleLead, ::topology::AgentRole::LegacyAnalyst);
    ::std::assert!(lead_can_spawn_analyst);

    let lead_can_spawn_historian =
        ::topology::can_spawn_child(::topology::AgentRole::ModuleLead, ::topology::AgentRole::BinaryHistorian);
    ::std::assert!(lead_can_spawn_historian);

    let lead_cannot_spawn_critic =
        ::topology::can_spawn_child(::topology::AgentRole::ModuleLead, ::topology::AgentRole::ArchitectureCritic);
    ::std::assert!(!lead_cannot_spawn_critic);

    let lead_cannot_spawn_root =
        ::topology::can_spawn_child(::topology::AgentRole::ModuleLead, ::topology::AgentRole::RootOrchestrator);
    ::std::assert!(!lead_cannot_spawn_root);
}

#[test]
fn test_file_mutation_restrictions() {
    let auth_mod = ::core::option::Option::Some("auth");

    let p_auth_src = ::std::path::Path::new("modernized/auth/src/lib.rs");
    let analyst_read_only = ::topology::validate_file_write(::topology::AgentRole::LegacyAnalyst, p_auth_src, auth_mod);
    ::std::assert!(analyst_read_only.is_err());

    let p_analysis_report = ::std::path::Path::new("analysis/report.md");
    let historian_read_only = ::topology::validate_file_write(
        ::topology::AgentRole::BinaryHistorian,
        p_analysis_report,
        ::core::option::Option::None,
    );
    ::std::assert!(historian_read_only.is_err());

    let critic_read_only =
        ::topology::validate_file_write(::topology::AgentRole::ArchitectureCritic, p_auth_src, auth_mod);
    ::std::assert!(critic_read_only.is_err());

    let auditor_read_only =
        ::topology::validate_file_write(::topology::AgentRole::SecurityAuditor, p_auth_src, auth_mod);
    ::std::assert!(auditor_read_only.is_err());

    let coder_valid = ::topology::validate_file_write(::topology::AgentRole::TargetCoder, p_auth_src, auth_mod);
    ::std::assert!(coder_valid.is_ok());

    let p_auth_tests = ::std::path::Path::new("modernized/auth/tests/auth_test.rs");
    let coder_invalid_test =
        ::topology::validate_file_write(::topology::AgentRole::TargetCoder, p_auth_tests, auth_mod);
    ::std::assert!(coder_invalid_test.is_err());

    let p_billing_src = ::std::path::Path::new("modernized/billing/src/lib.rs");
    let coder_invalid_module =
        ::topology::validate_file_write(::topology::AgentRole::TargetCoder, p_billing_src, auth_mod);
    ::std::assert!(coder_invalid_module.is_err());

    let p_src_main = ::std::path::Path::new("src/main.rs");
    let coder_invalid_outside =
        ::topology::validate_file_write(::topology::AgentRole::TargetCoder, p_src_main, ::core::option::Option::None);
    ::std::assert!(coder_invalid_outside.is_err());

    let test_valid_mod = ::topology::validate_file_write(::topology::AgentRole::TestEngineer, p_auth_tests, auth_mod);
    ::std::assert!(test_valid_mod.is_ok());

    let p_integration_test = ::std::path::Path::new("tests/integration_test.rs");
    let test_valid_generic = ::topology::validate_file_write(
        ::topology::AgentRole::TestEngineer,
        p_integration_test,
        ::core::option::Option::None,
    );
    ::std::assert!(test_valid_generic.is_ok());

    let test_invalid_src = ::topology::validate_file_write(::topology::AgentRole::TestEngineer, p_auth_src, auth_mod);
    ::std::assert!(test_invalid_src.is_err());

    let p_analysis_ledger = ::std::path::Path::new("analysis/ledger.json");
    let root_valid_analysis = ::topology::validate_file_write(
        ::topology::AgentRole::RootOrchestrator,
        p_analysis_ledger,
        ::core::option::Option::None,
    );
    ::std::assert!(root_valid_analysis.is_ok());

    let p_state = ::std::path::Path::new("state.json");
    let root_valid_state =
        ::topology::validate_file_write(::topology::AgentRole::RootOrchestrator, p_state, ::core::option::Option::None);
    ::std::assert!(root_valid_state.is_ok());

    let root_invalid_src =
        ::topology::validate_file_write(::topology::AgentRole::RootOrchestrator, p_auth_src, auth_mod);
    ::std::assert!(root_invalid_src.is_err());

    let p_todo = ::std::path::Path::new("worktree/auth/todo.md");
    let lead_valid_todo = ::topology::validate_file_write(::topology::AgentRole::ModuleLead, p_todo, auth_mod);
    ::std::assert!(lead_valid_todo.is_ok());

    let lead_invalid_src = ::topology::validate_file_write(::topology::AgentRole::ModuleLead, p_auth_src, auth_mod);
    ::std::assert!(lead_invalid_src.is_err());
}

#[test]
fn test_tool_mounting_restrictions() {
    let historian_scc = ::topology::validate_tool_call(::topology::AgentRole::BinaryHistorian, "rea_scc_analysis");
    ::std::assert!(historian_scc.is_ok());

    let historian_graph = ::topology::validate_tool_call(::topology::AgentRole::BinaryHistorian, "rea_symbol_graph");
    ::std::assert!(historian_graph.is_ok());

    let historian_slice = ::topology::validate_tool_call(::topology::AgentRole::BinaryHistorian, "rea_dataflow_slice");
    ::std::assert!(historian_slice.is_ok());

    let historian_decompile =
        ::topology::validate_tool_call(::topology::AgentRole::BinaryHistorian, "rea_decompile_function");
    ::std::assert!(historian_decompile.is_ok());

    let historian_xrefs = ::topology::validate_tool_call(::topology::AgentRole::BinaryHistorian, "rea_xrefs");
    ::std::assert!(historian_xrefs.is_ok());

    let historian_retained =
        ::topology::validate_tool_call(::topology::AgentRole::BinaryHistorian, "rea_query_retained_view");
    ::std::assert!(historian_retained.is_ok());

    let rules_retained =
        ::topology::validate_tool_call(::topology::AgentRole::BusinessRulesExtractor, "rea_query_retained_view");
    ::std::assert!(rules_retained.is_ok());

    let rules_decompile =
        ::topology::validate_tool_call(::topology::AgentRole::BusinessRulesExtractor, "rea_decompile_function");
    ::std::assert!(rules_decompile.is_err());

    let rules_scc = ::topology::validate_tool_call(::topology::AgentRole::BusinessRulesExtractor, "rea_scc_analysis");
    ::std::assert!(rules_scc.is_err());

    let rules_slice =
        ::topology::validate_tool_call(::topology::AgentRole::BusinessRulesExtractor, "rea_dataflow_slice");
    ::std::assert!(rules_slice.is_err());

    let other_roles = [
        ::topology::AgentRole::RootOrchestrator,
        ::topology::AgentRole::LegacyAnalyst,
        ::topology::AgentRole::ArchitectureCritic,
        ::topology::AgentRole::SecurityAuditor,
        ::topology::AgentRole::BoundaryMediator,
        ::topology::AgentRole::ModuleLead,
        ::topology::AgentRole::TestEngineer,
        ::topology::AgentRole::TargetCoder,
    ];

    for role in other_roles {
        let call_res = ::topology::validate_tool_call(role, "rea_decompile_function");
        ::std::assert!(call_res.is_err());

        let call_query = ::topology::validate_tool_call(role, "rea_query_retained_view");
        ::std::assert!(call_query.is_err());
    }

    let search_coder = ::topology::validate_tool_call(::topology::AgentRole::TargetCoder, "search");
    ::std::assert!(search_coder.is_ok());

    let replace_coder = ::topology::validate_tool_call(::topology::AgentRole::TargetCoder, "hashline_replace");
    ::std::assert!(replace_coder.is_ok());

    let replace_analyst = ::topology::validate_tool_call(::topology::AgentRole::LegacyAnalyst, "hashline_replace");
    ::std::assert!(replace_analyst.is_err());
}

#[test]
fn test_mutation_scope_mappings() {
    let scope_root = ::topology::mutation_scope(::topology::AgentRole::RootOrchestrator);
    ::std::assert_eq!(scope_root, ::topology::FileMutationScope::AnalysisLedgerOnly);

    let scope_lead = ::topology::mutation_scope(::topology::AgentRole::ModuleLead);
    ::std::assert_eq!(scope_lead, ::topology::FileMutationScope::WorktreeTodoOnly);

    let scope_analyst = ::topology::mutation_scope(::topology::AgentRole::LegacyAnalyst);
    ::std::assert_eq!(scope_analyst, ::topology::FileMutationScope::ReadOnly);

    let scope_historian = ::topology::mutation_scope(::topology::AgentRole::BinaryHistorian);
    ::std::assert_eq!(scope_historian, ::topology::FileMutationScope::ReadOnly);

    let scope_coder = ::topology::mutation_scope(::topology::AgentRole::TargetCoder);
    ::std::assert!(::core::matches!(scope_coder, ::topology::FileMutationScope::Scoped { .. }));
}

#[test]
fn test_agent_node_builder() {
    let auth_str = "auth".to_string();
    let auth_opt = ::core::option::Option::Some(auth_str);
    let node = ::topology::AgentNode::builder()
        .role(::topology::AgentRole::TargetCoder)
        .module_name("auth".to_string())
        .build();
    ::std::assert_eq!(node.role, ::topology::AgentRole::TargetCoder);
    ::std::assert_eq!(node.module_name, auth_opt);
}
