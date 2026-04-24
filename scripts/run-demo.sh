#!/usr/bin/env bash
# AICrypto demo runner — runs a binary in demo mode (auto-exits after completion)
# Usage: ./scripts/run-demo.sh <binary-name> [cargo args...]
# Example: ./scripts/run-demo.sh signal-runtime
#          ./scripts/run-demo.sh api-gateway -- --test-flag

set -euo pipefail

BINARY_NAME="${1:?Usage: run-demo.sh <binary-name> [cargo args...]}"
shift

LOG_FILE="/tmp/aicrypto-${BINARY_NAME}-demo.log"

echo "Running ${BINARY_NAME} in demo mode..."
cargo run --bin "${BINARY_NAME}" --no-default-features "$@" > "${LOG_FILE}" 2>&1
EXIT_CODE=$?

if [ ${EXIT_CODE} -eq 0 ]; then
    echo "✅ ${BINARY_NAME} completed successfully (exit code 0)"
else
    echo "❌ ${BINARY_NAME} exited with code ${EXIT_CODE}"
fi

echo ""
echo "--- Output (${LOG_FILE}) ---"
grep -E "INFO|WARN|ERROR" "${LOG_FILE}" | head -50
