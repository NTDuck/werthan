# R0.5 — RefuteLiterature: protocol self-attack (01-meta-strategy.md)

Persona: **RefuteLiterature**. Target: `docs/report/20261029/01-meta-strategy.md` (the
protocol), per `01` §7 self-review requirement. Date: 2026-10-30. All seven r0 papers
cross-read (permitted); only my own r0.5 file written. Objection IDs `L-r05-*`.

## Steelman

The protocol's claims from my lens, strongest form:

- **P-1.** Its evidence system (§4: tiers E1–E5 + session-corpus row, dual anchoring,
  conflict-weighing "E1 beats E2 at equal regime", regime-transfer rule, oracle-relativity
  rule) is engineered against evidence theater — and it *types its own provenance*:
  DRV is honestly labeled a raw Gemini transcript, and future-dated citations are
  pre-classified `session-corpus` (never sole support).
- **P-2.** The named E2 anchors (§4 row: "Harel 1987; UCT and its regret pathologies;
  StateFlow; MAST failure-mode frequencies") are real published works, and the
  protocol characterizes them precisely — e.g. RQ4's "StateFlow's measured +13–28% at
  3–5× lower cost for *transducer FSMs*" is not a strawman but matches the primary
  source's own formalism choice (verified this round, see L-r05-5).
- **P-3.** The prior attempt ("PASIPHAE") is deliberately quarantined: unreferenced,
  slated for deletion, and banned from downstream citation ("nothing downstream of this
  file may cite it", preamble) — the protocol confines its own lineage rather than
  leaning on it.
- **P-4.** X4's tier assignment "by citation to SLM-capability evidence" routes to a
  defined thread (§6 L6) whose consultation produces ledgered output, including
  negative findings recorded as results.
- **P-5.** Round 0.5 itself is the designed mechanism for catching protocol defects;
  this paper exercising it is the mechanism working, not failing.

## Objections

**L-r05-1 (MAJOR). Unannotated session-corpus claims inside the protocol's own text.**
§5.1's persona-justification column asserts factual claims about the prior attempt
("a misattributed cascade result (homogeneous trained triad presented as zero-shot
cascade) survived into the final text of a prior report"; "a 13.3 h mutation-check plan
… survived one full design cycle"; the OOM honesty-gate incident), and the preamble's
eight improvements are comparative claims about PASIPHAE ("prior attempt's requirements
were evidence-anchored but not falsifiability-typed"; "converged plausibly too early").
The comparison target is being deleted, so these claims are **unfalsifiable by
construction within this engagement** — and none carries the `session-corpus`
annotation §4 mandates for exactly this class. Every persona's founding grievance
(including mine, in §5.1's RefuteLiterature row) is currently an unstamped,
unverifiable assertion. Falsifier class: none available (target quarantined) — which is
itself the defect. Disposition options: annotate as `session-corpus` in §5.1/preamble,
or re-derive the affected process parameters from first principles (e.g. the
minimum-three-rounds rule already has an independent justification: "clean-first-wave
convergence is a known artifact of polite critics").

**L-r05-2 (MAJOR). E2 has a verifier-independence gap.** §4 defines E2 as
"Peer-reviewed / published, **verified against primary source this engagement**" — but
no rule states *who* verifies, *what record* the verification must leave, or *whether
the verifying party may be the one leaning on the claim*. 07-bibliography.md plans to
tag "verification method" per source, but §4 never requires a load-bearing claim to
link its E2 stamp to a 07 entry; and K5 (chair greps for missing stamps) audits stamp
*presence*, not verification *quality*. A council member can do a shallow existence
check and self-issue E2 for a claim their own Round-2 objection depends on — the exact
laundering K5 exists to stop, one level up. Falsifying evidence class: any load-bearing
E2 stamp in r1+ lacking a retrievable verification record (URL, version, quote, date,
verifier).

**L-r05-3 (MAJOR). The demotion ratchet is asymmetric between future-authenticity and
future-regime.** `session-corpus` citations "can never be sole support for a verdict"
and must be re-verified — but a published paper measured on frontier models (StateFlow:
GPT-3.5/4-Turbo; MAST: GPT-4o/Claude-3 systems judged by o1) is full E2, permitted as
sole support subject only to a "regime-transfer caveat recorded". In a report whose
deployment regime is fixed at 27B/1×3090, these two risk classes have comparable
evidentiary force; the protocol prices one at zero and the other at a footnote. Falsifier
class: any 05 dominance verdict resting solely on frontier-measured numbers. Proposed
fix: a `regime-mismatched` class — load-bearing citations measured outside the
deployment model class cannot be sole support either, symmetric to `session-corpus`.

**L-r05-4 (MAJOR). X4's tier-assignment rule leans on an unverified, empty thread.**
§2 X4: "Tiers are assigned by citation to SLM-capability evidence (§6 thread SLM)."
Thread L6 (§6) names what to look for ("context degradation curves … coordination tax")
but contains zero sources and no verification plan beyond aspiration. X4 is not
decorative: it *disqualifies candidates* ("an architecture that assumes frontier-model
capabilities … is disqualified or penalized", §2 X4; feasibility rule §3.1). So the
mechanism that can silently decide the pareto frontier is anchored to a literature
thread with no verified entries — T-assignments would be E5 judgments wearing citation
costume. Falsifier class: any T1–T4 assignment in 05 whose cited L6 source does not
exist/verify at that time. Proposed fix: before 05 scoring, L6 must hold ≥1 verified
source per claimed tier boundary, else T-assignments are stamped `[E5]` and may
*penalize* but not *disqualify* (E1 local receipts + E4 first-principles arguments
remain admissible for disqualification).

**L-r05-5 (MINOR, verification report on the protocol's own E2 anchors — executed
this round, live).**
- **StateFlow — PASSES.** arXiv:2403.11322v5, Wu et al., COLM 2024
  (https://arxiv.org/abs/2403.11322). Abstract verbatim: "StateFlow achieves 13% and
  28% higher success rates compared to ReAct in InterCode SQL and ALFWorld benchmark,
  with 5× and 3× less cost respectively." §2: "we base our model on a transducer
  finite-state machine" — the protocol's RQ4 characterization ("transducer FSMs") is
  accurate to the letter, not a strawman. Regime note (mandatory caveat): GPT-3.5-Turbo
  and GPT-4-Turbo (1106), temperature 0, tasks = InterCode SQL/Bash + ALFWorld text
  games — frontier API models, no code2code, no repository-scale multi-module setting.
- **MAST — PASSES with a version hazard.** arXiv:2503.13657, Cemri et al., NeurIPS 2025
  Datasets & Benchmarks (https://arxiv.org/abs/2503.13657). v2 HTML: 14 failure modes,
  3 categories, frequencies FC1 41.77% / FC2 36.94% / FC3 21.30%, per-FM figures
  (e.g. FM-1.3 step repetition 17.14%, FM-2.6 reasoning–action mismatch 13.98%,
  FM-3.2+3.3 verification 13.48%). **Version drift:** the current /abs/ abstract (fetched
  today) describes "MAST-Data, a comprehensive dataset of 1600+ annotated traces …
  across 7 popular MAS frameworks", while v2 (fetched today) is built on 200+ traces /
  150+ in grounded theory. The frequencies are version-sensitive; any council citation
  of "MAST failure-mode frequencies" without a version pin is a laundering hazard of
  exactly K5's class. Regime note: frontier models throughout.
- **UCT — PASSES with scope caveats.** Coquelin & Munos, "Bandit Algorithms for Tree
  Search", UAI 2007 (https://arxiv.org/abs/cs/0703062): "we show that UCT is too
  'optimistic' in some cases, leading to a regret O(exp(exp(D)))" under noisy payoffs.
  Scope: plain UCT, i.i.d. payoff noise, stationary. It does **not** cover softmax-UCB
  hybrids, LLM rollouts, or non-stationary rewards — so the protocol's L1 thread question
  ("softmax-UCB over LLM rollouts at SLM scale") remains open after this anchor; the
  anchor supports "UCT under noise" only.
- **Harel 1987** — not re-verified this round (low authenticity risk; semantic-fidelity
  check queued as my Round-1 A4).

**L-r05-6 (MINOR). The protocol's own improvement claims are pre-verdict inputs.**
The preamble states its eight improvements are "each auditable in the deliverables that
follow" — but improvements 1–5 are comparative against the quarantined prior attempt
(see L-r05-1), so their audit trail is self-referential: the deliverable audited is the
protocol's own assertion that it improved. This does not invalidate the rules (they
stand on first principles) but the audit claim should be scoped to improvements 6–8
(axis quantification, dual anchoring, risk register), which are genuinely
intra-checkable.

## Evidence & Derivations

Verification records (all live fetches, 2026-10-30, verifier: RefuteLiterature):

| Anchor | Primary source | What I checked | Result |
|---|---|---|---|
| StateFlow | arXiv:2403.11322v5 (COLM 2024) | numbers, formalism, model regime | numbers verbatim match; transducer FSM confirmed; GPT-3.5/4-Turbo regime → transfer caveat required |
| MAST | arXiv:2503.13657 (NeurIPS 2025 D&B) | categories, FM frequencies, model regime, versions | v2 frequencies extracted; latest abstract shows MAST-Data 1600+ traces → version pin mandatory; GPT-4o/Claude-3/o1 regime |
| UCT | arXiv:cs/0703062 (UAI 2007) | regret pathology claim, scope | O(exp(exp(D))) under noisy payoffs confirmed; plain UCT, i.i.d. noise — no softmax-UCB/non-stationary coverage |
| L6 / SLM thread | — | existence of any verified entry | none; thread is a plan, not evidence |

Textual grounding: `01` preamble ¶2 ("each auditable in the deliverables that follow";
"nothing downstream of this file may cite it"); §4 tier table rows E2/E3/session-corpus;
§2 X4 "Tiers are assigned by citation to SLM-capability evidence"; §5.1 failure column;
§6 L6 row. Cross-read findings used: RefuteTheorist r0 T-r1-2 and my r0 A9 already
queued the same anchors — this round executes the verification early so Round 1 inherits
stamped results (K5 anti-laundering, applied to ourselves first).

## Requests to the council

1. **Disposition L-r05-1..4 per §5.4.4** (trichotomy with quoted amendments); L-r05-5/6
   need only the version-pin and scoping fixes quoted into 01 or the ledger.
2. **Adopt a version-pinning rule:** every cited primary source in 07-bibliography
   carries {URL, version/date fetched, quote relied on, verifier persona, regime note};
   load-bearing E2 claims must link their stamp to that record (fixes L-r05-2).
3. **Adopt the symmetric `regime-mismatched` rule** (fixes L-r05-3): frontier-measured
   load-bearing citations cannot be sole support for a verdict in the fixed-27B report,
   same as `session-corpus`.
4. **Set an L6 admission gate** (fixes L-r05-4): no T1–T4 assignment in 05 without ≥1
   verified L6 source per boundary claimed; unverified T-assignments are `[E5]`,
   penalize-only.
5. **Annotate or strip** the §5.1/preamble prior-attempt claims as `session-corpus`
   (fixes L-r05-1/6); keep the rules, drop the unverifiable comparative justifications
   where first-principles ones exist.
6. **RQ ownership (unchanged + protocol hygiene):** I retain RQ4 primary, RQ2 secondary,
   RQ8 supporting, and add: any literature/citation defect found in *council-produced*
   files (03/05/07/08) is mine to file in any round, per §5.1's stance.
