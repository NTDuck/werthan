Feature: STE100 compliant text passes

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
