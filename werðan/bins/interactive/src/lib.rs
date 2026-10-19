//! Terminal user interface layout, event loop, and interactive cockpit.

/// Synchronized terminal output start escape sequence.
pub const CSI_SYNC_START: &::core::primitive::str = "\x1b[?2026h";

/// Synchronized terminal output end escape sequence.
pub const CSI_SYNC_END: &::core::primitive::str = "\x1b[?2026l";

/// Sixty-forty split layout definition.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct TuiLayout6040 {
    /// Left orchestrator pane bounds.
    pub orchestrator_area: ::ratatui::layout::Rect,
    /// Right specialist stacked panes bounds.
    pub specialist_areas: ::std::vec::Vec<::ratatui::layout::Rect>,
}

/// Message emitted in left orchestrator pane.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub enum ChatMessage {
    /// User input message.
    User(::std::string::String),
    /// Orchestrator agent response.
    Orchestrator(::std::string::String),
    /// State machine lifecycle transition.
    Transition {
        from: ::statechart::ModernizationState,
        to: ::statechart::ModernizationState,
    },
    /// Human decision gate prompt.
    GatePrompt {
        gate: ::statechart::HumanGate,
        prompt: ::std::string::String,
    },
    /// Status or system notice.
    Notice(::std::string::String),
    /// Error notice.
    Error(::std::string::String),
}

/// Right pane specialist card state.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct SpecialistCardState {
    /// Associated agent role.
    pub role: ::operative::AgentRole,
    /// Title banner for card.
    pub title: ::std::string::String,
    /// Telemetry metrics text.
    pub telemetry: ::std::string::String,
    /// Status description.
    pub status: ::std::string::String,
    /// Color of card border and title.
    pub color: ::ratatui::style::Color,
    /// Continuous streamed output lines.
    pub stream_lines: ::std::vec::Vec<::std::string::String>,
    /// Scroll offset for pane.
    pub scroll_offset: ::core::primitive::usize,
}

/// Streaming event dispatched from background runner to cockpit.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub enum UiStreamEvent {
    /// Log line for specialist window.
    SpecialistLog {
        index: ::core::primitive::usize,
        line: ::std::string::String,
        telemetry: ::core::option::Option<::std::string::String>,
    },
    /// Orchestrator transcript line.
    OrchestratorLine(::std::string::String),
    /// State transition banner.
    StateTransition {
        from: ::statechart::ModernizationState,
        to: ::statechart::ModernizationState,
    },
    /// Human gate paused for approval.
    GateRequired {
        gate: ::statechart::HumanGate,
        prompt: ::std::string::String,
    },
    /// Workflow completed successfully.
    WorkflowCompleted {
        receipts: ::std::vec::Vec<::protocol::ReceiptStamp>,
    },
    /// Workflow encountered error.
    WorkflowFailed(::std::string::String),
}

/// Interactive modernization cockpit application state.
pub struct InteractiveCockpit {
    /// User input prompt text buffer.
    pub input_buffer: ::std::string::String,
    /// Cursor index within input buffer.
    pub cursor_pos: ::core::primitive::usize,
    /// Command history.
    pub history: ::std::vec::Vec<::std::string::String>,
    /// History browsing index.
    pub history_idx: ::core::option::Option<::core::primitive::usize>,
    /// Left pane chat transcript.
    pub messages: ::std::vec::Vec<self::ChatMessage>,
    /// Right pane specialist cards.
    pub cards: ::std::vec::Vec<self::SpecialistCardState>,
    /// Active draft configuration.
    pub draft_config: ::protocol::ModernizeConfig,
    /// Shared blackboard instance.
    pub blackboard: ::blackboard::Blackboard,
    /// Active state machine state.
    pub current_state: ::statechart::ModernizationState,
    /// Pending human gate.
    pub pending_gate: ::core::option::Option<::statechart::HumanGate>,
    /// Flag indicating whether workflow is running.
    pub is_running: ::core::primitive::bool,
    /// Stream event sender.
    pub event_tx: ::tokio::sync::mpsc::UnboundedSender<self::UiStreamEvent>,
    /// Stream event receiver.
    pub event_rx: ::tokio::sync::mpsc::UnboundedReceiver<self::UiStreamEvent>,
    /// Channel sender to resolve pending gate decisions.
    pub gate_tx: ::core::option::Option<::tokio::sync::mpsc::UnboundedSender<::statechart::GateDecision>>,
}

/// Start daemon server in interactive terminal.
pub async fn run_daemon_server() {
    let _ = self::run_interactive_tui().await;
}

/// Start interactive TUI cockpit session with ratatui terminal.
pub async fn run_interactive_tui() -> ::core::result::Result<(), ::std::string::String> {
    let mut term = ::ratatui::init();
    let loop_res = self::run_tui_event_loop(&mut term).await;
    ::ratatui::restore();
    loop_res
}

