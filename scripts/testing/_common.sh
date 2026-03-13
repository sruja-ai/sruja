#!/usr/bin/env bash

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

error() { echo -e "${RED}${BOLD}✗ ERROR:${NC} $1" >&2; }
warn() { echo -e "${YELLOW}${BOLD}⚠ WARNING:${NC} $1" >&2; }
success() { echo -e "${GREEN}${BOLD}✓${NC} $1"; }
info() { echo -e "${BLUE}→${NC} $1"; }
header() { echo -e "${CYAN}${BOLD}$1${NC}"; }

project_root() {
  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  cd "$script_dir/../.." >/dev/null
  pwd
}

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" &>/dev/null; then
    error "Missing required tool: $cmd"
    exit 1
  fi
}

timeout_cmd() {
  if command -v timeout &>/dev/null; then
    echo "timeout"
  elif command -v gtimeout &>/dev/null; then
    echo "gtimeout"
  else
    echo ""
  fi
}

run_with_timeout() {
  local duration="$1"
  shift
  local tcmd
  tcmd="$(timeout_cmd)"
  if [ -n "$tcmd" ]; then
    "$tcmd" "$duration" "$@"
  else
    "$@"
  fi
}

ensure_sruja_cli() {
  local root="$1"
  if [ -x "$root/target/release/sruja" ]; then
    echo "$root/target/release/sruja"
    return 0
  fi
  info "Building sruja CLI (release)..."
  (cd "$root" && cargo build --release -p sruja-cli >/dev/null)
  echo "$root/target/release/sruja"
}

artifacts_root() {
  local root="$1"
  echo "$root/evaluation/local-artifacts/testing"
}

