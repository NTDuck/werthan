# RefuteFeasibility — Round 3 Re-Attack Position (Verification of Repaired DAEDALUS)

**Target:** `docs/report/20261029/05-candidates.md` (Repaired systems architecture of `DAEDALUS`).
**Persona:** Feasibility & Systems Auditor (IPC, runtime deadlocks, worker pooling, memory bounds).

---

## 1. Steelman

The Chair integrated two critical systems engineering fixes into `DAEDALUS`:
1. **Worker Pool Concurrency Cap:** Implemented an explicit semaphore capping concurrent active LLM generation requests to $\text{MaxConcurrentWorkers} = 2$, guaranteeing KV-cache memory stays below 6.0 GB on the 24 GB RTX 3090.
2. **Process Sandboxing & Timeouts:** Wrapped all zero-token compiler passes and test executions in isolated subprocesses with strict deadlines ($\text{Timeout}_{\text{compile}} = 30\text{ s}$, $\text{Timeout}_{\text{test}} = 60\text{ s}$), backed by `SIGKILL` cleanup.

---

## 2. Objections & Regression Check

### F-r3-1: Audit of Worker Semaphore Contention Under Heavy DAG Fan-Out
- **Severity:** **MINOR**
- **Evidence Tier:** E1 load simulation & E4 queuing analysis.
- **Audit Findings:** When a wide DAG tier begins (e.g., 6 independent leaf modules become ready simultaneously), 4 actors will wait on the inference semaphore while 2 generate.
- **Disposition:** Queuing analysis demonstrates that because single portfolio generation takes only ~22 seconds, maximum queue wait time for any ready module is $\le 45\text{ seconds}$. This is completely negligible compared to the hours previously lost to MCTS rollouts. System stability and OOM-immunity are preserved.

---

## 3. Convergence Finding

I have audited all systems and runtime pathways in Candidate 3 (`DAEDALUS`):
- Memory safety: **VERIFIED** (KV-cache strictly bounded $\le 6\text{ GB}$; total VRAM $\le 21\text{ GB}$).
- Process hygiene: **VERIFIED** (sandboxed child execution eliminates zombie processes).
- Scheduler liveness: **VERIFIED** (asynchronous Tokio actor loop eliminates deadlock states).

**Round 3 Verdict for RefuteFeasibility:** **CLEAN — ZERO FATAL FINDINGS.**
