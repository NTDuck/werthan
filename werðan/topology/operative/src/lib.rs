//! Asymmetric agent hierarchy and spawning topology.


/// Unique identifier for an operative.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::hash::Hash,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct OperativeId(pub ::std::string::String);

/// Model tier for an operative.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum ModelTier {
    /// System 2 frontier LLM.
    System2Frontier,
    /// System 1 local fast model.
    System1Laya,
}

/// Allowed tools for an operative, enforcing read/write isolation.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::core::default::Default
)]
pub struct ToolSet {
    /// List of allowed tool names.
    pub allowed_tools: ::std::vec::Vec<::std::string::String>,
    /// Whether target code can be read.
    pub can_read_target_code: ::core::primitive::bool,
    /// Whether test assertions can be mutated.
    pub can_mutate_tests: ::core::primitive::bool,
    /// Whether target code can be mutated.
    pub can_mutate_target_code: ::core::primitive::bool,
}

/// Guardrail policy for an operative.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::core::default::Default
)]
pub enum GuardrailPolicy {
    /// Strict policy.
    #[default]
    Strict,
    /// Audit only.
    AuditOnly,
    /// Guardrails disabled.
    Disabled,
}

/// Blueprint for building an operative.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct OperativeBlueprint {
    /// Operative identifier.
    pub id: self::OperativeId,
    /// Role prompt template text.
    pub role_prompt: ::std::string::String,
    /// Model execution tier.
    pub model_tier: self::ModelTier,
    /// Isolation and tool permissions.
    pub allowed_tools: self::ToolSet,
    /// Laya guardrail policy.
    pub laya_guardrail: self::GuardrailPolicy,
}

/// Telemetry stream chunk emitted by an operative.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct OperativeStreamChunk {
    /// Source operative identifier.
    pub operative_id: self::OperativeId,
    /// Optional parent cluster identifier.
    pub cluster_id: ::core::option::Option<::std::string::String>,
    /// Optional target module vertex identifier.
    pub vertex_id: ::core::option::Option<::std::string::String>,
    /// Stream delta text chunk.
    pub delta: ::std::string::String,
    /// True when final chunk for current turn.
    pub is_final: ::core::primitive::bool,
}
/// Distinct agent roles in the asymmetric tree.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::hash::Hash,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum AgentRole {
    RootOrchestrator,
    LegacyAnalyst,
    BinaryHistorian,
    BusinessRulesExtractor,
    ArchitectureCritic,
    SecurityAuditor,
    BoundaryMediator,
    ModuleLead,
    TestEngineer,
    TargetCoder,
}

impl self::AgentRole {
    /// Return the static role depth in the agent tree.
    pub fn depth(self) -> self::TreeDepth {
        self::role_depth(self)
    }

    /// Check if this role can spawn any subagents.
    pub fn can_spawn(self) -> ::core::primitive::bool {
        self::can_spawn(self)
    }

    /// Check if this role can spawn the designated child role.
    pub fn can_spawn_child(self, child: self::AgentRole) -> ::core::primitive::bool {
        self::can_spawn_child(self, child)
    }

    /// Return the file mutation scope for this role.
    pub fn mutation_scope(self) -> self::FileMutationScope {
        self::mutation_scope(self)
    }

    /// Return the list of tools mounted on this role.
    pub fn mounted_tools(self) -> &'static [&'static ::core::primitive::str] {
        self::mounted_tools(self)
    }
}

/// Depth in the asymmetric agent tree.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::cmp::PartialOrd,
    ::core::cmp::Ord,
    ::core::hash::Hash,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum TreeDepth {
    Depth0,
    Depth1,
    Depth2,
}

/// Subagent spawning permissions.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum SpawningPermission {
    /// Can spawn Depth 1 subagents.
    Depth1,
    /// Can spawn Depth 2 subagents and on-demand oracles.
    Depth2,
    /// Cannot spawn subagents.
    None,
}

/// Permitted file mutation boundary for a role.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum FileMutationScope {
    AnalysisLedgerOnly,
    WorktreeTodoOnly,
    ReadOnly,
    Scoped {
        allowed_prefixes: ::std::vec::Vec<::std::string::String>,
    },
}

