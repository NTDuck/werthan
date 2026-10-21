Feature: Bash tool executes shell commands
  As the process execution entry point
  I want command output capture
  So callers run build and inspection commands

  Scenario: Bash tool executes command and captures output
    When I call the bash tool with command "echo cucumber_bash_verified"
    Then the bash tool call succeeds
    And the bash stdout contains "cucumber_bash_verified"
