#!/usr/bin/env bash
# Demo: Architecture Intelligence at commit A, then drift from A to commit B
#
# Scenario: User provides a repo and two commits. We show:
#   1. Full Architecture Intelligence at the baseline commit (inventory, health, findings).
#   2. Drift from baseline to head: what got worse, what improved, which violations are NEW.
#
# Usage:
#   ./run_commit_drift_demo.sh [REPO] [BASELINE_REF] [HEAD_REF]
#   Defaults: REPO=.  BASELINE_REF=HEAD~1  HEAD_REF=HEAD
#
# Examples:
#   ./run_commit_drift_demo.sh . main HEAD
#   ./run_commit_drift_demo.sh /path/to/repo abc123 def456
#   ./run_commit_drift_demo.sh . origin/main HEAD
#
# Requires: git, sruja CLI (make build). Repo must be a git repo with the refs available.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Find sruja (same as run_demo.sh)
if [ -f "${REPO_ROOT}/evaluation/real-world-test/lib.sh" ]; then
  . "${REPO_ROOT}/evaluation/real-world-test/lib.sh"
else
  find_sruja() {
    [ -f "${REPO_ROOT}/target/release/sruja" ] && echo "${REPO_ROOT}/target/release/sruja" && return
    [ -f "${REPO_ROOT}/target/debug/sruja" ] && echo "${REPO_ROOT}/target/debug/sruja" && return
    command -v sruja >/dev/null 2>&1 && echo "sruja" && return
    echo ""
  }
fi

REPO="${1:-.}"
BASELINE_REF="${2:-HEAD~1}"
HEAD_REF="${3:-HEAD}"

# Resolve to absolute path for repo
if [ "$REPO" = "." ]; then
  REPO_ABS="$(pwd)"
else
  REPO_ABS="$(cd "$REPO" && pwd)"
fi

SRUJA=$(find_sruja)
if [ -z "$SRUJA" ]; then
  echo "❌ sruja CLI not found. From repo root run: make build"
  exit 1
fi

# Must be a git repo
if ! git -C "$REPO_ABS" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "❌ Not a git repository: $REPO_ABS"
  exit 1
fi

# Resolve refs to short SHAs for display
BASELINE_SHA=$(git -C "$REPO_ABS" rev-parse --short "${BASELINE_REF}" 2>/dev/null || echo "$BASELINE_REF")
HEAD_SHA=$(git -C "$REPO_ABS" rev-parse --short "${HEAD_REF}" 2>/dev/null || echo "$HEAD_REF")

echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  Commit-to-Commit Architecture Intelligence Demo                 ║"
echo "║  Baseline: $BASELINE_REF ($BASELINE_SHA) → Head: $HEAD_REF ($HEAD_SHA)"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

# ─── Phase 1: Architecture Intelligence at baseline commit ───
echo "────────────────────────────────────────────────────────────────────"
echo "  [1] Architecture Intelligence at baseline commit ($BASELINE_REF)"
echo "────────────────────────────────────────────────────────────────────"

WORKTREE_DIR=""
cleanup_worktree() {
  if [ -n "$WORKTREE_DIR" ] && [ -d "$WORKTREE_DIR" ]; then
    git -C "$REPO_ABS" worktree remove --force "$WORKTREE_DIR" 2>/dev/null || true
  fi
}
trap cleanup_worktree EXIT

WORKTREE_DIR=$(mktemp -d "${TMPDIR:-/tmp}/sruja-baseline-XXXXXX")
git -C "$REPO_ABS" worktree add --detach "$WORKTREE_DIR" "$BASELINE_REF" >/dev/null 2>&1 || {
  echo "❌ Could not checkout baseline ref: $BASELINE_REF"
  exit 1
}

echo "Running quickstart at baseline (in worktree)..."
echo ""
"$SRUJA" quickstart -r "$WORKTREE_DIR"
echo ""

# Cache baseline graph so drift-pr can reuse it (avoids scanning base again)
CACHE_DIR="$REPO_ABS/.sruja/cache"
mkdir -p "$CACHE_DIR"
CACHE_FILENAME="${BASELINE_REF//\//_}"
CACHE_FILENAME="${CACHE_FILENAME//./_}"
CACHE_PATH="$CACHE_DIR/${CACHE_FILENAME}.json"
echo "Caching baseline graph at $CACHE_PATH for drift comparison..."
"$SRUJA" scan -r "$WORKTREE_DIR" -o "$CACHE_PATH" 2>/dev/null || true

cleanup_worktree
trap - EXIT
WORKTREE_DIR=""

echo ""
echo "────────────────────────────────────────────────────────────────────"
echo "  [2] Drift: what changed from baseline ($BASELINE_REF) to head ($HEAD_REF)"
echo "────────────────────────────────────────────────────────────────────"
echo ""

# Phase 2: PR-scoped drift (base = baseline, head = current working tree)
# The repo is scanned at its current checkout; ensure you are on the desired "head" commit
# (e.g. checkout your branch first, then run this script).
"$SRUJA" drift-pr -r "$REPO_ABS" -b "$BASELINE_REF" -H "$HEAD_REF"

echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  ✅ Commit drift demo complete                                   ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Insights provided:"
echo "  • Baseline: inventory, health score, top findings at $BASELINE_REF"
echo "  • Drift: health delta, NEW violations introduced between the two commits"
echo "  • Use in CI: sruja drift-pr -r . --base origin/main --format github-actions"
echo ""
