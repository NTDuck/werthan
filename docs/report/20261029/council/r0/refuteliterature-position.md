# R0 — RefuteLiterature calibration position

Persona: **RefuteLiterature** (council, Round 0 calibration). Date: 2026-10-30.
Contract: `docs/report/20261029/README.md` + `01-meta-strategy.md` §4 (evidence tiers,
regime-transfer rule, session-corpus rule), §5.1–5.3 (persona, rounds, skeleton).
Round 0 = calibration only; no substantive attacks; no objection IDs yet (L-r1-* start in Round 1).
Independence honored: I have read nothing under `council/` except this file.

## Steelman

The incumbent (`docs/references/METHODOLOGY.md`, "the <...> framework"; derivation
record `METHODOLOGY-DRV.md`) makes the following **literature-grounded** claims, stated
in their strongest defensible form. Each is what I will hold to account — not strawmen.

**LM-1. Paradigm-failure citations anchor the five design decisions.** Each of the five
headline decisions is justified by a named existing paradigm's failure:
ReCodeAgent "lack[s] backward topological edges" (METHODOLOGY.md §1.2);
manager-worker systems "(e.g., GVS5H)" induce "state hallucination and context window
saturation … particularly in SLMs" (§1.1); autonomous `spawn_subagent` creates an
"O(K) context bottleneck … and uncheckpointed shadow state" (§1.3); flat FSMs force
"$O(m^{|V|})$ composite states" (§1.2); global MCTS causes an "$O(|V|\times|\mathcal{A}|)$
action-space explosion" (§1.5). The derivation (DRV turn 3, §5) attributes these to
"Ibrahimzada et al. (2026) ReCodeAgent", "Harel (1987)", "Salemi et al. / LbMAS
(2025–2026)", "Erman et al. (1980) Hearsay-II", "Wu et al. AutoGen", "LangGraph", and
"MOISE+ / ASPECS", plus the Claude Code Modernization plugin's 5-phase lifecycle (DRV
turn 5, B.4) and GVS5H "Zero-Shot Self-Orchestration with Ledger-Based Control" (DRV
turn 3, §4.2). Best form: these citations are offered as *mechanism evidence* (the
paradigms really lack the named capability), not as measurements of the alternatives'
performance.

**LM-2. The formalism's provenance claim.** §1.2 and DRV §[3] claim Harel (1987)
supplies exactly four mechanisms — hierarchy (XOR), orthogonality (AND), broadcast
event calculus, deep history $H^*$ — and that the system "is formal[ized] via a Harel
Statechart" (Definition 3). Best form: Harel 1987 genuinely introduces these; the
methodology uses them under their standard semantics.

