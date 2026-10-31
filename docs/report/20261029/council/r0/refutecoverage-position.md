# Round 0 — Calibration Position Paper: RefuteCoverage

Persona: **RefuteCoverage** (01-meta-strategy.md §5.1). Stance-commitment: hunt for
INCOMPLETE failure-mode coverage — double-counted foreclosures (two mechanisms claiming
to prevent the same failure, neither actually preventing it), foreclosures with no
owning mechanism, and scope waivers that quietly drop a real failure class. Standing
distinction for every paper I write: **RECORDING a failure is not PREVENTING it.** A
receipt, log line, or LLM verdict that documents a failure after it has occurred
forecloses nothing; only mechanisms that act *before* or *instead of* the failure
(on constraint, isolation, or mechanized oracle) prevent it. The origin of this
persona is the OOM-kill "covered" by an honesty gate that by definition cannot
execute after a SIGKILL.

Round 0 rule: no substantive attacks. This paper states the incumbent's coverage
claims as I will hold them to account, the evidence classes that could falsify each,
my RQ ownership, and my planned Round 1 attack surfaces (listed, not argued).

---

## Steelman

What the incumbent methodology (`docs/references/METHODOLOGY.md`, ground truth per
ADR-0005; derivations in `METHODOLOGY-DRV.md`) claims, from my lens, is that **every
observed failure class of prior MAS paradigms has exactly one owning foreclosure
mechanism**, organized as:

- **C1. Reward-gated hardening forecloses vacuous passes.** "achieving $R(s_t)=1.0$
  is a necessary and sufficient condition for full compilation, 100% characterization
  test parity, and full semantic rule adherence" (METHODOLOGY.md §4.2, Gated
  Hierarchical Reward Function). A module may enter `M4_Hardened` only at $R=1.0$
  (§4, Micro-FSM, M2→M4 edge; VAL-M2-06, VAL-MACRO-12).
- **C2. Two-stage guards foreclose both malformed and semantically hollow phase
  artifacts.** $g = g_{\text{struct}} \land g_{\text{sem}}$ at every phase boundary
  (METHODOLOGY.md §3.2; Design Decision 4): $g_{\text{struct}}$ is zero-token and
  mechanized; $g_{\text{sem}}$ is a "sharded, read-isolated Adversarial Semantic
  Guard" whose per-phase obligations are enumerated (S1 citations, S2 implicit
  coupling, S3 rule-card completeness/hallucination, S4 over-engineering, S6
  sign-off — §3 table; VAL-MACRO-08).
- **C3. Read-isolation forecloses test weakening.** $\mathcal{A}_{\text{test}}$ is
  "strictly read-isolated from candidate $c'$ to prevent test weakening" (§4.2,
  agent 3; VAL-ATEST-01) — the candidate cannot edit its own oracle.
- **C4. The three-tier regression calculus forecloses uncontained failures.**
  Tier A (intra-module, zero global traffic), Tier B (surgical $O(1)$-hop broadcast
  on a *real* edge $e_{pi}$, arbiter-validated), Tier C (SCC condensation + deep
  history) — "errors are contained to the smallest possible scope" (§5). VAL-MACRO-13,
  VAL-MACRO-17, VAL-FSM-01 pin the legal transition set.
- **C5. The futile-dispatch cap forecloses unbounded burn.** "the engine terminates
  with `ModuleRetryExhausted` instead of cycling park, re-ready, re-dispatch and
  burning unbounded solver rollout budgets" (VAL-MACRO-18; METHODOLOGY.md §4
  budget-exhaustion fallback in Algorithm 2).
- **C6. Transactional persistence forecloses state loss.** $\Sigma(\mathcal{G})$ is
  "transactionally persisted" — crash recovery / deterministic replay (Definition 1;
  VAL-BB-G-02; DRV §1.1 "zero-cost crash recovery").
- **C7. Layered final verification forecloses residual defects.** S6: "100% E2E
  integration test pass rate; zero critical SAST/CVE vulnerabilities" plus
  `ReleaseAuditor` sign-off (§3 table, S6 row).

