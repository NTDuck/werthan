# Round 0 — Calibration Position Paper: RefuteScaler

Persona: **RefuteScaler** (budget & scalability arithmetic). Round: 0 (calibration — no
substantive attacks; attack surfaces listed, not argued). Date: 2026-10-30.
Lens commitment: token counts, GPU-seconds, fan-out/slot contention, state-vector
growth in $|V|$, worst-case single-module pathologies, wall-clock bounds. I exist
because a 13.3 h mutation-check plan once survived a full design cycle inside a 2 h
bound before anyone did the arithmetic.

## Steelman

What the incumbent methodology claims, from my lens — i.e., its quantitative/cost
claims, each with a file+quote anchor. I steelman these as the strongest defensible
versions; these are the claims I will hold to account in Round 1.

- **C1 (state-space compression).** Flat-FSM control forces lockstep rollbacks and
  $O(m^{|V|})$ composite states; Harel AND-regions reduce module-state complexity to
  $O(m \cdot |V|)$. Anchor: `docs/references/METHODOLOGY.md` §1.2: "tracking
  $|V|$ modules across $m$ module states requires $O(m^{|V|})$ composite states" →
  "reducing module state complexity from $O(m^{|V|})$ to $O(m \cdot |V|)$".
- **C2 (surgical regression cost).** Broadcast Event Calculus makes cross-module
  regressions "surgical $O(1)$-hop" events that touch only $v_p$ and $v_i$. Anchor:
  METHODOLOGY.md §1.2: "Broadcast Event Calculus (for surgical $O(1)$-hop
  cross-module regressions)".
- **C3 (fan-out context economy).** Declarative Scatter-Gather eliminates the
  "$O(K)$ context bottleneck" of parent-buffered subagent returns, and every shard
  respects the SLM context constraint $\text{Tokens}(\text{shard}_k) \le L_{\max}$.
  Anchor: METHODOLOGY.md §1.3 and §3.1.
- **C4 (reward sufficiency).** $R(s_t)=1.0$ is "a necessary and sufficient condition
  for full compilation, 100% characterization test parity, and full semantic rule
  adherence", with weights $(0.2, 0.6, 0.2)$. Anchor: METHODOLOGY.md §4 "Gated
  Hierarchical Reward Function".
- **C5 (encapsulation bounds search cost).** Per-vertex ARIADNE encapsulation avoids
  the global "$O(|V| \times |\mathcal{A}|)$ action-space explosion"; per-module cost
  is bounded by the rollout budget $B_{\max}$. Anchor: METHODOLOGY.md §1.5, §4/Alg 2.
- **C6 (warm-start cheapness).** A regressed module "initializes a child repair
  branch directly from $c_p^\star$ … adapting its interface in a few
  $\mathcal{A}_{\text{patch}}$ rollouts without re-discovering core logic." Anchor:
  METHODOLOGY.md §5.2 "Warm-Start Efficiency".
- **C7 (free deep-history resume).** On $H^*$ re-entry, unaffected regions are
  "restored directly to their prior state … preserving M4_Hardened modules without
  re-execution." Anchor: METHODOLOGY.md §3 (Deep History Pseudostate); derivation
  transcript `METHODOLOGY-DRV.md` §[3]: "resume directly in M4_Hardened without
  re-running a single token."
- **C8 (wall-clock viability under the regime).** The deployment regime carries a
  2 h campaign deadline (X3: "2 h campaign bound"); the methodology's machinery is
  asserted compatible with it (implied by the whole of METHODOLOGY.md; nowhere does
  it state a budget/wall-clock mechanism inside $\Sigma(\mathcal{G})$). Regime anchor:
  `docs/report/20261029/01-meta-strategy.md` §2 X3.

## Objections

Round 0: planned attack surfaces only (IDs S-r1-* begin in Round 1). Each listed with
the falsifier class I intend to attempt.

1. **Flat-vector redundancy of C1.** The incumbent's own state vector
   $\Sigma(\mathbf{M}) = [M(v_1),\dots,M(v_{|V|})]$ (Def. 1) *is* an
   $O(m\cdot|V|)$-configuration representation without any Harel machinery. If a flat
   per-module vector plus a Kahn scheduler achieves the same configuration-space size,
   orthogonality buys nothing on this axis and C1's justification collapses to
   aesthetic (RQ4/X5 machinery tax). Falsifier class: E4 derivation showing
   configuration count identical under both formalisms.
2. **Unbounded control loops.** Algorithm 1 contains "re-invoke $W_k$ until
   $g_{\text{struct}}$ passes" and a `continue loop` on `REJECT_LOCAL` with no stated
   retry cap; $\mathcal{Q}_{\text{events}}$ is a single FIFO shared by $|V|$
   orthogonal regions (contention/serialization unbounded). No retry cap, no
   per-phase token cap, no deadline state exists in $\Sigma(\mathcal{G})$.
   Falsifier class: E4 worst-case replay of documented constants showing the 2 h
   bound unenforceable; E1 citation of toml (17.1 h = 8.5× bound).
3. **Receipts vs the wall-clock and token claims (C6/C8).** Executed receipts
   (`benchmarks/results/daemon/*.json`, E1): the single honest PASS (heapq) ran
   11,669,900 ms ≈ 3.24 h — over the 2 h bound with 107 requests for one ~500-LoC
   single-vertex module; toml burned 774 requests / 17.1 h; aggregate 1,858 requests
   with 774 on one pathological module (>7× failure-cost asymmetry;
   `01-meta-strategy.md` §2 X2). Falsifier class: E1 arithmetic table
   (RQ6-style: per-module dispatches × agents-per-dispatch × tokens-per-call vs
   measured throughput ~268 tok/s shared).
