# Round 0 Calibration — RefuteMeasurement

Persona: metrics that do not measure what they claim — oracle-relativity, survivorship
bias in truncated/killed runs, receipt provenance, partial-credit semantics.
Standing precedent: the 2026-10-18 skel report's 87.5% mean pass rate, produced under
stub-campaign semantics with simulated receipts, collapsing to 1 honest pass in 8 under
real inference (20251025 README §1 Summary). Every quantitative claim I touch gets the
question: *measured under which oracle, over which survivor set, from which receipt
provenance class?*

## Steelman

What the incumbent methodology claims, from my lens — restated in its strongest form,
with anchors. I will hold these exact claims to account in Round 1.

- **C1 (verdict sufficiency).** Achieving $R(s_t)=1.0$ is "a necessary and sufficient
  condition for full compilation, 100% characterization test parity, and full semantic
  rule adherence" — `docs/references/METHODOLOGY.md` §4, "Gated Hierarchical Reward
  Function". The reward is $0.2\,\mathbb{I}_{\text{compile}} + \mathbb{I}_{\text{compile}}
  \cdot [0.6\,\text{PassRate}_{\text{char}} + 0.2\,\text{Score}_{\text{harden}}]$, where
  $\text{PassRate}_{\text{char}}$ is measured over $\mathcal{B}_{CT}^{(v_i)}$ — tests
  synthesized by $\mathcal{A}_{\text{test}}$, an LLM agent — and
  $\text{Score}_{\text{harden}}$ is computed by $\mathcal{A}_{\text{eval}}^{\text{distill}}$,
  a "Lightweight LLM" parity critic of the same model stack. Steelman: the *necessity*
  direction is arithmetically airtight ($R=1.0$ forces all three components to their
  maxima). The methodology also read-isolates $\mathcal{A}_{\text{test}}$ from $c'$
  precisely to prevent test weakening. The claim is oracle-relative by construction and
  the methodology never states otherwise.
- **C2 (acceptance criteria).** A campaign PASSES iff (1) `compilation_success == true`,
  (2) `test_pass_rate == 1.0`, (3) `unimplemented_remaining == 0`, (4) `llm_requests
  > 0`, with wall time in 10 min–2 h — `benchmarks/README.md` §6, §5. These are
  conjunction-gated precisely so a weak single field cannot carry a verdict (toml's
  vacuous pass at `test_pass_rate=1.0` was gated out by criterion 1).
- **C3 (honest measurement).** The receipt artifacts in `benchmarks/results/daemon/`
  are E1 "executed here" evidence; the 20251025 report explicitly demotes the earlier
  87.5% simulated-receipt number and labels only heapq v18 as an honest PASS
  (`docs/report/20251025/README.md` §1 Summary, §2.3 "superseded measurement").
- **C4 (aggregates).** "Aggregate llm_requests across finished runs = 1,858" and "1
  honest pass in 8 real campaigns" (20251025 README §1) are the campaign-level cost and
  success statistics; killed v18 runs are excluded *and flagged* as "⏳ no verdict … not
  an interpretable v18 data point", not silently dropped.
- **C5 (X1 semantics).** Axis X1 of `01-meta-strategy.md` §2 already requires success
  rates to be reported raw *and* under stub-sensitivity-hardened acceptance, jointly
  with oracle strength (X8), and X9 prices reconstructability from persisted state.

## Objections

Round 0: planned attack surfaces only — targets and the falsifier class I intend to
attempt. No objection IDs (those start in Round 1); no substantive attacks argued here.

