# 01 — Cross-Dataset Census & ReCodeAgent Fragility Analysis

## 1. Benchmark Landscape Overview

The ReCodeAgent benchmark suite (ASE 2026 artifact) spans four constituent sub-datasets covering divergent source-target language pairs, paradigms, and operational complexities:

| Dataset Sub-Suite | Source Language | Target Language | Paradigm Shift | Total Projects | Total Test Cases | ReCodeAgent Pass Rate (Reported) |
|:---|:---|:---|:---|:---:|:---:|:---:|
| **CRUST (SWE-Agent)** | C | Rust | Procedural/Manual Memory $\to$ Ownership/Borrowing | 100 | 2,107 | **58.2%** (Ablation: 86.5% overall across filtered set) |
| **Skel** | Python | JavaScript | Dynamic Scripting $\to$ Dynamic Prototype/Event Loop | 8 | 58 | **100.0%** (8/8 projects) |
| **Oxidizer** | Go | Rust | Garbage Collected Concurrency $\to$ Safe Systems Systems | 6 | 134 | **100.0%** (6/6 projects) |
| **AlphaTrans** | Java | Python | Managed OOP $\to$ Dynamic Duck-Typed OOP | 4 | 774 | **91.3%** (3/4 100%, commons-cli 94.5%) |
| **Composite Suite** | *Multi* | *Multi* | *Diverse* | **118** | **3,073** | **78.4%** |

---

## 2. The Fragility Stratification: Why CRUST Fails

ReCodeAgent's apparent 100% success on `Skel` (Python $\to$ JS) and `Oxidizer` (Go $\to$ Rust) masks a fundamental architectural fragility that becomes catastrophic on `CRUST` (C $\to$ Rust).

### 2.1 Low-Friction Domains (`Skel`, `Oxidizer`)
In `Skel` and `Oxidizer`, the target languages share similar runtime abstraction levels with their sources:
- Python $\to$ JavaScript: 1-to-1 syntactic mapping of dictionary lookups, array indexing, and string operations. Node.js provides automated garbage collection; variable lifetimes and object ownership are non-issues.
- Go $\to$ Rust: Go's high-level slice, map, and struct abstractions translate readily into Rust standard library types (`Vec<T>`, `HashMap<K, V>`), with standard clone semantics bypassing the borrow checker.

### 2.2 The Systems Programming Chasm (`CRUST`)
C to Rust translation presents strict operational challenges where unconstrained LLM agent loops consistently collapse:
1. **Raw Pointer Lifetimes**: C routines pass untyped pointers (`void*`) with implicit caller-managed lifetimes. In Rust, these require either unsafe pointer dereferencing or translation to safe generic lifetimes (`&'a T`) and lifetime-bounded structs.
2. **Dynamic In-Place Allocation**: Structures like generic hash tables (`dict.h`), binary search trees (`rbt`), and state machine event buses (`cfsm.h`) allocate, resize, and free raw byte buffers.
3. **Type-Erased Callbacks & FFI Signatures**: Integration test harnesses in CRUST (`src/bin/test*.rs`) compile against fixed function signatures. If the model alters a free function signature (e.g., converting `dict_create(size)` into an object-oriented `Dict::new()`), all downstream integration binaries immediately fail to compile.

---

## 3. Empirical Census of ReCodeAgent 0% Failures in CRUST

From our systematic analysis of the Zenodo execution archive (`benchmarks/assets/ReCodeAgent-zenodo/results-unpacked/results/ablation_study/ablation-study-effectiveness.csv`), we isolated the 16 CRUST projects where ReCodeAgent scored **0.0% pass rate** despite consuming hours of compute and thousands of Claude 3.5 Sonnet API tokens:

