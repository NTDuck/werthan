Feature: Github tool queries repository data
  As the repository access entry point
  I want GitHub search operations
  So callers pull repository details into context

  Scenario: Github tool searches repository details
    When I call the github tool with op "search_repos" and query "werthan"
    Then the github tool call succeeds
    And the github status is "success"
