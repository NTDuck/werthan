# 1 · Experimental Setup & Project Selection

## 1.1 Empirical Project Selection from ReCodeAgent Zenodo Traces

To rigorously benchmark werðan against ReCodeAgent under real-world constraints, the target project was selected directly by forensic scanning of ReCodeAgent's primary Zenodo run artifacts (`10.5281/zenodo.19214481`, unpackaged under `benchmarks/assets/ReCodeAgent-zenodo/results-unpacked/results/trajectories/`).

Out of 118 translation projects in the dataset, 13 projects exhibited catastrophic non-convergence in ReCodeAgent's execution traces. Among these, `crust.cfsm` (translating a C finite state machine library to Rust) represents a classic failure mode of unconstrained LLM agent loops:

- **Directory**: `data/tool_projects/crust/cfsm` (C legacy: `c/src/c_fsm.c`, 111 LoC; target Rust: `rust/src/cfsm.rs`, 27 LoC).
- **ReCodeAgent Outcome**: `validation_passed: False`, `iterations: 5` (exhausted maximum translation-validation iteration limit).
- **Execution Time**: 3,100.89 seconds (51.6 minutes) of Anthropic Claude 3.5 Sonnet token burn.
- **Root Cause in ReCodeAgent**: The agent's Translator repeatedly failed to reconcile Rust lifetime and ownership semantics in state transition callbacks (`Option<fn(fsm: Option<Box<&mut CfsmCtx<T>>>)>`), entering an infinite loop of non-return where the Validator Agent continuously flagged `Status: FAIL (1 critical bug found)`.

## 1.2 Serving Stack and Hardware Invariants

The evaluation was conducted strictly on consumer-grade local hardware without frontier cloud model fallbacks:

| Parameter | Specification |
|---|---|
| **GPU Hardware** | 1× NVIDIA GeForce RTX 3090 (24 GB GDDR6X VRAM) |
| **Model** | `swift-1.5-qwen3.8-27b` (27B parameter SLM, abliterated) |
| **Serving Framework** | OpenAI-compatible local HTTP endpoint |
| **Endpoint Address** | `http://127.0.0.1:8092/v1` |
| **Context Window** | 65,536 tokens |
| **Inference Concurrency** | Dedicated single-card serial rollout pipeline |

## 1.3 Campaign Configuration

The campaign configuration is strictly quarantined under `benchmarks/assets/configs/ReCodeAgent/crust/cfsm.yml`:

```yaml
campaign_id: crust-cfsm-bench
execution_mode: full_auto
paths:
  input_legacy_dir: benchmarks/assets/ReCodeAgent/data/tool_projects/crust/cfsm/c
  workspace_root: benchmarks/outputs/crust/cfsm/rust
  off_limits_globs: []
stack:
  source_language: c
  binary_artifacts: []
  use_ghidra_rea: false
  target_language: rust
  target_edition_or_framework: '2024'
  target_build_cmd: cargo check
  target_test_cmd: cargo test
  legacy_test_harness_cmd: make test
inference:
  base_url: ''
  model: swift-1.5-qwen3.8-27b
  max_parallel_shards: 6
  max_shards: 6
  inference_max_tokens: 49152
  inference_temperature: 0.2
  inference_timeout_secs: 1200
  build_timeout_secs: 300
  test_timeout_secs: 300
verification:
  max_self_heal_retries: 6
  require_byte_identical_io: false
ariadne:
  max_rollouts: 64
```

## 1.4 Acceptance Criteria

Following `benchmarks/README.md` §6 and methodology validation contracts:
1. **Compilation**: Clean compilation with `cargo check` (`compilation_success == true`).
2. **Behavioral Parity**: Execution of test harnesses with `cargo test` (`test_pass_rate > 0.0`).
3. **No Stub Collapse**: Clean eradication of all `unimplemented!()` stubs (`unimplemented_remaining == 0`).
4. **Inference Honesty**: Positive non-zero LLM request count (`llm_requests > 0`).
