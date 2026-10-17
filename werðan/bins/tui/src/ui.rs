//! User interface rendering widgets for the terminal dashboard.

/// Draw the complete multi-pane layout to the provided terminal frame.
pub fn draw(state: &crate::app::AppState, frame: &mut ::ratatui::Frame<'_>) {
    let frame_area = frame.area();
    let margin_area = ::ratatui::layout::Rect {
        x: { frame_area.x + 2 },
        y: { frame_area.y + 1 },
        width: { frame_area.width.saturating_sub(4) },
        height: { frame_area.height.saturating_sub(2) },
    };

    let h_layout = ::ratatui::layout::Layout::default();
    let h_dir = h_layout.direction(::ratatui::layout::Direction::Horizontal);
    let h_constraints = [
        ::ratatui::layout::Constraint::Percentage(58),
        ::ratatui::layout::Constraint::Length(1),
        ::ratatui::layout::Constraint::Min(0),
    ];
    let main_chunks = h_dir.constraints(h_constraints).split(margin_area);

    render_left_window(state, frame, main_chunks[0]);
    render_separator(frame, main_chunks[1]);
    render_agents(state, frame, main_chunks[2]);
}

fn render_left_window(state: &crate::app::AppState, frame: &mut ::ratatui::Frame<'_>, area: ::ratatui::layout::Rect) {
    let border_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let title_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::Cyan);
    let title_span = ::ratatui::text::Span::styled(" ✦ orchestrator (council) ", title_style);
    let title_line = ::ratatui::text::Line::from(::std::vec![title_span]);

    let outer_block = ::ratatui::widgets::Block::bordered()
        .border_type(::ratatui::widgets::BorderType::Rounded)
        .border_style(border_style)
        .title(title_line);

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let v_layout = ::ratatui::layout::Layout::default();
    let v_dir = v_layout.direction(::ratatui::layout::Direction::Vertical);
    let v_constraints = [
        ::ratatui::layout::Constraint::Min(3),
        ::ratatui::layout::Constraint::Length(1),
        ::ratatui::layout::Constraint::Length(1),
    ];
    let chunks = v_dir.constraints(v_constraints).split(inner_area);

    render_messages(state, frame, chunks[0]);
    render_status_bar(state, frame, chunks[1]);
    render_unboxed_prompt(state, frame, chunks[2]);
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
                ::ratatui::text::Span::styled("❯ user: ", prefix_style),
                ::ratatui::text::Span::styled(prompt, text_style),
            ]);
            let blank = ::ratatui::text::Line::from("");
            let item = ::ratatui::widgets::ListItem::new(::std::vec![user_line, blank]);
            items.push(item);
        } else if raw.starts_with("Master: ") {
            let sys_style = ::ratatui::style::Style::default().italic().fg(::ratatui::style::Color::DarkGray);
            let sys_line = ::ratatui::text::Line::from(::std::vec![
                ::ratatui::text::Span::styled("ℹ ", sys_style),
                ::ratatui::text::Span::styled(raw, sys_style),
            ]);
            let blank = ::ratatui::text::Line::from("");
            let item = ::ratatui::widgets::ListItem::new(::std::vec![sys_line, blank]);
            items.push(item);
        } else if raw.contains("<tool_call>") {
            render_tool_block(raw, &mut items);
        } else if raw.contains("<think>") || raw.starts_with("✻ ") {
            render_thought_block(raw, &mut items);
        } else {
            render_text_block(raw, &mut items);
        }
    }
    items
}

fn render_tool_block<'a>(
    raw: &'a ::core::primitive::str,
    items: &mut ::std::vec::Vec<::ratatui::widgets::ListItem<'a>>,
) {
    let tool_head_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::Cyan);
    let rail_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let tool_code_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Rgb(156, 220, 254));

    let header_span = ::ratatui::text::Span::styled("⇶ tool execution [call]", tool_head_style);
    let header_line = ::ratatui::text::Line::from(header_span);
    let mut lines = ::std::vec![header_line];
    for line_str in raw.lines() {
        let content_line = ::ratatui::text::Line::from(::std::vec![
            ::ratatui::text::Span::styled("  │ ", rail_style),
            ::ratatui::text::Span::styled(line_str, tool_code_style),
        ]);
        lines.push(content_line);
    }
    let blank_line = ::ratatui::text::Line::from("");
    lines.push(blank_line);
    let item = ::ratatui::widgets::ListItem::new(lines);
    items.push(item);
}

