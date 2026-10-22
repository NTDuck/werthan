Feature: Synthesis and conflict negotiation
  As the macro statechart engine
  I want each module vertex to transition through the six micro-states
  To coordinate DAG transformation and warm start regressions

  Scenario: Successful synthesis transitions M2 AriadneMcts to M4 Hardened and emits ModuleHardened
    Given a new module cluster for "module_alpha"
    When the cluster advances through wake ready
    And the cluster advances through hydration completed
    And a candidate achieves unit reward with code "pub fn compute() -> i32 { 10 }"
    Then the micro-state is "M4_Hardened"
    And the module holds hardened status
    And the outbox contains broadcast event "ModuleHardened"

  Scenario: Extrinsic contract defect transitions M2 AriadneMcts to M3 ContractNegotiate
    Given a new module cluster for "module_alpha"
    When the cluster advances through wake ready
    And the cluster advances through hydration completed
    And the evaluator flags extrinsic contract conflict on parent "parent_core"
    Then the micro-state is "M3_ContractNegotiate"

  Scenario: Conflict dispatch returns module from M3 ContractNegotiate to M0 Blocked
    Given a new module cluster for "module_alpha"
    When the cluster advances through wake ready
    And the cluster advances through hydration completed
    And the evaluator flags extrinsic contract conflict on parent "parent_core"
    And the cluster advances through conflict dispatched
    Then the micro-state is "M0_Blocked"
    And the module holds blocked status
