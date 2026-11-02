# 01 — Meta-Strategy: How the Critique Will Be Run and How "Pareto-Optimal" Is Defined

**Purpose.** This document is the protocol under which every downstream file in this
report will be produced. It is written to be *executed by a third party*: given this
file alone, a disinterested executor should be able to run the critique, adjudicate
objections, and reach the same verdicts as the council. Where a judgment call is
unavoidable, this file names who makes it and by what rule.

**Scope of this file.** Meta-strategy only. No critique content, no candidate content,
no literature claims beyond naming research threads to consult (§6). Today's date:
2026-10-30.

**Relationship to prior art.** A previous methodology-optimization attempt exists in
session history (a "PASIPHAE" proposal with a 6-wave audit). Its directory is slated
for deletion and is deliberately **not** referenced from this report; nothing
downstream of this file may cite it. This protocol improves on that attempt in eight
specific, checkable ways (each auditable in the deliverables that follow):

1. **More falsifiable research questions** — every RQ carries a stated falsifier or is
   explicitly classified as non-falsifiable scoping (prior attempt's requirements were
   evidence-*anchored* but not falsifiability-*typed*).
2. **More adversarial rounds** — the council runs until two consecutive fatal-free
   rounds, with a mandatory minimum of three attack rounds even if the first appears
   clean (prior attempt converged plausibly too early; clean-first-wave convergence is
   a known artifact of polite critics).
3. **Steelman-before-attack rebuttal rule with a mechanical check** — every objection
   opens with a restatement of the attacked position strong enough that the attacked
   side would sign it.
4. **Position papers, not table rows** — one written artifact per critic per round,
   independently reviewable; ledger rows cite the position paper that produced them
   (prior attempt recorded dispositions without full attack transcripts, making the
   reasoning unauditable).
5. **A baseline-typicality critic** — a standing persona whose sole job is to attack
   *the comparison class itself* (are the alternatives we consider representative, or
   strawmen? are mandatory trivial baselines actually run?).
6. **Dominance computed on explicitly quantified or tier-ordered axes**, not adjectives
   — an axis is either measurable, tier-orderable, or it is dropped.
7. **Higher evidence tiers with dual anchoring** — a claim cited as "evidence" must
   anchor to a surviving primary artifact AND be classified by tier; hearsay tier
   claims can never be load-bearing (§4).
8. **A critique-process risk register with named tripwires** (§7) — self-application of
   the adversarial standard to the meta-strategy itself.

---

## 1. Research questions the critique must answer

Each RQ is typed by falsifiability: **F** = falsifiable (a specific observation would
refute the current best answer), **P** = partially falsifiable (the answer has a
falsifiable core but a framing judgment), **S** = scoping (non-falsifiable; exists to
bound the search, not to be "answered"). F-typed RQs must each state their falsifier.