The steelman's strongest form: the incumbent does not merely *have* mechanisms — it
has a validation-contract catalog (`docs/specs/validation-contracts.md`, VAL-*) that
pins each foreclosure as a testable assertion, and the engine receipts
(`benchmarks/results/daemon/*.json`) plus the failure taxonomy
(`docs/report/20261029/02-evidence-base.md` §2, F1–F8; the campaign narrative in
`docs/report/20251025/README.md`) show the organization *detecting and repairing*
coverage gaps (v17/v18 honest `BudgetExhausted`; VAL-M2-05/VAL-MACRO-18 added).
That is genuine coverage discipline, and my Round 1 objections must engage with the
VAL catalog as it stands, not with a pre-fix strawman.

---

## Objections

Per Round 0 rules, these are **planned attack surfaces only** (targets + falsifier
classes I intend to attempt in Round 1). Objection IDs `C-r1-*` begin in Round 1.

1. **Ownerless failure classes (F5, F6).** The taxonomy's INFRA/OPEX rows — host
   OOM killing three v18 daemons mid-write with no result JSONs (F5; 20251025
   README postscript), transport 500s/socket hangs/restart-cap exhaustion (F6) —
   have **no owning mechanism anywhere in METHODOLOGY.md**. Transactional
   persistence (C6) recovers *engine state after restart*; it cannot execute during
   a session-scope SIGKILL, and no mechanism bounds concurrent-daemon memory,
   checkpoints candidate artifacts incrementally at write boundaries, or owns the
   watchdog/restart policy (that lives in ops, outside the methodology). Planned
   falsifier class: E1 audit of `METHODOLOGY.md` + VAL catalog for any mechanism
   whose trigger condition is evaluable *during* an OOM/hang; expected finding is
   absence, i.e. a coverage matrix row with no owner.
2. **Recording-vs-preventing in the semantic layer.** $\text{Score}_{\text{harden}}$
   is computed by $\mathcal{A}_{\text{eval}}^{\text{distill}}$, a "Lightweight LLM"
   (same model weights, O3 tier on axis X8): it *scores* rule adherence; it does not
   *prevent* a rule violation. Similarly `ReleaseAuditor` "signs off" — a record.
   Planned falsifier class: E1/E4 — the toml receipt shape (18-line stub,
   `test_pass_rate=1.0`, F4) shows the oracle layer certifying a vacuous artifact;
   if the same-model critic can score a stub's rules as satisfied, C1's
   "sufficiency" half is a *recording* claim, not a *preventing* claim.
3. **Double-counted foreclosure: S6 E2E gate vs per-module M4 characterization.**
   Both the $M4$ commit path (characterization suite $\mathcal{B}_{CT}$, C1) and the
   S6 gate (C7) claim to foreclose behavioral-parity failures. But the E2E suites
   audited at S6 are themselves generated artifacts (IntegrationVerifier) with no
   stated provenance constraint or read-isolation from the candidate — the very
   weakness C3 forecloses intra-module is unowned at the inter-module level. Two
   mechanisms claim the parity foreclosure; if both consume model-generated oracles,
   neither prevents it. Falsifier class: E4 derivation + search of METHODOLOGY/VAL
   for any mutation-, sensitivity-, or differential-strength requirement on E2E
   suites (X8 O1/O2 tier evidence).
4. **The foreclosure that manufactures a new failure: futile cap vs the heapq
   paradox (F3).** VAL-MACRO-18 forecloses unbounded burn by *terminating* at cap —
   but E1 shows it terminated a campaign whose workspace already held an
   all-criteria-green candidate (heapq: comp=true, pass=1.0, 0 stubs, 107 requests,
   error = futile-cap message). The cap's trigger (dispatch count) never consults
   the candidate-green condition. Planned falsifier class: E1 receipt + E4 argument
   that a pre-hardening green-check at the cap boundary is a strictly weaker
   requirement than what the methodology already demands at M4.
