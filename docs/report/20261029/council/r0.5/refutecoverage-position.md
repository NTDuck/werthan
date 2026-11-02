# Round 0.5 — Protocol Self-Attack: RefuteCoverage

Target: `docs/report/20261029/01-meta-strategy.md` (the critique protocol itself), per
its §7 self-review requirement: "this meta-strategy itself is attacked once (Round 0.5)
by the same personas using these rules — the incumbent target being this document's
protocol."

Lens: RECORDING is not PREVENTING, applied to the protocol's own failure model. A
tripwire that fires after the damage — or whose countermeasure names no owner, no
force, and no consequence — records a failure mode; it does not prevent it. The same
coverage disciplines I applied to METHODOLOGY.md in Round 0 (ownerless failure
classes, double-counted foreclosures, scope waivers) are here applied to 01 §5.4–§7.

Round 0.5 ground rules honored: objection IDs `C-r05-*`; severities claimed (FATAL =
a protocol defect that would invalidate downstream verdicts); grounding = exact
protocol quotes; I read all seven r0 papers under this round's cross-read permission
and cite them only as E1 observations about the council process, not as attacks on
peers.

---

## Steelman

The protocol's strongest self-claim, from my lens, is that it does not merely name
its own failure modes — it *mechanizes* their handling. In its best form:

- **P1. Every known critique-process failure has a named tripwire with a named
  countermeasure.** §7: "The chair audits the tripwires at every round boundary and
  reports tripwire status in the ledger." Eight risks (K1–K8), each with a detection
  tripwire and a countermeasure row — this is more failure-model coverage than any
  comparable critique protocol I have audited.
- **P2. Dispositions are mechanically constrained.** §5.4.4: "Every FATAL/MAJOR
  finding gets exactly one disposition: *fixed* … *accepted-with-rationale* … or
  *rejected-with-counter-evidence* … 'Acknowledged' alone is not a disposition."
  Silent repair is banned with a quote-old/quote-new requirement (§5.4.5), and
  repairs are auditable ("Repairs are quoted verbatim in the ledger", K8).
- **P3. Convergence cannot be premature by construction.** §5.5: "two consecutive
  fatal-free attack rounds, with a minimum of three attack rounds total, AND every
  F-type RQ from §1 has an attempted falsifier" — the minimum-3-rounds rule exists
  precisely because "clean-first-wave convergence is a known artifact of critics
  reading the same text with the same priors" (§5.2).
- **P4. Unclosable claims cannot hide.** §1 adjudication rule: "An F-type RQ whose
  falsifier was never attempted is not closed." Evidence starvation has an honest
  terminal state: "the ledger records the exact experiment that would discriminate,
  as a named falsification test in `08`" (§5.5).
- **P5. The protocol pre-concedes its own auditability.** §7 K8 names chair capture,
  §5.1 bounds the chair ("does not adjudicate technical objections"), §5.2 states
  that a repair introducing a new claim "is a new finding, not a fix", and §7
  mandates this very Round 0.5 self-attack before Round 1.

I accept P1–P5 as the positions of record. My Round 0.5 job is to test whether the
protocol's tripwires *prevent* the failure modes they name, whether its failure
model is *complete*, and whether any protection is *double-counted*.

---

## Objections

### C-r05-1 — No admission gate: ungrounded findings enter the ledger unchallenged. **FATAL**

