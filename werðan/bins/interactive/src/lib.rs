//! Terminal user interface layout, event loop, and interactive cockpit.

use ::configurator::AskBridge;

/// Mock module replacing invalidated infras and topology crates.
pub mod mock;

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
        from: self::mock::ModernizationState,
        to: self::mock::ModernizationState,
    },
    /// Human decision gate prompt.
    GatePrompt {
        gate: self::mock::HumanGate,
        prompt: ::std::string::String,
    },
    /// Status or system notice.
    Notice(::std::string::String),
    /// Error notice.
    Error(::std::string::String),
}

/// Ask bridge surfacing configurator questions to the cockpit transcript.
///
/// Contract: `ask` pushes one Notice message per question into the owned
/// buffer `emitted`, and the caller drains `emitted` into
/// `InteractiveCockpit::messages`. The caller preloads the `canned`
/// answer queue before calling `ask`. When no canned answer exists for a
/// question id, the bridge returns the recommended option when present,
/// else an empty string. An empty string leaves the wizard default in
/// place, because `apply_answer` ignores empty values.
pub struct TuiAskBridge {
    /// Question messages formatted for the cockpit transcript.
    pub emitted: ::std::vec::Vec<self::ChatMessage>,
    /// Preloaded answers keyed by question id.
    pub canned: ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
}

impl self::TuiAskBridge {
    /// Create empty bridge without canned answers.
    pub fn new() -> Self {
        Self {
            emitted: ::std::vec::Vec::new(),
            canned: ::std::vec::Vec::new(),
        }
    }
}

impl ::core::default::Default for self::TuiAskBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl ::configurator::AskBridge for self::TuiAskBridge {
    fn ask(
        &mut self,
        questions: ::std::vec::Vec<::configurator::ConfigQuestion>,
    ) -> ::std::vec::Vec<::configurator::ConfigAnswer> {
        questions
            .into_iter()
            .map(|question| -> ::configurator::ConfigAnswer {
                self.emit_question(&question);
                let canned_res = self
                    .canned
                    .iter()
                    .find(|(canned_id, _)| canned_id == &question.id);
                let value = match canned_res {
                    ::core::option::Option::Some((_, canned_value)) => canned_value.clone(),
                    ::core::option::Option::None => match question.recommended {
                        ::core::option::Option::Some(idx) => question
                            .options
                            .get(idx)
                            .cloned()
                            .unwrap_or_default(),
                        ::core::option::Option::None => ::std::string::String::new(),
                    },
                };
                let answer_id = { question.id.clone() };
                ::configurator::ConfigAnswer {
                    id: { answer_id },
                    value,
                }
            })
            .collect()
    }
}

impl self::TuiAskBridge {
    /// Push one formatted question onto the transcript buffer.
    fn emit_question(
        &mut self,
        question: &::configurator::ConfigQuestion,
    ) {
        let options_text = question
            .options
            .iter()
            .map(|option| -> ::std::string::String { ::std::format!("(a) {option}") })
            .collect::<::std::vec::Vec<::std::string::String>>()
            .join(" ");
        let recommended_text = match question.recommended {
            ::core::option::Option::Some(idx) => question
                .options
                .get(idx)
                .map(|opt| -> ::std::string::String { ::std::format!(" [recommended: {opt}]") })
                .unwrap_or_default(),
            ::core::option::Option::None => ::std::string::String::new(),
        };
        let text = ::std::format!(
            "Question [{}]: {}{} {}",
            question.id,
            question.question,
            recommended_text,
            options_text
        );
        self.emitted.push(self::ChatMessage::Notice(text));
    }
}

/// Completion dropdown category.
#[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::fmt::Debug)]
pub enum DropdownKind {
    /// Slash command autocomplete dropdown.
    SlashCommand,
    /// Agent mention autocomplete dropdown.
    AgentMention,
}

/// Item displayed inside completion dropdown.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct DropdownItem {
    /// Autocomplete token text.
    pub token: ::std::string::String,
    /// Description hint text.
    pub description: ::std::string::String,
}

/// Right pane specialist card state.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct SpecialistCardState {
    /// Stream identifier name (lowercased and hyphenated).
    pub name: ::std::string::String,
    /// Associated agent role.
    pub role: self::mock::AgentRole,
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

impl self::SpecialistCardState {
    /// Create new specialist card state with lowercased hyphenated name.
    pub fn new(
        name: &::core::primitive::str,
        role: self::mock::AgentRole,
        color: ::ratatui::style::Color,
    ) -> Self {
        let clean_name = self::format_stream_name(name);
        Self {
            name: { clean_name.clone() },
            role: { role },
            title: { clean_name },
            telemetry: "idle".to_string(),
            status: "Idle".to_string(),
            color: { color },
            stream_lines: ::std::vec::Vec::new(),
            scroll_offset: 0,
        }
    }
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
        from: self::mock::ModernizationState,
        to: self::mock::ModernizationState,
    },
    /// Human gate paused for approval.
    GateRequired {
        gate: self::mock::HumanGate,
        prompt: ::std::string::String,
    },
    /// Workflow completed successfully.
    WorkflowCompleted {
        receipts: ::std::vec::Vec<self::mock::ReceiptStamp>,
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
    pub draft_config: self::mock::ModernizeConfig,
    /// Active configurator wizard agent when a wizard session started.
    pub wizard: ::core::option::Option<::configurator::ConfiguratorAgent>,
    /// Shared blackboard instance.
    pub blackboard: self::mock::Blackboard,
    /// Persisted runner state vector: single source of truth for rendering.
    pub state_vector: self::mock::RunnerStateVector,
    /// Pending human gate.
    pub pending_gate: ::core::option::Option<self::mock::HumanGate>,
    /// Flag indicating whether workflow is running.
    pub is_running: ::core::primitive::bool,
    /// Stream event sender.
    pub event_tx: ::tokio::sync::mpsc::UnboundedSender<self::UiStreamEvent>,
    /// Stream event receiver.
    pub event_rx: ::tokio::sync::mpsc::UnboundedReceiver<self::UiStreamEvent>,
    /// Channel sender to resolve pending gate decisions.
    pub gate_tx: ::core::option::Option<::tokio::sync::mpsc::UnboundedSender<self::mock::GateDecision>>,
    /// Selected dropdown index.
    pub dropdown_selected_idx: ::core::primitive::usize,
    /// Flag indicating whether dropdown is temporarily dismissed.
    pub dropdown_dismissed: ::core::primitive::bool,
    /// Flag indicating whether session should terminate.
    pub should_quit: ::core::primitive::bool,
}

