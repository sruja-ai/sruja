#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/_common.sh"

ROOT="$(project_root)"
SRUJA_BIN="$(ensure_sruja_cli "$ROOT")"

usage() {
  cat <<'EOF'
service_detection_sanity.sh

Quick local sanity check for service detection changes.

Requires repos already present under:
  /tmp/sruja_test_terraform
  /tmp/sruja_test_vscode
  /tmp/sruja_test_kubernetes

Usage:
  ./scripts/testing/service_detection_sanity.sh

EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

echo "Testing Enhanced Service Detection"
echo "=================================="
echo ""

TARGET_REPO="/tmp/sruja_test_terraform"
if [ -d "$TARGET_REPO" ]; then
    echo "Testing Terraform (CLI tool - should have 0 services)..."
    "$SRUJA_BIN" quickstart -r "$TARGET_REPO" 2>&1 | grep -E "services|Health Score"
    echo ""
fi

TARGET_REPO="/tmp/sruja_test_vscode"
if [ -d "$TARGET_REPO" ]; then
    echo "Testing VSCode (should have fewer services, more accurate)..."
    "$SRUJA_BIN" quickstart -r "$TARGET_REPO" 2>&1 | grep -E "services|Health Score"
    echo ""
fi

TARGET_REPO="/tmp/sruja_test_kubernetes"
if [ -d "$TARGET_REPO" ]; then
    echo "Testing Kubernetes (should have similar or more services)..."
    "$SRUJA_BIN" quickstart -r "$TARGET_REPO" 2>&1 | grep -E "services|Health Score"
    echo ""
fi

echo "Done!"
