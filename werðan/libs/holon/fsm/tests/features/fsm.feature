Feature: ARIADNE Cluster Micro-FSM
  As the macro statechart engine
  I want each module vertex to transition through the six micro-states
  To coordinate DAG transformation and warm start regressions

  Scenario: Module vertex starts in M0 Blocked waiting for upstream parents
    Given a new module cluster for "module_alpha"
    Then the micro-state is "M0_Blocked"
    And the module holds blocked status

  Scenario: Advance from M0 Blocked to M1 ContextHydration on wake ready
    Given a new module cluster for "module_alpha"
    When the cluster advances through wake ready
    Then the micro-state is "M1_ContextHydration"

  Scenario: Advance from M1 ContextHydration to M2 AriadneMcts on hydration completed
    Given a new module cluster for "module_alpha"
    When the cluster advances through wake ready
    And the cluster advances through hydration completed
    Then the micro-state is "M2_AriadneMcts"
    And the module holds active status

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

  Scenario: Downstream conflict invalidates M4 Hardened module to M5 Invalidated
    Given a new module cluster for "module_alpha"
    When the cluster advances through wake ready
    And the cluster advances through hydration completed
    And a candidate achieves unit reward with code "pub fn compute() -> i32 { 10 }"
    And downstream invalidation occurs with delta "requires param timeout: u64"
    Then the micro-state is "M5_Invalidated"
    And the activation mode is "WarmStartRegression"
    And the warm start count is 1

  Scenario: Warm start resume transitions M5 Invalidated back to M2 AriadneMcts with warm start mode
    Given a new module cluster for "module_alpha"
    When the cluster advances through wake ready
    And the cluster advances through hydration completed
    And a candidate achieves unit reward with code "pub fn compute() -> i32 { 10 }"
    And downstream invalidation occurs with delta "requires param timeout: u64"
    And the cluster advances through warm start ready
    Then the micro-state is "M2_AriadneMcts"
    And the activation mode is "WarmStartRegression"

  Scenario: Undocumented cyclic dependency emits CyclicDependencyFound regression event
    Given a new module cluster for "module_alpha"
    When the cluster advances through wake ready
    And the cluster advances through hydration completed
    And the evaluator flags cyclic dependency involving parent "parent_cycle"
    Then the outbox contains broadcast event "CyclicDependencyFound"

  Scenario: Held state machine type changes across typed transitions
    Given a new module cluster for "module_alpha"
    When the cluster advances through wake ready
    Then the held machine type is "M1_ContextHydration"
