---
status: "accepted"
date: 2026-11-02
decision-makers: ["NTDuck", "omp"]
supersedes: ["0005-methodology-source-of-truth.md"]
---

# Pin METHODOLOGY2.md (Evidence-Gated Adaptive Graph Migration / EGAGM) as Authoritative System Methodology

## Context and Problem Statement

The system previously designated `docs/references/METHODOLOGY.md` as canonical ground truth via [ADR-0005](0005-methodology-source-of-truth.md). That model specified a 7-phase Harel Statechart ($S_0 \dots S_6$), rigid 1:1 file decomposition assumptions, monolithic ARIADNE Monte Carlo Tree Search across all compilation units, and continuous heuristic rewards coupling generation with acceptance.

Extensive empirical evaluations and forensics revealed critical limitations across existing modernization paradigms:
1. **Rule-Based Transpilation Failures (C2Rust analysis)**: Syntactic transpilers preserve C raw pointers (`*const T`, `*mut T`), manual heap allocation primitives (`malloc`/`free`), and unstructured C ABIs, abandoning memory safety, lifetime invariants, and idiomatic target encapsulation.
2. **Static Feed-Forward LLM Failures (ReCodeAgent benchmark forensics)**: Feed-forward 4-stage pipelines (`Analyzer` $\to$ `Planner` $\to$ `Translator` $\to$ `Validator`) suffer non-monotonic breakdown when downstream type requirements invalidate upstream interfaces. Furthermore, granting validators write access to workspaces induces the circular test-tampering defect (e.g. generating passing synthetic tests or commenting out assertions while achieving 0% ground-truth pass rate).
3. **Rigid 1:1 Topology & Monolithic Search Bottlenecks (Architecture 6 + ARIADNE evaluation)**: Forcing identical module topology between source and target prevents idiomatic decomposition (e.g., splitting a monolithic C header into separate Rust modules or collapsing redundant C shims). Monolithic MCTS search introduces exponential branch explosions and wasteful token spend on straightforward deterministic leaf modules.
4. **Adversarial Research Council Derivations (Pareto Frontier P1–P4)**: The Council evaluated four candidate architectures on the sound-idiomatic-cost Pareto frontier:
   - **P1 (Strict Formal Verified Migration)**: Dependent types and proof synthesis; prohibitively expensive and unscalable for enterprise codebases.
   - **P2 (Evidence-Gated Adaptive Graph Migration / EGAGM)**: Dual-graph topology, 5 granular maturity levels, strict independent Boolean acceptance gates, immutable cryptographic evidence ledgers, and cost-aware adaptive solver portfolios.
   - **P3 (Optimistic Semantic Parallel)**: Speculative high-throughput generation with coarse validation; fails to prevent context drift and circular validation.
   - **P4 (Interactive Continuous Modernization)**: Human-in-the-loop co-pilot; fails autonomous scaling objectives.

Which architecture and specification shall serve as the authoritative system methodology and ground truth for werðan?

## Decision Drivers

* **Dual-Graph Independence**: Decoupling the source call/dependency graph ($\mathcal{G}_{\text{src}}$) from the target module/crate graph ($\mathcal{G}_{\text{tgt}}$) enables idiomatic refactoring, module consolidation, and safe interface redesign rather than forced 1:1 syntactic mirror translation.
* **Non-Monotonic Resiliency via Granular Maturity Levels**: Replacing coarse phase-locking with 5 progressive maturity levels ($L_0$ to $L_4$) allows fine-grained dependency coordination, concurrent downstream hydration, and targeted rollback invalidation without resetting entire repository pipelines.
* **Anti-Gaming and Verifiable Soundness**: Strict separation of generation heuristics from an independent Boolean acceptance authority ($\mathcal{G}_{\text{accept}}$), certified through a version-aware immutable evidence ledger ($\mathcal{E}_{\text{ledger}}$) backed by isolated sandboxes and read-only test oracles.
* **Cost-Aware Adaptive Search Portfolios**: Replacing uniform brute-force MCTS with an adaptive solver selection mechanism ($\Pi_{\text{adaptive}}$) that allocates compute proportionally: direct syntactic translation for trivial leaf modules, rule-guided template synthesis for standard modules, and bounded search/repair portfolios only for complex interdependent hubs.
* **Unification of Formal Theory and Benchmarking Reality**: P2 (EGAGM) directly synthesizes lessons learned from the ReCodeAgent CRUST benchmarks, C2Rust transpiler deconstruction, and local 27B SLM inference constraints.

## Considered Options

* **Option 1: Retain METHODOLOGY.md (7-Phase Harel Statechart + Monolithic ARIADNE)**
  - *Shortcomings*: Enforces 1:1 file translation, lacks formal dual-graph mappings, lacks progressive module maturity levels ($L_0 \dots L_4$), conflates heuristic search reward with acceptance verification, and applies uniform search regardless of module complexity.
