# Sprint 06 - mdBook Docs Site

## Goal

Publish a user-facing docs site that turns the workspace from “read the README and source” into a navigable product.

## Why This Sprint Comes Now

The docs site is much more stable once the core APIs and release posture are settled.

## Scope

- Set up mdBook structure.
- Add user guide content for installation, architecture, codegen flow, runtime usage, and examples.
- Publish to GitHub Pages.

## Out Of Scope

- Deep API documentation already handled by rustdoc
- New transport or interop features

## Suggested PR Breakdown

1. mdBook scaffold and navigation
2. Core guide pages
3. GitHub Pages publishing workflow

## Implementation Checklist

- Create a minimal book structure with clear top-level chapters.
- Link to existing architectural and testing docs where useful.
- Include quickstarts for:
  - parsing and generating from ARXML
  - running battery service
  - running diagnostics example
- Add deployment automation for GitHub Pages.
- Ensure the published book does not duplicate rustdoc unnecessarily.

## Testing Requirements Before Merge

### Coverage Goals

- Full validation of documentation build, navigation, and publishing flow.
- All user-facing quickstart commands in the book must be validated against the current repo state.

### Required Test Types

- Local `mdBook` build test.
- Link validation across internal pages and outbound references where tooling supports it.
- Smoke test of the GitHub Pages workflow.
- Docs-command verification for all copy-paste command sequences.

### Required Positive Cases

- The book builds without warnings that indicate broken structure.
- Navigation works across chapters.
- Quickstart flows for codegen and examples match the real repository layout.
- GitHub Pages deployment workflow completes successfully in a test run or dry run.

### Required Negative Cases

- Broken links fail the sprint.
- Commands that no longer work fail the sprint until docs are corrected.
- Missing assets, images, or pages referenced by the book fail the sprint.

### Non-Regression Checks

- The mdBook does not drift from README and rustdoc on core setup steps.
- Published content does not reference files or commands removed by earlier sprints.

### Merge Gate

- mdBook builds cleanly.
- Link validation passes.
- Every documented quickstart path has been smoke-tested.

## Exit Criteria

- mdBook builds locally.
- GitHub Pages deployment path is defined and tested.
- A new user can understand the project flow without reading the source tree first.

## Merge Notes

Prefer one PR for content structure and one for publishing automation if that keeps review cleaner.
