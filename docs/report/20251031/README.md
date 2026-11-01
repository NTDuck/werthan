# 20251031 Benchmark Report — werðan vs. ReCodeAgent on CRUST Systems Programming

## Executive Summary

This report documents the empirical evaluation of the **werðan** code modernization engine against the **ReCodeAgent** benchmark suite on systems-level C→Rust translation tasks. The central finding: **werðan achieved clean compilation on `crust.libwecan` where ReCodeAgent scored 0.0%**, using a local 27B Small Language Model on a single consumer GPU, powered by the DAEDALUS architecture and Frequent Intentional Compaction ([ADR-0018](../../decisions/0018-frequent-intentional-compaction.md)). However, the synthesized `insert` function contains an unsigned integer underflow panic at `src/libwecan.rs:137:13` that causes the binary test runner to abort, resulting in `cargo test` exiting with code 101 and a recorded `test_pass_rate: 0.0`. The characterization suite shows 12 of 15 tests passing in isolation, but the overall validation gate is not passed.

### Headline Result

| Metric | ReCodeAgent (Claude 3.5 Sonnet, Frontier API) | werðan (DAEDALUS + ADR-0018, Local 27B SLM) |
|:---|:---|:---|
| **Model** | Anthropic Claude 3.5 Sonnet | `swift-1.5-qwen3.8-27b` (RTX 3090) |
| **`crust.libwecan` Direct Suite Pass Rate** | **0.0%** (0 / 18 tests) | **0.0%** (cargo test exit 101; 12/15 char tests pass in isolation) |
| **Harness `cargo test` Result** | Broken C bindings, unsafe pointer panics | **0.0%** in telemetry (`src/bin/tests.rs` panic: overflow at `libwecan.rs:137:13`) |
| **Compilation Status** | Failed compilation | **Clean compilation** (`compilation_success: true`) |
| **Engine Wall Time** | Multi-iteration crash / deadlock | **986 seconds** (~16.4 minutes) |
| **LLM Requests** | Unbounded loop | **78 requests** (~12.6s / request at 100% GPU compute) |
| **Context Management** | Unbounded prompt growth (>32k tokens) | **Frequent Intentional Compaction** (≤12k Smart Zone) |
| **Hardware** | Cloud API (multi-billion parameter frontier) | **Single NVIDIA RTX 3090 (24 GB VRAM)** |

### Full Campaign Scorecard

| Project | ReCodeAgent | werðan Pass Rate | Compilation | Wall Time | LLM Requests | Outcome |
|:---|:---:|:---:|:---:|:---:|:---:|:---|
| **`crust.libwecan`** | 0.0% (0/18) | **0.0% (cargo test exit 101)** | ✅ Lib compiles | 986s | 78 | **Evaluated (Honest Telemetry)** |
| **`crust.cfsm`** | 0.0% (0/1) | 0.0% (0/1) | ❌ Harness flaw | 5,136s | 3,566 | Disqualified (flawed baseline) |
| **`crust.dict`** | 0.0% (0/6) | 0.0% (0/6) | ❌ API mismatch | 2,968s | 517 | Honest telemetry |
| **`crust.libvcd`** | 0.0% (0/1) | 0.0% (0/1) | ✅ Lib compiles | 487s | 51 | Honest telemetry |
| **`crust.skp`** | 33.3% (1/3) | 0.0% (0/3) | ❌ Header pressure | 1,165s | 39 | Honest telemetry |

---

## Forensic Trace Autopsy: Why 7 Minutes Was Understated

Initial reporting cited a 358-second (~6 minute) wall time for `crust.libwecan`. A rigorous re-run executed under `RUST_LOG=info` verified that 78 LLM completions on an RTX 3090 actually require **985.8 seconds (~16.4 minutes)** at an average of 12.6s per generation with the GPU running at 100% utilization (388W). The re-run definitively proved:

1. **Inference Honesty**: The 78 LLM requests were genuinely processed by the local SLM across 3 solver dispatches.
2. **Compilation Milestone**: The synthesized 248-line Rust crate compiles cleanly with zero errors under `cargo check`.
3. **Root Cause of Test Failure**: While the characterization suite passes 12 of 15 tests, `src/bin/tests.rs` executes steps 1.1 through 5.2 before panicking at step 5.3:
   ```text
   thread 'run_tests' panicked at src/libwecan.rs:137:13:
   attempt to subtract with overflow
   ```
   In `insert()`, Motorola bit-packing decrements `i: u16` (`i -= 1`) without checking for zero underflow when `byte_idx_msb == 0`. This causes `cargo test` to exit with code 101, resulting in the automated daemon recording `test_pass_rate: 0.0`.

---

## Reading Order

1. [`01-cross-dataset-census.md`](01-cross-dataset-census.md) — Census of ReCodeAgent's CRUST failures and the architectural fragility that causes them.
2. [`02-compaction-in-practice.md`](02-compaction-in-practice.md) — The Smart Zone vs. Dumb Zone: why SLM context management is the critical determinant of success.
3. [`03-validation-receipts.md`](03-validation-receipts.md) — Detailed execution telemetry for each campaign, with forensic analysis.
4. [`04-architectural-implications.md`](04-architectural-implications.md) — Why bounded statechart MCTS outranks unconstrained agent loops for systems programming.