**LM-3. The ARIADNE import.** §1.5 / §4.2 import ARIADNE ("Softmax-UCB MCTS planner,
4-slice local blackboard, 5-component specialist pool") with a stated boundary
adaptation (rule extractor + extrinsic blocker detector). Best form: ARIADNE is a real
published system whose reward formulation the methodology reproduces faithfully
(reward equation, softmax-UCB with prior term $\gamma \cdot P_{\text{prior}}$).

**LM-4. The meta-strategy's own E2 anchor list.** `01` §4 designates "Harel 1987; UCT
and its regret pathologies; StateFlow; MAST failure-mode frequencies" as E2
(peer-reviewed, primary-verified) evidence, and RQ4 pre-commits the transfer leap that
must be policed: "StateFlow's measured +13–28% at 3–5× lower cost for *transducer
FSMs* does not force *Harel orthogonality*." Best form: these are the load-bearing
published numbers the critique itself will rely on; they must survive the same
verification I demand of the incumbent.

**LM-5. Discipline claims the incumbent already makes.** The derivation file is an
honest session transcript (Gemini, 2026-10-21) — the incumbent does not claim it is a
peer-reviewed derivation, and `01` §4 already types future-dated citations as
`session-corpus` (never sole support for a verdict). The incumbent has pre-conceded
the tier of its own provenance.

## Objections

Round 0: planned attack surfaces only (per protocol §5.2); IDs and severity start Round 1.

| # (Round-1 plan) | Target claim | Attack surface (to be verified against primary source *during* Round 1) | Falsifying evidence class |
|---|---|---|---|
| A1 | LM-1 / ReCodeAgent | Verify "Ibrahimzada et al. (2026)" exists at a primary source (arXiv/journal); verify the 4-agent Analyzer→Planner→Translator→Validator topology and that the paper does not contain backward planning edges or a validation-driven re-plan loop the methodology ignores. Check the models it was measured on (frontier vs SLM) — regime mismatch for any capability claim imported here. | Nonexistent/misdated source; paper describes re-planning the critique denies; frontier-only measurement. |
| A2 | LM-1 / GVS5H | Verify GVS5H ("Zero-Shot Self-Orchestration with Ledger-Based Control"): does the paper itself claim to avoid state hallucination via ledger+verifier (which would make "induces state hallucination" an overclaim or at least require a regime argument)? Which models, what task scale? | Paper contradicts the attributed failure mode; no SLM regime evidence. |
| A3 | LM-3 / ARIADNE | Verify ARIADNE's existence, exact name expansion, agent pool count, softmax-UCB formula (incl. the $\gamma \cdot P_{\text{prior}}$ term — is that in the source or added here?), and 4-slice blackboard. Check whether ARIADNE's evaluation regime (model class, budget) supports any SLM-scale benefit inference. | Name/author/fabrication risk; formula drift; evaluation on frontier models only. |
| A4 | LM-2 / Harel semantics | Verify Harel 1987 against the primary text: (a) does the broadcast/event semantics there match an *asynchronous FIFO* event queue $\mathcal{Q}_{\text{events}}$ (Definition 1) plus "boolean verdict from an LLM critic" (two-stage guards, §3.2), or is the incumbent's semantics a synchronous-broadcast deviation being passed off as Harel's? (b) does deep history $H^*$ restore *state* only, whereas the methodology also restores search trees/blackboards $\mathcal{T}, \mathcal{B}$ (M5 → M2 warm start, §5.2)? | Semantic deviation attributed to the formalism; misattribution of properties Harel does not grant. |
| A5 | RQ4 leap (StateFlow) | Verify StateFlow's primary source: the +13–28% and 3–5× cost numbers, the model regime, and whether the paper measures *hierarchical* statecharts or flat transducers — the exact transfer argument `01` RQ4 already flags. | Number not in source; measured on transducer-only tasks; frontier-model regime. |
| A6 | LM-1 / LbMAS + Hearsay-II | Verify "Salemi et al., LbMAS" exists as a published system (name/author/date); verify Hearsay-II (Erman et al. 1980) attribution and that the blackboard-lineage claim (specialists self-trigger off a shared representation) is accurately stated. | Nonexistent or misattributed LbMAS; Hearsay-II lineage used as E5 dressed as E2. |
| A7 | LM-1 / MOISE+ | Verify MOISE+/ASPECS citations; the comparison matrix (DRV turn 3, §4) rates holonic "Deontic/role deliberation overhead" as "Very High" with no measurement — confirm no published measurement exists (E5 laundering risk in `05`'s dominance table). | No measured source; adjective-tier claim presented as comparative evidence. |
| A8 | Plugin alignment | Verify the Anthropic Claude Code Modernization plugin's 5 pre-DAG phases exist as described (DRV turn 5, B.4; METHODOLOGY.md §3 header "Aligning with the domain lifecycle of the Claude Code Modernization workflow") — a product claim, not a paper; check the cited repo. | Phantom plugin features; phase names/dates wrong. |
| A9 | Meta-strategy's own anchors | Verify "MAST failure-mode frequencies" and "UCT and its regret pathologies" as E2: does MAST report the frequencies the council will cite, and in what regime? Which UCT pathology paper, and does it cover softmax-UCB under noisy LLM rewards (not just adversarial bandits)? | Wrong frequencies; pathology result regime-mismatched to LLM rollouts. |
| A10 | LM-1 / complexity citations | The "$O(m^{|V|})$ vs $O(m\cdot|V|)$" argument is presented as a formal justification (§1.2) — check whether any cited source actually states this, and whether a flat-FSM strawman is being attacked (no one proposes enumerating the product space). | Ghost/strawman citation; arithmetic presented as literature-anchored when it is E4-only. |

Also logged as provenance observations (not objections): the derivation record's many
dangling "arXiv"/"ResearchGate" citation markers (DRV throughout) mean several cited
works' *locators were lost in the derivation path* — Round 1 must treat every DRV
citation as needing re-derivation from the primary source, including the ones that look
familiar (AutoGen's 2023 vs 2024 arXiv revision discrepancy is already visible in DRV
turn 2, §5).

## Evidence & Derivations

- Claim inventory method: full reads of `METHODOLOGY.md` (340 lines) and
  `METHODOLOGY-DRV.md` (880 lines); quote anchors above are to section heads/turns, to
  be narrowed to exact lines in Round 1 objections per `01` §5.4.2.
- Tier audit of the incumbent's literature, per `01` §4: Harel 1987, Hearsay-II,
  AutoGen, MOISE+ are plausibly E2 *after* verification (planned A4/A6/A7); ReCodeAgent,
  GVS5H, LbMAS, ARIADNE are candidate `session-corpus` (future-dated or unverifiable
  IDs) until proven otherwise (A1–A3); the plugin alignment is product documentation,
  not literature (A8); the $O(\cdot)$ arguments are E4 derivations that must not be
  presented as citation-backed (A10). Under the session-corpus rule, if A1–A3 fail
  primary verification, LM-1's paradigm-failure justifications drop to E5 for those
  paradigms and the affected design decisions fall back on E4/first-principles or
  E1 receipts — a demotion, not automatically a refutation.
- No numbers were verified this round (Round 0 prohibition); A1–A10 are the
  verification queue, to be executed against primary sources live in Round 1.

## Requests to the council

1. **RQ ownership.** I claim primary ownership of **RQ4** (forced vs supported vs
   aesthetic vs contradicted verdicts hinge on citation authenticity and regime-transfer
   arguments — the StateFlow transducer→Harel leap is my exemplar), secondary on
   **RQ2** (thread L2: whether statechart machinery is validated in practice anywhere
   published), and a supporting role on **RQ8** (overclaiming audit of the final
   verdict's wording). RQ1's reward-sufficiency formal claim I concede to
   RefuteTheorist; RQ3/RQ6 arithmetic to RefuteScaler/BaselineTypicality. This keeps
   ownership disjoint per risk K1.
2. **Verification budget.** Round 1 will involve live primary-source checks (A1–A10 ≈
   10 lookups). I ask the chair to accept live web verification as E2-confirming
   activity during rounds, per §5.1's mandate, and to record verification method per
   source in `07-bibliography.md`.
3. **Session-corpus pre-commitment.** I ask the council to agree now: any citation
   from the derivation transcript that fails primary verification in Round 1 is
   automatically stamped `session-corpus` in all downstream files, and any claim that
   leans on it alone is struck from `05`'s scoring, not merely footnoted.
4. **Symmetry request.** Per `01` §3, the incumbent's executed receipts count as
   evidence for it; symmetrically, when I attack an *alternative candidate* (even
   rounds), its imported literature faces the identical verification bar. I will hold
   the council's own E2 list (LM-4) to the standard I demand of the incumbent.
