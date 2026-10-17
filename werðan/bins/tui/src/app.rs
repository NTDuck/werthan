//! Application state and event loop coordination using Rig models.

/// Create an Arc<RwLock<String>> output buffer initialized to "Idle".
pub fn create_slot_output() -> ::std::sync::Arc<::std::sync::RwLock<::std::string::String>> {
    let idle = "Idle".to_string();
    let lock = ::std::sync::RwLock::new(idle);
    ::std::sync::Arc::new(lock)
}

/// A generic slot representing a Rig agent and its live output buffer.
#[derive(::bon::Builder, ::core::clone::Clone)]
pub struct AgentSlot {
    /// Agent identifier name.
    #[builder(into)]
    pub name: ::std::string::String,

    /// Native Rig agent instance.
    pub agent: ::rig::agent::Agent,

    /// Latest response output string protected by RwLock for real-time streaming.
    pub output: ::std::sync::Arc<::std::sync::RwLock<::std::string::String>>,
}

/// State holding models, agents, messages, and configuration.
#[derive(::bon::Builder)]
pub struct AppState {
    /// Current prompt input string.
    #[builder(default)]
    pub input: ::std::string::String,

    /// History of messages in the master orchestrator panel.
    #[builder(default)]
    pub master_messages: ::std::vec::Vec<::std::string::String>,

    /// Generic list of non-orchestrator Rig agents rendered in the right pane.
    pub agents: ::std::vec::Vec<self::AgentSlot>,

    /// Flag indicating whether the application should exit.
    #[builder(default)]
    pub should_quit: ::core::primitive::bool,

    /// Status line string.
    #[builder(default = "Ready (Press Esc or Ctrl+C to quit)".to_string())]
    pub status: ::std::string::String,

    /// Rig dynamic model for orchestrator.
    pub orchestrator_model: ::rig::driver::DynModel<::rig::operation::Completion>,

    /// Underlying model configuration.
    pub config: crate::config::ModelConfig,

    /// Council execution DAG.
    pub dag: crate::tree::CouncilDag,

    /// Pending prompt string awaiting execution.
    pub pending_prompt: ::core::option::Option<::std::string::String>,
}

impl self::AppState {
    /// Execute a non-interactive smoke run for headless verification.
    pub async fn run_smoke(&mut self) {
        let prompt = "Analyze workspace structure";
        self.dispatch_prompt(prompt, ::core::option::Option::None).await;
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
        let mut dirty = true;
        while !self.should_quit {
            if dirty {
                let draw_res = terminal.draw(|f| self.draw(f));
                if draw_res.is_err() {
                    break;
                }
                dirty = false;
            }

            if let ::core::option::Option::Some(prompt) = self.pending_prompt.take() {
                self.dispatch_prompt(&prompt, ::core::option::Option::Some(terminal)).await;
                dirty = true;
                continue;
            }

            let timeout = ::std::time::Duration::from_millis(20);
            let has_event = ::crossterm::event::poll(timeout).unwrap_or(false);
            if has_event {
                let zero_dur = ::std::time::Duration::ZERO;
                while let ::core::result::Result::Ok(true) = ::crossterm::event::poll(zero_dur) {
                    let event_res = ::crossterm::event::read();
                    let event = match event_res {
                        ::core::result::Result::Ok(e) => e,
                        ::core::result::Result::Err(_) => break,
                    };
                    self.handle_event(event).await;
                    dirty = true;
                }
            }
        }
    }

    async fn dispatch_prompt(
        &mut self,
        prompt: &::core::primitive::str,
        terminal: ::core::option::Option<
            &mut ::ratatui::Terminal<::ratatui::backend::CrosstermBackend<::std::io::Stdout>>,
        >,
    ) {
        self.execute_relay(prompt, terminal).await;
        self.status = "Ready (Press Esc or Ctrl+C to quit)".to_string();
    }

