//! Global repository blackboard with macro state vector.
//! Implements METHODOLOGY.md Definition 1 for the global blackboard G and the state vector Sigma(G).
//! The state vector Sigma(G) holds the macro state, per-module micro states, the deep history map
//! Theta_H*, and the FIFO event queue.

pub mod artifacts;
pub mod contracts;
pub mod dag;
pub mod inventory;
pub mod path_resolver;
pub mod rules;
pub mod scc;
pub mod state_vector;
pub mod work_manifest;

pub use self::artifacts::{ArtifactRegistry, ModuleArtifact};
pub use self::contracts::ContractRegistry;
pub use self::dag::CondensedDag;
pub use ::global_fsm::{
    MacroState, MacroStateError, MacroStateMachine, MacroStatechart, S0IntentPreflight, S1Assess,
    S2MapDag, S3ExtractRules, S4BriefContracts, S5IterativeDag, S6SystemVerify, Terminated,
};
pub use self::inventory::{DialectProfile, HarnessConfig, RepositoryInventory};
pub use self::path_resolver::{default_blackboard_dir, resolve_component_path};
pub use self::rules::RuleRegistry;
pub use self::state_vector::{GlobalStateVector, MacroEvent, ModuleMicroState};
pub use self::work_manifest::{UnresolvedCrossLink, WorkManifest};

/// Global blackboard G per METHODOLOGY.md Definition 1.
/// Holds the inventory I_inv, the condensed DAG G=(V,E), rule cards R_rules,
/// edge contracts C_contracts, hardened artifacts A_artifacts, and the state vector Sigma(G).
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct GlobalBlackboard {
    /// Repository file inventory I_inv.
    pub inventory: self::inventory::RepositoryInventory,
    /// Condensed dependency DAG G = (V, E).
    pub dag: self::dag::CondensedDag,
    /// Verified business rule cards per vertex R_rules.
    pub rules: self::rules::RuleRegistry,
    /// Interface contracts per dependency edge keyed by parent and child pair C_contracts.
    pub contracts: self::contracts::ContractRegistry,
    /// Hardened target artifacts per vertex A_artifacts.
    pub artifacts: self::artifacts::ArtifactRegistry,
    /// Work manifests written by the Scatter stage per phase (Algorithm 1 S_m.1).
    pub work_manifests: ::std::vec::Vec<self::work_manifest::WorkManifest>,
    /// Unresolved cross-shard link tags awaiting the Reduce linker (Algorithm 1 S_m.3).
    pub cross_links: ::std::vec::Vec<self::work_manifest::UnresolvedCrossLink>,
    /// Transactionally persisted state configuration Sigma(G).
    pub state: self::state_vector::GlobalStateVector,
}
#[::bon::bon]
impl self::GlobalBlackboard {
    /// Construct a global blackboard composed from individual components using a bon builder.
    #[builder]
    pub fn builder(
        file_path: ::core::option::Option<::std::path::PathBuf>,
        output_dir: ::core::option::Option<::std::path::PathBuf>,
        #[builder(default = "default".to_string())]
        hash: ::std::string::String,
        #[builder(default = false)]
        create_if_missing: ::core::primitive::bool,
        inventory: ::core::option::Option<self::inventory::RepositoryInventory>,
        dag: ::core::option::Option<self::dag::CondensedDag>,
        rules: ::core::option::Option<self::rules::RuleRegistry>,
        contracts: ::core::option::Option<self::contracts::ContractRegistry>,
        artifacts: ::core::option::Option<self::artifacts::ArtifactRegistry>,
        state: ::core::option::Option<self::state_vector::GlobalStateVector>,
    ) -> ::core::result::Result<Self, ::std::io::Error> {
        let inventory = match inventory {
            ::core::option::Option::Some(component) => component,
            ::core::option::Option::None => {
                self::inventory::RepositoryInventory::builder()
                    .maybe_file_path(file_path.clone())
                    .maybe_output_dir(output_dir.clone())
                    .hash(hash.clone())
                    .create_if_missing(create_if_missing)
                    .build()?
            }
        };
        let dag = match dag {
            ::core::option::Option::Some(component) => component,
            ::core::option::Option::None => {
                self::dag::CondensedDag::builder()
                    .maybe_file_path(file_path.clone())
                    .maybe_output_dir(output_dir.clone())
                    .hash(hash.clone())
                    .create_if_missing(create_if_missing)
                    .build()?
            }
        };
        let rules = match rules {
            ::core::option::Option::Some(component) => component,
            ::core::option::Option::None => {
                self::rules::RuleRegistry::builder()
                    .maybe_file_path(file_path.clone())
                    .maybe_output_dir(output_dir.clone())
                    .hash(hash.clone())
                    .create_if_missing(create_if_missing)
                    .build()?
            }
        };
        let contracts = match contracts {
            ::core::option::Option::Some(component) => component,
            ::core::option::Option::None => {
                self::contracts::ContractRegistry::builder()
                    .maybe_file_path(file_path.clone())
                    .maybe_output_dir(output_dir.clone())
                    .hash(hash.clone())
                    .create_if_missing(create_if_missing)
                    .build()?
            }
        };
        let artifacts = match artifacts {
            ::core::option::Option::Some(component) => component,
            ::core::option::Option::None => {
                self::artifacts::ArtifactRegistry::builder()
                    .maybe_file_path(file_path.clone())
                    .maybe_output_dir(output_dir.clone())
                    .hash(hash.clone())
                    .create_if_missing(create_if_missing)
                    .build()?
            }
        };
        let state = match state {
            ::core::option::Option::Some(component) => component,
            ::core::option::Option::None => {
                self::state_vector::GlobalStateVector::builder()
                    .maybe_file_path(file_path.clone())
                    .maybe_output_dir(output_dir.clone())
                    .hash(hash.clone())
                    .create_if_missing(create_if_missing)
                    .build()?
            }
        };
        ::core::result::Result::Ok(Self {
            inventory,
            dag,
            rules,
            contracts,
            artifacts,
            work_manifests: ::std::vec::Vec::new(),
            cross_links: ::std::vec::Vec::new(),
            state,
        })
    }

