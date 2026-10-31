# RefuteFeasibility — Round 1 Attack Position (Targets: Incumbent METHODOLOGY.md)

**Target:** `docs/references/METHODOLOGY.md` (§3, §5) and engine implementation receipts.
**Persona:** Feasibility & Systems Auditor (IPC, runtime deadlocks, scheduler liveness, process crash recovery).

---

## 1. Steelman

`THESEUS` structures repository orchestration through a centralized Two-Tier Engine:
1. The **Global Orchestration Tier** runs a macro statechart monitoring overall campaign progress, managing DAG task readiness via a dynamic `ReadySet`, and coordinating cross-module transitions.
2. The **Holon Solver Tier** instantiates an isolated blackboard and engine per translation unit, executing fine-grained MCTS passes.
3. The **Three-Tier Regression Calculus** (§5) guarantees surgical rollback: Tier A (intra-module re-rollout), Tier B (surgical orthogonal transition along parent edge $e_{pi}$), and Tier C (deep history $H^*$ restoration via Tarjan SCC condensation).

---

## 2. Objections

### F-r1-1: Parentless Single-Vertex Modules Fabricate `"parent_upstream"`, Inducing Fatal Tier B Conflict Death
- **Severity:** **FATAL**
- **Evidence Tier:** E1 receipt (`benchmarks/results/daemon/skel-heapq.json` & v17 logs) + E4 code audit (`werðan/libs/global/engine/src/lib.rs`).
- **Target:** `METHODOLOGY.md` §5.2: *"Tier B: Surgical Orthogonal Transition ... triggered when module $v_i$ exhausts its local exploration budget $B_{\max}$"*, broadcasting rollback along predecessor edge $e_{pi}$.
- **Mechanism:** In isolated benchmark projects (e.g., `skel` projects consisting of a single file `source.py`), $|V|=1$ and $E=\emptyset$. When $v_0$ exhausted its rollout budget, the solver bridge attempted to lookup parent $e_{p0}$. Finding none, the legacy implementation synthesized a mock string `"parent_upstream"`. Because no such vertex existed in the DAG, the macro engine panicked with an unhandled state transition error, converting an exploration timeout into an unrecoverable system crash.

### F-r1-2: ReadySet Scheduler Deadlocks when Cycle Detection Reports Null Cycles
- **Severity:** **MAJOR**
- **Evidence Tier:** E1 receipt (`benchmarks/logs/*-fixverify.log`, 6/6 pre-fix daemon hangs).
- **Target:** `METHODOLOGY.md` §4 Algorithm 1 & §5.3 (Tier C cycle handling).
- **Mechanism:** When a module stalled, the engine scheduled a cycle check. In 6 out of 6 benchmark campaigns, the dependency graph contained zero circular dependencies. The cycle detection algorithm returned `Cycles: []`. The scheduler had no handler for an empty cycle report in an error state; it failed to return the module to `ReadySet` and failed to mark it `TERMINATED`. The daemon entered an infinite active-wait poll loop:
  ```text
  [WARN] no ready module while unhardened modules remain: source.py
  ```
  The process hung until external watchdog kill (3 per campaign).

### F-r1-3: Global In-Memory Blackboard Creates an IPC Serialization Bottleneck
- **Severity:** **MAJOR**
- **Evidence Tier:** E4 derivation & profiling of Tokio task dispatch.
- **Target:** `METHODOLOGY.md` §2 Blackboard Architecture.
- **Mechanism:** The incumbent routes all inter-agent artifacts (ASTs, symbol tables, compiler diagnostics, candidate source strings) through a single centralized Tokio `RwLock<Blackboard>`. When 4 generative subagents per module run concurrently across multiple shards, read/write contention on the root blackboard lock serializes execution, nullifying multi-core throughput gains.

---

## 3. Evidence & Derivations

### Trace of F-r1-1 from `v17` Campaign Log:
```text
2026-10-25 16:59:12 [ERROR] HolonSolver: module source.py exhausted budget B_max (16 rollouts)
2026-10-25 16:59:12 [DEBUG] GlobalEngine: resolve_parent_id(source.py) -> None; fallback to "parent_upstream"
2026-10-25 16:59:12 [FATAL] Statechart: transition EXTRINSIC_CONFLICT with target "parent_upstream" not found in active graph
2026-10-25 16:59:12 [FATAL] Process aborted with exit code 101 (panic)
```
This confirms that the incumbent's formal Tier B transition had zero runtime protection for single-vertex inputs.

---

## 4. Requests to the Council
1. Enter `F-r1-1` in the master ledger as **FATAL** and `F-r1-2` as **MAJOR**.
2. Require that alternative methodologies eliminate dynamic parent fallback synthesis and mandate strict liveness guarantees for task schedulers.
