# Sprint 09 - CXX Bridge

## Goal

Add a narrow C++ bridge that proves mixed Rust and C++ stacks are practical without overcommitting to a large ABI surface too early.

## Why This Sprint Comes Now

The bridge should build on already-proven transport and interop behavior, not become the place where those questions are first discovered.

## Scope

- Implement one narrow `cxx` bridge path.
- Limit the surface to one service and one direction.
- Document ownership, build assumptions, and limitations.

## Out Of Scope

- Full-language coverage for all generated services
- broad API stabilization for every future interop case

## Suggested PR Breakdown

1. Build system and dependency setup
2. Narrow bridge implementation
3. Example and docs

## Implementation Checklist

- Pick the smallest service boundary that proves the approach.
- Define explicit ownership between generated Rust, bridge code, and C++ caller or callee.
- Ensure the bridge does not distort the main Rust API design.
- Add one integration or smoke test if the environment can support it.

## Testing Requirements Before Merge

### Coverage Goals

- Full validation of the single supported bridge path.
- Build-system coverage for Rust, C++, and generated bridge artifacts.
- Negative coverage around ABI assumptions, ownership boundaries, and error propagation.

### Required Test Types

- Build test for the bridge on a clean environment.
- End-to-end smoke or integration test for the one supported direction.
- Basic memory and ownership sanity checks where tooling allows.
- Regression test confirming existing Rust-only APIs still behave unchanged.

### Required Positive Cases

- Bridge code compiles consistently.
- The supported C++ to Rust or Rust to C++ flow works end to end.
- Data passed across the bridge is correct for the chosen service contract.

### Required Negative Cases

- Build failure due to missing C++ prerequisites is detected clearly.
- Unsupported bridge usage is rejected or documented rather than failing ambiguously.
- Error propagation across the bridge behaves deterministically.

### Non-Regression Checks

- Rust-only examples and tests still pass.
- The bridge does not force unrelated public API changes without coverage and documentation.

### Merge Gate

- The supported bridge path has one automated integration or smoke test.
- Clean-environment build instructions have been validated.
- Limitations are documented based on real test observations, not assumptions.

## Exit Criteria

- One documented C++ to Rust or Rust to C++ path works.
- The scope is intentionally narrow and maintainable.
- Limitations and next steps are written down clearly.

## Merge Notes

Keep this sprint humble. The goal is credibility, not full coverage.
