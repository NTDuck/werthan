# 12 — Design Space: Axes, Options, Evidence

Nine load-bearing axes. For each: the options that have measured evidence, the quantitative comparison, and the position the werthan evidence forces. Bibliography keys resolve in `17-bibliography.md`.

## A1. Orchestration topology

| Option | Best evidence | Result |
|---|---|---|
| Free-form chat (CAMEL/AutoGen-style) | MAST traces; Zhang et al. MAD re-evaluation | MAD "often fails to beat single-agent CoT despite much higher compute"; conversation-as-control fails at horizon |
| Manager-worker pipeline (MetaGPT/ChatDev) | MetaGPT ICLR'24; MAST contrast | Fewer FC1+FC2 failures (−60/−68%) but **1.56× more FC3** (over-trusted verifier); fixed pipeline ignores difficulty |
| Manager–worker over filesystem ledger (GVS5H) | LCB-100, 11 models | +25.6pp on Qwen3.8-27B (66.8→92.4) ≈ frontier single-call; executes-tests-override-manager |
| Blackboard, volunteer KSs (LbMAS-Salemi) | 3 datasets × 4 backbones | +13–57% relative over master–slave & RAG; response-board isolation |
| Blackboard + control unit (LbMAS-Han&Zhang) | 6 benchmarks | 81.68 avg, cheapest tokens vs MaAS/AFlow/GPTSwarm; cleaner-agent load-bearing |
| Hierarchical statechart + orthogonal regions + blackboard (THESEUS) | our campaigns | Right instincts (surgical regressions, warm-start) but F1/F2/F3 realized — machinery unsound at edges (T1, T6) |

**Forced position: hybrid** — Harel statechart macro-shell with orthogonal module regions + typed blackboard + per-module evidence slices; ledger-style artifact files as the persistence layer. Keeps THESEUS's validated machinery (VAL-MACRO-*) and imports GVS5H's override rule and LbMAS's isolation.

**Disconfirming evidence, engaged (wave-1 literature audit, RefuteLit F1):** the anti-MAS line is real and is not waved away — BenchAgent: 5/6 MAS configs *lose* to a pooled single agent at matched compute; Tran & Kiela: at equal thinking-token budgets, single-agent matches MAS (DPI argument); AgentModernize: pipeline benefit is *inversely correlated with model strength*, and a 27B generator has fundamental single-call bottlenecks scaffolding cannot conjure away. Why the hybrid still wins in this regime: (a) the comparison class of those papers is *LLM-to-LLM agent chatter*, not deterministic-shell + mechanized verification — the strongest components in the forced position (receipt gates, stub-sensitivity, differential execution) are model-free and appear in no single-agent baseline; (b) BenchAgent's losing MAS share their failure modes with our F1/F2 (unmanaged topologies → deadlock/rest-loop) — the deterministic statechart exists precisely to delete that class, which is why S19's uncontrolled small-model fleets collapse (99.67→72.01%) while a *bounded* shell should not; (c) the token-matched single-agent baseline is nonetheless **mandatory in the falsification test** (`10-meta-strategy.md` §5): a PASIPHAE-vs-single-agent-pooled sweep at equal tokens is the first experiment if the hybrid is challenged, and C4 is scored against C0-pooled in `13-candidates.md` for exactly this reason.

## A2. Control locus (who owns the transition function)

| Option | Evidence | Result |
|---|---|---|
| LLM-proposed (ReAct) | Rethinking-Scale S19 | On <10B models: completion 99.67→72.01%, tokens ×6.2, delegation failures dominate |
| Deterministic (StateFlow/LangGraph/GVS5H harness) | StateFlow | +13/+28% accuracy at 3–5× lower cost vs ReAct |
| Learned controller (AFlow/MaAS/ADAS) | AFlow | +5.7% avg; small models beat GPT-4o at 4.55% cost — but needs offline search budget + training data |

**Forced: deterministic shell + bounded agentic interiors.** S19 shows the failure mode deterministic control removes is precisely the one that kills small-model fleets; learned orchestration is unaffordable to develop for one repository class. This is also the repo's own convention (Hoverbear typestate FSMs, `.omp/rules/specs.md` §3).

## A3. Intra-module search procedure

| Option | Evidence | Result |
|---|---|---|
| Greedy single-trajectory repair | Oxidizer; AlphaTrans coupling effect | "stuck in repair loops"; one buggy callee shadows many callers |
| MCTS over drafts (LATS) | RethinkMCTS comparison | Beats greedy; reflections accumulate errors (−19–36% token penalty vs rethink) |
| MCTS + thought regeneration (RethinkMCTS) | EMNLP 2025 | P-UCB + in-place thought refinement; best granularity result |
| Portfolio race → commit → focused repair | Snell S12; LLM-Monkeys; CodeMonkeys | Compute-optimal allocation >4× vs best-of-N; coverage log-linear in samples; selection-over-ensemble +8.8pp |

