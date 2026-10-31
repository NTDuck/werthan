# 03 — Adversarial Critique of the Incumbent Methodology

**Target of Evaluation:** `docs/references/METHODOLOGY.md` (authoritative ground truth per ADR-0005) and its derivation transcript `docs/references/METHODOLOGY-DRV.md`.  
**Deployment Regime:** 27B-class Small Language Model (`swift-1.5-qwen3.8-27b`) on a single local NVIDIA RTX 3090, bounded context window (8k–32k tokens), local OpenAI-compatible serving stack (`127.0.0.1:8092/v1`).

---

## 1. Executive Summary of Findings

The incumbent methodology presents a mathematically elegant formulation—a Two-Tier Harel Statechart ($\mathcal{O}_{\text{engine}}$) coupled with Encapsulated ARIADNE MCTS Solvers (`M2`) over a shared Blackboard ($\mathcal{G}$). However, adversarial evaluation across the eight formal research questions (RQ1–RQ8 from `01-meta-strategy.md`) supported by real campaign receipts (`02-evidence-base.md`) reveals **four fatal structural flaws**:

1. **The Formal Guarantee is Empirically False (RQ1):** The flagship claim that achieving composite reward $R(s_t) = 1.0$ is "necessary and sufficient" for behavioral parity fails in both directions: it is *insufficient* under weak benchmark oracles (vacuous passes, F4) and *unnecessary* in practice (green workspace rejected by uncalibrated reward, F3).
2. **Category-Shape Mismatch on Single-Vertex DAGs (RQ2, RQ5):** The multi-tier regression calculus (Tier B/C) was designed for complex multi-module DAGs with deep dependency trees. When applied to single-vertex repositories (`skel`), the runtime engine fabricated nonexistent parent counterparties (`"parent_upstream"`), converting retryable budget exhaustion into terminal structural failure (F1, F2).
3. **MCTS Search Space vs. SLM Compute Budget Mismatch (RQ3, RQ4):** Running full MCTS rollouts with 4 generative agents per module vertex exceeds the throughput envelope of a single local GPU. With $B_{\max} = 16$, complex translations fail to converge; yet increasing $B_{\max}$ blows past the 2-hour campaign deadline. Furthermore, MCTS trees are discarded across dispatches without warm-start retention.
4. **Neglect of the SLM "Dumb Zone" & Absence of Compaction (RQ4, RQ6):** The methodology assumes ephemeral stateless workers can be hydrated with 1-hop subgraphs, but specifies no mechanism for **Frequent Intentional Compaction**. As tool feedback and stack traces accumulate, the SLM degrades into repetitive patching loops.

---

## 2. Detailed Critique by Research Question

### RQ1 (Falsifiable). Do the methodology's formal claims hold under actual oracle strength?

**Incumbent Claim (`METHODOLOGY.md` §4.2):**
> *"Because $\mathbb{I}_{\text{compile}} \in \{0,1\}$, $\text{PassRate}_{\text{char}} \in [0,1]$, and $\text{Score}_{\text{harden}} \in [0,1]$, achieving $R(s_t) = 1.0$ is a necessary and sufficient condition for full compilation, 100% characterization test parity, and full semantic rule adherence."*

**Adversarial Refutation:**
- **Sufficiency is Refuted by F4 (`skel-toml.json`):**
  In the `toml` campaign (old binary run, 17.1 h), the candidate file collapsed to an 18-line stub (`get_input`). The target test command (`node translated.js`) exited with code 0 because the test suite only exercised the trivial top-level function. The harness recorded `test_pass_rate = 1.0`. Under the methodology's reward formula, this stub achieves near-perfect reward despite omitting 95% of the repository's parsing logic. Native oracles lack stub-sensitivity; therefore, $R = 1.0$ does **not** guarantee semantic equivalence.
- **Necessity is Refuted by F3 (`skel-heapq.json` — The heapq Paradox):**
  In the `heapq` v18 campaign, the workspace held an artifact that satisfied all four external acceptance criteria (`compilation_success = true`, `test_pass_rate = 1.0`, zero stubs, 107 requests). However, internal reward scoring failed to backpropagate $R = 1.0$ due to calibration drift in $\text{Score}_{\text{harden}}$. The engine failed to emit `ev_ModuleHardened`, burned an additional hour of GPU time in futile rollouts, and terminated with an `error` at the futile dispatch cap. A fully correct candidate existed, yet $R = 1.0$ was never attained.

