Feature: Glob tool discovers matching files
  As the filesystem enumeration entry point
  I want pattern based file discovery
  So downstream steps reference real paths

  Scenario: Glob tool discovers matching files in crate
    When I call the glob tool with pattern "src/*.rs"
    Then the glob tool call succeeds
    And the glob result contains "src/lib.rs"
