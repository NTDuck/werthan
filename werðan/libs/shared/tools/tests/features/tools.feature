Feature: Behavioural integrity of shared tools conforming to omp specification

  Scenario: Read tool reads existing file contents and returns exact lines count
    Given a temporary file named "test_read.txt" with content:
      """
      first line
      second line
      third line
      """
    When I call the read tool for "test_read.txt"
    Then the read tool call succeeds
    And the read content contains "second line"
    And the read lines count is 3

  Scenario: Write tool creates a new file and persists bytes
    Given a target file path "test_write.txt"
    When I call the write tool with content:
      """
      hello from cucumber test
      multiline written content
      """
    Then the write tool call succeeds
    And the target file exists on disk
    And the target file content contains "hello from cucumber test"

  Scenario: Edit tool applies replacement patch to target file
    Given a temporary file named "test_edit.txt" with content:
      """
      line 1
      line 2 original
      line 3
      """
    When I call the edit tool on "test_edit.txt" with patch:
      """
      PUT 2.=2:
      +line 2 patched
      """
    Then the edit tool call succeeds
    And the target file content contains "line 2 patched"
    And the target file content does not contain "line 2 original"

  Scenario: Edit tool inserts content before line
    Given a temporary file named "test_insert_before.txt" with content:
      """
      alpha
      gamma
      """
    When I call the edit tool on "test_insert_before.txt" with patch:
      """
      PUT <2:
      +beta
      """
    Then the edit tool call succeeds
    And the target file content contains "beta"

  Scenario: Edit tool inserts content after line
    Given a temporary file named "test_insert_after.txt" with content:
      """
      first
      second
      """
    When I call the edit tool on "test_insert_after.txt" with patch:
      """
      PUT >2:
      +third
      """
    Then the edit tool call succeeds
    And the target file content contains "third"

  Scenario: Glob tool discovers matching files in crate
    When I call the glob tool with pattern "src/*.rs"
    Then the glob tool call succeeds
    And the glob result contains "src/lib.rs"

  Scenario: Grep tool searches regular expression pattern in files
    When I call the grep tool with pattern "Tool for" in path "src/read.rs"
    Then the grep tool call succeeds
    And the grep matches count is at least 1

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

  Scenario: Ast edit tool rewrites structural syntax pattern across files
    Given a temporary file named "test_ast_edit.rs" with content:
      """
      fn old_function_name() {}
      """
    When I call the ast edit tool on "test_ast_edit.rs" to rewrite "old_function_name" to "new_function_name"
    Then the ast edit tool call succeeds
    And the target file content contains "new_function_name"
    And the target file content does not contain "old_function_name"

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

  Scenario: Bash tool executes command and captures output
    When I call the bash tool with command "echo cucumber_bash_verified"
    Then the bash tool call succeeds
    And the bash stdout contains "cucumber_bash_verified"

  Scenario: Eval tool evaluates python code snippet
    When I call the eval tool with python code "print('cucumber_eval_' + 'verified')"
    Then the eval tool call succeeds
    And the eval stdout contains "cucumber_eval_verified"

  Scenario: Debug tool inspects adapter status
    When I call the debug tool with action "status"
    Then the debug tool call succeeds
    And the debug status is "ok"

  Scenario: Github tool searches repository details
    When I call the github tool with op "search_repos" and query "werthan"
    Then the github tool call succeeds
    And the github status is "success"
