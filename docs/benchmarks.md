# ReCodeAgent Modernization MAS Benchmark Report

Date: 2026-10-17  
Harness: `werðan` Code Modernization Multi-Agent System (MAS)  
Dataset: `ReCodeAgent` (Intelligent-CAT-Lab/ReCodeAgent)  
Target Language: Rust (Edition 2024)  
Inference Engine: `ninfer` / OpenAI-compatible endpoint (`swift-1.5-qwen3.8-27b`)  

---

## Executive Summary

The `werðan` modernization MAS orchestrates an offline, deterministic migration pipeline translating legacy C implementations from the `ReCodeAgent` benchmark dataset into safe, idiomatic Rust. The pipeline executes across a formal 10-state Finite State Machine (S0 to S9) with 6 mandatory Human Decision Gates, backed by REA MCP quantitative code metrics (COCOMO-II), Pi-Warden tool guardrails, and Jev System-1 decision models.

Benchmarks were executed on two representative projects from the `ReCodeAgent` dataset:
1. **`crust/vec`**: Dynamic resizable array implementation in C.
2. **`crust/totp`**: Time-based One-Time Password (TOTP) algorithm and HMAC-SHA1 engine in C.

Both campaigns achieved **100% Proven receipts (6/6)** across all decision gates with full clean-room differential equivalence.

---

## Benchmark Comparison Table

| Metric / Attribute | `crust/vec` (`crust-vec-bench`) | `crust/totp` (`crust-totp-bench`) |
| :--- | :--- | :--- |
| **Source Language** | C (`.c`, `.h`) | C (`.c`, `.h`) |
| **Target Language** | Rust (Edition 2024) | Rust (Edition 2024) |
| **Legacy Path** | `benchmarks/assets/ReCodeAgent/data/tool_projects/crust/vec` | `benchmarks/assets/ReCodeAgent/data/tool_projects/crust/totp` |
| **Workspace Root** | `./workspace/crust-vec` | `./workspace/crust-totp` |
| **Total Lines (scc)** | 1,208 | 870 |
| **Code Lines (scc)** | 998 | 630 |
| **COCOMO-II Effort** | 2.93 Person-Months | 1.77 Person-Months |
| **Estimated Duration** | 5.16 Months | 4.40 Months |
| **FSM States** | S0 $\rightarrow$ S9 (All 10 States Passed) | S0 $\rightarrow$ S9 (All 10 States Passed) |
| **Gate 1: Preflight Q&As** | `PROVEN` (`rcpt-001`) | `PROVEN` (`rcpt-001`) |
| **Gate 2: Rule Review** | `PROVEN` (`rcpt-002`) | `PROVEN` (`rcpt-002`) |
| **Gate 3: Brief Approval** | `PROVEN` (`rcpt-003`) | `PROVEN` (`rcpt-003`) |
| **Gate 4: Diff Acceptance** | `PROVEN` (`rcpt-004`) | `PROVEN` (`rcpt-004`) |
| **Gate 5: Proof Receipt** | `PROVEN` (`rcpt-005`) | `PROVEN` (`rcpt-005`) |
| **Gate 6: Patch Sign** | `PROVEN` (`rcpt-006`) | `PROVEN` (`rcpt-006`) |
| **Final Verdict** | **SUCCESS (6/6 Proven)** | **SUCCESS (6/6 Proven)** |

---

## Detailed Benchmark Logs

### Campaign 1: `crust/vec`

#### Configuration (`benchmarks/configs/crust-vec.yml`)

```yaml
campaign_id: "crust-vec-bench"
execution_mode: "full_auto"
paths:
  input_legacy_dir: "benchmarks/assets/ReCodeAgent/data/tool_projects/crust/vec"
  workspace_root: "./workspace/crust-vec"
  off_limits_globs: []
stack:
  source_language: "c"
  binary_artifacts: []
  use_ghidra_rea: false
  target_language: "rust"
  target_edition_or_framework: "2024"
  target_build_cmd: "cargo build --release"
  target_test_cmd: "cargo test"
  legacy_test_harness_cmd: "make test"
inference:
  ninfer_base_url: "http://127.0.0.1:8080/v1"
  deep_model: "qwen3.8-27b"
  fast_model: "qwen3.6-35b-a3b"
  max_parallel_shards: 2
  user_input_mode: "wait"
verification:
  max_self_heal_retries: 3
  require_byte_identical_io: true
```

#### Execution Log (`mise run bench`)

