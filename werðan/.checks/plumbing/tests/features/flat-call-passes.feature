Feature: Chain compliant call shapes

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

  Scenario: Method call as argument passes without warnings
    Given the source code:
      """
      fn run() {
          foo(x.bar());
      }
      """
    When I run plumbing
    Then I get 0 warning(s)
