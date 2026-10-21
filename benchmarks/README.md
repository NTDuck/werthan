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
  model: ashalliant/qwen3.8-27b-abliterated # Defaults to WERTHAN_MODEL
  max_parallel_shards: 2
  max_shards: 2
  inference_max_tokens: 32768
  inference_temperature: 0.2
  inference_timeout_secs: 600
  build_timeout_secs: 120
  test_timeout_secs: 120

verification:
  max_self_heal_retries: 3
  require_byte_identical_io: false

# Solver tuning
ariadne:
  max_rollouts: 16
```

### Workspace Root Constraint
- The daemon's `reset_workspace` requires `paths.workspace_root` to start with `benchmarks/outputs/`.
- `benchmarks/outputs/` is gitignored and serves as the scratchpad during transformation cycles.
- Final results are copied to `benchmarks/results/{yyyymmdd}/...` upon completion.

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
5. **Fixed Date Stamp (`{yyyymmdd}`):**
   - The timestamp `{yyyymmdd}` is fixed at the exact moment the benchmark run begins (not the shifting current wall clock during or after the run).
6. **Final Preserved Deliverables (Not Gitignored):**
   - **Translated Codebase:**
     ```
     benchmarks/results/{yyyymmdd}/{category}/{project}/{target_lang}/
     ```
   - **Metrics File:**
     ```
     benchmarks/results/{yyyymmdd}/{category}/{project}/results.yml
     ```
   - **Execution & Tracing Logs:**
     ```
     benchmarks/logs/{yyyymmdd}/{category}/{project}/
     ├── engine.log                 # Translation execution traces
     ├── inference.log              # Prompts, responses, and token counts
     └── harness.log                # Compiler and test runner output
     ```

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
  model: ashalliant/qwen3.8-27b-abliterated
  max_parallel_shards: 2
  max_shards: 2
  inference_max_tokens: 32768
  inference_temperature: 0.2
  inference_timeout_secs: 600
  build_timeout_secs: 120
  test_timeout_secs: 120
verification:
  max_self_heal_retries: 3
  require_byte_identical_io: false
ariadne:
  max_rollouts: 16
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
After completion, artifacts are exported to:
- **Translated JavaScript Codebase (Tracked in Git):**
  ```
  benchmarks/results/{yyyymmdd}/skel/bst/javascript/
  ```
- **Benchmark Metrics (`benchmarks/results/{yyyymmdd}/skel/bst/results.yml`):**
  ```yaml
  campaign_id: skel-bst-bench
  timestamp: "{yyyymmdd}"
  status: "PASSED"
  compilation_success: true
  test_pass_rate: 1.0
  wall_time_ms: 1245000
  engine_wall_time_ms: 1198000
  llm_requests: 42
  solver_dispatches: 1
  structural_retries: 0
  unimplemented_remaining: 0
  code_lines: 185
  cocomo_effort_pm: 0.42
  error: null
  ```
- **Diagnostic Logs:**
  ```
  benchmarks/logs/{yyyymmdd}/skel/bst/
  ├── engine.log
  ├── inference.log
  └── harness.log
  ```

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
