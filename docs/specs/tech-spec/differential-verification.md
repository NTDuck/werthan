# Behavioral Equivalence and Differential Testing Specification

Entry point: [../tech-spec.md](../tech-spec.md). This specification defines the mathematical foundations, execution pipeline, observation schema, divergence taxonomy, and persistent blackboard artifacts for **Behavioral Equivalence and Differential Verification** in the werðan architecture.

---

## 1. Executive Summary & Architectural Scope

In repository-scale modernization, syntactic compilation ($\mathbb{I}_{\text{compile}} = 1.0$) and self-generated characterization tests are necessary but insufficient for correctness. Co-generated tests suffer from **specification gaming** and **weak oracles**, wherein translated code passes vacuous assertions (e.g. exit code 0 without assertions, or unconditional stub returns).

Per the Two-Tier Harel Statechart and Encapsulated ARIADNE Multi-Agent Architecture ([`METHODOLOGY.md`](../../references/METHODOLOGY.md)), verification of a candidate modernization unit $c'$ is strictly partitioned:
1. **Stage 3a — Deterministic Execution Harness ($\mathcal{A}_{\text{eval}}^{\text{exec}}$)**: Non-LLM execution combining the compiler, static security linters, property-based suites, and the **Mechanized Differential Verification Engine**.
2. **Gate 5 (Proof Receipt)**: Imposes the invariant that external ground truth and differential behavioral equivalence outrank internal LLM self-endorsement.

Differential verification executes the legacy source program $P_s$ and the modernized target program $P_t$ against identical stimuli under controlled environments, asserting observable equivalence across six contract dimensions.

---

## 2. The Fundamental Contract of Behavioral Equivalence

Let $P_s$ be the legacy program or module in source language $\mathcal{L}_s$, and let $P_t$ be the translated candidate in target language $\mathcal{L}_t$.

### Definition 1: Behavioral Equivalence Contract

$$\forall (x, \sigma) \in \mathcal{S} \times \Sigma, \quad Obs(P_s, x, \sigma) \equiv Obs\big(P_t, \phi(x), \phi(\sigma)\big)$$

Where:
- $\mathcal{S}$ is the domain of test inputs, arguments, and invocation vectors.
- $\Sigma$ is the universe of system and environmental pre-states (file system, persistent store, memory, environment variables).
- $x \in \mathcal{S}$ is a language-neutral test scenario vector.
- $\sigma \in \Sigma$ is the initial environment state.
- $\phi: \mathcal{D}(\mathcal{L}_s) \to \mathcal{D}(\mathcal{L}_t)$ is the **semantic projection mapping** that projects source types, memory representations, and configuration objects into target domain representations.
- $Obs(P, x, \sigma)$ is the **Execution Observation Record** captured by the runtime sandbox during the evaluation of program $P$ under input $x$ and initial state $\sigma$.
- $\equiv$ denotes behavioral equivalence modulo the **Deterministic Normalization Policy** $\mathcal{N}$ defined in §6:
  $$Obs(P_s, x, \sigma) \equiv Obs\big(P_t, \phi(x), \phi(\sigma)\big) \iff \mathcal{N}\big(Obs(P_s, x, \sigma)\big) == \mathcal{N}\big(Obs(P_t, \phi(x), \phi(\sigma))\big)$$

---

## 3. The 6 Contract Dimensions

Behavioral equivalence is evaluated across six orthogonal contract dimensions. A failure in any enabled dimension constitutes a behavioral divergence.

```
+-----------------------------------------------------------------------+
|                         6 Contract Dimensions                         |
+-------------------+-------------------+-------------------------------+
| 1. Return Values  | 2. State Mutation | 3. Side Effects               |
|    Output data,   |    Memory heap,   |    Network I/O, IPC,          |
|    scalars, data  |    file systems,  |    system calls,              |
|    structures     |    databases      |    file descriptor ops        |
+-------------------+-------------------+-------------------------------+
| 4. Exceptions     | 5. Protocols      | 6. Ordering & Timing          |
|    Errors, panic, |    CLI arguments, |    Event sequences,           |
|    exit sentinels,|    stdout/stderr, |    causal chains,             |
|    errno values   |    exit code      |    call topology              |
+-------------------+-------------------+-------------------------------+
```

