//! Coding agent terminal user interface.

use ::ratatui::style::Stylize;

#[::tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    let stdin = ::std::io::stdin();
    let is_term = ::std::io::IsTerminal::is_terminal(&stdin);
    let mut args = ::std::env::args();
    let has_dry_run = args.any(|a| a == "--dry-run" || a == "--smoke");
    let mut app = self::AppState::new();
    if !is_term || has_dry_run {
        app.run_smoke().await;
        return;
    }
    app.run_interactive().await;
}

struct AppState {
    input: ::std::string::String,
    master_messages: ::std::vec::Vec<::std::string::String>,
    rig_output: ::std::string::String,
    omp_output: ::std::string::String,
    goose_output: ::std::string::String,
    codex_output: ::std::string::String,
    should_quit: ::core::primitive::bool,
    status: ::std::string::String,
    rig_model: ::bridge_rig::CompletionModel,
    omp_model: ::bridge_omp::CompletionModel,
    goose_model: ::bridge_goose::CompletionModel,
    codex_model: ::bridge_codex::CompletionModel,
}

impl self::AppState {
    fn new() -> Self {
        let rig_model = ::bridge_rig::CompletionModel::builder().model("gpt-4o".to_string()).build();
        let omp_model = ::bridge_omp::CompletionModel::builder().model("omp-default".to_string()).build();
        let goose_model = ::bridge_goose::CompletionModel::builder()
            .model("goose-default".to_string())
            .session_id("tui-session".to_string())
            .build();
        let codex_model = ::bridge_codex::CompletionModel::builder().model("codex-default".to_string()).build();

        Self {
            input: { ::std::string::String::new() },
            master_messages: { ::std::vec::Vec::new() },
            rig_output: { "Idle".to_string() },
            omp_output: { "Idle".to_string() },
            goose_output: { "Idle".to_string() },
            codex_output: { "Idle".to_string() },
            should_quit: false,
            status: { "Ready (Press Esc or Ctrl+C to quit)".to_string() },
            rig_model,
            omp_model,
            goose_model,
            codex_model,
        }
    }

    async fn run_smoke(&mut self) {
        let prompt = "Analyze workspace structure";
        self.dispatch_prompt(prompt).await;
        self.print_smoke_summary();
    }

    async fn run_interactive(&mut self) {
        let mut terminal = match self.init_terminal() {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(_) => return,
        };
        self.event_loop(&mut terminal).await;
        self.restore_terminal(&mut terminal);
    }

    fn init_terminal(
        &self,
    ) -> ::core::result::Result<
        ::ratatui::Terminal<::ratatui::backend::CrosstermBackend<::std::io::Stdout>>,
        ::std::io::Error,
    > {
        ::crossterm::terminal::enable_raw_mode()?;
        let mut stdout = ::std::io::stdout();
        let enter_alt = ::crossterm::terminal::EnterAlternateScreen;
        ::crossterm::execute!(stdout, enter_alt)?;
        let backend = ::ratatui::backend::CrosstermBackend::new(stdout);
        let term = ::ratatui::Terminal::new(backend)?;
        ::core::result::Result::Ok(term)
    }

    async fn event_loop(
        &mut self,
        terminal: &mut ::ratatui::Terminal<::ratatui::backend::CrosstermBackend<::std::io::Stdout>>,
    ) {
        while !self.should_quit {
            let draw_res = terminal.draw(|f| self.draw(f));
            if draw_res.is_err() {
                break;
            }
            let has_event = self.poll_event();
            if has_event {
                let event_res = ::crossterm::event::read();
                let event = match event_res {
                    ::core::result::Result::Ok(e) => e,
                    ::core::result::Result::Err(_) => break,
                };
                self.handle_event(event).await;
            }
        }
    }

