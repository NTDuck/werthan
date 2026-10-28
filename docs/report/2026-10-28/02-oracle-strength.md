# 02 — Oracle Strength: What the Verification Families Can and Cannot See

Four oracle families verify a translation. Each has quantified blind spots that a repair loop can exploit.

## Family 1 — Foreign-execution I/O snapshots (`oxidizer_mock_tests/*/exec-snapshots/*.json`)

Shape: 274 files, 1,593 entries across 6 projects (checkdigit 25, go-edlib 42, gohistogram 13, gonameparts 15, stats 123, textrank 56). Each entry has exactly four fields:

```json
{"inputs": [{}, "424242424242424"], "execution_success": true,
 "input_modifications": [], "return_value": 2}
```
(`oxidizer_mock_tests/checkdigit/exec-snapshots/(*…luhn).Generate.json`)

Can see: single-call return value, success boolean, shallow post-call dump of receiver/args.

Structurally cannot see:

- **Error identity.** Only 56 files contain any `execution_success: false`; 89/1,593 entries (5.6%) fail, all as `"return_value": null` with no exception type, message, or failure branch. A translation that fails for the *wrong reason* matches.
- **Call sequences.** Every entry is one isolated call against a freshly serialized receiver (`NumericHistogram.Add.json` entry 1: `inputs: [{"bins": [], "maxbins": 160, "total": 0}, …]`). A translation that corrupts `bins` on the *second* `Add` is invisible; Add→Add→Quantile state chains are decomposed into independent snapshots.
- **Side-effect ordering, timing, resource use**: nothing recorded.
- **`input_modifications` is a weak mutation proxy**: only 258/1,593 entries have any; it is a whole-struct equality dump — unintended extra-field changes fail, in-place-equivalent mutations of unobserved fields pass.
- **Float semantics are encoded as IEEE-754 bit patterns read as integers**: stats snapshot `4611686018427387904 → 2.0`, `9221120237041090561 → NaN`, `"AllowedNaN": true` in the data. Exact-equality oracles over these patterns can pass NaN-conforming / precision-drifting translations or fail numerically-correct ones.
- **Coverage ceiling**: `interoperation.json` maps 39 checkdigit symbols but only 25 have snapshots — the oracle covers ≤ ~64% of the API surface in the best project. `rust_pieces.json` is one hand-written example function — a smoke script, not an oracle.

## Family 2 — Gold test suites (`oxidizer_translations/*/tests/*.rs`)

| project | `#[test]` | asserts | error-path refs |
|---|---|---|---|
| checkdigit | 36 | 55 | 6 |
| go-edlib | 26 | 44 | 9 |
| gohistogram | 2 | 11 | 0 |
| gonameparts | ~0 usable | 0 | 0 |
| stats | 155 | 301 | 70 |
| textrank | 8 | 51 | 1 |

Structural downside: **they test the gold translation, not the legacy source — the oracle is second-hand**. Headers say `// Translation of mean_test.go`, `// Translated from GO describe_test.go`. If the gold mis-translated a Go semantic, the wrong behavior is enshrined as truth:

- `gohistogram/tests/histogram_test.rs:15-18`: `fn approx(x,y) … (x - y).abs() < 0.2` — a 0.2 absolute tolerance on quantiles hides drift.
- `gonameparts`: every Go internal test rewritten through the public `parse()` (*"since nameString is internal"*, `nameparts_test.rs:3-5`); `TestObviouslyBadName` is `let _ = parse("I am a Popsicle");` — an assert-free panic guard.
- Error-branch coverage exists only where Go tests had it (stats' `ERR_EMPTY_INPUT` string-equality assert); gohistogram 0, textrank 1.
- Exact `assert_eq!` on f64 in most stats tests — brittle across rounding.

## Family 3 — crust ported test binaries (`crust/*/rust/src/bin/{test,tests,…}.rs`)

180 `#[test]`-bearing files across ~100 projects; **97 contain exactly one `#[test]`**; canonical pattern is one `#[test] fn test()` + `fn main(){}` (`crust/amp/rust/src/bin/test.rs`).

- All-or-nothing signal: one test fn = no per-case feedback.
- Several are **assert-free** (`bostree/rust/src/bin/remove_bug.rs` 1 test/0 asserts; `cissy/rust/src/bin/test_csvfield.rs`; `libfor/rust/src/bin/test.rs` has 0 `#[test]`, only a local `verify()` helper) — pass = don't panic.
- Weak hygiene ported from C: `mvptree/rust/src/bin/testmvp.rs` uses `rand::thread_rng()`, `unsafe { NBCALCS += 1 }`, *prints* `FOUND --> …` instead of asserting, and writes `testfile.mvp` to CWD (order-dependent side effects).
- `fn main(){}` duplication: `cargo test` runs the asserts but the invoked *binary* runs nothing.
- Coverage is happy-path, ported from `test.c` ("Transpiled from test.c", `libpsbt`), occasionally one error branch the C author bothered with.
- Float asserts use `f32::EPSILON` on composite expressions (`expr/rust/src/bin/test.rs`) — flaky under reordering.

## Family 4 — Harness configs (`benchmarks/assets/configs/ReCodeAgent/*/*.yml`, 118 files)

| knob | values | consequence |
|---|---|---|
| `target_test_cmd` | `cargo test` ×106, `node translated.js` ×8 (skel), `pytest` ×4 | skel "pass" = one script's exit code; **zero per-test granularity** |
| `require_byte_identical_io` | **false ×118** | the snapshot oracle's exactness is waived everywhere, by default |
| `max_rollouts` | 16 (24 configs), 32 (61), 64 (28), 96 (5) | budget scales with category, not difficulty |
| `max_self_heal_retries` | 3/4/6/8, 1:1 with rollouts | retries are blind without per-test signal |
| `test_timeout_secs` | 120–480 | conflates slow-but-correct with wrong; an opaque timeout is the only feedback for a hang |

## Consequences for a repair loop

1. **Passing oracle with wrong semantics** is cheap: honor the snapshot encoding conventions (bit-pattern floats, `AllowedNaN`), fail-for-the-wrong-reason on error paths, or conform to gold's own tolerance bugs.
2. **No targeted feedback**: exit-code oracles + single-`#[test]` binaries mean self-heal retries (3–8) are blind re-rolls; the loop burns rollouts without convergence information.
3. **Stateful regressions are invisible** to single-call snapshots; mutation coverage is 16% of entries.
4. **Error-path blindness**: a repair that converts every failure to a panic or wrong `Result` variant passes nearly everywhere — the gradient points toward "fails somehow", not "fails correctly".
5. **Reward-hacking surface is large and legal**: deleting asserts, stripping `unwrap()`s, and not panicking are the highest-expected-value "repairs" under these oracles.
