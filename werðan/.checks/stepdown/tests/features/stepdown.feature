Feature: Enforce Clean Code Stepdown Rule (stepdown)

  Scenario: Callee defined before caller triggers warning
    Given the source code:
      """
      fn callee() {}
      fn caller() {
          callee();
      }
      """
    When I run stepdown
    Then I get 1 warning(s)
    And the warning mentions `stepdown rule violation`

  Scenario: Caller before callee passes without warnings
    Given the source code:
      """
      fn caller() {
          callee();
      }
      fn callee() {}
      """
    When I run stepdown
    Then I get 0 warning(s)

  Scenario: Callee ordering violation triggers warning
    Given the source code:
      """
      fn top() {
          second();
          first();
      }
      fn first() {}
      fn second() {}
      """
    When I run stepdown
    Then I get 1 warning(s)
    And the warning mentions `stepdown rule violation`

  Scenario: Deep transitive violation triggers warnings
    Given the source code:
      """
      fn leaf() {}
      fn mid() {
          leaf();
      }
      fn root() {
          mid();
      }
      """
    When I run stepdown
    Then I get 2 warning(s)
    And the warning mentions `stepdown rule violation`

  Scenario: Single function with no calls passes without warnings
    Given the source code:
      """
      fn lone_function() {
          let x = 1 + 2;
      }
      """
    When I run stepdown
    Then I get 0 warning(s)

  Scenario: Mutually recursive functions trigger caller-before-callee warning for backward call
    Given the source code:
      """
      fn ping() {
          pong();
      }
      fn pong() {
          ping();
      }
      """
    When I run stepdown
    Then I get 1 warning(s)
    And the warning mentions `stepdown rule violation`

  Scenario: Scope nested containers and methods independently
    Given the source code:
      """
      impl Foo {
          fn outer_caller(&self) {
              self.outer_callee();
          }
          fn outer_callee(&self) {}
      }
      """
    When I run stepdown
    Then I get 0 warning(s)
