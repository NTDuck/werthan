Feature: Enforce fully qualified paths (pathqual)

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

  Scenario: Fully qualified paths pass without warnings
    Given the source code:
      """
      use ::std::collections::HashMap;

      #[derive(::core::fmt::Debug)]
      pub struct SafeList {
          items: ::std::vec::Vec<::core::primitive::u64>,
      }

      pub fn log_item() {
          ::std::println!("safe");
      }
      """
    When I run pathqual
    Then I get 0 warning(s)

  Scenario: Method calls on instances pass without warnings
    Given the source code:
      """
      pub fn transform(val: ::std::string::String) -> ::core::primitive::usize {
          val.len()
      }
      """
    When I run pathqual
    Then I get 0 warning(s)

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

  Scenario: Fully qualified primitive types pass without warnings
    Given the source code:
      """
      pub fn add(x: ::core::primitive::i32, y: ::core::primitive::i32) -> ::core::primitive::i32 {
          x + y
      }
      """
    When I run pathqual
    Then I get 0 warning(s)

  Scenario: Allow self, super, and crate
    Given the source code:
      """
      impl ::my_crate::SafeList {
          pub fn self_test(&self) -> ::core::primitive::bool {
              true
          }
      }
      """
    When I run pathqual
    Then I get 0 warning(s)

  Scenario: Unqualified type in type alias triggers warning
    Given the source code:
      """
      type Counter = usize;
      """
    When I run pathqual
    Then I get 1 warning(s)
    And the warning mentions `usize`

  Scenario: Fully qualified type in type alias passes without warnings
    Given the source code:
      """
      type Counter = ::core::primitive::usize;
      """
    When I run pathqual
    Then I get 0 warning(s)

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

  Scenario: Fully qualified type in as cast passes without warnings
    Given the source code:
      """
      pub fn convert(val: ::core::primitive::usize) -> ::core::primitive::u64 {
          val as ::core::primitive::u64
      }
      """
    When I run pathqual
    Then I get 0 warning(s)

  Scenario: Inner crate attribute passes without warnings
    Given the source code:
      """
      #![allow(unused)]

      pub fn run() {}
      """
    When I run pathqual
    Then I get 0 warning(s)

  Scenario: Fully qualified macro call passes without warnings
    Given the source code:
      """
      pub fn make_vec() -> ::std::vec::Vec<::core::primitive::u8> {
          ::std::vec![1, 2, 3]
      }
      """
    When I run pathqual
    Then I get 0 warning(s)

  Scenario: Multi-parameter generic lists pass without warnings
    Given the source code:
      """
      pub struct TwoStageEvaluator<H, D> {
          harness: H,
          distiller: D,
      }

      impl<H, D> TwoStageEvaluator<H, D> {
          pub fn with_components(harness: H, distiller: D) -> Self {
              TwoStageEvaluator { harness, distiller }
          }
      }
      """
    When I run pathqual
    Then I get 0 warning(s)

  Scenario: Generic params survive statement semicolons inside bodies
    Given the source code:
      """
      pub struct Pair<H, D> {
          harness: H,
          distiller: D,
      }

      impl<H: ::core::default::Default, D: ::core::default::Default> ::core::default::Default for Pair<H, D> {
          fn default() -> Self {
              let harness = H::default();
              let distiller = D::default();
              Pair { harness, distiller }
          }
      }
      """
    When I run pathqual
    Then I get 0 warning(s)
