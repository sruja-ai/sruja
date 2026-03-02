#!/usr/bin/env bash
# Run architecture diff between two commits using the repo's git history.
# Scans at base ref, then at head ref, then runs sruja drift-diff on the two graphs.
#
# Usage:
#   ./drift_by_commit.sh REPO [BASE_REF] [HEAD_REF]
#
# Examples:
#   ./drift_by_commit.sh gitea              # base=main, head=HEAD
#   ./drift_by_commit.sh etcd main HEAD      # same
#   ./drift_by_commit.sh caddy HEAD~10 HEAD # last 10 commits
#
# Requires: repo already cloned (e.g. ./setup_repos.sh --complex), sruja CLI built.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib.sh"

REPO_NAME="${1:?Usage: $0 REPO [BASE_REF] [HEAD_REF]}"
BASE_REF="${2:-main}"
HEAD_REF="${3:-HEAD}"

REPOS_DIR="${SCRIPT_DIR}/test-repos"
REPO_PATH="${REPOS_DIR}/${REPO_NAME}"

SRUJA="$(find_sruja)"
if [ -z "$SRUJA" ]; then
  echo "Error: sruja CLI not found. Run 'make build' from repo root or set PATH." >&2
  exit 1
fi

if [ ! -d "$REPO_PATH" ]; then
  echo "Error: repo not found at ${REPO_PATH}. Run ./setup_repos.sh or ./setup_repos.sh --complex first." >&2
  exit 1
fi

if [ ! -d "$REPO_PATH/.git" ]; then
  echo "Error: not a git repo: ${REPO_PATH}" >&2
  exit 1
fi

# Temp files for graph JSONs (same filesystem as repo for sruja scan -r .)
BASE_GRAPH="${SCRIPT_DIR}/.sruja_graph_base_$$.json"
HEAD_GRAPH="${SCRIPT_DIR}/.sruja_graph_head_$$.json"
trap 'rm -f "$BASE_GRAPH" "$HEAD_GRAPH"' EXIT

cd "$REPO_PATH"
SAVED_REF="$(git rev-parse -q --abbrev-ref HEAD 2>/dev/null || git rev-parse HEAD)"

if ! git diff-index --quiet HEAD -- 2>/dev/null; then
  echo "Warning: working tree has uncommitted changes. Checking out refs may overwrite them." >&2
  read -r -p "Continue anyway? [y/N] " resp
  case "$resp" in
    [yY]) ;;
    *) exit 1 ;;
  esac
fi

echo "Base ref: $BASE_REF  |  Head ref: $HEAD_REF"
echo "Scanning at base ($BASE_REF)..."
git checkout -q "$BASE_REF"
"$SRUJA" scan . --output "$BASE_GRAPH"

echo "Scanning at head ($HEAD_REF)..."
git checkout -q "$HEAD_REF"
"$SRUJA" scan . --output "$HEAD_GRAPH"

echo "Restoring branch/ref: $SAVED_REF"
git checkout -q "$SAVED_REF"

echo ""
echo "Architecture diff (commits: $BASE_REF → $HEAD_REF)"
echo "────────────────────────────────────────────────────────"
"$SRUJA" drift-diff -b "$BASE_GRAPH" -h "$HEAD_GRAPH"