fn render_thought_block<'a>(
    raw: &'a ::core::primitive::str,
    items: &mut ::std::vec::Vec<::ratatui::widgets::ListItem<'a>>,
) {
    let think_head_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Rgb(119, 125, 136));
    let think_body_style = ::ratatui::style::Style::default().italic().fg(::ratatui::style::Color::Rgb(119, 125, 136));

    let header_span = ::ratatui::text::Span::styled("✻ Thought for deliberation", think_head_style);
    let header_line = ::ratatui::text::Line::from(header_span);
    let mut lines = ::std::vec![header_line];
    for line_str in raw.lines() {
        let content_line = ::ratatui::text::Line::from(::std::vec![
            ::ratatui::text::Span::raw("  "),
            ::ratatui::text::Span::styled(line_str, think_body_style),
        ]);
        lines.push(content_line);
    }
    let blank_line = ::ratatui::text::Line::from("");
    lines.push(blank_line);
    let item = ::ratatui::widgets::ListItem::new(lines);
    items.push(item);
}

fn render_text_block<'a>(
    raw: &'a ::core::primitive::str,
    items: &mut ::std::vec::Vec<::ratatui::widgets::ListItem<'a>>,
) {
    let orch_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::Yellow);
    let text_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::White);
    let orch_span = ::ratatui::text::Span::styled("✦ orchestrator · consensus summary:", orch_style);
    let header_line = ::ratatui::text::Line::from(orch_span);
    let mut lines = ::std::vec![header_line];
    for line_str in raw.lines() {
        let content_line = ::ratatui::text::Line::from(::std::vec![
            ::ratatui::text::Span::raw("  "),
            ::ratatui::text::Span::styled(line_str, text_style),
        ]);
        lines.push(content_line);
    }
    let blank_line = ::ratatui::text::Line::from("");
    lines.push(blank_line);
    let item = ::ratatui::widgets::ListItem::new(lines);
    items.push(item);
}

fn render_status_bar(state: &crate::app::AppState, frame: &mut ::ratatui::Frame<'_>, area: ::ratatui::layout::Rect) {
    let line = build_status_line(state);
    let bar_style = ::ratatui::style::Style::default()
        .bg(::ratatui::style::Color::Rgb(18, 18, 18))
        .fg(::ratatui::style::Color::White);
    let para = ::ratatui::widgets::Paragraph::new(line).style(bar_style);
    frame.render_widget(para, area);
}

fn build_status_line<'a>(state: &'a crate::app::AppState) -> ::ratatui::text::Line<'a> {
    let model_id = state.orchestrator_model_id();
    let status = state.status.as_str();

    let brand_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::Magenta);
    let dot_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let status_style = if status.contains("deliberating") || status.contains("synthesizing") {
        ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::Yellow)
    } else {
        ::ratatui::style::Style::default().fg(::ratatui::style::Color::Green)
    };
    let model_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::Cyan);
    let path_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Blue);
    let git_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Green);
    let subagent_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Yellow);

    let status_icon = if status.contains("deliberating") || status.contains("synthesizing") {
        "⠋ "
    } else {
        "● "
    };
    let status_display = ::std::format!("{status_icon}{status}");
    let provider = state.orchestrator_provider();
    let model_display = ::std::format!("◉ {model_id} ({provider})");
    let agent_count = state.agents.len();
    let subagents_display = ::std::format!("👥 {agent_count} agents");

    let sep = ::ratatui::text::Span::styled(" · ", dot_style);

    let spans = ::std::vec![
        ::ratatui::text::Span::styled(" π werðan", brand_style),
        sep.clone(),
        ::ratatui::text::Span::styled(status_display, status_style),
        sep.clone(),
        ::ratatui::text::Span::styled(model_display, model_style),
        sep.clone(),
        ::ratatui::text::Span::styled("📁 werthan", path_style),
        sep.clone(),
        ::ratatui::text::Span::styled("⑂ feat/agents", git_style),
        sep,
        ::ratatui::text::Span::styled(subagents_display, subagent_style),
    ];

    ::ratatui::text::Line::from(spans)
}

fn render_unboxed_prompt(
    state: &crate::app::AppState,
    frame: &mut ::ratatui::Frame<'_>,
    area: ::ratatui::layout::Rect,
) {
    let prefix_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::Yellow);
    let input_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::White);
    let cursor_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Cyan);

    let prefix_span = ::ratatui::text::Span::styled(" ❯ ", prefix_style);
    let input_span = ::ratatui::text::Span::styled(state.input.as_str(), input_style);
    let cursor_span = ::ratatui::text::Span::styled("▏", cursor_style);

    let input_line = ::ratatui::text::Line::from(::std::vec![prefix_span, input_span, cursor_span]);
    let para = ::ratatui::widgets::Paragraph::new(input_line);
    frame.render_widget(para, area);
}