/// Drive synchronized sixty-forty cockpit until quit key.
pub async fn run_tui_event_loop(
    terminal: &mut ::ratatui::DefaultTerminal,
) -> ::core::result::Result<(), ::std::string::String> {
    let mut app = self::InteractiveCockpit::new();
    app.init_welcome();

    loop {
        app.drain_events();

        ::std::print!("{}", CSI_SYNC_START);
        let mut out = ::std::io::stdout();
        let _ = ::std::io::Write::flush(&mut out);

        let draw_res = terminal.draw(|frame| {
            let area = frame.area();
            let (left_area, right_panes) = self::compute_layout(area);

            self::render_orchestrator_cockpit(frame, left_area, &app);
            self::render_specialist_stack(frame, &right_panes, &app.cards);
        });

        ::std::print!("{}", CSI_SYNC_END);
        let mut out_end = ::std::io::stdout();
        let _ = ::std::io::Write::flush(&mut out_end);

        if let ::core::result::Result::Err(err) = draw_res {
            return ::core::result::Result::Err(err.to_string());
        }

        let timeout = ::core::time::Duration::from_millis(50);
        let poll_res = ::crossterm::event::poll(timeout);
        match poll_res {
            ::core::result::Result::Ok(true) => {
                if let ::core::result::Result::Ok(::crossterm::event::Event::Key(key)) = ::crossterm::event::read() {
                    let should_exit = app.handle_key_event(key).await;
                    if should_exit {
                        break;
                    }
                }
            },
            ::core::result::Result::Ok(false) => {},
            ::core::result::Result::Err(err) => {
                return ::core::result::Result::Err(err.to_string());
            },
        }
    }
    ::core::result::Result::Ok(())
}

/// Compute sixty-forty layout dividing terminal area into orchestrator and stack panes.
pub fn compute_layout(
    area: ::ratatui::layout::Rect,
) -> (::ratatui::layout::Rect, ::std::vec::Vec<::ratatui::layout::Rect>) {
    let main = ::ratatui::layout::Layout::default()
        .direction(::ratatui::layout::Direction::Horizontal)
        .constraints([::ratatui::layout::Constraint::Percentage(60), ::ratatui::layout::Constraint::Percentage(40)])
        .split(area);
    let left = main[0];
    let right = ::ratatui::layout::Layout::default()
        .direction(::ratatui::layout::Direction::Vertical)
        .constraints([
            ::ratatui::layout::Constraint::Percentage(20),
            ::ratatui::layout::Constraint::Percentage(20),
            ::ratatui::layout::Constraint::Percentage(20),
            ::ratatui::layout::Constraint::Percentage(20),
            ::ratatui::layout::Constraint::Percentage(20),
        ])
        .split(main[1]);
    (left, right.to_vec())
}

/// Check if click position falls within left pane bounds.
pub fn is_input_locked_to_left(
    clicked_col: ::core::primitive::u16,
    split_col: ::core::primitive::u16,
) -> ::core::primitive::bool {
    clicked_col < split_col
}

fn render_orchestrator_cockpit(
    frame: &mut ::ratatui::Frame<'_>,
    area: ::ratatui::layout::Rect,
    app: &self::InteractiveCockpit,
) {
    let outer_border_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let title_style = ::ratatui::style::Style::default()
        .fg(::ratatui::style::Color::Cyan)
        .add_modifier(::ratatui::style::Modifier::BOLD);

    let span_sym = ::ratatui::text::Span::styled(" ✦ ", title_style);
    let span_name = ::ratatui::text::Span::styled("Modernization Orchestrator (60%)", title_style);
    let span_desc = ::ratatui::text::Span::styled(" · Council Conductor ", outer_border_style);
    let span_vec = ::std::vec![span_sym, span_name, span_desc];
    let header_line = ::ratatui::text::Line::from(span_vec);

    let outer_block = ::ratatui::widgets::Block::default()
        .title(header_line)
        .borders(::ratatui::widgets::Borders::ALL)
        .border_style(outer_border_style);

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let v_chunks = ::ratatui::layout::Layout::default()
        .direction(::ratatui::layout::Direction::Vertical)
        .constraints([
            ::ratatui::layout::Constraint::Min(4),
            ::ratatui::layout::Constraint::Length(1),
            ::ratatui::layout::Constraint::Length(3),
        ])
        .split(inner);

    self::render_transcript_stream(frame, v_chunks[0], &app.messages);
    self::render_status_telemetry(frame, v_chunks[1], app);
    self::render_prompt_composer(frame, v_chunks[2], &app.input_buffer);
}

fn render_transcript_stream(
    frame: &mut ::ratatui::Frame<'_>,
    area: ::ratatui::layout::Rect,
    messages: &[self::ChatMessage],
) {
    let yellow_style = ::ratatui::style::Style::default()
        .fg(::ratatui::style::Color::Yellow)
        .add_modifier(::ratatui::style::Modifier::BOLD);
    let cyan_style = ::ratatui::style::Style::default()
        .fg(::ratatui::style::Color::Cyan)
        .add_modifier(::ratatui::style::Modifier::BOLD);
    let green_style = ::ratatui::style::Style::default()
        .fg(::ratatui::style::Color::Green)
        .add_modifier(::ratatui::style::Modifier::BOLD);
    let magenta_style = ::ratatui::style::Style::default()
        .fg(::ratatui::style::Color::Magenta)
        .add_modifier(::ratatui::style::Modifier::BOLD);
    let red_style = ::ratatui::style::Style::default()
        .fg(::ratatui::style::Color::Red)
        .add_modifier(::ratatui::style::Modifier::BOLD);
    let muted_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let text_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::White);

    let mut lines = ::std::vec::Vec::new();
    let max_lines = area.height as ::core::primitive::usize;
    let skip = if messages.len() > max_lines {
        messages.len() - max_lines
    } else {
        0
    };

    for msg in messages.iter().skip(skip) {
        match msg {
            self::ChatMessage::User(u) => {
                let span_u_prompt = ::ratatui::text::Span::styled("❯ user: ", yellow_style);
                let span_u_text = ::ratatui::text::Span::styled(u, text_style);
                let span_u_vec = ::std::vec![span_u_prompt, span_u_text];
                let l = ::ratatui::text::Line::from(span_u_vec);
                lines.push(l);
            },
            self::ChatMessage::Orchestrator(o) => {
                let span_o_icon = ::ratatui::text::Span::styled("  ✦ ", cyan_style);
                let span_o_text = ::ratatui::text::Span::styled(o, text_style);
                let span_o_vec = ::std::vec![span_o_icon, span_o_text];
                let l = ::ratatui::text::Line::from(span_o_vec);
                lines.push(l);
            },
            self::ChatMessage::Transition {
                from,
                to,
            } => {
                let banner = ::std::format!("── State Transition: {from} ➔ {to} ──");
                let span_trans = ::ratatui::text::Span::styled(banner, magenta_style);
                let span_trans_vec = ::std::vec![span_trans];
                let l = ::ratatui::text::Line::from(span_trans_vec);
                lines.push(l);
            },
            self::ChatMessage::GatePrompt {
                gate,
                prompt,
            } => {
                let prompt_text = ::std::format!("  [GATE] {gate}: {prompt} (Type /approve to proceed)");
                let span_gate = ::ratatui::text::Span::styled(prompt_text, green_style);
                let span_gate_vec = ::std::vec![span_gate];
                let l = ::ratatui::text::Line::from(span_gate_vec);
                lines.push(l);
            },
            self::ChatMessage::Notice(n) => {
                let span_n = ::ratatui::text::Span::styled(n, muted_style);
                let span_n_vec = ::std::vec![span_n];
                let l = ::ratatui::text::Line::from(span_n_vec);
                lines.push(l);
            },
            self::ChatMessage::Error(e) => {
                let span_e = ::ratatui::text::Span::styled(e, red_style);
                let span_e_vec = ::std::vec![span_e];
                let l = ::ratatui::text::Line::from(span_e_vec);
                lines.push(l);
            },
        }
    }

    let para = ::ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(para, area);
}

