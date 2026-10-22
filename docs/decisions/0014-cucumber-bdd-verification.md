---
status: "accepted"
date: 2026-10-22
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
---

# Cucumber BDD Verification of Methodology Behavior Parity

## Context and Problem Statement

Unit tests verify functions; they do not verify that the engine's observable behavior matches the formal methodology in `docs/references/METHODOLOGY.md` (pinned by [ADR-0005](0005-methodology-source-of-truth.md)). Monolithic Gherkin feature files also mix unrelated behaviors, so one failing scenario obscures which behavior regressed. Behavior parity needs executable specifications that read as methodology language.

How does the workspace verify engine and linter behavior against the methodology with executable, behavior-scoped specifications?

## Decision Drivers

* Methodology traceability: scenarios use Gherkin `Given/When/Then` language that maps to METHODOLOGY.md definitions.
* Behavior-scoped isolation: one feature file per behavior; a red scenario names its regression precisely.
* Coverage across tiers: both the holon engine (search, reward, regression) and the `.checks` linters carry cucumber suites.
* Monolith split: large feature files divide along behavior lines, not implementation lines.

## Considered Options

* Unit tests only, with prose review against the methodology.
* One monolithic feature file per crate.
* Per-behavior Gherkin feature files in both the engine crates and the linter crates, run by cucumber.

## Decision Outcome

Chosen option: "Gherkin cucumber suites verify behavior parity with METHODOLOGY.md — engine/fsm features in holon-engine and per-linter features in `.checks`; monolithic feature files split per behavior", because executable scenarios tie each methodology behavior to a test that names it.

Feature inventory:

| Suite | Feature files |
| :--- | :--- |
| holon-engine | `search.feature`, `reward.feature`, `regression.feature` |
| `.checks` linters (e.g. `flatmods`) | `legacy-layout.feature`, `modern-layout.feature` |

* `search.feature` exercises MCTS search behavior; `reward.feature` pins the gated scalar reward semantics; `regression.feature` verifies warm-start and invalidation transitions.
* Linter features verify that legacy-layout and modern-layout code each lint under their respective rules.
* Monolithic feature files are split so each file owns exactly one behavior.

### Consequences

* Good, because a failing scenario identifies the methodology behavior that regressed, not just a function.
* Good, because feature files serve as executable documentation of the methodology's behavioral claims.
* Bad, because Gherkin scenarios need maintenance whenever behavior intentionally changes; the change goes through an ADR first.
* Neutral, because unit tests remain the tool for function-level edge cases; cucumber owns behavior-level parity.

### Confirmation

`cargo test --workspace` runs the cucumber suites; current totals are 49 unit tests plus 18 cucumber scenarios, all green. Each feature file's scenarios cite the METHODOLOGY.md definition they exercise. Feature files stay one-behavior-per-file — reviewed when new features are added.

## More Information

Evidence: commits c8dec2d, 92fad48, b94112f. Session evidence: "HOLON... all tests green (49 unit + 18 cucumber)".

The statechart behaviors under test live in the architecture of [ADR-0010](0010-holon-modular-architecture.md); the reward semantics the suites pin are calibrated in [ADR-0012](0012-ariadne-constant-calibration.md).
