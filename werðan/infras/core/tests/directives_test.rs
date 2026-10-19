//! Tests for keywords crate.

#[::core::prelude::v1::test]
fn test_extract_magic_directives_empty() {
    let directives = ::protocol::directives::extract_magic_directives("hello world");
    ::core::assert!(!directives.ultrathink);
    ::core::assert!(!directives.orchestrate);
    ::core::assert!(!directives.workflowz);
    ::core::assert_eq!(directives.cleaned_prompt, "hello world");
}

#[::core::prelude::v1::test]
fn test_extract_magic_directives_present() {
    let input = "ultrathink\norchestrate\nworkflowz\nDo something useful\n";
    let directives = ::protocol::directives::extract_magic_directives(input);
    ::core::assert!(directives.ultrathink);
    ::core::assert!(directives.orchestrate);
    ::core::assert!(directives.workflowz);
    ::core::assert_eq!(directives.cleaned_prompt, "Do something useful\n");
}

#[::core::prelude::v1::test]
fn test_extract_magic_directives_inline() {
    let input = "Please ultrathink about this task!";
    let directives = ::protocol::directives::extract_magic_directives(input);
    ::core::assert!(directives.ultrathink);
    ::core::assert_eq!(directives.cleaned_prompt, "Please about this task!");
}
