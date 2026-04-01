#!/bin/bash
set -e
cd /home/ziad/playground/ara-rs

echo "=== build ==="
cargo build -p cargo-arxml 2>&1

echo ""
echo "=== probe tests (--nocapture) ==="
cargo test -p cargo-arxml --test probe_deployment_schema -- --nocapture --test-threads=1 2>&1 || true

echo ""
echo "=== full cargo-arxml tests ==="
cargo test -p cargo-arxml 2>&1

echo ""
echo "=== workspace clippy ==="
cargo clippy --workspace -- -D warnings 2>&1

echo ""
echo "=== fmt check ==="
cargo fmt --all -- --check 2>&1

echo ""
echo "=== all workspace tests ==="
cargo test --workspace 2>&1
