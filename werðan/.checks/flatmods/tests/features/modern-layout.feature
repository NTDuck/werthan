Feature: Modern flat module compliance

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

  Scenario: Sub-module declaration in flat file passes without warnings
    Given the file path `src/parent.rs` and source code:
      """
      pub mod child;
      """
    When I run flatmods
    Then I get 0 warning(s)
