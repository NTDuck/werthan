Feature: Enforce modern flat module structure (flatmods)

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

  Scenario: Modern module declaration directly in main.rs passes without warnings
    Given the file path `src/main.rs` and source code:
      """
      mod foo;
      mod bar;

      fn main() {}
      """
    When I run flatmods
    Then I get 0 warning(s)

  Scenario: Modern module declaration directly in lib.rs passes without warnings
    Given the file path `src/lib.rs` and source code:
      """
      pub mod foo;
      pub mod bar;
      """
    When I run flatmods
    Then I get 0 warning(s)

  Scenario: Flat module file passes without warnings
    Given the file path `src/foo.rs` and source code:
      """
      pub fn hello() {}
      """
    When I run flatmods
    Then I get 0 warning(s)

  Scenario: Nested module file in subdirectory passes without warnings
    Given the file path `src/foo/bar.rs` and source code:
      """
      pub fn sub_item() {}
      """
    When I run flatmods
    Then I get 0 warning(s)

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

  Scenario: Sub-module declaration in flat file passes without warnings
    Given the file path `src/parent.rs` and source code:
      """
      pub mod child;
      """
    When I run flatmods
    Then I get 0 warning(s)
