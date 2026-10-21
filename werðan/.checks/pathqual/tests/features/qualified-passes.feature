Feature: Qualified path compliance

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

  Scenario: Fully qualified type in type alias passes without warnings
    Given the source code:
      """
      type Counter = ::core::primitive::usize;
      """
    When I run pathqual
    Then I get 0 warning(s)

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
