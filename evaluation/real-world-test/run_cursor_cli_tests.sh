#!/usr/bin/env bash
# Run the full Cursor CLI test pipeline: discovery agent, agent-per-repo, lint, export.
#
# Prerequisites: Cursor CLI (agent) in PATH, sruja CLI (make build or in PATH).
#
# Usage:
#   bash run_cursor_cli_tests.sh                # Discovery + agent on default repos + lint + export all
#   bash run_cursor_cli_tests.sh --no-agent    # Skip agent-per-repo (discovery + lint + export only)
#   bash run_cursor_cli_tests.sh --quick      # Lint + export only (no discovery, no agent); fast CI
#   bash run_cursor_cli_tests.sh --full       # Discovery + agent on full default repo list
#
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib.sh"
REPOS_DIR="${SCRIPT_DIR}/test-repos"
RESULTS_DIR="${SCRIPT_DIR}/run_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT="${RESULTS_DIR}/CURSOR_CLI_TESTS_${TIMESTAMP}.md"
mkdir -p "$RESULTS_DIR"

RUN_AGENT_REPOS="1"    # default: DO run agent on repos
RUN_DISCOVERY="1"      # default: DO run discovery (step 1)
AGENT_LIST=""          # default: use full REPOS_LIST from run_agent_architecture_all_repos.sh
while [ $# -gt 0 ]; do
  case "$1" in
    --no-agent) RUN_AGENT_REPOS=""; shift ;;
    --quick)    RUN_DISCOVERY=""; RUN_AGENT_REPOS=""; shift ;;
    --full)     RUN_AGENT_REPOS="1"; AGENT_LIST=""; shift ;;
    -h|--help)
      echo "Usage: $0 [OPTIONS]"
      echo "  (default)     Discovery + agent on default repos + lint + export all"
      echo "  --no-agent    Skip agent-per-repo (discovery + lint + export only)"
      echo "  --quick       Lint + export only (no discovery, no agent); fast"
      echo "  --full        Discovery + agent on full default list"
      echo "  -h, --help    Show this help"
      exit 0
      ;;
    *) shift ;;
  esac
done

# --- Prereqs ---
if [ -n "$RUN_DISCOVERY" ] || [ -n "$RUN_AGENT_REPOS" ]; then
  if ! command -v agent >/dev/null 2>&1; then
    echo "❌ Cursor CLI (agent) not in PATH. Install: curl https://cursor.com/install -fsS | bash"
    exit 1
  fi
fi
SRUJA=$(find_sruja)
if [ -z "$SRUJA" ]; then
  echo "❌ sruja CLI not found. From repo root: make build"
  exit 1
fi

START_TIME=$(date +%s)
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  Cursor CLI – full test pipeline                                ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo "Report: $REPORT"
echo ""

{
  echo "# Cursor CLI test run"
  echo ""
  echo "**Started:** $(date -Iseconds)"
  echo "**agent:** $(agent --version 2>/dev/null || echo '?')"
  echo "**sruja:** $($SRUJA --version 2>/dev/null || echo '?')"
  echo ""
} > "$REPORT"

# --- 1. Discovery agent test (express, non-interactive) ---
if [ -n "$RUN_DISCOVERY" ]; then
  echo "▶ 1. Discovery agent test (express, --force)"
  DISCOVERY_LOG="${RESULTS_DIR}/discovery_express_${TIMESTAMP}.txt"
  if bash "${SCRIPT_DIR}/run_discovery_agent_test.sh" --force > "$DISCOVERY_LOG" 2>&1; then
    discovery_ok="✓"
  else
    discovery_ok="✗ (see $DISCOVERY_LOG)"
  fi
  echo "   $discovery_ok"
  { echo "## 1. Discovery agent test (express)"; echo ""; echo "| Result | $discovery_ok |"; echo ""; } >> "$REPORT"
else
  echo "▶ 1. Discovery agent test (skipped; --quick)"
  discovery_ok="— (skipped)"
  { echo "## 1. Discovery agent test (express)"; echo ""; echo "| Result | $discovery_ok |"; echo ""; } >> "$REPORT"
fi

