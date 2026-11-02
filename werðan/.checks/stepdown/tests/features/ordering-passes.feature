Feature: Stepdown compliant ordering

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

  Scenario: Single function with no calls passes without warnings
    Given the source code:
      """
      fn lone_function() {
          let x = 1 + 2;
      }
      """
    When I run stepdown
    Then I get 0 warning(s)

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

  Scenario: Public functions obeying stepdown pass without warnings
    Given the source code:
      """
      pub fn public_caller() {
          internal_callee();
      }
      fn internal_callee() {}
      """
    When I run stepdown
    Then I get 0 warning(s)

  Scenario: Async functions obeying stepdown pass without warnings
    Given the source code:
      """
      async fn fetch_all() {
          fetch_one().await;
      }
      async fn fetch_one() {}
      """
    When I run stepdown
    Then I get 0 warning(s)

  Scenario: In-order multiple callees pass without warnings
    Given the source code:
      """
      fn top_level() {
          step_one();
          step_two();
      }
      fn step_one() {}
      fn step_two() {}
      """
    When I run stepdown
    Then I get 0 warning(s)
