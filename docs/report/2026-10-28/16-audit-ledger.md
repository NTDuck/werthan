# 16 — Audit Ledger (Waves 1–2)

Adversarial audit of the PASIPHAE front-runner. Wave 1: six independent REFUTE lenses (theory, scale, coverage, literature, feasibility, measurement) — every lens ran as an independent read-only auditor with line-cited findings. Dispositions: **fixed** (design text amended), **accepted-with-rationale**, or **rejected-with-counter-evidence**. A finding is fatal if unresolved it invalidates the proposal's claims or feasibility.

## Wave 1 — REFUTE pool

### RefuteThe (theory & search soundness)

| # | Finding | Severity | Disposition |
|---|---|---|---|
| T-F1 | Successive elimination invalid for non-stationary arms (repair shifts reward distributions; principled starters eliminated pre-repair) | FATAL | **Fixed**: race arms scored on depth-1 *repair* rollouts, not zero-shot drafts; race restarts on oracle upgrades; stall escape `ev_StrategyStalled` demotes winner, revives runner-up (≤2). `14` §2. |
| T-F2 | Cited search literature (Snell, Tolpin & Shimony, Coquelin & Munos) does not establish race-then-commit for non-stationary arms; "bounded-sample guarantees" overclaimed | VALID | **Fixed**: all regret-bound language struck; race is an empirical compute-budget heuristic (F7 evidence + Snell allocation), claim downgraded in `12` A3, `13` C4.1, `14` §2. |
| T-F3 | Epistemic-vs-stochastic conflation: candidate sampling is genuinely stochastic; stronger oracles make rewards sparser, so cutting exploration starves search | FATAL | **Fixed**: claims disentangled in `14` §2 ("why not softmax" ¶); guaranteed exploration floor $N_{\min}$ per difficulty tier adopted; oracle gating decoupled from exploration budget (`12` A3). |
| T-F4 | Layered oracle early-commit: L0/L1-lucky strategies beat semantically-sound-but-syntactically-weak ones; commit purges alternatives irrecoverably | VALID | **Fixed**: L2 differential probes run on a bounded subset *during* the race; stall-triggered demote/revive from checkpoint (`12` A4, `13` C4.2, `14` §2). |
| T-F5 | `ExhaustedStrategies` totality hole: park-with-satisfied-parents ⇒ infinite re-dispatch loop; exclude-from-ReadySet ⇒ liveness false positive | FATAL | **Fixed**: defined micro-FSM semantics — futile counter increments; at cap terminal `M_Failed(StrategyExhaustion)` + `ev_ModuleFailed(v_i)`; never parks with satisfied parents, never idles in dispatchable set (`14` §2 totality rules). |
| T-F6 | Peer slices without a directed edge cannot broadcast Tier-B conflicts (R3 forbids), yet shared-scope conflicts are real; assembly ownership undefined | VALID | **Fixed**: decomposition emits explicit directed assembly edges; file-level composite container owns assembly; deterministic blame attribution invalidates only the blamed slice (`11` R7, `12` A6, `14` §1). |
| T-F7 | Progress-gated budgets: unquantified ε-gain resets permit infinite futile-cap evasion | VALID | **Fixed**: progress quantum $\epsilon_{\text{prog}} = 0.05$ (or oracle-stage advance), ≤3 resets/vertex, wall-clock sub-budget overrides (`11` R5, `12` A3×A7 note, `10` §5b, `14` §2). |
| T-F8 | Attack "racing abandons MCTS proof-tree reuse" | NON-ISSUE | **Rejected**: A3 retains MCTS machinery within the repair phase; racing replaces only root-level strategy spending (`12` A3). |

### RefuteScale (scalability arithmetic)