/// Start daemon server in interactive terminal.
pub async fn run_daemon_server() {
    let _ = self::run_interactive_tui().await;
}

/// Start interactive TUI cockpit session with ratatui terminal.
pub async fn run_interactive_tui() -> ::core::result::Result<(), ::std::string::String> {
    let streams = self::default_specialist_streams();
    self::run_interactive_tui_with_streams(streams).await
}

/// Return default specialist streams with lowercased and hyphenated names.
pub fn default_specialist_streams() -> ::std::vec::Vec<self::SpecialistCardState> {

    let card_analyst = self::SpecialistCardState {
        name: "legacy-analyst".to_string(),
        role: self::mock::AgentRole::LegacyAnalyst,
        title: "legacy-analyst".to_string(),
        telemetry: "scc: idle · AST pending".to_string(),
        status: "Idle".to_string(),
        color: ::ratatui::style::Color::Yellow,
        stream_lines: ::std::vec!["Awaiting orchestrator instruction.".to_string()],
        scroll_offset: 0,
    };

    let card_historian = self::SpecialistCardState {
        name: "binary-historian".to_string(),
        role: self::mock::AgentRole::BinaryHistorian,
        title: "binary-historian".to_string(),
        telemetry: "rea: compact · Ghidra ready".to_string(),
        status: "Idle".to_string(),
        color: ::ratatui::style::Color::Blue,
        stream_lines: ::std::vec!["Awaiting binary discovery in S1.".to_string()],
        scroll_offset: 0,
    };

    let card_rules = self::SpecialistCardState {
        name: "rules-extractor".to_string(),
        role: self::mock::AgentRole::BusinessRulesExtractor,
        title: "rules-extractor".to_string(),
        telemetry: "airlock: active · 0 cards".to_string(),
        status: "Idle".to_string(),
        color: ::ratatui::style::Color::Cyan,
        stream_lines: ::std::vec!["Awaiting shard allocation in S4.".to_string()],
        scroll_offset: 0,
    };

    let card_lead = self::SpecialistCardState {
        name: "module-lead".to_string(),
        role: self::mock::AgentRole::ModuleLead,
        title: "module-lead".to_string(),
        telemetry: "worktrees: 0 CoW · prewalk".to_string(),
        status: "Idle".to_string(),
        color: ::ratatui::style::Color::Magenta,
        stream_lines: ::std::vec!["Awaiting transformation wave in S6.".to_string()],
        scroll_offset: 0,
    };

    let card_critic = self::SpecialistCardState {
        name: "architecture-critic".to_string(),
        role: self::mock::AgentRole::ArchitectureCritic,
        title: "architecture-critic".to_string(),
        telemetry: "laya rubric: 12 rules · zero CWE".to_string(),
        status: "Idle".to_string(),
        color: ::ratatui::style::Color::Green,
        stream_lines: ::std::vec!["Awaiting audit review in S6c/S8.".to_string()],
        scroll_offset: 0,
    };

    ::std::vec![card_analyst, card_historian, card_rules, card_lead, card_critic]
}

/// Format stream name into lowercased and hyphenated identifier without numbering.
pub fn format_stream_name(input: &::core::primitive::str) -> ::std::string::String {
    let mut result = ::std::string::String::new();
    let trimmed = input.trim();
    let alpha_start = trimmed.find(::core::primitive::char::is_alphabetic).unwrap_or(0);
    let stripped = &trimmed[alpha_start..];

    let mut prev_hyphen = false;
    for ch in stripped.chars() {
        if ch.is_ascii_alphanumeric() {
            let lower = ch.to_ascii_lowercase();
            result.push(lower);
            prev_hyphen = false;
        } else {
            let is_sep = ch == ' ' || ch == '-' || ch == '_' || ch == '/';
            let is_valid_gap = is_sep && !prev_hyphen && !result.is_empty();
            if is_valid_gap {
                result.push('-');
                prev_hyphen = true;
            }
        }
    }

    let trimmed_result = result.trim_end_matches('-');
    trimmed_result.to_string()
}

/// Start interactive TUI cockpit session with given streams.
pub async fn run_interactive_tui_with_streams(
    streams: ::std::vec::Vec<self::SpecialistCardState>,
) -> ::core::result::Result<(), ::std::string::String> {
    let mut term = ::ratatui::init();
    let loop_res = self::run_tui_event_loop(&mut term, streams).await;
    ::ratatui::restore();
    loop_res
}

/// Drive synchronized sixty-forty cockpit until quit key.
pub async fn run_tui_event_loop(
    terminal: &mut ::ratatui::DefaultTerminal,
    streams: ::std::vec::Vec<self::SpecialistCardState>,
) -> ::core::result::Result<(), ::std::string::String> {
    let mut app = self::InteractiveCockpit::new(streams);
    app.init_welcome();

    loop {
        app.drain_events();

        ::std::print!("{}", CSI_SYNC_START);
        let mut out = ::std::io::stdout();
        let _ = ::std::io::Write::flush(&mut out);

        let draw_res = terminal.draw(|frame| {
            let area = frame.area();
            let cards_len = app.cards.len();
            let (left_area, right_panes) = self::compute_layout(area, cards_len);

            self::render_orchestrator_cockpit(frame, left_area, &app);
            self::render_specialist_stack(frame, &right_panes, &app.cards);
        });

        ::std::print!("{}", CSI_SYNC_END);
        let mut out_end = ::std::io::stdout();
        let _ = ::std::io::Write::flush(&mut out_end);

        if let ::core::result::Result::Err(err) = draw_res {
            let err_str = err.to_string();
            return ::core::result::Result::Err(err_str);
        }

        let timeout = ::core::time::Duration::from_millis(50);
        let poll_res = ::crossterm::event::poll(timeout);
        match poll_res {
            ::core::result::Result::Ok(true) => {
                let event_res = ::crossterm::event::read();
                if let ::core::result::Result::Ok(::crossterm::event::Event::Key(key)) = event_res {
                    let should_exit = app.handle_key_event(key).await;
                    if should_exit {
                        break;
                    }
                }
            },
            ::core::result::Result::Ok(false) => {},
            ::core::result::Result::Err(err) => {
                let err_str = err.to_string();
                return ::core::result::Result::Err(err_str);
            },
        }
    }
    ::core::result::Result::Ok(())
}