### 3.1 Return Values
- **Definition**: Direct functional return values, output pointer parameters, or composite return structures.
- **Verification Rule**: Serialized functional outputs must align under projection $\phi$. Struct fields, collections, and primitives must match in type semantics, key-value mappings, and element ordering (where collections are order-sensitive).

### 3.2 State Mutations
- **Definition**: Persistent mutations applied to the environment or in-memory state graph outside the direct return value.
- **Verification Rule**: Let $\sigma'$ be the post-execution state. For all observable state partitions $\mathcal{K} \subseteq \Sigma$, $\phi(\sigma'_s[\mathcal{K}]) = \sigma'_t[\mathcal{K}]$. This includes database records, mutated object graphs, and file system modifications on designated volumes.

### 3.3 Side Effects
- **Definition**: External interactions mediated by the operating system or runtime runtime substrate during execution.
- **Verification Rule**: Network calls (HTTP/gRPC endpoints, payloads, headers), IPC messages, POSIX signal emissions, and file write calls must match in target destination, call count, and payload contents. Mocking layers capture and verify side-effect logs.

### 3.4 Exceptions and Error States
- **Definition**: Unhappy-path termination, including thrown exceptions, returned error objects, signals (e.g. `SIGSEGV`, `SIGABRT`), and `errno` assignments.
- **Verification Rule**: If $P_s$ raises error $E_s$ on input $x$, $P_t$ must yield an error $E_t$ such that $\phi(E_s) \sim E_t$. Modernized code must not convert errors into silent default values (`Option::None` or empty collections), nor panic where legacy code handled errors gracefully.

### 3.5 Protocols (CLI, Stdio, and Process Execution)
- **Definition**: Standard POSIX command-line interfaces: CLI flag semantics, `stdout` streams, `stderr` streams, and operating system exit codes.
- **Verification Rule**:
  - Exit code parity: $\text{ExitCode}(P_s) == \text{ExitCode}(P_t)$.
  - Standard output and error parity: Streams must match line-by-line following semantic normalization (whitespace stripping, dynamic token masking).

### 3.6 Ordering and Timing
- **Definition**: Temporal sequence of operations, message delivery orders, and computational latency bounds.
- **Verification Rule**:
  - **Ordering**: The sequence of external side effects and asynchronous events must preserve causal consistency ($e_a \prec e_b \implies e'_a \prec e'_b$).
  - **Timing**: Modernized target code must not introduce non-terminating loops or algorithmic regressions ($O(N^2)$ vs $O(N)$). Latency must remain within a bounded multiplier factor $\kappa_{\text{latency}}$ of baseline execution: $\text{WallTime}(P_t) \le \kappa_{\text{latency}} \cdot \text{WallTime}(P_s) + \delta_{\text{overhead}}$.

---

## 4. Differential Verification Pipeline

The differential verification pipeline executes deterministically within Stage 3a of the module solver.

```
       +------------------------------------+
       | Language-Neutral Scenario Database |
       +-----------------+------------------+
                         |
           +-------------+-------------+
           | (x, \sigma)               | \phi(x), \phi(\sigma)
           v                           v
+---------------------+     +---------------------+
| Legacy Source Runner|     | Target Modern Runner|
|   (Oracle Sandbox)  |     |   (Target Sandbox)  |
+----------+----------+     +----------+----------+
           |                           |
           | Obs(P_s, x, \sigma)       | Obs(P_t, \phi(x), \phi(\sigma))
           v                           v
    +-----------------------------------------+
    |      Behavior Comparator Engine         |
    |  - Semantic Token Normalization         |
    |  - Float Epsilon Bounding               |
    |  - Key Ordering & Structure Matching    |
    +--------------------+--------------------+
                         |
                         v
              +---------------------+
              | Differential Report |
              |  - Divergence Class |
              |  - Parity Verdict   |
              +---------------------+
```

