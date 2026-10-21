Feature: Redundant let annotation detection

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

  Scenario: Disallow explicit type annotation when RHS is a function call
    Given the source code:
      """
      fn run() {
          let val: ::std::string::String = parse_val(input);
      }
      """
    When I run turbofish
    Then I get 1 warning(s)

  Scenario: Catch violation following an if let block
    Given the source code:
      """
      fn run() {
          if let ::core::option::Option::Some(x) = opt {}
          let y: ::core::primitive::u32 = get_val();
      }
      """
    When I run turbofish
    Then I get 1 warning(s)
    And the warning mentions `use turbofish on call expression instead of type annotation in `let` binding`

  Scenario: Disallow explicit type annotation on let mut binding with call expression
    Given the source code:
      """
      fn run() {
          let mut items: ::std::vec::Vec<::core::primitive::u32> = iter.collect();
      }
      """
    When I run turbofish
    Then I get 1 warning(s)
    And the warning mentions `use turbofish on call expression instead of type annotation in `let` binding`

  Scenario: Disallow redundant type annotation when call already uses turbofish
    Given the source code:
      """
      fn run() {
          let items: ::std::vec::Vec<::core::primitive::u32> = iter.collect::<::std::vec::Vec<::core::primitive::u32>>();
      }
      """
    When I run turbofish
    Then I get 1 warning(s)
    And the warning mentions `use turbofish on call expression instead of type annotation in `let` binding`
