//! Tests for tui layout and interactive functions.

#[test]
fn test_compute_layout_6040() {
    let area = ::ratatui::layout::Rect::new(0, 0, 100, 50);
    let (left, right_panes) = ::interactive::compute_layout(area, 5);

    ::std::assert_eq!(left.width, 60);
    ::std::assert_eq!(left.height, 50);

    ::std::assert_eq!(right_panes.len(), 5);
    for pane in right_panes {
        ::std::assert_eq!(pane.x, 60);
        ::std::assert_eq!(pane.width, 40);
        ::std::assert_eq!(pane.height, 10);
    }
}

#[test]
fn test_compute_layout_dynamic_panes() {
    let area = ::ratatui::layout::Rect::new(0, 0, 100, 60);
    let (left, right_panes) = ::interactive::compute_layout(area, 3);

    ::std::assert_eq!(left.width, 60);
    ::std::assert_eq!(left.height, 60);

    ::std::assert_eq!(right_panes.len(), 3);
    for pane in right_panes {
        ::std::assert_eq!(pane.x, 60);
        ::std::assert_eq!(pane.width, 40);
        ::std::assert_eq!(pane.height, 20);
    }
}

#[test]
fn test_compute_layout_zero_panes() {
    let area = ::ratatui::layout::Rect::new(0, 0, 100, 50);
    let (left, right_panes) = ::interactive::compute_layout(area, 0);

    ::std::assert_eq!(left.width, 60);
    ::std::assert_eq!(left.height, 50);
    ::std::assert!(right_panes.is_empty());
}

#[test]
fn test_is_input_locked_to_left() {
    let locked_10 = ::interactive::is_input_locked_to_left(10, 60);
    ::std::assert!(locked_10);
    let locked_59 = ::interactive::is_input_locked_to_left(59, 60);
    ::std::assert!(locked_59);
    let locked_60 = ::interactive::is_input_locked_to_left(60, 60);
    ::std::assert!(!locked_60);
    let locked_75 = ::interactive::is_input_locked_to_left(75, 60);
    ::std::assert!(!locked_75);
}

#[test]
fn test_highlight_magic_keywords() {
    let prompt = "please ultrathink and orchestrate the workflowz run";
    let tokens = ::interactive::highlight_magic_keywords(prompt);

    let magic_tokens = tokens
        .into_iter()
        .filter(|(_, is_magic)| *is_magic)
        .map(|(w, _)| w)
        .collect::<::std::vec::Vec<::std::string::String>>();

    ::std::assert_eq!(
        magic_tokens,
        ::std::vec![
            ::std::string::String::from("ultrathink"),
            ::std::string::String::from("orchestrate"),
            ::std::string::String::from("workflowz")
        ]
    );
}

#[test]
fn test_csi_sync_constants() {
    ::std::assert_eq!(::interactive::CSI_SYNC_START, "\x1b[?2026h");
    ::std::assert_eq!(::interactive::CSI_SYNC_END, "\x1b[?2026l");
}

#[test]
fn test_format_stream_name_lowercased_hyphenated_no_numbers() {
    let name1 = ::interactive::format_stream_name(" ◈ 1. Legacy Analyst ");
    ::std::assert_eq!(name1, "legacy-analyst");

    let name2 = ::interactive::format_stream_name("2. Binary Historian / REA");
    ::std::assert_eq!(name2, "binary-historian-rea");

    let name3 = ::interactive::format_stream_name("3. Rules Extractor");
    ::std::assert_eq!(name3, "rules-extractor");

    let name4 = ::interactive::format_stream_name("foo_bar_baz");
    ::std::assert_eq!(name4, "foo-bar-baz");

    let name5 = ::interactive::format_stream_name("My Custom Agent");
    ::std::assert_eq!(name5, "my-custom-agent");
}

#[test]
fn test_default_specialist_streams_have_no_numbering() {
    let streams = ::interactive::default_specialist_streams();
    ::std::assert_eq!(streams.len(), 5);

    for stream in &streams {
        let has_digit = stream.name.chars().any(|ch| ch.is_ascii_digit());
        ::core::assert!(!has_digit);
        let is_lower = stream.name == stream.name.to_lowercase();
        ::core::assert!(is_lower);
        let has_spaces = stream.name.contains(' ');
        ::core::assert!(!has_spaces);
    }
}

