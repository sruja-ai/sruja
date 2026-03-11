#!/usr/bin/env bash
# Run the full local test flow (clone/setup, demo, evaluate), capture results, and write observations.
# Use this to "test and observe" then improve based on results.
#
# Usage: ./run_test_and_observe.sh [--no-clone] [--multi-repo]
#   --no-clone   Skip clone/setup; use existing test-repos (faster for repeat runs)
#   --multi-repo Run quickstart + drift on ALL repos in test-repos/ and add summary to observations
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib.sh"
REPOS_DIR="${SCRIPT_DIR}/test-repos"
RUN_RESULTS="${SCRIPT_DIR}/run_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OBSERVE_FILE="${RUN_RESULTS}/test_and_observe_${TIMESTAMP}.md"

SKIP_CLONE=""
MULTI_REPO=""
for arg in "$@"; do
  case "$arg" in
    --no-clone)  SKIP_CLONE=1 ;;
    --multi-repo) MULTI_REPO=1 ;;
    -h|--help)
      echo "Usage: $0 [--no-clone] [--multi-repo]"
      echo "  Run full test flow: setup (optional), prepare_skill, run_demo, evaluate_architecture, capture observations."
      echo "  --no-clone   Use existing test-repos (skip clone and prepare_skill)"
      echo "  --multi-repo Run quickstart + drift on all test-repos and add multi-repo summary to observations"
      exit 0
      ;;
  esac
done

SRUJA=$(find_sruja)
if [ -z "$SRUJA" ]; then
  echo "❌ sruja CLI not found. Run: make build   (from repo root)"
  exit 1
fi

mkdir -p "$RUN_RESULTS"
echo ""
echo "══════════════════════════════════════════════════════════════"
echo "  Test and observe – $(date -Iseconds)"
echo "══════════════════════════════════════════════════════════════"
echo ""

OBSERVATIONS=""
add_obs() { OBSERVATIONS="${OBSERVATIONS}$1"; }

# ─── 1. Setup (optional) ───
if [ -z "$SKIP_CLONE" ]; then
  echo "📂 Setup: ensuring express and fastapi in test-repos..."
  mkdir -p "$REPOS_DIR"
  if [ ! -d "${REPOS_DIR}/express" ]; then
    git clone --depth 1 https://github.com/expressjs/express.git "${REPOS_DIR}/express" 2>/dev/null && echo "  ✓ cloned express" || echo "  ✗ clone express failed"
  else
    echo "  ✓ express exists"
  fi
  if [ ! -d "${REPOS_DIR}/fastapi" ]; then
    git clone --depth 1 https://github.com/tiangolo/fastapi.git "${REPOS_DIR}/fastapi" 2>/dev/null && echo "  ✓ cloned fastapi" || echo "  ✗ clone fastapi failed"
  else
    echo "  ✓ fastapi exists"
  fi
  echo "📂 Preparing skills in test-repos..."
  "${SCRIPT_DIR}/prepare_skill_in_real_projects.sh" 2>/dev/null || true
  echo ""
fi

# ─── 2. Demo (quickstart + drift + baseline) ───
echo "▶ Run demo (quickstart + drift + baseline)..."
REPO_PATH="${REPOS_DIR}/express"
if [ ! -d "$REPO_PATH" ]; then
  add_obs "- Demo skipped: express not found. Run without --no-clone.\n"
else
  if "$SCRIPT_DIR/run_demo.sh" --baseline > "${RUN_RESULTS}/demo_baseline_${TIMESTAMP}.txt" 2>&1; then
    add_obs "- run_demo.sh --baseline: OK (quickstart + drift + drift vs baseline).\n"
    echo "  ✓ Demo complete"
  else
    add_obs "- run_demo.sh --baseline: FAILED (check run_results/demo_baseline_*.txt).\n"
    echo "  ✗ Demo had non-zero exit"
  fi
fi
echo ""

# ─── 3. Evaluate architecture (express has architecture.sruja after --baseline) ───
echo "▶ Evaluate architecture (express)..."
if [ -f "${REPO_PATH}/architecture.sruja" ]; then
  if "${SCRIPT_DIR}/evaluate_architecture.sh" express > "${RUN_RESULTS}/evaluate_express_${TIMESTAMP}.txt" 2>&1; then
    add_obs "- evaluate_architecture.sh express: OK (stats + lint + report in results/).\n"
    echo "  ✓ Evaluation complete"
  else
    add_obs "- evaluate_architecture.sh express: non-zero exit (see run_results/evaluate_express_*.txt).\n"
    echo "  ✗ Evaluation had issues"
  fi
else
  add_obs "- evaluate_architecture.sh skipped: no architecture.sruja in express (run run_demo.sh --baseline first).\n"
  echo "  ⊘ No architecture.sruja (run demo with --baseline first)"
fi
echo ""

