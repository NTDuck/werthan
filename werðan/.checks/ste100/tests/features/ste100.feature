Feature: Enforce ASD-STE100 compliance (ste100)

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

  Scenario: Passive voice triggers advisory warning
    Given the source code:
      """
      /// The panel is removed before inspection.
      pub fn inspect() {}
      """
    When I run ste100
    Then I get 1 warning(s)
    And the warning mentions `Possible passive voice`

  Scenario: Present perfect triggers advisory warning
    Given the source code:
      """
      /// We have received the report from the controller.
      pub fn report() {}
      """
    When I run ste100
    Then I get 1 warning(s)
    And the warning mentions `Compound tense`

  Scenario: Hedges are not flagged
    Given the source code:
      """
      /// The request may have failed due to timeout.
      pub fn check_hedge() {}
      """
    When I run ste100
    Then I get 0 warning(s)

  Scenario: Compliant messages and doc comments pass without warnings
    Given the source code:
      """
      /// Enforce valid module structure.
      ///
      /// Call this function in the build script:
      /// ```no_run
      /// ::flatmods::check();
      /// ```
      pub fn run() {
          ::std::println!("cargo:warning=Operation completed successfully.");
      }
      """
    When I run ste100
    Then I get 0 warning(s)
