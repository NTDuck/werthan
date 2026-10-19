//! Integration and validation tests for agent topology.

#[test]
fn test_role_depths() {
    let depth_root = ::operative::role_depth(::operative::AgentRole::RootOrchestrator);
    ::std::assert_eq!(depth_root, ::operative::TreeDepth::Depth0);

    let depth_analyst = ::operative::role_depth(::operative::AgentRole::LegacyAnalyst);
    ::std::assert_eq!(depth_analyst, ::operative::TreeDepth::Depth1);

    let depth_historian = ::operative::role_depth(::operative::AgentRole::BinaryHistorian);
    ::std::assert_eq!(depth_historian, ::operative::TreeDepth::Depth1);

    let depth_rules = ::operative::role_depth(::operative::AgentRole::BusinessRulesExtractor);
    ::std::assert_eq!(depth_rules, ::operative::TreeDepth::Depth1);

    let depth_critic = ::operative::role_depth(::operative::AgentRole::ArchitectureCritic);
    ::std::assert_eq!(depth_critic, ::operative::TreeDepth::Depth1);

    let depth_auditor = ::operative::role_depth(::operative::AgentRole::SecurityAuditor);
    ::std::assert_eq!(depth_auditor, ::operative::TreeDepth::Depth1);

    let depth_mediator = ::operative::role_depth(::operative::AgentRole::BoundaryMediator);
    ::std::assert_eq!(depth_mediator, ::operative::TreeDepth::Depth1);

    let depth_lead = ::operative::role_depth(::operative::AgentRole::ModuleLead);
    ::std::assert_eq!(depth_lead, ::operative::TreeDepth::Depth1);

    let depth_test = ::operative::role_depth(::operative::AgentRole::TestEngineer);
    ::std::assert_eq!(depth_test, ::operative::TreeDepth::Depth2);

    let depth_coder = ::operative::role_depth(::operative::AgentRole::TargetCoder);
    ::std::assert_eq!(depth_coder, ::operative::TreeDepth::Depth2);
}

#[test]
fn test_spawning_rules() {
    let can_root_spawn = ::operative::can_spawn(::operative::AgentRole::RootOrchestrator);
    ::std::assert!(can_root_spawn);

    let can_lead_spawn = ::operative::can_spawn(::operative::AgentRole::ModuleLead);
    ::std::assert!(can_lead_spawn);

    let leaves = [
        ::operative::AgentRole::LegacyAnalyst,
        ::operative::AgentRole::BinaryHistorian,
        ::operative::AgentRole::BusinessRulesExtractor,
        ::operative::AgentRole::ArchitectureCritic,
        ::operative::AgentRole::SecurityAuditor,
        ::operative::AgentRole::BoundaryMediator,
        ::operative::AgentRole::TestEngineer,
        ::operative::AgentRole::TargetCoder,
    ];

    for leaf in leaves {
        let can_leaf_spawn = ::operative::can_spawn(leaf);
        ::std::assert!(!can_leaf_spawn);

        for candidate in leaves {
            let can_spawn_leaf_child = ::operative::can_spawn_child(leaf, candidate);
            ::std::assert!(!can_spawn_leaf_child);
        }
    }

    let depth1_targets = [
        ::operative::AgentRole::LegacyAnalyst,
        ::operative::AgentRole::BinaryHistorian,
        ::operative::AgentRole::BusinessRulesExtractor,
        ::operative::AgentRole::ArchitectureCritic,
        ::operative::AgentRole::SecurityAuditor,
        ::operative::AgentRole::BoundaryMediator,
        ::operative::AgentRole::ModuleLead,
    ];

    for child in depth1_targets {
        let root_spawn_d1 = ::operative::can_spawn_child(::operative::AgentRole::RootOrchestrator, child);
        ::std::assert!(root_spawn_d1);
    }

    let root_cannot_spawn_root =
        ::operative::can_spawn_child(::operative::AgentRole::RootOrchestrator, ::operative::AgentRole::RootOrchestrator);
    ::std::assert!(!root_cannot_spawn_root);

    let root_cannot_spawn_coder =
        ::operative::can_spawn_child(::operative::AgentRole::RootOrchestrator, ::operative::AgentRole::TargetCoder);
    ::std::assert!(!root_cannot_spawn_coder);

    let root_cannot_spawn_test =
        ::operative::can_spawn_child(::operative::AgentRole::RootOrchestrator, ::operative::AgentRole::TestEngineer);
    ::std::assert!(!root_cannot_spawn_test);

    let lead_can_spawn_coder =
        ::operative::can_spawn_child(::operative::AgentRole::ModuleLead, ::operative::AgentRole::TargetCoder);
    ::std::assert!(lead_can_spawn_coder);

    let lead_can_spawn_test =
        ::operative::can_spawn_child(::operative::AgentRole::ModuleLead, ::operative::AgentRole::TestEngineer);
    ::std::assert!(lead_can_spawn_test);

    let lead_can_spawn_analyst =
        ::operative::can_spawn_child(::operative::AgentRole::ModuleLead, ::operative::AgentRole::LegacyAnalyst);
    ::std::assert!(lead_can_spawn_analyst);

    let lead_can_spawn_historian =
        ::operative::can_spawn_child(::operative::AgentRole::ModuleLead, ::operative::AgentRole::BinaryHistorian);
    ::std::assert!(lead_can_spawn_historian);

    let lead_cannot_spawn_critic =
        ::operative::can_spawn_child(::operative::AgentRole::ModuleLead, ::operative::AgentRole::ArchitectureCritic);
    ::std::assert!(!lead_cannot_spawn_critic);

    let lead_cannot_spawn_root =
        ::operative::can_spawn_child(::operative::AgentRole::ModuleLead, ::operative::AgentRole::RootOrchestrator);
    ::std::assert!(!lead_cannot_spawn_root);
}