/// Compute sixty-forty layout dividing terminal area into orchestrator and stack panes.
pub fn compute_layout(
    area: ::ratatui::layout::Rect,
    pane_count: ::core::primitive::usize,
) -> (::ratatui::layout::Rect, ::std::vec::Vec<::ratatui::layout::Rect>) {
    let default_layout = ::ratatui::layout::Layout::default();
    let h_dir = ::ratatui::layout::Direction::Horizontal;
    let main_constraints = [
        ::ratatui::layout::Constraint::Percentage(60),
        ::ratatui::layout::Constraint::Percentage(40),
    ];
    let main = default_layout
        .direction(h_dir)
        .constraints(main_constraints)
        .split(area);
    let left = main[0];

    if pane_count == 0 {
        let empty_vec = ::std::vec::Vec::new();
        return (left, empty_vec);
    }

    let count_u32 = pane_count as ::core::primitive::u32;
    let ratio_constraint = ::ratatui::layout::Constraint::Ratio(1, count_u32);
    let constraints = ::std::vec![ratio_constraint; pane_count];
    let v_dir = ::ratatui::layout::Direction::Vertical;
    let right_layout = ::ratatui::layout::Layout::default();
    let right = right_layout
        .direction(v_dir)
        .constraints(constraints)
        .split(main[1]);
    let right_vec = right.to_vec();
    (left, right_vec)
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
            ::ratatui::layout::Constraint::Length(1),
        ])
        .split(inner);

    self::render_transcript_stream(frame, v_chunks[0], &app.messages);
    self::render_status_telemetry(frame, v_chunks[1], app);
    self::render_prompt_composer(frame, v_chunks[2], &app.input_buffer);

    if !app.dropdown_dismissed {
        let active = self::resolve_dropdown_items(&app.input_buffer, app.cursor_pos, &app.cards);
        if let ::core::option::Option::Some((kind, items)) = active {
            self::render_dropdown_popup(frame, v_chunks[2], kind, &items, app.dropdown_selected_idx);
        }
    }
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
            self::ChatMessage::User(user_text) => {
                let span_u_prompt = ::ratatui::text::Span::styled("❯ user: ", yellow_style);
                let span_u_text = ::ratatui::text::Span::styled(user_text, text_style);
                let span_u_vec = ::std::vec![span_u_prompt, span_u_text];
                let rendered_line = ::ratatui::text::Line::from(span_u_vec);
                lines.push(rendered_line);
            },
            self::ChatMessage::Orchestrator(orch_text) => {
                let span_o_icon = ::ratatui::text::Span::styled("  ✦ ", cyan_style);
                let span_o_text = ::ratatui::text::Span::styled(orch_text, text_style);
                let span_o_vec = ::std::vec![span_o_icon, span_o_text];
                let rendered_line = ::ratatui::text::Line::from(span_o_vec);
                lines.push(rendered_line);
            },
            self::ChatMessage::Transition {
                from,
                to,
            } => {
                let banner = ::std::format!("── State Transition: {from} ➔ {to} ──");
                let span_trans = ::ratatui::text::Span::styled(banner, magenta_style);
                let span_trans_vec = ::std::vec![span_trans];
                let rendered_line = ::ratatui::text::Line::from(span_trans_vec);
                lines.push(rendered_line);
            },
            self::ChatMessage::GatePrompt {
                gate,
                prompt,
            } => {
                let prompt_text = ::std::format!("  [GATE] {gate}: {prompt} (Type /approve to proceed)");
                let span_gate = ::ratatui::text::Span::styled(prompt_text, green_style);
                let span_gate_vec = ::std::vec![span_gate];
                let rendered_line = ::ratatui::text::Line::from(span_gate_vec);
                lines.push(rendered_line);
            },
            self::ChatMessage::Notice(notice_text) => {
                let span_n = ::ratatui::text::Span::styled(notice_text, muted_style);
                let span_n_vec = ::std::vec![span_n];
                let rendered_line = ::ratatui::text::Line::from(span_n_vec);
                lines.push(rendered_line);
            },
            self::ChatMessage::Error(error_text) => {
                let span_e = ::ratatui::text::Span::styled(error_text, red_style);
                let span_e_vec = ::std::vec![span_e];
                let rendered_line = ::ratatui::text::Line::from(span_e_vec);
                lines.push(rendered_line);
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

    let state_str = app.state_vector.macro_state.to_string();
    let gate_str = match &app.pending_gate {
        ::core::option::Option::Some(gate_name) => gate_name.to_string(),
        ::core::option::Option::None => "none".to_string(),
    };
    // The rendered state span derives from the persisted state vector, so the
    // UI never reads a duplicate state source.
    let telemetry_text = self::telemetry_state_text(&app.state_vector);
    ::std::debug_assert!(telemetry_text.starts_with(&state_str));

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

    let stream_count = app.cards.len();
    let stream_label = ::std::format!("👥 {stream_count} streams");
    let span_spec = ::ratatui::text::Span::styled(stream_label, yellow_style);

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
    let yellow_bold = ::ratatui::style::Style::default()
        .fg(::ratatui::style::Color::Yellow)
        .add_modifier(::ratatui::style::Modifier::BOLD);
    let text_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::White);
    let magic_style = ::ratatui::style::Style::default()
        .fg(::ratatui::style::Color::Magenta)
        .add_modifier(::ratatui::style::Modifier::BOLD);
    let cursor_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Cyan);

    let span_prompt = ::ratatui::text::Span::styled("❯ ", yellow_bold);
    let mut spans = ::std::vec![span_prompt];

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
    let span_cur = ::ratatui::text::Span::styled("▏", cursor_style);
    spans.push(span_cur);

    let prompt_line = ::ratatui::text::Line::from(spans);
    let para = ::ratatui::widgets::Paragraph::new(prompt_line);
    frame.render_widget(para, area);
}

/// Highlight tokens ultrathink, orchestrate, and workflowz.
pub fn highlight_magic_keywords(
    prompt: &::core::primitive::str,
) -> ::std::vec::Vec<(::std::string::String, ::core::primitive::bool)> {
    let mut result = ::std::vec::Vec::new();
    let words = prompt.split_whitespace();
    for word in words {
        let is_magic = word == "ultrathink" || word == "orchestrate" || word == "workflowz";
        let word_str = word.to_string();
        result.push((word_str, is_magic));
    }
    result
}

