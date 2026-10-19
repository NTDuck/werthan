# Pi-Warden Guardrail Rules

Campaign: {{campaign_id}}
System: {{sys}}
Workspace: {{workspace_root}}

Pi-warden enforces strict pre-execution tool interception and post-execution completion validation for all agents operating within the modernization pipeline. Any tool call or completion attempt violating these invariants is immediately rejected and held.

## Invariant 1: Scoped Write Isolation
- **Rule**: Write tools (`write`, `edit`, `replace`, `insert`) are ONLY permitted within `modernized/<sys>/` (rendered: `{{workspace_root}}/modernized/{{sys}}/`).
- **Restriction**: NEVER write, edit, overwrite, append, or delete files outside `modernized/<sys>/`.
- **Legacy Protection**: The legacy source tree (`legacy/<sys>/` or `{{input_legacy_dir}}`) is strictly read-only (`chmod a-w`). Any modification attempt outside `modernized/<sys>/` will trigger an immediate execution halt with `ToolViolationHeld`.

## Invariant 2: Zero Placeholder Stubs & Incomplete Code
- **Rule**: Translated target code must be complete, compiling, and fully implemented.
- **Forbidden Patterns**:
  - `TODO` comments or deferred implementation markers.
  - `unimplemented!()`, `todo!()`, or `panic!("not implemented")` stubs.
  - Placeholder return values, dummy mock fallbacks, or no-op handlers in production paths.
- **Enforcement**: Any target file containing placeholder stubs will fail the transformation gate and trigger an automated self-healing rewrite cycle.

## Invariant 3: Verified Clean-Room Test Execution
- **Rule**: No agent may claim `"done"` or declare task completion without prior verified execution of clean-room test suites.
- **Verification Requirements**:
  - The test tool or command (`{{target_test_cmd}}`) must be executed via the authorized runner before completion is signaled.
  - Zero tests executed (`tests_run == 0` or all skipped) is classified as a verification failure (`VerificationError::ZeroTestsExecuted`).
  - Output differential tests must demonstrate semantic equivalence against legacy baseline fixtures.
- **Enforcement**: Completion claims without verified test execution records are rejected with `PiWardenVerdict::Block`.

## Invariant 4: Hashline Anchor Integrity
- **Rule**: Code modifications performed by translation agents must preserve content-hashed line anchors to prevent offset drift and silent corruption.
