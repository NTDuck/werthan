Feature: STE100 style rule detection

  Scenario: Semicolon in message triggers warning
    Given the source code:
      """
      fn log_status() {
          ::std::println!("First message; second message");
      }
      """
    When I run ste100
    Then I get 1 warning(s)
    And the warning mentions `STE bans the semicolon`

  Scenario: Phrasal verb in doc comment triggers warning
    Given the source code:
      """
      /// Please spin up the worker process before continuing.
      pub fn start() {}
      """
    When I run ste100
    Then I get 1 warning(s)
    And the warning mentions `Soft phrasal verb`

  Scenario: Marketing adjective in comment triggers warning
    Given the source code:
      """
      // This is a robust implementation.
      pub fn process() {}
      """
    When I run ste100
    Then I get 1 warning(s)
    And the warning mentions `Marketing adjective`

  Scenario: Nominalization in message triggers warning
    Given the source code:
      """
      fn analyze() {
          ::std::println!("perform an analysis of data");
      }
      """
    When I run ste100
    Then I get 1 warning(s)
    And the warning mentions `Action frozen into a noun`

  Scenario: Long sentence in doc comment triggers warning
    Given the source code:
      """
      /// This is an excessively long and detailed description of the function that goes on and on without stopping to explain every minor detail until the word limit is completely exceeded.
      pub fn long_doc() {}
      """
    When I run ste100
    Then I get 1 warning(s)
    And the warning mentions `Sentence has 30 words`
