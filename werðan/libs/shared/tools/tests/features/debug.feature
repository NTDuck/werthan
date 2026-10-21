Feature: Debug tool inspects debug adapters
  As the debugger entry point
  I want adapter status inspection
  So callers confirm debug infrastructure readiness

  Scenario: Debug tool inspects adapter status
    When I call the debug tool with action "status"
    Then the debug tool call succeeds
    And the debug status is "ok"