5. **The parentful twin of F1 survives in the methodology text.** Algorithm 2's
   budget-exhaustion fallback pushes
   `ev_ContractConflict(Parent(v_i), v_i, Δ_blocker(u_best), B_PR)` — i.e. **budget
   exhaustion is remapped to a structural conflict broadcast** whenever a real
   parent edge exists (§5.2 Tier B explicitly includes "when $v_i$ exhausts $B_{\max}$
   rollouts blocked on $e_{pi}$"; VAL-M2-05 encodes it). The engine had to *fix*
   exactly this remap for parentless modules (v18 honest `BudgetExhausted`). The
   methodology legalized the disease for the parentful case. Planned falsifier
   class: E4 + E1 (v17 heapq: budget condition → conflict shape → 194 requests →
   cap death).
6. **Deep-history restoration vs contract invalidation (Tier C).** On
   `S5→S2→H*(S5)`, "every orthogonal region $R(v_i)$ **unaffected by** the
   topological repair is restored … preserving `M4_Hardened` modules without
   re-execution" (Definition 3; VAL-MACRO-10). "Unaffected" is never defined
   against the *new* $v_{\text{SCC}}$ boundary: a hardened module whose contracts
   were rewritten during the S4 re-stub for $v_{\text{SCC}}$ is restored to M4 with
   artifacts built against stale contracts, and no re-verification obligation exists
   in the text or the VAL catalog. Double-count risk: both H* restoration and Tier B
   claim contract-change containment; the intersection (contract changed *by a
   Tier C repair* touching a restored module) may be owned by neither.
   Falsifier class: E4 boundary-case derivation + VAL-catalog gap audit
   (no VAL assertion couples VAL-MACRO-10 restoration to contract deltas).
7. **Warm-start stale statistics (M5).** M5 preserves $\mathcal{T}^{(v_i)}$ and
   $\mathcal{B}_{SH}$ while appending $\Delta_{\text{spec}}$ (VAL-BB-03). Rewards,
   visit counts, and priors in the retained tree were computed under the
   *invalidated* problem model; nothing in §5.2 or VAL-BB-03 requires re-scoring or
   prior invalidation. Tier A (reward backprop) and Tier B (warm start) each claim
   containment for their fault class; the stale-statistics defect falls in both
   mechanisms' shadow. Falsifier class: E4 (UCB means conditioned on superseded
   $\mathcal{B}_{PM}$; no re-evaluation obligation stated) + E5-flagged hypothesis
   that stale priors bias post-regression selection.