# --- 2. Agent on repos (optional) ---
if [ -n "$RUN_AGENT_REPOS" ]; then
  echo "▶ 2. Agent on repos (detailed architecture)"
  if [ -n "$AGENT_LIST" ]; then
    bash "${SCRIPT_DIR}/run_agent_architecture_all_repos.sh" --list $AGENT_LIST || true
  else
    bash "${SCRIPT_DIR}/run_agent_architecture_all_repos.sh" || true
  fi
  echo "   Done (see run_results/ for per-repo report)"
  { echo "## 2. Agent on repos"; echo ""; echo "Run completed. See \`run_results/AGENT_ARCHITECTURE_PER_REPO_*.md\`."; echo ""; } >> "$REPORT"
else
  echo "▶ 2. Agent on repos (skipped; run without \`--no-agent\` or \`--quick\` to include)"
  { echo "## 2. Agent on repos"; echo ""; echo "Skipped (run without \`--no-agent\` or \`--quick\` to include agent on repos)."; echo ""; } >> "$REPORT"
fi

# --- 3. Lint all architecture.sruja under test-repos ---
echo "▶ 3. Lint all architecture.sruja in test-repos/"
{
  echo "## 3. Lint all architecture.sruja"
  echo ""
  echo "| Repo | File | Lines | Lint |"
  echo "|------|------|-------|------|"
} >> "$REPORT"
lint_fail=0
lint_passed=0
for dir in "$REPOS_DIR"/*/; do
  [ -d "$dir" ] || continue
  name=$(basename "$dir")
  [ "$name" = "MANIFEST" ] && continue
  f="${dir}architecture.sruja"
  if [ -f "$f" ]; then
    lines=$(wc -l < "$f" | tr -d ' ')
    if $SRUJA lint "$f" >/dev/null 2>&1; then
      lint_ok="✓"
      lint_passed=$((lint_passed + 1))
    else
      lint_ok="✗"
      lint_fail=$((lint_fail + 1))
    fi
    echo "| $name | architecture.sruja | $lines | $lint_ok |" >> "$REPORT"
    echo "   $name: $lines lines, $lint_ok"
  fi
done
echo "" >> "$REPORT"

# --- 4. Export markdown (all repos with architecture.sruja) ---
echo "▶ 4. Export markdown (all repos)"
EXPORT_DIR="${RESULTS_DIR}/export_${TIMESTAMP}"
mkdir -p "$EXPORT_DIR"
export_ok="✓"
export_fail=0
{
  echo "## 4. Export markdown"
  echo ""
  echo "| Repo | Output | Result |"
  echo "|------|--------|--------|"
} >> "$REPORT"
for dir in "$REPOS_DIR"/*/; do
  [ -d "$dir" ] || continue
  name=$(basename "$dir")
  [ "$name" = "MANIFEST" ] && continue
  f="${dir}architecture.sruja"
  if [ -f "$f" ]; then
    out="${EXPORT_DIR}/${name}_architecture.md"
    if $SRUJA export markdown "$f" > "$out" 2>/dev/null; then
      echo "| $name | $out | ✓ |" >> "$REPORT"
      echo "   $name → ${name}_architecture.md ✓"
    else
      echo "| $name | — | ✗ |" >> "$REPORT"
      echo "   $name ✗"
      export_fail=$((export_fail + 1))
      export_ok="✗ ($export_fail failed)"
    fi
  fi
done
echo "" >> "$REPORT"

# --- Summary + duration ---
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
{
  echo "## Summary"
  echo ""
  echo "| Step | Result |"
  echo "|------|--------|"
  echo "| 1. Discovery (express) | $discovery_ok |"
  echo "| 2. Agent on repos | $([ -n "$RUN_AGENT_REPOS" ] && echo "run" || echo "skipped") |"
  echo "| 3. Lint | $lint_passed passed, $lint_fail failed |"
  echo "| 4. Export markdown | $export_ok |"
  echo "| **Artifacts** | Report: \`$REPORT\`; exports: \`$EXPORT_DIR/\` |"
  echo "| **Duration** | ${DURATION}s |"
  echo ""
  echo "**Finished:** $(date -Iseconds)"
} >> "$REPORT"

echo ""
echo "✅ Done. Report: $REPORT (${DURATION}s)"
[ $lint_fail -gt 0 ] && echo "⚠️  $lint_fail architecture.sruja file(s) failed lint." && exit 1
[ "$discovery_ok" != "✓" ] && [ "$discovery_ok" != "— (skipped)" ] && echo "⚠️  Discovery test failed. See $DISCOVERY_LOG" && exit 1
exit 0
