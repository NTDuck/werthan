# Round 0.5 — Protocol Self-Attack: BaselineTypicality

**Target:** `docs/report/20261029/01-meta-strategy.md` (the protocol), read as a
comparison-class artifact. Cross-round reading used (r0 papers of all six sibling
personas); within-round independence honored — no r0.5 sibling paper has been read.
Objection IDs `B-r05-*`. Severity assigned per §5.3 vocabulary; the chair dispositions
before Round 1.

## Steelman

The protocol's strongest defense of its comparison-class machinery, stated so its
authors would sign it:

- **P1. Axis admission is anti-vibes by construction.** "an axis is admitted only if
  it is (a) measurable from receipts/logs, or (b) tier-orderable with tiers defined
  in this document, or (c) derivable from first principles with arithmetic that a
  reviewer can check. Axes failing all three are dropped as vibes" (§2). Two axes
  were *added* (X8, X9) and one deleted ("coordination overhead") on stated merit —
  the axis set is argued, not inherited.
- **P2. The incumbent is scored under its own rules.** "The incumbent is a
  candidate. The current `METHODOLOGY.md` architecture is scored on the same axes
  with the same evidence rules... The incumbent's executed receipts count for it —
  no asymmetry in whose evidence is real" (§3). The matched-evidence-tier rule
  ("comparisons between candidates are valid only at matched evidence tier") exists
  precisely to stop receipt-laundered dominance claims.
- **P3. Trivial baselines are protected by name.** "Mandatory trivial baselines are
  candidates... If a trivial baseline lands on the frontier, that is a *reportable
  result*, not an embarrassment to be engineered away" (§3); RQ3 (F) names all
  three and its falsifier is "any of these baselines matching or beating the
  incumbent."
- **P4. The claim form is pre-deflated.** "What 'pareto-optimal' does NOT mean
  here. Not global optimality... The delivered claim is always of the form: *no
  candidate in the bounded space survives the audit while dominating the frontier*"
  (§3); and §5.5 provides the honest-exit: non-convergence and evidence starvation
  are both terminal-verdict-eligible.
- **P5. The space grows only on evidence.** "the space grows only through
  evidence-backed motions, never through affection for a clever mechanism" (§3);
  K3 flags "mechanism without rent" by default; K4 blocks frontier membership
  without an X6 estimate.
- **P6. Bias tripwires exist for exactly my worry.** K2 (asymmetry toward
  non-incumbents), K3 (complexity aura), K7 (corpus myopia) — the protocol
  anticipates that the comparison class itself can be gamed.

I accept P1–P6 as the protocol's claims of record. My objections test whether these
safeguards survive contact with their own mechanics.

## Objections

- **B-r05-1 (FATAL). The protocol mandates baselines but schedules nothing that could
  execute them, and its own matched-tier rule then makes RQ3 unanswerable.**
  Grounding: RQ3's falsifier requires baselines "matching or beating the incumbent
  on the axes of §2 ... on the executable corpus" (§1); the dominance rule says a
  receipt-holding candidate "is compared to others' executed numbers, not to their
  projections" (§3). No provision anywhere in `01` (or the README's table) assigns
  an owner, a schedule, a GPU budget, or an implementation path to baselines
  (a)/(b)/(c). The engagement's own non-goals state "Code changes: this is a
  documentation/analysis workstream" (README) — yet baseline (a) and (b) require
  runnable harness code and baseline (c) requires an engine variant. Per the
  adjudication rule, "An F-type RQ whose falsifier was never attempted is not
  closed" (§1): as written, RQ3 is *structurally* unclosable, and every dominance
  row involving a baseline is either invalid (vs incumbent) or projection-vs-
  projection (baseline vs baseline) — i.e., the mandatory baselines are named but
  cannot be scored. The K6 tripwire ("defenders bury fatals under
  accepted-with-rationale dispositions") predicts exactly this failure. Sub-defect:
  "token-matched" (RQ3) has no matching rule — same *token* count is undefined
  against the incumbent's failure-cost asymmetry (107 requests for the heapq PASS
  vs 774 for the toml FAIL, E1), so baseline (a)'s budget is ambiguous by >7×
  before anyone runs anything.
- **B-r05-2 (FATAL). The nine design slots are the incumbent's ontology; the space
  cannot express whole classes of regime-feasible alternatives.** Grounding: a
  candidate occupies "control topology + control locus + search procedure + oracle
  stack + memory substrate + translation-unit granularity + scheduling/budget
  policy + verification routing + honesty gates" (§3). Missing slots: (i) **model
  adaptation** — fine-tuning/LoRA on a translation corpus is regime-feasible on the
  fixed 3090 and could legitimately trade machinery for training compute, but no
  slot carries it; (ii) **human-in-the-loop** — E1 via cross-round citation: the
  implemented engine already contains HITL gates (`engine-and-blackboard.md` §4,
  per RefuteFeasibility r0), yet the protocol's space has no human-oversight
  dimension at all; (iii) **static-analysis/tooling investment** — the fixed regime
  allows buying correctness with deterministic tooling instead of LLM machinery,
  and no slot prices it. A frontier over a space that cannot represent these is
  "pareto-optimal" only in the sense a fish tank is an ocean; the §3 disclaimer
  ("not global optimality") covers space *limits*, not space *gerrymandering*.
  RQ7 is typed (S) scoping, but scoping still must be non-circular: the bounded
  space is the incumbent's self-image rendered as coordinates.
