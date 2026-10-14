#[derive(::core::fmt::Debug, ::core::default::Default, ::cucumber::World)]
pub struct FlatmodsWorld {
    file_path: ::std::string::String,
    source: ::std::string::String,
    warnings: ::std::vec::Vec<::std::string::String>,
}

#[::cucumber::given(regex = r"^the file path `(.+)` and source code:\s*$")]
async fn given_file_and_source(
    world: &mut self::FlatmodsWorld,
    step: &::cucumber::gherkin::Step,
    path: ::std::string::String,
) {
    world.file_path = path;
    if let ::core::option::Option::Some(docstring) = &step.docstring {
        world.source = docstring.clone();
    } else {
        world.source.clear();
    }
}

#[::cucumber::when(regex = r"^I run flatmods$")]
async fn run_flatmods(world: &mut self::FlatmodsWorld) {
    let path = ::std::path::PathBuf::from(&world.file_path);
    world.warnings = ::flatmods::check_source(&world.source, &path);
}

#[::cucumber::then(regex = r"^I get (\d+) warning\(s\)$")]
async fn assert_warning_count(world: &mut self::FlatmodsWorld, count: ::core::primitive::usize) {
    ::core::assert_eq!(world.warnings.len(), count, "warnings: {:?}", world.warnings);
}

#[::cucumber::then(regex = r"^the warning mentions `(.+)`$")]
async fn assert_warning_mentions(world: &mut self::FlatmodsWorld, ident: ::std::string::String) {
    let found = world.warnings.iter().any(|w| w.contains(&ident));
    ::core::assert!(found, "Assertion failed. Expected warning that mentions `{ident}`, but got: {:?}", world.warnings);
}

#[::tokio::main]
async fn main() {
    let args = ::std::env::args().collect::<::std::vec::Vec<::std::string::String>>();
    if args.iter().any(|arg| arg == "--list") {
        if !args.iter().any(|arg| arg == "--ignored") {
            ::std::println!("flatmods::cucumber: test");
        }
        return;
    }

    <self::FlatmodsWorld as ::cucumber::World>::cucumber()
        .with_default_cli()
        .fail_on_skipped()
        .run_and_exit("tests/features")
        .await;
}
