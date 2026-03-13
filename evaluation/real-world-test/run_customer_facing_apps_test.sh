#!/usr/bin/env bash
# Run Sruja (quickstart + drift + discover) on repos that are close to customer-facing applications.
# Use this to validate Sruja on product-like apps: e-commerce, SaaS, admin UIs, collaboration, scheduling.
#
# Prerequisites:
#   ./setup_repos.sh --apps      # clone gitea, saleor, documenso, cal.com
#   ./setup_repos.sh --complex   # adds react-admin, saleor (if not already)
#   Optional: add idurar-erp-crm, ever-gauzy via setup_repos.sh --production then run with --list
#
# Usage:
#   ./run_customer_facing_apps_test.sh              # run on CUSTOMER_FACING_APPS list (only existing)
#   ./run_customer_facing_apps_test.sh --list a b   # run on test-repos/a, test-repos/b
#   ./run_customer_facing_apps_test.sh --setup      # clone --apps first, then run
#
# Output: run_results/CUSTOMER_FACING_APPS_TEST_<timestamp>.md and per-repo logs
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib.sh"
REPOS_DIR="${SCRIPT_DIR}/test-repos"
RESULTS_DIR="${SCRIPT_DIR}/run_results"
SRUJA=$(find_sruja)
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT="${RESULTS_DIR}/CUSTOMER_FACING_APPS_TEST_${TIMESTAMP}.md"
LOG_DIR="${RESULTS_DIR}/customer_facing_${TIMESTAMP}"
mkdir -p "$RESULTS_DIR" "$LOG_DIR"

# Curated list: apps that are product-like / customer-facing (not frameworks or libs).
# Match names from setup_repos.sh --apps and --complex (gitea, saleor, documenso, cal.com, react-admin).
CUSTOMER_FACING_APPS=(gitea saleor documenso cal.com react-admin)

if [ -z "$SRUJA" ]; then
  echo "❌ sruja CLI not found. Run: make build   (from Sruja repo root)"
  exit 1
fi

DO_SETUP=""
MODE="default"
LIST_REPOS=()
while [ $# -gt 0 ]; do
  case "$1" in
    --setup) DO_SETUP=1 ; shift ;;
    --list)  MODE="list" ; shift ; LIST_REPOS=("$@") ; break ;;
    -h|--help)
      echo "Usage: $0 [OPTIONS]"
      echo "  Run quickstart + drift + discover on customer-facing application repos."
      echo ""
      echo "  (default)     Run on: ${CUSTOMER_FACING_APPS[*]} (only repos that exist)"
      echo "  --setup       Clone repos first: ./setup_repos.sh --apps; then run"
      echo "  --list R1 R2  Run on test-repos/R1, test-repos/R2, ..."
      echo ""
      echo "Prerequisites: ./setup_repos.sh --apps   (gitea, saleor, documenso, cal.com)"
      echo "               ./setup_repos.sh --complex (adds react-admin, saleor)"
      exit 0
      ;;
    *) shift ;;
  esac
done

if [ -n "$DO_SETUP" ]; then
  echo "▶ Cloning customer-facing app repos (--apps)..."
  "${SCRIPT_DIR}/setup_repos.sh" --apps
  echo ""
fi

if [ "$MODE" = "list" ]; then
  REPOS_TO_RUN=()
  for name in "${LIST_REPOS[@]}"; do
    [ -d "${REPOS_DIR}/${name}" ] && REPOS_TO_RUN+=("$name")
  done
else
  REPOS_TO_RUN=()
  for name in "${CUSTOMER_FACING_APPS[@]}"; do
    [ -d "${REPOS_DIR}/${name}" ] && REPOS_TO_RUN+=("$name")
  done
fi

if [ ${#REPOS_TO_RUN[@]} -eq 0 ]; then
  echo "No repos found. Run: ./setup_repos.sh --apps   (or --complex for react-admin)"
  echo "Then run this script again."
  exit 1
fi

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  Sruja test: customer-facing applications                         ║"
echo "║  Repos: ${REPOS_TO_RUN[*]}"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo "Report: $REPORT"
echo "Logs:   $LOG_DIR/"
echo ""

{
  echo "# Sruja on customer-facing applications — test report"
  echo ""
  echo "**Generated:** $(date -Iseconds)"
  echo "**Repos:** ${REPOS_TO_RUN[*]}"
  echo ""
  echo "| Repo | Quickstart | Drift | Discover | Notes |"
  echo "|------|------------|-------|----------|-------|"
} > "$REPORT"

for name in "${REPOS_TO_RUN[@]}"; do
  repo_path="${REPOS_DIR}/${name}"
  log_qs="${LOG_DIR}/${name}_quickstart.txt"
  log_drift="${LOG_DIR}/${name}_drift.txt"
  log_disc="${LOG_DIR}/${name}_discover.txt"
  notes=""

  echo "▶ $name ..."

  # Use timeout if available (GNU coreutils; not on macOS by default)
  run_cmd() {
    local t="$1" ; shift
    if command -v timeout >/dev/null 2>&1; then
      timeout "$t" "$@"
    else
      "$@"
    fi
  }

  qs_ok="✗"
  if run_cmd 180 "$SRUJA" quickstart -r "$repo_path" -f text > "$log_qs" 2>&1; then
    qs_ok="✓"
  else
    notes="quickstart timeout/fail"
  fi

  drift_ok="✗"
  if run_cmd 90 "$SRUJA" drift -r "$repo_path" -f text > "$log_drift" 2>&1; then
    drift_ok="✓"
  else
    [ -n "$notes" ] && notes="$notes; " || true
    notes="${notes}drift timeout/fail"
  fi

  disc_ok="—"
  if run_cmd 45 "$SRUJA" discover --context -r "$repo_path" > "$log_disc" 2>&1; then
    disc_ok="✓"
  else
    disc_ok="✗"
  fi

  # One-line health from quickstart if present
  if [ -s "$log_qs" ] && grep -q "Health Score\|health" "$log_qs" 2>/dev/null; then
    health_line=$(grep -E "Health Score|health score" "$log_qs" | head -1 | sed 's/^[[:space:]]*//' | head -c 40)
    [ -n "$health_line" ] && notes="${notes:+$notes | }$health_line"
  fi

  echo "| $name | $qs_ok | $drift_ok | $disc_ok | ${notes:-—} |" >> "$REPORT"
  echo "  Quickstart: $qs_ok | Drift: $drift_ok | Discover: $disc_ok"
done

{
  echo ""
  echo "---"
  echo ""
  echo "## Logs"
  echo ""
  echo "Per-repo logs in \`$(basename "$LOG_DIR")/\`:"
  echo "- \`<repo>_quickstart.txt\` — \`sruja quickstart -r <repo>\`"
  echo "- \`<repo>_drift.txt\` — \`sruja drift -r <repo>\`"
  echo "- \`<repo>_discover.txt\` — \`sruja discover --context -r <repo>\`"
  echo ""
  echo "## Repo definitions (setup_repos.sh)"
  echo ""
  echo "- **--apps:** gitea, saleor, documenso, cal.com (product-like apps)"
  echo "- **--complex:** adds react-admin, saleor, gitea, etcd, caddy, temporal, minio"
  echo ""
} >> "$REPORT"

echo ""
echo "✅ Report: $REPORT"
echo "   Logs:   $LOG_DIR/"