- **B-r05-3 (MAJOR). Axis tier definitions hardcode incumbent mechanism vocabulary,
  so rivals are definition-gamed downward.** Three instances: (i) X5: "V1 = control
  transitions fully enumerated + contract-testable (VAL-* families exist)" — the
  top tier names the incumbent's own contract family as the exemplar; an
  equally-verifiable candidate using a different validation mechanism scores V1
  only by analogy. (ii) X9: "B1 = transactionally persisted state vector + typed
  event bus + receipts with provenance; B2 = logs only" — B1 is Σ(G) + Q_events
  verbatim (Def. 1 of METHODOLOGY.md), and an append-only event log that is fully
  reconstructable is mechanically scored B2 "logs only" despite satisfying X9's
  stated purpose ("Can a third party reconstruct *why* a campaign terminated from
  persisted state alone?"). (iii) X7: "An architecture's recovery tiers should
  bound blast radius" — a prevention-first design with no recovery tiers (e.g.,
  frozen contracts, or full re-translation per module) has no locality classes to
  score and is penalized or N/A by construction. The axis admission rule (P1) is
  fine; the tier *definitions* smuggle mechanism identity in where outcome should
  be measured.
- **B-r05-4 (MAJOR). The matched-evidence-tier dominance rule, combined with the
  receipts asymmetry, makes the frontier bloat rather than decide.** With nine
  axes and tier-matching, incumbent-vs-unrun-rival comparisons are invalid and
  rival-vs-rival comparisons are projection-vs-projection (allowed, "labeled as
  such" §3) — but a projection-vs-projection comparison on five tier-ordered axes
  (X4, X5, X7, X8, X9) is largely undecidable, so most candidate pairs will be
  incomparable and the "pareto frontier" (§3) will contain nearly everything. That
  is the mirror image of the chair's feared foreordained conclusion: not "complex
  machinery never wins" but "nothing is ever comparable," which makes the §3 claim
  form vacuous. Tie-breaks fire only for "evidence-equivalent" pairs (§3) —
  incomparable pairs never reach them. The protocol needs a rule for what
  incomparability *does* to frontier membership (currently: it grants membership
  silently).
- **B-r05-5 (MAJOR). Candidate intake demands attackable, never practitioner-typical.**
  Intake (§3) requires regime-feasibility, implementation-relevant precision, and
  attackability — nothing requires that a candidate be one a practitioner under
  this constraint would plausibly run, or that carries any published deployment
  evidence. Consequence in both directions: the space can fill with council-authored
  novelties that are easy to attack (manufacturing clean rounds) and omit the
  boring defaults practitioners actually run. If the final frontier consists
  mostly of candidates invented by this council for this report, "audit-surviving
  pareto frontier" is a self-refereed result regardless of how honest the audit
  is.
- **B-r05-6 (MINOR). K2's 3:1 asymmetry ratio is uninterpretable under parity-
  alternating targets.** The tripwire: "count FATAL findings per target per round;
  a >3:1 ratio against non-incumbent candidates in any round is flagged" (§7 K2),
  with countermeasure "Alternate attack targets by round parity (§5.2)". But §5.2
  *itself* makes findings per target spike by round parity by design — odd rounds
  attack the incumbent, so incumbent findings mechanically spike on odd rounds and
  non-incumbent findings on even rounds; the ratio as written flags the protocol's
  own schedule. Additionally the denominator is undefined when several
  alternatives are attacked in one round, and raw counts ignore that the incumbent
  has an order of magnitude more claim-text to attack. The tripwire will either
  misfire (eroding trust in tripwires generally) or be waived (eroding K6
  discipline).
- **B-r05-7 (MINOR). The axis set omits outcomes the incumbent's own objective
  names.** METHODOLOGY.md's stated objective includes "(3) adherence to target
  architectural idioms and static security constraints" (METHODOLOGY.md preamble),
  but no axis measures translation *quality* beyond oracle-passing (X1) — a
  methodology producing unidiomatic-but-passing code ties the incumbent on all
  nine axes. Also absent: human-oversight cost (the implementation has HITL gates,
  per B-r05-2's E1 anchor, yet no axis prices the human). Under the protocol's own
  admission rule these are addable on merit; their absence tilts the frontier
  toward "whatever passes weak oracles cheapest."

## Evidence & Derivations

- **Protocol quotes:** all verbatim above with section anchors (§1 RQ3 + adjudication
  rule; §2 axis table X5/X7/X9 + admission rule; §3 candidate definition, intake 1–3,
  dominance clause, trivial-baseline clause, claim-form clause; §5.2 round parity;
  §5.5 starvation route; §7 K2/K6; README non-goals).
- **E4 — baseline execution budget that the protocol never allocates.** From E1
  receipts (heapq: 497-LoC single-vertex module, 107 requests, 3.24 h wall; toml:
  774 requests, 17.1 h; aggregate 1,858 requests over 8 projects), one executed
  baseline (a) over the same corpus is plausibly 8–20 GPU-hours on the fixed 3090
  (sharing serving throughput ~268 tok/s), plus harness implementation work for
  (a)/(b) and an engine variant for (c). This is not prohibited by GPU scarcity —
  it is prohibited by the engagement's docs-only non-goal, which the protocol never
  reconciles with its own RQ3 falsifier. Options available to the chair: (i) scope
  exception with named owner and budget; (ii) E4 receipt-reconstruction ("paper
  execution" of baselines from existing receipts, labeled projection); (iii)
  pre-commit now to the §5.5 starvation verdict with the discriminating experiment
  named in `08`. What is not available is the current state: mandated, unscheduled,
  unclosable.
- **E4 — slot-coverage check for B-r05-2.** Take three concrete regime-feasible
  candidates: (i) "fine-tune 27B on (Python→JS) pairs mined from the corpus
  lineage, then single-agent decode with the existing §6 acceptance gate" —
  expressible only by emptying five of the nine slots; (ii) "human approves
  interface contracts at S4 (HITL), machinery unchanged" — no slot carries the
  human; (iii) "translation-validation oracle (mechanized equivalence checking)
  replaces LLM parity scoring" — expressible, but X5's V1 definition gives it no
  tier without a VAL-* analog. None is vaporeware per intake rule 2; all are
  outside or degraded by the current enumeration.
- **Tier stamps:** textual-protocol findings are E4-grade (checkable derivations from
  protocol text); receipt/budget arithmetic is E1-based E4. No E5 used.
- **Cross-round citations used:** RefuteFeasibility r0 (HITL gates in implemented
  engine; ModuleSolver/`ClusterEngine` seams), RefuteMeasurement r0 (receipt key
  list — no provenance field), RefuteTheorist r0 (T-r1-3: factored-state counter-
  example to $O(m^{|V|)}$). All are r0 papers; no r0.5 sibling read.

## Requests to the council

1. **Chair, before Round 1 (resolves B-r05-1):** pick and ledger one of (i)/(ii)/(iii)
   above for baseline disposition, and define the token-matching rule for baseline
   (a) (recommendation: match on the *conditional mixture* of X2 — matched
   $\Pr[\text{succ}]\cdot\mathbb{E}[\text{cost}\mid\text{succ}]$ budget, not matched
   raw totals — otherwise the 7× asymmetry makes any matching choice a result in
   itself).
2. **Chair, for `04-design-space.md` (resolves B-r05-2/5):** add slots for model
   adaptation, human oversight, and deterministic-tooling investment; add an intake
   stamp `practitioner-typical` vs `novel` per candidate, and require `05` to report
   the novelty share of the frontier; publish the option-set enumeration with its
   Cartesian size and infeasibility exclusions so space size is auditable. Space
   growth still only via evidence-backed motions — the three new slots enter the
   same way.
3. **Chair, for §2 (resolves B-r05-3):** rewrite X5/X7/X9 tier definitions in
   outcome terms with a mechanism-vocabulary ban (X9: replace B1/B2 tiers with a
   direct reconstruction test — "can a third party, given only persisted artifacts,
   state why the campaign terminated"; X5: replace "(VAL-* families exist)" with
   "control transitions fully enumerated and machine-checkable by any contract or
   model-checking artifact the candidate ships"; X7: score *realized* blast radius
   from receipts, and give prevention-first designs an explicit equivalence rule
   rather than an N/A).
4. **Chair, for §3 (resolves B-r05-4):** add an incomparability rule: a pair is
   incomparable only if the disagreement is evidence-starved per §5.5 — in which
   case the pair's discriminating experiment is named in `08` and *neither* gets
   unexamined frontier membership; incomparability must be ledgered per pair, not
   silent.
5. **Chair, for §7 K2 (resolves B-r05-6):** replace the per-round ratio with a
   per-attack-window normalization: FATALs per candidate divided by number of
   attack rounds aimed at that candidate and (optionally) by its claim count in the
   target text; flag only cross-window outliers.
6. **Peers:** B-r05-3's outcome-vs-mechanism rewrite affects everyone's axis scoring;
   RefuteMeasurement's X9 reconstruction test (their M-j) is the natural home for
   the X9 fix — I propose they own the rewrite and I co-sign. RefuteFeasibility: the
   three missing slots in B-r05-2 will need X6 estimates if admitted — flagging now
   so the K4 gate doesn't strangle them at intake.
