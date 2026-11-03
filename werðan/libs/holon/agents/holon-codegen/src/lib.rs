//! Code generation agent for the ARIADNE cluster.
//!
//! Exposes one Rig agent with the synthesis preamble wired, plus the domain
//! call that prompts it and extracts the candidate module source.

pub mod agent;

pub use self::agent::{
    codegen_system_prompt, extract_code, fault_candidate, parse_synthesis_output,
    render_codegen_prompt, CodeSynthesisOutput, HolonCodeGenAgent, HolonCodeGenAgentBuilder,
};
