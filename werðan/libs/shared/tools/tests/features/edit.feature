Feature: Edit tool applies surgical patches
  As the text mutation entry point
  I want replacement and insertion operations
  So existing files change without full rewrites

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
