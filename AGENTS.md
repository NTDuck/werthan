# werðan — Agent Entry Point

Read `docs/specs/INDEX.md` first: it routes every domain to its spec file, maps `ADR-XXXX` tags to `docs/decisions/` files, and defines the change protocol (which spec a code change must touch in the same commit).

## Operating contract

1. **Methodology ground truth**: `docs/references/METHODOLOGY.md` (with `docs/references/METHODOLOGY-DRV.md` for derivations) is read-only and supersedes every other architectural description. Pinning decision: [ADR-0005](docs/decisions/0005-methodology-source-of-truth.md).
2. **Specs stay in lockstep**: any change to code behavior, interfaces, commands, workspace crates, dependencies, or tools updates the corresponding spec file (see INDEX routing) in the same commit. Code contradicting a spec, or a spec contradicting code, is a defect.
3. **ADRs are append-only**: new decisions add a numbered file under `docs/decisions/` following the MADR template (`.omp/rules/references/madr/template/adr-template.md`); when a decision invalidates an older one, mark the old file's status `superseded by [NNNN-slug.md](NNNN-slug.md)` (real target file) and leave its body untouched.
4. **Living specs**: `docs/specs/**` is written in present tense with no historical narrative; history lives in `docs/decisions/` as the append-only ledger.
5. **FSM convention** (`.omp/rules/specs.md` §3): state machines use the state-as-struct Hoverbear pattern; unit-variant enums are reflection tags, never state machines.
6. **No hardcoded configuration** (`.omp/rules/`): operational parameters bubble up to `.env` ingested at compile time by the `envload` crate; see [ADR-0007](docs/decisions/0007-compile-time-env-ingestion.md) and [ADR-0011](docs/decisions/0011-env-config-parameters.md).
7. **Quality gates**: workspace linters (`flatmods`, `pathqual`, `plumbing`, `ste100`, `stepdown`, `turbofish`) run through every crate's `build.rs`; cucumber BDD suites verify engine and linter behavior (see `docs/specs/tech-spec/benchmark-harness.md` and [ADR-0014](docs/decisions/0014-cucumber-bdd-verification.md)).

## Canonical resources

| Resource | Role |
| :--- | :--- |
| `docs/specs/INDEX.md` | Spec router, ADR tag map, change protocol |
| `docs/references/METHODOLOGY.md` | Read-only methodology ground truth |
| `docs/references/METHODOLOGY-DRV.md` | Foundational derivations |
| `docs/decisions/0001` … `0014` | MADR decision ledger |
| `benchmarks/README.md` | Benchmark procedures and config reference |
| `.omp/rules/specs.md` | Spec synchronization rules (repo convention) |

## Functionality map

| Functionality | Entry point | Status |
| :--- | :--- | :--- |
| Interactive TUI cockpit | `werðan/bins/interactive` (`mise interactive`) | active |
| Headless benchmark daemon | `werðan/bins/daemon` (`mise daemon`, flags `--benchmark/--benchmark-all/--smoke/--dry-run`) | active |
| Global engine tier | `werðan/libs/global/*` (blackboard, engine, fsm, agents/{arbiter,workers,global-critics}) | active |
| Holon solver tier | `werðan/libs/holon/*` (blackboard, engine, fsm, agents/{strategy,codegen,testgen,patch,evaluator}) | active |
| Shared tools + config | `werðan/libs/shared/*` (tools, configschema, agents/configurator) | active |
| Linter gates | `werðan/.checks/*` (flatmods, pathqual, plumbing, ste100, stepdown, turbofish, envload) | active |
| Laya experiments | `werðan/.experiments/*` (`mise exp-laya-guardrails`, `exp-laya-as-a-judge`, `exp-laya-decision-model`) | active |
| Examples | `werðan/.examples/{mylib,mybin}` | active |

## Verification sweep

```bash
cargo check --workspace --tests
cargo clippy --workspace
cargo test --workspace            # includes cucumber suites (tools, holon-fsm, holon-engine, .checks)
cargo run -p daemon -- --smoke    # inference endpoint reachability
```

NixOS note: prefix test/run commands with `LD_LIBRARY_PATH` pointing at the host OpenSSL store path (re-derive with `nix eval nixpkgs#openssl.outPath` when stale). Conventional Commits enforced by the `commit-msg` hook (`.githooks/`, `core.hooksPath`).