/// Resolve active dropdown suggestions based on input buffer cursor position.
pub fn resolve_dropdown_items(
    input_buffer: &::core::primitive::str,
    cursor_pos: ::core::primitive::usize,
    cards: &[self::SpecialistCardState],
) -> ::core::option::Option<(self::DropdownKind, ::std::vec::Vec<self::DropdownItem>)> {
    let bounded_pos = cursor_pos.min(input_buffer.len());
    let prefix = &input_buffer[..bounded_pos];

    let token_start = match prefix.rfind(' ') {
        ::core::option::Option::Some(idx) => idx + 1,
        ::core::option::Option::None => 0,
    };
    let current_token = &prefix[token_start..];

    if current_token.starts_with('/') {
        let commands = self::available_slash_commands();
        let matches = commands
            .into_iter()
            .filter(|item| item.token.starts_with(current_token))
            .collect::<::std::vec::Vec<self::DropdownItem>>();
        if matches.is_empty() {
            ::core::option::Option::None
        } else {
            ::core::option::Option::Some((self::DropdownKind::SlashCommand, matches))
        }
    } else if let ::core::option::Option::Some(query) = current_token.strip_prefix('@') {
        let mut matches = ::std::vec::Vec::new();
        for card in cards {
            if card.name.starts_with(query) {
                let token = ::std::format!("@{}", card.name);
                let description = card.status.clone();
                let item = self::DropdownItem {
                    token,
                    description,
                };
                matches.push(item);
            }
        }
        if matches.is_empty() {
            ::core::option::Option::None
        } else {
            ::core::option::Option::Some((self::DropdownKind::AgentMention, matches))
        }
    } else {
        ::core::option::Option::None
    }
}

/// Parse user reply text into wizard answer pairs.
///
/// Accepts `<id> = <value>` and `set <id> to <value>` forms. The value
/// spans everything after the separator so multi-word commands parse.
fn parse_answer_pairs(text: &::core::primitive::str) -> ::std::vec::Vec<(::std::string::String, ::std::string::String)> {
    let trimmed = text.trim();
    let set_suffix_res = trimmed
        .strip_prefix("set ")
        .and_then(|rest| rest.split_once(" to "));
    if let ::core::option::Option::Some((id, value)) = set_suffix_res {
        let id_text = id.trim().trim_matches('"').trim_matches('\'').to_string();
        let value_text = value.trim().trim_matches('"').trim_matches('\'').to_string();
        return ::std::vec![(id_text, value_text)];
    }
    let eq_res = trimmed.split_once('=');
    match eq_res {
        ::core::option::Option::Some((id, value)) => {
            let id_text = id.trim().to_string();
            let value_text = value.trim().trim_matches('"').trim_matches('\'').to_string();
            if id_text.is_empty() || value_text.is_empty() {
                return ::std::vec::Vec::new();
            }
            ::std::vec![(id_text, value_text)]
        },
        ::core::option::Option::None => ::std::vec::Vec::new(),
    }
}

/// Map legacy `/init` key aliases onto canonical wizard question ids.
fn normalize_answer_key(
    key: &::core::primitive::str,
    value: &::core::primitive::str,
) -> (::std::string::String, ::std::string::String) {
    let canonical = match key {
        "input" | "legacy" => "input_legacy_dir",
        "output" | "workspace" => "workspace_root",
        "lang" | "source_lang" => "source_language",
        "target" | "target_lang" => "target_language",
        "id" | "campaign" => "campaign_id",
        "mode" => "execution_mode",
        _ => key,
    };
    let mode_value = if key == "mode" && value == "auto" {
        "full_auto"
    } else {
        value
    };
    (canonical.to_string(), mode_value.to_string())
}

/// Map canonical campaign configuration onto the mock runner schema.
fn campaign_to_mock(cfg: &::configschema::CampaignConfig) -> self::mock::ModernizeConfig {
    let input_dir = cfg.paths.input_legacy_dir.to_string_lossy().to_string();
    let workspace_dir = cfg.paths.workspace_root.to_string_lossy().to_string();
    let source_language = { cfg.stack.source_language.clone() };
    let target_language = { cfg.stack.target_language.clone() };
    let campaign_id = { cfg.campaign_id.clone() };
    let execution_mode = match cfg.execution_mode {
        ::configschema::ExecutionMode::Hitl => self::mock::ExecutionMode::Hitl,
        ::configschema::ExecutionMode::FullAuto => self::mock::ExecutionMode::FullAuto,
    };
    let paths = self::mock::ModernizePaths {
        input_legacy_dir: { input_dir },
        workspace_root: { workspace_dir },
    };
    let stack = self::mock::ModernizeStack {
        source_language,
        target_language,
    };
    self::mock::ModernizeConfig {
        campaign_id,
        execution_mode,
        paths,
        stack,
    }
}

/// Return catalog of supported interactive slash commands.
pub fn available_slash_commands() -> ::std::vec::Vec<self::DropdownItem> {
    ::std::vec![
        self::DropdownItem {
            token: "/help".to_string(),
            description: "Display command manual".to_string(),
        },
        self::DropdownItem {
            token: "/init".to_string(),
            description: "Draft configuration from legacy input".to_string(),
        },
        self::DropdownItem {
            token: "/start".to_string(),
            description: "Execute modernization state machine".to_string(),
        },
        self::DropdownItem {
            token: "/approve".to_string(),
            description: "Approve pending human gate decision".to_string(),
        },
        self::DropdownItem {
            token: "/reject".to_string(),
            description: "Reject pending human gate with reason".to_string(),
        },
        self::DropdownItem {
            token: "/save-config".to_string(),
            description: "Serialize draft config to YAML".to_string(),
        },
        self::DropdownItem {
            token: "/load-config".to_string(),
            description: "Deserialize draft config from YAML".to_string(),
        },
        self::DropdownItem {
            token: "/status".to_string(),
            description: "Display telemetry and obligations".to_string(),
        },
        self::DropdownItem {
            token: "/clear".to_string(),
            description: "Clear orchestrator transcript".to_string(),
        },
        self::DropdownItem {
            token: "/quit".to_string(),
            description: "Terminate interactive cockpit session".to_string(),
        },
    ]
}