§5.4.3 mandates grounding ("Every objection must either (a) cite evidence at tier
E1–E3 …, (b) carry a complete E4 derivation, or (c) be explicitly marked
`[E5 hypothesis]`") but **no clause says who checks this, or when**. The chair's
role is bounded: "The chair enforces process (order, tiers, ledger discipline) but
does not adjudicate technical objections" (§5.1). Whether an E4 derivation is
"complete" or an E2 citation carries its regime-transfer argument is precisely a
technical judgment — so grounding verification falls into the gap between the
chair's mandate and the critics' self-certification. The nearest countermeasure, K5,
covers only one narrow slice: "the chair greps for load-bearing claims lacking
E1–E4 stamps at each round boundary" — tier *stamps on numbers*, not grounding
*validity*, and only "at each round boundary", i.e. after the paper is written and
circulated. An ungrounded FATAL (severity self-assigned; see C-r05-10) that reaches
the ledger must then be dispositioned, consuming a repair cycle, and every downstream
verdict in `03`/`05`/`08` that cites the row inherits its defect. The protocol
prevents nothing here; at best the chair strikes after the fact (recording).
Falsifier class: E4 — exhaustive scan of §5 and §7 for any clause assigning
grounding-verification ownership or timing; expected finding: none exists.

### C-r05-2 — Evidence mutation has no re-open rule: the ledger's evidence base can drift out from under final dispositions. **FATAL**

The ledger is append-only, dispositions are terminal ("exactly one disposition",
§5.4.4), and papers are frozen per round. §5.2's regression checking covers only
*repairs*: "the *next* round re-attacks the changed text (regression checking of
repairs is mandatory…)". Nothing re-attacks *changed evidence*. Yet the protocol's
own E1 base demonstrates exactly this hazard, twice, within the critique's own
receipts: (i) `compilation_success` semantics drift — "The definition is now:
`target_build_cmd` … OK and zero unimplemented stubs. The pre-fix finished runs …
report `pass_rate = 1.0` yet `compilation_success = false` — under the old semantics
these fields were conflated" (`docs/report/20251025/README.md` §2.3; F8 in
`docs/report/20261029/02-evidence-base.md` §2); (ii) artifact supersession — "The
earlier 10-24 JSON (wave dispatch deadlock, 14 reqs) is superseded by this run"
(20251025 §1, toml row). A disposition recorded in Round N citing an E1 receipt
whose field semantics or artifact identity later changes is silently stale in every
downstream file. The critique exists to catch measurement drift (RefuteMeasurement's
entire lane) — and the protocol that hosts the critique has no mechanism to prevent
or even flag the same defect in its own ledger. A verdict in `08` resting on such a
row is invalidated. Falsifier class: E4 — scan §5 for any re-open/re-validation
trigger keyed to evidence (vs. text) changes; plus E1 demonstration above.

### C-r05-3 — Convergence-by-fatigue is ownerless: silence converges as fast as merit. **MAJOR**

§5.5's gate counts *fatal-free rounds*, not *engaged rounds*. Nothing obliges a
critic to file any objection at all: two consecutive rounds in which every critic
submits a paper with zero grounded objections satisfy "two consecutive fatal-free
attack rounds … with a minimum of three attack rounds total" exactly as well as two
rounds of genuine, exhaustive attack. K6 polices the *inverse* failure — "critics
manufacture MINOR findings to look busy" — but the symmetric failure (file nothing
to reach convergence) has no tripwire. The min-3-rounds rule (P3) forces more rounds;
it does not make any round substantive. It therefore *records* a round count, and
prevents nothing: the "polite convergence" artifact §5.2 names survives the rule
unchanged. Falsifier class: E4 — construct the degenerate play (all critics submit
empty papers for rounds 1–3; all F-type falsifiers marked "attempted: deferred";
convergence declared) and show no clause in §5.5 or §7 blocks it.

### C-r05-4 — The K2 asymmetry flag has detection but no force. **MAJOR**

K2: "count FATAL findings per target per round; a >3:1 ratio against non-incumbent
candidates in any round is flagged and must be justified in the round summary."
Three coverage defects in one sentence: (i) detection is *post-damage* — the skewed
round has already happened, its findings are already in the append-only ledger, and
their severities already shaped the repair agenda; (ii) "must be justified in the
round summary" names no justifier, no standard of justification, and no consequence
if the justification is weak — a justification that satisfies nobody still clears
the flag by existing; (iii) the flag counts *findings per target* but is blind to
the disposition-space asymmetry documented in C-r05-8. As written, K2 is a
recording instrument presented as a countermeasure. Falsifier class: E4 — parse the
K2 row for the countermeasure column's operative verb; expected finding: no actor is
empowered to nullify or discount a flagged round.

