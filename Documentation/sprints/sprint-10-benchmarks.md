# Sprint 10 - Benchmarks

## Goal

Add reproducible benchmarks that provide performance confidence once the architecture and interop story have stabilized.

## Why This Sprint Comes Last

Benchmark work gets invalidated easily if transport, codegen, or interop behavior is still moving. Doing it late keeps the numbers more meaningful.

## Scope

- Add `criterion` benchmarks for key paths.
- Measure at least request and response, plus one serialization-heavy or pub/sub path.
- Document environment, methodology, and caveats.

## Out Of Scope

- Premature micro-optimization unrelated to a measured problem
- marketing claims without reproducible setup

## Suggested PR Breakdown

1. Benchmark harness and baseline scenarios
2. Result reporting and documentation
3. Optional follow-up fixes for obvious regressions

## Implementation Checklist

- Pick benchmarks that map to real user questions.
- Keep benchmark fixtures stable and easy to rerun.
- Record environment details so results are interpretable.
- If comparing against `vsomeip`, document fairness limits and configuration differences.

## Testing Requirements Before Merge

### Coverage Goals

- Benchmark harness correctness must be validated before performance numbers are trusted.
- Results must be reproducible enough to compare runs meaningfully.

### Required Test Types

- Smoke test that benchmark targets build and execute.
- Validation tests for benchmark fixture correctness.
- Repeatability check across multiple runs on the same environment.
- Optional sanity comparison against expected order-of-magnitude ranges if baselines exist.

### Required Positive Cases

- Each benchmark scenario runs to completion.
- Measured paths correspond to the documented code paths.
- Benchmark docs capture hardware, OS, toolchain, and configuration details.

### Required Negative Cases

- Invalid benchmark setup or missing fixture data fails clearly.
- Misleading comparisons, such as incompatible `vsomeip` configuration, are documented or blocked.
- Benchmarks that accidentally measure setup noise instead of steady-state behavior are corrected before merge.

### Non-Regression Checks

- Adding the benchmark harness does not break normal workspace builds or tests.
- Benchmark code remains isolated from production code paths unless explicitly intended.

### Merge Gate

- Benchmark harness is runnable in a documented environment.
- Results are reproducible enough to distinguish signal from noise.
- Methodology and caveats are written down beside the reported numbers.

## Exit Criteria

- Benchmarks run reproducibly.
- Results are documented with enough context to interpret them.
- Any major regressions are identified clearly, even if they are deferred.

## Merge Notes

Do not mix benchmark harness setup with broad optimization changes. First make measurement trustworthy, then optimize in separate work if needed.
