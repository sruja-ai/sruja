#!/usr/bin/env bash
# Run Sruja (quickstart + drift + discover) on production-grade test repos and append to a detailed report.
#
# Usage:
#   ./setup_repos.sh --production   # clone production repos first (optional)
#   ./run_production_apps_report.sh [repo1 repo2 ...]
#   ./run_production_apps_report.sh  # run on all repos under test-repos that are in PRODUCTION_LIST
#
# Output: run_results/PRODUCTION_APPS_REPORT_<timestamp>.md (and raw logs per repo)
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib.sh"
REPOS_DIR="${SCRIPT_DIR}/test-repos"
SRUJA=$(find_sruja)
RESULTS_DIR="${SCRIPT_DIR}/run_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT="${RESULTS_DIR}/PRODUCTION_APPS_REPORT_${TIMESTAMP}.md"

# Repos from setup_repos.sh --production (subset we run by default if no args)
PRODUCTION_LIST=(erpnext suitecrm espocrm ever-gauzy idurar-erp-crm saleor shopizer mattermost-server rocketchat sentry openmrs-core)

if [ -z "$SRUJA" ]; then
  echo "❌ sruja CLI not found. Run: make build (from Sruja repo root)"
  exit 1
fi

mkdir -p "$RESULTS_DIR"

# If no args, run on all production repos that exist
if [ $# -eq 0 ]; then
  REPOS_TO_RUN=()
  for name in "${PRODUCTION_LIST[@]}"; do
    [ -d "${REPOS_DIR}/${name}" ] && REPOS_TO_RUN+=("$name")
  done
  if [ ${#REPOS_TO_RUN[@]} -eq 0 ]; then
    echo "No production repos found. Run: ./setup_repos.sh --production"
    exit 1
  fi
else
  REPOS_TO_RUN=("$@")
fi

echo "Sruja Production Apps Report — $(date -I)"
echo "Repos: ${REPOS_TO_RUN[*]}"
echo "Report: $REPORT"
echo ""

{
  echo "# Sruja on Production-Grade Applications — Report"
  echo ""
  echo "**Generated:** $(date -Iseconds)"
  echo "**Repos:** ${REPOS_TO_RUN[*]}"
  echo ""
  echo "---"
  echo ""
} > "$REPORT"

for name in "${REPOS_TO_RUN[@]}"; do
  repo_path="${REPOS_DIR}/${name}"
  if [ ! -d "$repo_path" ]; then
    echo "⚠ Skip $name (not cloned)"
    continue
  fi

  echo "▶ Running Sruja on $name..."
  log_quickstart="${RESULTS_DIR}/production_quickstart_${name}_${TIMESTAMP}.txt"
  log_drift="${RESULTS_DIR}/production_drift_${name}_${TIMESTAMP}.txt"
  log_discover="${RESULTS_DIR}/production_discover_${name}_${TIMESTAMP}.txt"

  {
    echo "## $name"
    echo ""
    echo "**Path:** \`$repo_path\`"
    echo ""
  } >> "$REPORT"

  # Quickstart (with timeout for very large repos)
  if timeout 120 "$SRUJA" quickstart -r "$repo_path" -f text > "$log_quickstart" 2>&1; then
    echo "### Quickstart (excerpt)" >> "$REPORT"
    echo "" >> "$REPORT"
    echo '```' >> "$REPORT"
    head -80 "$log_quickstart" >> "$REPORT"
    echo '```' >> "$REPORT"
    echo "" >> "$REPORT"
  else
    echo "### Quickstart" >> "$REPORT"
    echo "Command timed out or failed. See \`$(basename "$log_quickstart")\`." >> "$REPORT"
    echo "" >> "$REPORT"
  fi

  # Drift
  if timeout 60 "$SRUJA" drift -r "$repo_path" -f text > "$log_drift" 2>&1; then
    echo "### Drift (excerpt)" >> "$REPORT"
    echo "" >> "$REPORT"
    echo '```' >> "$REPORT"
    cat "$log_drift" >> "$REPORT"
    echo '```' >> "$REPORT"
    echo "" >> "$REPORT"
  else
    echo "### Drift" >> "$REPORT"
    echo "See \`$(basename "$log_drift")\`." >> "$REPORT"
    echo "" >> "$REPORT"
  fi

  # Discover context
  if timeout 30 "$SRUJA" discover --context -r "$repo_path" > "$log_discover" 2>&1; then
    echo "### Discover context" >> "$REPORT"
    echo "" >> "$REPORT"
    echo '```' >> "$REPORT"
    cat "$log_discover" >> "$REPORT"
    echo '```' >> "$REPORT"
    echo "" >> "$REPORT"
  fi

  echo "  ✓ $name done"
done

echo ""
echo "✅ Report written to: $REPORT"
echo "   Raw logs: $RESULTS_DIR/production_*_${TIMESTAMP}.txt"