### Pipeline Components

1. **Language-Neutral Test Scenario ($\mathcal{S}_{\text{neutral}}$)**:
   A declarative JSON/YAML artifact defining inputs, parameters, environment overrides, fixture paths, mocked service responses, and execution bounds.
2. **Source Runner (Legacy Oracle Adapter, $\mathcal{R}_s$)**:
   Orchestrates compilation and sandboxed execution of the legacy codebase $P_s$. Injects nondeterminism harnesses, captures process I/O, memory snapshots, and serializes raw observation $Obs_s$.
3. **Target Runner (Modernized Target Adapter, $\mathcal{R}_t$)**:
   Compiles and executes candidate target code $P_t$ using the modern toolchain. Injects projected inputs $\phi(x)$ and initial environment $\phi(\sigma)$, capturing raw observation $Obs_t$.
4. **Behavior Comparator ($\mathcal{C}_{\text{diff}}$)**:
   Applies deterministic normalization transformations to $Obs_s$ and $Obs_t$, performs dimensional diffing, and synthesizes a structured verdict.

---

## 5. Execution Observation Schema

Both source and target runners must produce observation records adhering to the canonical `execution-observation` schema.

```json
{
  "$schema": "https://specs.werthan.org/schemas/execution-observation-v1.json",
  "observation_id": "obs-mod-math-042-scenario-001",
  "module_id": "matrix_ops",
  "scenario_id": "scen_gaussian_elimination_singular",
  "environment": {
    "clock_virtual_epoch_ms": 1700000000000,
    "rng_seed": "0x5EED0000CAFE0001",
    "locale": "C",
    "timezone": "UTC"
  },
  "execution_metadata": {
    "duration_ns": 4125890,
    "peak_memory_bytes": 1048576,
    "timed_out": false,
    "crashed": false
  },
  "result": {
    "status": "Err",
    "return_value": null,
    "error": {
      "type": "SingularMatrixError",
      "code": 104,
      "message": "Matrix determinant is zero; inverse undefined"
    }
  },
  "state_changes": [
    {
      "partition": "workspace/data/cache.bin",
      "pre_hash_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "post_hash_sha256": "4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a",
      "bytes_written": 128
    }
  ],
  "side_effects": {
    "filesystem_writes": [
      {
        "path": "/tmp/werthan_sandbox/log/matrix.log",
        "action": "Append",
        "content_hash": "a1b2c3d4..."
      }
    ],
    "network_calls": [],
    "system_calls": ["openat", "write", "close"]
  },
  "process": {
    "exit_code": 1,
    "stdout": "Processing matrix 100x100...\nError encountered: Singular matrix\n",
    "stderr": "[WARN] Matrix condition number exceeds threshold\n"
  }
}
```

---

## 6. Nondeterminism Controls & Semantic Normalization

Raw process outputs exhibit nondeterminism that produces false-positive divergences if compared verbatim. The differential engine enforces four strict nondeterminism controls and a normalization transform.

### 6.1 Clock Virtualization
- **Mechanism**: The sandbox injects simulated monotonic clocks via `LD_PRELOAD` wrappers or target test harness mocks for `clock_gettime`, `gettimeofday`, `time()`, and language-specific APIs (`std::time::Instant`, `time.time()`).
- **Policy**: Time starts at frozen epoch $t_0 = 1\,700\,000\,000\,000\text{ ms}$ and increments by deterministic synthetic step $\Delta t = 100\ \mu\text{s}$ per clock read.

### 6.2 RNG Seeding
- **Mechanism**: Random number generators are pinned. Hardware sources (`/dev/random`, `/dev/urandom`) are mapped to a deterministic pseudo-random stream seeded with $S_0 = \text{SHA256}(\text{scenario\_id})$.
- **Policy**: Any execution reading unseeded random entropy is flagged as an isolation violation.

