# 01 — The Evolution of Multi-Agent Systems for Code Modernization

## 1. Executive Context & Theoretical Problem Formulation

Automated repository-scale code-to-code translation ($\text{code2code}$ modernization) represents one of the most demanding frontiers in applied software engineering and artificial intelligence. Given a legacy codebase $\mathcal{R}_{\text{leg}}$ in a source language $\mathcal{L}_{\text{src}}$ (such as procedural C), an execution harness $\mathcal{H}_{\text{exec}}$, and a target specification in a modern language $\mathcal{L}_{\text{tgt}}$ (such as memory-safe Rust), the system must synthesize a modernized repository $\mathcal{R}_{\text{tgt}}$ satisfying three formal invariants:

1. **Syntactic and Type-Level Compilation**: $\mathbb{I}_{\text{compile}}(\mathcal{R}_{\text{tgt}}) = 1.0$ across all internal and inter-module boundaries.
2. **Behavioral and Semantic Parity**: $\text{PassRate}_{\text{char}}(\mathcal{R}_{\text{tgt}}) = 1.0$ across all domain invariants, property tests, and characterization suites extracted from legacy behavior.
3. **Idiomatic Architecture and Static Security**: $\text{Score}_{\text{harden}}(\mathcal{R}_{\text{tgt}}) = 1.0$ adhering to target language ownership, concurrency, and safety paradigms without raw-pointer shims or type-erased compromises.

### The Axiom of Non-Monotonic Discovery

Standard multi-agent workflows (e.g., in web navigation, document summarization, or greenfield code generation) operate under an assumption of *monotonic task progression*: work completed in early steps remains valid as later steps execute.