**Forced: portfolio race, then commit, then focused repair.** Rationale: rollout rewards here are *noisy binary-ish* (compile gate + stochastic test outcomes on a 27B generator), where UCB-style exploration spends budget re-confirming known failures. **Honest claim status (wave-1 theory audit, RefuteThe F1–F3):** this is an *empirical compute-budget heuristic*, not a bandit theorem. Successive elimination assumes stationary arms; here a "strategy arm" is non-stationary (repair shifts the reward distribution, and a principled-but-slow starter can look worse than a superficial heuristic pre-repair). Two design consequences, adopted: (1) race arms are scored on **short depth-2 probe repair rollouts, not zero-shot drafts** (depth-1 re-introduces myopic horizon bias — principled multi-turn strategies need $d \ge 2$; RecheckTheory W2-F4), so the race samples what repair can achieve rather than draft luck; (2) the committed winner's repair trajectory that stalls ($\Delta R < 0.6/N_{\text{suite}}$ for $K_{\text{stall}}$ rollouts) triggers `ev_StrategyStalled` → demote, revive the runner-up from checkpoint (bounded: ≤2 revivals per vertex, then terminal classification). The race also keeps a **guaranteed minimum exploration floor** $N_{\min}$ rollouts per difficulty tier (RefuteThe F3): stronger oracles make rewards sparser (more true negatives), and the generator needs sampling volume to find sparse positives — oracle gating and exploration budget are decoupled. The literature supports the *shape* (Snell: difficulty-adaptive allocation >4× vs best-of-N; Tolpin & Shimony SR+CR two-phase; F7's softmax-UCB dispersion failure), but "bounded-sample guarantees" is struck: no regret bound is claimed. MCTS machinery (tree, backprop) is retained *within* the repair phase on the winning strategy, so ARIADNE's evidence reuse is not discarded.

## A4. Oracle stack

| Option | Evidence | Result |
|---|---|---|
| Exit-code only | D3 | One binary outcome; no per-test feedback reaches repair |
| Test-only (LLM-generated suites) | ReCodeAgent function-vs-test gap; Ahmed et al. | 86.5% test vs 97.8% function validation; 21.8–33% test overfitting |
| + differential execution | tHinter, ACToR, RustAssure | −71% lines-to-review; +36.7% pass; symbolic differential catches what fuzzing misses |
| + mock-based isolated validation | MockingBird | 25.14%→43.10% functional equivalence, 0 FP / 0.9% FN — best published validator |
| Layered: symbolic gates → differential → adversarial tests → verdict aggregation | MatchFixAgent | 99.2% verdicts at 1,650 LoC; short-circuits skip 25–35% of LLM calls |

**Forced: layered stack, adversarially independent.** G1/R2/R8. **Budget policy (wave-1 scalability audit, RefuteScale F1 — the arithmetic is real):** full mutation scoring per candidate is infeasible (100 modules × 16 candidates × 30 mutants × 1 s ≈ 13.3 h vs a 2 h bound). The layered oracle therefore assigns execution cost by *what* is being certified, not a flat policy:

- **Stub-sensitivity (L1-lite)** — delete bodies / substitute trivial stubs, require the suite to fail: ~1 whole-module deletion run per suite — ≤~1.4% of campaign budget for skel-class single-file suites, up to ~17% of $W$ for compiled crates (RecheckScale W2-S1.2 cost honesty). **Mandatory, always.** This alone kills the toml F4 class.
- **Mutation kills (L1-full)** — AST-mutation infra does not exist in-tree and per-language mutation tooling is an L-cost build (RefuteFeas F4); per RecheckScale W2-F1 / wave-3 W3-3, mutation is a **campaign-level spot audit**, not a per-module gate: ≤5 modules per campaign, ≤30 mutants each, 10 s per-mutant timeout, early-abort at first survivor; a failed audit escalates to mandatory suite amplification for that generator's outputs and receipt-labels its prior hardens `stub_sensitivity_only`; campaigns that skip the audit carry the same label — the honest label replacing implied mutation strength. Residual risk (wave-4, named): whole-file stub-sensitivity does not catch happy-path assertion overfitting; up to 95% of modules in a 100-module campaign are un-audited, so for executable legacy L2 differential execution is the working check — for non-executable legacy the gap is documented, not closed.
- **Differential execution (L2)** — the strong layer, deterministic and model-free; probe with a bounded input subset during the race (RefuteThe F4: race must see a slice of the strong oracle or it commits to L0/L1-lucky strategies), full set at hardening.
- **Verdict aggregation** — only where execution is impossible (parity questions on non-executable contracts), never as a pass substitute for executables (EquiBench: LLM equivalence judgment is a distinct weak capability).

