# Contributing to ara-rs

Thanks for your interest in contributing! This guide covers everything you need to get started.

## Getting Started

### Prerequisites

- Rust current stable (intended MSRV 1.75, not yet CI-enforced)
- Linux (required for SOME/IP socket tests)
- A C++17 compiler (only needed for the `cxx-bridge` example)

### Setup

```bash
git clone https://github.com/ZiadAhmed9/ara-rs.git
cd ara-rs
cargo build --workspace
cargo test --workspace
```

### Project Structure

```
ara-rs/
├── cargo-arxml/       ARXML parser, validator, and code generator
├── ara-com/           Transport-agnostic async traits
├── ara-com-someip/    SOME/IP transport backend
├── examples/          Battery service, diagnostics service, CXX bridge
├── interop/           Docker-based vsomeip interop demo
├── meta-ara-rs/       Yocto meta-layer
├── docs/              mdBook user guide
└── Documentation/     Architecture, phases, sprint definitions
```

## How to Contribute

### Reporting Bugs

Use the [bug report template](https://github.com/ZiadAhmed9/ara-rs/issues/new?template=bug_report.yml). Include:
- Steps to reproduce
- Expected vs. actual behavior
- Rust version (`rustc --version`) and OS

### Suggesting Features

Use the [feature request template](https://github.com/ZiadAhmed9/ara-rs/issues/new?template=feature_request.yml). Describe the use case, not just the solution.

### Submitting Pull Requests

1. Fork the repo and create a branch from `master`
2. Make your changes
3. Ensure all checks pass:
   ```bash
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt --all -- --check
   ```
4. If you added a new public API, add rustdoc
5. If you changed behavior, update the relevant mdBook page in `docs/`
6. Open a PR against `master` using the pull request template

### Good First Issues

Issues labeled [`good first issue`](https://github.com/ZiadAhmed9/ara-rs/labels/good%20first%20issue) are scoped for newcomers. They include context on what to change and where to look. If you want to work on one, comment on the issue so others know it's taken.

## Code Style

- **Formatting**: `cargo fmt` (enforced in CI)
- **Linting**: `cargo clippy -- -D warnings` (enforced in CI)
- **Error handling**: Use `thiserror` for library errors. No `.unwrap()` in library code.
- **Async**: `async-trait` for trait definitions. `tokio` is the default runtime.
- **Naming**: Rust API guidelines — `snake_case` methods, `PascalCase` types
- **Comments**: Only where the logic isn't self-evident. No boilerplate doc comments on obvious methods.
- **Safety**: Minimize `unsafe`. Justify any use in a `// SAFETY:` comment.

## Commit Messages

- Use imperative mood: "Add field support" not "Added field support"
- First line under 72 characters
- Reference issues with `#123` when relevant
- One logical change per commit

## Testing

- Unit tests go next to the code they test (in-module `#[cfg(test)]`)
- Integration tests go in `tests/` within each crate
- Transport tests use loopback (`127.0.0.1`) with random ports
- Run benchmarks with `cargo bench -p ara-com-someip`

## Architecture Decisions

Major design decisions are documented in `Documentation/architecture.md`. If your change affects the architecture (new transport backend, codegen pipeline change, public trait modification), discuss it in an issue first.

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project: MIT OR Apache-2.0.
