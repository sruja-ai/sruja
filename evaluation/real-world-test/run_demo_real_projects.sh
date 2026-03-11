#!/usr/bin/env bash
# Run quickstart + drift on each repo under test-repos/ and append a one-line summary.
# Use after run_demo.sh / setup_repos.sh to validate the demo on real projects (CLI-only).
#
# Usage: ./run_demo_real_projects.sh [output_file]
# Default output: run_results/demo_real_projects_YYYYMMDD_HHMMSS.txt

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib.sh"
REPOS_DIR="${SCRIPT_DIR}/test-repos"
RESULTS_DIR="${SCRIPT_DIR}/run_results"
OUTPUT_FILE="${1:-${RESULTS_DIR}/demo_real_projects_$(date +%Y%m%d_%H%M%S).txt}"

SRUJA=$(find_sruja)
if [ -z "$SRUJA" ]; then
  echo "❌ sruja CLI not found. From repo root: make build"
  exit 1
fi

mkdir -p "$RESULTS_DIR"

echo "Sruja demo on real projects — $(date -Iseconds)" | tee "$OUTPUT_FILE"
echo "Repos: $REPOS_DIR" | tee -a "$OUTPUT_FILE"
echo "" | tee -a "$OUTPUT_FILE"

for repo_path in "${REPOS_DIR}"/*/; do
  [ -d "$repo_path" ] || continue
  name=$(basename "$repo_path")
  echo "--- $name ---" | tee -a "$OUTPUT_FILE"
  if quickout=$($SRUJA quickstart -r "$repo_path" -f text 2>&1); then
    health=$(echo "$quickout" | grep -oE '(Health: [0-9]+/100|Architecture Health Score \(structural only\): [0-9]+/100)' | grep -oE '[0-9]+/100' | head -1)
    [ -z "$health" ] && health="N/A"
    modules=$(echo "$quickout" | grep -oE '[0-9]+ (modules|components)' | head -1 || echo "")
    driftout=$($SRUJA drift -r "$repo_path" -f text 2>&1) || true
    violations=$(echo "$driftout" | grep -cE 'Error|Warning' 2>/dev/null) || violations=0
    echo "  Health: $health | $modules | drift violations: $violations" | tee -a "$OUTPUT_FILE"
  else
    echo "  quickstart failed" | tee -a "$OUTPUT_FILE"
  fi
  echo "" >> "$OUTPUT_FILE"
done

echo "Results written to: $OUTPUT_FILE"
