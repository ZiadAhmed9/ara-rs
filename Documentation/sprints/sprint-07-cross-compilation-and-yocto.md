# Sprint 07 - Cross Compilation And Yocto

## Goal

Prove the project fits embedded Linux target workflows, especially the Yocto-based environments common in automotive programs.

## Why This Sprint Comes Now

Target-platform integration is important, but it should not churn while the core product is still moving quickly.

## Scope

- Add cross-compilation guidance or CI checks for target architectures.
- Create a Yocto recipe or meta-layer starter.
- Validate at least one realistic target build path.

## Out Of Scope

- vsomeip interoperability work
- C++ bridge implementation
- performance benchmarking

## Suggested PR Breakdown

1. Cross-compilation setup and verification
2. Yocto recipe or layer
3. Docs and CI follow-up

## Implementation Checklist

- Pick target triples that match expected users.
- Document host prerequisites and target assumptions.
- Create Yocto packaging inputs for the crates that need to ship.
- Validate generated code and runtime crates in the chosen target flow.
- Add CI where it provides steady value without making iteration painful.

## Testing Requirements Before Merge

### Coverage Goals

- Reproducible validation for each declared target path.
- Packaging and build validation for both raw cross-compilation and Yocto recipe flow.
- Clear failure coverage for unsupported or misconfigured environments.

### Required Test Types

- Cross-compilation build tests for each declared target triple.
- Smoke test for at least one example or binary on the target path when feasible.
- Yocto recipe parse and build validation.
- CI validation for any newly added target jobs.

### Required Positive Cases

- All selected target triples build successfully.
- The Yocto recipe or layer is accepted by the Yocto tooling.
- At least one target artifact is proven runnable or link-complete as designed.

### Required Negative Cases

- Missing toolchain prerequisites are detected with actionable guidance.
- Unsupported target combinations fail cleanly.
- Recipe dependency mistakes fail in validation rather than surfacing later during release work.

### Non-Regression Checks

- Native x86_64 build and test flow remains green.
- Added target support does not break local developer workflows unnecessarily.
- Example generation and runtime crates remain packageable after cross-target changes.

### Merge Gate

- Every claimed target path has automated validation or an explicit documented exception.
- Yocto recipe validation has been run successfully.
- Native build remains green after target support changes.

## Exit Criteria

- At least one cross-compiled target build is reproducible.
- Yocto recipe or layer structure exists and is documented.
- Users can answer “can I run this on my target?” with confidence.

## Merge Notes

Separate environment setup from recipe logic when possible so failures are easier to debug and review.
