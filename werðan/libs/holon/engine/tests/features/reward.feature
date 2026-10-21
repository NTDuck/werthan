Feature: Gated hierarchical reward
  As the cluster controller runtime
  I want compiler-gated rewards to score candidate code synthesis
  To guarantee behavioral parity with test pass rate and rule coverage

  Scenario: Gated reward assigns zero score to uncompilable candidate
    Given an initialized local blackboard for module "module_math"
    When the controller evaluates uncompilable candidate code
    Then the computed reward equals 0.0

  Scenario: Full test pass rate and rule coverage yields unit reward
    Given an initialized local blackboard for module "module_math"
    When the controller evaluates candidate code with 1.0 test pass rate and 1.0 rule score
    Then the computed reward equals 1.0
