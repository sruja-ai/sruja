#!/usr/bin/env bash
# Run Cursor agent on each repo to generate a detailed architecture.sruja.
#
# Prerequisites: Cursor CLI (agent), sruja-architecture-agent skill, sruja in PATH.
#
# Usage:
#   bash run_agent_architecture_all_repos.sh           # Run on REPOS_LIST below
#   bash run_agent_architecture_all_repos.sh --all      # Run on all dirs in test-repos/
#   bash run_agent_architecture_all_repos.sh --list a b  # Run on test-repos/a, test-repos/b
#
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib.sh"
REPOS_DIR="${SCRIPT_DIR}/test-repos"
RESULTS_DIR="${SCRIPT_DIR}/run_results"
PROMPT_FILE="${SCRIPT_DIR}/prompts/discovery_agent_prompt_detailed.txt"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT="${RESULTS_DIR}/AGENT_ARCHITECTURE_PER_REPO_${TIMESTAMP}.md"
LOG_DIR="${RESULTS_DIR}/agent_per_repo_${TIMESTAMP}"
mkdir -p "$RESULTS_DIR" "$LOG_DIR"

# Default list: smaller/faster repos first, then larger (agent can take 1–3 min per repo)
REPOS_LIST=(express idurar-erp-crm saleor ever-gauzy gitea react-admin)

if ! command -v agent >/dev/null 2>&1; then
  echo "❌ Cursor CLI (agent) not in PATH. Install: curl https://cursor.com/install -fsS | bash"
  exit 1
fi

SRUJA=$(find_sruja)
[ -z "$SRUJA" ] && echo "⚠️  sruja not in PATH; lint check will be skipped per repo."

PROMPT="$(cat "$PROMPT_FILE")"

MODE="default"
LIST_REPOS=()
while [ $# -gt 0 ]; do
  case "$1" in
    --all)  MODE="all" ; shift ;;
    --list) MODE="list" ; shift ; LIST_REPOS=("$@") ; break ;;
    -h|--help)
      echo "Usage: $0 [OPTIONS]"
      echo "  (default)  Run agent on: ${REPOS_LIST[*]}"
      echo "  --all      Run on every directory under test-repos/"
      echo "  --list R1 R2 ...  Run on test-repos/R1, test-repos/R2, ..."
      exit 0
      ;;
    *) shift ;;
  esac
done

if [ "$MODE" = "list" ]; then
  REPOS_TO_RUN=()
  for name in "${LIST_REPOS[@]}"; do
    [ -d "${REPOS_DIR}/${name}" ] && REPOS_TO_RUN+=("$name")
  done
elif [ "$MODE" = "all" ]; then
  REPOS_TO_RUN=()
  for d in "$REPOS_DIR"/*/; do
    [ -d "$d" ] && REPOS_TO_RUN+=("$(basename "$d")")
  done
  # Skip MANIFEST and non-repos
  REPOS_TO_RUN=($(printf '%s\n' "${REPOS_TO_RUN[@]}" | grep -v '^MANIFEST$' || true))
else
  REPOS_TO_RUN=("${REPOS_LIST[@]}")
fi

# Filter to existing dirs only
FINAL_REPOS=()
for name in "${REPOS_TO_RUN[@]}"; do
  [ -d "${REPOS_DIR}/${name}" ] && FINAL_REPOS+=("$name")
done

if [ ${#FINAL_REPOS[@]} -eq 0 ]; then
  echo "No repos found. Add repos with: ./setup_repos.sh (or --complex / --production)"
  exit 1
fi

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  Cursor Agent: detailed architecture.sruja per repo               ║"
echo "║  Repos: ${FINAL_REPOS[*]}"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo "Report: $REPORT"
echo "Logs:   $LOG_DIR/"
echo ""

{
  echo "# Agent-generated architecture per repo"
  echo ""
  echo "**Generated:** $(date -Iseconds)"
  echo "**Repos:** ${FINAL_REPOS[*]}"
  echo ""
  echo "---"
  echo ""
} > "$REPORT"

for name in "${FINAL_REPOS[@]}"; do
  repo_path="${REPOS_DIR}/${name}"
  log_file="${LOG_DIR}/${name}.txt"
  echo "▶ $name ..."

  # Ensure the skill is available inside the repo for the agent (in addition to any global install).
  SKILL_SRC="${SCRIPT_DIR}/../../skills/sruja-architecture-agent"
  if [ -d "$SKILL_SRC" ]; then
    mkdir -p "${repo_path}/.agents/skills"
    rm -rf "${repo_path}/.agents/skills/sruja-architecture-agent" 2>/dev/null || true
    cp -R "$SKILL_SRC" "${repo_path}/.agents/skills/" 2>/dev/null || true
  fi

  repo_abs="$(cd "$repo_path" && pwd)"
  if agent -p -f --workspace "$repo_abs" "$PROMPT" > "$log_file" 2>&1; then
    agent_ok="✓"
  else
    agent_ok="✗ (check $log_file)"
  fi

  lint_ok="—"
  if [ -n "$SRUJA" ] && [ -f "${repo_path}/architecture.sruja" ]; then
    if "$SRUJA" lint "${repo_path}/architecture.sruja" >> "$log_file" 2>&1; then
      lint_ok="✓"
    else
      lint_ok="✗"
    fi
  elif [ -f "${repo_path}/architecture.sruja" ]; then
    lint_ok="(sruja not in PATH)"
  fi

  {
    echo "## $name"
    echo ""
    echo "| Step | Result |"
    echo "|------|--------|"
    echo "| Agent run | $agent_ok |"
    echo "| architecture.sruja | $([ -f "${repo_path}/architecture.sruja" ] && echo "created" || echo "not created") |"
    echo "| sruja lint | $lint_ok |"
    echo ""
  } >> "$REPORT"

  if [ -f "${repo_path}/architecture.sruja" ]; then
    line_count=$(wc -l < "${repo_path}/architecture.sruja")
    {
      echo "**Lines:** $line_count"
      echo ""
      echo "<details>"
      echo "<summary>Excerpt (first 60 lines)</summary>"
      echo ""
      echo '```sruja'
      head -60 "${repo_path}/architecture.sruja"
      echo '```'
      echo ""
      echo "</details>"
      echo ""
    } >> "$REPORT"
  fi

  echo "  Agent: $agent_ok | Lint: $lint_ok"
done

echo ""
echo "✅ Report: $REPORT"
echo "   Logs:   $LOG_DIR/"
