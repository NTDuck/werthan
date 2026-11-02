# RefuteTheorist — Round 0 Calibration Position

**Persona.** I hunt unsoundness in *formal* claims: search-theoretic properties of the
Softmax-UCB/ARIADNE selection rule under non-stationary, noisy LLM rewards; reward-function
gating claims; state-space complexity claims; and liveness/deadlock properties of the control
topology. I exist because "R=1.0 is necessary and sufficient" survived until receipt
arithmetic was actually run. Round 0 = calibration only: claims I will hold to account,
falsifier classes, RQ ownership, planned attack surfaces. No substantive objections yet.

## Steelman

What the incumbent (`docs/references/METHODOLOGY.md`, + `METHODOLOGY-DRV.md`) claims, from my
lens, in its strongest form — each with a file+quote anchor:

- **FT-1 (Reward gating, necessity+sufficiency).** §4 item 4: "achieving $R(s_t) = 1.0$ is a
  necessary and sufficient condition for full compilation, 100% characterization test parity,
  and full semantic rule adherence," where $R = w_{\text{comp}}\mathbb{I}_{\text{compile}} +
  \mathbb{I}_{\text{compile}}[w_{\text{char}}\text{PassRate}_{\text{char}} +
  w_{\text{hard}}\text{Score}_{\text{harden}}]$ with weights $(0.2, 0.6, 0.2)$
  (METHODOLOGY.md, "Gated Hierarchical Reward Function"). The *necessity* direction is
  arithmetically forced given the stated weight caps and ranges — I grant it. The *sufficiency*
  direction rests on $\text{Score}_{\text{harden}}$ (an LLM parity score, §4 Stage 3b) actually
  measuring rule adherence, and on $\text{PassRate}_{\text{char}}$ being measured on a
  compiled candidate.
- **FT-2 (Softmax-UCB soundness).** §4 item 1: UCB$(u) = \bar{R}(u) + C_{\text{exp}}
  \sqrt{\ln(N(v)+1)/N(u)} + \gamma \cdot P_{\text{prior}}(u)$, with child selection "following
  a temperature-controlled softmax distribution … To avoid premature greedy convergence on
  flawed refactoring templates." Strongest form: the hybrid inherits UCB's exploration
  guarantee while the softmax term adds robustness to noisy LLM reward estimates.
- **FT-3 (State-space complexity).** §1 item 2: "tracking $|V|$ modules across $m$ module
  states requires $O(m^{|V|})$ composite states," reduced by orthogonality to
  "$O(m \cdot |V|)$" (Design Decision 2; also DRV turn 3 §[3], "a flat FSM requires $4+6^{100}$
  states").
- **FT-4 (Liveness/containment of the control topology).** §5 preamble: the three-tier
  regression calculus "ensur[es] that errors are contained to the smallest possible scope";
  Tier B surgical broadcast ("Surgical Orthogonal Transition"), Tier C deep-history restore,
  and the warm-start path M5→M2. Algorithm 1's `While $S_{\text{macro}} \neq \texttt{TERMINATED}$`
  asserts termination semantics without stating a termination argument.
- **FT-5 (Warm-start validity).** §4 M5_Invalidated "preserving the existing MCTS search tree
  $\mathcal{T}^{(v_i)}$, strategy hypotheses $\mathcal{B}_{SH}^{(v_i)}$, and test suite
  $\mathcal{B}_{CT}^{(v_i)}$" — i.e., retained statistics remain meaningful search state after
  the problem model $\mathcal{B}_{PM}$ (and possibly $\mathcal{B}_{CT}$) mutates.
- **FT-6 (Surgical O(1) claim).** §1 item 2: "Broadcast Event Calculus (for surgical
  $O(1)$-hop cross-module regressions)."

## Objections — planned attack surfaces for Round 1

Listed, not argued. Each pairs a target with the falsifier class I intend to attempt.