/// Canonical tools mountable in the topology.
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::hash::Hash,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum MountedTool {
    Search,
    ReadHashline,
    HashlineReplace,
    ReaSccAnalysis,
    ReaSymbolGraph,
    ReaDataflowSlice,
    ReaDecompileFunction,
    ReaXrefs,
    ReaQueryRetainedView,
    VerifyRunner,
}

impl self::MountedTool {
    /// Return the string identifier for the tool.
    pub fn as_str(&self) -> &'static ::core::primitive::str {
        match self {
            Self::Search => "search",
            Self::ReadHashline => "read_hashline",
            Self::HashlineReplace => "hashline_replace",
            Self::ReaSccAnalysis => "rea_scc_analysis",
            Self::ReaSymbolGraph => "rea_symbol_graph",
            Self::ReaDataflowSlice => "rea_dataflow_slice",
            Self::ReaDecompileFunction => "rea_decompile_function",
            Self::ReaXrefs => "rea_xrefs",
            Self::ReaQueryRetainedView => "rea_query_retained_view",
            Self::VerifyRunner => "verify_runner",
        }
    }
}

/// Topology agent node definition.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::bon::Builder
)]
pub struct AgentNode {
    /// Agent role in the topology.
    pub role: self::AgentRole,
    /// Optional module identifier for module-scoped roles.
    pub module_name: ::core::option::Option<::std::string::String>,
}

/// Validate whether an agent role has permission to write to a path.
pub fn validate_file_write(
    role: self::AgentRole,
    path: &::std::path::Path,
    module_name: ::core::option::Option<&::core::primitive::str>,
) -> ::core::result::Result<(), ::std::string::String> {
    let raw_str = path.to_string_lossy();
    let forward_slash = raw_str.replace('\\', "/");
    let clean_path = forward_slash.strip_prefix("./").unwrap_or(&forward_slash);

    match role {
        self::AgentRole::LegacyAnalyst
        | self::AgentRole::BinaryHistorian
        | self::AgentRole::BusinessRulesExtractor
        | self::AgentRole::ArchitectureCritic
        | self::AgentRole::SecurityAuditor => {
            let msg = ::std::format!("Role {role:?} is read-only and cannot mutate files: {clean_path}");
            ::core::result::Result::Err(msg)
        },
        self::AgentRole::RootOrchestrator => {
            let is_analysis = clean_path.starts_with("analysis/")
                || clean_path == "analysis"
                || clean_path.contains("analysis_ledger");
            let is_state = clean_path == "state.json"
                || clean_path.ends_with("/state.json")
                || clean_path == ".state"
                || clean_path.ends_with("/.state")
                || clean_path.starts_with("state/")
                || clean_path == "state";
            if is_analysis || is_state {
                ::core::result::Result::Ok(())
            } else {
                let msg = ::std::format!(
                    "RootOrchestrator can only write to analysis/ ledger and state file, denied: {clean_path}"
                );
                ::core::result::Result::Err(msg)
            }
        },
        self::AgentRole::ModuleLead => {
            let is_todo = clean_path.ends_with("todo.md")
                || clean_path.ends_with("TODO.md")
                || clean_path.ends_with("todo.json")
                || clean_path.ends_with(".todo")
                || clean_path.contains("todo")
                || clean_path.contains("TODO");
            if is_todo {
                ::core::result::Result::Ok(())
            } else {
                let msg =
                    ::std::format!("ModuleLead can only write to local worktree <todo> ledger, denied: {clean_path}");
                ::core::result::Result::Err(msg)
            }
        },
        self::AgentRole::TargetCoder => {
            let is_allowed = match module_name {
                ::core::option::Option::Some(module) => {
                    let expected_prefix = ::std::format!("modernized/{module}/src/");
                    let is_prefix = clean_path.starts_with(&expected_prefix);
                    let is_exact = clean_path == ::std::format!("modernized/{module}/src");
                    is_prefix || is_exact
                },
                ::core::option::Option::None => {
                    let is_modernized = clean_path.starts_with("modernized/");
                    let has_src = clean_path.contains("/src/") || clean_path.ends_with("/src");
                    is_modernized && has_src
                },
            };
            if is_allowed {
                ::core::result::Result::Ok(())
            } else {
                let msg = match module_name {
                    ::core::option::Option::Some(module) => ::std::format!(
                        "TargetCoder cannot write outside modernized/{module}/src/**, denied: {clean_path}"
                    ),
                    ::core::option::Option::None => {
                        ::std::format!("TargetCoder cannot write outside modernized/<mod>/src/**, denied: {clean_path}")
                    },
                };
                ::core::result::Result::Err(msg)
            }
        },
        self::AgentRole::TestEngineer => {
            let is_allowed = match module_name {
                ::core::option::Option::Some(module) => {
                    let expected_prefix = ::std::format!("modernized/{module}/tests/");
                    let is_prefix = clean_path.starts_with(&expected_prefix);
                    let is_exact = clean_path == ::std::format!("modernized/{module}/tests");
                    let is_tests = clean_path.starts_with("tests/") || clean_path == "tests";
                    is_prefix || is_exact || is_tests
                },
                ::core::option::Option::None => {
                    let is_tests = clean_path.starts_with("tests/") || clean_path == "tests";
                    let is_modernized = clean_path.starts_with("modernized/");
                    let has_tests = clean_path.contains("/tests/") || clean_path.ends_with("/tests");
                    is_tests || (is_modernized && has_tests)
                },
            };
            if is_allowed {
                ::core::result::Result::Ok(())
            } else {
                let msg = match module_name {
                    ::core::option::Option::Some(module) => ::std::format!(
                        "TestEngineer cannot write outside modernized/{module}/tests/** or tests/**, denied: {clean_path}"
                    ),
                    ::core::option::Option::None => ::std::format!(
                        "TestEngineer cannot write outside tests/**, denied: {clean_path}"
                    ),
                };
                ::core::result::Result::Err(msg)
            }
        },
        self::AgentRole::BoundaryMediator => {
            let is_interface = clean_path.starts_with("interfaces/")
                || clean_path.contains("interface")
                || clean_path.contains("adapter");
            let is_modernized = clean_path.starts_with("modernized/");
            let is_src = clean_path.starts_with("src/");
            let is_allowed = is_interface || is_modernized || is_src;
            if is_allowed {
                ::core::result::Result::Ok(())
            } else {
                let msg = ::std::format!(
                    "BoundaryMediator cannot write outside module interface boundaries, denied: {clean_path}"
                );
                ::core::result::Result::Err(msg)
            }
        },
    }
}

