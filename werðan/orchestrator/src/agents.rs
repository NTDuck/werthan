//! Six specialist agent factories wired through rig AgentBuilder with role preambles.

use ::harness::hashline::apply_hashline_replace;
use ::harness::hashline::render_hashlined_view;
use ::harness::lsp_tools::search_files_in_process;
use ::harness::tools::HashlineReplaceTool;
use ::harness::tools::ReadHashlineTool;
use ::harness::tools::SearchTool;
use ::rig::agent::Agent;
use ::rig::agent::AgentBuilder;
use ::rig::providers::openai;

/// Preamble keeping each specialist under seven hundred tokens.
pub fn role_preamble(role: &::core::primitive::str, mandate: &::core::primitive::str) -> ::std::string::String {
    let rules =
        "Rules: respond concisely; cite file:line evidence; never fabricate output; escalate blockers immediately.";
    ::std::format!("You are the {role} specialist in the werthan modernization council.\nMandate: {mandate}\n{rules}")
}

/// Completion client from env; usable without OPENAI_API_KEY by defaulting to EMPTY.
pub fn completion_client() -> ::core::option::Option<openai::Client> {
    let key = match ::std::env::var("OPENAI_API_KEY") {
        ::core::result::Result::Ok(k) if !k.is_empty() => k,
        _ => "EMPTY".to_string(),
    };
    let base = match ::std::env::var("WERTHAN_BASE_URL") {
        ::core::result::Result::Ok(b) if !b.is_empty() => b,
        _ => "http://127.0.0.1:8092/v1".to_string(),
    };
    ::core::option::Option::Some(openai::Client::from_url(&key, &base))
}

/// Rules extractor agent built for dynamic shards.
pub fn build_shard_agent(model: &::core::primitive::str) -> ::core::option::Option<Agent<openai::CompletionModel>> {
    completion_client().map(|client| {
        let completion_model = client.completion_model(model);
        AgentBuilder::new(completion_model)
            .preamble(&role_preamble(
                "rules_extractor",
                "Treat legacy code as oracle; emit testable Given/When/Then rule cards ranked P0-P2.",
            ))
            .build()
    })
}

/// Helper to execute a single prompt against a built agent.
pub async fn complete_once(
    agent: &Agent<openai::CompletionModel>,
    prompt: &::core::primitive::str,
) -> ::core::result::Result<::std::string::String, ::rig::completion::PromptError> {
    use ::rig::completion::Prompt;
    agent.prompt(prompt).await
}

/// Simulated completion response when no provider is configured.
pub fn simulated_reply(role: &::core::primitive::str) -> ::std::string::String {
    ::std::format!("[{role}] simulated completion: provider key not set; acting read-only per mandate.")
}

/// Orchestrator agent factory (read-only tools).
pub fn build_orchestrator_agent() -> ::core::option::Option<AgentBuilder<openai::CompletionModel>> {
    completion_client().map(|client| {
        let model =
            client.completion_model(&::std::env::var("WERTHAN_MODEL").unwrap_or_else(|_| "qwen3.8-27b".to_string()));
        AgentBuilder::new(model)
            .preamble(&role_preamble(
                "orchestrator",
                "Run the FSM lifecycle, dispatch shards, collect receipts, and gate human approvals.",
            ))
            .tool(SearchTool)
    })
}

/// Legacy analyst agent factory (read-only tools).
pub fn build_legacy_analyst_agent() -> ::core::option::Option<AgentBuilder<openai::CompletionModel>> {
    completion_client().map(|client| {
        let model =
            client.completion_model(&::std::env::var("WERTHAN_MODEL").unwrap_or_else(|_| "qwen3.8-27b".to_string()));
        AgentBuilder::new(model)
            .preamble(&role_preamble(
                "legacy_analyst",
                "Cluster legacy domains, rank technical debt, map call graphs and copybooks.",
            ))
            .tool(SearchTool)
            .tool(ReadHashlineTool)
    })
}

/// Business rules extractor agent factory (read-only tools).
pub fn build_rules_extractor_agent() -> ::core::option::Option<AgentBuilder<openai::CompletionModel>> {
    completion_client().map(|client| {
        let model =
            client.completion_model(&::std::env::var("WERTHAN_MODEL").unwrap_or_else(|_| "qwen3.8-27b".to_string()));
        AgentBuilder::new(model)
            .preamble(&role_preamble(
                "rules_extractor",
                "Treat legacy code as oracle; emit testable Given/When/Then rule cards ranked P0-P2.",
            ))
            .tool(SearchTool)
            .tool(ReadHashlineTool)
    })
}

/// Translator and test agent factory (hashline editing plus verify runners).
pub fn build_translator_test_agent() -> ::core::option::Option<AgentBuilder<openai::CompletionModel>> {
    completion_client().map(|client| {
        let model = client.completion_model(
            &::std::env::var("WERTHAN_MODEL").unwrap_or_else(|_| "qwen3.8-27b".to_string()),
        );
        AgentBuilder::new(model)
            .preamble(&role_preamble(
                "translator_test",
                "Translate legacy slices to the target stack; pin behavior with characterization tests before rewrites.",
            ))
            .tool(HashlineReplaceTool)
            .tool(ReadHashlineTool)
            .tool(SearchTool)
    })
}

/// Architecture critic agent factory (read-only tools).
pub fn build_architecture_critic_agent() -> ::core::option::Option<AgentBuilder<openai::CompletionModel>> {
    completion_client().map(|client| {
        let model =
            client.completion_model(&::std::env::var("WERTHAN_MODEL").unwrap_or_else(|_| "qwen3.8-27b".to_string()));
        AgentBuilder::new(model)
            .preamble(&role_preamble(
                "architecture_critic",
                "Audit designs for over-engineering and unstated NFR violations; classify Blocker/High/Medium/Nit.",
            ))
            .tool(SearchTool)
    })
}

/// Security auditor agent factory (read-only tools).
pub fn build_security_auditor_agent() -> ::core::option::Option<AgentBuilder<openai::CompletionModel>> {
    completion_client().map(|client| {
        let model =
            client.completion_model(&::std::env::var("WERTHAN_MODEL").unwrap_or_else(|_| "qwen3.8-27b".to_string()));
        AgentBuilder::new(model)
            .preamble(&role_preamble(
                "security_auditor",
                "Scan for cross-stack translation regressions with CWE tags; mask secrets as Pr0d****.",
            ))
            .tool(SearchTool)
    })
}

/// Reference call-throughs kept for direct harness use without an agent loop.
pub fn direct_hashline_view(
    source: &::core::primitive::str,
    start_line: ::core::primitive::usize,
    max_lines: ::core::primitive::usize,
) -> ::std::string::String {
    render_hashlined_view(source, start_line, max_lines)
}

pub fn direct_hashline_replace(
    original: &::core::primitive::str,
    start_anchor: &::core::primitive::str,
    end_anchor: &::core::primitive::str,
    replacement: &::core::primitive::str,
) -> ::core::result::Result<::std::string::String, ::std::string::String> {
    apply_hashline_replace(original, start_anchor, end_anchor, replacement)
}

pub fn direct_search(
    root: &::std::path::Path,
    query: &::core::primitive::str,
) -> ::core::result::Result<::std::vec::Vec<::harness::lsp_tools::SearchMatch>, ::std::string::String> {
    search_files_in_process(root, query)
}
