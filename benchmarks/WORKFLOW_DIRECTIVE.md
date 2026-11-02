# Benchmark Campaign Workflow Directive & Evaluation Criteria

## 1. Core Workflow Stages (Non-Linear, Loop-Back Enabled)

1. **Stage 1 (Project Selection & Config Generation)**:
   - Select a project from the ReCodeAgent dataset where werðan (THESEUS architecture) is estimated to outperform ReCodeAgent.
   - Inspect ReCodeAgent execution traces from Zenodo (`benchmarks/assets/ReCodeAgent-zenodo/`).
   - Generate / tune the benchmark config in `benchmarks/assets/configs/ReCodeAgent/`.
   - **Target Test Command Mandate**: Benchmark configs MUST set `target_test_cmd` to a command that actually executes the project's external test target (e.g. `cargo test --bins` or `cargo test --bin <test_bin>`). Standard `cargo test` skips `[[bin]]` targets by default when no lib/unit tests exist, which leads `parse_test_counts` to observe zero `test result:` lines and `compute_pass_rate` to return 0.0.
2. **Stage 2 (System Execution)**:
   - **Supported Invocation Entrypoints**:
     - `mise run daemon -- --benchmark <config_path>` (or `mise daemon -- --benchmark <config_path>`)
     - `./target/release/daemon --benchmark <config_path>`
     - `cargo run -p daemon --release -- --benchmark <config_path>`
   - **Zero-Failure Invocation Mandate**: Running via `mise run daemon ...` or direct invocation MUST run cleanly without fatal crashes, panics, or runtime failures.
   - Entrypoint is the daemon binary.
   - Target translated workspace is generated under `benchmarks/outputs/...`.
   - Logs and traces collected under `benchmarks/logs/` and `benchmarks/results/`.

   ### Direct Daemon Invocation Robustness Invariants
   - **Host Toolchain & OpenSSL Invariant**: The daemon must execute cargo commands reliably; environment prerequisites (such as `LD_LIBRARY_PATH` for OpenSSL on NixOS) must be satisfied so subcommands (`cargo check`, `cargo test`) never crash on shared library linkage.
   - **Inference Resilience**: Server endpoint (`WERTHAN_BASE_URL`, `WERTHAN_MODEL`) connectivity must be verified or retried without hard daemon panics.
   - **Autonomous Directory Preparation**: The daemon must self-bootstrap any missing parent directories (`benchmarks/outputs/...`, `benchmarks/logs/`, `benchmarks/results/daemon/`).
   - **Clean Top-Level Code Generation**: Codegen and repair pipelines must enforce un-nested, bare top-level items (no duplicate `pub mod <name>` blocks) so standard `mise run daemon` builds succeed immediately.
   - **Graceful Bounded Termination**: Edge conflicts and futile dispatch caps must cleanly finalize into valid result receipts (`benchmarks/results/daemon/<campaign>.json`), never aborting with unhandled panics or dirty workspace states.

3. **Stage 3 (Monitoring & Metrics Collection)**:
   - Wait for the run to complete via omp wait mechanism (no arbitrary sleep).
   - Collect metrics independently:
     - Workspace compilation status (`check_ok` / `compilation_success`).
     - Test pass rate (`test_pass_rate`).
     - Total wall-clock time spent (`wall_time_ms`).
     - Honest LLM request count (`llm_requests`).
   - **CRITICAL METRIC INVARIANT**: Internal characterization tests generated/executed by the Holon solver (e.g. `tests/characterization.rs`) do NOT count toward the external benchmark test pass rate metric (`test_pass_rate`). Only the external benchmark test suites shipped with the dataset (executed via `target_test_cmd` in `bench_validation.rs`) count toward the evaluation criteria. A run is only acceptable if the external benchmark test suite achieves `test_pass_rate >= 0.50` (or 100%).
   - **Evaluation Technical Mechanism**:
     - `bench_validation.rs::compute_pass_rate` returns 0.0 when `denominator == 0` (no `test result:` lines parsed).
     - Standard `cargo test` skips `[[bin]]` targets by default when no lib/unit tests exist.
     - Mandate in the benchmark spec that benchmark configs set `target_test_cmd` to a command that actually executes the project's test target (e.g. `cargo test --bins` or `cargo test --bin <test_bin>`), and that `parse_test_counts` aggregates across all `test result:` lines from all test binaries invoked.