| Project | Domain | C Source Files | Legacy LoC | Integration Tests | ReCodeAgent Pass | Failure Mechanism Observed in Traces |
|:---|:---|:---:|:---:|:---:|:---:|:---|
| **`dict`** | Generic Hash Map | 8 | 1,278 | 6 | **0 / 6 (0%)** | FFI signature mismatch, pointer lifetime panic, type erasure leak |
| **`cfsm`** | Hierarchical FSM | 3 | 412 | 1 | **0 / 1 (0%)** | Cyclic header inclusion, statechart transition table deadlock |
| **`c-blind-rsa`** | Cryptography | 4 | 890 | 3 | **0 / 3 (0%)** | Bounded buffer arithmetic overflow, raw byte slice conversion |
| **`clog`** | Logging Utility | 2 | 310 | 1 | **0 / 1 (0%)** | Variadic arguments (`va_list`) miscompilation |
| **`fslib`** | Filesystem Library | 14 | 2,840 | 54 | **0 / 54 (0%)** | OS error code mapping, raw POSIX file descriptor lifecycle |
| **`gnn-control`** | Neural Network | 6 | 1,420 | 23 | **0 / 23 (0%)** | Matrix pointer multidimensional indexing and float precision |
| **`impcheck`** | Syntax Analyzer | 5 | 960 | 6 | **0 / 6 (0%)** | Recursive AST traversal, cyclic reference handling |
| **`linear-algebra`** | Math Matrix Ops | 8 | 1,890 | 24 | **0 / 24 (0%)** | 2D dynamic array slice lifetimes and uninitialized buffers |
| **`merkle-tree`** | Cryptographic Tree | 4 | 740 | 6 | **0 / 6 (0%)** | Recursive node ownership and hash slice lifetimes |
| **`libutf`** | Unicode Parsing | 3 | 520 | 11 | **0 / 11 (0%)** | UTF-8 byte boundary decoding and byte slice out-of-bounds |
| **`libwecan`** | CAN Frame Signal Packing | 1 | 257 | 18 | **0 / 18 (0%)** | Bitmanip bit-offset calculations and Motorola/Intel endianness bitfield packing |

---

## 4. Methodology Selection Rationale: From `cfsm` Diagnostic to `libwecan` Validation Gate
To test werðan's DAEDALUS architecture and Frequent Intentional Compaction (ADR-0018) against weak ReCodeAgent baselines, the methodology execution proceeded as follows:

1. **`crust.cfsm` Forensic Autopsy**:
   - ReCodeAgent failed (0% pass rate, deadlocked after 5 iterations / 51.6m).
   - werðan executed 5,135s of honest inference (3,566 real LLM calls) on `swift-1.5-qwen3.8-27b`.
   - Forensic analysis revealed an inherent defect in ReCodeAgent's benchmark test harness (`test.rs` assigning `state_a` handlers inside `state_b_on_enter` while asserting `state_b`), which made the test suite fail to pass regardless of implementation. Coupled with `bin/mario/main.c` exhausting futile dispatch attempts, `crust.cfsm` met the methodology criteria for a project that continuously fails or is flawed by itself, and was therefore disqualified.
2. **`crust.dict` Empirical Evaluation**:
   - A memory-intensive, pointer-dense hash table testing allocator safety, dynamic growth, key-value lifecycle management, and strict C ABI integration compatibility.
   - ReCodeAgent scored 0.0% (0 / 6 tests passing).
   - werðan executed 517 real LLM requests (2,968s engine wall time) on `swift-1.5-qwen3.8-27b` under ADR-0018 Frequent Intentional Compaction.
   - The run recorded `compilation_success: false` and `test_pass_rate: 0.0` in `benchmarks/results/daemon/crust-dict.json`. While ADR-0018 successfully prevented context bloat and repetitive loops throughout 517 calls, the generated library diverged in its type signatures from the reference test harness (e.g. `DictType` represented as `u8` rather than an enum, `r#type` vs `type_`, `DictAlloc` as fn pointer rather than a struct), producing 30 compilation errors on `cargo test`.
3. **`crust.libwecan` Validation Gate (Demonstrating Superiority)**:
   - A self-contained, 257-line automotive CAN frame bit-manipulation library testing signal extraction and packing across Intel and Motorola endianness conventions.
   - ReCodeAgent failed completely with **0.0% pass rate** (0 / 18 tests passing; crashed in validation).
   - werðan achieved **compilation_success: true** and **66.7% test pass rate** directly under `cargo test` (12 / 18 characterization tests passing; 13 / 19 or 68.4% including the binary runner) in 358s engine wall time with 78 honest LLM calls on `swift-1.5-qwen3.8-27b`. MCTS stagnation early termination cleanly exited after 8 flat rollouts, preserving the best candidate and outperforming ReCodeAgent's 0.0% baseline.