# ─── 4. Quickstart (and optionally multi-repo quickstart + drift) ───
if [ -n "$MULTI_REPO" ] && [ -d "$REPOS_DIR" ]; then
  echo "▶ Multi-repo: quickstart + drift on all test-repos..."
  MULTI_LOG="${RUN_RESULTS}/multi_repo_${TIMESTAMP}.txt"
  MULTI_TABLE="${RUN_RESULTS}/multi_repo_${TIMESTAMP}.md"
  echo "| Repo | Quickstart | Drift | Skill present |" > "$MULTI_TABLE"
  echo "|------|------------|-------|---------------|" >> "$MULTI_TABLE"
  repocount=0
  for repo_path in "${REPOS_DIR}"/*/; do
    [ -d "$repo_path" ] || continue
    name=$(basename "$repo_path")
    repocount=$((repocount + 1))
    echo "  [$repocount] $name..."
    qs_ok="✗"
    drift_ok="✗"
    if $SRUJA quickstart -r "$repo_path" >> "$MULTI_LOG" 2>&1; then qs_ok="✓"; fi
    if $SRUJA drift -r "$repo_path" >> "$MULTI_LOG" 2>&1; then drift_ok="✓"; fi
    skill_ok="✗"
    [ -f "${repo_path}.agents/skills/sruja-architecture/SKILL.md" ] && skill_ok="✓"
    echo "| $name | $qs_ok | $drift_ok | $skill_ok |" >> "$MULTI_TABLE"
  done
  add_obs "- Multi-repo: quickstart + drift on $repocount repo(s). Summary: run_results/multi_repo_${TIMESTAMP}.md\n"
  echo "  ✓ Multi-repo summary in $MULTI_TABLE"
else
  echo "▶ Quickstart fastapi (smoke check)..."
  FASTAPI_PATH="${REPOS_DIR}/fastapi"
  if [ -d "$FASTAPI_PATH" ]; then
    if $SRUJA quickstart -r "$FASTAPI_PATH" 2>&1 | tee "${RUN_RESULTS}/quickstart_fastapi_${TIMESTAMP}.txt" | tail -5 | grep -q "Next Steps\|Health Score"; then
      add_obs "- quickstart fastapi: OK.\n"
      echo "  ✓ Quickstart fastapi OK"
    else
      add_obs "- quickstart fastapi: run completed (check output).\n"
      echo "  ✓ Quickstart completed"
    fi
  else
    add_obs "- quickstart fastapi: skipped (repo not present).\n"
    echo "  ⊘ fastapi not found"
  fi
fi
echo ""

# ─── 5. Write observations ───
{
  echo "# Test and observe – $TIMESTAMP"
  echo ""
  echo "**Date:** $(date -Iseconds)"
  echo ""
  echo "## Steps run"
  echo "1. Setup (clone + prepare_skill or --no-clone)"
  echo "2. run_demo.sh --baseline (express)"
  echo "3. evaluate_architecture.sh express"
  if [ -n "$MULTI_REPO" ]; then
    echo "4. Multi-repo quickstart + drift (all test-repos)"
  else
    echo "4. quickstart fastapi"
  fi
  echo ""
  echo "## Observations"
  echo ""
  printf "%b" "$OBSERVATIONS"
  echo ""
  if [ -n "$MULTI_REPO" ] && [ -f "${RUN_RESULTS}/multi_repo_${TIMESTAMP}.md" ]; then
    echo "## Multi-repo summary"
    echo ""
    cat "${RUN_RESULTS}/multi_repo_${TIMESTAMP}.md"
    echo ""
  fi
  echo "## Artifacts"
  echo "- \`run_results/demo_baseline_${TIMESTAMP}.txt\`"
  echo "- \`run_results/evaluate_express_${TIMESTAMP}.txt\`"
  if [ -n "$MULTI_REPO" ]; then
    echo "- \`run_results/multi_repo_${TIMESTAMP}.md\` and \`.txt\`"
  else
    echo "- \`run_results/quickstart_fastapi_${TIMESTAMP}.txt\`"
  fi
  echo "- \`results/evaluation_express_*.md\` (if evaluation ran)"
  echo ""
  echo "## Skills vs CLI"
  echo "- **CLI** (quickstart, drift, lint) does **not** use Sruja skills; it is deterministic."
  echo "- **Skills** are for the **Cursor agent**: they guide the AI when generating or editing \`.sruja\` files."
  echo "- To see if skills help: run \`agent -p \"...\"\` in a test-repo (with skill present), then run \`sruja lint architecture.sruja\` and review quality."
  echo ""
  echo "## Next"
  echo "- Review observations and fix any failures."
  echo "- Run \`./evaluate_architecture.sh express\` and fill the manual checklist if needed."
  echo "- For Cursor CLI agent test: see LOCAL_CURSOR_CLI_TESTING.md."
  echo "- Run with \`--multi-repo\` to run quickstart + drift on all test-repos."
} > "$OBSERVE_FILE"

echo "══════════════════════════════════════════════════════════════"
echo "  Observations written to: $OBSERVE_FILE"
echo "══════════════════════════════════════════════════════════════"
echo ""