Repository modernization fundamentally violates this assumption due to **non-monotonic discovery**. In complex software systems:
* Implementation details discovered late during the translation of downstream module $v_i$ frequently reveal that an interface contract $\mathcal{C}(v_p, v_i)$ designed by an upstream parent $v_p$ is architecturally unviable, incomplete, or fundamentally unidiomatic in the target language.
* Translating legacy memory patterns (e.g., C pointer aliasing, type-erased callback tables, or cyclic references) into a target language with strict affine type systems (Rust's borrow checker) requires topological coordination that cannot be anticipated by isolated static analysis.

Understanding the current state-of-the-art methodology requires tracing how modernizing architectures evolved over successive paradigms to solve this non-monotonic discovery problem.

---

## 2. Chronological Deconstruction of Architectural Paradigms

```
 Era 1: Monolithic Prompting (Single Turn)
   └── Failure: Context window saturation, zero topological awareness.
 Era 2: Static Graph Pipelines (ReCodeAgent, ASE 2026)
   └── Failure: Feed-forward lockstep; no backward recovery; "Illusion of Validation".
 Era 3: Centralized Conversational / Manager-Worker Hierarchies (ChatDev, MetaGPT)
   └── Failure: Manager state hallucination; chat token bloat into "Dumb Zone" (>12k).
 Era 4: Flat Global Finite State Machines (FSMs)
   └── Failure: State space explosion O(m^|V|); lockstep global rollbacks.
 Era 5: Autonomous Recursive Subagent Spawning (ReAct + subagent tool)
   └── Failure: O(K) context bottleneck at parent; uncheckpointed shadow state.
 Era 6: Single-Trajectory Greedy Self-Debugging
   └── Failure: Entrapment in local minima; borrow-checker thrashing.
 Era 7: Two-Tier Harel Statechart + Encapsulated ARIADNE MCTS (werðan)
   └── Solution: Decoupled control, orthogonal AND-regions, UCB tree search, multi-tier recovery.
```

---

### Era 1: Monolithic Single-Turn LLM Prompting

Early attempts at neural code translation treated whole repositories as concatenated textual inputs passed to a large language model with instructions such as `"Translate this C project to Rust."`

* **Operational Topology**: Monolithic $1 \to 1$ mapping.
* **Failure Modes**:
  1. **Context Window Blowout**: Exceeds token limits on repositories $> 500$ lines.
  2. **Type and ABI Hallucination**: The model invents arbitrary function signatures across file splits, resulting in irreconcilable compiler errors at module link boundaries.
  3. **Loss of Invariants**: Without execution harnesses or iterative feedback, hidden domain logic and edge cases are silently dropped.

---

### Era 2: Static Graph-Pipelines (ReCodeAgent, ASE 2026)

To move beyond monolithic prompting, ReCodeAgent introduced a multi-agent framework structuring the modernization lifecycle into four sequential phases: `Analyzer` $\to$ `Planning` $\to$ `Translator` $\to$ `Validator`. Dependencies are represented as a Directed Acyclic Graph (DAG), and translation proceeds topologically.

* **Operational Topology**: Feed-forward pipeline with localized iteration loops:
  $$\text{Analyze}(\mathcal{R}_{\text{leg}}) \longrightarrow \text{Plan}(G) \longrightarrow \Big[ \text{Translate}(v_i) \longleftrightarrow \text{Validate}(v_i) \Big]_{i=1}^{|V|}$$

* **Why It Appeared Successful in Trivial Domains**:
  On high-level language pairs such as `Skel` (Python $\to$ JavaScript) and `Oxidizer` (Go $\to$ Rust), ReCodeAgent reported near-100% success. In those domains, memory management is garbage-collected or handled via high-level cloneable collections (`Vec`, `HashMap`), allowing 1-to-1 syntactic translation without architectural restructuring.

* **The Catastrophic Breakdown in Systems Programming (CRUST: C $\to$ Rust)**:
  On the CRUST dataset (100 C projects translated to Rust), ReCodeAgent's reported pass rate dropped to 58.2%, with **16 entire projects scoring 0.0%**. The reasons are structural:
  1. **Strict Feed-Forward Blindness**: Once `Planning` establishes interface skeletons and target types, there is **no backward topological edge**. If translating downstream unit $v_i$ proves that upstream module $v_p$'s interface cannot satisfy Rust's borrow checker, the system cannot invalidate or update $v_p$. It either generates uncompilable glue code or crashes.
  2. **Monolithic File Context Saturation**: Whole files and lengthy planning documents are passed into translator prompts, rapidly pushing context lengths past the threshold where reasoning degrades.
  3. **The "Illusion of Validation" Defect**:
     Our forensic inspection of the Zenodo ReCodeAgent execution archives (`benchmarks/assets/ReCodeAgent-zenodo/results-unpacked/`) revealed a critical phenomenon. In `recodeagent.crust.libwecan.c.rust/orchestrator.log`, the `Validator Agent` recorded:
     > `"Validation Summary - PASS ... ZERO issues found ... All tests pass in both languages"`
     
     Yet in the published ablation study (`ablation-study-effectiveness.csv`), ReCodeAgent scored **0.0% (0 / 18 tests passing)**. Why? The validator subagent ran an isolated local unit test on `src/lib.rs` and completely failed to evaluate the workspace integration test target `libwecanTest_generated.rs`, which failed `cargo check` with **18 unresolved compiler errors** (`error[E0425]: cannot find function ... in this scope`). The static agent claimed victory while the repository was broken.

---

### Era 3: Centralized Conversational / Manager-Worker Hierarchies

To introduce flexibility, frameworks like ChatDev, MetaGPT, and GVS5H deployed conversational manager-worker hierarchies. A central LLM "Orchestrator" or "Manager" agent interacts with worker agents (Coder, Tester, Reviewer) by maintaining conversational message history and routing tasks via natural language.

* **Operational Topology**: Star topology with a conversational hub:
  $$\mathcal{M}_{\text{manager}} \underset{\text{chat}}{\overset{\text{instructions}}{\rightleftharpoons}} \{ \mathcal{W}_{\text{coder}}, \mathcal{W}_{\text{tester}}, \mathcal{W}_{\text{critic}} \}$$

* **Failure Modes**:
  1. **State Hallucination over Long Horizons**: As modernizing a repository requires hundreds of tool calls and iterations, the manager's chat history expands. The LLM manager begins hallucinating which modules have passed verification, losing track of dependency orders.
  2. **Context Window Inflation into the "Dumb Zone"**: SLMs and large models alike suffer severe reasoning degradation once conversation buffers exceed $12\text{k}$ tokens. Passing cumulative conversation threads across workers burns millions of tokens without producing working code.
  3. **Polling Latency and Unbounded Loops**: Workers polling or conversing without strict typed protocol boundaries drift into conversational loops (e.g., polite agreements between Reviewer and Coder without actual code changes).

---

### Era 4: Flat Global Finite State Machines (FSMs)

Recognizing the fragility of conversational managers, early systems-engineering attempts replaced the LLM manager with a deterministic, programmatic Finite State Machine (FSM). The FSM tracked macro-phases (e.g., `Scan`, `Translate`, `Verify`).

* **Operational Topology**: Single flat global FSM: $\Sigma = \{S_0, S_1, \dots, S_N\}$.
* **Failure Modes**:
  1. **The Combinatorial State-Space Explosion**:
     To track $|V|$ modules across $m$ possible micro-states (e.g., `Blocked`, `Compiling`, `Tested`, `Failed`) in a flat FSM, the composite state space grows exponentially:
     $$|\Sigma_{\text{composite}}| = O(m^{|V|})$$
     For a 10-module project with 5 states per module, tracking global state requires $5^{10} \approx 9.7 \times 10^6$ composite states.
  2. **Lockstep Global Rollbacks**:
     Because flat FSMs lack orthogonal concurrency, a verification failure in module $v_8$ forces a global transition back to an earlier phase (e.g., `Phase_Translate`). This triggers a lockstep rollback across the entire repository, discarding verified, compilable code in modules $v_1 \dots v_7$.

---

### Era 5: Autonomous Recursive Subagent Spawning

Frameworks embracing the ReAct pattern often equipped agents with autonomous subagent creation tools (e.g., `spawn_subagent(prompt)`), allowing an agent to dynamically spawn child workers for rule extraction or file translation.

* **Operational Topology**: Dynamic unbounded agent tree.
* **Failure Modes**:
  1. **The $O(K)$ Context Bottleneck**:
     When a parent agent spawns $K$ parallel subagents, each subagent returns its output text into the parent's message buffer. The parent immediately suffers an $O(K)$ context bottleneck, exhausting its context window and truncating downstream instructions.
  2. **Uncheckpointed Shadow State**:
     Child agents spawn grandchild agents that mutate local files or make decisions outside the knowledge of the main workflow. If a run crashes, this shadow state cannot be rolled back or inspected, making deterministic checkpointing impossible.

---

### Era 6: Single-Trajectory Greedy Self-Debugging

When repairing broken translations, common agent architectures use single-trajectory greedy repair: the agent compiles the code, takes the compiler error trace, and asks the LLM to patch the code in-place, repeating until it compiles or reaches an iteration cap.

* **Operational Topology**: Linear feedback loop:
  $$c_0 \xrightarrow{\text{err}_0} c_1 \xrightarrow{\text{err}_1} c_2 \xrightarrow{\text{err}_2} \dots \xrightarrow{\text{err}_N} c_N$$

* **Failure Modes**:
  1. **Entrapment in Local Minima**:
     If the initial code $c_0$ adopted a fundamentally flawed architectural pattern (e.g., attempting to translate C pointer aliasing directly using raw mutable pointers in Rust `*mut T`), greedy repairs patch individual compiler syntax errors while digging deeper into borrow-checker contradictions. The agent oscillates between competing errors (e.g., adding `&mut` creates an aliasing violation; removing `&mut` creates an uninitialized memory error).
  2. **Diagnostic Bloat and Context Flooding**:
     Each compiler failure dumps dozens of lines of diagnostic notes, hints, and stack traces into the prompt. Without intentional compaction, the prompt rapidly degrades into the token "Dumb Zone", rendering the model incapable of generating coherent patches.

---

## 3. The Master Paradigm Comparison

The following matrix compares all seven eras across the fundamental axes of distributed software engineering and multi-agent systems:

| Architectural Paradigm | Control Topology | State Complexity | Non-Monotonic Recovery | Context Scale Upper Bound | Guard Mechanism | Search Exploration Strategy | Key Failure Mode |
|:---|:---|:---:|:---:|:---:|:---|:---|:---|
| **Era 1: Monolithic Prompt** | Zero (Single-turn) | $O(1)$ | None (Fail/Pass) | Unbounded ($>32\text{k}$) | None | Greedy single sample ($T=0$) | Context overflow, hallucination |
| **Era 2: Static Graph (ReCodeAgent)** | Feed-forward DAG pipeline | $O(|V|)$ | None (Forward only) | Unbounded file dumps | Programmatic (shallow local) | Greedy single trajectory | Upstream interface freeze, 18-error validation illusion |
| **Era 3: Conversational Manager** | Central LLM Chat Hub | $O(\text{msgs})$ | Unstructured conversational re-ask | Unbounded chat growth ($>20\text{k}$) | Conversational review | Ad-hoc dialogue repair | State hallucination, polite stalling loops |
| **Era 4: Flat Global FSM** | Single Root State Machine | $O(m^{|V|})$ (Exponential) | Lockstep global rollback | Moderate | Structural predicates | Deterministic transition | State space explosion, destructive rollback |
| **Era 5: Recursive Spawning** | Unbounded Agent Tree | $O(\text{Tree})$ | Uncheckpointed local backtracking | $O(K)$ parent bottleneck | Ad-hoc agent return | Uncontrolled depth search | Context exhaustion at parent, shadow state |
| **Era 6: Greedy Self-Repair** | Linear repair loop | $O(\text{iters})$ | In-place patch backtracking only | Context bloat from raw traces | Compiler exit code | Greedy hill-climbing | Entrapment in local minima, borrow-checker thrashing |
| **Era 7: Two-Tier Harel + ARIADNE (werðan)** | **Harel Statechart + Encapsulated MCTS** | **$O(m \cdot |V|)$ (Linear)** | **Multi-Tier Calculus (Tier A/B/C with $H^*$)** | **Bounded Smart Zone ($\le 12\text{k}$ via ADR-0018)** | **Two-Stage Hybrid ($g_{\text{struct}} \land g_{\text{sem}}$)** | **Softmax-UCB MCTS Tree Search over Strategies** | **Zero state hallucination, provable bounded recovery** |

---

## 4. Synthesis: Why the Two-Tier Harel + ARIADNE Architecture is Necessary

The synthesis of these historical lessons directly dictated the architecture of **werðan** (the THESEUS framework):

1. **Decoupled Control ($\mathcal{O}_{\text{engine}}$ vs $\mathcal{O}_{\text{arbiter}}$)**:
   The orchestrator must be a programmatic, deterministic runtime engine ($\mathcal{O}_{\text{engine}}$). The LLM is NEVER allowed to maintain system state in prompt memory; it is invoked only as a stateless arbiter ($\mathcal{O}_{\text{arbiter}}$) for semantic conflict resolution.
2. **Two-Tier Blackboard ($\mathcal{G}$ and $\mathcal{B}^{(v_i)}$)**:
   Repository-wide artifacts (inventory, condense DAG, export contracts, hardened target code) reside in the persistent Global Blackboard $\mathcal{G}$. Module-internal exploration resides in an isolated 4-slice Local Blackboard $\mathcal{B}^{(v_i)}$, eliminating inter-agent context cross-talk.
3. **Orthogonal AND-Regions with Deep History ($H^*$)**:
   By structuring module transformation as an orthogonal product $S_5 = \bigparallel_{i=1}^{|V|} R(v_i)$, state complexity collapses from $O(m^{|V|})$ to $O(m \cdot |V|)$. If an upstream contract regresses, Deep History ($H^*$) preserves all unaffected verified modules without re-execution.
4. **Declarative Scatter-Gather ($D_{\max}=1$)**:
   Imperative subagent spawning is strictly banned. Global discovery phases execute as deterministic 3-stage sub-statecharts (`Scatter` $\to$ `Parallel Map` $\to$ `Reduce`) over bounded shards satisfying $\text{Tokens}(\text{shard}_k) \le L_{\max}$.
5. **Encapsulated Intra-Module MCTS (ARIADNE)**:
   Greedy self-debugging is replaced with a Monte Carlo Tree Search exploring distinct refactoring hypotheses ($\mathcal{A}_{\text{str}}$), code candidates ($\mathcal{A}_{\text{gen}}$), isolated adversarial tests ($\mathcal{A}_{\text{test}}$), and surgical patches ($\mathcal{A}_{\text{patch}}$) scored by a gated reward function.
6. **Hardware-Aware Compaction (ADR-0018)**:
   Diagnostic traces are compacted ($\le 2048$ chars, top-3 compiler errors) to guarantee that SLM reasoning stays strictly within the high-fidelity $\le 12\text{k}$ token "Smart Zone".