**RQ1 (F). Do the methodology's formal claims hold under the regime's actual oracle
strength?**
The central formal claim is that achieving $R(s_t)=1.0$ is "necessary and sufficient"
for full compilation, 100% characterization parity, and full semantic rule adherence.
The reward is computed from $\mathcal{A}_{\text{eval}}^{\text{exec}}$ outputs
(compile bit, characterization pass rate) and an LLM parity score. *Falsifier for the
defense of this claim:* a demonstration (executed, or constructed from the campaign
receipts already in `benchmarks/results/daemon/`) that a candidate can reach
$R=1.0$ under suites that a deliberately-broken candidate also passes (the "toml
18-line stub passed at `test_pass_rate=1.0`" receipt shape), i.e. that $R=1.0$ is not
sufficient for behavioral parity. If no such demonstration can be constructed or
executed, the defense stands.

**RQ2 (F). Is the architecture's machinery *exercised* by the deployment corpus, and
where it is not, is the claim retired or synthetically validated?**
Tier C (SCC condensation + Deep History) requires a discovered dependency cycle;
the multi-file project population contains zero SCCs. *Falsifier:* a compiled-LoC or
campaign-log audit showing Tier C executed at least once in real inference, or the
existence of synthetic validation contracts (the `VAL-*` system) exercising it with
recorded runs. Absent both, any paper claim resting on Tier C must be labeled
`unexercised` — and if the machinery's cost is material while its validation is nil,
the critique must quantify that cost/benefit ratio.

**RQ3 (F). Does the orchestration machinery pay for itself against trivial baselines
in this regime?**
The mandatory trivial baselines are: (a) a token-matched single 27B agent given the
same total budget and the same oracles; (b) a linear pipeline with a repair loop and
no statechart, no MCTS, no blackboard; (c) the incumbent at reduced machinery
(e.g. ARIADNE replaced by portfolio-race-then-commit). *Falsifier:* any of these
baselines matching or beating the incumbent on the axes of §2 (not just success
rate — on the pareto frontier) on the executable corpus. Prior local evidence
(1 honest pass in 8 real campaigns; aggregate 1,858 requests with 774 burned by one
pathological module) makes this a live question, not a formality.

**RQ4 (P). Which of the methodology's design decisions are *forced* by evidence in
this regime, and which are inherited aesthetic commitments?**
Each of the five headline decisions (deterministic statechart engine; Harel
statechart formalism; declarative scatter-gather with $D_{\max}=1$; two-stage guards;
encapsulated per-module MCTS) gets a verdict: **forced** (evidence in this regime
rules out alternatives), **supported** (evidence favors but does not force it),
**aesthetic** (plausibility only), or **contradicted**. The falsifiable core: for any
decision marked *forced*, a critic must name the evidence class that would not have
forced it (e.g. StateFlow's measured +13–28% at 3–5× lower cost for *transducer FSMs*
does not force *Harel orthogonality* — the two are different formalisms and the
inference from one to the other is a named leap the critique must either justify or
downgrade).

**RQ5 (F). Is the failure-mode coverage of the methodology complete against the
observed taxonomy, and are the mappings one-to-one?**
Every mechanism in `METHODOLOGY.md` claimed to foreclose an observed failure class
(F1–F8) must map to exactly one owning mechanism; double-counted foreclosures and
foreclosures with no mechanism are defects. *Falsifier:* the coverage matrix in
`03` admits a row where an observed failure has no owning mechanism, or two
mechanisms both claim the same foreclosure with neither actually preventing it
(a receipt *records* a vacuous pass; only oracle-strength mechanisms *prevent* it —
the audit must distinguish recording from preventing, per mechanism).

**RQ6 (P). Is the token/compute economics of the architecture viable at
single-GPU 27B serving throughput?**
Budget arithmetic must be executed, not asserted: per-module dispatch counts ×
agents-per-dispatch × tokens-per-call against measured aggregate throughput, slot
contention under fan-out, and the campaign wall-clock bound. *Falsifier:* an
arithmetic table in `03` where a single workload class (e.g. from-scratch synthesis
of a whole library) exhausts the campaign budget under the incumbent's
configuration, at the *documented* default constants.

**RQ7 (S). What is the bounded design space of alternatives worth considering?**
Scoping question: fixes the axes (§2) and the candidate-generation rule (§3). Its
"answer" is the bounded set, not a claim about the world.

**RQ8 (P). What is the honest epistemic status of the final result?**
Whatever survives must be stated as an *audit-surviving pareto frontier within the
bounded space under the stated cost model* — never as global optimality. `08` must
carry explicit falsification tests (which executed observation would overturn the
ranking) and residual risks with owners. The falsifiable core: the claim must
predict that at least one named, executed test distinguishes the frontier candidates
from the incumbent; if no distinguishing executed test exists, the frontier claim
collapses to "indistinguishable at current evidence," which is itself a reportable
verdict.

**Adjudication rule.** For every RQ, the ledger records: answer, falsifiability type,
evidence tier(s) supporting it, and the named falsifier that was *attempted*. An F-type
RQ whose falsifier was never attempted is not closed.

---

## 2. Evaluation axes (with justification; additions and deletions on merit)

A methodology is compared as a vector over nine axes. Rules for axis admission: an
axis is admitted only if it is (a) measurable from receipts/logs, or (b) tier-orderable
with tiers defined in this document, or (c) derivable from first principles with
arithmetic that a reviewer can check. Axes failing all three are dropped as vibes.
The prior attempt's axis set is a starting point; two axes are added and one is
re-scoped, with justifications inline.

| # | Axis | Justification | Measurement / tier definition |
|---|---|---|---|
| X1 | **Convergence power (task success under hardened oracles)** | The point of the system is translating repositories; under weak oracles success is purchasable by specification gaming, so this axis is only meaningful *jointly* with oracle strength (X8). | Tiered: measured pass rate on the executable corpus under (i) raw acceptance, (ii) stub-sensitivity-hardened acceptance. Measurable from receipts. |
| X2 | **Token & GPU economics** | 27B on one RTX 3090: aggregate throughput ~268 tok/s *shared* across concurrent workers; fan-out divides effective throughput. Failure-cost asymmetry is first-class (observed: 107 requests for a success vs 774 for a pathological failure — >7×). | GPU-seconds via the conditional mixture $\Pr[\text{succ}]\cdot\mathbb{E}[\text{cost}\mid\text{succ}] + (1-\Pr[\text{succ}])\cdot\mathbb{E}[\text{cost}\mid\text{fail}]$. Measurable from receipts. |
| X3 | **Wall-clock latency & parallelism** | Repository-scale jobs have deadline semantics (2 h campaign bound); a methodology that wins on tokens but serializes loses. | Measured wall-clock to terminal; parallel width exploited (determined by DAG structure, not by the methodology). Measurable. |
| X4 | **Robustness to the weak SLM regime** | The entire premise is that a 27B model — with bounded effective context and degraded long-horizon state tracking — must do this work. An architecture that assumes frontier-model capabilities (unbounded context, reliable multi-step planning in one window, honest self-report) is disqualified or penalized. | Tier-ordered: T1 = works with documented evidence at 27B; T2 = plausible at 27B with named mechanism addressing a documented SLM deficiency (context degradation, state hallucination); T3 = requires frontier capability; T4 = assumes it silently. Tiers are assigned by citation to SLM-capability evidence (§6 thread SLM). |
| X5 | **Formal verifiability & auditability** | Deterministic control code can be validated by replay, model-checked, or contract-tested; LLM-driven control cannot. This axis prices the *difference* between Harel statechart, plain FSM, LLM-orchestrated, and fully emergent control — including whether the chosen formalism's extra machinery (orthogonality, deep history) is itself verifiable by anything the repo owns. | Tier-ordered: V1 = control transitions fully enumerated + contract-testable (VAL-* families exist); V2 = deterministic but unenumerated; V3 = mixed; V4 = LLM-controlled transitions. Plus a *machinery-verification tax*: any formal machinery must ship its own validation contract or its claims are labeled `unexercised`. |
| X6 | **Implementation & maintenance complexity** | Unpriced complexity is the classic failure of architecture proposals. Every mechanism costs LoC, config surface (ADR-0009 canonical schema), and future debugging (the fabricated-counterparty bug lived through binary lineages v1–v16). | Measured: incremental LoC + config fields + distinct state space size vs the incumbent's implementation, with a complexity tier (S/M/L/rewrite). Backed by the actual codebase trait boundaries (`ModuleSolver`, harness adapters, ADR ledger). |
| X7 | **Failure containment & regression locality** | Non-monotonic discovery means the *average* campaign dies from a recovery path, not from translation difficulty (5/8 local campaigns died on engine-accounting artifacts). An architecture's recovery tiers should bound blast radius and should never misclassify a budget condition as a structural conflict. | Tier-ordered + measured: locality class of each failure tier (intra-module / neighborhood / global), liveness guarantee presence (does a stuck state always terminalize?), and observed terminal-error classes from receipts. |
| X8 | **Oracle strength & verification independence** | Success is only as real as the oracle. This axis scores the methodology's *hardened* verification capacity: read-isolation between test and code agents, independence of validators (same-weights self-endorsement vs mechanized checks), stub-sensitivity, mutation capacity. **Addition vs prior art:** promoted from a buried requirement to a first-class axis because the local receipts show oracle weakness, not model weakness, is the dominant spec-gaming vector. | Tier-ordered: O1 = mechanized differential + sensitivity + mutation; O2 = sensitivity + read-isolated adversarial; O3 = same-model LLM parity scoring only; O4 = exit-code-only. |
| X9 | **Observability & honest measurement** | Can a third party reconstruct *why* a campaign terminated from persisted state alone? Receipts, provenance classes, truncation semantics, partial-credit classification. Without this, X1–X3 are not estimable at all. | Tier-ordered: B1 = transactionally persisted state vector + typed event bus + receipts with provenance; B2 = logs only; B3 = in-context state (unreconstructable). Measurable against the daemon's actual persistence. |

**Deleted axis.** "Coordination overhead" as a standalone axis (present in the prior
attempt's comparison matrices) is folded into X2 and X3 — it is not separately
measurable in this regime and its prior use mixed token cost with latency.

**Weighting.** Axes are *not* collapsed into a weighted sum (any weighting is itself a
contestable judgment; §3 forbids it). They are used only for dominance (§3) and for
explicit trade-off statements in `05`.

---

## 3. Pareto-optimality: precise definition

**Candidate definition.** A candidate is any *fully specified methodology* (control
topology + control locus + search procedure + oracle stack + memory substrate +
translation-unit granularity + scheduling/budget policy + verification routing +
honesty gates — the same nine load-bearing slots the incumbent occupies) that:

1. **Is regime-feasible**: runs on the fixed regime (27B SLM, single RTX 3090, local
   serving, no frontier API calls) — candidates requiring a second model are
   infeasible by X4/X2 arithmetic unless their capacity claims are met mechanistically.
2. **Is describable at implementation-relevant precision**: an implementer could build
   it from the candidate text plus the existing codebase without inventing semantics.
   Vaporware candidates ("imagine a self-improving orchestrator") are rejected at
   intake, not scored low.
3. **Is attackable**: it can be subjected to the council protocol (§5) — i.e., it makes
   claims that can be falsified.

**Dominance.** Candidate $A$ *dominates* candidate $B$ iff for every axis
$X \in \{X1..X9\}$, $A$ is at least as good as $B$ (measured value equal or better
under the tier orders; strictly better on at least one axis), **where comparisons
between candidates are valid only at matched evidence tier** — a candidate with
executed receipts is compared to others' executed numbers, not to their projections;
projection-vs-projection comparisons are allowed and labeled as such. The
**pareto frontier** is the set of undominated candidates.

**The incumbent is a candidate.** The current `METHODOLOGY.md` architecture is scored
on the same axes with the same evidence rules. The incumbent's executed receipts count
for it — no asymmetry in whose evidence is real.

**Mandatory trivial baselines are candidates.** The single token-matched 27B agent,
the linear pipeline + repair loop, and any incumbent-minus-machinery variant are
intake-eligible by definition (they trivially satisfy 1–3). If a trivial baseline
lands on the frontier, that is a *reportable result*, not an embarrassment to be
engineered away; it directly tests whether the orchestration machinery pays rent
(RQ3).

**Bounded search space.** The space is bounded along the nine design slots with the
option sets enumerated in `04-design-space.md`; a candidate is a point in the
Cartesian product minus regime-infeasible combinations. Claimed novelty outside the
enumeration requires a council motion (any critic may propose an axis-option addition
with evidence; additions are voted by ledger and appended — the space grows only
through evidence-backed motions, never through affection for a clever mechanism).

**What "pareto-optimal" does NOT mean here.** Not global optimality over all
conceivable methodologies; not optimality across regimes (only the fixed 27B regime);
not a weighted-sum maximum (§2). The delivered claim is always of the form: *no
candidate in the bounded space survives the audit while dominating the frontier*.

**Tie-breaks.** If two candidates remain undominated and evidence-equivalent, the
council applies, in order: (1) simpler (lower X6 tier); (2) fewer `unexercised`
machinery claims; (3) the one whose failure modes are better understood from
executed receipts. This ordering is fixed here so it cannot be chosen
post-hoc to save a favorite.

---

## 4. Evidence standards

**Evidence tiers** (a claim's maximum permitted tier is stamped on it; load-bearing
claims must be T1–T3):

| Tier | Class | Examples in this repo | Permitted use |
|---|---|---|---|
| E1 | **Executed here** — measurement from this repo's runs/receipts/logs, reproducible from surviving artifacts | campaign JSONs in `benchmarks/results/daemon/`, 20251025 failure taxonomy, VAL contract test runs | Any claim; highest weight in conflicts |
| E2 | **Peer-reviewed / published, verified against primary source this engagement** | Harel 1987; UCT and its regret pathologies; StateFlow; MAST failure-mode frequencies | Load-bearing claims, with regime-transfer caveat recorded when the paper's regime differs (frontier models, API latency, other languages) |
| E3 | **Published-but-not-primary-verified** (verified via snippet/secondary) | arXiv numbers confirmed only via search results | Supporting claims only; never sole support for a fatal/fatal-refuting verdict |
| E4 | **First-principles derivation** — arithmetic or logical argument with every step checkable | budget arithmetic for rollout counts; $O(m\cdot|V|)$ vs $O(m^{|V|})$ state-space argument | Load-bearing where no measurement exists; must be shown in full, not asserted |
| E5 | **Plausibility / analogy** | "statecharts are used in avionics, therefore…" | Framing only; a verdict may never rest on E5. Every E5 claim in a position paper must be labeled `[E5]` |
| — | **Session-corpus citations** (papers with future-dated or unverifiable IDs from the prior session) | e.g. session-verified arXiv IDs of 2026 | Usable only with the explicit annotation `session-corpus` and never as sole support for a verdict; where a claim matters, the council must re-verify against the primary source or demote it |

**Regime-transfer rule.** Literature evidence measured on frontier models, API
latencies, or other languages transfers only with a stated transfer argument (what is
structurally similar, what differs, in which direction the bias goes). An unargued
transfer is a named objection class ("illegitimate import") that any critic may raise.

**Conflict weighing.** When evidence conflicts: E1 beats E2 at equal regime; E2 beats
E1 only when the E1 measurement is demonstrably regime-misaligned (e.g. corpus too
small to detect the effect — the argument must be quantitative); within a tier,
larger-n and pre-registered designs beat anecdote. Every conflict resolution is
recorded in the ledger with both sides stated.

**Claims lacking evidence.** Treated as hypotheses, not facts: they may be argued
(E4) or flagged (E5) but may not be asserted in a verdict. The recurring anti-pattern
to guard against — demonstrated locally by the simulated-receipt 87.5% pass rate
collapsing to 12.5% under real inference — is *evidence theater*: numbers that look
like measurements but are projections. Any such number in a position paper must carry
its tier or be struck by the chair.

**Oracle-relativity rule.** All success-rate measurements in this repo are
oracle-relative (the benchmark's native suites are weak). Any X1 claim must state
which oracle-strength regime it was measured under; raw rates are reported alongside
hardened rates, never alone.

---

## 5. Council protocol

### 5.1 Critic personas (distinct adversarial stances)

Seven standing critics; each persona's stance is a *commitment about where attacks
come from*, so the council cannot all miss the same class of defect. Personas are
adversarial toward the *claim under review*, never toward each other's persons.

| Persona | Stance (what they are hunting for) | Failure of the prior attempt they exist to prevent |
|---|---|---|
| **RefuteTheorist** | Soundness of formal claims: search theory (UCB/softmax behavior under non-stationary, noisy rewards), reward-function gating claims, state-space complexity claims, liveness/deadlock properties of the control topology. | The $R=1.0$ necessity-and-sufficiency claim survived until someone actually ran the receipt arithmetic. |
| **RefuteScaler** | Budget and scalability arithmetic: token counts, GPU-seconds, fan-out contention, state-vector growth in $|V|$, worst-case single-module pathologies. | A 13.3 h mutation-check plan inside a 2 h bound survived one full design cycle before arithmetic was applied. |
| **RefuteFeasibility** | Implementability against the actual codebase: trait boundaries, config schema, harness adapters, migration cost, dead config after changes. | Proposals were scored as if implementation were free; an M-tier cost is a real cost at single-maintainer scale. |
| **RefuteCoverage** | Completeness of failure-mode coverage; double-counted foreclosures; scope waivers that quietly drop a real failure class. | OOM-kill was "covered" by an honesty gate that by definition cannot execute after a SIGKILL. |
| **RefuteLiterature** | Citation authenticity, regime transfer, overclaiming; every load-bearing citation verified against the primary source during the round, not recalled from memory. | A misattributed cascade result (homogeneous trained triad presented as zero-shot cascade) survived into the final text of a prior report. |
| **RefuteMeasurement** | Whether the metrics measure what they claim: oracle-relativity, survivorship bias in truncated runs, provenance of receipts, partial-credit semantics. | Simulated receipts inflating pass rates from 12.5% to 87.5%. |
| **BaselineTypicality** | Attacks the *comparison class*: are the alternatives considered representative? are trivial baselines actually scored or just named? is the incumbent being strawmanned — or its machinery being strawmanned by its own critics? | Prior comparison tables asserted paradigm weaknesses with no measurement and no trivial baseline ever actually run. |

**Chair** (not a persona; a role): the report owner. The chair enforces process
(order, tiers, ledger discipline) but does not adjudicate technical objections —
objections are resolved by disposition against evidence, and any chair decision on a
contested disposition is itself ledgered as an objectable move.

### 5.2 Round structure

Rounds are numbered from 0.

- **Round 0 (calibration).** Each critic produces a one-page position paper: what the
  current methodology claims, what evidence class could falsify each claim, and which
  RQ they claim. No attacks on substance yet — this round exists to expose each
  critic's priors so later rounds cannot hide behind them.
- **Rounds 1..N (attack rounds).** Target alternates: odd rounds attack the
  incumbent (and any prior consensus); even rounds attack the leading alternative
  candidates. Each critic writes a position paper per round (§5.3) containing only
  objections grounded per §5.4.
- **Repair interludes.** After each round, the chair produces a repair draft of
  affected deliverables; the *next* round re-attacks the changed text (regression
  checking of repairs is mandatory: a repair that introduces a new objectionable claim
  is a new finding, not a fix).
- **Minimum rounds.** At least three attack rounds even if the first is fatal-free —
  clean-first-wave convergence is a known artifact of critics reading the same text
  with the same priors.

### 5.3 Written artifact conventions (`docs/report/20261029/council/`)

- Path: `council/r{N}/{persona}-position.md` — one position paper per critic per
  round. No round may be summarized into a table without the underlying papers.
- Position paper skeleton (mandatory headings):
  `## Steelman`, `## Objections`, `## Evidence & Derivations`, `## Requests to the
  council`. Objections are individually numbered `{persona-initial}-r{N}-{seq}`
  (e.g. `T-r2-3`), so ledger rows and later rounds can reference them stably.
- The append-only ledger lives at both `06-audit-ledger.md` and a mirror
  `council/ledger.md` (mirror regenerated from the master by the chair; the master is
  authoritative). Ledger row: finding id, round, target, severity, evidence tier,
  disposition, disposition rationale with anchor.
- Severity vocabulary (fixed): **FATAL** (unresolved, invalidates the target's claims
  or feasibility), **MAJOR** (invalidates a specific mechanism or number), **MINOR**
 (wording/consistency), **NON-ISSUE** (the objection fails; the *rejection rationale*
  is itself ledgered and citable).

### 5.4 Mandatory rebuttal rules

1. **Steelman before attack.** Every objection opens with a restatement of the
   attacked position, in its strongest form, including its best supporting evidence.
   Mechanical check: the restatement must be specific enough that a defender of the
   position would accept it as their own claim (the defender is given one ledgered
   line to accept or amend the steelman; an amended steelman becomes the object of
   record).
2. **No strawmen, no ghosts.** An objection may not attack a claim the target does
   not make. Citing the exact sentence (file + quote) of the attacked claim is
   mandatory; attacking an implication the target explicitly disclaims is a
   NON-ISSUE with a note.
3. **Grounding rule.** Every objection must either (a) cite evidence at tier E1–E3
   (with the transfer argument if E2/E3 from another regime), or (b) carry a
   complete E4 derivation, or (c) be explicitly marked `[E5 hypothesis]` — in which
   case it cannot be scored above MINOR and cannot contribute to fatality.
4. **Disposition trichotomy.** Every FATAL/MAJOR finding gets exactly one
   disposition: *fixed* (text amended; the amendment is quoted in the ledger),
   *accepted-with-rationale* (finding stands; scope/claim reduced; the reduced claim
   is quoted), or *rejected-with-counter-evidence* (counter-evidence tier stated and
   anchored). "Acknowledged" alone is not a disposition.
5. **No silent repair.** A repair that changes a claim must quote the old claim and
   the new claim in the ledger. Repairs that weaken a claim must not weaken it below
   what the evidence supports in the *other* direction either (anti-deflation check:
   critics may object that a repair over-concedes).
6. **Independence.** Critics write their papers without reading each other's papers
   in the same round (papers are submitted before the round's cross-read window);
   cross-citation is allowed in the *next* round only. This preserves genuinely
   independent lenses.

### 5.5 Mechanical convergence and stop criteria

- A round is **fatal-free** iff the round's ledger contains zero findings in
  FATAL severity with disposition other than *rejected-with-counter-evidence* (a
  rejected FATAL does not count as fatal for convergence purposes, but its rejection
  rationale must itself survive one subsequent round unchallenged).
- **Convergence**: two consecutive fatal-free attack rounds, with a minimum of three
  attack rounds total, AND every F-type RQ from §1 has an attempted falsifier, AND
  every surviving claim's evidence tier is stamped.
- **Forced stop (failure mode of the protocol itself)**: if after eight attack rounds
  fatals persist, the council declares the space *unconverged* and `08-final-verdict.md`
  must say so, listing the irreducible disagreements with both sides' best evidence.
  An honest non-convergence verdict is an acceptable terminal state; a manufactured
  one is not.
- **Non-convergence due to evidence starvation**: if a disagreement persists because
  no available evidence discriminates the options, the ledger records the exact
  experiment that would discriminate, as a named falsification test in `08`.

---

## 6. Literature research plan (per axis)

Threads are consulted to *arm specific axes and RQs*, not browsed generically. Each
thread lists what is being looked for and what would count as a finding. Session-corpus
citations must be re-verified at the primary source during the round that leans on
them; anything unverifiable gets demoted per §4.

| Thread | Arms | What to look for |
|---|---|---|
| **L1. LLM-MCTS & tree-search agents** (MCTS selection under noisy/non-stationary LLM rewards; process-reward-guided search; tree vs linear repair) | X1, RQ1, RQ4 (the encapsulated-ARIADNE decision) | Whether softmax-UCB over LLM rollouts has measured benefit at SLM scale; known pathologies (UCT under noise, premature greedy convergence); what the alternative (portfolio race, best-of-N with verifier selection) measures. |
| **L2. Statechart & formal methods** (Harel statecharts, model checking, translation validation, refinement) | X5, RQ2, RQ4 | Evidence on whether statechart machinery (orthogonality, deep history) is *validated* in practice, and at what cost; whether flat FSMs or transducers suffice at this scale; translation-validation as a cheaper oracle than LLM parity scoring. |
| **L3. Blackboard & coordination architectures** (Hearsay-II lineage; modern LLM blackboards; event-driven control) | X7, RQ4, RQ7 | Measured comparisons of blackboard vs pipeline vs hierarchy; load-balancing and termination guarantees; whether push-based ephemeral workers have known failure modes (livelock, event storms). |
| **L4. DAG scheduling & decomposition granularity** (topological scheduling, SCC condensation, sub-file block decomposition, assembly contracts) | X3, X7, RQ5, RQ7 | Evidence on translation-unit granularity effects (whole-file vs block vs fragment); scheduling with resource contention; liveness of wave schedulers under invalidation cascades. |
| **L5. Multi-agent verification & oracle design** (test generation quality, differential testing, mutation analysis, LLM-as-judge biases, witness generation) | X1, X8, RQ1, RQ5 | Measured rates of LLM-generated test weakness; read-isolation and independence effects; mechanized oracles that need no model capacity (the X8 O1 tier). |
| **L6. SLM capability studies** (context degradation curves, long-horizon state tracking, self-report reliability, small-model agent coordination tax) | X4, RQ3, RQ6 | Quantified SLM deficits that any candidate must design around; evidence on whether MAS machinery helps or hurts small models specifically (the coordination-tax literature). |
| **L7. Test-time compute economics** (compute-optimal scaling, budget allocation, failure-cost asymmetry, speculative execution) | X2, X3, RQ6 | Compute-optimal allocation evidence for fixed budgets; whether adaptive budgeting beats uniform under failure-cost asymmetry. |
| **L8. Repository-scale empirical migration studies** (industrial migrations, benchmark validity, honest-measurement practice) | X9, RQ3, RQ8 | What actually failed at scale in industry; benchmark validity threats (solution leakage, weak-test false passes); measurement practices worth importing. |

Per-thread output: an annotated source list in `07-bibliography.md` with verification
method and regime notes, plus any executed arithmetic (E4) the thread's evidence
forces. Negative findings are recorded ("thread consulted, no regime-transferable
evidence found") — absence of evidence is a ledgered result, not a silence.

---

## 7. Risk register for the critique process itself

The meta-strategy is itself subject to hostile review; these are its known failure
modes, each with a detection tripwire and a countermeasure. The chair audits the
tripwires at every round boundary and reports tripwire status in the ledger.

| # | Risk | Detection tripwire | Countermeasure |
|---|---|---|---|
| K1 | **Consistency bias** — critics converge because they read the same repo, same receipts, same framing; the council certifies rather than challenges. | Round 0 papers are diffed: if >80% of claimed falsifiability anchors overlap, calibration has failed and personas are re-briefed with disjoint RQ ownership. | Disjoint RQ ownership per persona; independence rule §5.4.6; minimum three attack rounds. |
| K2 | **Sunk-cost attachment to prior proposals** — the PASIPHAE lineage (or any earlier design in session memory) anchors the critique; alternatives get attacked harder than the incumbent. | Ledger asymmetry check: count FATAL findings per target per round; a >3:1 ratio against non-incumbent candidates in any round is flagged and must be justified in the round summary. | Alternate attack targets by round parity (§5.2); incumbent scored with identical evidence rules; baseline personas review disposition balance. |
| K3 | **Overweighting flashy architectures** — MCTS, statecharts, and holonic vocabulary create an aura of rigor; complexity is mistaken for capability. | Every mechanism must show its RQ and axis effect at a specific evidence tier; a mechanism with no measured or derived effect on any axis is flagged "mechanism without rent" and is a MINOR finding against its host candidate by default. | The mandatory trivial baselines (§3); complexity axis X6; tie-break ordering prefers simplicity (§3). |
| K4 | **Underweighting implementation cost** — proposals scored as designs while the repo must actually build them at single-maintainer scale. | RefuteFeasibility must produce LoC/config estimates for every frontier candidate; a candidate lacking an X6 estimate cannot reach the frontier. | X6 tier required for frontier membership; trait-boundary anchoring in the actual codebase. |
| K5 | **Evidence-tier laundering** — E5 plausibility quietly promoted to E2 by repetition across papers. | Ledger stamps tier per claim; the chair greps for load-bearing claims lacking E1–E4 stamps at each round boundary. | §4 stamps; chair strikes unstamped load-bearing numbers. |
| K6 | **Protocol gaming** — critics manufacture MINOR findings to look busy, or defenders bury fatals under accepted-with-rationale dispositions that reduce claims to vacuity. | Disposition audit: an accepted-with-rationale that leaves a claim unfalsifiable (no remaining test) is itself a MAJOR finding. | §5.4.4 trichotomy with quoted amendments; anti-deflation check §5.4.5. |
| K7 | **Corpus myopia** — overfitting the critique to the 8-project skel corpus and its idiosyncrasies (single-vertex DAGs, zero SCCs), generalizing from n=1 honest pass. | Any E1-based generalization must state corpus scope explicitly; sample-size caveats are mandatory in `03` and `05`. | Regime-transfer and scope rules (§4); RQ8's residual-risk register; synthetic validation (RQ2) as the alternative evidence route. |
| K8 | **Chair capture** — the chair's repair drafts quietly reinterpret findings. | Repairs are quoted verbatim in the ledger (§5.4.4); any critic may object to a repair's wording in the next round. | Ledger quoting rules; objection of repairs is in-scope for every persona. |

**Self-review requirement.** Before the council begins, this meta-strategy itself is
attacked once (Round 0.5) by the same personas using these rules — the incumbent
target being this document's protocol. Findings against the protocol are fixed or
dispositioned in the same manner, and the fixes are recorded in `06` before Round 1.