    fn draw(&self, frame: &mut ::ratatui::Frame<'_>) {
        let h_layout = ::ratatui::layout::Layout::default();
        let h_dir = h_layout.direction(::ratatui::layout::Direction::Horizontal);
        let h_constraints =
            [::ratatui::layout::Constraint::Percentage(50), ::ratatui::layout::Constraint::Percentage(50)];
        let chunks = h_dir.constraints(h_constraints).split(frame.area());

        self.render_master(frame, chunks[0]);
        self.render_slaves(frame, chunks[1]);
    }

    fn render_master(&self, frame: &mut ::ratatui::Frame<'_>, area: ::ratatui::layout::Rect) {
        let v_layout = ::ratatui::layout::Layout::default();
        let v_dir = v_layout.direction(::ratatui::layout::Direction::Vertical);
        let v_constraints = [
            ::ratatui::layout::Constraint::Min(5),
            ::ratatui::layout::Constraint::Length(3),
            ::ratatui::layout::Constraint::Length(1),
        ];
        let chunks = v_dir.constraints(v_constraints).split(area);

        let master_title = ::ratatui::text::Line::from(" Orchestrator Agent (Master) ").bold();
        let master_block = ::ratatui::widgets::Block::bordered().title(master_title);
        let mut items = ::std::vec::Vec::new();
        for m in &self.master_messages {
            let line = m.as_str();
            let item = ::ratatui::widgets::ListItem::new(line);
            items.push(item);
        }
        let list = ::ratatui::widgets::List::new(items).block(master_block);
        frame.render_widget(list, chunks[0]);

        let input_title = ::ratatui::text::Line::from(" Prompt Input ").bold();
        let input_block = ::ratatui::widgets::Block::bordered().title(input_title);
        let input_line = self.input.as_str();
        let input_para = ::ratatui::widgets::Paragraph::new(input_line).block(input_block);
        frame.render_widget(input_para, chunks[1]);

        let status_line = ::ratatui::text::Line::from(self.status.as_str());
        let status_para = ::ratatui::widgets::Paragraph::new(status_line);
        frame.render_widget(status_para, chunks[2]);
    }

    fn render_slaves(&self, frame: &mut ::ratatui::Frame<'_>, area: ::ratatui::layout::Rect) {
        let v_layout = ::ratatui::layout::Layout::default();
        let v_dir = v_layout.direction(::ratatui::layout::Direction::Vertical);
        let v_constraints = [
            ::ratatui::layout::Constraint::Ratio(1, 4),
            ::ratatui::layout::Constraint::Ratio(1, 4),
            ::ratatui::layout::Constraint::Ratio(1, 4),
            ::ratatui::layout::Constraint::Ratio(1, 4),
        ];
        let chunks = v_dir.constraints(v_constraints).split(area);

        let rig_title = ::ratatui::text::Line::from(" Slave 1: Rig (OpenAI Bridge) ").bold();
        let rig_block = ::ratatui::widgets::Block::bordered().title(rig_title);
        let rig_line = self.rig_output.as_str();
        let rig_para = ::ratatui::widgets::Paragraph::new(rig_line).block(rig_block);
        frame.render_widget(rig_para, chunks[0]);

        let omp_title = ::ratatui::text::Line::from(" Slave 2: Omp (RPC Bridge) ").bold();
        let omp_block = ::ratatui::widgets::Block::bordered().title(omp_title);
        let omp_line = self.omp_output.as_str();
        let omp_para = ::ratatui::widgets::Paragraph::new(omp_line).block(omp_block);
        frame.render_widget(omp_para, chunks[1]);

        let goose_title = ::ratatui::text::Line::from(" Slave 3: Goose (ACP Bridge) ").bold();
        let goose_block = ::ratatui::widgets::Block::bordered().title(goose_title);
        let goose_line = self.goose_output.as_str();
        let goose_para = ::ratatui::widgets::Paragraph::new(goose_line).block(goose_block);
        frame.render_widget(goose_para, chunks[2]);

        let codex_title = ::ratatui::text::Line::from(" Slave 4: Codex (Protocol Bridge) ").bold();
        let codex_block = ::ratatui::widgets::Block::bordered().title(codex_title);
        let codex_line = self.codex_output.as_str();
        let codex_para = ::ratatui::widgets::Paragraph::new(codex_line).block(codex_block);
        frame.render_widget(codex_para, chunks[3]);
    }