### 6.3 Environment Normalization
- All executions occur under sanitized environments:
  - `TZ=UTC`
  - `LC_ALL=C`
  - `LANG=C`
  - `TMPDIR=/tmp/werthan_sandbox/{scenario_id}`
- Process environment variables are stripped of host variables (`USER`, `HOME`, `PATH` sanitized).

### 6.4 Floating-Point Comparison Policy
Direct IEEE 754 bit-exact comparison between languages and compilers (e.g. GCC x87 excess precision vs Rust LLVM strict SSE2/AVX) is mathematically unsound. The comparator enforces a dual relative/absolute tolerance threshold:

$$|a - b| \le \epsilon_{\text{abs}} + \epsilon_{\text{rel}} \cdot |a|$$

- **Default Constants**: $\epsilon_{\text{abs}} = 10^{-6}$, $\epsilon_{\text{rel}} = 10^{-5}$.
- **Special Sentinels**:
  - $+0.0 \equiv -0.0$.
  - $\text{NaN} \equiv \text{NaN}$ iff both operands represent undefined mathematical outcomes in the scenario context.
  - $+\infty \equiv +\infty$ and $-\infty \equiv -\infty$.

### 6.5 Dynamic Token Masking & Canonical Normalization
Before evaluating diffs:
1. **Key Sorting**: JSON structures and dictionary keys are canonically sorted.
2. **Volatile Token Masking**:
   - Memory addresses (`0x[0-9a-fA-F]{4,16}`) are replaced by `<PTR>`.
   - UUIDs (`[0-9a-fA-F-]{36}`) are replaced by `<UUID>`.
   - Sandbox paths (`/tmp/werthan_sandbox/[^\s]+`) are replaced by `<SANDBOX_PATH>`.
   - ISO8601 timestamps are replaced by `<TIMESTAMP>`.
3. **Whitespace Canonicalization**: Trailing whitespace and carriage returns (`\r\n` vs `\n`) are normalized.

---

## 7. 5 Sources of Scenarios

To achieve robust behavioral coverage and prevent overfitting, the Differential Verification Engine ingests test scenarios from five complementary sources:

```
+------------------------------------------------------------------------+
|                          5 Sources of Scenarios                        |
+--------------------------+---------------------------------------------+
| Source                   | Mechanics & Purpose                         |
+--------------------------+---------------------------------------------+
| 1. Developer Tests       | Extracted from legacy repo; establishes     |
|                          | baseline regression ground truth.           |
+--------------------------+---------------------------------------------+
| 2. Execution-Derived     | Record-and-replay traces capturing real-     |
|    Snapshots             | world runtime inputs and serialized state.  |
+--------------------------+---------------------------------------------+
| 3. Coverage-Guided       | Dynamic concolic/fuzzing generation         |
|    Generated Tests       | maximizing branch coverage on legacy AST.   |
+--------------------------+---------------------------------------------+
| 4. Property-Based        | Generative invariant testing: round-trip,   |
|    Testing & Fuzzing     | associativity, identity, idempotency.       |
+--------------------------+---------------------------------------------+
| 5. Metamorphic Testing   | Transformation relations: f(T(x)) == f(x)   |
|                          | without needing an explicit oracle value.   |
+--------------------------+---------------------------------------------+
```

### 7.1 Developer Tests
- Extracted existing test files, test binaries, and CI scripts present in the legacy repository.
- Provides immediate regression coverage of human-written edge cases.

### 7.2 Execution-Derived Snapshots
- Captured production traces or test-suite execution dumps capturing live arguments, database records, and returned structures.
- Ensures high-fidelity preservation of complex, domain-specific data structures.

### 7.3 Coverage-Guided Generated Tests
- Dynamic test generation driven by legacy code coverage metrics.
- Leverages branch coverage instrumentation (e.g. LLVM Sanity Coverage / GCC gcov) to generate input inputs that exercise previously unreached control flow paths.

