#!/usr/bin/env bash
# Run workspace test coverage and report.
# Excludes sruja-wasm from the report (browser/WASM target, not exercised by cargo test).
# To reach 80%+: run this script; add tests for low-coverage modules if needed.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

EXCLUDE_FROM_REPORT="${EXCLUDE_FROM_REPORT:-sruja-wasm}"

echo "Running: cargo llvm-cov --all-features --workspace --no-clean --exclude-from-report $EXCLUDE_FROM_REPORT"
cargo llvm-cov --all-features --workspace --no-clean --exclude-from-report "$EXCLUDE_FROM_REPORT" "$@"