8. **Guard independence is asserted, not mechanized.** $g_{\text{sem}}$ critics are
   "Adversarial", `RuleAuditor` is an "independent agent" (§3 table) — but no
   mechanism enforces independence (different weights are impossible in the fixed
   regime; read-isolation is the only independence lever, and it is specified for
   $\mathcal{A}_{\text{test}}$/critics but the *critics'* own competence has no
   falsifier in the VAL catalog beyond VAL-CRITIC-01..03's shape assertions).
   A same-weights critic may be structurally unable to detect the SLM-class
   hallucinations it is assigned to foreclose. Falsifier class: E2 (LLM-as-judge
   self-agreement literature, regime-transfer argument required) + synthetic
   VAL-style probe design.
9. **Scope waiver: single-vertex corpora.** The taxonomy's cross-cutting theme 2:
   "The dominant engine failures are category-shape mismatches" (F1, F2 — ReadySet
   emptiness). METHODOLOGY.md's coverage story is written for multi-module waves;
   the observed deployment corpus is single-vertex. If the Round 1 coverage matrix
   silently evaluates only the multi-module paths, that is exactly the scope waiver
   this persona exists to catch. Falsifier class: E1 corpus audit — evaluate every
   VAL assertion's applicability to $|V|=1$.
10. **Unexercised machinery claiming coverage (bridges to RQ2).** Tier C (SCC
    condensation + H*) has zero SCCs in the corpus; its foreclosure claims
    (C4-Tier-C) rest on VAL assertions (VAL-MACRO-09/10) that are contract tests,
    not recorded inference runs. Per 01-meta-strategy §2 X5 "machinery-verification
    tax": any claim resting on Tier C must be labeled `unexercised` unless synthetic
    VAL runs with recorded evidence exist. Planned falsifier class: check whether
    recorded VAL contract runs exist as artifacts; if not, Tier C coverage claims
    are E4-at-best.

---

## Evidence & Derivations

- **E1 anchors used above:** `benchmarks/results/daemon/skel-heapq.json`
  (`compilation_success: true, test_pass_rate: 1.0, llm_requests: 107,
  solver_dispatches: 3, error: "module source.py exhausted the futile dispatch cap
  after 3 attempts"`) — F3/attack 4; `skel-toml.json` (774 requests, 17.1 h,
  collapsed 18-line stub, pass 1.0) — F4/attack 2; bst/colorsys/mathgen
  (`comp=false, pass=1.0`, futile-cap terminal) — F1/attack 5 context.
  Failure-mode table F1–F8: `docs/report/20261029/02-evidence-base.md` §2
  (standardized citable source for F1–F8; `docs/report/20251025/README.md` is the
  campaign narrative with the same receipts).
- **Text anchors:** METHODOLOGY.md §4.2 (reward), §3 table (guards, S6),
  §5 (Tier A/B/C; Tier B "or when $v_i$ exhausts $B_{\max}$ rollouts blocked on
  $e_{pi}$"), Algorithm 2 fallback, Definition 3 (H*), M5 state. VAL anchors:
  VAL-M2-05/06, VAL-MACRO-08/09/10/12/13/17/18, VAL-ATEST-01, VAL-BB-03,
  VAL-CRITIC-01..03, VAL-BB-G-02.
- **E4 sketch (attack 4):** the cap boundary check needs only
  $\mathbb{I}_{\text{compile}}(c_{\text{workspace}}) \land
  (\text{PassRate}_{\text{char}} = 1.0) \land (\text{stubs} = 0)$ — three quantities
  the harness already computes for the reward (same §6 criteria the benchmark gate
  applies). heapq demonstrates the omission cost ~1 h of post-convergence burn and
  a misrecorded terminal. Full derivation deferred to Round 1 per Round 0 rules.
- **Tier stamps:** all receipt-based statements above are E1; textual-gap findings
  are E4-eligible once written as complete derivations in Round 1; attack 7's
  selection-bias consequence is currently `[E5 hypothesis]` until derived.

## Requests to the council

1. **Adopt the recording-vs-preventing test as a standing rubric row in the RQ5
   coverage matrix** (03-incumbent-critique): for each (failure class, mechanism)
   cell, state whether the mechanism's trigger executes *before* the failure
   (prevents) or only *after/around* it (records). Cells that only record cannot
   close an F1–F8 row.
2. **Standardize the F1–F8 pointer** to `docs/report/20261029/02-evidence-base.md` §2
   (the enumerated table) with `docs/report/20251025/README.md` as the E1 receipt
   narrative, so RQ5's "observed taxonomy" has one citable source.
3. **Chair: request evidence status for the VAL catalog** — are there recorded runs
   of `cargo test --test validation_contract` as artifacts (E1), or only the
   contract text? RQ2 and attack 10 both hinge on this.
4. **Scope-waiver disclosure rule:** any Round 1+ coverage analysis must state the
   DAG-shape regime (multi-vertex vs $|V|=1$) in which each foreclosure is claimed,
   per risk K7.
5. **RQ ownership proposal (for chair ratification):** I claim **RQ5** outright
   (coverage completeness, one-to-one mappings, recording-vs-preventing
   classification); I claim a contributing stake in **RQ1** (the sufficiency half of
   the $R=1.0$ claim is a coverage question), **RQ2** (unexercised machinery cannot
   support foreclosure claims), **RQ4** (the futile cap and the budget→conflict
   remap are "forced vs aesthetic" verdicts with coverage consequences), and
   **RQ8** (residual-risk register must include ownerless failure classes).