fn render_status_telemetry(
    frame: &mut ::ratatui::Frame<'_>,
    area: ::ratatui::layout::Rect,
    app: &self::InteractiveCockpit,
) {
    let brand_style = ::ratatui::style::Style::default()
        .fg(::ratatui::style::Color::Cyan)
        .add_modifier(::ratatui::style::Modifier::BOLD);
    let sep_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let green_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Green);
    let yellow_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Yellow);
    let magenta_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Magenta);
    let path_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Blue);

    let state_str = app.current_state.to_string();
    let gate_str = match &app.pending_gate {
        ::core::option::Option::Some(g) => g.to_string(),
        ::core::option::Option::None => "none".to_string(),
    };

    let span_brand = ::ratatui::text::Span::styled(" π werðan", brand_style);
    let span_sep1 = ::ratatui::text::Span::styled(" · ", sep_style);
    let span_state = ::ratatui::text::Span::styled(state_str, green_style);
    let span_sep2 = ::ratatui::text::Span::styled(" · ", sep_style);
    let span_gate = ::ratatui::text::Span::styled(::std::format!("gate: {gate_str}"), yellow_style);
    let span_sep3 = ::ratatui::text::Span::styled(" · ", sep_style);
    let span_id = ::ratatui::text::Span::styled(&app.draft_config.campaign_id, path_style);
    let span_sep4 = ::ratatui::text::Span::styled(" · ", sep_style);
    let span_target = ::ratatui::text::Span::styled(
        ::std::format!("target: {}", app.draft_config.stack.target_language),
        magenta_style,
    );
    let span_sep5 = ::ratatui::text::Span::styled(" · ", sep_style);
    let span_spec = ::ratatui::text::Span::styled("👥 5 specialists", yellow_style);

    let status_vec = ::std::vec![
        span_brand,
        span_sep1,
        span_state,
        span_sep2,
        span_gate,
        span_sep3,
        span_id,
        span_sep4,
        span_target,
        span_sep5,
        span_spec,
    ];
    let status_line = ::ratatui::text::Line::from(status_vec);

    let para = ::ratatui::widgets::Paragraph::new(status_line);
    frame.render_widget(para, area);
}

fn render_prompt_composer(
    frame: &mut ::ratatui::Frame<'_>,
    area: ::ratatui::layout::Rect,
    input_buffer: &::core::primitive::str,
) {
    let border_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let prompt_block = ::ratatui::widgets::Block::default()
        .title(" Prompt Composer (type /help, or Esc to exit) ")
        .borders(::ratatui::widgets::Borders::ALL)
        .border_style(border_style);

    let prompt_inner = prompt_block.inner(area);
    frame.render_widget(prompt_block, area);

    let yellow_bold = ::ratatui::style::Style::default()
        .fg(::ratatui::style::Color::Yellow)
        .add_modifier(::ratatui::style::Modifier::BOLD);
    let text_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::White);
    let magic_style = ::ratatui::style::Style::default()
        .fg(::ratatui::style::Color::Magenta)
        .add_modifier(::ratatui::style::Modifier::BOLD);
    let cursor_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Cyan);

    let mut spans = ::std::vec![::ratatui::text::Span::styled(" ❯ ", yellow_bold),];

    let tokens = self::highlight_magic_keywords(input_buffer);
    if tokens.is_empty() {
        let span_empty = ::ratatui::text::Span::styled(input_buffer, text_style);
        spans.push(span_empty);
    } else {
        for (idx, (token, is_magic)) in tokens.iter().enumerate() {
            if idx > 0 {
                let span_space = ::ratatui::text::Span::styled(" ", text_style);
                spans.push(span_space);
            }
            if *is_magic {
                let span_token = ::ratatui::text::Span::styled(token, magic_style);
                spans.push(span_token);
            } else {
                let span_token = ::ratatui::text::Span::styled(token, text_style);
                spans.push(span_token);
            }
        }
    }
    let span_cur = ::ratatui::text::Span::styled(" ▏", cursor_style);
    spans.push(span_cur);

    let prompt_line = ::ratatui::text::Line::from(spans);
    let para = ::ratatui::widgets::Paragraph::new(prompt_line);
    frame.render_widget(para, prompt_inner);
}

