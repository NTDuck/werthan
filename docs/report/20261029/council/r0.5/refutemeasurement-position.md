# Round 0.5 — Protocol Self-Attack: RefuteMeasurement

Target: `docs/report/20261029/01-meta-strategy.md` (the protocol), per 01 §7
self-review requirement. Lens: measurement validity of the protocol's own
measurement rules. Severity vocabulary per 01 §5.3. Cross-read of all six
sibling r0 papers performed (permitted); only my own r0.5 file read among r0.5
papers.

## Steelman

The protocol's measurement apparatus, in its strongest defensible form:

- **P1 (tiered evidence with dual anchoring).** §4: a claim cited as evidence must
  "anchor to a surviving primary artifact AND be classified by tier"; E1 = "measurement
  from this repo's runs/receipts/logs, reproducible from surviving artifacts"; hearsay
  can never be load-bearing; unstamped load-bearing numbers are struck (K5).
- **P2 (oracle-relativity rule).** §4: "All success-rate measurements in this repo are
  oracle-relative … Any X1 claim must state which oracle-strength regime it was measured
  under; raw rates are reported alongside hardened rates, never alone."
- **P3 (self-policing tripwires).** §7: K1 diffs Round 0 papers for ">80% overlap" of
  falsifiability anchors; K5 has "the chair greps for load-bearing claims lacking E1–E4
  stamps at each round boundary"; K7 mandates corpus-scope caveats on any E1
  generalization.
- **P4 (X1 is measurable).** §2 X1: pass rate measured "under (i) raw acceptance, (ii)
  stub-sensitivity-hardened acceptance. Measurable from receipts."
- **P5 (matched-tier dominance).** §3: comparisons "valid only at matched evidence tier
  — a candidate with executed receipts is compared to others' executed numbers, not to
  their projections."
- **P6 (X9 prices honest measurement).** §2 X9: B1 = "transactionally persisted state
  vector + typed event bus + receipts with provenance"; "Measurable against the daemon's
  actual persistence."
- **P7 (RQ6 pins its constants).** §1 RQ6 falsifier: budget arithmetic executed "at the
  *documented* default constants."

I accept all seven as the positions of record. The objections below show P2, P4, P5,
and P6 are jointly inconsistent with the receipt schema the protocol designates as its
own E1 example corpus, and that P3's tripwires are themselves unmeasured.

## Objections

- **M-r05-1 (FATAL) — X1 tier (ii) is named but never defined, and is not measurable
  from receipts.** P4 requires pass rates under "stub-sensitivity-hardened acceptance"
  and claims this is "Measurable from receipts." Grepping the citable corpus
  (`docs/report/20261029/`, `docs/specs/`, `docs/references/`, `docs/decisions/`), the
  phrase appears only in the X1 row itself — no document defines the hardening
  predicate: which suite, which stub-substitution operation, what sensitivity check,
  who executes it, where its result is recorded. The receipt schema cannot carry it
  (E1: the `BenchResult` key list in `benchmarks/results/daemon/skel-*.json` contains
  `test_pass_rate`, `unimplemented_remaining`, etc. — no suite-strength, oracle-strength,
  or stub-sensitivity field; `unimplemented_remaining` counts stubs *in the candidate*,
  not sensitivity *of the suite*). Consequences: (a) X1's "measurable from receipts" is
  false for tier (ii); (b) P2's rule ("raw rates reported alongside hardened rates,
  never alone") mandates a report of a quantity that has no defined measurement — the
  protocol's own anti-theater rule requires producing evidence theater; (c) every
  downstream dominance score on X1 (the primary success axis) would be an unfalsifiable
  projection stamped as measurement. (Resolution: the operational hardened-accept predicate
  is now formally defined in `docs/report/20261029/02-evidence-base.md` §3.1).
