Feature: Grep tool searches file contents
  As the text search entry point
  I want regular expression matches across files
  So callers locate code by content

  Scenario: Grep tool searches regular expression pattern in files
    When I call the grep tool with pattern "Tool for" in path "src/read.rs"
    Then the grep tool call succeeds
    And the grep matches count is at least 1
