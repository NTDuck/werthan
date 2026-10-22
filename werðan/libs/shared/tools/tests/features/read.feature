Feature: Read tool surfaces file contents
  As the file inspection entry point
  I want exact line counts from existing files
  To ground subsequent reasoning in verbatim source

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
