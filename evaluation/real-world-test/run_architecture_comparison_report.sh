#!/usr/bin/env bash
# Run comparison (golden vs generated) for all test-repos that have architecture.sruja,
# and optionally include evaluate_architecture stats. Writes a summary markdown report.
#
# Usage:
#   ./run_architecture_comparison_report.sh
#   ./run_architecture_comparison_report.sh express saleor   # only these repos
#
# If run_results/ contains generated_<repo>.sruja (or generated_<repo>_*.sruja), compare to golden.
# Otherwise report baseline stats (golden only) and lint.
#
# Output: run_results/ARCHITECTURE_COMPARISON_REPORT_<timestamp>.md
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib.sh"
REPOS_DIR="${SCRIPT_DIR}/test-repos"
RESULTS_DIR="${SCRIPT_DIR}/run_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT="${RESULTS_DIR}/ARCHITECTURE_COMPARISON_REPORT_${TIMESTAMP}.md"

# Repos to process: from args, or all with architecture.sruja
if [ $# -ge 1 ]; then
  REPOS=("$@")
else
  REPOS=()
  for d in "${REPOS_DIR}"/*/; do
    [ -d "$d" ] || continue
    [ -f "${d}architecture.sruja" ] || continue
    REPOS+=("$(basename "$d")")
  done
fi

mkdir -p "$RESULTS_DIR"
SRUJA=$(find_sruja)

norm() { echo "$1" | tr -d ' \n'; }
count() {
  local file="$1"
  local pattern="$2"
  grep -c -e "$pattern" "$file" 2>/dev/null || echo "0"
}

stats_line() {
  local f="$1"
  local sys=$(norm "$(count "$f" '= system')")
  local con=$(norm "$(count "$f" '= container')")
  local comp=$(norm "$(count "$f" '= component')")
  local db=$(($(norm "$(count "$f" '= database')") + $(norm "$(count "$f" '= datastore')")))
  local rel=$(norm "$(count "$f" '->')")
  echo "${sys}|${con}|${comp}|${db}|${rel}"
}

lint_status() {
  local f="$1"
  [ -z "$SRUJA" ] && echo "skip" && return
  $SRUJA lint "$f" >/dev/null 2>&1 && echo "pass" || echo "fail"
}

{
  echo "# Architecture comparison report"
  echo ""
  echo "**Generated:** $(date -Iseconds)"
  echo "**Repos:** ${REPOS[*]:-(none with architecture.sruja)}"
  echo ""
  echo "---"
  echo ""

  for name in "${REPOS[@]}"; do
    golden="${REPOS_DIR}/${name}/architecture.sruja"
    [ -f "$golden" ] || continue

    echo "## $name"
    echo ""
    g_stats=$(stats_line "$golden")
    IFS='|' read -r g_sys g_con g_comp g_db g_rel <<< "$g_stats"
    g_lint=$(lint_status "$golden")
    echo "| Metric | Golden |"
    echo "|--------|--------|"
    echo "| Systems | $g_sys |"
    echo "| Containers | $g_con |"
    echo "| Components | $g_comp |"
    echo "| Datastores | $g_db |"
    echo "| Relationships | $g_rel |"
    echo "| Lint | $g_lint |"
    echo ""

    # Look for generated file to compare
    gen=""
    for candidate in "${RESULTS_DIR}/generated_${name}.sruja" "${RESULTS_DIR}"/generated_${name}_*.sruja; do
      [ -f "$candidate" ] && gen="$candidate" && break
    done
    if [ -n "$gen" ]; then
      n_stats=$(stats_line "$gen")
      IFS='|' read -r n_sys n_con n_comp n_db n_rel <<< "$n_stats"
      n_lint=$(lint_status "$gen")
      echo "**Comparison vs generated:** \`$(basename "$gen")\`"
      echo ""
      echo "| Metric | Golden | Generated | Delta |"
      echo "|--------|--------|-----------+-------|"
      echo "| Systems | $g_sys | $n_sys | $((n_sys - g_sys)) |"
      echo "| Containers | $g_con | $n_con | $((n_con - g_con)) |"
      echo "| Components | $g_comp | $n_comp | $((n_comp - g_comp)) |"
      echo "| Datastores | $g_db | $n_db | $((n_db - g_db)) |"
      echo "| Relationships | $g_rel | $n_rel | $((n_rel - g_rel)) |"
      echo "| Lint | $g_lint | $n_lint | — |"
      echo ""
    else
      echo "*No generated file in run_results/ for this repo. To compare: save agent output as \`run_results/generated_${name}.sruja\` and re-run this script.*"
      echo ""
    fi
  done

  echo "---"
  echo ""
  echo "To compare two files manually: \`./compare_architecture.sh <golden.sruja> <generated.sruja>\`"
  echo "See [EVALUATION_METHODOLOGY.md](EVALUATION_METHODOLOGY.md)."
} > "$REPORT"

echo "📄 Report written to: $REPORT"
echo ""
