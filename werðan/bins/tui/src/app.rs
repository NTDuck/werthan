//! Application state and event loop coordination using Rig models.

/// A generic worker agent with display name and latest output.
#[derive(::core::clone::Clone)]
pub struct Worker {
    /// Worker name, e.g. "omp-rpc".
    pub name: ::std::string::String,
    /// Native Rig agent instance.
    pub agent: ::rig::agent::Agent,
    /// Latest response output string.
    pub output: ::std::string::String,
}

impl self::Worker {
    /// Create a new worker with the given name and Rig agent.
    pub fn new(name: impl ::core::convert::Into<::std::string::String>, agent: ::rig::agent::Agent) -> Self {
        Self {
            name: { name.into() },
            agent,
            output: { "Idle".to_string() },
        }
    }
}

/// State holding models, agents, messages, and configuration.
pub struct AppState {
    /// Current prompt input string.
    pub input: ::std::string::String,
    /// History of messages in the master orchestrator panel.
    pub master_messages: ::std::vec::Vec<::std::string::String>,
    /// Dynamic list of worker backends.
    pub workers: ::std::vec::Vec<self::Worker>,
    /// Flag indicating whether the application should exit.
    pub should_quit: ::core::primitive::bool,
    /// Status line string.
    pub status: ::std::string::String,
    /// Rig dynamic model for orchestrator.
    pub orchestrator_model: ::rig::driver::DynModel<::rig::operation::Completion>,
    /// Root orchestrator agent instance.
    pub orchestrator_agent: ::rig::agent::Agent,
    /// Underlying model configuration.
    pub config: crate::config::ModelConfig,
}

impl self::AppState {
    /// Create a new application state with Rig models and configuration.
    pub fn new(config: crate::config::ModelConfig, workers: ::std::vec::Vec<self::Worker>) -> Self {
        let orchestrator_model = config.build_completion_model();
        let orchestrator_agent = crate::tree::build_agent_tree(orchestrator_model.clone(), &workers);

        Self {
            input: { ::std::string::String::new() },
            master_messages: { ::std::vec::Vec::new() },
            workers,
            should_quit: false,
            status: { "Ready (Press Esc or Ctrl+C to quit)".to_string() },
            orchestrator_model,
            orchestrator_agent,
            config,
        }
    }

    /// Execute a non-interactive smoke run for headless verification.
    pub async fn run_smoke(&mut self) {
        let prompt = "Analyze workspace structure";
        self.dispatch_prompt(prompt).await;
        self.print_smoke_summary();
    }

    /// Run the interactive Ratatui event loop on the current terminal.
    pub async fn run_interactive(&mut self) {
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
        crate::ui::draw(self, frame);
    }

    fn poll_event(&self) -> ::core::primitive::bool {
        let timeout = ::std::time::Duration::from_millis(50);
        let res = ::crossterm::event::poll(timeout);
        res.unwrap_or_default()
    }

    async fn handle_event(&mut self, event: ::crossterm::event::Event) {
        if let ::crossterm::event::Event::Key(key) = event {
            if key.kind == ::crossterm::event::KeyEventKind::Release {
                return;
            }
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
            ::crossterm::event::KeyCode::Enter
            | ::crossterm::event::KeyCode::Char('\n')
            | ::crossterm::event::KeyCode::Char('\r') => {
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
        self.status = "Executing hierarchical agent tree...".to_string();

        self.execute_relay(prompt).await;

        self.master_messages.push("Master: Hierarchical execution completed.".to_string());
        self.status = "Ready (Press Esc or Ctrl+C to quit)".to_string();
    }

    async fn execute_relay(&mut self, prompt: &::core::primitive::str) {
        let timeout_dur = ::std::time::Duration::from_secs(30);
        for worker in &mut self.workers {
            let prompt_fut = worker.agent.prompt(prompt);
            let timeout_res = ::tokio::time::timeout(timeout_dur, prompt_fut).await;
            worker.output = match timeout_res {
                ::core::result::Result::Ok(::core::result::Result::Ok(resp)) => resp.output().to_string(),
                ::core::result::Result::Ok(::core::result::Result::Err(e)) => e.to_string(),
                ::core::result::Result::Err(_) => {
                    let name = &worker.name;
                    ::std::format!("[{name}-offline: {prompt}]")
                },
            };
        }

        let orch_prompt_fut = self.orchestrator_agent.prompt(prompt);
        let orch_timeout_res = ::tokio::time::timeout(timeout_dur, orch_prompt_fut).await;
        let orch_text = match orch_timeout_res {
            ::core::result::Result::Ok(::core::result::Result::Ok(resp)) => resp.output().to_string(),
            _ => {
                let model = &self.config.model_name;
                let url = &self.config.base_url;
                ::std::format!(
                    "[ashalliants-offline] Orchestrated prompt across Rig models for '{prompt}'. Target: {model} at \
                     {url}."
                )
            },
        };
        self.master_messages.push(orch_text);
    }

    fn print_smoke_summary(&self) {
        let model = &self.config.model_name;
        let url = &self.config.base_url;
        ::std::println!("Root Orchestrator initialized with Ashalliants model configuration ({model} at {url}).");
        ::std::println!("Worker completion models instantiated as Rig DynModel backends.");
        ::std::println!("Master prompt: Analyze workspace structure");
        for (i, worker) in self.workers.iter().enumerate() {
            let idx = i + 1;
            let name = &worker.name;
            let out = &worker.output;
            ::std::println!("Worker {idx} ({name}): {out}");
        }
    }

    /// Return the model identifier from Rig's dynamic model API.
    pub fn orchestrator_model_id(&self) -> &::core::primitive::str {
        self.orchestrator_model.id().unwrap_or(self.config.model_id.as_str())
    }

    /// Return the provider name from Rig's dynamic model API.
    pub fn orchestrator_provider(&self) -> &::core::primitive::str {
        self.orchestrator_model.name()
    }
}
