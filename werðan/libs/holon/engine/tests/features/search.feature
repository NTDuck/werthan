Feature: MCTS search tree exploration
  As the cluster controller runtime
  I want Monte Carlo Tree Search to govern intra-module code synthesis
  To guarantee behavioral parity with compiler-gated rewards

  Scenario: Initial rollout seeds characterization tests and synthesizes hardened module
    Given an initialized local blackboard for module "module_math"
    When the controller executes search with rollout budget 5
    Then the search yields typed outcome "Hardened"
    And the held machine is in state "M4_Hardened"
    And the tree contains candidate nodes

  Scenario: Softmax-UCB selects child node based on exploration and prior utility
    Given an initialized search tree with parent and two child candidates
    When the tree performs Softmax-UCB selection with temperature 0.5
    Then the selection identifies an expandable child node

  Scenario: Warm start regression preserves existing search tree and prior nodes
    Given an initialized local blackboard for module "module_math"
    When the controller executes search with rollout budget 5
    And downstream invalidation shifts activation mode to "WarmStartRegression"
    And the controller executes search under warm start mode
    Then the search tree preserves existing nodes across rollouts
