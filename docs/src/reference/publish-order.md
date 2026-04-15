# Publish Order

The three ara-rs crates must be published to crates.io in dependency order.

## Dependency Chain

```
ara-com             ← no workspace dependencies (publish first)
    ▲
    │
ara-com-someip      ← depends on ara-com
    
cargo-arxml         ← no workspace dependencies (independent)
```

## Publish Sequence

```bash
# 1. Publish the core traits crate
cargo publish -p ara-com

# 2. Wait for ara-com to appear in the crates.io index, then validate and publish
cargo package -p ara-com-someip   # final tarball gate
cargo publish -p ara-com-someip

# 3. Publish the code generator
cargo publish -p cargo-arxml
```

Wait for each crate to appear in the crates.io index before publishing the next one. The `examples/` crates have `publish = false` and are not published.

## Version Policy

All three crates share the same version number (currently `0.1.0`). Bump versions together to keep the workspace consistent.

For the full pre-publish checklist, see [RELEASE_CHECKLIST.md](https://github.com/ZiadAhmed9/ara-rs/blob/master/RELEASE_CHECKLIST.md).
