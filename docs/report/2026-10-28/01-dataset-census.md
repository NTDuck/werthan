# 01 — Dataset Census: What the Benchmark Actually Contains

Base: `benchmarks/assets/ReCodeAgent/data/tool_projects/` (118 projects, 4 categories) plus sibling `oxidizer_mock_tests/` / `oxidizer_translations/`.

## 1. Coverage matrix

| Category | Projects | Pair | Src files / LoC | Target files / LoC | Build system | Dep manifest | Test harness |
|---|---|---|---|---|---|---|---|
| crust | 100 | C→Rust | 901 (.c+.h) / 168,132 | 583 .rs / 38,110 | 93 Makefile, 54 CMakeLists.txt | 100 Cargo.toml (78 empty `[dependencies]`) | C: test dirs in all 100; Rust: 180 `#[test]` bin files, `#[cfg(test)]` in only 6 projects |
| oxidizer | 6 | Go→Rust (FFI-mock) | 137 .go / 28,133 | 96 .rs / 19,026 | 6 go.mod | 6 identical 119-crate Cargo.locks (harness: serde, bolero, arbitrary, rug…) | 60 `_test.go`; Rust tests live outside in `oxidizer_translations/*/tests/` |
| alphatrans | 4 | Java→Python | 257 .java / 59,401 | 162 .py / 15,411 | 4 pom.xml | Maven-managed, none in dataset | 121 Java test files; Python `pytest.ini` points at `src/test` **which does not exist**; only a conftest.py |
| skel | 8 | Python→JS | 12 .py / 9,541 | 13 .js / 8,127 | none | none | none in dataset (external harness; `target_test_cmd: node translated.js`) |

LoC distribution (source side): crust min 125 / median 1,071.5 / max 29,489 (libfor — but 28,240 of those lines are the generated `for-gen.c`; hand-written `for.c` is 514); oxidizer 818–15,470; alphatrans 6,669–28,110; skel 355–2,092. The dataset is micro-scale with 3–4 outliers.

## 2. Stub census — crust is a synthesis benchmark

- **3,090 `unimplemented!()` macro occurrences** across 284 of 583 `.rs` files; 289 files mention `unimplemented` in total (the extra 5 are prose references in test comments/prints, e.g. `xopt/rust/src/bin/test.rs:107`).
- Top 20 by stub count: libfor 415/432 fns (96%), tisp 102/105 (97%), kairocompiler 105/112 (94%), recordmanager 86/91 (95%), linear-algebra-c 168/193 (87%), impcheck 100/117 (85%), fslib 131/235 (56%), lambda-calculus-eval 72/95 (76%), libutf 69/88 (78%), skp 67/75 (89%), c-blind-rsa-signatures 48/52 (92%), ltre 63/69 (91%), jccc 56/73 (77%), libpgn 58/87 (67%), megalania 53/79 (67%), genetic-NN 64/110 (58%), c-aces 51/69 (74%), cset 46/64 (72%), blt 44/56 (79%), libm17 39/54 (72%).
- In 96/100 projects the only stub-free files are ported test suites (`src/bin/test*.rs`) and 0–4-line `lib.rs` re-export shims. **Only 2 projects have any implemented library logic** (`bostree/rust/src/lib.rs` invariant-check traversal, `fslib/rust/src/lib.rs` assert helpers).
- Head scaffolding is near-zero-doc-comment: `use` + derives + struct/enum scaffolding, then walls of signature-`unimplemented!()` bodies (~15–30% scaffolding lines). Typed idiomatic models exist (14 type decls in tisp, `Result<>` errors, `HashMap` instead of `void*`) — a human authored the type layer, the behavior layer is entirely absent.
- Consequence: a crust "translation" is a **whole-library reimplementation against a pre-authored type model**. There is no partial-credit scaffold; per-function difficulty is unbounded above the signature.

Oxidizer is the inverse profile: 6 projects, **0 stubs** in `tool_projects` — fully-implemented gold translations (verified: `luhn.rs generate()` has the real Luhn loop). Its "stubs" are `extern` reroute declarations (364 `__foreign` + 382 `__with_callees_mocked` variants in `oxidizer_mock_tests`), i.e. a **mocking/verification** benchmark, disjoint from crust's generation benchmark.

## 3. SCC reality: zero cycles anywhere

Tarjan over all 100 crust and all 4 alphatrans include/import graphs: **0 cycles (max SCC = 1)**. Real multi-file graphs exist (genetic-NN 94 edges/52 files, megalania 76/44, lambda-calculus-eval 56/23) but are hub-and-spoke DAGs; libm17's 42 `.c` files have only 3 include edges (Unity test headers only). alphatrans commons-csv: 35 files, 34 intra-project edges, 0 cycles.

Dynamic constructs that would stress static DAGs, per source-side grep: function pointers in 6,140 lines across 182 files (dominant idiom), `goto` 870 lines/20 files, `setjmp/longjmp` 9 files (8 are committed Unity/CuTest test frameworks; only `crust/skp/c/src/skp.h` is real code), `dlopen/dlsym` 1 file (`crust/tisp/c/tib/io.c`), variadics 32 files, genuine Java reflection in mains (`TypeHandler.java:42,115` `Class.forName` + `:121` `newInstance()`; `ValidatorAction.java:598`). The target sides are ~4.4× (crust) and ~3.8× (alphatrans) smaller than sources; **how these constructs were translated is nowhere recorded and nowhere scored**.

## 4. Dataset hygiene (measurement contamination risks)

Committed into the "source" trees: 144 `.gcno`, 48 `.gcda`, 69 `.gcov` (annotated coverage listings, e.g. `crust/bostree/c/bostree.c.gcov`, spanning 23 projects), 86 `.o`, 4 `.a`, 6 `.so`, stale binaries, committed build trees in 6 projects (fleur, graph-recogniser, kairocompiler, libm17, libtinyfseq, mdb) with `CMakeCache.txt`, `coverage_report.json` inside `42-kocaeli-printf/c/`, 17 `.c_old` mutated leftovers, prebuilt `libgointeroperation.so` in every oxidizer project.

skel side files are **manual patches booked in the evaluation ledger**: `skel/heapq/javascript/tracer_skip.js:1-2` — *"This is a patch for 2 fragments in the translated code. They are counted as 'user_fix' in evaluation"* (a hand-written polyglot `compare()`); `skel/strsim/javascript/tracer_skip.js` hoists CJK literals the tooling couldn't parse; `skel/mathgen/javascript/tracer_skip.js:2` requires the `numjs` package (not in any manifest — `require('./translated.js')` fails with `Cannot find module 'numjs'`, the only skel artifact that cannot even load). skel results partially reflect manual patching, not pure translation.

## 5. Census conclusions

1. The benchmark measures **synthesis against typed scaffolding** (crust) and **mock-verified refactoring** (oxidizer) — two disjoint skills — plus two micro-scale smoke categories (skel, alphatrans).
2. No circular dependencies, no multi-module SCC stress, no third-party ecosystem translation anywhere.
3. The dynamic-dispatch idioms that dominate legacy C (function pointers) are present in sources but invisible in scoring.
4. Hygiene artifacts (coverage listings, build trees, `user_fix` patches) mean "source" sides are not pristine inputs.
5. Scale claims (repository-level) rest on 3–4 outliers, two of which are bulk-generated or bulk-boilerplate (`for-gen.c`, oxidizer harness locks).