| # | Finding | Severity | Disposition |
|---|---|---|---|
| S-F1 | Per-candidate mutation scoring arithmetic: 1,600 candidates × 30 mutants × 1 s ≈ 13.3 h ≈ 6.7× the 2 h bound; no budget policy specified | FATAL | **Fixed**: mutation gates the *suite* once per module (≤30-mutant sample, 10 s/mutant, early-abort at first survivor) inside the module sub-budget; stub-sensitivity (≈1.4% of budget) is the mandatory layer; unaffordable campaigns receipt-label `oracle_strength: stub_sensitivity_only` (`12` A4, `13` C4.2, `14` §3). |
| S-F2 | Cascade routing self-contradiction: R8 forbids same-model self-endorsement but no second model fits a 24 GB card; token-capacity arithmetic shows 100×16×1.2K ≈ 1.93 M tokens ≈ the entire 2 h output budget | FATAL | **Fixed**: A8/R11 redefined as verification-capacity concentration (mechanized model-free checkers + procedural read-isolated validation + `self_endorsed` flag barred from solo certification); remote frontier out of regime; token-capacity corollary makes R10/A6 preconditions of feasibility (`11` R11, `12` A8, `13` C4.4). |
| S-F3 | Sub-file slice vertices: 337 slices/8 skel projects, per-slice dispatches multiply wall-clock; shared-scope siblings cascade in $O(S^2)$ | FATAL | **Fixed**: composite batching (strongly-coupled slice sets become one vertex, ≤ ~2× file count), directed assembly edges, file-level container with blame attribution (`12` A6, `11` R7). |
| S-F4 | ε-gain runaway arithmetic: 5 resets × 160 rollouts ≈ 50 min ≈ 41.5% of campaign bound for one pathological module | FATAL | **Fixed**: progress quantum + ≤3 resets + derived wall-clock sub-budget (toml-class module capped ~40% of $W$) (`10` §5b, `11` R5). |
| S-F5 | Stub-sensitivity check feasibility: 100 suites × 1 run ≈ 1.67 min ≈ 1.4% of budget | NON-ISSUE | **Accepted**: L1-lite is cheap; the mandatory layer. |

### RefuteCover (failure-coverage completeness — 28-row mapping table)

| # | Finding | Severity | Disposition |
|---|---|---|---|
| C-F1 | D7 (package-manager dependency surface) mapped to R6 falsely — a harness adapter executes, it does not translate manifests | FATAL | **Fixed**: D7 reclassified out of scope (dataset/coverage boundary); foreclosure claims removed from R6; `dependency_untranslated` partial-credit class defined (`11` scope corrections, `14` §8). |
| C-F2 | F5 (OOM `SIGKILL`) claimed killed by honesty gates — in-process gates cannot execute post-mortem; no prevention exists | FATAL | **Fixed**: operator-side prevention promoted into R9 (cgroup memory cap, watchdog restart budget as harness-bounds config, resource telemetry in receipts); cap violations classify `resource_capped` partial-credit (`11` R9, `14` §1). |
| C-F3 | F6 (transport 500s/hangs) double-counted under R6 (target-code harness) and R9 — no LLM transport mechanism specified | FATAL | **Fixed**: transport failure class assigned to the daemon transport layer (ADR-0016 lineage) with retry/backoff, keepalive, and a restart budget distinguishing transport from solver faults; R6 scope note added (`11` R6). |
| C-F4 | F4 double-counted under R1/R6 — a faithful receipt *records* the stub pass; only R2/R8 prevent it | VALID | **Fixed**: Forecloses columns corrected (`11` R1: D9,F3,F8; R6: D3,F6,G6). |
| C-F5 | R5 progress-gated budgets fail on flat-reward from-scratch synthesis (D5: R=0.0 until complete) | VALID | **Fixed**: termination no longer rests on reward progress alone — difficulty tiering + wall-clock sub-budgets carry flat-reward vertices to a terminal classification (`11` R5). |
| C-F6 | G8a/G8c: Tier-C/macro-regression machinery retained without any validation corpus; differentiating claims unexercised | VALID | **Fixed**: new R13 — SCC/composite/regression code paths ship synthetic VAL-SPLIT/VAL-ASSEMBLY contracts; unexercised claims labeled `unexercised` in receipts (`11` scope corrections, `13` C4.5, `14` §7 row 7). |
| C-F7 | Failure-mode count arithmetic (26 canonical vs 28 rows; G8 split into G8a/b/c) | MINOR | **Fixed**: G8 sub-ids formalized in `11` and coverage table. |
| C-F8 | Attack "racing destroys MCTS backprop" | NON-ISSUE | **Rejected**: same as T-F8. |
| C-F9 | Attack "single ground truth breaks replay testing" | NON-ISSUE | **Rejected**: R9 provenance classes keep simulated receipts valid for harness unit tests; R1 binds only hardening certification (`12` A9). |