### C-r05-5 — K1's countermeasure presumes an ownership-assignment mechanism that does not exist; Round 0 already broke it. **MAJOR**

K1: "Round 0 papers are diffed: if >80% of claimed falsifiability anchors overlap,
calibration has failed and personas are re-briefed with disjoint RQ ownership."
Two defects. (i) The diff itself is unowned — "papers are diffed" is passive voice:
no actor, no method, no deadline (before Round 1? before Round 2?). This is the
protocol's own recording-vs-preventing smell: the tripwire is real, the response is
a ghost. (ii) "disjoint RQ ownership" is assumed to be *assignable*, but Round 0's
papers (E1, `council/r0/`) show four critics claiming RQ4 — RefuteFeasibility ("I
claim: **RQ4** primary"), RefuteTheorist ("**RQ4** limited to the 'forced vs
aesthetic' verdicts"), RefuteLiterature ("primary ownership of **RQ4**"),
BaselineTypicality ("co-claim **RQ4**") — and three claiming parts of RQ5
(RefuteCoverage outright; RefuteMeasurement "joint ownership … of RQ5's
recording-vs-preventing distinction"; RefuteTheorist "limited to the
*recording-vs-preventing* axis for reward-gated mechanisms"). No clause in §5 or §7
provides an adjudication rule for conflicting claims; by default the chair assigns —
which concentrates exactly the discretion K8 warns about ("the chair's repair drafts
quietly reinterpret findings") into the council's most structural decision. The
overlap tripwire and the ownership mechanism are *two claimed foreclosures of the
same failure (consistency bias) with neither actually preventing it* — a
double-counted foreclosure inside the protocol itself (see C-r05-6).

### C-r05-6 — Double-counted foreclosure: independence rule and K1 diff both claim the consistency-bias foreclosure; neither touches its actual cause. **MAJOR**

§5.4.6: "Critics write their papers without reading each other's papers in the same
round … This preserves genuinely independent lenses." K1 diffs anchors after the
fact. Both claim to prevent correlated criticism. But the dominant source of
correlated priors in this council is the **shared reading corpus**: every critic
reads the same METHODOLOGY.md, the same receipts, the same meta-strategy — §5.4.6
cannot touch that, and K1 fires only on *anchor* overlap, so two critics reaching
the same defect via disjoint anchors (e.g. RefuteTheorist and RefuteMeasurement both
closing in on the R=1.0 sufficiency claim from formal soundness vs. measurement
semantics — visible in their Round 0 papers) pass both tripwires while the council
still fails to cover anything else. Independence of *reading order* is not
independence of *priors*; a post-hoc diff of *anchors* is not a test of
*conclusion* diversity. Neither mechanism prevents the failure they are jointly
credited with.

### C-r05-7 — "Attempted falsifier" has no strength bound: the anti-vacuity gate can itself be passed vacuously. **MAJOR**

§1: "An F-type RQ whose falsifier was never attempted is not closed." §5.5 makes
attempted-falsifiers a convergence AND-clause. But "attempted" is nowhere defined
against a strength floor. RQ3's falsifier requires *running* baselines: "any of
these baselines matching or beating the incumbent … on the executable corpus" —
executable GPU work the protocol never budgets or schedules. If "attempted" admits
"considered and deferred", RQ3 closes without its falsifier ever executing, the
adjudication rule (P4) records the attempt, and convergence proceeds. BaselineTypicality's
Round 0 paper independently reached the same gap from the comparison-class side
("an explicit decision: run at least one baseline … or label all baseline rows
`projection` and accept that RQ3's falsifier was never attempted"). The protocol
double-counts here too: the adjudication rule and the §5.5 AND-clause both claim to
prevent unclosed RQs, but with an unbounded "attempted" they are the same weak gate
counted twice. Falsifier class: E4 — attempt to construct a convergence ledger in
which every F-type RQ row reads "falsifier attempted (analysis only)" and no clause
objects.

### C-r05-8 — Round-parity asymmetry in the disposition space biases the surviving text toward alternatives. **MAJOR**

§5.2: "odd rounds attack the incumbent (and any prior consensus); even rounds attack
the leading alternative candidates." The incumbent's ground truth is read-only
(README governance: "`METHODOLOGY.md` remains read-only (ADR-0005)"), so an odd-round
FATAL against the incumbent can only be dispositioned *accepted-with-rationale*
(scope reduced) or routed to an ADR *proposal* — the defective text itself survives
every round. Even-round attacks hit chair-drafted candidate text, where the
trichotomy's *fixed* disposition is available and the chair repairs it before the
next round. Over N rounds, the corpus of surviving text is systematically drained of
alternative defects and systematically retains incumbent defects — a structural skew
in what the final comparison (`05`'s dominance table) reads against, in the
*opposite* direction of the K2 skew it monitors. K2 counts findings per target; it
does not count *dispositionability* per target. This is a scope waiver in the
protocol's failure model: the failure class "surviving text biased by repair
asymmetry" has no tripwire at all. Falsifier class: E4 — simulate two candidates
with identical defect profiles, one read-only and one chair-repairable; show
divergent surviving-text quality under the protocol with all tripwires green.

### C-r05-9 — The steelman mechanical check has no default when the defender is silent. **MINOR**

§5.4.1: "the defender is given one ledgered line to accept or amend the steelman; an
amended steelman becomes the object of record." No deadline, no rule for silence. If
the defender never responds, does the unverified steelman stand as the object of
record? The rule *records* an offer of verification; it does not *prevent* an
objection from proceeding against a steelman the target never endorsed — the exact
strawman failure the rule exists to foreclose (and the enforcement gap is
compounded by C-r05-1, since no one owns chasing the response).

### C-r05-10 — Severity assignment and motion voting are unowned mechanisms. **MINOR**

Two owners are missing. (i) §5.3 fixes the severity vocabulary ("FATAL … MAJOR …
MINOR … NON-ISSUE") but no clause says who assigns severity to a finding; critics
self-assign (C-r05-1's exposure is proportional to this — an ungrounded finding
graded FATAL gets first claim on the repair agenda). (ii) §3's design-space growth
rule — "additions are voted by ledger" — defines no voting rule, no quorum, no
tiebreak, unlike §3's tie-break ordering which is explicitly fixed "so it cannot be
chosen post-hoc". An unowned vote is a chair decision in disguise (K8 surface).

### C-r05-11 — The docs-only workstream makes F5/F6 foreclosure proposals unverifiable in-protocol, with no label for that state. **MINOR**

README non-goals: "Code changes: this is a documentation/analysis workstream."
The observed taxonomy's INFRA/OPEX classes (F5 OOM kills, F6 transport/restart
failures) can receive only *proposed* foreclosures (ADR text); none can be executed
or receipt-validated inside this engagement. §4's evidence tiers handle this
(hypotheses, E4/E5) and §5.5's evidence-starvation clause gives the right pattern
("the ledger records the exact experiment that would discriminate"), but the
protocol provides no mandatory *unvalidatable-proposal* stamp for coverage claims
whose prevention cannot be demonstrated in-workstream — the same gap my Round 0
attack 10 identified for unexercised machinery, now on the protocol side. Without
the stamp, a proposed F5 foreclosure can sit in `05` looking discharged when nothing
could have tested it here.

---

## Evidence & Derivations

- **Protocol quotes:** all quoted strings are from
  `docs/report/20261029/01-meta-strategy.md` (§1, §3, §5.1–§5.5, §7), read in full
  this round; section anchors given inline per objection. The governing quote for
  the whole paper: §7's tripwire table is titled "Detection tripwire /
  Countermeasure" — a detection column is not a prevention column, and this audit
  is what happens when the two are conflated.
- **E1 (council process):** Round 0 papers under `council/r0/` — RQ4 claimed by four
  personas and RQ5 by three (exact attributions in C-r05-5), demonstrating that the
  K1 countermeasure's "disjoint RQ ownership" premise failed at first application,
  before any diff ran.
- **E1 (evidence mutation precedent):**
  `docs/report/20251025/README.md` §2.3 (`compilation_success` semantics drift,
  fixed same-day, F8) and §1 toml row (10-24 JSON "superseded by this run") — the
  protocol's own evidentiary base exhibits both drift and supersession, and §5 has
  no rule keyed to either. Cross-check: `docs/report/20261029/02-evidence-base.md`
  F8 row.
- **E4 derivations:** each objection carries its falsifier class inline; the two
  degenerate-play constructions (C-r05-3 silent convergence, C-r05-7 analysis-only
  falsifier closure) are complete as stated and can be formalized into ledger test
  cases on request.
- **Severity rationale:** C-r05-1 and C-r05-2 are FATAL because both inject
  unvalidated material into an append-only ledger that `03`/`05`/`08` treat as
  authoritative — the protocol's own definition of FATAL ("invalidates the target's
  claims or feasibility") applies to the verdict pipeline. C-r05-3 through C-r05-8
  are MAJOR (invalidate a specific mechanism of the protocol); C-r05-9 through
  C-r05-11 are MINOR.

## Requests to the council

1. **Admission gate (resolves C-r05-1, C-r05-10i).** Before a finding enters the
   ledger, a designated checker (chair for mechanical checks — tier stamps, quote
   presence; a non-chair critic for grounding *validity* — E4 completeness, E2
   regime-transfer argument) verifies §5.4.3 compliance; ungrounded findings are
   returned, not dispositioned. Severity is assigned by the checker from fixed
   criteria, with critic self-assignment recorded as a proposal.
2. **Evidence re-open rule (resolves C-r05-2).** Any ledger row citing an E1
   artifact carries its artifact digest/identity; if the artifact is superseded or
   its field semantics change, the row is automatically re-opened in the next round
   (the §5.2 regression-checking machinery, currently keyed to repaired *text*,
   extended to cited *evidence*). This is cheap and directly precedented by F8.
3. **Minimum-engagement rule (resolves C-r05-3).** A critic's round paper must
   contain either ≥1 grounded objection or an explicit lane-clearance statement
   ("no defects found in my lane this round, searched for X, Y, Z") — silence
   without a stated search does not count as a fatal-free contribution to
   convergence.
4. **Define "attempted" (resolves C-r05-7).** An F-type falsifier is attempted only
   if (a) executed, or (b) deferred with the exact discriminating experiment named
   per §5.5's evidence-starvation clause and carried into `08`. "Considered" is not
   attempted. This makes the BaselineTypicality run-or-label-projection decision
   mandatory rather than requested.
5. **Adjudicate RQ ownership before Round 1 (resolves C-r05-5).** The chair assigns
   conflicting Round 0 ownership claims by ledgered motion *now* (before K1's diff
   is even runnable), so the assignment is visible and objectable rather than
   implicit. Confirmed overlaps (RQ1 theorist/measurement, RQ5
   coverage/measurement/theorist) should be ratified as *joint* with named
   sub-lanes, not silently merged.
6. **Disposition-parity label (resolves C-r05-8, C-r05-4).** Ledger rows against
   read-only ground truth get stamped `disposition-bound: ADR-proposal` and the
   dominance table in `05` must state, per candidate, how many of its surviving
   claims were never repairable within the protocol — so the frontier comparison
   sees the asymmetry instead of inheriting it. Pair with giving the K2 flag an
   owner and a consequence (e.g., a flagged round triggers a mandatory re-attack of
   the under-attacked target, not a summary justification).
7. **Fix the double-counts explicitly (resolves C-r05-6, C-r05-9, C-r05-10ii,
   C-r05-11).** State in §5 that the independence rule covers reading order only
   and K1's diff is its only check (drop the implied joint foreclosure); add a
   defender-response deadline with a silence-defaults-to-rejection rule for
   steelman amendments; define the §3 motion vote (simple majority of active
   critics, chair votes only to break ties); add the `unvalidatable-in-workstream`
   stamp for coverage claims without executable verification.
