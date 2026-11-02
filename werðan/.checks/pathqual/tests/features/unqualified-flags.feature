Feature: Unqualified path detection

  Scenario: Unqualified standard types trigger warnings
    Given the source code:
      """
      fn make_list() -> Vec<::core::primitive::u64> {
          Vec::new()
      }
      """
    When I run pathqual
    Then I get 2 warning(s)
    And the warning mentions `Vec`

  Scenario: Unqualified macro invocations trigger warnings
    Given the source code:
      """
      fn hello() {
          println!("Hello world");
      }
      """
    When I run pathqual
    Then I get 1 warning(s)
    And the warning mentions `println`

  Scenario: Unqualified traits in derive attributes trigger warnings
    Given the source code:
      """
      #[derive(Debug, Clone)]
      struct User {
          name: ::std::string::String,
      }
      """
    When I run pathqual
    Then I get 2 warning(s)
    And the warning mentions `Debug`

  Scenario: Unqualified traits in bounds trigger warnings
    Given the source code:
      """
      fn process<T: Clone>(item: ::core::primitive::u64) {}
      """
    When I run pathqual
    Then I get 1 warning(s)
    And the warning mentions `Clone`

  Scenario: Unqualified use imports trigger warnings
    Given the source code:
      """
      use std::collections::HashMap;
      """
    When I run pathqual
    Then I get 1 warning(s)
    And the warning mentions `std`

  Scenario: Unqualified nested use group imports trigger warnings
    Given the source code:
      """
      use ::std::{fmt::Debug};
      """
    When I run pathqual
    Then I get 1 warning(s)
    And the warning mentions `fmt`

  Scenario: Unqualified struct fields trigger warnings
    Given the source code:
      """
      struct Container {
          data: String,
      }
      """
    When I run pathqual
    Then I get 1 warning(s)
    And the warning mentions `String`

  Scenario: Unqualified where clause bounds trigger warnings
    Given the source code:
      """
      impl<T> ::my_crate::Worker<T> where ::core::primitive::u64: Display {}
      """
    When I run pathqual
    Then I get 1 warning(s)
    And the warning mentions `Display`

  Scenario: Unqualified primitive types trigger warnings
    Given the source code:
      """
      pub fn add(x: i32, y: i32) -> i32 {
          x + y
      }
      """
    When I run pathqual
    Then I get 3 warning(s)
    And the warning mentions `i32`

  Scenario: Unqualified type in type alias triggers warning
    Given the source code:
      """
      type Counter = usize;
      """
    When I run pathqual
    Then I get 1 warning(s)
    And the warning mentions `usize`

  Scenario: Unqualified type in as cast triggers warning
    Given the source code:
      """
      pub fn convert(val: ::core::primitive::usize) -> ::core::primitive::u64 {
          val as u64
      }
      """
    When I run pathqual
    Then I get 1 warning(s)
    And the warning mentions `u64`
