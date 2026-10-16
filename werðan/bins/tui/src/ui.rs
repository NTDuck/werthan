//! User interface rendering widgets for the terminal dashboard.

/// Draw the complete multi-pane layout to the provided terminal frame.
pub fn draw(state: &crate::app::AppState, frame: &mut ::ratatui::Frame<'_>) {
    let frame_area = frame.area();
    let root_layout = ::ratatui::layout::Layout::default();
    let root_dir = root_layout.direction(::ratatui::layout::Direction::Vertical);
    let root_constraints = [::ratatui::layout::Constraint::Length(1), ::ratatui::layout::Constraint::Min(1)];
    let root_chunks = root_dir.constraints(root_constraints).split(frame_area);

    render_status_bar(state, frame, root_chunks[0]);

    let main_area = root_chunks[1];
    let h_layout = ::ratatui::layout::Layout::default();
    let h_dir = h_layout.direction(::ratatui::layout::Direction::Horizontal);
    let h_constraints = [
        ::ratatui::layout::Constraint::Percentage(70),
        ::ratatui::layout::Constraint::Length(1),
        ::ratatui::layout::Constraint::Min(0),
    ];
    let main_chunks = h_dir.constraints(h_constraints).split(main_area);

    render_master(state, frame, main_chunks[0]);
    render_separator(frame, main_chunks[1]);
    render_workers(state, frame, main_chunks[2]);
}

fn render_status_bar(state: &crate::app::AppState, frame: &mut ::ratatui::Frame<'_>, area: ::ratatui::layout::Rect) {
    let model_id = state.orchestrator_model_id();
    let provider = state.orchestrator_provider();
    let endpoint = state.config.base_url.as_str();
    let status = state.status.as_str();

    let dot_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Green);
    let brand_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::White);
    let dim_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let val_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::Cyan);
    let text_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::White);
    let status_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Yellow);

    let model_text = ::std::format!("{model_id} ({provider})");

    let spans = ::std::vec![
        ::ratatui::text::Span::styled(" ● ", dot_style),
        ::ratatui::text::Span::styled("werðan", brand_style),
        ::ratatui::text::Span::styled(" │ model: ", dim_style),
        ::ratatui::text::Span::styled(model_text, val_style),
        ::ratatui::text::Span::styled(" │ endpoint: ", dim_style),
        ::ratatui::text::Span::styled(endpoint, text_style),
        ::ratatui::text::Span::styled(" │ status: ", dim_style),
        ::ratatui::text::Span::styled(status, status_style),
    ];

    let line = ::ratatui::text::Line::from(spans);
    let bar_style = ::ratatui::style::Style::default()
        .bg(::ratatui::style::Color::Rgb(24, 25, 30))
        .fg(::ratatui::style::Color::White);
    let para = ::ratatui::widgets::Paragraph::new(line).style(bar_style);
    frame.render_widget(para, area);
}

fn render_master(state: &crate::app::AppState, frame: &mut ::ratatui::Frame<'_>, area: ::ratatui::layout::Rect) {
    let v_layout = ::ratatui::layout::Layout::default();
    let v_dir = v_layout.direction(::ratatui::layout::Direction::Vertical);
    let v_constraints = [
        ::ratatui::layout::Constraint::Length(2),
        ::ratatui::layout::Constraint::Min(3),
        ::ratatui::layout::Constraint::Length(1),
        ::ratatui::layout::Constraint::Length(1),
        ::ratatui::layout::Constraint::Length(1),
    ];
    let chunks = v_dir.constraints(v_constraints).split(area);

    let header_area = chunks[0];
    let messages_area = chunks[1];
    let div_area = chunks[2];
    let input_area = chunks[3];
    let help_area = chunks[4];

    render_window_header(frame, header_area, "orchestrator (rig)", ::ratatui::style::Color::Cyan);
    render_messages(state, frame, messages_area);

    let div_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let div_len = area.width as ::core::primitive::usize;
    let div_bar = "─".repeat(div_len);
    let div_span = ::ratatui::text::Span::styled(div_bar, div_style);
    let div_line = ::ratatui::text::Line::from(div_span);
    let div_para = ::ratatui::widgets::Paragraph::new(div_line);
    frame.render_widget(div_para, div_area);

    render_prompt_input(state, frame, input_area, help_area);
}