| # | Target (file + claim) | Attack surface | Falsifier class I will attempt |
|---|---|---|---|
| M-a | C1 sufficiency direction, METHODOLOGY.md §4 | $R=1.0$ is *sufficient* only relative to the strength of $\mathcal{B}_{CT}$ (LLM-generated) and $\text{Score}_{\text{harden}}$ (same-weights LLM parity). "Sufficient for behavioral parity" silently means "sufficient relative to this oracle". | E1: construct (from the toml receipt shape) or execute a deliberately-broken candidate reaching $R$-equivalent 1.0 — the §1-RQ1 falsifier; plus an E4 argument bounding sufficiency by oracle sensitivity. |
| M-b | C2 criterion 2, benchmarks/README.md §6 | `test_pass_rate == 1.0` measured *what suite against what code*? toml: test command exited 0 against an 18-line collapsed stub (`skel-toml.json`, E1). The criterion is vacuous without stub-sensitivity / mutation capacity of the suite. | E1 receipt audit: every finished run's pass rate paired with its candidate state at terminal (stub vs full candidate); mutation-check absence demonstrated. |
| M-c | C2 criterion 4, benchmarks/README.md §6 | `llm_requests > 0` is called "Inference Honesty" but a simulated receipt also has requests > 0 (the 87.5% incident). It is necessary, not sufficient, for provenance. | E1: show no field in `BenchResult` (receipt key list, E1) distinguishes provenance classes; the 10-min lower bound (§5) is the only proxy. |
| M-d | C3 receipt provenance classes | No typed provenance field exists (simulated / real / truncated / superseded). Provenance lives in prose (20251025 README), not in the artifact — unreconstructable by a third party from persisted state alone (X9 B-tier question). | E1: JSON inspection showing no provenance field; attempt to reconstruct the 87.5%→12.5% collapse from surviving artifacts alone. |
| M-e | C4 aggregates, 20251025 README §1 | Survivorship: 3 of 8 v18 runs OOM-killed → "1 PASS in 8" uses a denominator mixing finished, killed, and superseded-binary runs; the 1,858-request sum mixes v16/v17/v18 and old-binary lineages (provenance-mixed cost estimate). | E4: recompute every quoted aggregate under explicit survivor-set definitions; show each headline number is an artifact of one particular denominator choice. |
| M-f | C4 wall-time bound, benchmarks/README.md §5 | The 2 h bound is a *validity filter*, but toml (17.1 h) is reported as FAIL with its cost counted — is a run outside the measurement window evidence about the methodology or about a mis-set timeout? Asymmetric inclusion/exclusion rules distort X2/X3. | E1: classify each receipt's window compliance; E4: recompute pareto axes with/without out-of-window runs. |
| M-g | C5 X1 executable corpus scope | The corpus that makes any X1 rate measurable is 8 single-vertex skel projects, zero SCCs, one honest pass — oracle-relativity plus corpus myopia (K7). Success rates from this corpus are not estimators of anything repository-scale. | E1 scope statement audit; E4: confidence-interval arithmetic on n=8 with 3 non-measured (killed) points. |
| M-h | `compilation_success` semantics drift (20251025 §2.3, VAL-DAEMON-08) | Criterion 1's meaning *changed during the measurement window*; pre-fix receipts (comp=false, pass=1.0) and post-fix receipts are not the same measurement. Any cross-run comparison is confounded. | E1: field-semantics diff across binary lineage; verify the repaired definition is the one actually implemented in the receipts I audit. |
| M-i | Partial-credit semantics, METHODOLOGY.md §4 | $\text{Score}_{\text{harden}} \in [0,1]$ is a same-model LLM judgment folded into the search reward at weight 0.2 — it can steer MCTS toward candidates that *narrate* rule adherence (X8 O3 tier). Not fatal to C1 necessity, but it contaminates the reward that defines "hardened". | E1 receipt probe if fault-classification traces are persisted; E4 argument on O3-tier oracle steering; cross-ref RQ5's recording-vs-preventing rule. |
| M-j | X9 axis measurability (01 §2) | X9's B1 tier promises "transactionally persisted state vector + typed event bus + receipts with provenance" — but the daemon receipts (E1) carry none of the state vector; a third party cannot reconstruct why heapq terminated green-but-unhardened (20251025 §2.5) from the JSON alone. | E1: attempt the reconstruction; measure which X9 sub-claims survive. |

## Evidence & Derivations

