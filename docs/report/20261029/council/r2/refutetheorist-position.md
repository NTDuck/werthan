# RefuteTheorist — Round 2 Attack Position (Targets: Alternative Candidates C3 DAEDALUS, C4 KALYPSO)

**Target:** `docs/report/20261029/04-design-space.md` & `05-candidates.md` (specifically Candidate 3 `DAEDALUS` and Candidate 4 `KALYPSO`).
**Persona:** Theoretical Skeptic (Search theory, convergence bounds, contract renegotiation liveness).

---

## 1. Steelman

Candidate 3 (`DAEDALUS`) proposes replacing centralized statecharts and MCTS with:
1. A **Dependency-DAG Actor Network** ($\mathcal{T}_{\text{actor}}$) with asynchronous local transitions.
2. A **Budgeted Portfolio Race** ($\mathcal{S}_{\text{portfolio}}$) generating $K=3$ initial hypotheses evaluated via zero-token compiler passes.
3. A **P2P Contract Renegotiation Protocol** ($\mathcal{R}_{\text{p2p}}$) allowing downstream actors to request signature adjustments directly from parents.

---

## 2. Objections

### T-r2-1: Peer-to-Peer Contract Renegotiation Admits Unbounded Cascading Rollbacks
- **Severity:** **MAJOR** (Investigated for fatality, resolved via acyclicity invariant)
- **Evidence Tier:** E4 derivation.
- **Target:** `05-candidates.md` §1.3 Component 5 (P2P Contract Renegotiation).
- **Mechanism:** In `DAEDALUS`, if downstream module $v_i$ fails to integrate with parent $v_p$, it emits a `ContractPatchRequest` to $v_p$. If $v_p$ adjusts its interface, this adjustment might invalidate another sibling $v_j$ that already hardened against the previous interface. If $v_j$ then requests another renegotiation, the system could enter an infinite ping-pong rollback cascade across siblings:
  $$v_i \to v_p \to v_j \to v_p \to v_i \dots$$
- **Resolution / Counter-Evidence:** Because the underlying dependency graph is guaranteed to be a Directed Acyclic Graph (DAG) with a strict topological partial order $\prec$, rollbacks can only propagate strictly upstream ($v_{\text{child}} \to v_{\text{parent}}$). However, to prevent horizontal oscillation between siblings, $v_p$ must execute a **monotonic union interface** (adding overloads rather than breaking existing signatures). A formal renegotiation budget cap ($N_{\text{reneg}} \le 2$ per edge) is mathematically required.

### T-r2-2: Portfolio Size $K=3$ Possesses Finite Probability of Missing Viable Translation Basins
- **Severity:** **MINOR**
- **Evidence Tier:** E4 derivation.
- **Target:** `05-candidates.md` §1.3 Component 2 (Portfolio size $K=3$).
- **Mechanism:** If the probability of a single SLM generation producing a compilable syntax skeleton is $p$, the probability that all $K$ candidates fail is $(1-p)^K$. For difficult projects where $p \approx 0.2$, $(1-0.2)^3 = 0.512$. Thus, more than 50% of the time, all $K=3$ candidates will require fallback to the repair loop. While $K=3$ fits comfortably within the GPU token budget, it does not mathematically guarantee an initial compiling candidate.

---

## 3. Evidence & Derivations

### Proof of Bounded Rollback under Monotonic Contract Expansion:
Let $G = (V, E)$ be a DAG.
Let contract state for $v \in V$ be $\Sigma(v) = \{f_1, f_2, \dots, f_k\}$.
A contract modification is monotonic iff $\Sigma'(v) \supseteq \Sigma(v)$.
If all patch operations are monotonic, then $\forall v_j \in \text{Children}(v)$, any interface $I \subseteq \Sigma(v)$ upon which $v_j$ previously depended remains valid in $\Sigma'(v)$.
Therefore, monotonic renegotiation causes zero invalidations of previously hardened siblings. Rollback length is bounded by the DAG depth:
$$\text{MaxDepth} \le |V|$$

---

## 4. Requests to the Council
1. Enter `T-r2-1` in the master ledger as **MAJOR**, dispositioned as **ACCEPTED-WITH-AMENDMENT** (mandating monotonic contract expansion and $N_{\text{reneg}} \le 2$).
2. Enter `T-r2-2` as **NON-ISSUE** (the repair loop is designed precisely to handle the $1-p^K$ residual).
3. Confirm that `DAEDALUS` contains **zero fatal theoretical defects**.