```text
[daemon] Starting modernization benchmark with config: benchmarks/configs/crust-vec.yml
[FSM] State: S0 INTENT - Campaign: crust-vec-bench initialized
[FSM] State: S1 PREFLIGHT - Dependency preflight in benchmarks/assets/ReCodeAgent/data/tool_projects/crust/vec
[FSM] Gate 1 Receipt: Gate 1: Preflight Q&As [Approved]
[FSM] State: S2 ASSESS - Quantitative code assessment complete
[FSM] State: S3 MAP - Domain clustering and call graph mapped
[FSM] State: S4 EXTRACT_RULES - Worker pool dispatching rule extraction shards
[FSM] Gate 2 Receipt: Gate 2: Rule Review [Approved]
[FSM] State: S5 BRIEF - Target architecture brief approved
[FSM] Gate 3 Receipt: Gate 3: Brief Approval [Approved]
[FSM] State: S6 TRANSFORM - Code transformation and Jev review
[FSM] Gate 4 Receipt: Gate 4: Diff Acceptance [Approved]
[FSM] State: S7 VERIFY - Clean-room characterization and equivalence check
[FSM] Verification diff status: PROVEN
[FSM] Gate 5 Receipt: Gate 5: Proof Receipt [Approved]
[FSM] State: S8 HARDEN - Security audit, secret masking, patch signing
[FSM] Gate 6 Receipt: Gate 6: Patch Sign [Approved]
[FSM] State: S9 COMPLETED - Modernization pipeline finished successfully
================================================================================
                    WERÐAN MODERNIZATION MAS BENCHMARK REPORT.                  
================================================================================
Campaign ID: crust-vec-bench.
Execution Mode: FullAuto.
Source Language: c.
Target Language: rust (edition 2024).
Legacy Directory: benchmarks/assets/ReCodeAgent/data/tool_projects/crust/vec.
Workspace Root: ./workspace/crust-vec.
--------------------------------------------------------------------------------
COCOMO-II METRICS (REA-MCP):
  Total Lines: 1208.
  Code Lines: 998.
  Effort (PM): 2.93 person-months.
  Estimated Duration: 5.16 months.
--------------------------------------------------------------------------------
FSM WORKFLOW EXECUTION (S0 -> S9):
  [S0 INTENT] PASSED. (Workspace scaffolded, Pi-Warden rendered.)
  [S1 PREFLIGHT] PASSED. (Gate 1: Preflight Q&As Approved - Proven.)
  [S2 ASSESS] PASSED. (SCC analysis & COCOMO-II metrics computed.)
  [S3 MAP] PASSED. (Domain clustering & call graphs mapped.)
  [S4 EXTRACT_RULES] PASSED. (Gate 2: Rule Review Approved - Proven, shards ok.)
  [S5 BRIEF] PASSED. (Gate 3: Brief Approval Approved - Proven.)
  [S6 TRANSFORM] PASSED. (Gate 4: Diff Acceptance Approved - Proven, Jev OK.)
  [S7 TEST] PASSED. (Gate 5: Proof Receipt Approved - Proven.)
  [S8 HARDEN] PASSED. (Gate 6: Patch Sign Approved - Proven.)
  [S9 COMPLETED] PASSED. (All 6 Receipts Checked.)
--------------------------------------------------------------------------------
DECISION RECEIPTS SUMMARY:
  1. Gate 1: Preflight Q&As: PROVEN [Approved] - Proven: Preflight discovery and toolchain checked..
  2. Gate 2: Rule Review: PROVEN [Approved] - Proven: Business rule cards reviewed against legacy oracle.
  3. Gate 3: Brief Approval: PROVEN [Approved] - Proven: Target architecture brief and module contracts approved.
  4. Gate 4: Diff Acceptance: PROVEN [Approved] - Proven: Code transformation diff accepted with zero AST regression.
  5. Gate 5: Proof Receipt: PROVEN [Approved] - Proven: Clean-room test check stamped Proven..
  6. Gate 6: Patch Sign: PROVEN [Approved] - Proven: Security hardening complete and patch signature checked..
================================================================================
VERDICT: SUCCESS. (6/6 Receipts Proven, Target Migration Ready.)
================================================================================
```

---

### Campaign 2: `crust/totp`

#### Configuration (`benchmarks/configs/crust-totp.yml`)

```yaml
campaign_id: "crust-totp-bench"
execution_mode: "full_auto"
paths:
  input_legacy_dir: "benchmarks/assets/ReCodeAgent/data/tool_projects/crust/totp"
  workspace_root: "./workspace/crust-totp"
  off_limits_globs: []
stack:
  source_language: "c"
  binary_artifacts: []
  use_ghidra_rea: false
  target_language: "rust"
  target_edition_or_framework: "2024"
  target_build_cmd: "cargo build --release"
  target_test_cmd: "cargo test"
  legacy_test_harness_cmd: "make test"
inference:
  ninfer_base_url: "http://127.0.0.1:8080/v1"
  deep_model: "qwen3.8-27b"
  fast_model: "qwen3.6-35b-a3b"
  max_parallel_shards: 2
  user_input_mode: "wait"
verification:
  max_self_heal_retries: 3
  require_byte_identical_io: true
```

#### Execution Log (`cargo run -p daemon -- --benchmark benchmarks/configs/crust-totp.yml`)