**Conclusion:** The formal guarantee is **falsified**. Reward and acceptance gate must be unified under a single ground-truth execution stream hardened with stub-sensitivity.

---

### RQ2 (Falsifiable). Does the Harel Statechart scale under local SLM execution, or is it an over-engineered formalization?

**Incumbent Claim (`METHODOLOGY.md` §1, §3, §5):**
> *"We formalize control via a Harel Statechart $\mathcal{H}$ combining XOR Macro-Superstates, Orthogonal AND-Regions, Broadcast Event Calculus, and Deep History ($H^*$) preserving verified modules across macro-regressions."*

**Adversarial Refutation:**
1. **Unexercised Complexity:** The entire Tier C macro-regression machinery ($S_5 \to S_2$, Tarjan SCC condensation, $H^*$ pseudostate restoration) was designed to handle dynamically discovered circular import cycles. In the entire 118-project ReCodeAgent dataset, **there are zero circular dependency cycles** (D6). The include graphs are strictly acyclic. The Tier C machinery is dead weight that adds immense conceptual and implementation complexity without empirical justification.
2. **Liveness Deadlocks in Practice:** While the statechart is theoretically complete, in practice its orthogonal region coordination suffered from severe liveness bugs:
   - In 6 out of 6 pre-fix campaigns (`fixverify.log`), the wave scheduler deadlocked with `error=wave dispatch deadlock: no ready module while unhardened modules remain` (F2).
   - In single-module projects (`skel`), the absence of incoming dependency edges caused `resolve_parent_id()` to invent a fake parent `"parent_upstream"`, firing Tier B broadcasts into a void and incrementing the futile counter to fatal termination (F1).
3. **SLM Cognitive Overhead:** The statechart requires LLM agents to output structured broadcast events (`ev_ContractConflict`, `ev_CyclicDependencyFound`). SLMs frequently emit malformed event payloads or misclassify intrinsic syntax bugs as extrinsic contract conflicts.

**Conclusion:** The Harel Statechart's depth and broadcast calculus introduce substantial failure surfaces that crippled early daemon lineages. A simpler, dependency-directed actor model with localized contract negotiation is superior for SLM regimes.

---

### RQ3 (Partially Falsifiable). Is Encapsulated Intra-Module MCTS (ARIADNE) Tractable on a Single GPU?

**Incumbent Claim (`METHODOLOGY.md` §4):**
> *"Inside state M2, module $v_i$ is solved by an isolated ARIADNE cluster consisting of a Deterministic MCTS Controller, 4 Generative LLM Subagents, and a Two-Stage Hybrid Evaluator operating over $\mathcal{B}^{(v_i)}$."*

**Adversarial Refutation:**
1. **Severe GPU Contention & Serial Latency:**
   On a single RTX 3090, local inference throughput is capped at ~268 tokens/s aggregate. Running MCTS requires evaluating child nodes sequentially or through tight batching:
   - Expanding a single hypothesis: $\mathcal{A}_{\text{str}}$ generates 3 strategies $\to$ $\mathcal{A}_{\text{gen}}$ synthesizes candidate $\to$ $\mathcal{A}_{\text{test}}$ generates characterization tests $\to$ compiler runs $\to$ $\mathcal{A}_{\text{eval}}^{\text{distill}}$ scores. This sequence consumes 4–6 LLM inferences per rollout.
   - At $B_{\max} = 32$ rollouts, a single module requires 120–180 LLM requests.
   - On the `toml` project (old binary), MCTS burned **774 LLM requests and 17.1 hours** of continuous GPU time before collapsing into an 18-line stub.
2. **Cold-Start Penalty Across Dispatches (F7):**
   The incumbent specifies that when an ARIADNE cluster hits budget exhaustion ($B_{\max} = 16$), the module parks in `M0_Blocked` and increments the futile counter. When redispatched, the MCTS tree is re-initialized from scratch! Discarding the search tree forces the engine to re-explore failed strategy branches, guaranteeing that difficult modules (heapq merge, rbt rotations) never converge within the 3-dispatch futile cap.

