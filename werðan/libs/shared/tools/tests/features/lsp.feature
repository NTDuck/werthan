Feature: LSP tool drives language intelligence
  As the language server entry point
  I want status, symbols, diagnostics, and renames
  So callers query and mutate code through the compiler service

  Scenario: LSP tool returns status of language servers
    When I call the lsp tool with action "status"
    Then the lsp tool call succeeds
    And the lsp status is "ok"
    And the lsp output contains server "rust-analyzer"

  Scenario: LSP tool discovers symbols in source file
    When I call the lsp tool for symbols matching "DisplayExt" in "src/lib.rs"
    Then the lsp tool call succeeds
    And the lsp status is "ok"

  Scenario: LSP tool identifies syntax diagnostics
    Given a temporary file named "test_bad_syntax.rs" with content:
      """
      fn broken() {
          let number = (1 + 2;
      }
      """
    When I call the lsp tool for diagnostics on "test_bad_syntax.rs"
    Then the lsp tool call succeeds
    And the lsp status is "ok"
    And the lsp output contains diagnostic message "Mismatched closing delimiter"

  Scenario: LSP tool performs identifier rename across file
    Given a temporary file named "test_rename.rs" with content:
      """
      let user_count = 10;
      println!("{}", user_count);
      """
    When I call the lsp tool to rename "user_count" to "active_users" in "test_rename.rs"
    Then the lsp tool call succeeds
    And the lsp status is "ok"
    And the target file content contains "active_users"
    And the target file content does not contain "user_count"