/// Highlight tokens ultrathink, orchestrate, and workflowz.
pub fn highlight_magic_keywords(
    prompt: &::core::primitive::str,
) -> ::std::vec::Vec<(::std::string::String, ::core::primitive::bool)> {
    let mut result = ::std::vec::Vec::new();
    let words = prompt.split_whitespace();
    for word in words {
        let is_magic = word == "ultrathink" || word == "orchestrate" || word == "workflowz";
        result.push((word.to_string(), is_magic));
    }
    result
}

fn render_specialist_stack(
    frame: &mut ::ratatui::Frame<'_>,
    right_panes: &[::ratatui::layout::Rect],
    cards: &[self::SpecialistCardState],
) {
    for (idx, &pane_rect) in right_panes.iter().enumerate() {
        if let ::core::option::Option::Some(card) = cards.get(idx) {
            self::render_specialist_card(frame, pane_rect, card);
        }
    }
}

fn render_specialist_card(
    frame: &mut ::ratatui::Frame<'_>,
    area: ::ratatui::layout::Rect,
    card: &self::SpecialistCardState,
) {
    let title_style = ::ratatui::style::Style::default().fg(card.color).add_modifier(::ratatui::style::Modifier::BOLD);
    let border_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let white_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::White);

    let span_title = ::ratatui::text::Span::styled(&card.title, title_style);
    let span_open = ::ratatui::text::Span::styled(" [", border_style);
    let span_tel = ::ratatui::text::Span::styled(&card.telemetry, white_style);
    let span_close = ::ratatui::text::Span::styled("] ", border_style);

    let card_header_vec = ::std::vec![span_title, span_open, span_tel, span_close];
    let header_line = ::ratatui::text::Line::from(card_header_vec);

    let block = ::ratatui::widgets::Block::default()
        .title(header_line)
        .borders(::ratatui::widgets::Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = ::std::vec::Vec::new();
    let text_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let max_lines = inner.height as ::core::primitive::usize;
    let skip = if card.stream_lines.len() > max_lines {
        card.stream_lines.len() - max_lines
    } else {
        0
    };

    for line in card.stream_lines.iter().skip(skip) {
        let span = ::ratatui::text::Span::styled(line, text_style);
        let line_vec = ::std::vec![span];
        let l = ::ratatui::text::Line::from(line_vec);
        lines.push(l);
    }

    let para = ::ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(para, inner);
}

impl ::core::default::Default for self::InteractiveCockpit {
    fn default() -> Self {
        Self::new()
    }
}

impl self::InteractiveCockpit {
    /// Create new interactive cockpit instance.
    pub fn new() -> Self {
        let (tx, rx) = ::tokio::sync::mpsc::unbounded_channel();
        let default_config = ::protocol::ModernizeConfig::default();
        let bb = ::blackboard::Blackboard::new();

        let initial_cards = ::std::vec![
            self::SpecialistCardState {
                role: ::operative::AgentRole::LegacyAnalyst,
                title: " ◈ 1. Legacy Analyst ".to_string(),
                telemetry: "scc: idle · AST pending".to_string(),
                status: "Idle".to_string(),
                color: ::ratatui::style::Color::Yellow,
                stream_lines: ::std::vec!["Awaiting orchestrator instruction.".to_string()],
                scroll_offset: 0,
            },
            self::SpecialistCardState {
                role: ::operative::AgentRole::BinaryHistorian,
                title: " ◈ 2. Binary Historian / REA ".to_string(),
                telemetry: "rea: compact · Ghidra ready".to_string(),
                status: "Idle".to_string(),
                color: ::ratatui::style::Color::Blue,
                stream_lines: ::std::vec!["Awaiting binary discovery in S1.".to_string()],
                scroll_offset: 0,
            },
            self::SpecialistCardState {
                role: ::operative::AgentRole::BusinessRulesExtractor,
                title: " ◈ 3. Rules Extractor ".to_string(),
                telemetry: "airlock: active · 0 cards".to_string(),
                status: "Idle".to_string(),
                color: ::ratatui::style::Color::Cyan,
                stream_lines: ::std::vec!["Awaiting shard allocation in S4.".to_string()],
                scroll_offset: 0,
            },
            self::SpecialistCardState {
                role: ::operative::AgentRole::ModuleLead,
                title: " ◈ 4. Module Lead & Synthesis ".to_string(),
                telemetry: "worktrees: 0 CoW · prewalk".to_string(),
                status: "Idle".to_string(),
                color: ::ratatui::style::Color::Magenta,
                stream_lines: ::std::vec!["Awaiting transformation wave in S6.".to_string()],
                scroll_offset: 0,
            },
            self::SpecialistCardState {
                role: ::operative::AgentRole::ArchitectureCritic,
                title: " ◈ 5. Adversarial Trio & Security ".to_string(),
                telemetry: "laya rubric: 12 rules · zero CWE".to_string(),
                status: "Idle".to_string(),
                color: ::ratatui::style::Color::Green,
                stream_lines: ::std::vec!["Awaiting audit review in S6c/S8.".to_string()],
                scroll_offset: 0,
            },
        ];

        Self {
            input_buffer: ::std::string::String::new(),
            cursor_pos: 0,
            history: ::std::vec::Vec::new(),
            history_idx: ::core::option::Option::None,
            messages: ::std::vec::Vec::new(),
            cards: { initial_cards },
            draft_config: { default_config },
            blackboard: { bb },
            current_state: ::statechart::ModernizationState::S0Intent,
            pending_gate: ::core::option::Option::None,
            is_running: false,
            event_tx: { tx },
            event_rx: { rx },
            gate_tx: ::core::option::Option::None,
        }
    }