fn render_separator(frame: &mut ::ratatui::Frame<'_>, area: ::ratatui::layout::Rect) {
    let height = area.height as ::core::primitive::usize;
    let sep_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
    let lines = (0..height)
        .map(|_| {
            let span = ::ratatui::text::Span::styled("│", sep_style);
            ::ratatui::text::Line::from(span)
        })
        .collect::<::std::vec::Vec<_>>();
    let para = ::ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(para, area);
}

fn render_agents(state: &crate::app::AppState, frame: &mut ::ratatui::Frame<'_>, area: ::ratatui::layout::Rect) {
    let count = state.agents.len();
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

    for (idx, slot) in state.agents.iter().enumerate() {
        let color = colors[idx % colors.len()];
        render_agent_card(frame, chunks[idx], slot, color);
    }
}

fn render_agent_card(
    frame: &mut ::ratatui::Frame<'_>,
    area: ::ratatui::layout::Rect,
    slot: &crate::app::AgentSlot,
    color: ::ratatui::style::Color,
) {
    let title = ::std::format!(" ◈ {} ", slot.name);
    let title_style = ::ratatui::style::Style::default().bold().fg(color);
    let border_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);

    let title_span = ::ratatui::text::Span::styled(title, title_style);
    let title_line = ::ratatui::text::Line::from(::std::vec![title_span]);
    let block = ::ratatui::widgets::Block::bordered()
        .border_type(::ratatui::widgets::BorderType::Rounded)
        .border_style(border_style)
        .title(title_line);

    let out_guard = slot.output.read().unwrap_or_else(::std::sync::PoisonError::into_inner);
    let lines = format_agent_lines(out_guard.as_str());
    let wrap = ::ratatui::widgets::Wrap {
        trim: false,
    };
    let para = ::ratatui::widgets::Paragraph::new(lines).block(block).wrap(wrap);
    frame.render_widget(para, area);
}

fn format_agent_lines<'a>(output: &'a ::core::primitive::str) -> ::std::vec::Vec<::ratatui::text::Line<'a>> {
    if output == "Idle" || output.is_empty() {
        let idle_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
        let idle_line = ::ratatui::text::Line::from(::std::vec![::ratatui::text::Span::styled(
            "· idle (awaiting council mandate)",
            idle_style
        ),]);
        return ::std::vec![idle_line];
    }

    let mut lines = ::std::vec::Vec::new();
    if output.contains("<tool_call>") {
        let tool_head_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::Cyan);
        let rail_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::DarkGray);
        let tool_code_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Rgb(156, 220, 254));

        let header_span = ::ratatui::text::Span::styled("⇶ tool execution [call]", tool_head_style);
        let head_line = ::ratatui::text::Line::from(header_span);
        lines.push(head_line);
        for line_str in output.lines() {
            let content_line = ::ratatui::text::Line::from(::std::vec![
                ::ratatui::text::Span::styled("  │ ", rail_style),
                ::ratatui::text::Span::styled(line_str, tool_code_style),
            ]);
            lines.push(content_line);
        }
    } else if output.contains("<think>") || output.starts_with("✻ ") {
        let think_head_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::Rgb(119, 125, 136));
        let think_body_style =
            ::ratatui::style::Style::default().italic().fg(::ratatui::style::Color::Rgb(119, 125, 136));

        let header_span = ::ratatui::text::Span::styled("✻ Thought for deliberation", think_head_style);
        let head_line = ::ratatui::text::Line::from(header_span);
        lines.push(head_line);
        for line_str in output.lines() {
            let content_line = ::ratatui::text::Line::from(::std::vec![
                ::ratatui::text::Span::raw("  "),
                ::ratatui::text::Span::styled(line_str, think_body_style),
            ]);
            lines.push(content_line);
        }
    } else {
        let badge_style = ::ratatui::style::Style::default().bold().fg(::ratatui::style::Color::Green);
        let text_style = ::ratatui::style::Style::default().fg(::ratatui::style::Color::White);

        let badge_line = ::ratatui::text::Line::from(::std::vec![::ratatui::text::Span::styled(
            "✔ proposed solution:",
            badge_style
        ),]);
        lines.push(badge_line);

        for out_line in output.lines() {
            let line = ::ratatui::text::Line::from(::std::vec![
                ::ratatui::text::Span::raw("  "),
                ::ratatui::text::Span::styled(out_line, text_style),
            ]);
            lines.push(line);
        }
    }

    lines
}