### 7.4 Property-Based Testing & Fuzzing
- Automated generative fuzzing based on declared algebraic invariants:
  - **Round-Trip Identity**: $\text{decode}(\text{encode}(x)) == x$.
  - **Idempotency**: $f(f(x)) == f(x)$.
  - **Commutativity / Invariance**: $f(a, b) == f(b, a)$.
- Employs bounded random shrinking to locate minimal reproducible counterexamples.

### 7.5 Metamorphic Testing
- Solves the **Oracle Problem** for complex domain computations (e.g. search heuristics, numerical simulations, optimization algorithms).
- Checks metamorphic relations across input transformations without requiring absolute output computation:
  $$\text{Relation}: \quad \mathcal{M}_{\text{in}}(x_1) \to x_2 \implies \mathcal{M}_{\text{out}}(f(x_1)) \sim f(x_2)$$
  *Example*: Sorting a list or permuting inputs must not change the set of output elements or aggregate sums.

---

## 8. Separation of Source Correctness vs Translation Correctness

A critical defect in naive differential testing is penalizing candidate translations when the **legacy source program itself is buggy, incomplete, or crashes**.

### The Legacy As-Is Principle

> **Invariant**: Differential verification measures **Behavioral Parity**, not theoretical ideal correctness.

1. **Source Bug Fidelity**: If legacy program $P_s$ exhibits an idiosyncratic, non-crashing quirk or bug on input $x$, translation $P_t$ **must replicate the behavior** unless an explicit architectural contract override in `ProblemModelSlice` mandates a divergence.
2. **Fault Attribution Invariant**:
   - If execution of $P_s$ fails due to an unhandled crash, memory fault (`SIGSEGV`), or environment failure, the scenario result is classified as **`SOURCE_DEFECT`** or **`HARNESS_DEFECT`**.
   - A `SOURCE_DEFECT` scenario is disqualified from the differential denominator and cannot fail candidate $P_t$.
   - Candidate $P_t$ fails if and only if $P_s$ successfully produces observation $Obs_s$ and $P_t$ produces non-equivalent observation $Obs_t$ (`TRANSLATION_DEFECT`).

---

## 9. Divergence Classification Taxonomy

When the Behavior Comparator detects $Obs_s \not\equiv Obs_t$, the divergence is classified into one of six mutually exclusive categories:

| Divergence Class | Identifier | Description | Corrective Action |
| :--- | :--- | :--- | :--- |
| **Value Mismatch** | `DIV_VALUE` | Functional return values or output parameters differ outside numeric tolerances. | Holon repair agent ($\mathcal{A}_{\text{patch}}$) repairs algorithmic translation logic. |
| **State Mismatch** | `DIV_STATE` | Persistent store, memory buffer, or file modification diffs do not match. | Repair state mutation hooks and synchronization structures. |
| **Side-Effect Mismatch** | `DIV_EFFECT` | Network calls, IPC packets, or system calls differ in frequency, order, or payload. | Align external communication and logging calls with legacy semantics. |
| **Protocol Mismatch** | `DIV_PROTOCOL`| Exit codes, CLI argument parsing, stdout/stderr formatting differ. | Align CLI flag definitions and standard stream serializations. |
| **Source Defect** | `DIV_SOURCE` | Legacy implementation crashed, leaked memory, or exhibited undefined behavior. | Mark scenario as source defect; exclude from pass rate denominator. |
| **Harness Defect** | `DIV_HARNESS` | Timeout due to sandbox resource starvation, serialization crash in runner adapter. | Reconfigure harness timeout or adapter serialization; rerun. |

---

## 10. Persistent Blackboard Artifacts

Differential verification reads from and writes to standardized artifacts on the Global Blackboard $\mathcal{G}$ and Local Blackboard $\mathcal{B}^{(v_i)}$:

