//! Code generation agent for the ARIADNE cluster.
//!
//! Exposes one Rig agent with the synthesis preamble wired, plus the domain
//! call that prompts it and extracts the candidate module source.

pub mod agent;

pub use self::agent::{
    codegen_system_prompt, ensure_top_level_pub, extract_code, fault_candidate,
    render_codegen_prompt, unwrap_outer_mod, HolonCodeGenAgent, HolonCodeGenAgentBuilder,
};
