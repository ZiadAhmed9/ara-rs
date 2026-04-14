# Sprint 05 - Crates.io Release Prep

## Goal

Prepare the workspace for a clean first crates.io release without rushing into publication before metadata, docs, and examples are coherent.

## Why This Sprint Comes Now

By this point the core story is present: trustworthy generator behavior, documented APIs, transport support, and more than one example.

## Scope

- Audit crate metadata, versions, licensing, keywords, categories, repository links, and descriptions.
- Prepare changelog and release notes.
- Verify package contents for all crates.
- Decide whether publication happens at the end of this sprint or immediately after.

## Out Of Scope

- mdBook site hosting
- Yocto integration
- new runtime features

## Suggested PR Breakdown

1. Metadata and packaging cleanup
2. Release notes and changelog
3. Dry-run validation and publication checklist

## Implementation Checklist

- Review all `Cargo.toml` package metadata.
- Confirm readmes and crate-level docs match current behavior.
- Run `cargo package` or equivalent validation for each crate.
- Decide on versioning policy and initial release numbers.
- Create a release checklist covering:
  - tests green
  - docs up to date
  - examples compile
  - license files included
  - publish order across crates

## Testing Requirements Before Merge

### Coverage Goals

- Packaging validation for every publishable crate.
- Release artifact validation for metadata, included files, and installability expectations.
- Full pre-release regression pass on the workspace.

### Required Test Types

- `cargo package` or equivalent dry-run validation for each crate.
- Full workspace build and test run.
- Verification that packaged artifacts contain required license, README, and source files.
- Installability smoke check using packaged output where practical.

### Required Positive Cases

- Each crate packages successfully.
- Crate metadata is complete and accurate.
- Packaged readmes render the intended project description.
- Version constraints and inter-crate dependency versions resolve correctly.

### Required Negative Cases

- Missing required metadata, invalid categories, or packaging exclusions fail the sprint.
- Missing license or README artifacts fail the sprint.
- Broken path dependencies or bad publish order assumptions fail before publication.

### Non-Regression Checks

- The full workspace test suite remains green at release candidate state.
- Example projects still compile from the release-prep branch.
- No unpublished local-only paths remain in manifest content meant for crates.io.

### Merge Gate

- All crates pass dry-run packaging validation.
- Full workspace regression suite is green.
- A written release checklist exists and has been verified against the branch state.

## Exit Criteria

- Each crate is ready to publish without metadata surprises.
- Release notes exist.
- Packaging has been validated locally.

## Merge Notes

Treat actual `cargo publish` as a milestone step, not a hidden side effect inside unrelated code changes.