## A5. Memory substrate

| Option | Evidence | Result |
|---|---|---|
| Chat history | CAMEL; MAST FM-1.4 | Context loss 2.80%; telephone-game distortion is monotonic (2502.20258) |
| Ledger files (GVS5H) | LCB-100 | 28/35 single-call failures solved by manager arm — external memory survives token caps |
| Blackboard slices (ARIADNE/LbMAS) | ARIADNE | 4-slice evidence store; immutability of problem model prevents drift |
| Checkpointed state (LangGraph) | production | Per-super-step snapshots; resumability; the hook for warm-start |

**Forced: typed blackboard (four slices) + checkpointed event-sourced state, persisted as artifacts.** Preserves VAL-BB-* contracts; warm-start resume across dispatches (G3) becomes a checkpoint restore, not an ad-hoc remap.

## A6. Translation-unit granularity

| Option | Evidence | Result |
|---|---|---|
| Whole file/module | METHODOLOGY Def 1 | Blind to intra-file coupling — D4's cross-block contracts |
| Sub-file blocks (ReCodeAgent gold) | 03-skel-block-granularity.md | 8–81 slices; late-bound `class_var`; dropped global block 0 |
| Fragments with schema + reverse-call order (AlphaTrans) | FSE'25 | 96.4% syntactic; validation is the ceiling |
| Skeletons with sound-decomposition guarantee (Skel) | PLDI'25 | 95% fragments auto-translated, whole-program tests pass |

**Forced: sub-file vertices *where coupling requires it*, each carrying an assembly contract** (late-bound bindings, attachment lists, module-state ownership, cross-slice exception/comparison/numeric semantics). DAG construction gains a coupling-driven splitter; assembly contracts are checked at slice assembly time, deterministically. **Scalability guard (wave-1, RefuteScale F3):** naive per-slice vertices multiply statechart re-entry overhead — skel files average 42 slices (up to 81), and per-slice LLM dispatches with independent context hydration would multiply wall-clock beyond the bound, while shared-scope siblings (late-bound `class_var`, module globals) invalidate each other in $O(S^2)$ Tier-B cascades. Design consequences, adopted (rewritten per wave-3 W3-2 — the earlier ≤ ~2× file-count formula was the rule wave-2 refuted): (1) decomposition is **coupling-tiered**: blocks sharing a binding are grouped into one vertex only while the group fits the per-vertex context budget (~300 LoC); a binding-closure larger than that stays **one whole-file vertex** (decomposition refused where coupling forbids it — W3-9 confirms this is the correct reading of D4: ReCodeAgent's defect was *artificial shredding* of whole-file bindings, so refusal avoids the defect class at its root; on this corpus skel collapses to whole-file vertices and R7's slice machinery is `unexercised` there per R13 labeling); (2) decomposable files emit **explicit directed assembly edges** between batch vertices (RefuteThe F6: peer slices without edges cannot broadcast Tier-B conflicts under R3); (3) a file-level container owns final assembly verification once child batches reach `M4_Hardened`, with deterministic blame attribution invalidating only the blamed batch. Implementation hooks named by the feasibility audit (RefuteFeas F5–F6): splitter lands after `tarjan_condense` on the file graph; `guard_s2` must accept sub-file identifiers mapping to parent inventory entries; new `VAL-SPLIT`/`VAL-ASSEMBLY` contract families.

## A7. Scheduling & budget policy

| Option | Evidence | Result |
|---|---|---|
| Uniform $B_{\max}$ per dispatch + futile cap, no resume | our v1–v17 | F1 + F7: 48 chances, then death; toml 774 requests to nowhere |
| Wave scheduling with liveness invariant | VAL-MACRO-18 (landed) | F2 fixed; keep |
| Difficulty-adaptive per-vertex effort | MaAS 6–45% cost; Snell >4× | Aligns spend with synthesis-vs-patch difficulty |
| Progress-gated continuation | F7 evidence | Futile-cap reset on best-reward gain; warm-start resume carries tree + fault records |

**Forced: wave scheduling + difficulty-adaptive budgets + progress-gated continuation.** Budget sizing from the log-linear coverage curve (LLM-Monkeys): expansion until marginal reward-gain per request drops below the next vertex's expected value; hard vertices (synthesis-sized, D5) get order-of-magnitude more budget than patch-sized ones.

## A8. Model routing

