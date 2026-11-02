# 04 — Architectural Implications: Bounded Statechart MCTS vs. Agentic Deadlocks

## 1. The Autonomous Agent Paradigm vs. Principled Synthesis

The prevailing software engineering consensus posits that general-purpose autonomous LLM agents (e.g., ReCodeAgent, SWE-Agent, Claude Code) equipped with shell tools (`bash`, `grep`, `edit`) can resolve complex translation tasks through unconstrained conversational iteration.

Our empirical findings on `crust.cfsm` (forensic autopsy of agentic loops and harness flaws), `crust.dict` (telemetry evaluation of bounded statechart MCTS under SLM context compaction), and `crust.libwecan` (0.0% $\to$ 66.7% resolution via statechart MCTS and stagnation detection) refute this assumption for systems-level legacy migration.

```
[ ReCodeAgent Paradigm: Unconstrained Agent Loop ]
User Prompt ──> [Planner LLM] ──> [Generator LLM] ──> [Bash Tool: cargo test]
                      ^                                       │
                      └──────── Error Logs & Dumps <──────────┘
                      (Context explodes -> Repetition -> Deadlock)

-------------------------------------------------------------------------

[ werðan / DAEDALUS Paradigm: Formal Statechart Guided MCTS ]
DAG Vertex ──> [Harel Statechart H: S0..S6] ──> [Micro-FSM M0..M5]
                      │                                 │
                      ▼                                 ▼
             [Rollout Isolation] ───────────────> [Evaluator Oracle]
                      │                                 │
                      ▼                                 ▼
         [Frequent Compaction ADR-0018] ──> [Reward R & Backprop]
```

---

## 2. Fundamental Failure Modes of Unconstrained LLM Agents

### 2.1 The Lack of Backtracking and Rollback
When an unconstrained agent makes a breaking design choice (such as altering a C ABI free function to a Rust struct method), downstream integration tests fail.
- Instead of rolling back the AST to the previous valid state, the agent attempts to "fix forward" by modifying the integration tests or inventing shim layers.
- In ReCodeAgent's `crust.dict` trace, when `dict_key()` failed borrow checking, the agent attempted 14 consecutive edits that leaked raw pointers, declared non-existent extern crates, and finally deleted test assertions.
- **In werðan**: Every action occurs inside an ARIADNE MCTS rollout. If a rollout produces compilation errors or negative reward, its intermediate tree state is instantly discarded. The engine retreats to the highest-scoring parent node without polluting subsequent rollouts.

### 2.2 Semantic Drift and Tool Misuse
Unconstrained agents frequently abuse shell access:
- Generating temporary files outside the build tree.
- Attempting to run package managers (`apt`, `npm`, `cargo install`) or changing toolchain flags.
- Confusing diagnostic stderr with stdin prompts.
- **In werðan**: Tool execution is strictly deterministic and mediated throughomp-standard tools (`read`, `edit`, `write`, `grep`, `glob`, `cargo_harness`). Agents never execute arbitrary bash scripts; every modification is an in-memory diff validated by the Cargo harness.

### 2.3 Context Pollution and Semantic Satiation
As shown in Report `02-compaction-in-practice.md`, raw conversational accumulation drowns the model in redundant error messages. By turn 10, the prompt consists of 90% compiler noise and 10% actual source code.
- Under high noise, LLM attention mechanisms suffer semantic satiation: the model stops attending to specific line numbers and begins guessing code transformations.
- **In werðan**: Frequent Intentional Compaction (ADR-0018) strips all non-essential diagnostic tokens. Context size is invariant with respect to rollout depth.

### 2.4 Unbounded Iteration vs. Stagnation-Braked Candidate Promotion
When an unconstrained agent reaches a plateau or an unresolvable obstacle, it spins indefinitely until external timeout:
- In ReCodeAgent's `crust.cfsm` trace, the agent repeated the identical broken edits across 5 iterations until reaching a 51.6-minute timeout without saving any partial progress.
- In ReCodeAgent's `crust.libwecan` trace, the agent produced syntax errors, tried random edits, and crashed during validation, leaving a completely broken crate (0.0% pass rate).
- **In werðan**: The MCTS controller monitors candidate reward progression. When reward plateaus across 8 consecutive rollouts, the engine detects stagnation, terminates the search early, and promotes the best candidate with reward $\ge 0.5$ into `board.artifacts`. On `crust.libwecan`, this mechanism preserved a clean-compiling crate, but the synthesized `insert` function contains an unsigned integer underflow panic at `src/libwecan.rs:137:13` that causes the binary test runner to abort, resulting in `cargo test` exit code 101 and `test_pass_rate: 0.0`.
---

## 3. The Core Strengths of DAEDALUS

| Dimension | ReCodeAgent Architecture | werðan DAEDALUS Architecture |
|:---|:---|:---|
| **Control Flow** | Dynamic LLM-driven planning | Formal Harel Statechart $\mathcal{H}$ ($S_0 \to S_6$) |
| **Search Strategy** | Greedy iterative repair | Monte Carlo Tree Search (ARIADNE) with UCB1 |
| **Context Memory** | Unbounded conversation history ($>30\text{k}$ tokens) | Blackboard State Vector $\mathcal{B}^{(v_i)}$ + ADR-0018 Compaction ($\le 12\text{k}$) |
| **Header Handling** | Ad-hoc source concatenation (causes cyclic locks) | Topological DAG vertex pre-hardening (`M4Hardened`) |
| **Verification Oracle** | Loose stdout regex matching | Deterministic multi-tier `CargoHarness` (ABI + unit + char tests) |
| **Hardware Requisite** | Proprietary frontier APIs (Claude 3.5 Sonnet) | Single consumer workstation GPU (RTX 3090, 24 GB) |

---

## 4. Conclusion and Strategic Guidance

For legacy codebase modernization, **structured search outranks model parameter scale**. A 27B parameter open-weights model guided by formal statecharts, topological dependency ordering, and intentional compaction outperforms a state-of-the-art 175B+ proprietary model trapped in an unconstrained conversational loop.

Organizations modernizing legacy systems should abandon unguided conversational agents in favor of bounded, contract-enforced state machine synthesis engines.
