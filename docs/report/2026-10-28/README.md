# 2026-10-28 — ReCodeAgent Execution-Trace Analysis: Downsides the <...> Methodology Must Overcome

**Question.** What do ReCodeAgent's own artifacts reveal about the failure modes its architecture suffers, that the <...> methodology (`docs/references/METHODOLOGY.md`, `METHODOLOGY-DRV.md`) must be designed to overcome?

**Data ground truth.** The dataset at `benchmarks/assets/ReCodeAgent/data/` contains **no agent execution traces** (no rollouts, trajectories, LLM logs, or per-run JSONLs ship with it). What it does contain — and what this analysis is grounded in:

| Artifact class | Location | Size |
|---|---|---|
| Gold skeleton translations (targets full of `unimplemented!()`) | `data/tool_projects/{crust,oxidizer,alphatrans,skel}` | 118 projects, 208 MB |
| Foreign-execution I/O snapshots (the closest thing to per-call traces) | `data/oxidizer_mock_tests/*/exec-snapshots/*.json` | 274 files, 1,593 call records |
| Gold test suites + mock-harness source | `data/oxidizer_mock_tests/`, `data/oxidizer_translations/`, `tool_projects/crust/*/rust/src/bin/*.rs` | 180 `#[test]` binaries |
| Harness configs (oracle + budget semantics) | `benchmarks/assets/configs/ReCodeAgent/*/*.yml` | 118 configs |
| Real-campaign failure artifacts (our own daemon runs against this dataset) | `benchmarks/results/daemon/*.json`, `benchmarks/logs/*-fixverify.log`, `docs/report/2025-10-25/README.md` | 5 JSONs, 8 logs |

The "execution traces" of ReCodeAgent are therefore **reconstructed from three proxy layers**: what the dataset makes solvable/unsolvable (census), what its oracles can and cannot observe (oracle strength), and what actually happened when real inference ran against it (failure taxonomy). Each layer exposes independent downsides.

## The ten downsides (summary)

| # | Downside | Evidence (file) | Detail |
|---|---|---|---|
| D1 | **Single-call I/O snapshots cannot see stateful or error-path behavior** | `oxidizer_mock_tests/stats/exec-snapshots/github.com-montanaflynn-stats.SampleVariance.json` | 1,593 entries, one call each; 89 failing entries carry no error identity; no call sequences |
| D2 | **Oracles are second-hand and weaker than the source truth** | `oxidizer_translations/gonameparts/tests/…`, `crust/amp/rust/src/bin/test.rs` | Gold-suite-derived specs enshrine gold mistranslations (0.2 float tolerance); 97/180 crust test files have exactly one `#[test]`; some assert-free |
| D3 | **`require_byte_identical_io: false` on all 118 configs + exit-code-only test commands** | `configs/ReCodeAgent/skel/bst.yml` | `node translated.js` = single binary outcome; no per-test feedback reaches the repair loop |
| D4 | **Block-granularity translation unit shreds cross-block semantics** | `tool_projects/skel/*/javascript/translated.js` | 8–81 `BLOCK BEGIN/END` slices per project; late-bound `class_var`, dropped global block 0, cross-block exception/iteration/numeric contracts |
| D5 | **Every crust task is from-scratch synthesis, not translation** | `tool_projects/crust` | 3,090 `unimplemented!()` stubs across 284/583 `.rs` files; the gold ships only signatures + tests |
| D6 | **Zero SCC stress in 104 multi-file projects** | include/import graphs, Tarjan | 0 cycles; the methodology's Tier-C machinery (§5.3) has no empirical stress in this benchmark |
| D7 | **No package-manager-level dependency surface** | `crust/*/rust/Cargo.toml` | 78/100 empty `[dependencies]`; oxidizer ships six byte-identical 119-crate harness locks |
| D8 | **Engine failures on real runs are category-shape mismatches, not translation failures** | `benchmarks/results/daemon/skel-*.json` | 1 honest PASS / 5 FAIL / 3 OOM-killed; dominant killers were fabricated-counterparty conflicts and wave deadlocks on single-vertex DAGs |
| D9 | **Reward certification and acceptance gate use different ground truth** | `skel-heapq.json`, `skel-toml.json` | heapq: all 4 criteria green in workspace, reward never certified 1.0; toml: `test_pass_rate = 1.0` against an 18-line collapsed stub |
| D10 | **Honesty gap: simulated receipts vs real inference** | `benchmarks/results/recodeagent-benchmark-report.md` | skel 87.5% pass simulated → 12.5% real (1/8); 65 ms → 3.2–17.1 h per project |

Subfiles:

- [`01-dataset-census.md`](01-dataset-census.md) — coverage matrix, stub census, SCC/dynamic-construct reality, hygiene.
- [`02-oracle-strength.md`](02-oracle-strength.md) — the four oracle families and their quantified blind spots.
- [`03-skel-block-granularity.md`](03-skel-block-granularity.md) — per-project block coupling evidence.
- [`04-failure-taxonomy.md`](04-failure-taxonomy.md) — F1–F8 failure modes from the real campaigns.
- [`05-methodology-gaps.md`](05-methodology-gaps.md) — each downside mapped to the METHODOLOGY mechanism that must overcome it.

## Methodology mapping (condensed)

| Downside | METHODOLOGY.md mechanism that must overcome it | Gap to close |
|---|---|---|
| D1, D2, D3 | $\mathcal{A}_{\text{test}}$ (§4.2) seeds $\mathcal{B}_{CT}$; $\mathcal{A}_{\text{eval}}^{\text{distill}}$ (§4.2) computes $\text{Score}_{\text{harden}}$ | Nothing specifies **oracle strength**: a characterization suite with no error-path, sequence, or mutation coverage can pass a semantically wrong candidate at $R = 1.0$ |
| D4 | $\text{Hydrate}$ 1-hop $\mathcal{G}_{\text{slice}}$ (§2, Def 4); DAG vertices = "single file/module" | The benchmark's real translation unit is *sub-file blocks with cross-block state coupling*; vertex granularity must model intra-file slices + assembly contracts or hydration is blind to what breaks |
| D5 | ARIADNE per-DAG-vertex (§4) | Every crust vertex is a whole-library rewrite against typed scaffolding; $B_{\max}$-per-dispatch budgets and $\mathcal{A}_{\text{str}}$ hypothesis counts must be sized for synthesis difficulty, not patch difficulty |
| D6 | Tier C macro-regression, $H^*$ (§5.3) | No dataset stress exists; correctness of Tier C must come from validation contracts, not this benchmark |
| D7 | $S_4$ contract stubbing over edges (§3) | Third-party ecosystem translation (manifests, versions, FFI shims) is entirely absent from both benchmark and methodology |
| D8 | Tier B broadcast (§5.2); wave scheduler (§4) | **Proved by our runs**: parentless vertices + budget exhaustion must never route into conflict machinery; ReadySet must guarantee liveness (park, don't deadlock) |
| D9 | $R(s_t)$ gating (§4.2); `M4_Hardened` | Reward must be computed from the *same* execution facts the acceptance gate uses, or modules park/overrun as heapq did |
| D10 | $S_0$ preflight, honesty gates | Acceptance criteria must reject simulated/stub receipts; wall-clock and resource bounds are part of the harness contract |

## Reading order

`04-failure-taxonomy.md` is the empirical core (what actually breaks); `03-skel-block-granularity.md` is the deepest single finding about the dataset's translation unit; `05-methodology-gaps.md` is the actionable synthesis.