- **E1 — receipts.** All five `benchmarks/results/daemon/skel-*.json` read directly.
  Shared shape: `test_pass_rate = 1.0` on *every* receipt including the four FAILs;
  `error` = futile-cap message on all five including the PASS (heapq); receipt keys are
  `campaign_id, code_lines, cocomo_*, compilation_success, engine_wall_time_ms, error,
  llm_requests, macro_states_visited, solver_dispatches, structural_retries,
  test_pass_rate, total_lines, unimplemented_*, wall_time_ms, worker_executions` — no
  provenance field, no oracle-strength field, no suite-identity field. This is the
  empirical base for M-c, M-d, M-h: the verdict semantics live partly outside the
  artifact.
- **E1 — toml vacuous pass.** `skel-toml.json`: `compilation_success = false`,
  `test_pass_rate = 1.0`, 774 requests, terminal futile-cap error; 20251025 §2.3
  documents the candidate collapsed to an 18-line `get_input` stub ("the test command
  still exited 0, so pass 1.0 reflects a test suite that passes against the stub").
  Criterion 1 caught it; criterion 2 alone would have passed it. Direct demonstration
  that criterion 2 does not measure behavioral parity (M-b).
- **E1 — heapq reward/artifact gap.** 20251025 §2.5: artifact passes all four §6
  criteria while the engine's internal reward "never certified the already-green
  candidate" — the internal metric ($R$) and the external verdict (§6) disagree on the
  same run. Whatever C1 claims about $R=1.0$, the only executed run shows the *verdict
  metric* and the *reward metric* diverging (M-a, M-i).
- **E1 — simulated-receipt precedent.** 20251025 §1 Summary: 2026-10-18 report showed
  "100% compile rate, 87.5% mean pass rate, 7/8 passed … under stub-campaign semantics
  with simulated receipts". Provenance classes are not hypothetical; they moved a
  headline number by 7×.
- **E4 — survivorship arithmetic.** Finished v18-or-earlier honest terminals: heapq
  (v18) + 5 finished FAILs = 6 of 8; 3 v18 runs killed with no result JSON. "1 in 8"
  treats unmeasured points as failures in the denominator narrative while excluding
  them from any rate; "1,858 requests" sums across ≥3 binary lineages with different
  semantics for the same fields. Both quoted aggregates are denominator-dependent; I
  will demand the denominator be stated wherever they are reused in `03`/`05`.
- **Evidence tiers of this paper:** all quantitative statements above are E1 (receipts,
  campaign report) or E4 (arithmetic shown); no E5 used.

## Requests to the council

1. **RQ ownership.** I claim primary ownership of **RQ8** (epistemic status of the final
   result: tier stamping, falsification-test statements, survivorship hygiene of every
   aggregate) and **RQ5's recording-vs-preventing distinction** (a receipt that records
   a pass is not evidence that a mechanism prevents a vacuous pass). I claim joint
   ownership (with RefuteTheorist) of **RQ1** on the measurement-validity side of the
   $R=1.0$ claim, and (with RefuteScaler) of **RQ3's metric-comparability precondition**:
   incumbent vs trivial-baseline comparisons are valid only at matched oracle strength
   and matched survivor-set definitions. RefuteTheorist retains RQ1's formal
   soundness core; RefuteCoverage retains RQ5's coverage-matrix completeness.
2. **Axis measurement rights.** X1, X8, X9 measurement semantics are mine to audit in
   `03`/`05`; I will treat any X1 rate quoted without its oracle regime as struck per
   the oracle-relativity rule (01 §4).
3. **Provenance standard request.** I ask the council to adopt, as a Round 1 deliverable
   requirement, that every number quoted in `02`/`03`/`05` carry (oracle regime, survivor
   set, provenance class, lineage) — four fields, or it is evidence theater by default.
4. **Calibration note.** My attacks are aimed at claim-measurement fit, not at the
   architecture's cleverness; where RefuteTheorist attacks whether $R=1.0$ is *sound*,
   I attack whether the *numbers derived from it* mean what the prose says. Overlap on
   M-a is expected and deliberate (dual lens on the central claim); K1's 80%-overlap
   tripwire should be evaluated after diffing all seven papers, not preempted here.
