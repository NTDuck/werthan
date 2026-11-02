Feature: Legacy mod.rs detection

  Scenario: Legacy mod.rs file triggers warning
    Given the file path `src/sub/mod.rs` and source code:
      """
      pub fn util() {}
      """
    When I run flatmods
    Then I get 1 warning(s)
    And the warning mentions `legacy`

  Scenario: Deeply nested legacy mod.rs file triggers warning
    Given the file path `src/deep/nested/package/mod.rs` and source code:
      """
      pub fn deep_fn() {}
      """
    When I run flatmods
    Then I get 1 warning(s)
    And the warning mentions `legacy`

  Scenario: Sub-module declaration in legacy mod.rs triggers warning
    Given the file path `src/sub/mod.rs` and source code:
      """
      mod child;
      """
    When I run flatmods
    Then I get 2 warning(s)
    And the warning mentions `declare sub-module `child``

  Scenario: Multiple sub-module declarations in mod.rs trigger warnings for each
    Given the file path `src/sub/mod.rs` and source code:
      """
      pub mod alpha;
      pub mod beta;
      """
    When I run flatmods
    Then I get 3 warning(s)
    And the warning mentions `declare sub-module `alpha``

  Scenario: Inline module with braces in mod.rs only triggers legacy file warning
    Given the file path `src/sub/mod.rs` and source code:
      """
      mod inline {
          pub fn inner() {}
      }
      """
    When I run flatmods
    Then I get 1 warning(s)
    And the warning mentions `legacy`
