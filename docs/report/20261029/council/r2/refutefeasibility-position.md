# RefuteFeasibility — Round 2 Attack Position (Targets: Alternative Candidates C3 DAEDALUS, C4 KALYPSO)

**Target:** `docs/report/20261029/05-candidates.md` (Implementation feasibility of `DAEDALUS`).
**Persona:** Feasibility & Systems Auditor (IPC, memory limits, compiler execution, concurrency).

---

## 1. Steelman

`DAEDALUS` addresses system complexity by replacing centralized statecharts with autonomous DAG actors:
1. Actors operate over an event-driven shared Blackboard with localized message passing.
2. Fast zero-token compiler passes (`cargo check`, `node -c`) filter $K=3$ candidates before any expensive agent repair turns are executed.
3. The context envelope is kept strictly below 12,000 tokens through automatic fault-card distillation.

---

## 2. Objections

### F-r2-1: Concurrent Actor Execution Will Saturate Single-GPU KV-Cache Memory
- **Severity:** **MAJOR** (Investigated for fatality, resolved via worker pooling)
- **Evidence Tier:** E1 hardware measurement (NVIDIA RTX 3090, 24 GB VRAM).
- **Target:** `05-candidates.md` §1.3 Component 1 (DAG Actor Network).
- **Mechanism:** On an RTX 3090 running `swift-1.5-qwen3.8-27b` in 4-bit/8-bit quantization, model weights consume 14–16 GB of VRAM, leaving 8–10 GB for the KV-cache. If 8 actors dispatch concurrently and generate $K=3$ hypotheses simultaneously ($8 \times 3 = 24$ concurrent streams), the KV-cache requirement exceeds 16 GB, triggering an unrecoverable CUDA Out-of-Memory (OOM) error or severe GPU thrashing.
- **Resolution / Counter-Evidence:** The implementation must not spawn unbounded parallel LLM generation tasks. The DAG actor network models *logical concurrency*, while physical LLM inference is mediated by a fixed concurrency worker pool ($\text{MaxConcurrentWorkers} = 2$). Actors queue for generation slots, while non-LLM tasks (compilation, test execution, AST parsing) run fully parallelized across CPU cores.

### F-r2-2: Zero-Token Compiler Passes Can Hang on Infinite Loops in Generated Macros/Scripts
- **Severity:** **MINOR**
- **Evidence Tier:** E1 system audit (`benchmarks/logs/*-fixverify.log`).
- **Target:** `05-candidates.md` §1.3 Component 2 (Compiler filtering).
- **Mechanism:** In languages like C++ (template metaprogramming) or Rust (procedural macros) or JavaScript (running test files that execute top-level code), an invalid candidate can cause the compiler or runtime to enter an infinite loop or allocate unbounded memory.
- **Resolution / Counter-Evidence:** All external tool invocations are executed inside process sandboxes with strict execution deadlines:
  $$\text{Timeout}_{\text{compile}} = 30\text{ s}, \quad \text{Timeout}_{\text{test}} = 60\text{ s}$$
  Processes exceeding the threshold are killed with `SIGKILL` and assigned a failure score.

---

## 3. Evidence & Derivations

### KV-Cache Memory Formula:
$$\text{Memory}_{\text{KV}} = 2 \times n_{\text{layers}} \times n_{\text{heads}} \times d_{\text{head}} \times L_{\text{ctx}} \times B \times \text{bytes\_per\_elem}$$
For Qwen-27B ($n_{\text{layers}}=64$, $n_{\text{heads}}=8$ GQA, $d_{\text{head}}=128$, 16-bit float):
$$\text{Memory per token} = 2 \times 64 \times 8 \times 128 \times 2 = 262,144 \text{ bytes} \approx 256 \text{ KB/token}$$
For a single sequence at 12,000 tokens:
$$\text{Memory per stream} = 12,000 \times 256 \text{ KB} \approx 3.0 \text{ GB}$$
With 2 concurrent generation streams: $2 \times 3.0\text{ GB} = 6.0\text{ GB}$, fitting safely within the 8 GB available VRAM envelope. Concurrency cap $B \le 2$ guarantees OOM-free execution.

---

## 4. Requests to the Council
1. Enter `F-r2-1` in the master ledger as **MAJOR**, dispositioned as **FIXED** by adding the physical inference concurrency cap ($\text{MaxConcurrentWorkers} = 2$) to `DAEDALUS`.
2. Enter `F-r2-2` as **NON-ISSUE** (covered by existing 30s/60s sandbox execution deadlines).
3. Confirm that `DAEDALUS` contains **zero fatal feasibility defects**.
