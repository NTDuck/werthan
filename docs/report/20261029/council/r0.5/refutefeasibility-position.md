# Round 0.5 — Protocol Self-Attack: RefuteFeasibility

Persona: implementability and maintenance cost — this round aimed at the protocol
itself (`docs/report/20261029/01-meta-strategy.md`, 420 lines) per §7 "Self-review
requirement". Cross-read of all seven Round 0 papers done (director's instruction;
Round 0.5 independence preserved — this is the only r0.5 paper I read or wrote).
These are substantive objections with IDs and severity proposals, per §5.4; each
carries a quote anchor and an evidence tier. FATAL bar applied as defined (§5.3): a
protocol defect that would invalidate downstream verdicts or make the protocol
unexecutable as written.

## Steelman

The meta-strategy's strongest implementability claims, which I accept before attacking:

- **SP-1 (executability by a third party is a design goal).** §1 header: "It is written
  to be *executed by a third party*: given this file alone, a disinterested executor
  should be able to run the critique, adjudicate objections, and reach the same verdicts."
  The document largely delivers: fixed severity vocabulary (§5.3), fixed tie-break
  ordering (§3), fixed disposition trichotomy (§5.4.4), named tripwires (§7).
- **SP-2 (dual-ledger redundancy is deliberate).** "The append-only ledger lives at both
  `06-audit-ledger.md` and a mirror `council/ledger.md` (mirror regenerated from the
  master by the chair; the master is authoritative)" (§5.3). Master/mirror with declared
  authority is a coherent crash-recovery shape.
- **SP-3 (the protocol self-prices complexity where it matters).** §7 K4 explicitly
  names my persona's mandate; §3 intake rule 2 rejects vaporware ("an implementer could
  build it from the candidate text plus the existing codebase without inventing
  semantics"); X6 is a first-class axis.
- **SP-4 (convergence has a forced stop).** §5.5 "Forced stop … if after eight attack
  rounds fatals persist" bounds the process — the protocol does terminate.

None of the objections below dispute these intents. They attack executability gaps.

## Objections

### F-r05-1 — [FATAL] Crash-recovery for critic sessions is unspecified: the protocol has no checkpoint/resume story for its own executors.

Quote: §5.2 "Each critic writes a position paper per round (§5.3)"; §5.4.6
"independence … papers are submitted before the round's cross-read window"; §5.5
convergence criteria count *ledger findings*, not submitted papers.

Grounding: this defect already fired once in the live council. During my Round 0.5
writing, three consecutive file-write attempts failed on transport drops (provider
stream closed mid-write) before the file landed — the paper you are reading is the
fourth attempt. The protocol's own runtime environment (a council of concurrent
LLM-critic sessions on a shared host) is *exactly* the operating condition where the
20251025 receipts show three of eight daemons were OOM-killed mid-campaign (E1,
20251025 postscript). The protocol specifies what happens to a *round* when a critic's
paper is lost: nothing. Is a missing paper a round-N objection of record? Does the
critic re-enter mid-round having already cross-read (violating §5.4.6) or restart
blind? The convergence counter (§5.5) can reach "two consecutive fatal-free rounds"
because dead critics wrote nothing — silence is indistinguishable from fatal-free.

Fix class (chair dispositions): per-critic round manifests — a submitted paper's
presence + hash recorded in the ledger at submission; a critic who loses session state
resumes from the ledger master (its own prior papers + all ledgered rows), never from
untracked memory; a round is fatal-free only when all seven papers exist or a critic's
absence is itself ledgered.

### F-r05-2 — [FATAL] Intake rule 2 ("implementation-relevant precision") has no defined bar, so candidate intake is unfalsifiable — and the protocol's own flagship attack-class (K4/X6) inherits the gap.

Quote: §3 "Is describable at implementation-relevant precision: an implementer could
build it from the candidate text plus the existing codebase without inventing
semantics. Vaporware candidates … are rejected at intake, not scored low."