| Option | Evidence | Result |
|---|---|---|
| Single model everywhere | incumbent | 27B verifier too weak to steer (R11 evidence); 27B planner mediocre at strategy diversity |
| Heterogeneous cascade at decision points | ProST S18; NVIDIA SLM position; GVS5H small-model-gains-more | Stronger orchestrator/verifier capacity is Pareto-optimal; cheap fleet for generation — **corrected per RefuteLit F2**: ProST (arXiv:2509.04508) shows *progressive sub-task training* on a homogeneous 14-14-14 triad is Pareto-optimal; it does not evidence zero-shot prompt cascades, and its heterogeneous triad was 14-7-7. The ProST anchor supports trained/structured orchestration, not runtime model mixing. |
| All-frontier | cost model | Contradicts regime; AFlow shows scaffold can substitute for scale at 4.55% cost |

**Forced: verification-capacity concentration (single-GPU form; carries the difficulty-tier admission policy, wave-3 W3-5 / wave-4 W4-5).** Tier assignment per vertex is a deterministic blackboard predicate (LoC, stub density D5, binding-closure size, fan-in); the hard tier (40–64 candidate-steps) is quota-capped at ⌈0.10·|V|⌉ vertices prioritized by fan-in × coupling score; mid-campaign escalation requires evicting the lowest-priority scheduled hard vertex; a patch-tier vertex showing ≥2 one-test progress gains within its 8 steps earns one progress-earned 8-step extension charged to its reset allowance. Fleet model (27B) for generation/patch/extraction volumes; the strongest *available* capacity at arbiter/critic/selection nodes. **Honest single-GPU accounting (wave-1, RefuteScale F2 — accepted as binding):** no second model co-hosts on a 24 GB card, so a "cascade" of model *sizes* does not exist here, and same-weights prompting is not independent verification (Articulate-but-Wrong 31.7% self-endorsement). The forced position is therefore: (a) **mechanized verification capacity** at decision points — differential execution, stub-sensitivity, coverage, parity checkers: model-free, unlimited by VRAM, and the only truly "stronger" verifier available; (b) **procedural capacity** — read-isolated validator prompts, multi-sample verdict voting, adversarial witness generation on the same 27B, each self-endorsement-flagged `self_endorsed: true` and barred from solo hardening certification (R11); (c) remote frontier calls at low-rate decision points remain an explicit **non-goal in this regime** (fixed local serving stack, `10-meta-strategy.md` §1) — recorded as the regime boundary of the ProST/SLM-economics argument, not an available option. Token-capacity corollary (corrected per RecheckScale W2-F4): 2 h × 268 tok/s ≈ 1.93 M output tokens ÷ 1.2 K tokens/step ≈ **1,608 candidate-steps ≈ 16 steps/module average over 100 modules** — a hard-tier allocation of 40–64 steps is a zero-sum spend against the campaign mean, not a per-module entitlement; crust-class all-synthesis corpora (D5) cannot fit 100 modules in 2 h at any architecture and run proportionally fewer vertices per campaign. The difficulty-adaptive effort split (R10) and composite batching (A6) are preconditions of feasibility, not refinements.

## A9. Honesty & measurement gates

| Option | Evidence | Result |
|---|---|---|
| Simulated receipts as decor | D10 | 87.5% fictional pass rate; invalidates everything downstream |
| Receipts recorded but ungated | v18 partial | OOM kills still produce "measurements" of truncated artifacts |
| Single-ground-truth honesty gates | G4+G7 | `llm_requests>0` + provenance + terminal artifact integrity + resource caps |

**Forced: single-ground-truth gates, enforced at harness layer.** The harness receipt is the atom of evidence; every acceptance metric, reward term, and hardening event cites it; provenance (real/simulated/remote) recorded and gated.

## Axis interaction notes

- A3 × A7: portfolio racing changes what "futile" means — a dispatch that produced reward *information* (better empirical means on some strategies) is not futile even with no new candidate; the cap counts *zero-information* dispatches **beyond the progress quantum** — one characterization test's worth, $\Delta R \ge 0.6/N_{\text{suite}}$ absolute (wave-2 fix, RecheckMeas F4: the earlier fixed $\epsilon{=}0.05$ discarded genuine single-test progress when $N_{\text{suite}} > 12$; relative readings divide by zero at $R_{\text{best}}{=}0$; oracle-stage advance clears the counter without consuming a reset allowance; ≤3 resets per vertex, wall-clock sub-budget overrides all counters).
- A4 × A5: mutation-kill and stub-sensitivity checks write oracle-strength receipts into $\mathcal{B}_{CT}$ metadata; PassRate is only trusted when suite strength ≥ gate.
- A6 × A1: assembly contracts live on DAG edges as a new artifact type; Tier-B conflicts can now target intra-file slices, which requires the parentless-park rule (R3) generalized to slice-level anchoring.
- A8 × A9: remote frontier calls are out of regime (A8, single-GPU form); the residual receipt distinction is same-weights self-endorsement — campaigns must report the `self_endorsed` rate at arbiter/critic/selection nodes or the verification-capacity claim is unfalsifiable.
