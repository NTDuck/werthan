Feature: STE100 grammar rule detection

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

  Scenario: Raw string literal containing semicolon triggers warning
    Given the source code:
      """
      fn log_raw() {
          let _ = r#"First message; second message"#;
      }
      """
    When I run ste100
    Then I get 1 warning(s)
    And the warning mentions `STE bans the semicolon`

  Scenario: Block comment containing marketing adjective triggers warning
    Given the source code:
      """
      /* This is a robust implementation. */
      pub fn process() {}
      """
    When I run ste100
    Then I get 1 warning(s)
    And the warning mentions `Marketing adjective`

  Scenario: Inner doc comment containing nominalization triggers warning
    Given the source code:
      """
      //! Please perform an analysis of data before running.
      pub fn init() {}
      """
    When I run ste100
    Then I get 1 warning(s)
    And the warning mentions `Action frozen into a noun`

  Scenario: Multiple string literals on the same line trigger warnings
    Given the source code:
      """
      fn log_both() {
          let _ = ("First; part", "Second; part");
      }
      """
    When I run ste100
    Then I get 2 warning(s)
    And the warning mentions `STE bans the semicolon`