/// Validate whether an agent role has permission to call a tool.
pub fn validate_tool_call(
    role: self::AgentRole,
    tool_name: &::core::primitive::str,
) -> ::core::result::Result<(), ::std::string::String> {
    let tools = self::mounted_tools(role);
    let is_mounted = tools.contains(&tool_name);
    if is_mounted {
        ::core::result::Result::Ok(())
    } else {
        let msg = ::std::format!("Tool '{tool_name}' is not mounted on role '{role:?}'");
        ::core::result::Result::Err(msg)
    }
}

/// Validate whether a parent role can spawn a specific child role.
pub fn can_spawn_child(parent: self::AgentRole, child: self::AgentRole) -> ::core::primitive::bool {
    match parent {
        self::AgentRole::RootOrchestrator => {
            let depth = self::role_depth(child);
            ::core::matches!(depth, self::TreeDepth::Depth1)
        },
        self::AgentRole::ModuleLead => {
            ::core::matches!(
                child,
                self::AgentRole::TestEngineer
                    | self::AgentRole::TargetCoder
                    | self::AgentRole::LegacyAnalyst
                    | self::AgentRole::BinaryHistorian
            )
        },
        _ => false,
    }
}

/// Check whether an agent role can spawn any subagents.
pub fn can_spawn(role: self::AgentRole) -> ::core::primitive::bool {
    ::core::matches!(role, self::AgentRole::RootOrchestrator | self::AgentRole::ModuleLead)
}

