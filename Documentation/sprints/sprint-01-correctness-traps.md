# Sprint 01 - Correctness Traps

## Goal

Remove the two highest-risk correctness issues before adding new features:

- Stop silently inventing service or method IDs.
- Make validator behavior honest: implement the advertised checks or remove unsupported API surface.

## Why This Sprint Comes First

These issues can create invalid generated artifacts and misleading user expectations. Every later sprint depends on the code generator and validator being trustworthy.

## Scope

- Fix auto-ID behavior so missing IDs become explicit warnings or errors.
- Implement `MissingTypeRef` and `InvalidMethodId`, or remove those variants from the public API and README claims.
- Align README, rustdoc, and CLI output with actual behavior.
- Add focused tests for each corrected behavior.

## Out Of Scope

- New transport features
- Docs site work
- Example expansion beyond test fixtures

## Suggested PR Breakdown

1. Auto-ID behavior change plus tests
2. Validator honesty change plus tests
3. README and API cleanup if needed

## Implementation Checklist

- Audit where IDs are synthesized in `cargo-arxml`.
- Decide whether missing IDs should be hard errors, warnings, or configurable.
- Ensure generated code never masks malformed ARXML by inventing identifiers silently.
- Audit `cargo-arxml/src/validator/mod.rs`, error types, and docs for claims that are not implemented.
- Add regression tests covering:
  - missing service or method IDs
  - unresolved type references
  - invalid or conflicting method IDs
- Update command output so failures explain what to fix in the ARXML.

## Testing Requirements Before Merge

### Coverage Goals

- Full decision coverage for the new ID validation paths.
- Full branch coverage for validator paths touched by `MissingTypeRef` and `InvalidMethodId`.
- Regression coverage for every bug fixed in this sprint.

### Required Test Types

- Unit tests for validator rules and error construction.
- Parser or codegen integration tests using realistic ARXML fixtures.
- CLI tests for user-visible failure messages where behavior changes.
- Regression tests proving the previous silent-success path no longer passes.

### Required Positive Cases

- Valid ARXML with explicit IDs passes validation and generation.
- Valid type references resolve cleanly.
- Valid method IDs remain accepted and generate stable output.

### Required Negative Cases

- Missing service IDs fail or warn exactly as designed.
- Missing method IDs fail or warn exactly as designed.
- Unresolved type references are detected and surfaced with actionable error text.
- Invalid method IDs are rejected.
- Conflicting IDs are rejected without partial generation side effects.

### Non-Regression Checks

- Existing valid fixtures continue to generate identical or intentionally updated output.
- No unrelated validator rules regress.
- Error variants exposed publicly match actual runtime behavior and docs.

### Merge Gate

- All new rules are covered by automated tests.
- At least one fixture-based integration test exercises each corrected path end to end.
- No known path remains where malformed ARXML silently produces generated code.

## Exit Criteria

- No silent ID invention remains in the supported path.
- Validator behavior matches the documented API.
- Tests fail before the fix and pass after it.
- README and rustdoc no longer overpromise.

## Merge Notes

Prefer landing behavior and tests together. If the public API changes, keep doc updates in the same sprint so downstream users are not left with stale guidance.