```
+------------------------------------------------------------------------+
|                     Persistent Blackboard Artifacts                    |
+--------------------------+---------+-----------------------------------+
| Artifact Key             | Scope   | Schema / Content                  |
+--------------------------+---------+-----------------------------------+
| `verification-spec`      | Global  | Configuration of enabled contract |
|                          |         | dimensions, tolerances, timeouts  |
+--------------------------+---------+-----------------------------------+
| `scenario-corpus`        | Global  | Language-neutral scenario vectors |
|                          |         | and fixture references            |
+--------------------------+---------+-----------------------------------+
| `source-observations`    | Global  | Cached baseline observations      |
|                          |         | Obs(P_s) for all scenarios        |
+--------------------------+---------+-----------------------------------+
| `target-observations`    | Local   | Candidate observations Obs(P_t)   |
|                          |         | for current candidate c'          |
+--------------------------+---------+-----------------------------------+
| `differential-report`    | Local   | Pairwise comparison, divergence   |
|                          |         | classification, parity scores     |
+--------------------------+---------+-----------------------------------+
| `coverage-report`        | Local   | Branch and statement coverage of  |
|                          |         | the differential corpus           |
+--------------------------+---------+-----------------------------------+
| `mutation-report`        | Local   | Stub-sensitivity audit and mutant |
|                          |         | kill ratios                       |
+--------------------------+---------+-----------------------------------+
| `verification-report`    | Global  | Final cryptographic Gate 5 Proof  |
|                          |         | Receipt for module hardening      |
+--------------------------+---------+-----------------------------------+
```

---

## 11. Evidence-Based Reporting Schema

Upon completing differential evaluation, the comparator generates a cryptographic, verifiable `verification-report` artifact.

```json
{
  "$schema": "https://specs.werthan.org/schemas/differential-verification-report-v1.json",
  "report_id": "rep-diff-20261102-matrix_ops-c7f8a9",
  "timestamp": "2026-11-02T14:32:00Z",
  "module_id": "matrix_ops",
  "candidate_hash": "c7f8a9e0123456789abcdef0123456789abcdef0",
  "verdict": "PASSED",
  "summary": {
    "total_scenarios": 120,
    "valid_scenarios": 118,
    "source_defects": 2,
    "harness_defects": 0,
    "passed": 118,
    "failed": 0,
    "pass_rate": 1.0
  },
  "dimensional_scores": {
    "return_values": 1.0,
    "state_mutations": 1.0,
    "side_effects": 1.0,
    "exceptions_errors": 1.0,
    "protocols": 1.0,
    "ordering_timing": 1.0
  },
  "tolerances_applied": {
    "float_epsilon_abs": 1e-6,
    "float_epsilon_rel": 1e-5,
    "latency_multiplier_cap": 2.5
  },
  "divergences": [],
  "mutation_sensitivity": {
    "stub_sensitivity_verified": true,
    "unconditional_stub_detected": false,
    "mutants_tested": 12,
    "mutants_killed": 12,
    "mutation_score": 1.0
  },
  "proof_receipt": {
    "source_runner_hash": "a1b2c3d4...",
    "target_runner_hash": "e5f6a7b8...",
    "corpus_digest": "9f8e7d6c5b4a...",
    "signature": "3045022100..."
  }
}
```

---

## 12. Verification Lifecycle & Statechart Integration

Differential verification links directly to the Two-Tier Harel Statechart:
1. **$S_0$ INTENT Gate**: Preflight checks verify that legacy source adapters and target toolchain runners exist and execute deterministically.
2. **$M_2$ Evaluation Cycle**: Every candidate code rollout $c'$ undergoes Stage 3a differential verification on the local scenario corpus $\mathcal{B}_{CT}^{(v_i)}$.
3. **$M_4$ Hardening Transition**: Transition from $M_2 \to M_4$ strictly requires:
   $$\text{PassRate}_{\text{diff}} == 1.0 \quad \land \quad \text{StubSensitive}(c') == \text{True}$$
4. **$S_4$ INTEGRATION Wave Verification**: Multi-module wave verification executes cross-module integration scenarios to verify composed behavioral equivalence before global hardening.
