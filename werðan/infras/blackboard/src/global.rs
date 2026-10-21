//! Global Repository Blackboard G and append-only transaction journal.
//! Formalized per §2 Definition 1 in `METHODOLOGY.md`.

use ::redb::ReadableDatabase;
use ::redb::ReadableTable;

/// Citation observation tag for business rules.
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
pub enum RuleTag {
    /// Empirically observed in legacy code or binary trace.
    Observed,
    /// Inferred or deduced by analysis heuristics.
    Inferred,
}

/// Verified business rule card with Given/When/Then clauses and file:line citations.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct RuleCard {
    /// Unique rule identifier.
    #[builder(into)]
    pub id: ::std::string::String,
    /// Rule title or domain requirement.
    #[builder(into)]
    pub title: ::std::string::String,
    /// Given condition.
    #[builder(into)]
    pub given: ::std::string::String,
    /// When trigger or action.
    #[builder(into)]
    pub when: ::std::string::String,
    /// Then expected invariant.
    #[builder(into)]
    pub then: ::std::string::String,
    /// Legacy file:line citation anchor.
    #[builder(into)]
    pub citation: ::std::string::String,
    /// Evidence status.
    pub tag: self::RuleTag,
}

/// Target interface contract across a dependency edge (v_p, v_i).
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct InterfaceSpec {
    /// Upstream provider vertex identifier.
    #[builder(into)]
    pub provider_vertex: ::std::string::String,
    /// Downstream consumer vertex identifier.
    #[builder(into)]
    pub consumer_vertex: ::std::string::String,
    /// Target language type signature or API header stub.
    #[builder(into)]
    pub interface_stub: ::std::string::String,
    /// Preconditions and behavioral contracts.
    #[serde(default)]
    #[builder(default)]
    pub preconditions: ::std::vec::Vec<::std::string::String>,
}

/// Hardened target module artifacts stored in A_artifacts.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct HardenedArtifact {
    /// Target module vertex identifier.
    #[builder(into)]
    pub vertex_id: ::std::string::String,
    /// Hardened target language source code.
    #[builder(into)]
    pub target_code: ::std::string::String,
    /// Passing characterization test suite source code.
    #[builder(into)]
    pub characterization_tests: ::std::string::String,
    /// Final composite reward achieved (must be 1.0 for M4_Hardened).
    pub reward: ::core::primitive::f64,
    /// SHA-256 cryptographic commit stamp.
    #[builder(into)]
    pub hash_stamp: ::std::string::String,
}

/// File metadata record in repository inventory.
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct InventoryFile {
    /// Relative path within repository.
    #[builder(into)]
    pub path: ::std::string::String,
    /// Source language or dialect.
    #[builder(into)]
    pub language: ::std::string::String,
    /// Estimated lines of code.
    pub lines_of_code: ::core::primitive::usize,
    /// SHA-256 content digest.
    #[builder(into)]
    pub content_hash: ::std::string::String,
}

/// Repository file inventory, dialect profile, and build harness config (I_inv).
#[derive(
    ::bon::Builder,
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::default::Default,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct InventoryManifest {
    /// All discovered files in repository.
    #[serde(default)]
    #[builder(default)]
    pub files: ::std::collections::BTreeMap<::std::string::String, self::InventoryFile>,
    /// Dialect profiles identified.
    #[serde(default)]
    #[builder(default)]
    pub dialects: ::std::vec::Vec<::std::string::String>,
    /// Build and test harness command configuration.
    #[builder(into)]
    pub build_harness: ::std::string::String,
}

/// Condensed Dependency Directed Acyclic Graph G = (V, E).
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::default::Default,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct CondensedDAG {
    /// Vertices (atomic translation units or collapsed SCCs).
    pub vertices: ::std::collections::BTreeSet<::std::string::String>,
    /// Files mapped to each vertex: vertex_id -> set of relative file paths.
    pub vertex_files: ::std::collections::BTreeMap<::std::string::String, ::std::vec::Vec<::std::string::String>>,
    /// Directed edges: provider -> consumer (consumer depends on provider).
    pub edges: ::std::collections::BTreeSet<(::std::string::String, ::std::string::String)>,
}

impl self::CondensedDAG {
    /// Create a new empty condensed DAG.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a vertex with associated files.
    pub fn add_vertex(&mut self, vertex: &::core::primitive::str, files: ::std::vec::Vec<::std::string::String>) {
        self.vertices.insert(vertex.to_string());
        self.vertex_files.insert(vertex.to_string(), files);
    }

    /// Add a dependency edge: provider -> consumer.
    pub fn add_edge(&mut self, provider: &::core::primitive::str, consumer: &::core::primitive::str) {
        self.vertices.insert(provider.to_string());
        self.vertices.insert(consumer.to_string());
        self.edges.insert((provider.to_string(), consumer.to_string()));
    }

