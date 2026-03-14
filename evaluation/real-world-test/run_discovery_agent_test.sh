#!/usr/bin/env bash
# Test the discovery flow with Cursor CLI (agent): agent should ask questions then generate architecture.
#
# Prerequisites:
#   - Cursor CLI installed: agent in PATH (curl https://cursor.com/install -fsS | bash)
#   - Sruja skill: npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
#   - Optional: sruja CLI for lint (make build from sruja repo, or in PATH)
#
# Usage (run with bash):
#   bash run_discovery_agent_test.sh              # Run agent in test-repos/express (interactive)
#   bash run_discovery_agent_test.sh --force      # Non-interactive: auto-approve commands (-f)
#   bash run_discovery_agent_test.sh --repo path  # Run agent in path
#   bash run_discovery_agent_test.sh --dry-run   # Print prompt and exit
#
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib.sh"
REPOS_DIR="${SCRIPT_DIR}/test-repos"
DEMO_REPO="express"
REPO_PATH="${REPOS_DIR}/${DEMO_REPO}"
DRY_RUN=""
AGENT_FORCE=""   # -f / --force: pass to agent for non-interactive (auto-approve commands)
PROMPT_FILE="${SCRIPT_DIR}/prompts/discovery_agent_prompt.txt"

for arg in "$@"; do
  case "$arg" in
    -h|--help)
      echo "Usage: $0 [OPTIONS]"
      echo ""
      echo "  (none)        Run Cursor agent in test-repos/express with discovery prompt"
      echo "  --force, -f   Non-interactive: agent -p -f (auto-approve commands)"
      echo "  --repo PATH   Use PATH as repo (default: test-repos/express)"
      echo "  --dry-run     Print prompt and repo path, do not run agent"
      echo "  -h, --help    Show this help"
      echo ""
      echo "Prerequisites: agent (Cursor CLI), sruja-architecture skill, optional: sruja CLI"
      exit 0
      ;;
    --repo)
      shift 2 || true
      REPO_PATH="$1"
      ;;
    --force|-f) AGENT_FORCE="-f" ;;
    --dry-run) DRY_RUN="1" ;;
  esac
done

# Resolve --repo if next arg
for i in 1 2; do
  [ "$1" = "--repo" ] && REPO_PATH="$2" && shift 2 || true
  shift || true
done 2>/dev/null || true

if [ ! -f "$PROMPT_FILE" ]; then
  echo "❌ Prompt file not found: $PROMPT_FILE"
  exit 1
fi

PROMPT="$(cat "$PROMPT_FILE")"

# Ensure repo exists
if [ ! -d "$REPO_PATH" ]; then
  echo "📂 Cloning ${DEMO_REPO} (one-time setup)..."
  mkdir -p "$REPOS_DIR"
  "${SCRIPT_DIR}/setup_repos.sh" 2>/dev/null || {
    git clone --depth 1 "https://github.com/expressjs/express.git" "$REPO_PATH" 2>/dev/null || true
  }
  [ -d "$REPO_PATH" ] || { echo "❌ Repo not found: $REPO_PATH"; exit 1; }
  echo "   ✓ Done"
fi

if [ -n "$DRY_RUN" ]; then
  echo "Repo: $REPO_PATH"
  echo "Prompt file: $PROMPT_FILE"
  echo "--- Prompt ---"
  echo "$PROMPT"
  echo "--- End prompt ---"
  echo ""
  echo "Run manually: cd $REPO_PATH && agent -p \"\$PROMPT\""
  exit 0
fi

# Check for Cursor CLI
if ! command -v agent >/dev/null 2>&1; then
  echo "❌ Cursor CLI (agent) not found in PATH."
  echo "   Install: curl https://cursor.com/install -fsS | bash"
  exit 1
fi

SRUJA=$(find_sruja)
if [ -z "$SRUJA" ]; then
  echo "⚠️  sruja CLI not found; agent will run but you cannot lint from this script."
  echo "   Install: make build (from sruja repo) or curl -fsSL https://sruja.ai/install.sh | bash"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  Discovery agent test (Cursor CLI)                               ║"
echo "║  Repo: $REPO_PATH"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "The agent will: (1) list discovery questions, (2) generate architecture.sruja, (3) run sruja lint."

echo "After it finishes: open architecture.sruja, read the \"Discovery questions I would ask\" section, answer them, and then run a refinement pass using the refinement guidance in skills/sruja-architecture/REFERENCE.md (Gather → Ask → Build)."
[ -n "$AGENT_FORCE" ] && echo "Running non-interactive (--force)." || echo "Approve any commands the agent proposes."
echo ""

# Ensure the skill is available inside the repo for the agent (in addition to any global install).
# This avoids ambiguous agent behavior when global skills are not configured.
SKILL_SRC="${SCRIPT_DIR}/../../skills/sruja-architecture"
if [ -d "$SKILL_SRC" ]; then
  mkdir -p "${REPO_PATH}/.agents/skills"
  rm -rf "${REPO_PATH}/.agents/skills/sruja-architecture" 2>/dev/null || true
  cp -R "$SKILL_SRC" "${REPO_PATH}/.agents/skills/" 2>/dev/null || true
fi

# -p = print to console (script-friendly); optional -f = auto-approve commands; --workspace = run in repo (use absolute path)
REPO_ABS="$(cd "$REPO_PATH" && pwd)"
cd "$REPO_PATH"
agent -p $AGENT_FORCE --workspace "$REPO_ABS" "$PROMPT"

# Validate if sruja available and file was created
if [ -n "$SRUJA" ] && [ -f "architecture.sruja" ]; then
  echo ""
  echo "--- Validating generated file ---"
  "$SRUJA" lint architecture.sruja && echo "✓ sruja lint passed" || echo "⚠️  sruja lint had errors"
fi
