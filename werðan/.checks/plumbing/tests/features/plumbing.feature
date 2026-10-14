Feature: Enforce chaining nested function calls (plumbing)

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

  Scenario: Method chains pass without warnings
    Given the source code:
      """
      fn run() {
          x.foo().bar().baz();
      }
      """
    When I run plumbing
    Then I get 0 warning(s)

  Scenario: Single level calls pass without warnings
    Given the source code:
      """
      fn run() {
          foo(x);
      }
      """
    When I run plumbing
    Then I get 0 warning(s)

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
