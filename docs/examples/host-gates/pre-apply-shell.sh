#!/usr/bin/env bash
# Example: run before applying an agent-generated patch locally.
set -euo pipefail

REPO="${1:-.}"
PROFILE="${2:-coding}"

echo "== Sruja focus (optional) =="
# sruja focus -r "$REPO" --file path/to/changed.rs -f for-ai

echo "== Sruja verify-task ($PROFILE) =="
sruja verify-task --profile "$PROFILE" -r "$REPO" -f json

echo "OK: verify-task passed"