    /// Return parents that the specified vertex depends on.
    pub fn parents_of(&self, vertex: &::core::primitive::str) -> ::std::vec::Vec<::std::string::String> {
        self.edges
            .iter()
            .filter(|(_, consumer)| consumer == vertex)
            .map(|(provider, _)| provider.clone())
            .collect::<::std::vec::Vec<::std::string::String>>()
    }

    /// Return children that depend on the specified vertex.
    pub fn children_of(&self, vertex: &::core::primitive::str) -> ::std::vec::Vec<::std::string::String> {
        self.edges
            .iter()
            .filter(|(provider, _)| provider == vertex)
            .map(|(_, consumer)| consumer.clone())
            .collect::<::std::vec::Vec<::std::string::String>>()
    }
}

/// Transactional mutations committed to the append-only journal before event broadcast.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub enum JournalDelta {
    /// Set inventory manifest.
    SetInventory(self::InventoryManifest),
    /// Update condensed DAG.
    UpdateDAG(self::CondensedDAG),
    /// Add business rule card for a vertex.
    AddRuleCard {
        vertex_id: ::std::string::String,
        card: self::RuleCard,
    },
    /// Set edge contract interface spec.
    SetContract {
        edge_key: (::std::string::String, ::std::string::String),
        spec: self::InterfaceSpec,
    },
    /// Commit hardened module artifact.
    CommitArtifact {
        vertex_id: ::std::string::String,
        artifact: self::HardenedArtifact,
    },
    /// Macro-state transition.
    SetMacroState(::std::string::String),
    /// Module micro-state transition.
    SetModuleMicroState {
        vertex_id: ::std::string::String,
        micro_state: ::std::string::String,
    },
    /// Deep history snapshot saved.
    SaveDeepHistory {
        snapshot_id: ::std::string::String,
        states: ::std::collections::BTreeMap<::std::string::String, ::std::string::String>,
    },
}

const JOURNAL_TABLE: ::redb::TableDefinition<::core::primitive::u64, &[::core::primitive::u8]> =
    ::redb::TableDefinition::new("journal_deltas");

/// Redb-backed append-only transaction journal.
pub struct BlackboardJournal {
    db: ::redb::Database,
    next_seq: ::std::sync::atomic::AtomicU64,
}

impl BlackboardJournal {
    /// Open or create journal database at path.
    pub fn open<P: ::core::convert::AsRef<::std::path::Path>>(path: P) -> ::core::result::Result<Self, ::std::string::String> {
        let db = ::redb::Database::create(path).map_err(|e| ::std::format!("Failed to open redb journal: {e}"))?;
        let init_res: ::core::result::Result<(), ::std::string::String> = (|| {
            let tx = db.begin_write().map_err(|e| ::std::format!("Write tx failed: {e}"))?;
            tx.open_table(self::JOURNAL_TABLE).map_err(|e| ::std::format!("Open table failed: {e}"))?;
            tx.commit().map_err(|e| ::std::format!("Commit failed: {e}"))?;
            ::core::result::Result::Ok(())
        })();
        if let ::core::result::Result::Err(e) = init_res {
            return ::core::result::Result::Err(::std::format!("Failed to initialize journal table: {e}"));
        }

        let next_seq = {
            let read_tx = db.begin_read().map_err(|e| ::std::format!("Read tx failed: {e}"))?;
            let table = read_tx.open_table(self::JOURNAL_TABLE).map_err(|e| ::std::format!("Open table failed: {e}"))?;
            let last_entry = table.last().map_err(|e| ::std::format!("Table scan failed: {e}"))?;
            match last_entry {
                ::core::option::Option::Some((seq, _)) => seq.value() + 1,
                ::core::option::Option::None => 1,
            }
        };

        ::core::result::Result::Ok(Self {
            db,
            next_seq: ::std::sync::atomic::AtomicU64::new(next_seq),
        })
    }

    /// Append a serializable delta transaction to the journal.
    pub fn append(&self, delta: &self::JournalDelta) -> ::core::result::Result<::core::primitive::u64, ::std::string::String> {
        let seq = self.next_seq.fetch_add(1, ::std::sync::atomic::Ordering::SeqCst);
        let payload = ::serde_json::to_vec(delta).map_err(|e| ::std::format!("Serialization failed: {e}"))?;

        let write_tx = self.db.begin_write().map_err(|e| ::std::format!("Write tx failed: {e}"))?;
        {
            let mut table = write_tx.open_table(self::JOURNAL_TABLE).map_err(|e| ::std::format!("Open table failed: {e}"))?;
            table.insert(seq, payload.as_slice()).map_err(|e| ::std::format!("Insert failed: {e}"))?;
        }
        write_tx.commit().map_err(|e| ::std::format!("Commit failed: {e}"))?;

        ::core::result::Result::Ok(seq)
    }