- **M-r05-2 (FATAL) — the protocol's E1 definition and its own example corpus are
  structurally survivorship-biased, and no rule forces a survivor-set label.** §4 E1:
  "reproducible from **surviving** artifacts" — and the listed E1 example is "campaign
  JSONs in `benchmarks/results/daemon/`". The 3 OOM-killed v18 campaigns (rbt, html,
  strsim) produced no result JSONs (E1: 20251025 §1 postscript; F5 in the failure
  taxonomy: "no result JSONs exist"). So the E1 example corpus is exactly the survivor
  set of the campaign cohort: every E1 aggregate over it (pass rate, request totals,
  wall-clock distributions feeding X2/X3) over-represents finished runs by
  construction. The protocol contains an oracle-relativity rule (P2) but no
  survivor-set analog: nowhere does it require that an E1 aggregate state its
  denominator (finished / killed / superseded-binary / measured). 20251025 §1 does
  this correctly by hand ("1 PASS / 5 FAIL / 3 killed … not an interpretable v18 data
  point"), but the protocol does not obligate `02`/`03`/`05`/`08` to continue the
  practice — and 02's charter ("every claim re-verified against **surviving** primary
  artifacts") hard-codes the bias into the evidence-base file. Unmeasured campaigns
  are not failures, but treating "surviving" as the sampling frame silently converts
  "no measurement" into "measurement of the survivors."
- **M-r05-3 (FATAL) — "matched evidence tier" is defined, but workload matching is
  not; receipts-vs-receipts comparisons are therefore not valid as permitted.** P5
  prevents receipt-vs-projection laundering but never defines what makes two
  receipts *comparable*: the executable corpus spans 355–2,060 LoC per project
  (E1: 20251025 §1 table), and the configs scale budgets by LoC tier (E1:
  `benchmarks/README.md` §2 — `max_rollouts` 16→96 across T1–T4, inference timeout
  600→1,800 s), so receipts even among finished runs differ in budget regime by ~6×
  rollouts and 3× timeouts. Additionally, receipts carry binary-lineage provenance
  (v16/v17/v18/old-binary) with different field semantics (the
  `compilation_success` drift, 20251025 §2.3) and supersession status (the toml
  old-binary run is "superseded"), none of which is a schema field the matched-tier
  clause can condition on. As written, §3 would admit comparing candidate A's
  T1-tier 497-LoC receipt against candidate B's T2-tier 1,636-LoC receipt as
  "matched" because both are E1 — invalidating any RQ3/RQ6 falsifier attempt built
  on it and any frontier ranking that follows. The dominance rule needs a
  workload-matching predicate (LoC tier, binary lineage, oracle regime, survivor
  class) or the frontier is rank-noise.
- **M-r05-4 (MAJOR) — the tripwires that police measurement are themselves unmeasured:
  no measurement of the measurement.** K5's enforcement is "the chair greps for
  load-bearing claims lacking E1–E4 stamps" — a manual, unlogged action whose output
  is never persisted; a round boundary cannot be audited for whether the grep ran or
  what it found (compare W4-9-class phantom-disposition risk, which the prior
  art's own ledger showed is a real failure mode of unverified manual fixes). K1's
  calibration check — ">80% of claimed falsifiability anchors overlap" — never
  defines the unit of overlap (objections? quotes? RQ claims?), the denominator, or
  who computes it, so calibration success/failure is itself unfalsifiable. A protocol
  whose X9 axis demands third-party reconstructability (P6) ships enforcement
  mechanisms that a third party cannot reconstruct. Repair: tripwire executions must
  produce ledger artifacts (the grep's hit list; the computed overlap figure with
  its unit definition).
- **M-r05-5 (MAJOR) — X9's B1 tier is unattained by the incumbent's actual receipts,
  making X9 comparisons vacuous until the schema changes.** B1 requires "receipts with
  provenance" (P6). E1: the `BenchResult` schema has 18 keys (`campaign_id`,
  `compilation_success`, `test_pass_rate`, `unimplemented_*`, `llm_requests`,
  `solver_dispatches`, `worker_executions`, `macro_states_visited`,
  `structural_retries`, `wall_time_ms`, `engine_wall_time_ms`, `error`, `code_lines`,
  `total_lines`, cocomo fields) — no provenance class, no binary lineage, no
  oracle-strength, no truncation/partial-credit classification. Provenance today
  lives in prose (20251025 README), not in the artifact — precisely the gap that let
  simulated receipts score 87.5% in a prior report. Therefore the incumbent measures
  below B1 (best case B2-: logs plus untyped receipts), and if all candidates also
  measure B2, X9 contributes no strict improvement to any dominance comparison —
  the axis that exists to price "honest measurement" cannot discriminate anything
  until receipts carry provenance fields. Corollary: the heapq paradox (all §6
  criteria green, reward never certified, E1) is reconstructable only from the prose
  report, not from persisted state — X9's own diagnostic question fails on the
  incumbent's flagship receipt.
- **M-r05-6 (MAJOR) — RQ6's falsifier premise "the *documented* default constants" is
  ambiguous between two documented constant sets.** METHODOLOGY.md pins
  $\tau = 0.35, \gamma = 0.15$; the in-tree defaults that actually produced every E1
  receipt are $\tau = 0.50, \gamma = 0.50$, `ucb_c_exp` 1.414, with the divergence
  ratified by ADR-0012 — and `migration.yml` further exposes per-config overrides
  scaled by LoC tier (`max_rollouts` 16–96). "Documented defaults" could mean any of
  the three; an RQ6 arithmetic table built on the wrong set is precisely the
  "13.3 h inside a 2 h bound" class of error the protocol exists to prevent. Any
  RQ6/RQ2 adjudication must stamp which constant set governed the receipts it cites
  (methodology-pinned, in-tree default, or per-config override) — RefuteFeasibility's
  r0 request 5 asks this; I upgrade it to a protocol-level requirement because it is
  a measurement-provenance defect, not a feasibility preference.

## Evidence & Derivations

- **E1 — receipt schema** (all five `benchmarks/results/daemon/skel-*.json`, read
  this round): key list as quoted in M-r05-5; no provenance/oracle-strength/suite-
  strength/lineage field on any receipt. Grounds M-r05-1 and M-r05-5.
- **E1 — survivor set** (20251025 §1 + postscript; F5 in
  `docs/report/20261029/02-evidence-base.md` §2 for the taxonomy row): 8 campaigns,
  6 finished (1 v18 PASS + 5 pre-fix/v17/old-binary FAILs), 3 v18 killed by host OOM
  with no result JSONs; v18 cohort measured = 1 of 4. Grounds M-r05-2.
- **E1 — workload and regime spread** (20251025 §1 LoC column; `benchmarks/README.md`
  §2 LoC-tier table): 355–2,060 LoC; `max_rollouts` 16/32/64/96 by tier; timeouts
  600/900/1,200/1,800 s. Grounds M-r05-3.
- **E1 — constant provenance** (`docs/decisions/0012-ariadne-constant-calibration.md`,
  read in full): METHODOLOGY τ=0.35/γ=0.15 vs in-tree 0.50/0.50, ratified divergence,
  daemon override surface. Grounds M-r05-6.
- **E1 — definitional gap** (grep over `docs/`): "stub-sensitivity"/"hardened" as an
  *acceptance predicate* appears in the citable 20261029 corpus only in 01 §2 X1; the
  operational definitions in the repo are formalized in `docs/report/20261029/02-evidence-base.md`
  §3.1.
- **E4 — survivorship direction:** with 3 of 8 cohort members unmeasured and all
  measurements drawn from the 6 survivors, any estimator of cohort-level
  $\Pr[\text{succ}]$ or $\mathbb{E}[\text{cost}]$ (X2's mixture inputs) is conditional
  on survival; the mixture $\Pr[\text{succ}]\cdot\mathbb{E}[\text{cost}\mid\text{succ}]
  + (1-\Pr[\text{succ}])\cdot\mathbb{E}[\text{cost}\mid\text{fail}]$ is then not
  estimable from E1 receipts at all without a survivor-set model — the protocol's
  headline cost axis inherits the bias in M-r05-2.
- Tier stamps: every quantitative statement above is E1 or E4; no E5; no session-corpus
  citations.

## Requests to the council

1. **RQ ownership (reaffirmed, with r0.5 additions).** RQ8 primary; RQ5's
   recording-vs-preventing distinction; joint RQ1 (measurement-validity side) and RQ3
   (metric-comparability precondition). I add: **the workload-matching predicate of
   M-r05-3 and the survivor-set labeling rule of M-r05-2 are mine to draft** in
   `03`/`05`, since they are measurement-validity instruments, not cost arithmetic
   (RefuteScaler consumes them as inputs).
2. **Repairs I ask the chair to disposition (each is cheap):**
   (a) inline the hardened-acceptance predicate into `01` §2 X1 or `02` — suite,
   substitution operation, executor, receipt field — or demote X1 tier (ii) to
   `[E5 hypothesis]` and strike it from P2's reporting mandate;
   (b) add a survivor-set/provenance labeling rule to §4 (an E1 aggregate without
   denominator + lineage + oracle regime is struck, same as unstamped tiers);
   (c) define the workload-matching predicate for §3's dominance clause (LoC tier,
   binary lineage, constant set, oracle regime, survivor class);
   (d) require tripwire executions to emit ledger artifacts (M-r05-4);
   (e) either extend the receipt schema with provenance/oracle-strength fields or
   stamp the incumbent (and likely all candidates) below B1 and record that X9 is
   non-discriminating at current schema (M-r05-5);
   (f) pin "documented default constants" to a named set + ADR-0012 citation in RQ6
   (M-r05-6).
3. **K1 note for the calibration diff.** Per M-r05-4, the K1 overlap check needs a
   unit before it can run: I propose overlap = shared (target claim, falsifier class)
   pairs across personas' r0 papers, denominator = all r0 planned attack surfaces.
   M-a of my r0 paper deliberately overlaps RefuteTheorist T-r1-1 on the R=1.0
   sufficiency claim (dual lens, declared in my r0 §Requests); the chair should count
   declared dual-lens overlaps separately from accidental ones.
4. **Anti-deflation guard on my own objections.** If the chair repairs X1 tier (ii)
   by demotion rather than definition, that is acceptable only if P2's reporting rule
   is amended in the same pass (a rule requiring an unmeasurable report is itself a
   defect) — per §5.4.5, repairs must not leave the protocol requiring evidence
   theater.
