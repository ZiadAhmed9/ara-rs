# Release Checklist — ara-rs v0.1.0

## Pre-publish verification

- [x] `cargo test --workspace` — all tests green
- [x] `cargo clippy --workspace` — zero warnings
- [x] `cargo fmt --all -- --check` — formatting clean
- [x] `cargo package --allow-dirty -p ara-com` — packages and verifies
- [x] `cargo package --allow-dirty -p cargo-arxml` — packages and verifies
- [x] `ara-com-someip` — validated via `cargo package --list`, `cargo check`, `cargo test` (all 15 tests pass), and metadata inspection. Full `cargo package` tarball creation requires `ara-com` to be on crates.io first; run `cargo package -p ara-com-someip` after step 1 of the publish order below to complete this gate.

## Package contents verified

Each crate includes:
- [x] `LICENSE-MIT` and `LICENSE-APACHE`
- [x] `README.md`
- [x] All source files and tests
- [x] `Cargo.toml` with complete metadata (description, keywords, categories, repository, homepage, readme, license)

## Crate metadata audit

| Field | ara-com | ara-com-someip | cargo-arxml |
|-------|---------|----------------|-------------|
| version | 0.1.0 | 0.1.0 | 0.1.0 |
| license | MIT OR Apache-2.0 | MIT OR Apache-2.0 | MIT OR Apache-2.0 |
| description | present | present | present |
| keywords | 5 | 4 | 5 |
| categories | 2 | 2 | 2 |
| repository | set | set | set |
| homepage | set | set | set |
| readme | README.md | README.md | README.md |

## Publish order

Crates must be published in dependency order:

```
1. cargo publish -p ara-com
2. Wait for ara-com to appear in the crates.io index, then:
   cargo package -p ara-com-someip    # final tarball gate — must pass before publishing
   cargo publish -p ara-com-someip
3. cargo publish -p cargo-arxml       # independent, but publish last for consistency
```

Wait for each crate to appear in the crates.io index before publishing the next one.
The `examples/` crates have `publish = false` and are not published.

## Post-publish

- [ ] Verify crate pages render correctly on crates.io
- [ ] Tag release: `git tag v0.1.0 && git push origin v0.1.0`
- [ ] Create GitHub release with CHANGELOG.md content
- [ ] Update README.md badge links if adding crates.io badges
