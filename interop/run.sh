#!/usr/bin/env bash
# Run the ara-rs <-> vsomeip interop demo.
#
# Prerequisites: Docker and Docker Compose
#
# Usage:
#   ./interop/run.sh
#
# The script builds both containers, runs the demo, and reports success/failure
# based on the vsomeip client exit code.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$REPO_ROOT"

echo "=== Building interop containers ==="
docker compose -f interop/docker-compose.yml build

echo ""
echo "=== Running interop demo ==="
echo "  ara-rs server: BatteryService (0x4010) on 172.20.0.10:30509"
echo "  vsomeip client: GetVoltage(battery_id=1) from 172.20.0.20"
echo ""

# Run with abort-on-container-exit so we stop when the client finishes.
# Capture the exit code of the vsomeip-client container.
set +e
docker compose -f interop/docker-compose.yml up \
    --abort-on-container-exit \
    --exit-code-from vsomeip-client
EXIT_CODE=$?
set -e

echo ""
echo "=== Cleaning up ==="
docker compose -f interop/docker-compose.yml down --volumes --remove-orphans 2>/dev/null || true

if [ "$EXIT_CODE" -eq 0 ]; then
    echo ""
    echo "=== INTEROP TEST PASSED ==="
    echo "vsomeip C++ client successfully called ara-rs Rust server over SOME/IP."
else
    echo ""
    echo "=== INTEROP TEST FAILED (exit code: $EXIT_CODE) ==="
    echo "Check container logs above for details."
fi

exit "$EXIT_CODE"
