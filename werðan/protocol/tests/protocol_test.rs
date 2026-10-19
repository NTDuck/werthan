//! Unit and property tests for protocol.

#[test]
fn test_protocol_types_and_serialization() {
    let config = ::protocol::ModernizeConfig::default();
    ::std::assert_eq!(config.campaign_id, "default-campaign");
    ::std::assert_eq!(config.execution_mode, ::protocol::ExecutionMode::Hitl);

    let json_res = ::serde_json::to_string(&config);
    ::std::assert!(json_res.is_ok());

    let role = ::protocol::PaneRole::Orchestrator;
    ::std::assert_eq!(::std::format!("{role}"), "orchestrator");

    let cmd = ::protocol::DaemonToPaneCmd::AbortCurrentTurn;
    let serialized_cmd = ::serde_json::to_string(&cmd).unwrap();
    ::std::assert!(serialized_cmd.contains("abort_current_turn"));

    let event = ::protocol::PaneToDaemonEvent::RegisterPane {
        role: ::protocol::PaneRole::LegacyAnalyst,
        pid: 1234,
    };
    let serialized_event = ::serde_json::to_string(&event).unwrap();
    ::std::assert!(serialized_event.contains("register_pane"));
}

#[test]
fn test_magic_keywords_handling() {
    ::std::assert_eq!(::protocol::MAGIC_KEYWORDS, "orchestrate ultrathink workflowz");

    let empty = ::protocol::append_magic_keywords("");
    ::std::assert_eq!(empty, "orchestrate ultrathink workflowz");

    let whitespace = ::protocol::append_magic_keywords("   ");
    ::std::assert_eq!(whitespace, "orchestrate ultrathink workflowz");

    let regular = ::protocol::append_magic_keywords("Modernize legacy code");
    ::std::assert_eq!(regular, "Modernize legacy code\n\norchestrate ultrathink workflowz");

    let already_has = ::protocol::append_magic_keywords("Modernize legacy code\n\norchestrate ultrathink workflowz");
    ::std::assert_eq!(already_has, "Modernize legacy code\n\norchestrate ultrathink workflowz");

    let tools = ::std::vec![];
    let cmd = ::protocol::DaemonToPaneCmd::dispatch_prompt("t1", "m1", "Review code", tools);
    if let ::protocol::DaemonToPaneCmd::DispatchPrompt {
        prompt,
        ..
    } = cmd
    {
        ::std::assert_eq!(prompt, "Review code\n\norchestrate ultrathink workflowz");
    } else {
        ::core::panic!("Expected DispatchPrompt variant");
    }

    let mut direct_cmd = ::protocol::DaemonToPaneCmd::DispatchPrompt {
        task_id: { "t2".to_string() },
        model: { "m2".to_string() },
        prompt: { "Direct prompt".to_string() },
        allowed_tools: { ::std::vec![] },
    };
    direct_cmd.ensure_magic_keywords();
    if let ::protocol::DaemonToPaneCmd::DispatchPrompt {
        prompt,
        ..
    } = direct_cmd
    {
        ::std::assert_eq!(prompt, "Direct prompt\n\norchestrate ultrathink workflowz");
    } else {
        ::core::panic!("Expected DispatchPrompt variant");
    }
}
