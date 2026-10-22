Feature: Warm start regression recovery
  As the macro statechart engine
  I want each module vertex to transition through the six micro-states
  To coordinate DAG transformation and warm start regressions

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
