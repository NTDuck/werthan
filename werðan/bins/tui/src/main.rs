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
    if !is_term || has_dry_run {
        let env_res = ::tracing_subscriber::EnvFilter::try_from_default_env();
        let filter = match env_res {
            ::core::result::Result::Ok(f) => f,
            ::core::result::Result::Err(_) => ::tracing_subscriber::EnvFilter::new("info"),
        };
        let subscriber = ::tracing_subscriber::fmt().with_env_filter(filter);
        let _ = subscriber.try_init();
    }

    let mut config = self::config::ModelConfig::default();
    if let ::core::result::Result::Ok(base_url) = ::std::env::var("BASE_URL") {
        config.base_url = base_url;
    } else if let ::core::result::Result::Ok(base_url) = ::std::env::var("ASHALLIANTS_BASE_URL") {
        config.base_url = base_url;
    }

    if let ::core::result::Result::Ok(model_id) = ::std::env::var("MODEL_ID") {
        config.model_id = model_id;
    } else if let ::core::result::Result::Ok(model_id) = ::std::env::var("ASHALLIANTS_MODEL") {
        config.model_id = model_id;
    }

    let omp_bridge = ::bridge_omp::OmpCompletionModel::builder()
        .model(config.model_id.clone())
        .base_url(config.base_url.clone())
        .build();

    let goose_bridge = ::bridge_goose::GooseCompletionModel::builder()
        .model(config.model_id.clone())
        .base_url(config.base_url.clone())
        .session_id("tui-session".to_string())
        .build();

    let codex_bridge = ::bridge_codex::CodexCompletionModel::builder()
        .model(config.model_id.clone())
        .base_url(config.base_url.clone())
        .build();

    let omp_agent = omp_bridge.into_agent(
        "You are Council Member Omp, specializing in software architecture, system boundaries, and design coherence.",
    );

    let goose_agent = goose_bridge.into_agent(
        "You are Council Member Goose, specializing in runtime protocols, communication flows, and operational \
         execution.",
    );

    let codex_agent = codex_bridge.into_agent(
        "You are Council Member Codex, specializing in concrete implementation, algorithms, and data structures.",
    );

    let rig_model = config.build_completion_model();
    let rig_agent = ::rig::agent::AgentBuilder::new(rig_model)
        .name("rig_native_agent")
        .preamble("You are Council Member Rig, specializing in driver integration, verification, and contract safety.")
        .max_tokens(1024)
        .build();

    let orch_agent = self::tree::build_orchestrator(config.build_completion_model());

    let council_specs = [
        ("omp-rpc", "Architecture"),
        ("goose-acp", "Protocols"),
        ("codex-protocol", "Implementation"),
        ("rig-native", "Verification"),
    ];

    let members = council_specs
        .iter()
        .map(|&(name, role)| {
            self::tree::CouncilNode::builder()
                .name(name)
                .role(role)
                .build()
        })
        .collect::<::std::vec::Vec<_>>();

    let dag = self::tree::CouncilDag::builder()
        .orchestrator(orch_agent)
        .members(members)
        .build();

    let agent_specs = [
        ("omp-rpc", omp_agent),
        ("goose-acp", goose_agent),
        ("codex-protocol", codex_agent),
        ("rig-native", rig_agent),
    ];

    let agents = agent_specs
        .into_iter()
        .map(|(name, agent)| {
            let output = self::app::create_slot_output();
            self::app::AgentSlot::builder()
                .name(name)
                .agent(agent)
                .output(output)
                .build()
        })
        .collect::<::std::vec::Vec<_>>();

    let orchestrator_model = config.build_completion_model();
    let mut app = self::app::AppState::builder()
        .orchestrator_model(orchestrator_model)
        .config(config)
        .agents(agents)
        .dag(dag)
        .build();

    if !is_term || has_dry_run {
        app.run_smoke().await;
        return;
    }
    app.run_interactive().await;
}