#[test]
fn test_slash_command_dropdown_matching() {
    let streams = ::interactive::default_specialist_streams();
    let res_slash = ::interactive::resolve_dropdown_items("/", 1, &streams);
    ::core::assert!(res_slash.is_some());
    let (kind, items) = res_slash.unwrap();
    ::core::assert_eq!(kind, ::interactive::DropdownKind::SlashCommand);
    ::core::assert!(!items.is_empty());

    let res_q = ::interactive::resolve_dropdown_items("/q", 2, &streams);
    ::core::assert!(res_q.is_some());
    let (_, q_items) = res_q.unwrap();
    ::core::assert_eq!(q_items.len(), 1);
    ::core::assert_eq!(q_items[0].token, "/quit");
}

#[test]
fn test_agent_mention_dropdown_matching() {
    let streams = ::interactive::default_specialist_streams();
    let res_at = ::interactive::resolve_dropdown_items("@", 1, &streams);
    ::core::assert!(res_at.is_some());
    let (kind, items) = res_at.unwrap();
    ::core::assert_eq!(kind, ::interactive::DropdownKind::AgentMention);
    ::core::assert_eq!(items.len(), 5);

    let res_leg = ::interactive::resolve_dropdown_items("@leg", 4, &streams);
    ::core::assert!(res_leg.is_some());
    let (_, leg_items) = res_leg.unwrap();
    ::core::assert_eq!(leg_items.len(), 1);
    ::core::assert_eq!(leg_items[0].token, "@legacy-analyst");
}

#[::tokio::test]
async fn test_quit_command_terminates_session() {
    let streams = ::interactive::default_specialist_streams();
    let mut app = ::interactive::InteractiveCockpit::new(streams);
    app.input_buffer = "/quit".to_string();
    app.cursor_pos = 5;

    let enter_code = ::crossterm::event::KeyCode::Enter;
    let empty_mods = ::crossterm::event::KeyModifiers::empty();
    let key = ::crossterm::event::KeyEvent::new(enter_code, empty_mods);
    let should_exit = app.handle_key_event(key).await;
    ::core::assert!(should_exit);
    ::core::assert!(app.should_quit);
}

#[::tokio::test]
async fn test_esc_key_does_not_quit() {
    let streams = ::interactive::default_specialist_streams();
    let mut app = ::interactive::InteractiveCockpit::new(streams);
    app.input_buffer = "partial input".to_string();
    app.cursor_pos = 13;

    let esc_code = ::crossterm::event::KeyCode::Esc;
    let empty_mods = ::crossterm::event::KeyModifiers::empty();
    let key = ::crossterm::event::KeyEvent::new(esc_code, empty_mods);
    let should_exit = app.handle_key_event(key).await;
    ::core::assert!(!should_exit);
    ::core::assert!(!app.should_quit);
    ::core::assert!(app.input_buffer.is_empty());
}

#[::tokio::test]
async fn test_tab_completion_for_slash_and_at() {
    let streams = ::interactive::default_specialist_streams();
    let mut app = ::interactive::InteractiveCockpit::new(streams);

    app.input_buffer = "/q".to_string();
    app.cursor_pos = 2;
    let tab_code = ::crossterm::event::KeyCode::Tab;
    let empty_mods1 = ::crossterm::event::KeyModifiers::empty();
    let key_tab = ::crossterm::event::KeyEvent::new(tab_code, empty_mods1);
    let _ = app.handle_key_event(key_tab).await;
    ::core::assert_eq!(app.input_buffer, "/quit ");

    app.input_buffer = "@bin".to_string();
    app.cursor_pos = 4;
    let tab_code2 = ::crossterm::event::KeyCode::Tab;
    let empty_mods2 = ::crossterm::event::KeyModifiers::empty();
    let key_tab2 = ::crossterm::event::KeyEvent::new(tab_code2, empty_mods2);
    let _ = app.handle_key_event(key_tab2).await;
    ::core::assert_eq!(app.input_buffer, "@binary-historian ");
}

#[test]
fn test_right_pane_horizontal_margin() {
    let pane_rect = ::ratatui::layout::Rect::new(60, 0, 40, 10);
    let margin = ::ratatui::layout::Margin {
        horizontal: 1,
        vertical: 0,
    };
    let card_area = pane_rect.inner(margin);
    ::std::assert_eq!(card_area.x, 61);
    ::std::assert_eq!(card_area.width, 38);
    ::std::assert_eq!(card_area.y, 0);
    ::std::assert_eq!(card_area.height, 10);
}

#[test]
fn test_human_gate_extrinsic_conflict() {
    let gate = ::interactive::mock::HumanGate::GateOnExtrinsicConflict;
    let gate_string = gate.to_string();
    ::std::assert_eq!(gate_string, "Gate - Extrinsic Conflict Escalation.");
}
