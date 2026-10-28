# 05 — Mapping Downsides to Methodology Gaps

Each ReCodeAgent downside (D1–D10 in README) with the specific METHODOLOGY.md mechanism that must overcome it, the mechanism's current blind spot, and the design decision the evidence forces. All are proposals; METHODOLOGY.md is read-only ground truth — changes route through MADR ADRs.

## G1. Oracle strength is unspecified — $\mathcal{A}_{\text{test}}$ / $\mathcal{A}_{\text{eval}}^{\text{distill}}$ (D1, D2, D3)

- **Evidence.** 1,593 single-call snapshots with no error identity (02 §1); second-hand gold suites with 0.2 float tolerances (02 §2); 97/180 crust binaries with one `#[test]`, some assert-free; `require_byte_identical_io: false` ×118; skel's oracle is an exit code.
- **Blind spot.** METHODOLOGY §4.2 defines $\mathcal{B}_{CT}$ as "characterization I/O snapshots and property-based tests" and requires $\text{PassRate}_{\text{char}} = 1.0$ for $R = 1.0$ — but **nothing specifies a minimum strength for the suite itself**. A weak suite passes a semantically wrong candidate at full reward (toml's vacuous pass is the realized form). Conversely a brittle suite (exact f64 equality) punishes correct candidates.
- **Design decision forced.** Oracle-strength gating: $\mathcal{A}_{\text{eval}}^{\text{distill}}$ must measure suite sensitivity (error-path coverage, call-sequence coverage, mutation kills on the candidate, stub-sensitivity: a suite that passes against a deleted implementation must fail its module) before $\text{PassRate}_{\text{char}}$ can gate $M4$. Per-test feedback must reach $\mathcal{B}_{PR}$, not a binary exit code.

## G2. The translation unit is intra-file, not the file — Hydrate / $\mathcal{G}_{\text{slice}}$ (D4)

- **Evidence.** skel's block slices (8–81 per project) with late-bound `class_var`, dropped global block 0, cross-block exception/comparison/initialization contracts, and assembly sections no block contains (03).
- **Blind spot.** METHODOLOGY Def 1 sets vertices at "single file/module or SCC"; Def 4 hydrates a 1-hop $\mathcal{G}_{\text{slice}}$. Both are blind to coupling *inside* a vertex: $\mathcal{A}_{\text{gen}}$ writing one block receives no contract about sibling blocks' initialization order, thrown error classes, or operator semantics — exactly the defects ReCodeAgent's own gold artifacts exhibit (03 §1–4).
- **Design decision forced.** Either (a) sub-file translation units in $V$ (with an assembly-contract artifact capturing late-bound bindings, attachment lists, and module-state ownership), or (b) an explicit intra-vertex contract layer in $\mathcal{B}_{PM}$. ADR required; (a) changes DAG construction, (b) changes hydration only.

## G3. Budget semantics for synthesis-sized problems — $B_{\max}$ / futile cap (D5, F1, F7)

- **Evidence.** 3,090 stubs/100 projects = every crust vertex is a whole-library rewrite; 48 rollout chances per skel project; budget exhaustion remapped to a terminal conflict (F1); no warm-start resume across dispatches; no futile-cap reset on reward gain (F7).
- **Blind spot.** METHODOLOGY §4 treats $B_{\max}$ as a uniform knob and the Budget-Exhaustion Fallback (Alg 2, step 5) emits `ev_ContractConflict(Parent(v_i), …)` — **which is undefined for a parentless vertex** ($e_{pi}$ doesn't exist). Our engine inherited exactly this hole (F1). The fallback also discards the search tree on re-dispatch.
- **Design decision forced.** (a) The fallback must be total: parentless budget exhaustion parks in `M0_Blocked`/`M5`-equivalent, never fabricates a counterparty (already landed as VAL-M2-05; keep the invariant in the spec). (b) Budget policy: per-category $B_{\max}$, warm-start resume carrying $u_{\text{best}}$ + $\mathcal{B}_{PR}$ across dispatches, and futile-cap reset on reward-delta progress.

## G4. Reward and acceptance gate must share ground truth — $R(s_t)$ / `M4_Hardened` (D9, F3, F4, F8)

- **Evidence.** heapq: all four acceptance criteria observably green, reward never certified 1.0, module parked at the futile cap. toml inverse: `test_pass_rate = 1.0` against an 18-line stub. F8: metric-definition drift made criterion 1 unmeasurable.
- **Blind spot.** §4.2's $R(s_t) = w_{\text{comp}}\mathbb{I}_{\text{compile}} + \mathbb{I}_{\text{compile}}[\dots]$ computes its terms inside the cluster, while the campaign gate recomputes `compilation_success`/`test_pass_rate` from the workspace. Two evaluators, two truths.
- **Design decision forced.** Single source of execution facts: the harness run that produces acceptance metrics *is* the evidence $\mathbb{I}_{\text{compile}}$ and $\text{PassRate}_{\text{char}}$ consume (event-sourced hardening: the `ev_ModuleHardened` payload cites the harness receipt it certified). ADR required.

## G5. Wave-scheduler liveness — ReadySet / Tier C handling (D8, F2)

- **Evidence.** 6/6 pre-fix campaigns died at `no ready module while unhardened modules remain`; invalid Tier C cycle reports returned early without parking the reporter (VAL-MACRO-17/18 fixes).
- **Blind spot.** §4 defines $\text{ReadySet}(\mathcal{G})$ over `M0_Blocked` modules with hardened parents, but **specifies no invariant that every non-terminal module remains dispatchable or parked** — a module in any other micro-state (or parked incorrectly) is invisible to both the wave loop and the error path.
- **Design decision forced.** Liveness invariant as a validation contract: ∄ module in {M1, M2, M3} when ReadySet = ∅ and no event in flight ⇒ terminal `ModuleRetryExhausted`, with every invalid-report path parking its reporter. (Landed; encode as a checkable invariant, not prose.)

## G6. Harness interface is a first-class config plane (D3, D7, F4, F6)

- **Evidence.** Known open gap: `holon-evaluator`'s `CargoHarness` hardcodes `cargo check`/`cargo test` while configs declare `target_build_cmd`/`target_test_cmd` (`node -c translated.js`) that nothing consumes — skel candidates can never compile under the current harness, guaranteeing budget exhaustion; `require_byte_identical_io` unused; test-format per target language undecided.
- **Blind spot.** METHODOLOGY takes $\mathcal{H}_{\text{exec}}$ as an abstract input; nothing makes its interface (build cmd, test format, receipt schema, timeout policy) part of the architecture. The realized consequence: 17.1 h campaigns burning 774 requests on un-compilable candidates, and F4/F6 unchecked.
- **Design decision forced.** $\mathcal{H}_{\text{exec}}$ adapter trait consumed end-to-end (build/test/receipt schema per target language), wall-clock bounds enforced at the harness layer, receipts carried in $\mathcal{A}_{\text{artifacts}}$. Prerequisite for G4's single-ground-truth design.

## G7. Honesty gates and measurement validity — $S_0$ preflight (D10, F5, F6, F8)

- **Evidence.** Simulated receipts scored 87.5% pass in 65 ms/project; real inference: 12.5% in 3.2–17.1 h; 3/8 v18 runs OOM-killed leaving no JSONs; transport 500s ate restart caps.
- **Blind spot.** §0's preflight verifies the *legacy* harness compiles and INTENT is valid — nothing verifies the *campaign itself* is honestly measuring (real inference receipts, wall-clock bounds, resource headroom, terminal-integrity of persisted artifacts).
- **Design decision forced.** Campaign-level honesty criteria: `llm_requests > 0`, receipt provenance (real vs simulated) recorded and gated, artifact integrity checks at terminal (no mid-write truncation accepted as a measurement), concurrency/memory caps as operator contract.

## G8. Benchmark construction gaps the methodology cannot fix but must acknowledge (D5–D7, D10)

Zero SCC stress in 104 multi-file projects; 78/100 empty Cargo.tomls; dynamic-dispatch idioms (6,140 lines of function pointers, real Java reflection) present in sources but unscored; gcov/build-tree/`user_fix` contamination; micro-scale median (~1 k LoC). Consequences for the <...> program:

1. **This benchmark cannot validate Tier C ($H^*$, SCC condensation), multi-vertex wave scheduling at scale, or contract negotiation across real dependency edges** — those need validation contracts and/or a curated multi-module corpus (the SCC-free census is the reference point; `2dpartint`, `megalania`, `genetic-NN` are the deepest DAGs available, all acyclic).
2. **Success metrics must not be imported from it unexamined**: its compile/pass rates are oracle-relative (02), and its "repository-scale" claim rests on outliers.
3. The methodology's differentiating claims (non-monotonic recovery, warm-start regressions, adversarial guards) are precisely the ones this dataset never exercises — evaluate them on our own multi-module campaigns, not on ReCodeAgent's corpus.

## Priority order (evidence × cost)

| Rank | Gap | Why first |
|---|---|---|
| 1 | G6 harness configurability | Blocks any JS/other-target measurement at all; already-specified config knobs exist unconsumed |
| 2 | G4 reward↔gate single ground truth | Realized both directions (heapq, toml); invalidates campaign results silently |
| 3 | G3 budget/fallback totality | Realized as F1 across the whole binary lineage; cheap (v18 landed the pattern) |
| 4 | G1 oracle-strength gating | Directly determines whether $R=1.0$ means anything |
| 5 | G5 liveness invariant | Realized as F2; encode landed fixes as contracts |
| 6 | G7 honesty gates | Cheap; protects all other measurements |
| 7 | G2 translation-unit granularity | Deepest finding; largest design surface; needs its own ADR + corpus |
| 8 | G8 benchmark acknowledgment | Documentation-level; scopes claims in specs/ADRs |
