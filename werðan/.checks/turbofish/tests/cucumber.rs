#[derive(::core::fmt::Debug, ::core::default::Default, ::cucumber::World)]
pub struct TurbofishWorld {
    source: ::std::string::String,
    warnings: ::std::vec::Vec<::std::string::String>,
}

#[::cucumber::given(regex = r"^the source code:\s*$")]
async fn given_source(world: &mut self::TurbofishWorld, step: &::cucumber::gherkin::Step) {
    if let ::core::option::Option::Some(docstring) = &step.docstring {
        world.source = docstring.clone();
    } else {
        world.source.clear();
    }
}

#[::cucumber::when(regex = r"^I append the line `(.+)`$")]
async fn append_line(world: &mut self::TurbofishWorld, line: ::std::string::String) {
    world.source.push_str(&line);
    world.source.push('\n');
}

#[::cucumber::when(regex = r"^I run turbofish$")]
async fn run_turbofish(world: &mut self::TurbofishWorld) {
    let path = ::std::path::PathBuf::from("test.rs");
    world.warnings = ::turbofish::check_source(&world.source, &path);
}

#[::cucumber::then(regex = r"^I get (\d+) warning\(s\)$")]
async fn assert_warning_count(world: &mut self::TurbofishWorld, count: ::core::primitive::usize) {
    ::core::assert_eq!(world.warnings.len(), count, "warnings: {:?}", world.warnings);
}

#[::cucumber::then(regex = r"^the warning mentions `(.+)`$")]
async fn assert_warning_mentions(world: &mut self::TurbofishWorld, ident: ::std::string::String) {
    let found = world.warnings.iter().any(|w| w.contains(&ident));
    ::core::assert!(found, "Assertion failed. Expected warning that mentions `{ident}`, but got: {:?}", world.warnings);
}

#[::tokio::main]
async fn main() {
    let args = ::std::env::args().collect::<::std::vec::Vec<::std::string::String>>();
    if args.iter().any(|arg| arg == "--list") {
        if !args.iter().any(|arg| arg == "--ignored") {
            ::std::println!("turbofish::cucumber: test");
        }
        return;
    }

    <self::TurbofishWorld as ::cucumber::World>::cucumber()
        .with_default_cli()
        .fail_on_skipped()
        .run_and_exit("tests/features")
        .await;
}