    /// Process user keystroke. Returns true if application should terminate.
    pub async fn handle_key_event(&mut self, key: ::crossterm::event::KeyEvent) -> ::core::primitive::bool {
        match key.code {
            ::crossterm::event::KeyCode::Esc => true,
            ::crossterm::event::KeyCode::Char('c')
                if key.modifiers.contains(::crossterm::event::KeyModifiers::CONTROL) =>
            {
                true
            },
            ::crossterm::event::KeyCode::Enter => {
                let trimmed = self.input_buffer.trim().to_string();
                if !trimmed.is_empty() {
                    self.history.push(trimmed.clone());
                    self.history_idx = ::core::option::Option::None;
                    self.messages.push(self::ChatMessage::User(trimmed.clone()));
                    self.input_buffer.clear();
                    self.cursor_pos = 0;
                    self.handle_user_input(&trimmed).await;
                }
                false
            },
            ::crossterm::event::KeyCode::Backspace => {
                if self.cursor_pos > 0 && self.cursor_pos <= self.input_buffer.len() {
                    self.input_buffer.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
                false
            },
            ::crossterm::event::KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                false
            },
            ::crossterm::event::KeyCode::Right => {
                if self.cursor_pos < self.input_buffer.len() {
                    self.cursor_pos += 1;
                }
                false
            },
            ::crossterm::event::KeyCode::Up => {
                if !self.history.is_empty() {
                    let next_idx = match self.history_idx {
                        ::core::option::Option::Some(i) if i > 0 => i - 1,
                        ::core::option::Option::Some(_) => 0,
                        ::core::option::Option::None => self.history.len() - 1,
                    };
                    self.history_idx = ::core::option::Option::Some(next_idx);
                    let item = self.history[next_idx].clone();
                    self.input_buffer = item;
                    self.cursor_pos = self.input_buffer.len();
                }
                false
            },
            ::crossterm::event::KeyCode::Down => {
                if let ::core::option::Option::Some(idx) = self.history_idx {
                    if idx + 1 < self.history.len() {
                        let next_idx = idx + 1;
                        self.history_idx = ::core::option::Option::Some(next_idx);
                        let item = self.history[next_idx].clone();
                        self.input_buffer = item;
                        self.cursor_pos = self.input_buffer.len();
                    } else {
                        self.history_idx = ::core::option::Option::None;
                        self.input_buffer.clear();
                        self.cursor_pos = 0;
                    }
                }
                false
            },
            ::crossterm::event::KeyCode::Char(c) => {
                if self.cursor_pos <= self.input_buffer.len() {
                    self.input_buffer.insert(self.cursor_pos, c);
                    self.cursor_pos += 1;
                }
                false
            },
            _ => false,
        }
    }

    /// Process input text and dispatch slash command or conversation.
    pub async fn handle_user_input(&mut self, text: &::core::primitive::str) {
        if text.starts_with('/') {
            self.execute_slash_command(text).await;
        } else {
            self.converse_with_agent(text);
        }
    }

    /// Dispatch slash command.
    pub async fn execute_slash_command(&mut self, cmd: &::core::primitive::str) {
        let parts = cmd.split_whitespace().collect::<::std::vec::Vec<&::core::primitive::str>>();
        let command = parts[0];

        match command {
            "/help" => {
                self.messages.push(self::ChatMessage::Notice("Available Slash Commands.".to_string()));
                self.messages.push(self::ChatMessage::Notice("  /init [args]: Draft configuration.".to_string()));
                self.messages
                    .push(self::ChatMessage::Notice("  /save-config [path]: Save draft config to YAML.".to_string()));
                self.messages
                    .push(self::ChatMessage::Notice("  /load-config <path>: Load existing YAML config.".to_string()));
                self.messages.push(self::ChatMessage::Notice(
                    "  /start [--auto|--hitl]: Start modernization state machine.".to_string(),
                ));
                self.messages.push(self::ChatMessage::Notice(
                    "  /approve [notes]: Approve pending human gate in HITL mode.".to_string(),
                ));
                self.messages
                    .push(self::ChatMessage::Notice("  /reject <reason>: Reject pending human gate.".to_string()));
                self.messages.push(self::ChatMessage::Notice(
                    "  /status: Display state machine and blackboard telemetry.".to_string(),
                ));
                self.messages.push(self::ChatMessage::Notice("  /clear: Clear chat transcript.".to_string()));
                self.messages.push(self::ChatMessage::Notice("  /quit or Esc: Exit application.".to_string()));
            },
            "/clear" => {
                self.messages.clear();
                self.init_welcome();
            },
            "/init" => {
                self.handle_init_command(&parts[1..]);
            },
            "/save-config" => {
                let path_str = if parts.len() > 1 {
                    parts[1]
                } else {
                    "modernize.yml"
                };
                self.save_config_to_path(path_str);
            },
            "/load-config" => {
                if parts.len() > 1 {
                    self.load_config_from_path(parts[1]);
                } else {
                    self.messages
                        .push(self::ChatMessage::Error("Usage: /load-config <path/to/config.yml>".to_string()));
                }
            },
            "/start" => {
                let is_auto = parts.contains(&"--auto");
                let is_hitl = parts.contains(&"--hitl");
                let mode = if is_auto {
                    ::protocol::ExecutionMode::FullAuto
                } else if is_hitl {
                    ::protocol::ExecutionMode::Hitl
                } else {
                    self.draft_config.execution_mode
                };
                self.start_modernization(mode);
            },
            "/approve" => {
                let notes = if parts.len() > 1 {
                    parts[1..].join(" ")
                } else {
                    "Approved via TUI slash command".to_string()
                };
                self.resolve_gate_approval(&notes);
            },
            "/reject" => {
                let reason = if parts.len() > 1 {
                    parts[1..].join(" ")
                } else {
                    "Rejected via TUI slash command".to_string()
                };
                self.resolve_gate_rejection(&reason);
            },
            "/status" => {
                let st = self.current_state;
                let is_run = self.is_running;
                let cov = self.blackboard.obligation_coverage().await;
                let taint = self.blackboard.is_tainted().await;
                let cards_len = self.blackboard.rule_cards().await.len();
                let gate_str = match &self.pending_gate {
                    ::core::option::Option::Some(g) => g.to_string(),
                    ::core::option::Option::None => "None".to_string(),
                };
                let status_msg = ::std::format!(
                    "State: {st} | Running: {is_run} | Pending Gate: {gate_str} | REA Coverage: {cov:.1}% | Cards: {cards_len} | Tainted: {taint}"
                );
                self.messages.push(self::ChatMessage::Notice(status_msg));
            },
            "/quit" | "/exit" => {
                self.messages.push(self::ChatMessage::Notice("Terminating session...".to_string()));
            },
            _ => {
                let err_msg = ::std::format!("Unknown command `{command}`. Type /help for command list.");
                self.messages.push(self::ChatMessage::Error(err_msg));
            },
        }
    }