#[test]
fn test_file_mutation_restrictions() {
    let auth_mod = ::core::option::Option::Some("auth");

    let p_auth_src = ::std::path::Path::new("modernized/auth/src/lib.rs");
    let analyst_read_only = ::operative::validate_file_write(::operative::AgentRole::LegacyAnalyst, p_auth_src, auth_mod);
    ::std::assert!(analyst_read_only.is_err());

    let p_analysis_report = ::std::path::Path::new("analysis/report.md");
    let historian_read_only = ::operative::validate_file_write(
        ::operative::AgentRole::BinaryHistorian,
        p_analysis_report,
        ::core::option::Option::None,
    );
    ::std::assert!(historian_read_only.is_err());

    let critic_read_only =
        ::operative::validate_file_write(::operative::AgentRole::ArchitectureCritic, p_auth_src, auth_mod);
    ::std::assert!(critic_read_only.is_err());

    let auditor_read_only =
        ::operative::validate_file_write(::operative::AgentRole::SecurityAuditor, p_auth_src, auth_mod);
    ::std::assert!(auditor_read_only.is_err());

    let coder_valid = ::operative::validate_file_write(::operative::AgentRole::TargetCoder, p_auth_src, auth_mod);
    ::std::assert!(coder_valid.is_ok());

    let p_auth_tests = ::std::path::Path::new("modernized/auth/tests/auth_test.rs");
    let coder_invalid_test =
        ::operative::validate_file_write(::operative::AgentRole::TargetCoder, p_auth_tests, auth_mod);
    ::std::assert!(coder_invalid_test.is_err());

    let p_billing_src = ::std::path::Path::new("modernized/billing/src/lib.rs");
    let coder_invalid_module =
        ::operative::validate_file_write(::operative::AgentRole::TargetCoder, p_billing_src, auth_mod);
    ::std::assert!(coder_invalid_module.is_err());

    let p_src_main = ::std::path::Path::new("src/main.rs");
    let coder_invalid_outside =
        ::operative::validate_file_write(::operative::AgentRole::TargetCoder, p_src_main, ::core::option::Option::None);
    ::std::assert!(coder_invalid_outside.is_err());

    let test_valid_mod = ::operative::validate_file_write(::operative::AgentRole::TestEngineer, p_auth_tests, auth_mod);
    ::std::assert!(test_valid_mod.is_ok());

    let p_integration_test = ::std::path::Path::new("tests/integration_test.rs");
    let test_valid_generic = ::operative::validate_file_write(
        ::operative::AgentRole::TestEngineer,
        p_integration_test,
        ::core::option::Option::None,
    );
    ::std::assert!(test_valid_generic.is_ok());

    let test_invalid_src = ::operative::validate_file_write(::operative::AgentRole::TestEngineer, p_auth_src, auth_mod);
    ::std::assert!(test_invalid_src.is_err());

    let p_analysis_ledger = ::std::path::Path::new("analysis/ledger.json");
    let root_valid_analysis = ::operative::validate_file_write(
        ::operative::AgentRole::RootOrchestrator,
        p_analysis_ledger,
        ::core::option::Option::None,
    );
    ::std::assert!(root_valid_analysis.is_ok());

    let p_state = ::std::path::Path::new("state.json");
    let root_valid_state =
        ::operative::validate_file_write(::operative::AgentRole::RootOrchestrator, p_state, ::core::option::Option::None);
    ::std::assert!(root_valid_state.is_ok());

    let root_invalid_src =
        ::operative::validate_file_write(::operative::AgentRole::RootOrchestrator, p_auth_src, auth_mod);
    ::std::assert!(root_invalid_src.is_err());

    let p_todo = ::std::path::Path::new("worktree/auth/todo.md");
    let lead_valid_todo = ::operative::validate_file_write(::operative::AgentRole::ModuleLead, p_todo, auth_mod);
    ::std::assert!(lead_valid_todo.is_ok());

    let lead_invalid_src = ::operative::validate_file_write(::operative::AgentRole::ModuleLead, p_auth_src, auth_mod);
    ::std::assert!(lead_invalid_src.is_err());
}

