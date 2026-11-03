---
status: "accepted"
date: 2026-11-03
decision-makers: ["NTDuck", "omp"]
---

# RepoMod-Bench Implementation-Agnostic Benchmark Integration

## Context and Problem Statement

The system previously evaluated modernization capabilities exclusively against the ReCodeAgent benchmark suite (specifically CRUST, Skel, Oxidizer, and Alphatrans subsets spanning 118 projects) using configurations in `benchmarks/assets/configs/ReCodeAgent/`.

While ReCodeAgent provided valuable initial telemetry, empirical evaluations, benchmark forensics, and research council findings highlighted three structural limitations:
1. **Repository Scale and Real-World Topology**: The majority of ReCodeAgent projects are micro-benchmarks ($\le 2\text{k}$ LOC) with synthetic single-crate or shallow module hierarchies. They fail to reflect the architectural scale, dependency entanglements, and interface boundaries of enterprise codebases.
2. **Test Exposure, Shortcut Generation, and Benchmark Gaming**: Target test fixtures and characterization suites in ReCodeAgent reside directly within the translated workspace directory (`tests/`, `src/lib.rs`). Even with strict repair-pool exclusion invariants under Gate 5 ($\text{TargetTestSuite} \cap \text{RepairCandidatePool} = \emptyset$), exposing tests to generative agents risks circular validation, test-driven shortcut synthesis, or assertion tailoring that inflates nominal pass rates without achieving true functional equivalence.
3. **White-Box AST Coupling**: Evaluation tied to internal unit-test symbol names forces target code to replicate legacy internal abstractions rather than enabling idiomatic restructuring under Evidence-Gated Adaptive Graph Migration (EGAGM, [ADR-0019](0019-methodology2-egagm.md)).

