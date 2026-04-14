# Sprint 02 - Public API Docs

## Goal

Make `ara-com` and `ara-com-someip` feel publishable and reviewable by giving all public APIs meaningful rustdoc coverage.

## Why This Sprint Comes Now

After Sprint 01, the next highest leverage improvement is developer trust. Good rustdoc also exposes awkward APIs before release prep begins.

## Scope

- Add rustdoc to all public types, traits, functions, and modules in `ara-com`.
- Add rustdoc to all public types, traits, functions, and modules in `ara-com-someip`.
- Fix any public naming or signature confusion discovered during documentation.

## Out Of Scope

- GitHub Pages or mdBook publishing
- Feature work unrelated to docs clarity

## Suggested PR Breakdown

1. `ara-com` rustdoc pass
2. `ara-com-someip` rustdoc pass
3. Small API polish discovered during the pass

## Implementation Checklist

- Run through public exports crate by crate.
- Document purpose, invariants, and basic usage for:
  - transport traits
  - proxy and skeleton helpers
  - error types
  - configuration structs
  - service discovery types
- Add minimal examples where signatures are not self-explanatory.
- Check `cargo doc` output for broken links and awkward module structure.

## Testing Requirements Before Merge

### Coverage Goals

- All public items touched in this sprint are validated by docs generation.
- Every doctest or code sample added in rustdoc must compile or be explicitly marked as non-runnable with justification.

### Required Test Types

- `cargo doc` build validation for the workspace.
- Doctest execution for runnable examples.
- Broken-link and intra-doc-link checks.
- API surface review to ensure newly documented invariants match actual behavior.

### Required Positive Cases

- Public docs render successfully for `ara-com`.
- Public docs render successfully for `ara-com-someip`.
- Examples in rustdoc compile against the current API.

### Required Negative Cases

- Broken intra-doc links fail the sprint and must be fixed before merge.
- Stale examples or mismatched signatures fail the sprint and must be fixed before merge.

### Non-Regression Checks

- Documentation changes do not accidentally expose private internals or alter public exports unintentionally.
- Any API renames introduced for clarity are reflected in code, docs, and examples consistently.

### Merge Gate

- `cargo doc` completes cleanly.
- No broken links remain.
- Any rustdoc example included in public APIs has been compile-checked.

## Exit Criteria

- All public APIs in `ara-com` and `ara-com-someip` have useful rustdoc.
- Docs explain invariants such as instance binding and event-group routing.
- `cargo doc` completes cleanly.

## Merge Notes

Avoid mixing large behavior changes into this sprint. Small API renames are fine if they clearly improve public ergonomics and are easy to review.
