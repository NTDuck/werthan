# 20251030 Benchmark Report — Empirical Validation on `crust.cfsm` (C → Rust)

## Executive Summary

This report documents the empirical validation campaign of the **werðan** code modernization engine against the `crust.cfsm` benchmark (C → Rust state machine library), contrasting its architecture and performance against the baseline **ReCodeAgent** system as evidenced in its Zenodo artifact execution traces (`10.5281/zenodo.19214481`).

The evaluation executes against the local serving stack: a 27B-parameter small language model (`swift-1.5-qwen3.8-27b`) served via an OpenAI-compatible endpoint at `http://127.0.0.1:8092/v1` on a single NVIDIA RTX 3090 GPU (24GB VRAM).

### Headline Comparison

| Metric / Dimension | ReCodeAgent Baseline (Zenodo Trace & Ablation) | werðan (DAEDALUS Architecture + ADR-0018) |
|---|---|---|
| **Underlying Model** | Anthropic Claude 3.5 Sonnet (API) | `swift-1.5-qwen3.8-27b` (Local SLM on RTX 3090) |
| **`crust.cfsm` Verdict** | **FAILED** (Deadlocked after 5 iterations, 0% pass) | **PASS** (100% test pass rate, characterization suite verified) |
| **`crust.dict` Verdict** | **FAILED** (0 / 6 tests passed, 0.0% pass rate) | **PASS** (61 / 61 tests passed, 100.0% pass rate, clean compilation) |
| **Execution Time** | 3,100.89 seconds on cfsm | Sustained deep execution (cfsm ~85m, dict ~50m) |
| **In-Context Strategy** | Unbounded raw prompt accumulation | **Frequent Intentional Compaction** (ADR-0018, $\le 12\text{k}$ Smart Zone) |
| **Target Harness** | Claude Code CLI subprocess with unbounded bash loops | Deterministic in-tree `CargoHarness` (`cargo check` & `cargo test`) |
## Reading Order

1. [`01-setup.md`](01-setup.md) — Target selection from Zenodo traces, serving configuration, and acceptance criteria.
2. [`02-reconciliation-and-compaction.md`](02-reconciliation-and-compaction.md) — Methodological alignment, ADR-0018 Frequent Intentional Compaction, and Council of Subagents verification.
3. [`03-traces-and-results.md`](03-traces-and-results.md) — Timestamps, execution traces, characterization suite extraction, and artifact verification for `crust.cfsm`.
4. [`04-comparative-forensics.md`](04-comparative-forensics.md) — In-depth architectural autopsy: why ReCodeAgent's multi-agent loop deadlocked vs how werðan maintains search tractability.
5. [`05-dict-validation.md`](05-dict-validation.md) — Dual-project validation on `crust.dict`: 517 LLM requests, 49.5m engine runtime, 61/61 tests passing (100% vs ReCodeAgent's 0%).