* **Option 2: Adopt P2 Evidence-Gated Adaptive Graph Migration (EGAGM) via METHODOLOGY2.md**
  - Pin `docs/references/METHODOLOGY2.md` and derivation record `docs/references/METHODOLOGY2-DRV.md` as the canonical system ground truth.
  - Supersede ADR-0005 and establish EGAGM as the authoritative methodology across all engine, holon, and benchmark components.

## Decision Outcome

Chosen option: **Option 2: Pin `docs/references/METHODOLOGY2.md` (Evidence-Gated Adaptive Graph Migration / EGAGM) as Authoritative System Methodology**.

`docs/references/METHODOLOGY2.md` (accompanied by `docs/references/METHODOLOGY2-DRV.md`) supersedes `docs/references/METHODOLOGY.md` as the sole canonical source of truth for werðan's architecture, theoretical models, statecharts, and verification protocols.

### Core Architectural Tenets of EGAGM (P2)

1. **Dual-Graph Architecture ($\mathcal{G}_{\text{src}} \times \mathcal{G}_{\text{tgt}}$)**:
   - Source repository is mapped to source entity graph $\mathcal{G}_{\text{src}} = (V_{\text{src}}, E_{\text{src}})$.
   - Target repository is synthesized as target module graph $\mathcal{G}_{\text{tgt}} = (V_{\text{tgt}}, E_{\text{tgt}})$.
   - Explicit morphism and condensation mappings permit $N:M$ entity restructuring, breaking 1:1 legacy file entrapment.

2. **5 Progressive Maturity Levels ($L_0 \dots L_4$)**:
   - **$L_0$ (Unmapped / Discovered)**: Raw source entity extracted, target boundary unbound.
   - **$L_1$ (Contract Frozen / Interface Locked)**: Target signatures, type definitions, and public contracts locked; downstream consumers can safely hydrate and synthesize concurrently.
   - **$L_2$ (Syntactically Lowered)**: Internal implementation synthesized; compiles against $L_1+$ parent interfaces.
   - **$L_3$ (Unit Verified & Behaviorally Sound)**: Passes local unit tests, property suites, and mock differential oracles.
   - **$L_4$ (Integration Hardened / Proof Certified)**: Full repository compilation, passes pristine differential execution harness, evidence ledger receipt signed, committed to master branch.

3. **Independent Strict Acceptance Authority ($\mathcal{G}_{\text{accept}}$)**:
   - Evaluation is decoupled from generative agent reward functions.
   - Acceptance is a strict Boolean conjunction: $\mathbb{I}_{\text{compile}} \land \mathbb{I}_{\text{differential}} \land \mathbb{I}_{\text{sast}} \land \mathbb{I}_{\text{oracle}}$.
   - Test suites and execution harnesses execute in isolated sandboxes with read-only mounts, permanently preventing test-tampering or synthetic oracle mutation.

4. **Version-Aware Immutable Evidence Ledger ($\mathcal{E}_{\text{ledger}}$)**:
   - Every state transition to $L_4$ generates an immutable, content-addressed receipt containing input source hashes, target AST hashes, compiler diagnostic logs, test execution transcripts, and differential oracle stdout/stderr traces.
   - Eliminates circular verification and ensures 100% reproducible migration audits.

5. **Cost-Aware Adaptive Solver Portfolio ($\Pi_{\text{adaptive}}$)**:
   - Module complexity classifier routes units based on cyclomatic complexity, pointer density, and fan-in/fan-out metrics.
   - Low-complexity leaf units receive low-latency deterministic or template solvers; high-complexity core modules receive bounded tree search with diagnostic compaction.

### Consequences

* **Good**: Eliminates test-tampering and false-positive benchmark validation through isolated execution oracles and immutable ledger receipts.
* **Good**: Unlocks safe concurrent translation of downstream modules as soon as upstream contracts reach $L_1$, eliminating topological pipeline stalls.
* **Good**: Supports true idiomatic refactoring through dual-graph abstraction rather than forced 1:1 file translation.
* **Good**: Optimizes inference token expenditure and latency by dynamically selecting appropriate solver tiers rather than forcing uniform MCTS everywhere.
* **Neutral**: Downstream specification files (`docs/specs/**`) and engine crates will undergo systematic Phase 2+ synchronization to align contracts and FSM naming with EGAGM.

## More Information

* Canonical Methodology Reference: `docs/references/METHODOLOGY2.md`
* Derivations and Council Proceedings: `docs/references/METHODOLOGY2-DRV.md`
* Previous Pinning Decision (Superseded): [ADR-0005](0005-methodology-source-of-truth.md)