The rule names no precision unit. Pseudo-code? Trait signatures against
`global_engine::ModuleSolver` (E1: the seam exists, `werðan/libs/global/engine/src/solver.rs`,
102 LoC)? A `configschema::CampaignConfig` diff (ADR-0009)? Two executors applying
rule 2 will reach different intake verdicts; BaselineTypicality's B-surface-3 already
tripped it in Round 0: variant (c) "portfolio-race-then-commit" is currently a gesture.
And because X6 (§2 axis) is *measured from the candidate description* ("incremental
LoC + config fields + distinct state space size vs the incumbent's implementation"),
an under-specified intake bar poisons every X6 number downstream — my K4 estimates
would be derived from prose, i.e. E5 dressed as measurement.

Fix class: rule 2 must name its precision bar explicitly — e.g. minimum artifact set:
(1) control-flow deltas enumerated against the M0–M5/`MacroStateMachine` typestate
surfaces, (2) new/retired `CampaignConfig` fields with serde defaults, (3) which
existing crate each mechanism lands in. Intake verdicts become ledgered binary checks,
not judgments.

### F-r05-3 — [MAJOR] X6 is underspecified as a measurement instrument: two executors would not compute the same number.

Quote: §2 X6 "Measured: incremental LoC + config fields + distinct state space size vs
the incumbent's implementation, with a complexity tier (S/M/L/rewrite)."

Three undefined operators, each with an E1-demonstrated ambiguity in this repo:

1. **LoC baseline.** First-party vs vendored is undecidable from the axis text.
   Measured (E1, this round): `werðan/libs` + `werðan/bins` contain 451,932 `.rs` src
   lines including `werðan/libs/shared/tools/references/` (419k lines of vendored
   oh-my-pi reference trees); excluding it, ≈24.6k. A factor-of-18 disagreement from
   one unstated convention.
2. **Config-field delta.** Against ADR-0009's schema, does a "field" include section
   structs (`LlmBackendSection`, `CargoHarnessSection` — E1: both are schema-complete
   and daemon-unconsumed per benchmark-harness §7.7) or leaf keys only? The dead-field
   question changes the delta by the entire `ariadne.llm_backend`/`cargo_harness`
   subtree.
3. **Tier definition.** S/M/L/rewrite has no LoC or seam-count thresholds; §3's
   tie-break orders on X6 tier, so an undefined tier directly perturbs the final
   ranking.

Fix class: pin the convention in `01` (or a one-paragraph addendum): LoC = first-party
src lines excluding `references/` (the `.checks/pathqual`-enforced workspace, ADR-0010);
config delta = leaf-key count against `configschema::CampaignConfig` (ADR-0009
canonical parser); tier thresholds named (e.g. S ≤ 500 LoC & ≤ 3 fields, L ≥ 3k or new
crate, rewrite = crosses the `ModuleSolver`/engine trait boundary). Tier stamps on
X6 must cite this rule or be struck per §4's "struck by the chair" precedent.

### F-r05-4 — [MAJOR] K4 is inverted at the protocol's own scale: 7 personas × ≥3 attack rounds × paper+ledger+repair overhead is the protocol's largest unpriced cost, and §7 prices everyone else's complexity but not the protocol's.

Quote: §5.2 "At least three attack rounds even if the first is fatal-free"; §5.1
"Seven standing critics"; §5.4 rules 1–6 attach mandatory overhead per objection
(steelman restatement, defender ledger line, quote anchoring, disposition trichotomy
with quoted amendments).

E1 from Round 0 alone: seven papers total 896 lines for a round whose own rule was
"one-page position paper" (§5.2, line 292) — 2.2× the stated page budget, before any
ledger row, repair interlude, or mirror regeneration. Extrapolated per §5.2–5.5
(≥3 attack rounds + Round 0.5 + repair interludes after each round + chair tripwire
audits + ledger mirroring), the critique process costs more artifacts than the
methodology under audit's own spec tree (tech-spec.md + 4 domain files +
validation-contracts.md, E1 ≈ 6 files). The single-director (chair) is also the report
owner *and* the only repair author *and* the mirror regenerator — a bus-factor-1
coupling of exactly the kind X6 exists to tax. §7's tripwires audit *bias*, not
*throughput*: there is no budget rule that keeps Round N's cost proportional to the
findings it actually produced (Round 0 produced zero findings by design at 896 lines).

Fix class: per-round budget caps or proportionality rule (e.g. paper length bounded by
the previous round's ledgered-finding count; interludes merged when a round produced
only MINOR findings); a declared delegate for mirror regeneration when the chair is
blocked.

### F-r05-5 — [MINOR] §5.3's ledger master/mirror is named but its regeneration trigger, ownership during chair unavailability, and conflict rule are not; the mirror's existence invites exactly the divergence ADR-0009 exists to prevent.

Quote: §5.3 "mirror regenerated from the master by the chair; the master is
authoritative." Unstated: when (per round boundary? per append?), what happens if the
mirror is stale when a critic cites it (Round 1 cross-citation is against papers, but
§5.5 convergence reads the *ledger*), and whether a stale-mirror citation is a
NON-ISSUE or a process defect. ADR-0009's lesson ("field names, defaults, and serde
behavior drift between frontends") applies verbatim to a dual-file ledger.

Fix class: one sentence — mirror regenerated at round close, only; citations resolve
to the master; mirror drift discovered at round N is a ledgered MINOR against the
chair.

