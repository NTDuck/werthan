Feature: Eval tool runs code snippets
  As the scratch evaluation entry point
  I want python snippet execution with stdout capture
  So callers verify behavior without files

  Scenario: Eval tool evaluates python code snippet
    When I call the eval tool with python code "print('cucumber_eval_' + 'verified')"
    Then the eval tool call succeeds
    And the eval stdout contains "cucumber_eval_verified"