#[test]
fn test_tool_mounting_restrictions() {
    let historian_scc = ::operative::validate_tool_call(::operative::AgentRole::BinaryHistorian, "rea_scc_analysis");
    ::std::assert!(historian_scc.is_ok());

    let historian_graph = ::operative::validate_tool_call(::operative::AgentRole::BinaryHistorian, "rea_symbol_graph");
    ::std::assert!(historian_graph.is_ok());

    let historian_slice = ::operative::validate_tool_call(::operative::AgentRole::BinaryHistorian, "rea_dataflow_slice");
    ::std::assert!(historian_slice.is_ok());

    let historian_decompile =
        ::operative::validate_tool_call(::operative::AgentRole::BinaryHistorian, "rea_decompile_function");
    ::std::assert!(historian_decompile.is_ok());

    let historian_xrefs = ::operative::validate_tool_call(::operative::AgentRole::BinaryHistorian, "rea_xrefs");
    ::std::assert!(historian_xrefs.is_ok());

    let historian_retained =
        ::operative::validate_tool_call(::operative::AgentRole::BinaryHistorian, "rea_query_retained_view");
    ::std::assert!(historian_retained.is_ok());

    let rules_retained =
        ::operative::validate_tool_call(::operative::AgentRole::BusinessRulesExtractor, "rea_query_retained_view");
    ::std::assert!(rules_retained.is_ok());

    let rules_decompile =
        ::operative::validate_tool_call(::operative::AgentRole::BusinessRulesExtractor, "rea_decompile_function");
    ::std::assert!(rules_decompile.is_err());

    let rules_scc = ::operative::validate_tool_call(::operative::AgentRole::BusinessRulesExtractor, "rea_scc_analysis");
    ::std::assert!(rules_scc.is_err());

    let rules_slice =
        ::operative::validate_tool_call(::operative::AgentRole::BusinessRulesExtractor, "rea_dataflow_slice");
    ::std::assert!(rules_slice.is_err());

    let other_roles = [
        ::operative::AgentRole::RootOrchestrator,
        ::operative::AgentRole::LegacyAnalyst,
        ::operative::AgentRole::ArchitectureCritic,
        ::operative::AgentRole::SecurityAuditor,
        ::operative::AgentRole::BoundaryMediator,
        ::operative::AgentRole::ModuleLead,
        ::operative::AgentRole::TestEngineer,
        ::operative::AgentRole::TargetCoder,
    ];

    for role in other_roles {
        let call_res = ::operative::validate_tool_call(role, "rea_decompile_function");
        ::std::assert!(call_res.is_err());

        let call_query = ::operative::validate_tool_call(role, "rea_query_retained_view");
        ::std::assert!(call_query.is_err());
    }

    let search_coder = ::operative::validate_tool_call(::operative::AgentRole::TargetCoder, "search");
    ::std::assert!(search_coder.is_ok());

    let replace_coder = ::operative::validate_tool_call(::operative::AgentRole::TargetCoder, "hashline_replace");
    ::std::assert!(replace_coder.is_ok());

    let replace_analyst = ::operative::validate_tool_call(::operative::AgentRole::LegacyAnalyst, "hashline_replace");
    ::std::assert!(replace_analyst.is_err());
}

#[test]
fn test_mutation_scope_mappings() {
    let scope_root = ::operative::mutation_scope(::operative::AgentRole::RootOrchestrator);
    ::std::assert_eq!(scope_root, ::operative::FileMutationScope::AnalysisLedgerOnly);

    let scope_lead = ::operative::mutation_scope(::operative::AgentRole::ModuleLead);
    ::std::assert_eq!(scope_lead, ::operative::FileMutationScope::WorktreeTodoOnly);

    let scope_analyst = ::operative::mutation_scope(::operative::AgentRole::LegacyAnalyst);
    ::std::assert_eq!(scope_analyst, ::operative::FileMutationScope::ReadOnly);

    let scope_historian = ::operative::mutation_scope(::operative::AgentRole::BinaryHistorian);
    ::std::assert_eq!(scope_historian, ::operative::FileMutationScope::ReadOnly);

    let scope_coder = ::operative::mutation_scope(::operative::AgentRole::TargetCoder);
    ::std::assert!(::core::matches!(scope_coder, ::operative::FileMutationScope::Scoped { .. }));
}

#[test]
fn test_agent_node_builder() {
    let auth_str = "auth".to_string();
    let auth_opt = ::core::option::Option::Some(auth_str);
    let node = ::operative::AgentNode::builder()
        .role(::operative::AgentRole::TargetCoder)
        .module_name("auth".to_string())
        .build();
    ::std::assert_eq!(node.role, ::operative::AgentRole::TargetCoder);
    ::std::assert_eq!(node.module_name, auth_opt);
}
