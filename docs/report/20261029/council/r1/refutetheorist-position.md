# RefuteTheorist — Round 1 Attack Position (Targets: Incumbent METHODOLOGY.md)

**Target:** `docs/references/METHODOLOGY.md` (§1, §4, §5) and `docs/references/METHODOLOGY-DRV.md`.
**Persona:** Theoretical Skeptic (Search theory, reward gating soundness, state-space complexity, liveness).

---

## 1. Steelman

The incumbent methodology (`THESEUS`) presents a mathematically grounded architecture for repository-scale translation:
1. It replaces naive single-path LLM prompting with Monte Carlo Tree Search guided by an exploratory Softmax-UCB rule (§4 item 1), balancing empirical reward $\bar{R}(u)$, exploration bonus $C_{\text{exp}}\sqrt{\ln(N(v)+1)/N(u)}$, and prior probability $P_{\text{prior}}(u)$.
2. It constructs a gated hierarchical reward function $R(s_t) \in [0.0, 1.0]$ (§4 item 4) where compilation is an absolute gating condition ($w_{\text{comp}}=0.2$), followed by characterization pass rate ($w_{\text{char}}=0.6$) and semantic rule compliance ($w_{\text{hard}}=0.2$).
3. It proves state-space reduction from $O(m^{|V|})$ to $O(m \cdot |V|)$ using Harel statechart AND-orthogonality (§1 item 2).

---

## 2. Objections

### T-r1-1: Sufficiency of the Gated Reward $R(s_t) = 1.0$ is Arithmetically False under Weak Oracles
- **Severity:** **FATAL**
- **Evidence Tier:** E1 receipt (`benchmarks/results/daemon/skel-toml.json`) + E4 derivation.
- **Target:** `METHODOLOGY.md` §4 item 4: *"achieving $R(s_t) = 1.0$ is a necessary and sufficient condition for full compilation, 100% characterization test parity, and full semantic rule adherence."*
- **Mechanism:** In `skel-toml.json`, the candidate module collapsed into an 18-line stub:
  ```javascript
  module.exports = { parse: () => ({}), stringify: () => "" };
  ```
  The test suite executed with zero assertions failing, yielding $\text{PassRate}_{\text{char}} = 1.0$. The second-stage LLM judge assigned $\text{Score}_{\text{harden}} = 1.0$. Consequently, the scalar reward evaluates to:
  $$R = 0.2(1) + 1[0.6(1.0) + 0.2(1.0)] = 1.0$$
  The incumbent certified $R=1.0$ on an empty stub that implements zero domain behavior. Sufficiency is formally refuted: $R=1.0 \centernot\implies \text{SemanticParity}$.

### T-r1-2: Softmax-UCB Convergence Guarantees Collapse under Non-Stationary LLM Priors
- **Severity:** **MAJOR**
- **Evidence Tier:** E4 derivation.
- **Target:** `METHODOLOGY.md` §4 item 1: selection via Softmax over UCB scores.
- **Mechanism:** Classical UCB guarantees $O(\log N)$ regret only under stationary reward distributions with bounded noise. In `THESEUS`, the reward signal is derived from an SLM judge ($\text{Score}_{\text{harden}}$) whose evaluations drift as previous rollouts pollute the shared context. Furthermore, applying Softmax over UCB scores exponentiates the exploration bonus, destroying the polynomial bound on sub-optimal arm selection and degenerating into uniform random search when temperature $T > 0.5$.

### T-r1-3: The $O(m^{|V|})$ State-Space Blowup Claim is a Strawman Comparison
- **Severity:** **MAJOR**
- **Evidence Tier:** E4 derivation.
- **Target:** `METHODOLOGY.md` §1 item 2 & DRV turn 3 §[3]: *"a flat FSM requires $4+6^{100}$ states."*
- **Mechanism:** A multi-module dependency DAG does not require tracking the Cartesian product of all uncoordinated module states. In standard compiler dependency analysis and task scheduling, a topological sort requires only $O(|V| + |E|)$ space and time. Claiming that Harel statecharts are required to avoid an exponential $O(m^{|V|})$ state space conflates global joint state representation with local scheduling.

---

## 3. Evidence & Derivations

### Derivation of Reward Gaming (T-r1-1):
Let candidate AST be $C_{\text{stub}}$ with length $|C_{\text{stub}}| = 18 \text{ LoC}$.
Let oracle test suite be $\mathcal{T} = \{t_1, \dots, t_k\}$.
If $\forall t \in \mathcal{T}$, $t$ checks output truthiness without deep structural assertion, $t(C_{\text{stub}}) \to \text{PASS}$.
$$\text{PassRate}_{\text{char}}(C_{\text{stub}}) = \frac{\sum_{i=1}^k \mathbb{I}[t_i(C_{\text{stub}}) == \text{PASS}]}{k} = \frac{k}{k} = 1.0$$
Under `METHODOLOGY.md`, $\mathbb{I}_{\text{compile}} = 1$. If the evaluator is an SLM under context saturation, $\text{Score}_{\text{harden}} \approx 1.0$. Thus $R(s) = 1.0$, Q.E.D.

---

## 4. Requests to the Council
1. Enter finding `T-r1-1` in the master ledger as **FATAL**.
2. Demand that any alternative methodology replace scalar reward gating with a conjoined predicate requiring active stub-sensitivity.
