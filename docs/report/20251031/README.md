# 20251031 Benchmark Report — werðan vs. ReCodeAgent on CRUST Systems Programming

## Executive Summary

This report documents the empirical evaluation of the **werðan** code modernization engine against the **ReCodeAgent** benchmark suite on systems-level C→Rust translation tasks. The central finding: **werðan achieved a 66.7% test pass rate on `crust.libwecan` where ReCodeAgent scored 0.0%**, using a local 27B Small Language Model on a single consumer GPU, powered by the DAEDALUS architecture and Frequent Intentional Compaction ([ADR-0018](../../decisions/0018-frequent-intentional-compaction.md)).

### Headline Result

| Metric | ReCodeAgent (Claude 3.5 Sonnet, Frontier API) | werðan (DAEDALUS + ADR-0018, Local 27B SLM) |
|:---|:---|:---|
| **Model** | Anthropic Claude 3.5 Sonnet | `swift-1.5-qwen3.8-27b` (RTX 3090) |
| **`crust.libwecan` Pass Rate** | **0.0%** (0 / 18 tests) | **66.7%** (12 / 18 tests) |
| **Compilation Status** | Broken C bindings, unsafe pointer panics | **Clean compilation** (`compilation_success: true`) |
| **Engine Wall Time** | Multi-iteration crash / deadlock | **358 seconds** (~6 minutes) |
| **LLM Requests** | Unbounded loop | **78 requests** |
| **Context Management** | Unbounded prompt growth (>32k tokens) | **Frequent Intentional Compaction** (≤12k Smart Zone) |
| **Hardware** | Cloud API (multi-billion parameter frontier) | **Single NVIDIA RTX 3090 (24 GB VRAM)** |

### Full Campaign Scorecard

| Project | ReCodeAgent | werðan | Compilation | Wall Time | LLM Requests | Outcome |
|:---|:---:|:---:|:---:|:---:|:---:|:---|
| **`crust.libwecan`** | 0.0% (0/18) | **66.7% (12/18)** | ✅ Clean | 358s | 78 | **Passed Validation Gate** |
| **`crust.cfsm`** | 0.0% (0/1) | 0.0% (0/1) | ❌ Harness flaw | 5,136s | 3,566 | Disqualified (flawed baseline) |
| **`crust.dict`** | 0.0% (0/6) | 0.0% (0/6) | ❌ API mismatch | 2,968s | 517 | Honest telemetry |
| **`crust.libvcd`** | 0.0% (0/1) | 0.0% (0/1) | ✅ Lib compiles | 487s | 51 | Honest telemetry |
| **`crust.skp`** | 33.3% (1/3) | 0.0% (0/3) | ❌ Header pressure | 1,165s | 39 | Honest telemetry |

---

## Why werðan Succeeded Where ReCodeAgent Failed

### The Core Problem: Context Explosion in Unconstrained Agent Loops

ReCodeAgent's architecture is an unconstrained conversational loop: a Planner LLM generates code, a Validator runs `cargo test`, and raw compiler error logs are appended verbatim to the conversation history. Each iteration adds 1,500–3,500 tokens of diagnostic boilerplate. By iteration 4, the context exceeds 28,000 tokens — well past the "Dumb Zone" where even frontier models suffer attention entropy collapse, repetitive generation loops, and hallucinated dependencies.

### The werðan Solution: Bounded Statechart MCTS with Frequent Compaction

werðan's DAEDALUS architecture replaces conversational memory with a formal Harel statechart (S0→S6) and Monte Carlo Tree Search (ARIADNE). Three mechanisms prevent context explosion:

1. **Semantic Diagnostic Filtering** (`compact_diagnostic_trace`): Strips raw compiler output to ≤2,048 characters, retaining only the top 3 primary errors.
2. **MCTS Rollout Isolation**: Each rollout operates on an isolated candidate clone. Failed attempts are pruned; only the best surviving state propagates.
3. **Blackboard State Vector**: Context size is O(1) with respect to iteration count, bounded by module size (<8k tokens).

On `crust.libwecan`, this meant the 27B SLM operated with maximal syntactic precision throughout 78 requests, achieving clean compilation and 66.7% test pass rate — outperforming Claude 3.5 Sonnet's 0.0% on the same task.

---

## Reading Order

1. [`01-cross-dataset-census.md`](01-cross-dataset-census.md) — Census of ReCodeAgent's CRUST failures and the architectural fragility that causes them.
2. [`02-compaction-in-practice.md`](02-compaction-in-practice.md) — The Smart Zone vs. Dumb Zone: why SLM context management is the critical determinant of success.
3. [`03-validation-receipts.md`](03-validation-receipts.md) — Detailed execution telemetry for each campaign, with forensic analysis.
4. [`04-architectural-implications.md`](04-architectural-implications.md) — Why bounded statechart MCTS outranks unconstrained agent loops for systems programming.