fn render_dropdown_popup(
    frame: &mut ::ratatui::Frame<'_>,
    prompt_area: ::ratatui::layout::Rect,
    kind: self::DropdownKind,
    items: &[self::DropdownItem],
    selected_idx: ::core::primitive::usize,
) {
    if items.is_empty() {
        return;
    }

    let max_visible = 6;
    let item_count = items.len();
    let visible_count = item_count.min(max_visible);
    let visible_u16 = visible_count as ::core::primitive::u16;
    let popup_height = visible_u16 + 2;
    let popup_width = prompt_area.width.min(54);
    let popup_y = prompt_area.y.saturating_sub(popup_height);
    let popup_x = prompt_area.x;
    let popup_rect = ::ratatui::layout::Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(::ratatui::widgets::Clear, popup_rect);

    let is_slash = kind == self::DropdownKind::SlashCommand;
    let title_str = if is_slash {
        " Slash Commands [Tab] "
    } else {
        " Mention Agent [Tab] "
    };

    let border_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Cyan);
    let default_block = ::ratatui::widgets::Block::default();
    let block = default_block
        .title(title_str)
        .borders(::ratatui::widgets::Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(popup_rect);
    frame.render_widget(block, popup_rect);

    let mut lines = ::std::vec::Vec::new();
    let last_idx = item_count.saturating_sub(1);
    let sel = selected_idx.min(last_idx);

    let start_idx = if sel >= max_visible {
        sel + 1 - max_visible
    } else {
        0
    };

    for (offset, item) in items.iter().skip(start_idx).take(max_visible).enumerate() {
        let current_idx = start_idx + offset;
        let is_selected = current_idx == sel;

        let style_default = ::ratatui::style::Style::default();
        let token_style = if is_selected {
            style_default
                .fg(::ratatui::style::Color::Black)
                .bg(::ratatui::style::Color::Cyan)
                .add_modifier(::ratatui::style::Modifier::BOLD)
        } else {
            style_default
                .fg(::ratatui::style::Color::Yellow)
                .add_modifier(::ratatui::style::Modifier::BOLD)
        };

        let desc_style = if is_selected {
            style_default
                .fg(::ratatui::style::Color::Black)
                .bg(::ratatui::style::Color::Cyan)
        } else {
            style_default.fg(::ratatui::style::Color::DarkGray)
        };

        let prefix = if is_selected { "❯ " } else { "  " };
        let span_pfx = ::ratatui::text::Span::styled(prefix, token_style);
        let span_tok = ::ratatui::text::Span::styled(&item.token, token_style);
        let span_sep = ::ratatui::text::Span::styled(" · ", desc_style);
        let span_desc = ::ratatui::text::Span::styled(&item.description, desc_style);

        let row = ::std::vec![span_pfx, span_tok, span_sep, span_desc];
        let line = ::ratatui::text::Line::from(row);
        lines.push(line);
    }

    let para = ::ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn render_specialist_stack(
    frame: &mut ::ratatui::Frame<'_>,
    right_panes: &[::ratatui::layout::Rect],
    cards: &[self::SpecialistCardState],
) {
    let margin = ::ratatui::layout::Margin {
        horizontal: 1,
        vertical: 0,
    };
    for (idx, &pane_rect) in right_panes.iter().enumerate() {
        if let ::core::option::Option::Some(card) = cards.get(idx) {
            let card_area = pane_rect.inner(margin);
            self::render_specialist_card(frame, card_area, card);
        }
    }
}

fn render_specialist_card(
    frame: &mut ::ratatui::Frame<'_>,
    area: ::ratatui::layout::Rect,
    card: &self::SpecialistCardState,
) {
    let title_style = ::ratatui::style::Style::default()
        .fg(card.color)
        .add_modifier(::ratatui::style::Modifier::BOLD);
    let border_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let white_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::White);

    let span_sym = ::ratatui::text::Span::styled(" ◈ ", title_style);
    let span_title = ::ratatui::text::Span::styled(&card.name, title_style);
    let span_open = ::ratatui::text::Span::styled(" [", border_style);
    let span_tel = ::ratatui::text::Span::styled(&card.telemetry, white_style);
    let span_close = ::ratatui::text::Span::styled("] ", border_style);

    let card_header_vec = ::std::vec![span_sym, span_title, span_open, span_tel, span_close];
    let header_line = ::ratatui::text::Line::from(card_header_vec);

    let default_block = ::ratatui::widgets::Block::default();
    let block = default_block
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
        let rendered_line = ::ratatui::text::Line::from(line_vec);
        lines.push(rendered_line);
    }

    let para = ::ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(para, inner);
}

impl ::core::default::Default for self::InteractiveCockpit {
    fn default() -> Self {
        Self::with_default_streams()
    }
}

impl self::InteractiveCockpit {
    /// Create default interactive cockpit with standard specialist streams.
    pub fn with_default_streams() -> Self {
        let streams = self::default_specialist_streams();
        Self::new(streams)
    }

    /// Create new interactive cockpit instance receiving a vector of streams.
    pub fn new(streams: ::std::vec::Vec<self::SpecialistCardState>) -> Self {
        let (tx, rx) = ::tokio::sync::mpsc::unbounded_channel();
        let default_config = self::mock::ModernizeConfig::default();
        let bb = self::mock::Blackboard::new();

        Self {
            input_buffer: ::std::string::String::new(),
            cursor_pos: 0,
            history: ::std::vec::Vec::new(),
            history_idx: ::core::option::Option::None,
            messages: ::std::vec::Vec::new(),
            cards: { streams },
            draft_config: { default_config },
            wizard: ::core::option::Option::None,
            blackboard: { bb },
            state_vector: self::mock::RunnerStateVector::default(),
            pending_gate: ::core::option::Option::None,
            is_running: false,
            event_tx: { tx },
            event_rx: { rx },
            gate_tx: ::core::option::Option::None,
            dropdown_selected_idx: 0,
            dropdown_dismissed: false,
            should_quit: false,
        }
    }

