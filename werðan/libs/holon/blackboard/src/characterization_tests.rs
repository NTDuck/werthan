//! Characterization tests slice B_CT per METHODOLOGY.md Definition 2.
//! Holds black-box I/O fixtures, property-based invariant assertions, and
//! falsifying test inputs derived from legacy behavior.

/// Characterization tests slice containing black-box fixtures.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default, ::serde::Serialize, ::serde::Deserialize)]
pub struct CharacterizationTestsSlice {
    /// Executable test fixtures and unit assertions.
    pub test_cases: ::std::vec::Vec<::std::string::String>,
    /// Property-based invariant assertions.
    pub property_assertions: ::std::vec::Vec<::std::string::String>,
}

impl self::CharacterizationTestsSlice {
    /// Create empty characterization tests slice.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add unit test case to characterization test suite.
    pub fn add_test_case(&mut self, test_code: ::std::string::String) {
        self.test_cases.push(test_code);
    }

    /// Add property assertion to characterization test suite.
    pub fn add_property_assertion(&mut self, assertion: ::std::string::String) {
        self.property_assertions.push(assertion);
    }

    /// Count total characterization tests in slice.
    pub fn test_count(&self) -> ::core::primitive::usize {
        self.test_cases.len()
    }

    /// Return true when the characterization test suite contains test cases.
    pub fn is_populated(&self) -> ::core::primitive::bool {
        !self.test_cases.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_characterization_suite_operations() {
        let mut slice = self::CharacterizationTestsSlice::new();
        ::std::assert!(!slice.is_populated());
        slice.add_test_case("assert_eq!(parse(\"a\"), tree_a)".to_string());
        slice.add_property_assertion("roundtrip(parse(s)) == s".to_string());
        ::std::assert!(slice.is_populated());
        ::std::assert_eq!(slice.test_count(), 1);
        ::std::assert_eq!(slice.test_cases.len(), 1);
        ::std::assert_eq!(slice.property_assertions.len(), 1);
    }
}
