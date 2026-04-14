# Sprint 04 - Diagnostics Example

## Goal

Add a second non-trivial example service that proves the generator and runtime handle more than the battery-service happy path.

## Why This Sprint Comes Now

With TCP transport in place, a richer example validates the overall design before the first release push.

## Scope

- Create a diagnostics-oriented ARXML fixture and generated code path.
- Add a runnable example under `examples/`.
- Cover methods, events, or fields that differ meaningfully from the battery service.
- Document how to run the example.

## Out Of Scope

- Publishing to crates.io
- Docs site generation
- Cross-compilation or Yocto integration

## Suggested PR Breakdown

1. ARXML fixture and generated code expectations
2. Diagnostics example runtime
3. README or docs updates

## Implementation Checklist

- Choose a service shape that exercises complexity the battery example does not.
- Prefer covering one or more of:
  - multiple methods
  - richer payload types
  - event flow
  - field interaction
- Add integration checks to ensure the example still compiles as the generator evolves.
- Document the scenario and why it matters.

## Testing Requirements Before Merge

### Coverage Goals

- The diagnostics example must validate at least one capability not already proven by the battery example.
- Generated code, runtime behavior, and user-facing run instructions must all be exercised.

### Required Test Types

- Fixture-based codegen integration tests for the diagnostics ARXML.
- Build tests ensuring generated code compiles cleanly.
- Runtime integration tests for the example behavior.
- Smoke test for documented run commands.

### Required Positive Cases

- Diagnostics ARXML parses successfully.
- Generated diagnostics code compiles without manual edits.
- The example executable starts and performs its documented flow.
- Any richer payload, event, or field behavior works end to end.

### Required Negative Cases

- Invalid diagnostics fixture variants fail clearly if they violate generator assumptions.
- Example startup fails gracefully on missing config or unavailable transport prerequisites.

### Non-Regression Checks

- Battery-service example still builds and runs.
- Existing codegen output for unrelated fixtures does not regress unexpectedly.
- Example docs match the real commands and file paths.

### Merge Gate

- One automated test proves the diagnostics example is more than a compile-only demo.
- One automated check ensures the generated code stays reproducible.
- Existing example suite remains green.

## Exit Criteria

- A second real example builds and runs.
- The example demonstrates at least one meaningful capability not proven by the battery service.
- Documentation tells users how to try it quickly.

## Merge Notes

Keep generated artifacts reproducible. If the example exposes generator gaps, fix those in narrowly scoped supporting PRs rather than hiding them inside the example code.
