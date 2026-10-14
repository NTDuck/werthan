#[derive(::core::fmt::Debug, ::core::default::Default, ::cucumber::World)]
pub struct Ste100World {
    source: ::std::string::String,
    warnings: ::std::vec::Vec<::std::string::String>,
}

#[::cucumber::given(regex = r"^the source code:\s*$")]
async fn given_source(world: &mut self::Ste100World, step: &::cucumber::gherkin::Step) {
    if let ::core::option::Option::Some(docstring) = &step.docstring {
        world.source = docstring.clone();
    } else {
        world.source.clear();
    }
}

#[::cucumber::when(regex = r"^I append the line `(.+)`$")]
async fn append_line(world: &mut self::Ste100World, line: ::std::string::String) {
    world.source.push_str(&line);
    world.source.push('\n');
}

#[::cucumber::when(regex = r"^I run ste100$")]
async fn run_ste100(world: &mut self::Ste100World) {
    let path = ::std::path::PathBuf::from("test.rs");
    world.warnings = ::ste100::check_source(&world.source, &path);
}

#[::cucumber::then(regex = r"^I get (\d+) warning\(s\)$")]
async fn assert_warning_count(world: &mut self::Ste100World, count: ::core::primitive::usize) {
    ::core::assert_eq!(world.warnings.len(), count, "warnings: {:?}", world.warnings);
}

#[::cucumber::then(regex = r"^the warning mentions `(.+)`$")]
async fn assert_warning_mentions(world: &mut self::Ste100World, ident: ::std::string::String) {
    let found = world.warnings.iter().any(|w| w.contains(&ident));
    ::core::assert!(found, "Assertion failed. Expected warning that mentions `{ident}`, but got: {:?}", world.warnings);
}

#[::tokio::main]
async fn main() {
    let args = ::std::env::args().collect::<::std::vec::Vec<::std::string::String>>();
    if args.iter().any(|arg| arg == "--list") {
        if !args.iter().any(|arg| arg == "--ignored") {
            ::std::println!("ste100::cucumber: test");
        }
        return;
    }

    <self::Ste100World as ::cucumber::World>::cucumber()
        .with_default_cli()
        .fail_on_skipped()
        .run_and_exit("tests/features")
        .await;
}