RepoMod-Bench (arXiv:2602.22518: *"RepoMod-Bench: A Benchmark for Code Repository Modernization via Implementation-Agnostic Testing"*, KDD '26, Xuefeng Li, Nir Ben-Israel, Yotam Raz, Belal Ahmed, Doron Serebro, and Antoine Raux) addresses these challenges:
- **Scale**: 21 repository-scale benchmarks across 8 languages (1.6M total LOC, 11,616 tests), including 11 benchmarks targeting Rust (ranging from 19K to 191K LOC).
- **Implementation-Agnostic Evaluation**: Ground-truth tests reside outside the agent workspace in hidden test suites (`tests/`), evaluating behavior strictly via black-box interactions with the target binary (CLI stdin/stdout/exit status or REST endpoints).
- **Architectural Anti-Gaming**: The agent is provided only legacy source code (`src/`) and task requirements, with no read or write access to the test suite.

How should werðan integrate RepoMod-Bench into its benchmark harness, configuration hierarchy, and evaluation pipeline?

## Decision Drivers

* **Repository-Scale Modernization**: Validating EGAGM ([ADR-0019](0019-methodology2-egagm.md)) against authentic, multi-thousand-LOC codebases with complex build configurations and dependencies.
* **Implementation-Agnostic Black-Box Testing**: Assessing behavioral equivalence through public interface contracts ($Obs(P_s, x, \sigma) \equiv Obs(P_t, \phi(x), \phi(\sigma))$) rather than white-box unit-test symbol matching.
* **Anti-Gaming and Gate 5 Invariant Enforcement**: Enforcing $\text{TargetTestSuite} \cap \text{RepairCandidatePool} = \emptyset$ at the physical filesystem level by hiding evaluation tests entirely from agent context and tool execution spaces.
* **Regression and Ablation Parity**: Retaining existing ReCodeAgent benchmark assets and configs (`benchmarks/assets/configs/ReCodeAgent/`) to preserve historical baselines, ablation studies, and comparative evaluations.
* **Clean Configuration Layout**: Establishing `benchmarks/assets/configs/RepoMod-bench/` with campaign configs that conform to canonical `configschema::CampaignConfig` ([ADR-0009](0009-canonical-campaign-configschema.md)).

## Considered Options

* **Option 1: Complete Cutover to RepoMod-Bench (Deprecating ReCodeAgent)**
  - Replace all ReCodeAgent configurations, dataset assets, and harness paths with RepoMod-Bench.
  - *Shortcomings*: Destroys historical regression comparability, invalidates existing campaign logs, and eliminates the lightweight micro-benchmark testbed used for rapid local iteration.
* **Option 2: Dual-Harness Integration (Adopt RepoMod-Bench alongside ReCodeAgent)**
  - Retain existing ReCodeAgent configurations in `benchmarks/assets/configs/ReCodeAgent/` and assets in `benchmarks/assets/ReCodeAgent/` for regression and ablation parity.
  - Introduce `benchmarks/assets/configs/RepoMod-bench/` for RepoMod-Bench campaign configurations.
  - Extend the benchmark harness to support implementation-agnostic black-box testing and workspace mapping for `benchmarks/outputs/RepoMod-bench/...`.
  - Select `jmespath` (Go $\to$ Rust, 19.3K LOC, 888 tests) as the pilot Rust-target modernization candidate.

## Decision Outcome

Chosen option: **Option 2: Dual-Harness Integration (Adopt RepoMod-Bench alongside ReCodeAgent)**.

### Core Architectural Tenets

1. **Dual Benchmark Asset & Configuration Coexistence**:
   - ReCodeAgent configs remain in `benchmarks/assets/configs/ReCodeAgent/` (`crust/`, `skel/`, `oxidizer/`, `alphatrans/`).
   - RepoMod-Bench configs reside in `benchmarks/assets/configs/RepoMod-bench/`.
   - The `--benchmark-all` batch runner accepts explicit configuration directories to target either suite independently.

2. **Workspace Reset & Layout Mapping**:
   - ReCodeAgent workspaces map `benchmarks/outputs/<category>/<project>/<lang>` to `benchmarks/assets/ReCodeAgent/data/tool_projects/<category>/<project>/<lang>`.
   - RepoMod-Bench workspaces map `benchmarks/outputs/RepoMod-bench/<project>/<lang>` to the benchmark dataset at `benchmarks/assets/RepoMod-bench/dataset/<project>/`. The source repository is populated in `src/`, and the agent generates into `dst/` (or target workspace root).

3. **Implementation-Agnostic Black-Box Evaluation**:
   - Evaluation test suites reside in `benchmarks/assets/RepoMod-bench/dataset/<project>/tests/`.
   - The modernization engine and holon subagents never see, read, or patch these test files.
   - Post-run verification executes the external test command (`pytest` or language-specific test runner) against the compiled target binary (`run_cmd`), capturing stdout/stderr and exit codes.
   - Diagnostic compaction filters compiler error logs and test failure summaries for repair rollouts without leaking test source code into LLM prompts.

4. **Pilot Modernization Candidate Selection**:
   - `jmespath` (Go $\to$ Rust, 19,308 LOC, 888 compliance tests, source: `https://github.com/jmespath/go-jmespath`) is chosen as the initial candidate:
     - Pure computational isolation: CLI tool reading JSON from stdin with expression arguments (`./target/release/jp '<expr>'`), with zero network, database, or external service dependencies.
     - Strict specification compliance: 888 parameterized tests from official compliance suites covering complete JMESPath semantics.
     - High test density and compact repository scale.

### Consequences

* **Good**: Eliminates test tampering and circular validation by physically isolating test suites from the agent's workspace.
* **Good**: Scales benchmark evaluation from toy single-file crates to realistic multi-thousand-LOC software repositories.
* **Good**: Preserves full backward compatibility and ablation parity with existing ReCodeAgent campaign results.
* **Good**: Establishes clean black-box functional equivalence verification aligned with EGAGM and THESEUS validation contracts.
* **Neutral**: Requires Python 3.12+ and pytest in the harness environment to execute RepoMod-Bench evaluation suites.

### Confirmation

1. Verification with `cargo check --workspace` to ensure all workspace crates compile cleanly.
2. Configuration validation ensuring YAML campaign configs in `benchmarks/assets/configs/RepoMod-bench/` deserialize into `configschema::CampaignConfig`.
3. Specifications in `docs/specs/tech-spec/benchmark-harness.md` and `docs/specs/INDEX.md` updated in lockstep.
4. Managed skill `~/.omp/agent/managed-skills/werthan-repomod-benchmarks/SKILL.md` available for operator workflows.

## Pros and Cons of the Options

### Option 1: Complete Cutover to RepoMod-Bench

* Good, because it simplifies the repository to a single benchmark suite.
* Bad, because it deletes historical baselines, prevents ablation comparisons, and removes lightweight targets for fast local smoke testing.
* Bad, because existing tooling and documentation referencing ReCodeAgent would break.

### Option 2: Dual-Harness Integration (Adopt RepoMod-Bench alongside ReCodeAgent)

* Good, because it retains 118 existing ReCodeAgent configs for continuous regression testing.
* Good, because it enables gradual migration to repository-scale modernization benchmarks.
* Good, because it supports implementation-agnostic black-box evaluation without breaking existing white-box test harnesses.
* Neutral, because the harness must support both workspace layout conventions.

## More Information

* RepoMod-Bench Reference: Xuefeng Li et al., *"RepoMod-Bench: A Benchmark for Code Repository Modernization via Implementation-Agnostic Testing"*, arXiv:2602.22518, KDD '26.
* Codebase: `benchmarks/assets/RepoMod-bench`
* Methodology Alignment: [ADR-0019](0019-methodology2-egagm.md) (EGAGM), `docs/references/METHODOLOGY2.md`.
* Validation Contracts: `docs/specs/validation-contracts.md` (VAL-DAEMON-01 through VAL-DAEMON-04).