### F-r05-6 — [MINOR] The protocol's own file structure fails the hygiene standard it enforces: `01` is a single 420-line file mixing seven concerns (RQs, axes, pareto definition, evidence tiers, council protocol, literature plan, risk register) with no INDEX routing, while the codebase standard (tech-spec.md + domain files + INDEX routing, ADR-0010's `.checks` enforcement) decomposes at ~120 lines per concern.

Quote: §1 "This document is the protocol under which every downstream file in this
report will be produced." Every downstream executor must re-read all 420 lines each
round; cross-referencing inside the file is by §-number only (§5.4.6, §3, §7 K1-K8
referenced across 5 of 7 Round 0 papers — E1: 6/7 papers cite §5 or §7 by number).
This is not fatal — the file is stable and complete — but it is the same
single-file-monolith shape the workspace's own conventions split into domain files,
and the council is the second-largest maintenance surface after the engine.

Fix class: optional; if dispositions require rewrites of §5.2–5.5 anyway, split
council protocol into `01b-council-protocol.md` at that time. No standalone repair
demanded.

## Evidence & Derivations

- **E1 measurements taken this round** (all reproducible from the tree):
  - Paper lengths (Round 0): 896 lines total across 7 papers; range 52 (basinetypicality)–219 (refutecoverage); protocol's own Round 0 rule says "one-page position paper" (01 line 292).
  - `01-meta-strategy.md` = 420 lines; `README.md` = 48; `council/ledger.md` does not exist yet (E1: `find` returned no match) — expected pre-Round-1, noted as the moment F-r05-5's trigger question becomes live.
  - Crate LoC: `werðan/libs` + `werðan/bins` src `.rs` = 451,932 lines including `shared/tools/references/` (vendored, 419k); first-party ≈ 24.6k. Breakdown first-party: global 6,442 / holon 5,759 / shared 4,124 / bins 5,963.
  - Config schema: `configschema/src/lib.rs` = 324 lines, 64 `pub` items; `AriadneSection` carries `llm_backend: LlmBackendSection` and `cargo_harness: CargoHarnessSection` (schema-complete, daemon-unconsumed per `docs/specs/tech-spec/benchmark-harness.md` §7.7); daemon consumes `engine_budget_per_module`, `ucb_exploration_c`, `softmax_temperature`, `reward_weights` (`bench_runner.rs:380-393`).
  - `ModuleSolver` seam: `werðan/libs/global/engine/src/solver.rs` = 102 LoC.
  - Severity vocabulary and disposition rules located at 01:315-323, 335-348; self-review mandate at 01:417-419.
- **E4 (implicit, complete):** F-r05-3's factor-of-18 is arithmetic from the two LoC
  conventions (451,932 / 24,616 ≈ 18.4). F-r05-4's 2.2× is 896 / (7 × ~58) with ~58
  lines as a one-page proxy — stated so the chair can re-derive with any page proxy;
  the conclusion (Round 0 exceeded its own budget) holds for any proxy ≤ 128 lines.
- **No E5 claims.** No session-corpus citations. All quote anchors are to
  `docs/report/20261029/01-meta-strategy.md` (§ numbers per its own headers) or to
  repo files named above.

## Requests to the council

1. **Disposition priority.** F-r05-1 and F-r05-2 are FATAL-class against downstream
   verdicts (RQ3 closure depends on intake rule 2; convergence counting depends on
   crash recovery). I ask the chair to disposition them before Round 1 opens, since
   both change how Round 1 artifacts are produced, not just what they claim.
2. **Adopt F-r05-3's X6 pinning as a §2 axis amendment** (three sentences: LoC
   convention, config-field convention, tier thresholds). Without it, my K4-mandated
   X6 estimates cannot be executor-invariant, and `05`'s dominance tie-break (which
   orders on X6 tier) inherits the ambiguity.
3. **RQ ownership unchanged from my Round 0 paper** (RQ4 primary; RQ6 shared
   implementation-cost half; RQ2 cost-side; RQ3 limited to baseline-(c) construction
   feasibility) — restated here only so the chair's K1 disjointness check can diff all
   seven claims at once. One conflict is already visible in cross-read: RefuteLiterature
   and I both claim RQ4 primary; I cede RQ4's *citation-authentication* half to them
   and retain its *implementation-cost/migration* half (forced-vs-aesthetic verdicts
   are X6-informed).
4. **Scope ruling request.** Findings F-r05-5/F-r05-6 touch `council/ledger.md`
   (not yet existing) and `01`'s structure — both are protocol artifacts, not
   methodology ground truth; confirm repairs to them are in-scope for the docs
   workstream (METHODOLOGY.md remains read-only per ADR-0005; this round touches
   nothing under `docs/references/`).
