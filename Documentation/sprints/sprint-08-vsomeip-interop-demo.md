# Sprint 08 - vsomeip Interop Demo

## Goal

Demonstrate live interoperability with `vsomeip` using a reproducible Docker Compose setup.

## Why This Sprint Comes Now

Interop is a strong trust signal, but it depends on transport behavior and packaging being stable enough that failures are meaningful rather than moving-target noise.

## Scope

- Build a Docker Compose demo involving ara-rs and `vsomeip`.
- Prove one realistic request and response or pub/sub flow across the boundary.
- Document setup, commands, and expected results.

## Out Of Scope

- CXX bridge generation
- benchmark comparisons beyond what is needed to validate interop

## Suggested PR Breakdown

1. Containerized demo environment
2. Runtime wiring and smoke test
3. Documentation and troubleshooting notes

## Implementation Checklist

- Choose the smallest demo that proves real compatibility.
- Keep message definitions narrow and explicit.
- Capture expected packet flow and success criteria.
- Make the demo easy to rerun in CI or locally if practical.

## Testing Requirements Before Merge

### Coverage Goals

- Full end-to-end validation of the documented interop scenario.
- Evidence that the demo is reproducible outside the author machine.
- Failure coverage for startup ordering, network assumptions, and incompatible configuration.

### Required Test Types

- Docker Compose startup and teardown smoke test.
- End-to-end interop test for the documented request and response or pub/sub scenario.
- Log or packet-level verification confirming that both sides exchanged the intended messages.
- Repeatability test to ensure the demo is not flaky across multiple runs.

### Required Positive Cases

- All containers start successfully from a clean state.
- ara-rs discovers or reaches `vsomeip` as documented.
- The chosen business flow completes successfully.
- Logs or assertions confirm the expected message path occurred.

### Required Negative Cases

- Startup-order variation does not cause silent success or indefinite hangs.
- Missing container dependency or bad network configuration fails clearly.
- Service mismatch or incompatible IDs fail with actionable diagnostics.

### Non-Regression Checks

- Existing native transport and integration tests remain green.
- Demo docs exactly match the compose file, image names, and commands in the repo.

### Merge Gate

- The Docker Compose demo has at least one automated end-to-end validation path.
- The interop scenario passes repeatedly from a clean environment.
- Troubleshooting documentation covers the most likely failure modes seen during testing.

## Exit Criteria

- Docker Compose demo starts successfully.
- A documented ara-rs and `vsomeip` interaction works end to end.
- Reproduction steps are simple enough for external users.

## Merge Notes

Avoid widening the demo scope too early. One clean interop success is better than a broad but fragile setup.