### RefuteLit (literature verification — 12-claim table)

| # | Finding | Severity | Disposition |
|---|---|---|---|
| L-claims | StateFlow, GVS5H, AFlow, MaAS, Articulate-but-Wrong, MockingBird, TestGenEval, RLEF, CodePRM, Coquelin & Munos, Tolpin & Shimony all **CONFIRMED** against primary sources | — | No action. |
| L-F1 | Phantom audit resolution: `15-journey.md` claimed disconfirming literature (BenchAgent, Tran & Kiela, AgentModernize, SWE-Bench+) was "engaged in A1/A2" — it was only cited | VALID | **Fixed**: A1 engages the anti-MAS line explicitly (comparison-class argument, `12` A1); the token-matched single-agent baseline is named in `10` §5 as a falsification test and scored as candidate C0-pooled in `13-candidates.md` (wave-2 fix: originally claimed but absent). |
| L-F2 | ProST misattribution: 14-14-14 is a *homogeneous* trained triad (Pareto gains via progressive sub-task training), not a zero-shot cascade; heterogeneous triad was 14-7-7 | VALID | **Fixed**: ProST corrected everywhere (`11` R11, `12` A8, `13`, `14`, `15`, `17`); cascade language replaced with verification-capacity concentration. |
| L-F3 | "Noisy binary rewards under UCB is an open gap" contradicted by RLVεR (2601.04411), Noisy Verifiers (2510.00915), VerMCTS | VALID | **Fixed**: the design-space text no longer claims a literature gap; the race justification rests on F7's *measured* dispersion failure + allocation evidence, with the noisy-verifier MAB literature acknowledged as adjacent (`12` A3). |
| L-F4 | Epistemic/stochastic conflation (same root as T-F3) | VALID | **Fixed** with T-F3. |
| L-F5 | GVS5H authenticity | NON-ISSUE | **Accepted**: verified against primary source. |

### RefuteFeas (implementation feasibility, code-anchored)

| # | Finding | Severity | Disposition |
|---|---|---|---|
| F-F1 | `SearchTree`/`compute_ucb` are single-candidate scalar-reward; paired racing needs vector receipts and a distinct race controller; `BudgetExhausted` must split race vs repair phases | VALID | **Accepted as engineering cost (M)**: recorded in the change list; `ModuleSolver` trait boundary accommodates a two-phase controller (RefuteFeas F9: no architectural blocker). |
| F-F2 | ADR-0012 softmax/UCB constants fully obsolete under racing; config fields become dead | VALID | **Accepted**: supersession row already in `14` §7.5; config migration (deprecate `ucb_exploration_c`, `softmax_temperature`; add race parameters) recorded in the ADR change list. |
| F-F3 | `execute_external_candidate` is a binary stub: discards generated test files, scores by exit code only, `parse_test_counts` is cargo-format hardcoded | VALID | **Fixed in design**: R6 now names per-test result format + per-language parsers as requirements; the stub behavior is classified as the defect R6 exists to remove (`11` R6). |
| F-F4 | Mutation infra absent; AST mutation is L-cost; stub-sensitivity is S-cost (<150 LoC in `TwoStageEvaluator`) | VALID | **Fixed with S-F1**: stub-sensitivity mandatory/cheap; mutation optional budgeted gate; honest receipt label when absent. |
| F-F5 | `guard_s2` hardcodes whole-file identities; splitter must hook after `tarjan_condense`; artifact storage changes needed | VALID | **Accepted as engineering cost (M)**: hooks named in `12` A6 and `14` §7.7. |
| F-F6 | Two new VAL contract families (VAL-SPLIT, VAL-ASSEMBLY) + amendments to VAL-MACRO-02/BB-G-01/AEVAL-02 required | VALID | **Accepted**: adopted into R13 and change list item 7. |
| F-F7 | Transport strictly single-endpoint (`bench_llm::OpenAiCompatModel`); no multi-endpoint routing exists | VALID | **Accepted**: consistent with R11's single-GPU form (mechanized + procedural capacity, no second endpoint required); multi-endpoint schema remains change-list item 9 for non-regime use. |
| F-F8 | Cascade schema breaks ADR-0009 canonical config; 118 YAMLs need compatibility defaulting | VALID | **Accepted**: schema extension with serde backward-compat defaults recorded in change-list item 9. |
| F-F9 | Attack "PASIPHAE is architecturally impossible" | NON-ISSUE | **Rejected**: feasibility audit itself confirms tractability behind existing trait boundaries; costs are M-tier, not rewrite-tier. |

