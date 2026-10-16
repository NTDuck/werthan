Feature: OMP Bridge Rig Agent Conformance
  Scenario: Build a native Rig agent from OMP completion model
    Given an OMP completion model configured for no-memory and high quality
    When I build a native Rig agent with preamble "You are a specialist"
    Then the agent is successfully instantiated

  Scenario: Rig Agent prompt execution
    Given an OMP completion model configured for no-memory and high quality
    When I build a native Rig agent with preamble "You are a specialist"
    And I prompt the agent with "Hello"
    Then the agent returns a valid response

  Scenario: Rig Agent streaming execution
    Given an OMP completion model configured for no-memory and high quality
    When I build a native Rig agent with preamble "You are a specialist"
    And I stream the prompt "Explain Rust"
    Then the agent stream yields output events
