# Benchmark Guidelines & Execution Reference

This document defines the operational procedures, configuration schemas, artifact layouts, and execution parameters for running modernization benchmarks under `benchmarks/`.

---

## 1. Benchmark Suites & Datasets

The benchmark suite evaluates code modernization across diverse source and target language pairs:

| Category | Source Language | Target Language | Example Project |
| :--- | :--- | :--- | :--- |
| **`skel/`** | Python | JavaScript (`node`) | `skel/bst` (`python` -> `javascript`) |
| **`oxidizer/`** | Go | Rust (`cargo`) | `oxidizer/stats` (`go` -> `rust`) |
| **`alphatrans/`** | Java | Python (`pytest`) | `alphatrans/commons-cli` (`java` -> `python`) |
| **`crust/` / `swe-agent/`** | C | Rust (`cargo`) | `crust/vec` (`c` -> `rust`) |

### Dataset Paths
- **Pristine Source Datasets:**
  ```
  benchmarks/assets/ReCodeAgent/data/tool_projects/<category>/<project>/<source_lang>/
  ```
- **Campaign Configurations (118 Projects):**
  ```
  benchmarks/assets/configs/ReCodeAgent/<category>/<project>.yml
  ```

---

## 2. Configuration Schema

Campaign configurations are defined as YAML files conforming to the daemon's `BenchConfig` schema:

```yaml
campaign_id: <category>-<project>-bench
execution_mode: full_auto

paths:
  # Legacy source tree read by the campaign
  input_legacy_dir: benchmarks/assets/ReCodeAgent/data/tool_projects/<category>/<project>/<source_lang>
  # Target workspace scratchpad (MUST start with benchmarks/outputs/ for reset_workspace)
  workspace_root: benchmarks/outputs/<category>/<project>/<target_lang>
  off_limits_globs: []

stack:
  source_language: <source_lang>
  binary_artifacts: []
  use_ghidra_rea: false
  target_language: <target_lang>
  target_edition_or_framework: '2024'
  target_build_cmd: <build_command>
  target_test_cmd: <test_command>
  legacy_test_harness_cmd: <legacy_test_command>

inference:
  base_url: '' # Defaults to WERTHAN_BASE_URL (http://127.0.0.1:8092/v1)
  model: swift-1.5-qwen3.8-27b # Defaults to WERTHAN_MODEL
  max_parallel_shards: 2 # T1; scales to 6 with LoC tier (see table below)
  max_shards: 2 # T1; scales to 6 with LoC tier
  inference_max_tokens: 32768 # T1; scales to 49152 with LoC tier
  inference_temperature: 0.2
  inference_timeout_secs: 600 # T1; scales to 1800 with LoC tier
  build_timeout_secs: 120 # T1; scales to 480 with LoC tier
  test_timeout_secs: 120 # T1; scales to 480 with LoC tier

verification:
  max_self_heal_retries: 3 # T1; scales to 8 with LoC tier
  require_byte_identical_io: false

# Solver tuning
ariadne:
  max_rollouts: 16
```

### LoC-Tier Inference Scaling

All 118 configs are retuned per legacy codebase size so larger projects get
longer horizons and more solver effort. LoC counts every source file in
`paths.input_legacy_dir` (`.py`, `.c`, `.go`, `.java`, `.rs`, `.js`, `.ts`, `.h`).

| Tier | LoC range | Projects | max_rollouts | shards (par/max) | inference_max_tokens | inf t/o (s) | build/test t/o (s) | max_self_heal_retries |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| T1 | <= 500 | 24 | 16 | 2 / 2 | 32768 | 600 | 120 / 120 | 3 |
| T2 | 501 - 2000 | 61 | 32 | 4 / 4 | 40960 | 900 | 180 / 180 | 4 |
| T3 | 2001 - 10000 | 28 | 64 | 6 / 6 | 49152 | 1200 | 300 / 300 | 6 |
| T4 | > 10000 | 5 | 96 | 6 / 6 | 49152 | 1800 | 480 / 480 | 8 |

Tier tuning rationale against the serving stack (ninfer 0.15.0-rtx3090, RTX
3090 24GB, `total_slots: 6`, context window 65536, ~268 tok/s with draft
decoding):

- **max_tokens <= 49152 (75% of ctx):** leaves >= 16k tokens for prompt,
  rule cards, and reasoning traces in the 65k context window.
- **max_parallel_shards <= 6:** caps concurrent requests at the server's
  continuous-batching slot count; requesting more than 6 slots is wasted.
- **max_rollouts scales the MCTS budget per module:** the dominant wall-time
  driver (`ariadne.max_rollouts` feeds `holon_engine::EngineConfig::max_rollouts`).
- **inference/build/test timeouts scale with problem size** so big projects
  do not fail spuriously, and per-project wall time stays under the 2-hour
  anomaly bound in section 5.

### Workspace Root Constraint
- The daemon's `reset_workspace` requires `paths.workspace_root` to start with `benchmarks/outputs/`.
- `benchmarks/outputs/` is gitignored and serves as the scratchpad during transformation cycles.
- Result JSON lands in `BENCH_RESULTS_DIR` (default `benchmarks/results/daemon`) upon completion.