fn render_separator(frame: &mut ::ratatui::Frame<'_>, area: ::ratatui::layout::Rect) {
    let height = area.height as ::core::primitive::usize;
    let sep_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let mut lines = ::std::vec::Vec::new();
    for _ in 0..height {
        let span = ::ratatui::text::Span::styled("│", sep_style);
        let line = ::ratatui::text::Line::from(span);
        lines.push(line);
    }
    let para = ::ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(para, area);
}

fn render_workers(state: &crate::app::AppState, frame: &mut ::ratatui::Frame<'_>, area: ::ratatui::layout::Rect) {
    let count = state.workers.len();
    if count == 0 {
        return;
    }
    let v_layout = ::ratatui::layout::Layout::default();
    let v_dir = v_layout.direction(::ratatui::layout::Direction::Vertical);
    let ratio = count as ::core::primitive::u32;
    let constraints = ::std::vec![::ratatui::layout::Constraint::Ratio(1, ratio); count];
    let chunks = v_dir.constraints(constraints).split(area);

    let colors = [
        ::ratatui::style::Color::Yellow,
        ::ratatui::style::Color::Blue,
        ::ratatui::style::Color::Magenta,
        ::ratatui::style::Color::Cyan,
        ::ratatui::style::Color::Green,
    ];

    for (idx, worker) in state.workers.iter().enumerate() {
        let title = ::std::format!("worker ({})", worker.name);
        let color = colors[idx % colors.len()];
        render_worker_slice(frame, chunks[idx], &title, color, &worker.output);
    }
}

fn render_worker_slice(
    frame: &mut ::ratatui::Frame<'_>,
    area: ::ratatui::layout::Rect,
    title: &::core::primitive::str,
    color: ::ratatui::style::Color,
    output: &::core::primitive::str,
) {
    let v_layout = ::ratatui::layout::Layout::default();
    let v_dir = v_layout.direction(::ratatui::layout::Direction::Vertical);
    let v_constraints = [::ratatui::layout::Constraint::Length(2), ::ratatui::layout::Constraint::Min(1)];
    let chunks = v_dir.constraints(v_constraints).split(area);

    render_window_header(frame, chunks[0], title, color);
    render_worker_output(frame, chunks[1], output);
}

fn render_window_header(
    frame: &mut ::ratatui::Frame<'_>,
    area: ::ratatui::layout::Rect,
    title: &::core::primitive::str,
    color: ::ratatui::style::Color,
) {
    let title_style = ::ratatui::style::Style::default().bold().fg(color);
    let icon_span = ::ratatui::text::Span::styled("◈ ", title_style);
    let title_span = ::ratatui::text::Span::styled(title, title_style);
    let header_line = ::ratatui::text::Line::from(::std::vec![icon_span, title_span]);

    let div_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let div_len = area.width as ::core::primitive::usize;
    let div_bar = "─".repeat(div_len);
    let div_span = ::ratatui::text::Span::styled(div_bar, div_style);
    let div_line = ::ratatui::text::Line::from(div_span);

    let para = ::ratatui::widgets::Paragraph::new(::std::vec![header_line, div_line]);
    frame.render_widget(para, area);
}

fn render_messages(state: &crate::app::AppState, frame: &mut ::ratatui::Frame<'_>, area: ::ratatui::layout::Rect) {
    let items = format_message_items(&state.master_messages);
    let list = ::ratatui::widgets::List::new(items);
    frame.render_widget(list, area);
}