**Conclusion:** Full MCTS with multiple specialist subagents is too heavyweight for complex modules under local SLM inference. A **portfolio race** (generating $K$ diverse initial architectures in parallel, selecting the best compiling candidate, and performing greedy localized repair) yields dramatically higher token efficiency.

---

### RQ4 (Falsifiable). Does the Incumbent Methodology Prevent the SLM "Dumb Zone"?

**Adversarial Refutation:**
1. **The Compaction Blind Spot:**
   `METHODOLOGY.md` discusses `TaskActivationEnvelope` hydration, but **contains zero provisions for context compaction**.
   - During MCTS rollouts inside state `M2`, the repair agent $\mathcal{A}_{\text{patch}}$ receives cumulative compiler diagnostics, prior AST diffs, and test failure logs.
   - Without explicit compaction, prompt lengths rapidly exceed 16,000 tokens.
   - In 27B-class SLMs, entering the 16k–32k context regime degrades attention, causing the model to emit repetitive, hallucinated edits or ignore negative constraints.
2. **Need for Frequent Intentional Compaction:**
   An SLM-optimized methodology must enforce a strict invariant: **no agent activation envelope may exceed the Smart Zone ($\le 12\text{k tokens}$)**. Past execution history must be deterministically compressed into structured fault summaries before dispatching repair rollouts.

---

### RQ5 (Falsifiable). Does the Methodology Cover the Real-World Failure Taxonomy (F1–F8)?

Cross-referencing the incumbent mechanisms against the eight observed failure modes in `02-evidence-base.md`:

| Failure Mode | Incumbent Coverage in `METHODOLOGY.md` | Audit Verdict |
|---|---|---|
| **F1 (Fabricated Counterparty)** | Prohibits Tier B without real edge, but provided no handling for parentless budget exhaustion. | **FAILED in v1–v16** (Fixed in v18 code via `BudgetExhausted`). |
| **F2 (Wave Deadlock)** | Assumes ReadySet is always non-empty while unhardened modules exist. | **FAILED** (Deadlocked when modules entered invalid states). |
| **F3 (heapq Paradox)** | Claims $R=1.0 \iff \text{parity}$, but separation of reward from gate caused infinite loop. | **FAILED** (Calibration defect). |
| **F4 (Stub Collapse)** | Relies on $\mathcal{A}_{\text{eval}}^{\text{distill}}$ to catch stubs, but test passed stub cleanly. | **FAILED** (Oracle blindness). |
| **F5 (OOM Terminations)** | No memory budgeting or process isolation specified in methodology. | **UNCOVERED** (Ops failure). |
| **F6 (Transport Hangs)** | Assumes instant, reliable tool execution. | **UNCOVERED** (Required external watchdog). |
| **F7 (Budget Non-Convergence)** | Uniform $B_{\max} = 16$; no warm-start carryover across dispatches. | **FAILED** (Modules died at cap 3). |
| **F8 (Metric Drift)** | Formal definitions conflated build success with execution pass rate. | **FAILED** (Fixed in v18). |

**Conclusion:** The incumbent methodology directly induced F1, F2, F3, and F4, and offered zero mitigation for F5, F6, and F7.

---

### RQ6 (Partially Falsifiable). Is the Incumbent Methodology Pareto-Optimal?

**Adversarial Verdict: NO.**  
Along the multi-dimensional Pareto frontier:
- **Correctness:** Compromised by oracle gaming (F4) and reward calibration disconnects (F3).
- **Cost & Token Economics:** Severely degraded by unpruned MCTS branching and multi-hour overruns (toml 17.1 h).
- **SLM Feasibility:** Assumes frontier reasoning; lacks intentional compaction; suffers from context rot in long loops.
- **Simplicity & Maintainability:** The Harel Statechart with XOR/AND orthogonality, broadcast calculus, and Tarjan SCC condensation represents massive architectural over-engineering for repos that are strictly acyclic.

A truly Pareto-optimal methodology must simplify the control topology, replace MCTS with a budgeted portfolio race, enforce strict context compaction, and harden the acceptance gate with stub-sensitivity.
