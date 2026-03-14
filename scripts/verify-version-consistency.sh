#!/usr/bin/env bash
# Verify that all release-please–tracked versioned artifacts are in sync.
# Run from repo root. Used by CI to ensure crates, extension, and manifest stay consistent.
set -euo pipefail

REPO_ROOT="${1:-.}"
cd "$REPO_ROOT"

# Versions that must all match (single source of truth when release-please runs)
CARGO_VERSION=$(cargo metadata --no-deps --format-version 1 2>/dev/null | jq -r '.packages[0].version')
EXT_VERSION=$(jq -r '.version' extension/package.json)
LOCK_TOP=$(jq -r '.version' extension/package-lock.json)
LOCK_ROOT=$(jq -r '.packages[""].version' extension/package-lock.json)
MANIFEST_VERSION=$(jq -r '.["."]' .release-please-manifest.json)

err=0
check() {
  local name="$1"
  local val="$2"
  if [[ -z "$val" || "$val" == "null" ]]; then
    echo "error: $name could not be read"
    err=1
    return
  fi
  if [[ "$val" != "$CARGO_VERSION" ]]; then
    echo "error: version mismatch: $name=$val (expected $CARGO_VERSION from Cargo workspace)"
    err=1
  fi
}

check "extension/package.json" "$EXT_VERSION"
check "extension/package-lock.json (top-level)" "$LOCK_TOP"
check "extension/package-lock.json (packages.\"\")" "$LOCK_ROOT"
check ".release-please-manifest.json" "$MANIFEST_VERSION"

# All workspace crates must use version.workspace = true (no hardcoded version)
if grep -R --include='Cargo.toml' -E '^version\s*=\s*"[0-9]' crates/ 2>/dev/null; then
  echo "error: one or more crates use hardcoded version instead of version.workspace = true"
  err=1
fi

if [[ $err -eq 1 ]]; then
  echo "Version consistency check failed. See .github/workflows/README.md#version-consistency-release-please"
  exit 1
fi
echo "Version consistency OK: $CARGO_VERSION (crates, extension, manifest)"