### RefuteMeas (measurement validity)

| # | Finding | Severity | Disposition |
|---|---|---|---|
| M-F1 | Objective $\Pr[\text{accept}]$ is oracle-relative and rewards specification gaming (weak suites make stub-pass the degenerate optimum) | FATAL | **Fixed**: accept hardened from the inside via R2 (receipt counts only if suite passes stub-sensitivity + budgeted mutation checks); raw §6 rates reported alongside and expected to drop (`10` §1). |
| M-F2 | Cost model dimensionally degenerate; ignores failure-cost asymmetry (toml 774 req vs heapq 107) and slot queuing | VALID | **Fixed**: conditional mixture cost model with GPU-seconds units, slot-contention discount, and operator attention removed from the metric (unobservable in receipts); failure-cost asymmetry first-class; objective reclassified as an offline evaluation metric, not an online scheduler objective (`10` §1). |
| M-F3 | Truncation semantics undefined; discarding over-bound campaigns inflates $\Pr[\text{accept}]$ by survivorship bias | FATAL | **Fixed**: `10` §5b — truncation = measurement with partial-credit classification; only provenance/stub violations invalidate; resource violations classify `resource_capped`. |

## Completeness critic (cross-lens)

- Coverage mapping: 28 failure rows (F1–F8, D1–D10, G1–G8 split G8a/b/c) — after wave-1 fixes, every FATAL/VALID finding has a named mechanism or an explicit scope waiver; residual risks are parked in `10` §5 (epistemic status) and `14` §8 (non-fixes).
- Requirement count: R1–R13 (R13 added by C-F6); design-axis coverage table updated; no orphan rows.
- Cross-file consistency spot-checks after fixes: progress quantum, reset cap, oracle budget policy, ProST correction, R13 column — consistent across `10/11/12/13/14/15`.

## Wave 2 — re-audit of changed targets (executed)

Re-dispatched three lenses (`RecheckTheory`, `RecheckScale`, `RecheckMeas`) on the amended sections only, with instructions to attack the fixes, not re-litigate wave 1.

### RecheckTheory (theory re-audit of amended race/totality sections)

