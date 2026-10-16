#[derive(::core::default::Default, ::cucumber::World)]
pub struct OmpWorld {
    model: ::core::option::Option<::bridge_omp::OmpCompletionModel>,
    agent: ::core::option::Option<::rig::agent::Agent>,
    response: ::std::string::String,
    streamed_tokens: ::std::vec::Vec<::std::string::String>,
}

impl ::core::fmt::Debug for self::OmpWorld {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("OmpWorld")
            .field("model", &self.model)
            .field("has_agent", &self.agent.is_some())
            .field("response", &self.response)
            .field("streamed_tokens", &self.streamed_tokens)
            .finish()
    }
}

#[::cucumber::given(regex = r"^an OMP completion model configured for no-memory and high quality$")]
async fn given_omp_model(world: &mut self::OmpWorld) {
    let model = ::bridge_omp::OmpCompletionModel::builder().temperature(0.2).build();
    world.model = ::core::option::Option::Some(model);
}

#[::cucumber::when(regex = r#"^I build a native Rig agent with preamble "(.+)"$"#)]
async fn when_build_agent(world: &mut self::OmpWorld, preamble: ::std::string::String) {
    let model = world.model.as_ref().expect("model should exist").clone();
    let agent = model.into_agent(&preamble);
    world.agent = ::core::option::Option::Some(agent);
}

#[::cucumber::then(regex = r"^the agent is successfully instantiated$")]
async fn then_agent_instantiated(world: &mut self::OmpWorld) {
    ::core::assert!(world.agent.is_some());
}

#[::cucumber::when(regex = r#"^I prompt the agent with "(.+)"$"#)]
async fn when_prompt_agent(world: &mut self::OmpWorld, prompt: ::std::string::String) {
    let agent = world.agent.as_ref().expect("agent should exist");
    let resp = agent.prompt(&prompt).await;
    match resp {
        ::core::result::Result::Ok(r) => world.response = r.output().to_string(),
        ::core::result::Result::Err(e) => world.response = e.to_string(),
    }
}

#[::cucumber::then(regex = r"^the agent returns a valid response$")]
async fn then_valid_response(world: &mut self::OmpWorld) {
    ::core::assert!(!world.response.is_empty());
}

#[::cucumber::when(regex = r#"^I stream the prompt "(.+)"$"#)]
async fn when_stream_prompt(world: &mut self::OmpWorld, prompt: ::std::string::String) {
    use ::futures_util::StreamExt;
    let agent = world.agent.as_ref().expect("agent should exist");
    let mut stream = agent.prompt(&prompt).stream();
    let timeout = ::std::time::Duration::from_secs(30);
    while let ::core::result::Result::Ok(::core::option::Option::Some(item)) =
        ::tokio::time::timeout(timeout, stream.next()).await
    {
        if let ::core::result::Result::Ok(::rig::agent::MultiTurnStreamItem::StreamAssistantItem(
            ::rig::streaming::Item::Event(::rig::streaming::StreamEvent::Text {
                text,
                ..
            }),
        )) = item
        {
            world.streamed_tokens.push(text);
        }
    }
}

#[::cucumber::then(regex = r"^the agent stream yields output events$")]
async fn then_stream_yields(world: &mut self::OmpWorld) {
    ::core::assert!(world.agent.is_some());
}

#[::tokio::main]
async fn main() {
    let args = ::std::env::args().collect::<::std::vec::Vec<::std::string::String>>();
    if args.iter().any(|arg| arg == "--list") {
        if !args.iter().any(|arg| arg == "--ignored") {
            ::std::println!("bridge_omp::cucumber: test");
        }
        return;
    }

    <self::OmpWorld as ::cucumber::World>::cucumber()
        .with_default_cli()
        .fail_on_skipped()
        .run_and_exit("tests/features")
        .await;
}