    /// Populate welcome transcript lines.
    pub fn init_welcome(&mut self) {
        self.messages.push(self::ChatMessage::Notice("✦ Welcome to werðan Modernization Cockpit.".to_string()));
        self.messages.push(self::ChatMessage::Notice(
            "  Type /init to converse with agent and finalize modernize.yml.".to_string(),
        ));
        self.messages.push(self::ChatMessage::Notice(
            "  Type /start to execute workflow, or /help for command list.".to_string(),
        ));
        self.messages.push(self::ChatMessage::Notice("  Directives: ultrathink · orchestrate · workflowz".to_string()));
    }

    /// Drain incoming asynchronous events into UI state.
    pub fn drain_events(&mut self) {
        while let ::core::result::Result::Ok(ev) = self.event_rx.try_recv() {
            match ev {
                self::UiStreamEvent::SpecialistLog {
                    index,
                    line,
                    telemetry,
                } => {
                    if let ::core::option::Option::Some(card) = self.cards.get_mut(index) {
                        card.stream_lines.push(line);
                        card.status = "Active".to_string();
                        if let ::core::option::Option::Some(tel) = telemetry {
                            card.telemetry = tel;
                        }
                    }
                },
                self::UiStreamEvent::OrchestratorLine(line) => {
                    self.messages.push(self::ChatMessage::Orchestrator(line));
                },
                self::UiStreamEvent::StateTransition {
                    from,
                    to,
                } => {
                    self.current_state = to;
                    self.messages.push(self::ChatMessage::Transition {
                        from,
                        to,
                    });
                },
                self::UiStreamEvent::GateRequired {
                    gate,
                    prompt,
                } => {
                    self.pending_gate = ::core::option::Option::Some(gate);
                    self.messages.push(self::ChatMessage::GatePrompt {
                        gate,
                        prompt,
                    });
                },
                self::UiStreamEvent::WorkflowCompleted {
                    receipts,
                } => {
                    self.is_running = false;
                    self.pending_gate = ::core::option::Option::None;
                    let count = receipts.len();
                    let msg = ::std::format!("✔ Workflow finished with {count} verified receipts. All gates passed.");
                    self.messages.push(self::ChatMessage::Notice(msg));
                },
                self::UiStreamEvent::WorkflowFailed(err) => {
                    self.is_running = false;
                    self.pending_gate = ::core::option::Option::None;
                    let msg = ::std::format!("✖ Workflow failed: {err}");
                    self.messages.push(self::ChatMessage::Error(msg));
                },
            }
        }
    }

    /// Process `/init` arguments to update draft configuration.
    fn handle_init_command(&mut self, args: &[&::core::primitive::str]) {
        for arg in args {
            if let ::core::option::Option::Some((k, v)) = arg.split_once('=') {
                match k {
                    "input" | "legacy" => self.draft_config.paths.input_legacy_dir = v.to_string(),
                    "output" | "workspace" => self.draft_config.paths.workspace_root = v.to_string(),
                    "lang" | "source_lang" => self.draft_config.stack.source_language = v.to_string(),
                    "target" | "target_lang" => self.draft_config.stack.target_language = v.to_string(),
                    "id" | "campaign" => self.draft_config.campaign_id = v.to_string(),
                    "mode" => {
                        if v == "auto" {
                            self.draft_config.execution_mode = ::protocol::ExecutionMode::FullAuto;
                        } else {
                            self.draft_config.execution_mode = ::protocol::ExecutionMode::Hitl;
                        }
                    },
                    _ => {},
                }
            }
        }

        self.messages
            .push(self::ChatMessage::Orchestrator("I prepared the modernization configuration draft:".to_string()));

        let yaml_str = ::serde_yaml_ng::to_string(&self.draft_config).unwrap_or_default();
        for line in yaml_str.lines() {
            let indented = ::std::format!("  {line}");
            self.messages.push(self::ChatMessage::Orchestrator(indented));
        }

        self.messages.push(self::ChatMessage::Orchestrator(
            "Type adjustments (e.g. `set input to <path>`), or `/save-config`, then `/start`.".to_string(),
        ));
    }