    async fn execute_relay(
        &mut self,
        prompt: &::core::primitive::str,
        mut terminal: ::core::option::Option<
            &mut ::ratatui::Terminal<::ratatui::backend::CrosstermBackend<::std::io::Stdout>>,
        >,
    ) {
        use ::futures_util::StreamExt;
        let timeout_dur = ::std::time::Duration::from_secs(180);
        // Stage 1: orchestrator frames the problem
        self.status = "Council orchestrator framing the problem...".to_string();
        if let ::core::option::Option::Some(term) = terminal.as_mut() {
            let _ = term.draw(|f| self.draw(f));
        }
        let new_msg = ::std::string::String::new();
        self.master_messages.push(new_msg);
        let mut concern = ::std::string::String::new();
        {
            let mut frame_stream = self.dag.orchestrator.prompt(prompt).stream();
            let deadline = ::tokio::time::Instant::now() + timeout_dur;
            loop {
                let now = ::tokio::time::Instant::now();
                if now >= deadline {
                    break;
                }
                let remaining = deadline - now;
                let next_res = ::tokio::time::timeout(remaining, frame_stream.next()).await;
                match next_res {
                    ::core::result::Result::Ok(::core::option::Option::Some(::core::result::Result::Ok(item))) => {
                        match item {
                            ::rig::agent::MultiTurnStreamItem::StreamAssistantItem(::rig::streaming::Item::Event(
                                ::rig::streaming::StreamEvent::Text {
                                    text,
                                    ..
                                },
                            )) => {
                                concern.push_str(&text);
                                if let ::core::option::Option::Some(last) = self.master_messages.last_mut() {
                                    last.push_str(&text);
                                }
                                if let ::core::option::Option::Some(term) = terminal.as_mut() {
                                    let _ = term.draw(|f| self.draw(f));
                                }
                            },
                            ::rig::agent::MultiTurnStreamItem::StreamAssistantItem(::rig::streaming::Item::Event(
                                ::rig::streaming::StreamEvent::Reasoning {
                                    text,
                                    ..
                                },
                            )) => {
                                concern.push_str(&text);
                                if let ::core::option::Option::Some(last) = self.master_messages.last_mut() {
                                    last.push_str(&text);
                                }
                                if let ::core::option::Option::Some(term) = terminal.as_mut() {
                                    let _ = term.draw(|f| self.draw(f));
                                }
                            },
                            ::rig::agent::MultiTurnStreamItem::FinalResponse(resp) => {
                                if concern.is_empty() {
                                    concern = resp.output().to_string();
                                }
                                break;
                            },
                            _ => {},
                        }
                    },
                    _ => break,
                }
            }
        }

        // Stage 2: parallel council deliberation
        self.status = "Council deliberating proposals in parallel...".to_string();
        if let ::core::option::Option::Some(term) = terminal.as_mut() {
            let _ = term.draw(|f| self.draw(f));
        }

        let worker_tasks = self
            .agents
            .iter()
            .enumerate()
            .map(|(idx, slot)| {
                let member_node = &self.dag.members[idx];
                let member_prompt = self.dag.frame_member_prompt(member_node, &concern);
                let agent = slot.agent.clone();
                let name = slot.name.clone();
                let output_arc = slot.output.clone();
                let timeout = timeout_dur;
                {
                    let mut out = output_arc.write().unwrap_or_else(::std::sync::PoisonError::into_inner);
                    out.clear();
                }
                ::tokio::spawn(async move {
                    let mut stream = agent.prompt(&member_prompt).stream();
                    let deadline = ::tokio::time::Instant::now() + timeout;
                    loop {
                        let now = ::tokio::time::Instant::now();
                        if now >= deadline {
                            let mut out = output_arc.write().unwrap_or_else(::std::sync::PoisonError::into_inner);
                            out.push_str(&::std::format!("\n[{name}-timeout]"));
                            break;
                        }
                        let remaining = deadline - now;
                        let next_fut = stream.next();
                        let timeout_res = ::tokio::time::timeout(remaining, next_fut).await;
                        match timeout_res {
                            ::core::result::Result::Ok(::core::option::Option::Some(::core::result::Result::Ok(item))) => {
                                match item {
                                    ::rig::agent::MultiTurnStreamItem::StreamAssistantItem(
                                        ::rig::streaming::Item::Event(::rig::streaming::StreamEvent::Text {
                                            text,
                                            ..
                                        }),
                                    ) => {
                                        let mut out =
                                            output_arc.write().unwrap_or_else(::std::sync::PoisonError::into_inner);
                                        out.push_str(&text);
                                    },
                                    ::rig::agent::MultiTurnStreamItem::StreamAssistantItem(
                                        ::rig::streaming::Item::Event(::rig::streaming::StreamEvent::Reasoning {
                                            text,
                                            ..
                                        }),
                                    ) => {
                                        let mut out =
                                            output_arc.write().unwrap_or_else(::std::sync::PoisonError::into_inner);
                                        out.push_str(&text);
                                    },
                                    ::rig::agent::MultiTurnStreamItem::FinalResponse(resp) => {
                                        let mut out =
                                            output_arc.write().unwrap_or_else(::std::sync::PoisonError::into_inner);
                                        if out.is_empty() {
                                            let final_text = resp.output();
                                            out.push_str(&final_text);
                                        }
                                        break;
                                    },
                                    _ => {},
                                }
                            },
                            ::core::result::Result::Ok(::core::option::Option::Some(::core::result::Result::Err(e))) => {
                                let mut out = output_arc.write().unwrap_or_else(::std::sync::PoisonError::into_inner);
                                if out.is_empty() {
                                    let err_msg = ::std::format!("[{name}-error: {e}]");
                                    out.push_str(&err_msg);
                                }
                                break;
                            },
                            _ => break,
                        }
                    }
                })
            })
            .collect::<::std::vec::Vec<_>>();
        while !worker_tasks.iter().all(|t| t.is_finished()) {
            if let ::core::option::Option::Some(term) = terminal.as_mut() {
                let _ = term.draw(|f| self.draw(f));
            }
            let tick = ::std::time::Duration::from_millis(30);
            ::tokio::time::sleep(tick).await;
        }

        let _ = ::futures_util::future::join_all(worker_tasks).await;
        if let ::core::option::Option::Some(term) = terminal.as_mut() {
            let _ = term.draw(|f| self.draw(f));
        }

        self.status = "Orchestrator synthesizing council consensus...".to_string();
        if let ::core::option::Option::Some(term) = terminal.as_mut() {
            let _ = term.draw(|f| self.draw(f));
        }

        let deliberations = self
            .agents
            .iter()
            .map(|slot| {
                let out_text = slot.output.read().unwrap_or_else(::std::sync::PoisonError::into_inner).clone();
                (slot.name.clone(), out_text)
            })
            .collect::<::std::vec::Vec<_>>();
        let synthesis_prompt = self.dag.frame_synthesis_prompt(&concern, &deliberations);

        let new_msg = ::std::string::String::new();
        self.master_messages.push(new_msg);

        let mut orch_stream = self.dag.orchestrator.prompt(&synthesis_prompt).stream();
        let deadline = ::tokio::time::Instant::now() + timeout_dur;
        loop {
            let now = ::tokio::time::Instant::now();
            if now >= deadline {
                break;
            }
            let remaining = deadline - now;
            let next_fut = orch_stream.next();
            let timeout_res = ::tokio::time::timeout(remaining, next_fut).await;
            match timeout_res {
                ::core::result::Result::Ok(::core::option::Option::Some(::core::result::Result::Ok(item))) => {
                    match item {
                        ::rig::agent::MultiTurnStreamItem::StreamAssistantItem(::rig::streaming::Item::Event(
                            ::rig::streaming::StreamEvent::Text {
                                text,
                                ..
                            },
                        )) => {
                            if let ::core::option::Option::Some(last) = self.master_messages.last_mut() {
                                last.push_str(&text);
                            }
                            if let ::core::option::Option::Some(term) = terminal.as_mut() {
                                let _ = term.draw(|f| self.draw(f));
                            }
                        },
                        ::rig::agent::MultiTurnStreamItem::StreamAssistantItem(::rig::streaming::Item::Event(
                            ::rig::streaming::StreamEvent::Reasoning {
                                text,
                                ..
                            },
                        )) => {
                            if let ::core::option::Option::Some(last) = self.master_messages.last_mut() {
                                last.push_str(&text);
                            }
                            if let ::core::option::Option::Some(term) = terminal.as_mut() {
                                let _ = term.draw(|f| self.draw(f));
                            }
                        },
                        ::rig::agent::MultiTurnStreamItem::FinalResponse(resp) => {
                            if let ::core::option::Option::Some(last) = self.master_messages.last_mut() {
                                if last.is_empty() {
                                    let final_text = resp.output();
                                    last.push_str(&final_text);
                                }
                            }
                            break;
                        },
                        _ => {},
                    }
                },
                ::core::result::Result::Ok(::core::option::Option::Some(::core::result::Result::Err(e))) => {
                    if let ::core::option::Option::Some(last) = self.master_messages.last_mut() {
                        if last.is_empty() {
                            let err_msg = ::std::format!("[orchestrator-error: {e}]");
                            last.push_str(&err_msg);
                        }
                    }
                    break;
                },
                _ => break,
            }
        }
        if let ::core::option::Option::Some(term) = terminal.as_mut() {
            let _ = term.draw(|f| self.draw(f));
        }
    }

    fn draw(&self, frame: &mut ::ratatui::Frame<'_>) {
        crate::ui::draw(self, frame);
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
                    let user_msg = ::std::format!("User: {prompt}");
                    self.master_messages.push(user_msg);
                    self.status = "Delegating prompt to workers...".to_string();
                    self.pending_prompt = ::core::option::Option::Some(prompt);
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

    fn print_smoke_summary(&self) {
        let model = &self.config.model_name;
        let url = &self.config.base_url;
        ::std::println!("Root Orchestrator initialized with Ashalliants model configuration ({model} at {url}).");
        ::std::println!("Worker completion models instantiated as Rig DynModel backends.");
        ::std::println!("Master prompt: Analyze workspace structure");
        for (i, slot) in self.agents.iter().enumerate() {
            let idx = i + 1;
            let name = &slot.name;
            let out = slot.output.read().unwrap_or_else(::std::sync::PoisonError::into_inner);
            ::std::println!("Agent {idx} ({name}): {out}");
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
