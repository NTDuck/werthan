Feature: Nested call detection

  Scenario: Nested function calls trigger warnings
    Given the source code:
      """
      fn run() {
          foo(bar(bazz(x)));
      }
      """
    When I run plumbing
    Then I get 2 warning(s)
    And the warning mentions `bar`

  Scenario: Nested call in return position triggers warnings
    Given the source code:
      """
      fn compute() -> ::core::primitive::u64 {
          foo(bar(x))
      }
      """
    When I run plumbing
    Then I get 1 warning(s)
    And the warning mentions `bar`

  Scenario: Nested call in struct field initialization triggers warnings
    Given the source code:
      """
      fn build() -> Item {
          Item {
              val: foo(bar(x)),
          }
      }
      """
    When I run plumbing
    Then I get 1 warning(s)
    And the warning mentions `bar`

  Scenario: Nested calls in let bindings trigger warnings
    Given the source code:
      """
      fn run() {
          let y = foo(bar(x));
      }
      """
    When I run plumbing
    Then I get 1 warning(s)
    And the warning mentions `bar`

  Scenario: Nested call inside macro arguments triggers warnings
    Given the source code:
      """
      fn run() {
          ::std::println!("{}", foo(bar(x)));
      }
      """
    When I run plumbing
    Then I get 2 warning(s)
    And the warning mentions `bar`

  Scenario: Nested turbofish call triggers warnings
    Given the source code:
      """
      fn run() {
          foo(bar::<::core::primitive::u32>(x));
      }
      """
    When I run plumbing
    Then I get 1 warning(s)
    And the warning mentions `bar`

  Scenario: Multiple nested call arguments trigger warnings for each
    Given the source code:
      """
      fn run() {
          foo(bar(x), baz(y));
      }
      """
    When I run plumbing
    Then I get 2 warning(s)
    And the warning mentions `bar`
