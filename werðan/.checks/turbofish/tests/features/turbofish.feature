Feature: Turbofish Rule
  As a Rust developer
  I want to ensure callers use turbofish generics on call expressions
  Rather than type annotations on let bindings

  Scenario: Disallow explicit type annotation when RHS is a method call
    Given the source code:
      """
      fn run() {
          let items: ::std::vec::Vec<::core::primitive::u32> = iter.collect();
      }
      """
    When I run turbofish
    Then I get 1 warning(s)
    And the warning mentions `use turbofish on call expression instead of type annotation in `let` binding`

  Scenario: Allow turbofish on call expression without let type annotation
    Given the source code:
      """
      fn run() {
          let items = iter.collect::<::std::vec::Vec<::core::primitive::u32>>();
      }
      """
    When I run turbofish
    Then I get 0 warning(s)

  Scenario: Disallow explicit type annotation when RHS is a function call
    Given the source code:
      """
      fn run() {
          let val: ::std::string::String = parse_val(input);
      }
      """
    When I run turbofish
    Then I get 1 warning(s)

  Scenario: Allow explicit type annotation when RHS is not a call expression
    Given the source code:
      """
      fn run() {
          let x: ::core::primitive::u32 = 42;
          let s: &::core::primitive::str = "hello";
      }
      """
    When I run turbofish
    Then I get 0 warning(s)

  Scenario: Allow let bindings without type annotations
    Given the source code:
      """
      fn run() {
          let a = 1;
          let b = get_val();
      }
      """
    When I run turbofish
    Then I get 0 warning(s)