    /// Replay all journal deltas in sequential order.
    pub fn replay(&self) -> ::core::result::Result<::std::vec::Vec<self::JournalDelta>, ::std::string::String> {
        let read_tx = self.db.begin_read().map_err(|e| ::std::format!("Read tx failed: {e}"))?;
        let table = read_tx.open_table(self::JOURNAL_TABLE).map_err(|e| ::std::format!("Open table failed: {e}"))?;
        let mut deltas = ::std::vec::Vec::new();
        let iter = table.iter().map_err(|e| ::std::format!("Iter failed: {e}"))?;
        for item in iter {
            let (_, val) = item.map_err(|e| ::std::format!("Item read failed: {e}"))?;
            let delta = ::serde_json::from_slice::<self::JournalDelta>(val.value())
                .map_err(|e| ::std::format!("Deserialization failed: {e}"))?;
            deltas.push(delta);
        }
        ::core::result::Result::Ok(deltas)
    }
}

/// In-memory state storage for the Global Blackboard tuple G.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default)]
pub struct GlobalBlackboardData {
    pub inventory: self::InventoryManifest,
    pub dag: self::CondensedDAG,
    pub rule_cards: ::std::collections::BTreeMap<::std::string::String, ::std::vec::Vec<self::RuleCard>>,
    pub contracts: ::std::collections::BTreeMap<(::std::string::String, ::std::string::String), self::InterfaceSpec>,
    pub artifacts: ::std::collections::BTreeMap<::std::string::String, self::HardenedArtifact>,
    pub macro_state: ::std::string::String,
    pub module_micro_states: ::std::collections::BTreeMap<::std::string::String, ::std::string::String>,
    pub deep_history: ::std::collections::BTreeMap<::std::string::String, ::std::collections::BTreeMap<::std::string::String, ::std::string::String>>,
}

/// Thread-safe Global Repository Blackboard coordinating repository-wide modernization state.
#[derive(::core::clone::Clone)]
pub struct GlobalBlackboard {
    data: ::std::sync::Arc<::tokio::sync::RwLock<self::GlobalBlackboardData>>,
    journal: ::std::sync::Arc<self::BlackboardJournal>,
}

impl self::GlobalBlackboard {
    /// Initialize with a persistent redb journal.
    pub fn open<P: ::core::convert::AsRef<::std::path::Path>>(journal_path: P) -> ::core::result::Result<Self, ::std::string::String> {
        let journal = self::BlackboardJournal::open(journal_path)?;
        let mut data = self::GlobalBlackboardData::default();
        data.macro_state = "S0_Intent".to_string();

        // Replay any existing deltas from the journal
        let deltas = journal.replay()?;
        for delta in deltas {
            Self::apply_delta(&mut data, delta);
        }

        ::core::result::Result::Ok(Self {
            data: ::std::sync::Arc::new(::tokio::sync::RwLock::new(data)),
            journal: ::std::sync::Arc::new(journal),
        })
    }

    fn apply_delta(data: &mut self::GlobalBlackboardData, delta: self::JournalDelta) {
        match delta {
            self::JournalDelta::SetInventory(inv) => data.inventory = inv,
            self::JournalDelta::UpdateDAG(dag) => data.dag = dag,
            self::JournalDelta::AddRuleCard { vertex_id, card } => {
                data.rule_cards.entry(vertex_id).or_default().push(card);
            },
            self::JournalDelta::SetContract { edge_key, spec } => {
                data.contracts.insert(edge_key, spec);
            },
            self::JournalDelta::CommitArtifact { vertex_id, artifact } => {
                data.artifacts.insert(vertex_id, artifact);
            },
            self::JournalDelta::SetMacroState(st) => data.macro_state = st,
            self::JournalDelta::SetModuleMicroState { vertex_id, micro_state } => {
                data.module_micro_states.insert(vertex_id, micro_state);
            },
            self::JournalDelta::SaveDeepHistory { snapshot_id, states } => {
                data.deep_history.insert(snapshot_id, states);
            },
        }
    }

    /// Commit a delta transaction: writes to journal first, then updates in-memory state.
    pub async fn commit_delta(&self, delta: self::JournalDelta) -> ::core::result::Result<::core::primitive::u64, ::std::string::String> {
        let seq = self.journal.append(&delta)?;
        let mut guard = self.data.write().await;
        Self::apply_delta(&mut guard, delta);
        ::core::result::Result::Ok(seq)
    }

    /// Access snapshot of in-memory data.
    pub async fn read_data(&self) -> self::GlobalBlackboardData {
        self.data.read().await.clone()
    }
}