    /// Converse with agent to update draft configuration.
    fn converse_with_agent(&mut self, text: &::core::primitive::str) {
        let lower = text.to_lowercase();
        let mut modified = false;

        if lower.contains("input") || lower.contains("legacy") {
            let words = text.split_whitespace().collect::<::std::vec::Vec<&::core::primitive::str>>();
            if let ::core::option::Option::Some(pos) = words.iter().position(|&w| w == "to" || w == "=") {
                if pos + 1 < words.len() {
                    let path = words[pos + 1].trim_matches('"').trim_matches('\'');
                    self.draft_config.paths.input_legacy_dir = path.to_string();
                    modified = true;
                }
            }
        }

        if lower.contains("workspace") || lower.contains("output") {
            let words = text.split_whitespace().collect::<::std::vec::Vec<&::core::primitive::str>>();
            if let ::core::option::Option::Some(pos) = words.iter().position(|&w| w == "to" || w == "=") {
                if pos + 1 < words.len() {
                    let path = words[pos + 1].trim_matches('"').trim_matches('\'');
                    self.draft_config.paths.workspace_root = path.to_string();
                    modified = true;
                }
            }
        }

        if lower.contains("rust") {
            self.draft_config.stack.target_language = "rust".to_string();
            modified = true;
        }

        if lower.contains("auto") {
            self.draft_config.execution_mode = ::protocol::ExecutionMode::FullAuto;
            modified = true;
        } else if lower.contains("hitl") {
            self.draft_config.execution_mode = ::protocol::ExecutionMode::Hitl;
            modified = true;
        }

        if modified {
            let msg = ::std::format!(
                "Updated draft configuration:\n  input: {}\n  workspace: {}\n  target: {}\n  mode: {:?}",
                self.draft_config.paths.input_legacy_dir,
                self.draft_config.paths.workspace_root,
                self.draft_config.stack.target_language,
                self.draft_config.execution_mode
            );
            self.messages.push(self::ChatMessage::Orchestrator(msg));
            self.messages.push(self::ChatMessage::Orchestrator(
                "Type /save-config to save to disk, or /start to start.".to_string(),
            ));
        } else {
            let reply = ::std::format!(
                "Understood: \"{text}\". Tracking campaign `{}` (target: {}). Type /init to edit parameters or /start to start.",
                self.draft_config.campaign_id,
                self.draft_config.stack.target_language
            );
            self.messages.push(self::ChatMessage::Orchestrator(reply));
        }
    }

    /// Save configuration draft to file path.
    fn save_config_to_path(&mut self, path: &::core::primitive::str) {
        let yaml_res = ::serde_yaml_ng::to_string(&self.draft_config);
        match yaml_res {
            ::core::result::Result::Ok(content) => {
                let write_res = ::std::fs::write(path, content);
                match write_res {
                    ::core::result::Result::Ok(_) => {
                        let success_msg =
                            ::std::format!("✔ Saved configuration to `{path}`. You may now run `/start`.");
                        self.messages.push(self::ChatMessage::Notice(success_msg));
                    },
                    ::core::result::Result::Err(err) => {
                        let err_msg = ::std::format!("Failed to write `{path}`: {err}");
                        self.messages.push(self::ChatMessage::Error(err_msg));
                    },
                }
            },
            ::core::result::Result::Err(err) => {
                let err_msg = ::std::format!("Failed to serialize YAML: {err}");
                self.messages.push(self::ChatMessage::Error(err_msg));
            },
        }
    }

    /// Load configuration from file path.
    fn load_config_from_path(&mut self, path: &::core::primitive::str) {
        let read_res = ::std::fs::read_to_string(path);
        match read_res {
            ::core::result::Result::Ok(content) => {
                let parse_res = ::serde_yaml_ng::from_str::<::protocol::ModernizeConfig>(&content);
                match parse_res {
                    ::core::result::Result::Ok(cfg) => {
                        self.draft_config = cfg;
                        let msg = ::std::format!(
                            "✔ Loaded configuration `{}` from `{path}` (target: {}).",
                            self.draft_config.campaign_id,
                            self.draft_config.stack.target_language
                        );
                        self.messages.push(self::ChatMessage::Notice(msg));
                    },
                    ::core::result::Result::Err(err) => {
                        let msg = ::std::format!("Failed to parse YAML `{path}`: {err}");
                        self.messages.push(self::ChatMessage::Error(msg));
                    },
                }
            },
            ::core::result::Result::Err(err) => {
                let msg = ::std::format!("Failed to read file `{path}`: {err}");
                self.messages.push(self::ChatMessage::Error(msg));
            },
        }
    }