---

## 3. Workflow Specifications

### 3.1 Generic Specification for Any Random Project

For any arbitrary modernization target `{category}/{project}` with `{source_lang}` -> `{target_lang}`:

1. **Source Code Location:**
   ```
   benchmarks/assets/ReCodeAgent/data/tool_projects/{category}/{project}/{source_lang}/
   ```
2. **Configuration File:**
   ```
   benchmarks/assets/configs/ReCodeAgent/{category}/{project}.yml
   ```
3. **Execution Scratchpad (Intermediate):**
   ```
   benchmarks/outputs/{category}/{project}/{target_lang}/
   ```
   *(Note: This path is gitignored and reset prior to run from the pristine dataset).*
4. **Execution Command:**
   ```bash
   cargo run -p daemon --release -- --benchmark benchmarks/assets/configs/ReCodeAgent/{category}/{project}.yml
   ```
5. **Final Preserved Deliverables (Not Gitignored):**
   - **Translated Codebase:**
     ```
     benchmarks/results/daemon/<label>.json
     ```
     (See §5 for the result artifact layout: per-project result JSON and batch `summary.json`, labeled from the `workspace_root` path after `outputs/`.)

---

### 3.2 Concrete Walkthrough: `skel/bst` (Python -> JavaScript)

This concrete example walks through migrating `skel/bst`:

#### Project Attributes
- **Category:** `skel`
- **Project:** `bst` (Binary Search Tree)
- **Source Language:** Python (`python`)
- **Target Language:** JavaScript (`javascript`, executed via `node`)

#### 1. Input Source Code
The pristine Python legacy codebase is located at:
```
benchmarks/assets/ReCodeAgent/data/tool_projects/skel/bst/python/source.py
```

#### 2. Configuration File (`benchmarks/assets/configs/ReCodeAgent/skel/bst.yml`)
```yaml
campaign_id: skel-bst-bench
execution_mode: full_auto
paths:
  input_legacy_dir: benchmarks/assets/ReCodeAgent/data/tool_projects/skel/bst/python
  workspace_root: benchmarks/outputs/skel/bst/javascript
  off_limits_globs: []
stack:
  source_language: python
  binary_artifacts: []
  use_ghidra_rea: false
  target_language: javascript
  target_edition_or_framework: '2024'
  target_build_cmd: node -c translated.js
  target_test_cmd: node translated.js
  legacy_test_harness_cmd: pytest
inference:
  base_url: ''
  model: swift-1.5-qwen3.8-27b
  max_parallel_shards: 4
  max_shards: 4
  inference_max_tokens: 40960
  inference_temperature: 0.2
  inference_timeout_secs: 900
  build_timeout_secs: 180
  test_timeout_secs: 180
verification:
  max_self_heal_retries: 4
  require_byte_identical_io: false
ariadne:
  max_rollouts: 32
```

#### 3. Invocation
```bash
export WERTHAN_BASE_URL="http://127.0.0.1:8092/v1"
export WERTHAN_MODEL="swift-1.5-qwen3.8-27b"
cargo run -p daemon --release -- --benchmark benchmarks/assets/configs/ReCodeAgent/skel/bst.yml
```

#### 4. Scratchpad Workspace (Intermediate)
During execution, the daemon compiles and tests the JavaScript code inside:
```
benchmarks/outputs/skel/bst/javascript/
```

#### 5. Expected Output Artifacts (Post-Run)
After completion, the result artifact lands at:
```
benchmarks/results/daemon/skel-bst-javascript.json
```
(the label derives from the `workspace_root` segments after `outputs/`; see §5 for the full `BenchResult` field list and §2 for the workspace reset semantics). Execution traces stream to stdout via the tracing subscriber.

---

## 4. Execution Commands

### 4.1 Inference Server Setup
Local GPU reasoning runs on port `8092`:
```bash
export WERTHAN_BASE_URL="http://127.0.0.1:8092/v1"
export WERTHAN_MODEL="swift-1.5-qwen3.8-27b"
```

### 4.2 Single Project Execution
```bash
cargo run -p daemon --release -- --benchmark benchmarks/assets/configs/ReCodeAgent/skel/bst.yml
```

### 4.3 Batch Execution
```bash
cargo run -p daemon --release -- --benchmark-all benchmarks/assets/configs/ReCodeAgent/
```

---

## 5. Execution Time Bounds & Anomaly Detection

- **Lower Bound (duration > 10 min):** Real campaigns requiring rule extraction, interface stubbing, code translation, and test repair typically take over 10 minutes. A completion under 10 minutes indicates bypassed inference or empty stubs.
- **Upper Bound (duration < 2 hr):** Runs exceeding 2 hours indicate infinite repair loops or deadlock. The daemon times out and records an error if this limit is reached.
- **Expected Window:** 10 minutes to 2 hours per project under active GPU inference.

---

## 6. Success Criteria

1. **Compilation:** Clean compilation with target toolchain (`compilation_success: true`).
2. **Behavioral Parity:** 100% test pass rate (`test_pass_rate == 1.0`).
3. **No Unimplemented Stubs:** `unimplemented_remaining == 0`.
4. **Inference Honesty:** `llm_requests > 0`.
