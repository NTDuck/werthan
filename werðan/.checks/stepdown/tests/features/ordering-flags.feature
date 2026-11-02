Feature: Callee-before-caller detection

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

  Scenario: Async callee before caller triggers warning
    Given the source code:
      """
      async fn fetch_one() {}
      async fn fetch_all() {
          fetch_one().await;
      }
      """
    When I run stepdown
    Then I get 1 warning(s)
    And the warning mentions `stepdown rule violation`

  Scenario: Callee with turbofish called before definition triggers warning
    Given the source code:
      """
      fn parse_item<T>() {}
      fn load_all() {
          parse_item::<::core::primitive::u32>();
      }
      """
    When I run stepdown
    Then I get 1 warning(s)
    And the warning mentions `stepdown rule violation`
