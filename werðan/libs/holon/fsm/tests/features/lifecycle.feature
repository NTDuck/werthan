Feature: Micro-state lifecycle transitions
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

  Scenario: Held state machine type changes across typed transitions
    Given a new module cluster for "module_alpha"
    When the cluster advances through wake ready
    Then the held machine type is "M1_ContextHydration"
