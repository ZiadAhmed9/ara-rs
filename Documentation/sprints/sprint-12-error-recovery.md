# Sprint 12 — Error Recovery and Validator Hardening

## Goal

Make `cargo-arxml` resilient to malformed, partial, and messy ARXML files — the kind production teams actually feed into tools.

## Why Now

With the crate published, external users will try it on their own ARXML. Panics on bad input erode trust faster than missing features.

## Scope

- Graceful handling of malformed ARXML (no panics, actionable error messages)
- Source location in error messages (file, element path)
- Partial extraction: generate what you can, report what you can't
- Validator expansion: version consistency checks, cross-file reference validation
- Structured error output (JSON mode for CI integration)

## Out of Scope

- ARXML auto-repair or fixup suggestions
- Schema-level XSD validation

## Exit Criteria

- Feeding 5+ known-broken ARXML files produces helpful errors, not panics
- Error messages include file path and element context
- Existing valid ARXML produces identical output (no regression)