| # | Finding | Severity | Disposition |
|---|---|---|---|
| W2-T1.1 | Totality rule leaves an undefined micro-state below the futile cap (cannot park in M0, cannot sit in {M1–M3}, cannot fail); `ev_ModuleFailed` has no downstream cascade → children wait in M0_Blocked, liveness never trips, S5 hangs to wall clock | FATAL | **Fixed**: below-cap exhaustion parks in the existing parentless-park state with `strategies_exhausted: true`, excluded from ReadySet pending external unblock (R3 semantics, no eighth state); `ev_ModuleFailed(v_i)` deterministically marks downstream dependents `Failed(DependencyUnmet)` without dispatch (`14` §2 totality rules). |
| W2-T1.2 | Restart-on-oracle-upgrade has no budget semantics: no-refund starves (70%-spent module can't afford $N_{\min}$); refund loops; oracle-stage advance consumes the 3-reset allowance; intra-module gate advance can self-trigger restarts | FATAL | **Fixed**: only *external* oracle changes restart a pre-commit race; intra-module stage advance never restarts (post-commit upgrades re-gate the winner; invalidation routes through stall/revive); restarts refund nothing and an externally-triggered restart consumes one of the 3 reset allowances → cap terminates rather than loops (`10` §5b, `14` §2/§3). |
| W2-T1.3 | Stall-revive vs futile-cap decoupling: $K_{\text{stall}} \ge B_{\max}$ lets the cap kill first; demotion-as-elimination resets the futile counter (failure buys budget); post-revival-exhaustion transition undefined | VALID | **Fixed**: $K_{\text{stall}} < B_{\max}$ and stall counter persists across dispatches; demotion is not an elimination and resets nothing; after both revivals stall, dispatches count futile normally → `M_Failed(StrategyExhaustion)` at cap — no third path (`14` §2). |
| W2-T1.4 | Depth-1 probe horizon is myopic (principled strategies need $d \ge 2$ to clear cross-function signature repairs; shallow strategies win at $d=1$) | VALID | **Fixed**: probe depth set to $d = 2$ everywhere race arms are scored (`12` A3, `13` C4.1, `14` §2). |
| W2-T1.5 | "Bounded confidence" and journey's "simple-regret-optimal shape" smuggle optimality back after T-F2 struck it | VALID | **Fixed**: "dominates with bounded confidence" → "dominates on the paired outcome vectors"; journey phrasing downgraded to empirical heuristic (`14` §2, `15`). |
| W2-T1.6 | $N_{\min}$ scope ambiguous (per-arm per-race degenerates to round-robin at high $N_{\min}$; per-vertex lets a leader void the guarantee) | VALID | **Fixed**: scoped per-arm-per-race, $K \le 3$, $N_{\min} = 2$ depth-2 probes ≈ 26 s/race, deducted from the vertex sub-budget (`14` §2). |
| W2-T1.7 | L2 probes during race violate gate monotonicity (non-compiling candidates yield zero signal) or churn (dual execution every rollout) | VALID | **Fixed**: L2 probes only for L0-passing candidates, final race round only (`14` §2). |
| W2-T1.8 | "Decoupled $N_{\min}$ enables unbounded exploration" | NON-ISSUE | **Rejected**: wall-clock sub-budget + campaign bound override all counters. |
| W2-T1.9 | "Test anchors block necessary refactoring" | NON-ISSUE | **Rejected**: anchors preserve external behavior contracts, not internals. |

### RecheckScale (scalability re-audit of amended arithmetic)

| # | Finding | Severity | Disposition |
|---|---|---|---|
| W2-S1.1 | Per-suite mutation across 100 modules: 50 min (interpreted) – 8.3 h (worst) — still out of bounds; "early-abort" never fires on adequate suites; weak suites trigger amplification → restart loop | FATAL | **Fixed**: mutation demoted from per-module gate to **campaign-level spot audit** (≤5 modules/campaign, ≤30 mutants, 10 s/mutant, escalates to mandatory amplification for the affected generator's outputs; receipts label prior hardens `stub_sensitivity_only`); hardened-accept no longer requires mutation (`10` §1, `12` A4, `14` §3). |
| W2-S1.2 | Stub-sensitivity "1.4%" figure rests on interpreted single-function assumptions; compiled crates at 5–10 s incremental builds cost 7–17% of $W$; per-symbol stubbing is 70 min | VALID | **Fixed**: cost honesty rewritten — whole-file stubbing is the mandatory gate (one run/suite; up to ~17% $W$ compiled), per-symbol stubbing reserved for the mutation-audit tier (`12` A4, `14` §3). |
| W2-S1.3 | Composite batching "≤ 2× file count" contradicts coupling topology: binding closures span whole files (bst/rbt/html/toml/mathgen); forced 2-way partition cuts shared `class_var`/nonlocal state and regenerates $O(S^2)$ cascades | FATAL | **Fixed**: batching rule rewritten as coupling-tiers — shared-binding groups stay one vertex when they exceed the ~300 LoC context budget (decomposition refused where coupling forbids it); only genuinely separable clusters decompose (`12` A6, `13` C4.3, `14` §1). |
| W2-S1.4 | Token-capacity: "~64 steps/module" wrong by 4× — 1.93 M tokens ≈ 1,608 steps ≈ 16/module over 100 modules; 25 hard-tier modules at 64 steps consume the whole budget; §5b pathological cap leaves ~9.7 steps/module for the rest | FATAL | **Fixed**: corollary corrected in all files; difficulty tiers redefined as zero-sum fractions of the campaign mean (patch 8, hard 40–64 with fewer hard vertices); crust-class all-synthesis corpora declared incompatible with the 2 h bound at 100 modules (fewer vertices per campaign or larger bound) (`12` A8, `13`, `14` §6). |
| W2-S1.5 | Truncation: final harness pass is unbudgeted (4–25 min over the bound) and mid-flight candidates can score accept without full hardening | VALID | **Fixed**: pre-reserved tail budget $W_{\text{tail}} = \min(5\text{ min}, 2\%W)$; accept at truncation only from *existing* receipts already satisfying the hardened-accept predicate — no invented evidence (`10` §5b, `14` §6). |
| W2-S1.6 | R9 prevention as written is an in-process impossibility: unprivileged daemons cannot set their own cgroup limits; post-`SIGKILL` classification/receipt generation cannot execute in the dying process | FATAL | **Fixed**: external supervisor contract — daemon ships a supervisor unit (systemd scope `memory.max` or wrapper outside the cgroup) configured from harness-bounds fields; the supervisor survives the kill, reads the in-life telemetry journal, and writes the `resource_capped` terminal receipt (`11` R9, `14` §1). |

### RecheckMeas (measurement re-audit of amended objective/truncation/R13)

| # | Finding | Severity | Disposition |
|---|---|---|---|
| W2-M1.1 | Hardened-accept predicate inconsistent across files: mutation required in `10` §1 but skippable in `12`/`13`/`14`; L2 absent from the `10` definition | VALID | **Fixed**: single predicate in `10` §1 — four §6 criteria + mandatory stub-sensitivity + L2 differential gate (executable legacy); mutation reclassified as spot-audit instrument with `oracle_strength` enum on every receipt (`10` §1, `14` §4). |
| W2-M1.2 | Mixture cost model unestimable (operator attention unobservable) and unmaximizable online (fractional functional ≠ scheduler policy) | FATAL | **Fixed**: operator attention dropped from the metric; objective reclassified as an *offline* evaluation metric for ranking methodology candidates; the online scheduler runs deterministic policies (`10` §1). |
| W2-M1.3 | Truncation L2-bypass hole: module-level accept on raw §6 criteria at timeout replicates ReCodeAgent's 86.5%-vs-97.8% overfitting (deliberate over-run at campaign level is a NON-ISSUE: it minimizes $\pi^\star$) | VALID | **Fixed** (with W2-S1.5): truncation accept requires the existing full hardened-accept chain; partial-credit labels preserve the distinction (`10` §5b). |
| W2-M1.4 | Progress quantum $0.05$ uncalibrated: canonical reward weights one test at $0.6/N$, so genuine single-test progress ($N>12$) is discarded; relative reading divides by zero at $R_{\text{best}}=0$ (D5) | VALID | **Fixed**: quantum redefined as one test's worth, $\Delta R \ge 0.6/N_{\text{suite}}$, absolute not relative; oracle-stage advance clears the counter without consuming a reset allowance (`10` §5b). |
| W2-M1.5 | R13 `unexercised` labeling unauditable: no schema field in the receipt contract | VALID | **Fixed**: receipts carry a schema-level `unexercised: [claim_id]` array + `oracle_strength` enum (`14` §4). |
| W2-M1.6 | Phantom wave-1 dispositions: queuing note absent from `10` §1; single-agent baseline + "C0-pooled" scoring claimed but absent; `14` §7.7 citations dangle; §7 row 9 still said "Cascade routing" | FATAL | **Fixed**: queuing/slot-contention now in `10` §1; C0-pooled added as a scored falsification baseline in `13`; §7.7 citations corrected to §7 row 7; row 9 rewritten to verification-capacity concentration. |

## Convergence

Wave 2 attacked only the amended sections and surfaced **7 FATAL + 9 VALID** findings — all dispositioned as *fixed* above, with the amended text updated in the same pass (`10/11/12/13/14/15/16`). A third wave on the wave-2 amendments would re-run the same lenses on: coupling-tier batching, spot-audit mutation policy, zero-sum token allocation, totality/liveness cascade, external supervisor contract, and the hardened-accept predicate. ## Wave 3 - re-audit of wave-2 amendments (executed)

Single read-only lens (`Wave3Audit`) over the six wave-2 fix targets only.

| Finding | Verdict | Disposition |
|---|---|---|
| W3-1 | FATAL — below-cap strategy-exhausted modules park in M0, invisible to the {M1–M3} liveness invariant; empty ReadySet deadlocks the campaign (F2 replay) | **Fixed**: park permitted only with a live unblock source, else direct `M_Failed(StrategyExhaustion)`; liveness invariant extended to M0-held vertices + dependents with an Unreachable sweep (`14` §2, §6; `12` M2) |
| W3-2 | VALID — stale `≤ ~2× file` batching formula still in `12` A6 / `14` §1 while `13` C4.3 was amended | **Fixed**: both rewritten to coupling-tier grouping with explicit no-decompose-on-coupling clause (`12` A6, `14` §1) |
| W3-3 | VALID — mutation spot-audit + generator escalation existed only in ledger/meta-strategy, not in `12`/`14` | **Fixed**: spot-audit orchestration (≤5 modules, ≤30 mutants, generator escalation) synced into `12` A4 + `14` §3; residual overfitting risk named |
| W3-4 | VALID — stale $\epsilon_{prog}{=}0.05$ in `12`/`14` while `10`/`16` carried the $0.6/N_{suite}$ fix | **Fixed**: quantum redefined everywhere as $\Delta R \ge 0.6/N_{suite}$ absolute (`12` M2, `14` §1/§2/§6) |
| W3-5 | VALID — no tier-assignment predicate, hard-tier admission cap, or re-balancing; unconstrained hard tier breaks the 1,608-step envelope | **Fixed**: deterministic blackboard predicate + hard-tier quota ⌈0.15·|V|⌉ + eviction/extension rules (`12` A8, `14` §6, `11` R10) |
| W3-6 | MINOR — L2 parity gate threshold unquantified; non-executable legacy vacates L2 silently | **Fixed**: exact-match on byte-identical configs, ≥0.95 normalized otherwise, `parity: verdict_only` label for non-executable legacy (`10` §1, `14` §3) |
| W3-7 | MINOR — supervisor journal has no O_SYNC/flush mandate; cgroup kill drops the tail | **Fixed**: O_SYNC append, fsync per 5 s sample mandated (`11` R9, `14` §1) |
| W3-8 | MINOR — diamond-DAG failure cascade keeps sibling branches spending after sink terminal | **Fixed**: sibling cancellation at next micro-state boundary; DependencyUnmet recoverable only via R12 proof-object regression (`14` §2) |
| W3-9 | NON-ISSUE (attack refuted) — refusing decomposition on coupled files voids R7/D4 | Rejected: D4's defect was *artificial* shredding of whole-file bindings; refusal avoids the defect class; unexercised R7 machinery is receipt-labeled per R13 |

Three of these (W3-2/3/4) were wave-2 fixes claimed in the wave-2 section that had not actually landed in `12`/`14` — phantom dispositions of dispositions, caught only because wave 3 re-read the sources. Cost of skipping the re-read: the ledger would have certified fixes that existed only here.

**Convergence (10 §3 rule: two consecutive clean waves):** wave 3 surfaced 1 FATAL + 4 VALID + 3 MINOR — not clean. A wave-4 pass would re-run the lens on the wave-3 amendments (park-with-live-unblock-source semantics, tier admission quota, sibling cancellation, parity gate). **Status: wave-3 fixes applied; convergence remains provisional pending wave 4.**