    fn poll_event(&self) -> ::core::primitive::bool {
        let timeout = ::std::time::Duration::from_millis(50);
        let res = ::crossterm::event::poll(timeout);
        res.unwrap_or_default()
    }

    async fn handle_event(&mut self, event: ::crossterm::event::Event) {
        if let ::crossterm::event::Event::Key(key) = event {
            self.handle_key(key).await;
        }
    }

    async fn handle_key(&mut self, key: ::crossterm::event::KeyEvent) {
        match key.code {
            ::crossterm::event::KeyCode::Esc => {
                self.should_quit = true;
            },
            ::crossterm::event::KeyCode::Char('c')
                if key.modifiers.contains(::crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.should_quit = true;
            },
            ::crossterm::event::KeyCode::Enter => {
                let trimmed = self.input.trim();
                let prompt = trimmed.to_string();
                if !prompt.is_empty() {
                    self.input.clear();
                    self.dispatch_prompt(&prompt).await;
                }
            },
            ::crossterm::event::KeyCode::Backspace => {
                self.input.pop();
            },
            ::crossterm::event::KeyCode::Char(c) => {
                self.input.push(c);
            },
            _ => {},
        }
    }

    fn restore_terminal(
        &self,
        terminal: &mut ::ratatui::Terminal<::ratatui::backend::CrosstermBackend<::std::io::Stdout>>,
    ) {
        let _ = ::crossterm::terminal::disable_raw_mode();
        let mut stdout = ::std::io::stdout();
        let leave_alt = ::crossterm::terminal::LeaveAlternateScreen;
        let _ = ::crossterm::execute!(stdout, leave_alt);
        let _ = terminal.show_cursor();
    }

    async fn dispatch_prompt(&mut self, prompt: &::core::primitive::str) {
        let user_msg = ::std::format!("User: {prompt}");
        self.master_messages.push(user_msg);
        self.status = "Relaying to slave bridges...".to_string();
        self.execute_relay(prompt).await;
        self.master_messages.push("Master: Relayed to all 4 children bridges.".to_string());
        self.status = "Ready (Press Esc or Ctrl+C to quit)".to_string();
    }

    async fn execute_relay(&mut self, prompt: &::core::primitive::str) {
        let rig_res = self.rig_model.call(prompt).await;
        let rig_text = match rig_res {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(e) => e.to_string(),
        };
        self.rig_output = rig_text;

        let omp_res = self.omp_model.call(prompt).await;
        let omp_text = match omp_res {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(e) => e.to_string(),
        };
        self.omp_output = omp_text;

        let goose_res = self.goose_model.call(prompt).await;
        let goose_text = match goose_res {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(e) => e.to_string(),
        };
        self.goose_output = goose_text;

        let codex_res = self.codex_model.call(prompt).await;
        let codex_text = match codex_res {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(e) => e.to_string(),
        };
        self.codex_output = codex_text;
    }

    fn print_smoke_summary(&self) {
        ::std::println!("Orchestrator Master-Slave TUI initialized successfully.");
        ::std::println!("Master prompt: Analyze workspace structure");
        let rig_out = &self.rig_output;
        ::std::println!("Slave 1 (Rig): {rig_out}");
        let omp_out = &self.omp_output;
        ::std::println!("Slave 2 (Omp): {omp_out}");
        let goose_out = &self.goose_output;
        ::std::println!("Slave 3 (Goose): {goose_out}");
        let codex_out = &self.codex_output;
        ::std::println!("Slave 4 (Codex): {codex_out}");
    }
}