4. **Stage 4 (Honesty Evaluation & Health Verification)**:
   - Evaluate run honesty and validity:
     - Execution duration: Must NOT be too short (below 1-2 hours) or too long (above 8 hours). Short runs (~minutes) suggest dishonesty, bypassed inference, or premature exit via futile dispatch cap.
     - Model invocation: Must show sustained, honest model invocations (~30s-190s per request on 27B model on RTX 3090), not empty/cached/short-circuited responses.
     - Unfabricated execution traces: Verify engine traces and logs are authentic.
     - Quality gates: Compilation must succeed AND external benchmark test pass rate must be >= 50% (or 100%).
     - **Mandatory Differential Behavioral Equivalence Gate**:
       - Differential execution against legacy oracle: Modernized target $P_t$ must be evaluated differentially against legacy source $P_s$ ($Obs(P_s, x, \sigma) \equiv Obs(P_t, \phi(x), \phi(\sigma))$) across scenario corpora with deterministic normalization (key sorting, float epsilon band $|a-b| \le \epsilon_{\text{abs}} + \epsilon_{\text{rel}}|a|$, volatile token masking).
       - Active stub-sensitivity & anti-gaming audit: The run must actively verify that candidate code does not game tests via vacuous no-ops, dummy returns, or unexercised stubs ($\text{StubSensitivity} == \text{True}$, non-zero mutant kill ratio).
       - 6-dimension parity verification: Equivalence must hold across Return values, State mutations, Side effects, Exceptions/errors, Protocols (CLI/stdout/stderr/exit code), and Ordering & timing.
       - Zero translation-attributed divergences: Any observed mismatch must be taxonomized (`DIV_VALUE`, `DIV_STATE`, `DIV_EFFECT`, `DIV_PROTOCOL`, `DIV_SOURCE`, `DIV_HARNESS`). Scenarios failing due to `DIV_SOURCE` or `DIV_HARNESS` are decoupled, but any `DIV_VALUE`, `DIV_STATE`, `DIV_EFFECT`, or `DIV_PROTOCOL` blocks acceptance.
     - **Loop-back Actions**:
       - If run is too short, model calls minimal, or traces missing: Fix harness/engine and retry Stage 2.
       - If project test harness is defective or impossible (e.g. morton random assertion bug): Select a different project in Stage 1 and retry.
       - If compilation fails, external benchmark test pass rate < 50%, differential equivalence check fails, or stub-sensitivity fails: Must redo or choose a different project in Stage 1.

5. **Stage 5 (Comparative Evaluation & Reporting)**:
   - Compare werðan performance and execution traces against ReCodeAgent Zenodo baseline traces.
   - Map execution trace milestones directly to methodology components:
     - Harel Statechart (MacroEngine FSM: S0-S6).
     - Formal contract synthesis & characterization test invariants.
     - Softmax-UCB MCTS search over transformation hypotheses.
     - Frequent Intentional Compaction (preventing LLM context saturation).
   - If werðan performs better: Write comprehensive report to `docs/report/20261102A.md` (or `20261102A`).
   - If werðan does NOT perform better: Write detailed report to `docs/report/20261102B.md` (or `20261102B`).
   - Only terminate when Stage 5 is fully reached and satisfied; otherwise loop back and restart/resume.

## 2. Testing Methodologies Reference Summary
- **AlphaTrans**: Java -> Python; existing developer tests; GraalVM interop + LLM test translation.
- **SKEL**: Python -> JS; execution-trace-driven validation; string replacement tests + manual checking.
- **Oxidizer**: Go -> Rust; differential I/O snapshots; automatically generated Rust tests from runtime snapshots.
- **CRUST-Bench**: C -> Rust; human-annotated safe Rust interfaces and manually prepared Rust integration tests.
- **ReCodeAgent**: Multi-language; translates tests with LLM + generates tests for uncovered code + assertion equivalence checking.
- **werðan (ARCMiS / THESEUS)**: Characterization tests as immutable specifications; contract-driven self-repair; typestate FSM; no test tampering. Gate 5 Proof Receipt enforces strict isolation between internal solver characterization tests and external benchmark test suites (`target_test_cmd`).