/// Return the hierarchical depth of an agent role in the asymmetric tree.
pub fn role_depth(role: self::AgentRole) -> self::TreeDepth {
    match role {
        self::AgentRole::RootOrchestrator => self::TreeDepth::Depth0,
        self::AgentRole::LegacyAnalyst
        | self::AgentRole::BinaryHistorian
        | self::AgentRole::BusinessRulesExtractor
        | self::AgentRole::ArchitectureCritic
        | self::AgentRole::SecurityAuditor
        | self::AgentRole::BoundaryMediator
        | self::AgentRole::ModuleLead => self::TreeDepth::Depth1,
        self::AgentRole::TestEngineer | self::AgentRole::TargetCoder => self::TreeDepth::Depth2,
    }
}

/// Return spawning permissions for a given role.
pub fn spawning_permission(role: self::AgentRole) -> self::SpawningPermission {
    match role {
        self::AgentRole::RootOrchestrator => self::SpawningPermission::Depth1,
        self::AgentRole::ModuleLead => self::SpawningPermission::Depth2,
        _ => self::SpawningPermission::None,
    }
}

/// Return the static file mutation scope for an agent role.
pub fn mutation_scope(role: self::AgentRole) -> self::FileMutationScope {
    match role {
        self::AgentRole::RootOrchestrator => self::FileMutationScope::AnalysisLedgerOnly,
        self::AgentRole::ModuleLead => self::FileMutationScope::WorktreeTodoOnly,
        self::AgentRole::LegacyAnalyst
        | self::AgentRole::BinaryHistorian
        | self::AgentRole::BusinessRulesExtractor
        | self::AgentRole::ArchitectureCritic
        | self::AgentRole::SecurityAuditor => self::FileMutationScope::ReadOnly,
        self::AgentRole::BoundaryMediator => self::FileMutationScope::Scoped {
            allowed_prefixes: ::std::vec![
                ::std::string::String::from("interfaces/"),
                ::std::string::String::from("adapters/"),
                ::std::string::String::from("modernized/"),
            ],
        },
        self::AgentRole::TestEngineer => self::FileMutationScope::Scoped {
            allowed_prefixes: ::std::vec![
                ::std::string::String::from("modernized/<mod>/tests/"),
                ::std::string::String::from("tests/"),
            ],
        },
        self::AgentRole::TargetCoder => self::FileMutationScope::Scoped {
            allowed_prefixes: ::std::vec![::std::string::String::from("modernized/<mod>/src/"),],
        },
    }
}

/// Return the list of mounted tools for an agent role.
pub fn mounted_tools(role: self::AgentRole) -> &'static [&'static ::core::primitive::str] {
    match role {
        self::AgentRole::RootOrchestrator => &["search", "read_hashline"],
        self::AgentRole::LegacyAnalyst => &["search", "read_hashline"],
        self::AgentRole::BinaryHistorian => &[
            "search",
            "read_hashline",
            "rea_scc_analysis",
            "run_scc_analysis",
            "rea_symbol_graph",
            "rea_dataflow_slice",
            "rea_decompile_function",
            "rea_xrefs",
            "rea_query_retained_view",
        ],
        self::AgentRole::BusinessRulesExtractor => &["search", "read_hashline", "rea_query_retained_view"],
        self::AgentRole::ArchitectureCritic => &["search", "read_hashline"],
        self::AgentRole::SecurityAuditor => &["search", "read_hashline"],
        self::AgentRole::BoundaryMediator => &["search", "read_hashline", "hashline_replace"],
        self::AgentRole::ModuleLead => &["search", "read_hashline"],
        self::AgentRole::TestEngineer => &["search", "read_hashline", "hashline_replace", "verify_runner"],
        self::AgentRole::TargetCoder => &["search", "read_hashline", "hashline_replace"],
    }
}

/// Check whether a tool name belongs to the REA reverse engineering toolset.
pub fn is_rea_tool(tool_name: &::core::primitive::str) -> ::core::primitive::bool {
    let has_rea_prefix = tool_name.starts_with("rea_");
    let is_scc = tool_name == "run_scc_analysis";
    has_rea_prefix || is_scc
}
