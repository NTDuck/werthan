Feature: Write tool persists bytes to disk
  As the file creation entry point
  I want durable writes of supplied content
  So downstream tools operate on stable artifacts

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
