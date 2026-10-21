Feature: ARIADNE Cluster MCTS Controller
  As the cluster controller runtime
  I want Monte Carlo Tree Search to govern intra-module code synthesis
  To guarantee behavioral parity with compiler-gated rewards and surgical regressions

  Scenario: Initial rollout seeds characterization tests and synthesizes hardened module
    Given an initialized local blackboard for module "module_math"
    When the controller executes search with rollout budget 5
    Then the search yields typed outcome "Hardened"
    And the held machine is in state "M4_Hardened"
    And the tree contains candidate nodes

  Scenario: Gated reward assigns zero score to uncompilable candidate
    Given an initialized local blackboard for module "module_math"
    When the controller evaluates uncompilable candidate code
    Then the computed reward equals 0.0

  Scenario: Full test pass rate and rule coverage yields unit reward
    Given an initialized local blackboard for module "module_math"
    When the controller evaluates candidate code with 1.0 test pass rate and 1.0 rule score
    Then the computed reward equals 1.0

  Scenario: Softmax-UCB selects child node based on exploration and prior utility
    Given an initialized search tree with parent and two child candidates
    When the tree performs Softmax-UCB selection with temperature 0.5
    Then the selection identifies an expandable child node

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

  Scenario: Warm start regression preserves existing search tree and prior nodes
    Given an initialized local blackboard for module "module_math"
    When the controller executes search with rollout budget 5
    And downstream invalidation shifts activation mode to "WarmStartRegression"
    And the controller executes search under warm start mode
    Then the search tree preserves existing nodes across rollouts

  Scenario: Rollout budget exhaustion escalates blocker delta to parent
    Given an initialized local blackboard with parent contract "parent_core"
    And an execution harness that produces partial pass rate without unit reward
    When the controller executes search with rollout budget 2
    Then the search yields typed outcome "BudgetExhausted"
    And the held machine is in state "M3_ContractNegotiate"