fn format_message_items<'a>(
    messages: &'a [::std::string::String],
) -> ::std::vec::Vec<::ratatui::widgets::ListItem<'a>> {
    let mut items = ::std::vec::Vec::new();
    for msg in messages {
        let raw = msg.as_str();
        if let ::core::option::Option::Some(prompt) = raw.strip_prefix("User: ") {
            let prefix_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::Cyan);
            let text_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::White);
            let user_line = ::ratatui::text::Line::from(::std::vec![
                ::ratatui::text::Span::styled("❯ User: ", prefix_style),
                ::ratatui::text::Span::styled(prompt, text_style),
            ]);
            let blank = ::ratatui::text::Line::from("");
            let item = ::ratatui::widgets::ListItem::new(::std::vec![user_line, blank]);
            items.push(item);
        } else if raw.starts_with("Master: ") {
            let sys_style = ::ratatui::style::Style::default().italic().fg(::ratatui::style::Color::Yellow);
            let sys_line = ::ratatui::text::Line::from(::std::vec![
                ::ratatui::text::Span::styled("ℹ ", sys_style),
                ::ratatui::text::Span::styled(raw, sys_style),
            ]);
            let blank = ::ratatui::text::Line::from("");
            let item = ::ratatui::widgets::ListItem::new(::std::vec![sys_line, blank]);
            items.push(item);
        } else {
            let orch_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::Green);
            let text_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::White);
            let orch_span = ::ratatui::text::Span::styled("✦ Orchestrator:", orch_style);
            let header_line = ::ratatui::text::Line::from(orch_span);
            let content_line = ::ratatui::text::Line::from(::std::vec![
                ::ratatui::text::Span::raw("  "),
                ::ratatui::text::Span::styled(raw, text_style),
            ]);
            let blank = ::ratatui::text::Line::from("");
            let item = ::ratatui::widgets::ListItem::new(::std::vec![header_line, content_line, blank]);
            items.push(item);
        }
    }
    items
}

fn render_prompt_input(
    state: &crate::app::AppState,
    frame: &mut ::ratatui::Frame<'_>,
    input_area: ::ratatui::layout::Rect,
    help_area: ::ratatui::layout::Rect,
) {
    let prefix_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::Green);
    let input_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::White);
    let cursor_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Green);

    let prefix_span = ::ratatui::text::Span::styled("$ ", prefix_style);
    let input_span = ::ratatui::text::Span::styled(state.input.as_str(), input_style);
    let cursor_span = ::ratatui::text::Span::styled("█", cursor_style);

    let input_line = ::ratatui::text::Line::from(::std::vec![prefix_span, input_span, cursor_span]);
    let input_para = ::ratatui::widgets::Paragraph::new(input_line);
    frame.render_widget(input_para, input_area);

    let help_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let help_text = "  (Enter to submit, Esc or Ctrl+C to quit)";
    let help_span = ::ratatui::text::Span::styled(help_text, help_style);
    let help_line = ::ratatui::text::Line::from(help_span);
    let help_para = ::ratatui::widgets::Paragraph::new(help_line);
    frame.render_widget(help_para, help_area);
}

fn render_worker_output(
    frame: &mut ::ratatui::Frame<'_>,
    area: ::ratatui::layout::Rect,
    output: &::core::primitive::str,
) {
    if output == "Idle" {
        let idle_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
        let idle_span = ::ratatui::text::Span::styled("  · idle", idle_style);
        let idle_line = ::ratatui::text::Line::from(idle_span);
        let para = ::ratatui::widgets::Paragraph::new(idle_line);
        frame.render_widget(para, area);
        return;
    }

    let badge_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::Yellow);
    let text_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::White);

    let mut lines = ::std::vec::Vec::new();
    let badge_span = ::ratatui::text::Span::styled("  ● response:", badge_style);
    let badge_line = ::ratatui::text::Line::from(badge_span);
    lines.push(badge_line);

    for out_line in output.lines() {
        let span = ::ratatui::text::Span::styled(out_line, text_style);
        let line = ::ratatui::text::Line::from(::std::vec![::ratatui::text::Span::raw("    "), span,]);
        lines.push(line);
    }

    let wrap = ::ratatui::widgets::Wrap {
        trim: false,
    };
    let para = ::ratatui::widgets::Paragraph::new(lines).wrap(wrap);
    frame.render_widget(para, area);
}