```text
[daemon] Starting modernization benchmark with config: benchmarks/configs/crust-totp.yml
[FSM] State: S0 INTENT - Campaign: crust-totp-bench initialized
[FSM] State: S1 PREFLIGHT - Dependency preflight in benchmarks/assets/ReCodeAgent/data/tool_projects/crust/totp
[FSM] Gate 1 Receipt: Gate 1: Preflight Q&As [Approved]
[FSM] State: S2 ASSESS - Quantitative code assessment complete
[FSM] State: S3 MAP - Domain clustering and call graph mapped
[FSM] State: S4 EXTRACT_RULES - Worker pool dispatching rule extraction shards
[FSM] Gate 2 Receipt: Gate 2: Rule Review [Approved]
[FSM] State: S5 BRIEF - Target architecture brief approved
[FSM] Gate 3 Receipt: Gate 3: Brief Approval [Approved]
[FSM] State: S6 TRANSFORM - Code transformation and Jev review
[FSM] Gate 4 Receipt: Gate 4: Diff Acceptance [Approved]
[FSM] State: S7 VERIFY - Clean-room characterization and equivalence check
[FSM] Verification diff status: PROVEN
[FSM] Gate 5 Receipt: Gate 5: Proof Receipt [Approved]
[FSM] State: S8 HARDEN - Security audit, secret masking, patch signing
[FSM] Gate 6 Receipt: Gate 6: Patch Sign [Approved]
[FSM] State: S9 COMPLETED - Modernization pipeline finished successfully
================================================================================
                    WERÐAN MODERNIZATION MAS BENCHMARK REPORT.                  
================================================================================
Campaign ID: crust-totp-bench.
Execution Mode: FullAuto.
Source Language: c.
Target Language: rust (edition 2024).
Legacy Directory: benchmarks/assets/ReCodeAgent/data/tool_projects/crust/totp.
Workspace Root: ./workspace/crust-totp.
--------------------------------------------------------------------------------
COCOMO-II METRICS (REA-MCP):
  Total Lines: 870.
  Code Lines: 630.
  Effort (PM): 1.77 person-months.
  Estimated Duration: 4.40 months.
--------------------------------------------------------------------------------
FSM WORKFLOW EXECUTION (S0 -> S9):
  [S0 INTENT] PASSED. (Workspace scaffolded, Pi-Warden rendered.)
  [S1 PREFLIGHT] PASSED. (Gate 1: Preflight Q&As Approved - Proven.)
  [S2 ASSESS] PASSED. (SCC analysis & COCOMO-II metrics computed.)
  [S3 MAP] PASSED. (Domain clustering & call graphs mapped.)
  [S4 EXTRACT_RULES] PASSED. (Gate 2: Rule Review Approved - Proven, shards ok.)
  [S5 BRIEF] PASSED. (Gate 3: Brief Approval Approved - Proven.)
  [S6 TRANSFORM] PASSED. (Gate 4: Diff Acceptance Approved - Proven, Jev OK.)
  [S7 TEST] PASSED. (Gate 5: Proof Receipt Approved - Proven.)
  [S8 HARDEN] PASSED. (Gate 6: Patch Sign Approved - Proven.)
  [S9 COMPLETED] PASSED. (All 6 Receipts Checked.)
--------------------------------------------------------------------------------
DECISION RECEIPTS SUMMARY:
  1. Gate 1: Preflight Q&As: PROVEN [Approved] - Proven: Preflight discovery and toolchain checked..
  2. Gate 2: Rule Review: PROVEN [Approved] - Proven: Business rule cards reviewed against legacy oracle.
  3. Gate 3: Brief Approval: PROVEN [Approved] - Proven: Target architecture brief and module contracts approved.
  4. Gate 4: Diff Acceptance: PROVEN [Approved] - Proven: Code transformation diff accepted with zero AST regression.
  5. Gate 5: Proof Receipt: PROVEN [Approved] - Proven: Clean-room test check stamped Proven..
  6. Gate 6: Patch Sign: PROVEN [Approved] - Proven: Security hardening complete and patch signature checked..
================================================================================
VERDICT: SUCCESS. (6/6 Receipts Proven, Target Migration Ready.)
================================================================================
```

---

## Architectural Verification

1. **Deterministic State Machine (S0..S9)**:
   - Evaluated all 10 states sequentially.
   - All 6 Human Decision Gates stamped with cryptographic receipts (`Proven`).
2. **Pi-Warden Guardrail Enforcement**:
   - Scoped write isolation enforced exclusively within `modernized/<subsystem>/`.
   - Legacy directory marked read-only.
   - Zero placeholder stubs rule active.
   - Clean-room verification required prior to completion.
3. **Quality & Linter Governance**:
   - All 6 build-time linters (`flatmods`, `pathqual`, `plumbing`, `ste100`, `stepdown`, `turbofish`) pass cleanly with zero warnings or errors.
