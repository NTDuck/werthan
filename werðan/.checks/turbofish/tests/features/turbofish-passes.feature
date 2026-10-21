Feature: Turbofish compliant bindings

  Scenario: Allow turbofish on call expression without let type annotation
    Given the source code:
      """
      fn run() {
          let items = iter.collect::<::std::vec::Vec<::core::primitive::u32>>();
      }
      """
    When I run turbofish
    Then I get 0 warning(s)

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

  Scenario: Allow while let loops without false warnings
    Given the source code:
      """
      fn run() {
          while let ::core::option::Option::Some(x) = opt.next() {
              let y = x;
          }
      }
      """
    When I run turbofish
    Then I get 0 warning(s)
