---
description: Configuration values must not be hardcoded throughout the codebase. They must bubble up to main() or a dedicated config file/struct. Distinguish hardcoded constants (algorithm magic numbers, enum discriminants) from runtime configuration.
alwaysApply: true
---

# No Hardcoded Configuration

Configuration values must not be scattered as literals throughout the codebase. They must bubble up to `main()` or a dedicated config struct/file so they are discoverable, overridable, and auditable in one place.

## 1. What Counts as Configuration (Must Externalize)

- **Model Identifiers, API Endpoints, Base URLs**: e.g., LLM model names, inference endpoints, proxy URLs.
- **Timeouts, Retry Counts, Rate Limits, Concurrency Caps**: e.g., worker pool shards, process timeouts, self-healing retries.
- **File and Directory Paths**: Input legacy paths, output workspace paths, cache dirs that vary across environments.
- **Feature Flags & Operational Toggles**: Dynamic runtime behavioral toggles.
- **Default Values for User-Tunable Parameters**: Declared exclusively in canonical `Default` implementations of dedicated configuration schemas.
- **Magic Strings / Environment Lookups**: `std::env::var(...)` lookups scattered in lower-level libraries.

## 2. What Does NOT Count as Configuration (May Remain Hardcoded)

Do not confuse immutable domain/algorithmic constants with runtime user configuration:
- **Algorithm Constants**: Fixed mathematical values inherent to algorithm design (e.g., `1.414` for $\sqrt{2}$ in UCB exploration, fixed mathematical weight vectors).
- **Enum Discriminants & Protocol Match Arms**: Syntax tokens, AST node types, and protocol state identifiers defined by language or specification standards.
- **Mathematical Identities**: Fixed unit conversion ratios (e.g., `60` seconds per minute, `1024` bytes per kibibyte).
- **Compile-Time Type Tags & Protocol Sentinels**: Static serialization markers and type discriminators.
- **Test Fixtures & Assertion Literals**: Deterministic unit test expectations.

## 3. Where Configuration Must Live

- **Primary**: A dedicated configuration schema struct (e.g., `ModernizeConfig`, `ModernizeInference`) loaded from a declarative file (`migration.yml`, YAML, TOML, JSON).
- **Secondary**: `main()` or CLI entrypoints, which parse user arguments / environment flags, construct or override the configuration struct, and pass it down.
- **Never**: Scattered as string/numeric literals or ad-hoc `std::env::var(...)` queries inside inner libraries, worker tasks, or helper functions.

## 4. Enforcement & Invariants

- **Explicit Threading**: Configuration must be passed down through constructors or parameter lists rather than queried globally or hardcoded locally.
- **Defect Classification**: Hardcoded configuration literals and buried environment lookups in non-`main` domain code are defects.
- **Consolidation**: When refactoring, extract scattered configuration parameters into the dedicated configuration struct; do not introduce competing defaults.
