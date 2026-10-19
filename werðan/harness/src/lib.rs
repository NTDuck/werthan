//! Consolidated harness tooling: hashline addressing, search, compaction,
//! stream rules, sandboxing, configuration, and worker pooling.

/// Hashline content addressing and anchor replacement (absorbed hashline crate).
pub mod hashline;

/// Line search and LSP diagnostic formatting (absorbed lsp-tools crate).
pub mod lsp_tools;

/// Snapshot compaction and immutable cache stub formatting (absorbed snapcompact crate).
pub mod snapcompact;

/// Stream interception and dormant rule enforcement (absorbed stream-rules crate).
pub mod stream_rules;

/// In-process filesystem sandbox enforcement (absorbed sandbox crate).
pub mod sandbox;

/// Modernization configuration validation, scaffolding, and warden rendering (absorbed config crate).
pub mod config;

/// Headless worker pool for dynamic shards (absorbed worker-pool crate).
pub mod worker_pool;

/// Native rig tool implementations over harness primitives.
#[cfg(feature = "rig")]
pub mod tools;
