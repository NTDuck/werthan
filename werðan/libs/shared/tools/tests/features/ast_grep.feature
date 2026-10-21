Feature: Ast grep tool discovers syntax patterns
  As the structural search entry point
  I want tree pattern matches on real syntax
  So callers find code by shape, not text

  Scenario: Ast grep tool discovers structural syntax patterns
    Given a temporary file named "test_ast.rs" with content:
      """
      fn compute_value() -> i32 {
          100
      }
      """
    When I call the ast grep tool for "fn compute_value" in "test_ast.rs"
    Then the ast grep tool call succeeds
    And the ast grep matches count is at least 1