    /// Persist all composed blackboard components to their resolved paths.
    pub fn save(
        &self,
        file_path: ::core::option::Option<&::std::path::Path>,
        output_dir: ::core::option::Option<&::std::path::Path>,
        hash: &::core::primitive::str,
    ) -> ::core::result::Result<(), ::std::io::Error> {
        self.inventory.save(file_path, output_dir, hash)?;
        self.dag.save(file_path, output_dir, hash)?;
        self.rules.save(file_path, output_dir, hash)?;
        self.contracts.save(file_path, output_dir, hash)?;
        self.artifacts.save(file_path, output_dir, hash)?;
        self.state.save(file_path, output_dir, hash)?;
        ::core::result::Result::Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_operations() {
        let mut inventory = self::inventory::RepositoryInventory::new();
        ::std::assert!(inventory.is_empty());
        inventory.push("src/alpha.c");
        inventory.add_file("src/beta.c");
        ::std::assert_eq!(inventory.len(), 2);
        ::std::assert!(inventory.contains("src/alpha.c"));
        ::std::assert!(!inventory.contains("src/gamma.c"));
        ::std::assert_eq!(inventory.files(), &["src/alpha.c", "src/beta.c"]);
        let collected = inventory.iter().cloned().collect::<::std::vec::Vec<::std::string::String>>();
        ::std::assert_eq!(collected.len(), 2);
        let deref_slice: &[::std::string::String] = &inventory;
        ::std::assert_eq!(deref_slice.len(), 2);
    }

    #[test]
    fn test_dag_operations() {
        let mut dag = self::dag::CondensedDag::new();
        dag.vertices = ::std::vec!["alpha".to_string(), "beta".to_string()];
        dag.edges = ::std::vec![("alpha".to_string(), "beta".to_string())];
        ::std::assert!(dag.is_acyclic());
        let parents = dag.parents_of("beta");
        ::std::assert_eq!(parents.len(), 1);
        ::std::assert_eq!(parents[0], "alpha");
        let children = dag.children_of("alpha");
        ::std::assert_eq!(children.len(), 1);
        ::std::assert_eq!(children[0], "beta");
    }

    #[test]
    fn test_rules_operations() {
        let mut registry = self::rules::RuleRegistry::new();
        ::std::assert!(registry.is_empty());
        let card = ::holon_blackboard::BusinessRuleCard {
            rule_id: "rule_1".to_string(),
            given: "given clause".to_string(),
            when: "when clause".to_string(),
            then: "then clause".to_string(),
            citation: "src/alpha.c:10".to_string(),
        };
        registry.insert("alpha", ::std::vec![card]);
        ::std::assert_eq!(registry.len(), 1);
        ::std::assert!(registry.contains("alpha"));
        let lookup = registry.get("alpha");
        ::std::assert!(lookup.is_some());
        let cards_count = registry.all_rules().count();
        ::std::assert_eq!(cards_count, 1);
    }

    #[test]
    fn test_contracts_operations() {
        let mut registry = self::contracts::ContractRegistry::new();
        ::std::assert!(registry.is_empty());
        let edge = ("parent".to_string(), "child".to_string());
        let contract = ::holon_blackboard::EdgeContract {
            parent_id: "parent".to_string(),
            interface_spec: "spec".to_string(),
            preconditions: ::std::vec::Vec::new(),
        };
        registry.insert(edge.clone(), contract);
        ::std::assert_eq!(registry.len(), 1);
        ::std::assert!(registry.contains_edge(&edge));
        let child_contracts = registry.contracts_for_child("child");
        ::std::assert_eq!(child_contracts.len(), 1);
        let parent_contracts = registry.contracts_from_parent("parent");
        ::std::assert_eq!(parent_contracts.len(), 1);
    }

    #[test]
    fn test_artifacts_operations() {
        let mut registry = self::artifacts::ArtifactRegistry::new();
        ::std::assert!(registry.is_empty());
        let artifact = self::artifacts::ModuleArtifact {
            target_code: "pub fn alpha() {}".to_string(),
            test_suite: ::std::vec!["test_alpha".to_string()],
        };
        registry.commit("alpha", artifact);
        ::std::assert_eq!(registry.len(), 1);
        ::std::assert!(registry.is_hardened("alpha"));
        ::std::assert!(!registry.is_hardened("beta"));
        let alpha_slice = ::std::vec!["alpha".to_string()];
        ::std::assert!(registry.all_hardened(&alpha_slice));
        let beta_slice = ::std::vec!["beta".to_string()];
        ::std::assert!(!registry.all_hardened(&beta_slice));
    }

    #[test]
    fn test_state_vector_operations() {
        let mut state = self::state_vector::GlobalStateVector::initialize();
        ::std::assert_eq!(state.macro_state, ::global_fsm::MacroState::S0IntentPreflight);
        let first_event = state.pop_event();
        ::std::assert!(::core::matches!(first_event, ::core::option::Option::Some(self::state_vector::MacroEvent::Start)));
        state.module_states.insert("alpha".to_string(), self::state_vector::ModuleMicroState::M4Hardened);
        state.save_deep_history();
        state.module_states.insert("alpha".to_string(), self::state_vector::ModuleMicroState::M5Invalidated);
        let history_lookup = state.history_state_of("alpha");
        ::std::assert_eq!(history_lookup, ::core::option::Option::Some(self::state_vector::ModuleMicroState::M4Hardened));
        state.restore_deep_history();
        ::std::assert_eq!(
            state.module_states.get("alpha"),
            ::core::option::Option::Some(&self::state_vector::ModuleMicroState::M4Hardened)
        );
    }

    #[test]
    fn test_fsm_typestate_transitions() {
        let statechart = ::global_fsm::MacroStatechart::new();
        ::std::assert_eq!(statechart.state(), ::global_fsm::MacroState::S0IntentPreflight);
        ::std::assert!(statechart.history().is_empty());

        let step_s1 = statechart.advance().unwrap();
        ::std::assert_eq!(step_s1.state(), ::global_fsm::MacroState::S1Assess);
        let step_s2 = step_s1.advance().unwrap();
        ::std::assert_eq!(step_s2.state(), ::global_fsm::MacroState::S2MapDag);
        let step_s3 = step_s2.advance().unwrap();
        ::std::assert_eq!(step_s3.state(), ::global_fsm::MacroState::S3ExtractRules);
        let step_s4 = step_s3.advance().unwrap();
        ::std::assert_eq!(step_s4.state(), ::global_fsm::MacroState::S4BriefContracts);
        let step_s5 = step_s4.advance().unwrap();
        ::std::assert_eq!(step_s5.state(), ::global_fsm::MacroState::S5IterativeDag);

        let regressed_s2 = step_s5.clone().regress_to_s2().unwrap();
        ::std::assert_eq!(regressed_s2.state(), ::global_fsm::MacroState::S2MapDag);
        ::std::assert_eq!(
            regressed_s2.history(),
            &[
                ::global_fsm::MacroState::S0IntentPreflight,
                ::global_fsm::MacroState::S1Assess,
                ::global_fsm::MacroState::S2MapDag,
                ::global_fsm::MacroState::S3ExtractRules,
                ::global_fsm::MacroState::S4BriefContracts,
                ::global_fsm::MacroState::S5IterativeDag,
            ]
        );

        let step_s6 = step_s5.advance().unwrap();
        ::std::assert_eq!(step_s6.state(), ::global_fsm::MacroState::S6SystemVerify);
        let terminated = step_s6.advance().unwrap();
        ::std::assert_eq!(terminated.state(), ::global_fsm::MacroState::Terminated);
        let already_terminated = terminated.advance();
        ::std::assert_eq!(already_terminated, ::core::result::Result::Err(::global_fsm::MacroStateError::AlreadyTerminated));
    }

    #[test]
    fn test_blackboard_default_and_serde() {
        let board = self::GlobalBlackboard::default();
        ::std::assert!(board.inventory.is_empty());
        ::std::assert!(board.dag.vertices.is_empty());
        ::std::assert_eq!(board.state.macro_state, ::global_fsm::MacroState::S0IntentPreflight);

        let encoded = ::serde_json::to_string(&board).unwrap();
        let decoded = ::serde_json::from_str::<self::GlobalBlackboard>(&encoded).unwrap();
        ::std::assert_eq!(decoded.state.macro_state, board.state.macro_state);
    }

    #[test]
    fn test_default_blackboard_dir_structure() {
        let hash_key = "test_hash_12345";
        let default_dir = self::path_resolver::default_blackboard_dir(hash_key);
        let dir_string = default_dir.to_string_lossy();
        ::std::assert!(dir_string.contains(hash_key));
        ::std::assert!(dir_string.contains("werðan"));
    }

    #[test]
    fn test_component_builders_and_composition() {
        let sample_files = ::std::vec!["src/foo.rs".to_string(), "src/bar.rs".to_string()];
        let inventory = self::inventory::RepositoryInventory::builder()
            .files(sample_files.clone())
            .build()
            .expect("inventory builds");
        ::std::assert_eq!(inventory.len(), 2);

        let sample_vertices = ::std::vec!["foo".to_string(), "bar".to_string()];
        let sample_edges = ::std::vec![("foo".to_string(), "bar".to_string())];
        let dag = self::dag::CondensedDag::builder()
            .vertices(sample_vertices)
            .edges(sample_edges)
            .build()
            .expect("dag builds");
        ::std::assert!(dag.is_acyclic());

        let rules = self::rules::RuleRegistry::builder()
            .build()
            .expect("rules builds");
        let contracts = self::contracts::ContractRegistry::builder()
            .build()
            .expect("contracts builds");
        let artifacts = self::artifacts::ArtifactRegistry::builder()
            .build()
            .expect("artifacts builds");
        let state = self::state_vector::GlobalStateVector::builder()
            .build()
            .expect("state builds");

        let blackboard = self::GlobalBlackboard::builder()
            .inventory(inventory)
            .dag(dag)
            .rules(rules)
            .contracts(contracts)
            .artifacts(artifacts)
            .state(state)
            .build()
            .expect("blackboard composes");

        ::std::assert_eq!(blackboard.inventory.len(), 2);
        ::std::assert_eq!(blackboard.dag.vertices.len(), 2);
        ::std::assert!(blackboard.rules.is_empty());
    }

    #[test]
    fn test_blackboard_save_and_reload_from_output_dir() {
        let pid_text = ::std::process::id();
        let temp_dir_name = ::std::format!("werthan-blackboard-test-{pid_text}");
        let temp_output_dir = ::std::env::temp_dir().join(temp_dir_name);
        let _ = ::std::fs::create_dir_all(&temp_output_dir);

        let sample_files = ::std::vec!["src/main.rs".to_string()];
        let inventory = self::inventory::RepositoryInventory::builder()
            .output_dir(temp_output_dir.clone())
            .files(sample_files)
            .build()
            .expect("inventory builds with output_dir");

        let blackboard = self::GlobalBlackboard::builder()
            .output_dir(temp_output_dir.clone())
            .inventory(inventory)
            .build()
            .expect("blackboard builds with output_dir");

        blackboard.save(::core::option::Option::None, ::core::option::Option::Some(&temp_output_dir), "test_run")
            .expect("blackboard saves");

        ::std::assert!(temp_output_dir.join("inventory.json").is_file());
        ::std::assert!(temp_output_dir.join("dag.json").is_file());
        ::std::assert!(temp_output_dir.join("rules.json").is_file());
        ::std::assert!(temp_output_dir.join("contracts.json").is_file());
        ::std::assert!(temp_output_dir.join("artifacts.json").is_file());
        ::std::assert!(temp_output_dir.join("state.json").is_file());

        let reloaded_blackboard = self::GlobalBlackboard::builder()
            .output_dir(temp_output_dir.clone())
            .build()
            .expect("blackboard reloads from output_dir");

        ::std::assert_eq!(reloaded_blackboard.inventory.len(), 1);
        ::std::assert_eq!(reloaded_blackboard.inventory.files(), &["src/main.rs"]);

        let _ = ::std::fs::remove_dir_all(&temp_output_dir);
    }
}
