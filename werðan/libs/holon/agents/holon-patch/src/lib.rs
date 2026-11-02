//! Surgical code repair agent for the ARIADNE cluster.
//!
//! Exposes the Rig agent with the repair preamble wired, plus domain
//! functions to prompt the agent and extract the repaired module source.

pub mod agent;

pub use self::agent::{
    compact_diagnostic_trace, ensure_top_level_pub, extract_code, fault_candidate,
    patch_system_prompt, render_repair_prompt, truncate_diagnostics, unwrap_outer_mod,
    HolonPatchAgent, HolonPatchAgentBuilder, MAX_CHAR_BUDGET, MAX_DIAGNOSTIC_LINES,
};