- **T-r1-1 (planned, targets FT-1 sufficiency).** Class: E1 receipt arithmetic + E4 derivation.
  Attempt: reconstruct from `benchmarks/results/daemon/*.json` a candidate reaching the
  R=1.0 gate shape while failing behavioral parity (the "toml 18-line stub at
  `test_pass_rate=1.0`" receipt shape per 01-meta-strategy §1 RQ1). Preliminary probe
  (already run, to be formalized in R1): `skel-toml.json` records
  `test_pass_rate=1.0` with `compilation_success=false` — under FT-1's own gating, the
  receipt's PassRate field and the formal $\text{PassRate}_{\text{char}}$ cannot both be
  honest; either the implemented gate diverges from the formal one or the receipt semantics
  drift. Also attempt the necessity side's flip side: whether $R<1.0$ candidates can be
  behaviorally sound (the heapq paradox, 20251025 §03: artifact-level PASS never certified).
- **T-r1-2 (planned, targets FT-2).** Class: E2 (UCT/softmax-UCB regret pathologies under
  noisy, non-stationary rewards — verified against primary sources in-round) + E4 derivation.
  Attempt: show (a) softmax sampling over UCB scores is not UCB1 and carries no published
  regret bound; (b) the search violates UCB's i.i.d./stationarity assumption three ways —
  $\mathcal{B}_{PM}$ mutates mid-search (Tier B warm-start), $\mathcal{B}_{CT}$ expands
  mid-search (Algorithm 2 step 2 expansion), and $P_{\text{prior}}$ is LLM-generated, so
  $\bar{R}(u)$ statistics are means over *different oracles at different times* and
  non-comparable across nodes.
- **T-r1-3 (planned, targets FT-3).** Class: E4 derivation + E2 (Harel 1987 primary
  verification). Attempt: the $O(m^{|V|})$ baseline is a strawman — a flat FSM with per-module
  *factored* state variables is already $O(m \cdot |V|)$; $m^{|V|}$ only arises if one insists
  on enumerating the cross-product as a single flat state. If so, orthogonality buys the same
  asymptotics as any factored-state representation, and the complexity claim downgrades from
  "Harel reduces $O(m^{|V|})$ to $O(m\cdot|V|)$" to "Harel equals factored state here."
- **T-r1-4 (planned, targets FT-4).** Class: E4 counterexample construction (small adversarial
  event sequences on the Algorithm 1/2 transition system) + E1 (20251025 failure taxonomy).
  Attempt: (a) Tier B ping-pong — $v_i$ exhausts $B_{\max}$, blames $v_p$; $v_p$ warm-starts,
  re-hardens; $v_i$ exhausts again — the topology has no global counter on Tier B round trips
  (the futile-dispatch cap was an *observed ad-hoc patch*, not a liveness property: 20251025
  §Key findings 1 — binaries v1–v16 "died at the futile-dispatch cap (3)"); (b) the
  Budget-Exhaustion Fallback (Algorithm 2, final step) pushes
  `ev_ContractConflict(Parent(v_i), …)` — for a parentless root vertex this is precisely the
  fabricated-counterparty liveness death observed in E1; (c) Tier C quiesce: regions in
  mid-flight LLM rollouts inside `M2` are checkpointed via $\Theta_{H^*}(S_5) \leftarrow
  \mathbf{M}$, but in-flight search trees $\mathcal{T}^{(v_i)}$ of *active* (non-hardened)
  regions and child-edge remapping into collapsed $v_{\text{SCC}}$ are unspecified.
- **T-r1-5 (planned, targets FT-5).** Class: E4 derivation. Attempt: after $\Delta_{\text{spec}}$
  is appended to $\mathcal{B}_{PM}^{(v_p)}$, retained $\bar{R}(u)$ means were computed under the
  old constraint set — stale statistics bias Softmax-UCB toward pre-update strategies; the
  warm-start "few $\mathcal{A}_{\text{patch}}$ rollouts" efficiency claim (§5 Tier B) is
  asserted, never bounded or measured.
- **T-r1-6 (planned, targets FT-6).** Class: E4. Attempt: a contract conflict at DAG depth $d$
  can cascade (parent re-hardening changes exports → its parent invalidated → …), so total
  regression traffic is $O(\text{depth})$ events; "O(1)-hop" is true only per-event and may
  mislead about cascade cost unless a cascade bound exists.

## Evidence & Derivations

- **E1 anchors held in reserve:** `benchmarks/results/daemon/skel-toml.json`
  (`test_pass_rate=1.0`, `compilation_success=false`, `llm_requests=774`);
  `skel-heapq.json` (honest PASS at `llm_requests=107`, reward never certified — 20251025
  §Key findings 3); aggregate 1,858 requests / 1 honest pass in 8 campaigns
  (docs/report/20251025/README.md). All campaigns terminal 2026-10-26; n=8 corpus scope
  caveat mandatory per K7.
- **Falsifier classes per claim:** FT-1 → E1 receipt construction + E4; FT-2 → E2 primary-source
  verification + E4 (no-regret-bound argument, shown in full); FT-3 → E4 + E2 (Harel 1987);
  FT-4 → E4 explicit event-sequence counterexamples + E1; FT-5 → E4; FT-6 → E4.
- **Anticipated defense I pre-register (to keep myself honest):** for FT-2, the defender may
  concede regret-optimality and retreat to "softmax-UCB is a heuristic, no bound claimed."
  That retreat is acceptable only if METHODOLOGY.md's phrasing is downgraded in the same
  repair — the current text motivates the mechanism by *avoiding* premature convergence, an
  empirical claim, so a heuristic-retreat must also drop or evidence that motivation
  (anti-deflation check, §5.4.5).
- **No E5 claims used in this paper.** Session-corpus citations in DRV (ReCodeAgent, GVS5H,
  ARIADNE arXiv IDs) are RefuteLiterature's lane; I will consume their verification results in
  Round 2+ rather than duplicate.

## Requests to the council

1. **RQ ownership (primary).** I claim **RQ1** (formal claims under actual oracle strength) —
   all three sub-claims of FT-1 live here, and the receipt-shape falsifier is explicitly
   named in §1 RQ1.
2. **RQ ownership (partial).** **RQ4** limited to the "forced vs aesthetic" verdicts for the
   deterministic statechart decision (FT-3) and the encapsulated Softmax-UCB MCTS decision
   (FT-2) — the arithmetic leap from transducer-FSM evidence to Harel orthogonality named in
   §1 RQ4 is mine to quantify. **RQ5** limited to the *recording-vs-preventing* axis for
   reward-gated mechanisms (whether M4's $R=1.0$ exit *prevents* vacuous passes or merely
   *records* them). **RQ8** limited to drafting falsification tests for surviving formal
   claims.
3. **Not mine:** RQ2 (unexercised-machinery audit — RefuteCoverage/RefuteFeasibility territory;
   I take Tier C *liveness semantics* but not its exercised/cost audit), RQ3 (baseline
   payoff), RQ6 (budget arithmetic — RefuteScaler), RQ7 (scoping).
4. **Request to the chair:** in Round 1, when receipts are cited for RQ1, require the defender
   to state whether `test_pass_rate` in the daemon schema is the same quantity as
   $\text{PassRate}_{\text{char}}$ in METHODOLOGY.md — the FT-1 attack turns on this
   identification, and it is a one-line definitional answer.
5. **Request to peers:** RefuteLiterature — when verifying the UCT/ARIADNE literature thread
   (L1), flag which sources claim regret bounds for *softmax*-UCB variants specifically;
   RefuteMeasurement — own the receipt-schema-semantics drift question
   (`compilation_success` semantics drift is documented in 20251025 §03) so my T-r1-1 and
   their provenance audit do not double-count the same defect.
