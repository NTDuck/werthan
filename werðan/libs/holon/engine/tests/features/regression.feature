Feature: Multi-tier conflict regression
  As the cluster controller runtime
  I want surgical regressions to escalate contract conflicts
  To guarantee behavioral parity with tiered macro-regression handling

  Scenario: Extrinsic contract defect triggers Tier B contract conflict escalation
    Given an initialized local blackboard with parent contract "parent_core"
    And an execution harness that flags extrinsic contract conflict
    When the controller executes search with rollout budget 3
    Then the search yields typed outcome "ContractConflict"
    And the held machine is in state "M3_ContractNegotiate"

  Scenario: Runtime dependency cycle triggers Tier C macro-regression
    Given an initialized local blackboard with parent contract "parent_cycle"
    And an execution harness that flags cyclic dependency
    When the controller executes search with rollout budget 3
    Then the search yields typed outcome "CycleFound"
    And the held machine emits broadcast event "CyclicDependencyFound"

  Scenario: Rollout budget exhaustion escalates blocker delta to parent
    Given an initialized local blackboard with parent contract "parent_core"
    And an execution harness that produces partial pass rate without unit reward
    When the controller executes search with rollout budget 2
    Then the search yields typed outcome "BudgetExhausted"
    And the held machine is in state "M3_ContractNegotiate"