    /// Process user keystroke. Returns true if application should terminate.
    pub async fn handle_key_event(&mut self, key: ::crossterm::event::KeyEvent) -> ::core::primitive::bool {
        let dropdown_state = self::resolve_dropdown_items(&self.input_buffer, self.cursor_pos, &self.cards);
        let has_dropdown = dropdown_state.is_some() && !self.dropdown_dismissed;

        match key.code {
            ::crossterm::event::KeyCode::Esc => {
                if has_dropdown {
                    self.dropdown_dismissed = true;
                } else {
                    self.input_buffer.clear();
                    self.cursor_pos = 0;
                }
                false
            },
            ::crossterm::event::KeyCode::Char('c')
                if key.modifiers.contains(::crossterm::event::KeyModifiers::CONTROL) =>
            {
                true
            },
            ::crossterm::event::KeyCode::Tab => {
                if has_dropdown {
                    self.apply_dropdown_completion();
                }
                false
            },
            ::crossterm::event::KeyCode::Enter => {
                let trimmed = self.input_buffer.trim().to_string();
                if !trimmed.is_empty() {
                    self.history.push(trimmed.clone());
                    self.history_idx = ::core::option::Option::None;
                    self.messages.push(self::ChatMessage::User(trimmed.clone()));
                    self.input_buffer.clear();
                    self.cursor_pos = 0;
                    self.dropdown_dismissed = false;
                    self.dropdown_selected_idx = 0;
                    self.handle_user_input(&trimmed).await;
                }
                self.should_quit
            },
            ::crossterm::event::KeyCode::Backspace => {
                if self.cursor_pos > 0 && self.cursor_pos <= self.input_buffer.len() {
                    self.input_buffer.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                    self.dropdown_dismissed = false;
                    self.dropdown_selected_idx = 0;
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
                if has_dropdown {
                    if let ::core::option::Option::Some((_, items)) = dropdown_state {
                        let total = items.len();
                        if total > 0 {
                            if self.dropdown_selected_idx > 0 {
                                self.dropdown_selected_idx -= 1;
                            } else {
                                self.dropdown_selected_idx = total.saturating_sub(1);
                            }
                        }
                    }
                } else if !self.history.is_empty() {
                    let next_idx = match self.history_idx {
                        ::core::option::Option::Some(hist_idx) if hist_idx > 0 => hist_idx - 1,
                        ::core::option::Option::Some(_) => 0,
                        ::core::option::Option::None => self.history.len() - 1,
                    };
                    self.history_idx = ::core::option::Option::Some(next_idx);
                    let item = self.history[next_idx].clone();
                    self.input_buffer = item;
                    self.cursor_pos = self.input_buffer.len();
                    self.dropdown_dismissed = false;
                }
                false
            },
            ::crossterm::event::KeyCode::Down => {
                if has_dropdown {
                    if let ::core::option::Option::Some((_, items)) = dropdown_state {
                        let total = items.len();
                        if total > 0 {
                            self.dropdown_selected_idx = (self.dropdown_selected_idx + 1) % total;
                        }
                    }
                } else if let ::core::option::Option::Some(idx) = self.history_idx {
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
                    self.dropdown_dismissed = false;
                }
                false
            },
            ::crossterm::event::KeyCode::Char(character) => {
                if self.cursor_pos <= self.input_buffer.len() {
                    self.input_buffer.insert(self.cursor_pos, character);
                    self.cursor_pos += 1;
                    self.dropdown_dismissed = false;
                    self.dropdown_selected_idx = 0;
                }
                false
            },
            _ => false,
        }
    }

    /// Apply active dropdown completion into input buffer.
    pub fn apply_dropdown_completion(&mut self) -> ::core::primitive::bool {
        let active = self::resolve_dropdown_items(&self.input_buffer, self.cursor_pos, &self.cards);
        if let ::core::option::Option::Some((_kind, items)) = active {
            if !items.is_empty() {
                let last_idx = items.len().saturating_sub(1);
                let idx = self.dropdown_selected_idx.min(last_idx);
                let completed_token = &items[idx].token;

                let bounded_pos = self.cursor_pos.min(self.input_buffer.len());
                let prefix = &self.input_buffer[..bounded_pos];
                let token_start = match prefix.rfind(' ') {
                    ::core::option::Option::Some(pos) => pos + 1,
                    ::core::option::Option::None => 0,
                };
                let suffix = &self.input_buffer[bounded_pos..];

                let mut new_buffer = ::std::string::String::new();
                let head = &self.input_buffer[..token_start];
                new_buffer.push_str(head);
                new_buffer.push_str(completed_token);
                new_buffer.push(' ');
                let new_cursor = new_buffer.len();
                new_buffer.push_str(suffix);

                self.input_buffer = new_buffer;
                self.cursor_pos = new_cursor;
                self.dropdown_selected_idx = 0;
                return true;
            }
        }
        false
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
                self.messages.push(self::ChatMessage::Notice(
                    "  /init: Run the configuration wizard (no args) or set fields (key=value args).".to_string(),
                ));
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
                self.messages.push(self::ChatMessage::Notice("  /quit: Exit application.".to_string()));
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
                let wizard_mode = self
                    .wizard
                    .as_ref()
                    .map(|agent| -> self::mock::ExecutionMode {
                        match agent.draft.execution_mode {
                            ::configschema::ExecutionMode::FullAuto => self::mock::ExecutionMode::FullAuto,
                            ::configschema::ExecutionMode::Hitl => self::mock::ExecutionMode::Hitl,
                        }
                    });
                let mode = if is_auto {
                    self::mock::ExecutionMode::FullAuto
                } else if is_hitl {
                    self::mock::ExecutionMode::Hitl
                } else {
                    wizard_mode.unwrap_or(self.draft_config.execution_mode)
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
                let st = self.state_vector.macro_state;
                let is_run = self.is_running;
                let cov = self.blackboard.obligation_coverage().await;
                let taint = self.blackboard.is_tainted().await;
                let cards_len = self.blackboard.rule_cards().await.len();
                let gate_str = match &self.pending_gate {
                    ::core::option::Option::Some(gate_name) => gate_name.to_string(),
                    ::core::option::Option::None => "None".to_string(),
                };
                let status_msg = ::std::format!(
                    "State: {st} | Running: {is_run} | Pending Gate: {gate_str} | REA Coverage: {cov:.1}% | Cards: {cards_len} | Tainted: {taint}"
                );
                self.messages.push(self::ChatMessage::Notice(status_msg));
            },
            "/quit" | "/exit" => {
                self.messages.push(self::ChatMessage::Notice("Terminating session.".to_string()));
                self.should_quit = true;
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
                    let outcome = self::mock::StepOutcome::Advanced {
                        from: ::core::clone::Clone::clone(&from),
                        to: ::core::clone::Clone::clone(&to),
                    };
                    self.state_vector.record(&outcome);
                    self.messages.push(self::ChatMessage::Transition {
                        from,
                        to,
                    });
                },
                self::UiStreamEvent::GateRequired {
                    gate,
                    prompt,
                } => {
                    let outcome = self::mock::StepOutcome::GateRequired {
                        gate,
                        prompt: ::core::clone::Clone::clone(&prompt),
                    };
                    self.state_vector.record(&outcome);
                    self.pending_gate = ::core::option::Option::Some(gate);
                    self.messages.push(self::ChatMessage::GatePrompt {
                        gate,
                        prompt,
                    });
                },
                self::UiStreamEvent::WorkflowCompleted {
                    receipts,
                } => {
                    let outcome = self::mock::StepOutcome::Completed {
                        receipts: ::core::clone::Clone::clone(&receipts),
                    };
                    self.state_vector.record(&outcome);
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

    /// Process `/init` arguments to drive the configurator wizard.
    fn handle_init_command(&mut self, args: &[&::core::primitive::str]) {
        let mut agent = ::configurator::ConfiguratorAgent::builder().build();
        for arg in args {
            if let ::core::option::Option::Some((k, v)) = arg.split_once('=') {
                let (id, value) = self::normalize_answer_key(k, v);
                agent.apply_answer(&id, &value);
            }
        }

        if args.is_empty() {
            let script = agent.question_script();
            let mut bridge = self::TuiAskBridge::new();
            let answers = bridge.ask(script);
            for msg in bridge.emitted {
                self.messages.push(msg);
            }
            for answer in answers {
                agent.apply_answer(&answer.id, &answer.value);
            }
            self.messages.push(self::ChatMessage::Orchestrator(
                "Wizard complete. Reply with `<id> = <value>` to adjust any field.".to_string(),
            ));
        } else {
            self.messages.push(self::ChatMessage::Orchestrator(
                "I prepared the modernization configuration draft:".to_string(),
            ));
        }

        self.wizard = ::core::option::Option::Some(agent);
        let draft_yaml = self
            .wizard
            .as_ref()
            .map(|agent| -> ::std::string::String { agent.draft.to_yaml_text() })
            .unwrap_or_default();
        for line in draft_yaml.lines() {
            let indented = ::std::format!("  {line}");
            self.messages.push(self::ChatMessage::Orchestrator(indented));
        }

        self.messages.push(self::ChatMessage::Orchestrator(
            "Type adjustments (e.g. `campaign_id = my-campaign`), or `/save-config`, then `/start`.".to_string(),
        ));
    }

    /// Converse with agent to update the wizard draft configuration.
    fn converse_with_agent(&mut self, text: &::core::primitive::str) {
        if self.wizard.is_none() {
            self.converse_legacy_keywords(text);
            return;
        }
        let pairs = self::parse_answer_pairs(text);
        if pairs.is_empty() {
            self.converse_without_wizard(text);
            return;
        }
        let draft_yaml = {
            let agent_res = self.wizard.as_mut();
            let agent = match agent_res {
                ::core::option::Option::Some(agent) => agent,
                ::core::option::Option::None => return,
            };
            for (id, value) in pairs {
                agent.apply_answer(&id, &value);
            }
            agent.draft.to_yaml_text()
        };
        self.messages.push(self::ChatMessage::Orchestrator(
            "Updated wizard draft configuration:".to_string(),
        ));
        for line in draft_yaml.lines() {
            let indented = ::std::format!("  {line}");
            self.messages.push(self::ChatMessage::Orchestrator(indented));
        }
        self.messages.push(self::ChatMessage::Orchestrator(
            "Type /save-config to save to disk, or /start to start.".to_string(),
        ));
    }

    /// Apply legacy keyword parsing onto the mock draft configuration.
    fn converse_legacy_keywords(&mut self, text: &::core::primitive::str) {
        let lower = text.to_lowercase();
        let mut modified = false;

        if lower.contains("input") || lower.contains("legacy") {
            let words = text.split_whitespace().collect::<::std::vec::Vec<&::core::primitive::str>>();
            if let ::core::option::Option::Some(pos) = words.iter().position(|&word| word == "to" || word == "=") {
                if pos + 1 < words.len() {
                    let path = words[pos + 1].trim_matches('"').trim_matches('\'');
                    self.draft_config.paths.input_legacy_dir = path.to_string();
                    modified = true;
                }
            }
        }

        if lower.contains("workspace") || lower.contains("output") {
            let words = text.split_whitespace().collect::<::std::vec::Vec<&::core::primitive::str>>();
            if let ::core::option::Option::Some(pos) = words.iter().position(|&word| word == "to" || word == "=") {
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
            self.draft_config.execution_mode = self::mock::ExecutionMode::FullAuto;
            modified = true;
        } else if lower.contains("hitl") {
            self.draft_config.execution_mode = self::mock::ExecutionMode::Hitl;
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
            self.converse_without_wizard(text);
        }
    }

    /// Reply without an active wizard draft.
    fn converse_without_wizard(&mut self, text: &::core::primitive::str) {
        let reply = ::std::format!(
            "Understood: \"{text}\". Tracking campaign `{}` (target: {}). Type /init to edit parameters or /start to start.",
            self.draft_config.campaign_id,
            self.draft_config.stack.target_language
        );
        self.messages.push(self::ChatMessage::Orchestrator(reply));
    }

    /// Save configuration draft to file path.
    fn save_config_to_path(&mut self, path: &::core::primitive::str) {
        let content = match &self.wizard {
            ::core::option::Option::Some(agent) => agent.draft.to_yaml_text(),
            ::core::option::Option::None => {
                let yaml_res = ::serde_yaml_ng::to_string(&self.draft_config);
                match yaml_res {
                    ::core::result::Result::Ok(yaml) => yaml,
                    ::core::result::Result::Err(err) => {
                        let err_msg = ::std::format!("Failed to serialize YAML: {err}");
                        self.messages.push(self::ChatMessage::Error(err_msg));
                        return;
                    },
                }
            },
        };
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
    }

    /// Load configuration from file path.
    fn load_config_from_path(&mut self, path: &::core::primitive::str) {
        let read_res = ::std::fs::read_to_string(path);
        match read_res {
            ::core::result::Result::Ok(content) => {
                let parse_res = ::serde_yaml_ng::from_str::<self::mock::ModernizeConfig>(&content);
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
    fn start_modernization(&mut self, mode: self::mock::ExecutionMode) {
        if let ::core::option::Option::Some(agent) = &self.wizard {
            let synced = self::campaign_to_mock(&agent.draft);
            self.draft_config = synced;
        }
        if self.is_running {
            self.messages.push(self::ChatMessage::Error("Modernization workflow is already running.".to_string()));
            return;
        }

        self.draft_config.execution_mode = mode;
        let start_msg = ::std::format!(
            "▶ Starting modernization workflow for campaign `{}` (mode: {:?}).",
            self.draft_config.campaign_id,
            mode
        );
        self.messages.push(self::ChatMessage::Notice(start_msg));
        self.is_running = true;

        let (gate_tx, mut gate_rx) = ::tokio::sync::mpsc::unbounded_channel::<self::mock::GateDecision>();
        self.gate_tx = ::core::option::Option::Some(gate_tx);

        let ui_tx = self.event_tx.clone();
        let cfg = self.draft_config.clone();
        let bb = self.blackboard.clone();

        ::tokio::spawn(async move {
            let runner = self::mock::StateMachineRunner::new(cfg);
            let mut sub = bb.subscribe();

            let ui_tx_sub = ui_tx.clone();
            ::tokio::spawn(async move {
                while let ::core::result::Result::Ok(ev) = sub.recv().await {
                    match ev {
                        self::mock::BlackboardEvent::ObligationAdded {
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
                        self::mock::BlackboardEvent::RuleCardAdded {
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
                        self::mock::BlackboardEvent::ModuleWorktreeUpdated {
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
                        self::mock::BlackboardEvent::TaintDetected {
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
                    }
                }
            });

            let (assess_runner, first_outcome) = runner.step();
            if let self::mock::StepOutcome::Advanced { from, to } = first_outcome {
                let _ = ui_tx.send(self::UiStreamEvent::StateTransition { from, to });
                let log_line = ::std::format!("FSM transition: {from} -> {to}");
                let _ = ui_tx.send(self::UiStreamEvent::OrchestratorLine(log_line));
                let _ = ui_tx.send(self::UiStreamEvent::SpecialistLog {
                    index: 0,
                    line: ::std::format!("State advanced to {to}"),
                    telemetry: ::core::option::Option::Some(::std::format!("Active in {to}")),
                });
            }
            let dur = ::core::time::Duration::from_millis(50);
            ::tokio::time::sleep(dur).await;

            if assess_runner.is_auto() {
                let (approved_runner, advanced) = assess_runner.advance_auto();
                if let self::mock::StepOutcome::Advanced { from, to } = ::core::clone::Clone::clone(&advanced) {
                    let _ = ui_tx.send(self::UiStreamEvent::StateTransition { from, to });
                    let log_line = ::std::format!("FSM transition: {from} -> {to}");
                    let _ = ui_tx.send(self::UiStreamEvent::OrchestratorLine(log_line));
                }
                let (transform_runner, transform_outcome) = approved_runner.step();
                if let self::mock::StepOutcome::Advanced { from, to } = ::core::clone::Clone::clone(&transform_outcome) {
                    let _ = ui_tx.send(self::UiStreamEvent::StateTransition { from, to });
                    let log_line = ::std::format!("FSM transition: {from} -> {to}");
                    let _ = ui_tx.send(self::UiStreamEvent::OrchestratorLine(log_line));
                }
                let (verify_runner, verify_outcome) = transform_runner.step();
                if let self::mock::StepOutcome::Advanced { from, to } = ::core::clone::Clone::clone(&verify_outcome) {
                    let _ = ui_tx.send(self::UiStreamEvent::StateTransition { from, to });
                    let log_line = ::std::format!("FSM transition: {from} -> {to}");
                    let _ = ui_tx.send(self::UiStreamEvent::OrchestratorLine(log_line));
                }
                let completion = verify_runner.step();
                if let self::mock::StepOutcome::Completed { receipts } = completion {
                    let _ = ui_tx.send(self::UiStreamEvent::WorkflowCompleted { receipts });
                }
            } else {
                let gate_outcome = assess_runner.park_on_gate();
                if let self::mock::StepOutcome::GateRequired { gate, prompt } = gate_outcome {
                    let _ = ui_tx.send(self::UiStreamEvent::GateRequired { gate, prompt });
                }
                let decision_opt = gate_rx.recv().await;
                let decision = match decision_opt {
                    ::core::option::Option::Some(pending) => pending,
                    ::core::option::Option::None => {
                        let closed = "Gate channel closed".to_string();
                        let _ = ui_tx.send(self::UiStreamEvent::WorkflowFailed(closed));
                        return;
                    }
                };
                match assess_runner.submit_gate_decision(decision) {
                    ::core::result::Result::Ok((approved_runner, approved_outcome)) => {
                        if let self::mock::StepOutcome::Advanced { from, to } =
                            ::core::clone::Clone::clone(&approved_outcome)
                        {
                            let _ = ui_tx.send(self::UiStreamEvent::StateTransition { from, to });
                            let log_line = ::std::format!("FSM transition: {from} -> {to}");
                            let _ = ui_tx.send(self::UiStreamEvent::OrchestratorLine(log_line));
                        }
                        let (transform_runner, transform_outcome) = approved_runner.step();
                        if let self::mock::StepOutcome::Advanced { from, to } =
                            ::core::clone::Clone::clone(&transform_outcome)
                        {
                            let _ = ui_tx.send(self::UiStreamEvent::StateTransition { from, to });
                            let log_line = ::std::format!("FSM transition: {from} -> {to}");
                            let _ = ui_tx.send(self::UiStreamEvent::OrchestratorLine(log_line));
                        }
                        let (verify_runner, verify_outcome) = transform_runner.step();
                        if let self::mock::StepOutcome::Advanced { from, to } =
                            ::core::clone::Clone::clone(&verify_outcome)
                        {
                            let _ = ui_tx.send(self::UiStreamEvent::StateTransition { from, to });
                            let log_line = ::std::format!("FSM transition: {from} -> {to}");
                            let _ = ui_tx.send(self::UiStreamEvent::OrchestratorLine(log_line));
                        }
                        let completion = verify_runner.step();
                        if let self::mock::StepOutcome::Completed { receipts } = completion {
                            let _ = ui_tx.send(self::UiStreamEvent::WorkflowCompleted { receipts });
                        }
                    }
                    ::core::result::Result::Err(err) => {
                        let _ = ui_tx.send(self::UiStreamEvent::WorkflowFailed(err));
                    }
                }
            }
        });
    }

    /// Resolve pending gate with approval decision.
    fn resolve_gate_approval(&mut self, notes: &::core::primitive::str) {
        if let ::core::option::Option::Some(gate) = self.pending_gate.take() {
            if let ::core::option::Option::Some(tx) = &self.gate_tx {
                let decision = self::mock::GateDecision::Approved {
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
                let decision = self::mock::GateDecision::Rejected {
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

/// Build the status telemetry text from the persisted state vector. The
/// rendering layer consumes this single source of truth (VAL-INTERACTIVE-01).
pub fn telemetry_state_text(state_vector: &self::mock::RunnerStateVector) -> ::std::string::String {
    let state_text = state_vector.macro_state.to_string();
    let event_count = state_vector.event_log.len();
    ::std::format!("{state_text} | events: {event_count}")
}