4. **Rollout-ceiling arithmetic vs convergence needs.** Documented engine defaults
   give $B_{\max}=16$ rollouts/dispatch × futile cap 3 = 48 rollout chances; three of
   eight projects failed to converge inside it (`20251025/README.md` §2.2), and
   METHODOLOGY.md never states how $B_{\max}$ scales with module size — no
   $\text{LoC} \to B_{\max}$ relation anywhere. Falsifier class: E4 table of
   requests/rollout vs corpus convergence evidence; E1 gap analysis.
5. **Hidden $O(K)$/$O(|V|)$ costs relocated, not removed (C3/C7).** The Reduce/Linker
   "queries only the exported boundary headers" across all $K$ shards — input size
   grows with $K\cdot\text{boundary}$; each Tier C checkpoint copies the whole
   $|V|$-vector ($\Theta_{H^*}(S_5) \leftarrow \mathbf{M}$); warm-start retains full
   MCTS trees $\mathcal{T}^{(v_i)}$, $\mathcal{B}_{SH}$, $\mathcal{B}_{CT}$ per
   module — memory growth in $|V|$ with no stated bound (host OOM killed 3 of 8
   campaigns with 5 concurrent daemons, E1). Falsifier class: E4 memory/IO growth
   derivation; E1 OOM receipts.
6. **Fan-out slot contention unmodeled.** Regime throughput ~268 tok/s is *shared*;
   $S_5$ asserts $|V|$ concurrent regions and scatter phases assert $K$ concurrent
   workers, but METHODOLOGY.md states no concurrency cap, so effective per-worker
   throughput divides and the wall-clock claim (C8) silently assumes the division is
   free. Falsifier class: E4 throughput-division arithmetic; E1 request-latency
   recompute from receipts (~109 s/request average on heapq's 107 requests / 3.24 h).
7. **Worst-case single-module pathology.** $B_{\max}$ bounds rollouts, not tokens or
   seconds; one rollout can burn the 600 s `inference_timeout` (E1: html >15 s main
   loop, watchdog/restart-cap churn). METHODOLOGY.md has no per-module wall-clock or
   token budget, so a single adversarial module can dominate the campaign bound.
   Falsifier class: E4 bound construction; E1 html/strsim restart-cap receipts.
8. **Tier C cost/benefit ratio (RQ2).** Tier C machinery (checkpoint $|V|$-vector,
   $S_5\to S_2$ re-entry, SCC re-stubbing through $S_3$/$S_4$, $H^*$ restore) has
   material per-event cost and zero exercised instances in the corpus (zero SCCs).
   If cost > 0 and validation = 0, the machinery-verification tax (X5) applies.
   Falsifier class: E1 corpus audit (zero Tier C executions) + E4 per-event cost
   derivation.

## Evidence & Derivations

- **E1 receipts** (`benchmarks/results/daemon/skel-{heapq,toml,bst,colorsys,mathgen}.json`,
  re-read this round): heapq wall 11,669,900 ms / 107 reqs / 3 dispatches;
  toml 61,443,312 ms / 774 reqs; bst 22,144,759 ms; colorsys 14,159,337 ms;
  mathgen 17,653,051 ms. Every finished run violates the 10 min–2 h window or dies
  on comp=false; only heapq passes acceptance and it still exceeds 2 h.
- **E1 failure taxonomy** (`docs/report/20251025/README.md`): 1 honest pass in 8;
  48-rollout ceiling (§2.2); OOM kill of 3 in-flight daemons (postscript);
  5/8 campaigns dead on engine-accounting artifacts, not translation difficulty.
- **E4 pending derivations** (Round 1): flat-vector vs statechart configuration
  count; unbounded-loop wall-clock replay; throughput-division under fan-out;
  memory growth of retained MCTS trees in $|V|$.
- No literature claims this round; nothing above rests on E3/E5.

## Requests to the council

1. **RQ ownership claim.** I claim primary ownership of **RQ6** (token/compute
   economics executed as arithmetic, not asserted) and the cost/benefit-ratio clause
   of **RQ2** ("if the machinery's cost is material while its validation is nil, the
   critique must quantify that cost/benefit ratio"). I claim supporting (not sole)
   ownership of **RQ3**'s token-matched baseline arithmetic (baselines must be
   token-matched, so their budgets are my arithmetic too) and the X2/X3 measurements
   inside **RQ8**'s cost model.
2. **Boundary with RefuteTheorist.** RQ1's *soundness* of $R=1.0$ (C4) is theirs;
   the *cost* of the oracle stack (extra $\mathcal{A}_{\text{eval}}^{\text{distill}}$
   and $g_{\text{sem}}$ calls per rollout/phase, and the 7× failure-cost asymmetry
   those weak oracles create) is mine. I will not attack C4's logical sufficiency;
   I will attack its price.
3. **Boundary with RefuteMeasurement.** Receipt *provenance* and oracle-relativity
   are RefuteMeasurement's; I take receipt *arithmetic* (wall-clock, request counts,
   cost mixtures) as settled inputs once provenance is granted.
4. **Request to the chair.** X3's "2 h campaign bound" needs a canonical anchor
   (which document defines it) before I cite it as a falsifier premise; currently it
   appears in `01-meta-strategy.md` §2 X3 and implicitly in the benchmarks' verdict
   window (`20251025/README.md` §1, criterion: duration 10 min–2 h). I will treat the
   benchmarks window as the anchor unless corrected.
5. **Calibration note.** Per K1, my falsifier anchors are deliberately disjoint from
   the reward-soundness and coverage lenses: all eight surfaces above resolve to
   arithmetic on documented constants or executed receipts, not to formal-claim or
   citation disputes.
