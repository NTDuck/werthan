Feature: Ast edit tool rewrites syntax patterns
  As the structural rewrite entry point
  I want pattern based code transformation
  So refactors apply across files consistently

  Scenario: Ast edit tool rewrites structural syntax pattern across files
    Given a temporary file named "test_ast_edit.rs" with content:
      """
      fn old_function_name() {}
      """
    When I call the ast edit tool on "test_ast_edit.rs" to rewrite "old_function_name" to "new_function_name"
    Then the ast edit tool call succeeds
    And the target file content contains "new_function_name"
    And the target file content does not contain "old_function_name"
