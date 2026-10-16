//! Coding agent terminal user interface with hierarchical orchestrator.

mod app;
mod config;
mod tree;
mod ui;

#[::tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    let stdin = ::std::io::stdin();
    let is_term = ::std::io::IsTerminal::is_terminal(&stdin);
    let mut args = ::std::env::args();
    let has_dry_run = args.any(|a| a == "--dry-run" || a == "--smoke");

    let mut config = self::config::ModelConfig::default();
    if let ::core::result::Result::Ok(base_url) = ::std::env::var("ASHALLIANTS_BASE_URL") {
        config.base_url = base_url;
    }
    if let ::core::result::Result::Ok(model_id) = ::std::env::var("ASHALLIANTS_MODEL") {
        config.model_id = model_id;
    }

    let omp_bridge = ::bridge_omp::OmpCompletionModel::builder().model("omp-default".to_string()).build();
    let goose_bridge = ::bridge_goose::GooseCompletionModel::builder()
        .model("goose-default".to_string())
        .session_id("tui-session".to_string())
        .build();
    let codex_bridge = ::bridge_codex::CodexCompletionModel::builder().model("codex-default".to_string()).build();

    let omp_agent = omp_bridge.into_agent("You are the Omp RPC worker agent.");
    let goose_agent = goose_bridge.into_agent("You are the Goose ACP worker agent.");
    let codex_agent = codex_bridge.into_agent("You are the Codex protocol worker agent.");

    let rig_model = config.build_completion_model();
    let rig_agent = ::rig::agent::AgentBuilder::new(rig_model)
        .name("rig_native_agent")
        .preamble("You are a native Rig worker agent.")
        .build();

    let omp_worker = self::app::Worker::new("omp-rpc", omp_agent);
    let goose_worker = self::app::Worker::new("goose-acp", goose_agent);
    let codex_worker = self::app::Worker::new("codex-protocol", codex_agent);
    let rig_worker = self::app::Worker::new("rig-native", rig_agent);
    let workers = ::std::vec![omp_worker, goose_worker, codex_worker, rig_worker];
    let mut app = self::app::AppState::new(config, workers);
    if !is_term || has_dry_run {
        app.run_smoke().await;
        return;
    }
    app.run_interactive().await;
}