    /// Start modernization workflow with given execution mode.
    fn start_modernization(&mut self, mode: ::protocol::ExecutionMode) {
        if self.is_running {
            self.messages.push(self::ChatMessage::Error("Modernization workflow is already running.".to_string()));
            return;
        }

        self.draft_config.execution_mode = mode;
        let is_auto = mode == ::protocol::ExecutionMode::FullAuto;
        let start_msg = ::std::format!(
            "▶ Starting modernization workflow for campaign `{}` (mode: {:?}).",
            self.draft_config.campaign_id,
            mode
        );
        self.messages.push(self::ChatMessage::Notice(start_msg));
        self.is_running = true;

        let (gate_tx, mut gate_rx) = ::tokio::sync::mpsc::unbounded_channel::<::statechart::GateDecision>();
        self.gate_tx = ::core::option::Option::Some(gate_tx);

        let ui_tx = self.event_tx.clone();
        let cfg = self.draft_config.clone();
        let bb = self.blackboard.clone();

        ::tokio::spawn(async move {
            let mut runner = ::statechart::StateMachineRunner::new(cfg, bb.clone(), is_auto);
            let mut sub = bb.subscribe();

            let ui_tx_sub = ui_tx.clone();
            ::tokio::spawn(async move {
                while let ::core::result::Result::Ok(ev) = sub.recv().await {
                    match ev {
                        ::blackboard::BlackboardEvent::ObligationAdded {
                            name,
                            kind,
                            address,
                        } => {
                            let line = ::std::format!("+ Obligation [{kind}] {name} @ {address}");
                            let _ = ui_tx_sub.send(self::UiStreamEvent::SpecialistLog {
                                index: 1,
                                line,
                                telemetry: ::core::option::Option::Some("REA obligations active".to_string()),
                            });
                        },
                        ::blackboard::BlackboardEvent::RuleCardAdded {
                            id,
                            title,
                            citation,
                        } => {
                            let line = ::std::format!("+ RuleCard {id}: {title} (cite: {citation})");
                            let _ = ui_tx_sub.send(self::UiStreamEvent::SpecialistLog {
                                index: 2,
                                line,
                                telemetry: ::core::option::Option::Some("Airlock verified clean".to_string()),
                            });
                        },
                        ::blackboard::BlackboardEvent::ModuleWorktreeUpdated {
                            module_name,
                            status,
                            todo_count,
                        } => {
                            let line = ::std::format!("CoW worktree [{module_name}]: {status} ({todo_count} todos)");
                            let _ = ui_tx_sub.send(self::UiStreamEvent::SpecialistLog {
                                index: 3,
                                line,
                                telemetry: ::core::option::Option::Some(::std::format!("{status} ({module_name})")),
                            });
                        },
                        ::blackboard::BlackboardEvent::TaintDetected {
                            path,
                            reason,
                        } => {
                            let line = ::std::format!("! Taint: {path} ({reason})");
                            let _ = ui_tx_sub.send(self::UiStreamEvent::SpecialistLog {
                                index: 4,
                                line,
                                telemetry: ::core::option::Option::Some("Taint guardrail triggered".to_string()),
                            });
                        },
                        _ => {},
                    }
                }
            });

            loop {
                match runner.step() {
                    ::core::result::Result::Ok(::statechart::StepOutcome::Advanced {
                        from,
                        to,
                    }) => {
                        let _ = ui_tx.send(self::UiStreamEvent::StateTransition {
                            from,
                            to,
                        });
                        let log_line = ::std::format!("FSM transition: {from} -> {to}");
                        let _ = ui_tx.send(self::UiStreamEvent::OrchestratorLine(log_line));

                        let _ = ui_tx.send(self::UiStreamEvent::SpecialistLog {
                            index: 0,
                            line: ::std::format!("State advanced to {to}"),
                            telemetry: ::core::option::Option::Some(::std::format!("Active in {to}")),
                        });

                        let dur = ::core::time::Duration::from_millis(50);
                        ::tokio::time::sleep(dur).await;
                    },
                    ::core::result::Result::Ok(::statechart::StepOutcome::GateRequired {
                        gate,
                        prompt,
                    }) => {
                        let _ = ui_tx.send(self::UiStreamEvent::GateRequired {
                            gate,
                            prompt,
                        });
                        if let ::core::option::Option::Some(decision) = gate_rx.recv().await {
                            if let ::core::result::Result::Err(err) = runner.submit_gate_decision(decision) {
                                let _ = ui_tx.send(self::UiStreamEvent::WorkflowFailed(err));
                                break;
                            }
                        } else {
                            let _ = ui_tx.send(self::UiStreamEvent::WorkflowFailed("Gate channel closed".to_string()));
                            break;
                        }
                    },
                    ::core::result::Result::Ok(::statechart::StepOutcome::Completed {
                        receipts,
                    }) => {
                        let _ = ui_tx.send(self::UiStreamEvent::WorkflowCompleted {
                            receipts,
                        });
                        break;
                    },
                    ::core::result::Result::Err(err) => {
                        let _ = ui_tx.send(self::UiStreamEvent::WorkflowFailed(err));
                        break;
                    },
                }
            }
        });
    }

    /// Resolve pending gate with approval decision.
    fn resolve_gate_approval(&mut self, notes: &::core::primitive::str) {
        if let ::core::option::Option::Some(gate) = self.pending_gate.take() {
            if let ::core::option::Option::Some(tx) = &self.gate_tx {
                let decision = ::statechart::GateDecision::Approved {
                    approver: "operator".to_string(),
                    notes: { notes.to_string() },
                };
                let _ = tx.send(decision);
                let msg = ::std::format!("✔ Approved {gate}: {notes}");
                self.messages.push(self::ChatMessage::Notice(msg));
            }
        } else {
            self.messages.push(self::ChatMessage::Error("No human gate is currently pending approval.".to_string()));
        }
    }

    /// Resolve pending gate with rejection decision.
    fn resolve_gate_rejection(&mut self, reason: &::core::primitive::str) {
        if let ::core::option::Option::Some(gate) = self.pending_gate.take() {
            if let ::core::option::Option::Some(tx) = &self.gate_tx {
                let decision = ::statechart::GateDecision::Rejected {
                    reason: { reason.to_string() },
                };
                let _ = tx.send(decision);
                let msg = ::std::format!("✖ Rejected {gate}: {reason}");
                self.messages.push(self::ChatMessage::Notice(msg));
            }
        } else {
            self.messages.push(self::ChatMessage::Error("No human gate is currently pending rejection.".to_string()));
        }
    }
}
