#!/usr/bin/env bash
# Build a "diff-and-refine" prompt for the AI: repo context + current architecture + drift.
# The AI uses this to propose only changes to architecture.sruja (no full rewrite).
#
# Usage:
#   ./run_diff_refine_prompt.sh [repo_path] [architecture_file]
#   ./run_diff_refine_prompt.sh . architecture.sruja
#   ./run_diff_refine_prompt.sh test-repos/express test-repos/express/architecture.sruja
#
# Output: run_results/DIFF_REFINE_PROMPT_<timestamp>.txt (or stdout with -)
# Paste the output into your AI chat (Cursor, Claude, etc.) with the sruja-architecture skill.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib.sh"
REPO_PATH="${1:-.}"
ARCH_FILE="${2:-}"
RESULTS_DIR="${SCRIPT_DIR}/run_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT=""

# Resolve repo and architecture file
if [ -z "$ARCH_FILE" ]; then
  ARCH_FILE="${REPO_PATH}/architecture.sruja"
fi
if [ ! -d "$REPO_PATH" ]; then
  REPO_PATH="${SCRIPT_DIR}/${REPO_PATH}"
fi
if [ ! -d "$REPO_PATH" ]; then
  echo "❌ Repo path not found: $REPO_PATH"
  exit 1
fi
# If architecture file path was relative and we resolved REPO_PATH, resolve ARCH_FILE too
if [ ! -f "$ARCH_FILE" ] && [ -f "${SCRIPT_DIR}/${ARCH_FILE}" ]; then
  ARCH_FILE="${SCRIPT_DIR}/${ARCH_FILE}"
fi
if [ ! -f "$ARCH_FILE" ]; then
  # Try next to resolved REPO_PATH
  if [ -f "${REPO_PATH}/architecture.sruja" ]; then
    ARCH_FILE="${REPO_PATH}/architecture.sruja"
  else
    echo "❌ Architecture file not found: $ARCH_FILE"
    echo "   Create one first (e.g. generate with the skill), then run this script for diff-and-refine."
    exit 1
  fi
fi

mkdir -p "$RESULTS_DIR"

# Output to file unless last arg is "-"
for arg in "$@"; do
  [ "$arg" = "-" ] && OUTPUT="-"
done
[ "$OUTPUT" = "-" ] || OUTPUT="${RESULTS_DIR}/DIFF_REFINE_PROMPT_${TIMESTAMP}.txt"

build_prompt() {
  local repo="$1"
  local arch="$2"
  local sruja
  sruja=$(find_sruja)

  echo "--- DIFF-AND-REFINE PROMPT (paste into AI chat with sruja-architecture skill) ---"
  echo ""
  echo "Use the sruja-architecture skill in **diff-and-refine** mode. I have an existing architecture file and want you to propose only changes (additions, removals, relationship fixes) so it stays in sync with the codebase. Do not rewrite from scratch."
  echo ""
  echo "Repo path: $repo"
  echo "Current architecture file: $arch"
  echo ""
  echo "--- REPO CONTEXT (from sruja discover --context) ---"
  if [ -n "$sruja" ]; then
    "$sruja" discover --context -r "$repo" 2>/dev/null || true
  else
    echo "(sruja CLI not found; run from repo root: sruja discover --context -r .)"
  fi
  echo ""
  echo "--- DRIFT: code vs documented architecture (from sruja drift) ---"
  if [ -n "$sruja" ]; then
    "$sruja" drift -r "$repo" -a "$arch" -f text 2>/dev/null || true
  else
    echo "(sruja CLI not found; run: sruja drift -r . -a architecture.sruja)"
  fi
  echo ""
  echo "--- CURRENT ARCHITECTURE ELEMENTS (from sruja list) ---"
  if [ -n "$sruja" ]; then
    "$sruja" list "$arch" 2>/dev/null || true
  else
    echo "(sruja CLI not found)"
  fi
  echo ""
  echo "--- INSTRUCTIONS ---"
  echo "1. Read the repo context and drift output above."
  echo "2. Propose only the minimal changes to $arch to align with the codebase (add missing containers/components, remove obsolete ones, fix relationships)."
  echo "3. Output the full updated architecture.sruja content, or a clear list of edits (add/remove/change) with snippet."
  echo "4. Run sruja lint on the result and fix until it passes."
  echo ""
  echo "--- END PROMPT ---"
}

if [ "$OUTPUT" = "-" ]; then
  build_prompt "$REPO_PATH" "$ARCH_FILE"
else
  build_prompt "$REPO_PATH" "$ARCH_FILE" > "$OUTPUT"
  echo "📄 Diff-and-refine prompt written to: $OUTPUT"
  echo "   Paste its contents into your AI chat (with sruja-architecture skill) to get proposed changes."
fi
