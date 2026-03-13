#!/usr/bin/env bash
# Capture as rich architecture information as possible from one or more repos.
# Writes a single bundle per repo under run_results/rich_capture_<repo>_<timestamp>/.
#
# See docs/CAPTURING_RICH_ARCHITECTURE_FROM_REPOS.md for the pipeline and use cases.
#
# Usage:
#   ./run_rich_architecture_capture.sh <repo_name>              # one repo (e.g. saleor)
#   ./run_rich_architecture_capture.sh --list saleor documenso  # multiple
#   ./run_rich_architecture_capture.sh --list gitea --prompt    # include generate --prompt-only
#
# Prerequisites: sruja CLI (make build); test-repos/<name> must exist (e.g. ./setup_repos.sh --apps).
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib.sh"
REPOS_DIR="${SCRIPT_DIR}/test-repos"
RESULTS_DIR="${SCRIPT_DIR}/run_results"
SRUJA=$(find_sruja)
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
INCLUDE_PROMPT=""

if [ -z "$SRUJA" ]; then
  echo "❌ sruja CLI not found. Run: make build   (from Sruja repo root)"
  exit 1
fi

REPOS_TO_RUN=()
while [ $# -gt 0 ]; do
  case "$1" in
    --list)
      shift
      while [ $# -gt 0 ] && [ "${1:0:1}" != "-" ]; do
        REPOS_TO_RUN+=("$1")
        shift
      done
      ;;
    --prompt) INCLUDE_PROMPT=1 ; shift ;;
    -h|--help)
      echo "Usage: $0 [OPTIONS] [REPO_NAME]"
      echo ""
      echo "  Capture rich architecture info: scan, discover, context export, intent check, optional DSL."
      echo ""
      echo "  REPO_NAME           Single repo under test-repos/ (e.g. saleor)"
      echo "  --list R1 R2 ...    Multiple repos"
      echo "  --prompt            Also run sruja generate --prompt-only -o prompt.txt"
      echo ""
      echo "Output: run_results/rich_capture_<repo>_<timestamp>/"
      echo "  - graph.json, discover_context.txt, context_export.json"
      echo "  - intent_report.json (if intent dirs exist), architecture.sruja (if present)"
      echo "  - prompt.txt (if --prompt)"
      echo ""
      echo "See: docs/CAPTURING_RICH_ARCHITECTURE_FROM_REPOS.md"
      exit 0
      ;;
    *)
      if [ -z "${REPOS_TO_RUN[*]}" ]; then
        REPOS_TO_RUN=("$1")
      fi
      shift
      ;;
  esac
done

if [ ${#REPOS_TO_RUN[@]} -eq 0 ]; then
  echo "Usage: $0 <repo_name>   or   $0 --list repo1 repo2"
  echo "Repos must exist under test-repos/. Run ./setup_repos.sh --apps (or --complex) first."
  exit 1
fi

mkdir -p "$RESULTS_DIR"

for name in "${REPOS_TO_RUN[@]}"; do
  repo_path="${REPOS_DIR}/${name}"
  if [ ! -d "$repo_path" ]; then
    echo "⚠ Skip $name (not found: $repo_path)"
    continue
  fi

  out_dir="${RESULTS_DIR}/rich_capture_${name}_${TIMESTAMP}"
  mkdir -p "$out_dir"
  echo "▶ Capturing rich architecture: $name → $out_dir"

  # 1. Scan → graph
  if "$SRUJA" scan "$repo_path" --output "${out_dir}/graph.json" 2>/dev/null; then
    echo "  ✓ graph.json"
  else
    echo "  ✗ scan failed"
  fi

  # 2. Discover context
  if "$SRUJA" discover --context -r "$repo_path" > "${out_dir}/discover_context.txt" 2>/dev/null; then
    echo "  ✓ discover_context.txt"
  else
    echo "  ✗ discover --context failed"
  fi

  # 3. Context export (JSON)
  if "$SRUJA" context -r "$repo_path" -f json -o "${out_dir}/context_export.json" 2>/dev/null; then
    echo "  ✓ context_export.json"
  else
    echo "  ✗ context export failed"
  fi

  # 4. Intent check (if repo has intent dirs; may fail if none)
  if "$SRUJA" intent check -r "$repo_path" -i "$repo_path" -f json > "${out_dir}/intent_report.json" 2>/dev/null; then
    echo "  ✓ intent_report.json"
  else
    echo "  ⊘ intent check skipped or failed (no intent dirs or error)"
  fi

  # 5. Copy architecture.sruja if present
  if [ -f "${repo_path}/architecture.sruja" ]; then
    cp "${repo_path}/architecture.sruja" "${out_dir}/architecture.sruja"
    echo "  ✓ architecture.sruja (copied)"
  else
    echo "  ⊘ no architecture.sruja in repo"
  fi

  # Optional: prompt for any LLM
  if [ -n "$INCLUDE_PROMPT" ]; then
    if "$SRUJA" generate -r "$repo_path" --prompt-only -o "${out_dir}/prompt.txt" 2>/dev/null; then
      echo "  ✓ prompt.txt"
    else
      echo "  ✗ generate --prompt-only failed"
    fi
  fi

  echo "  → $out_dir"
  echo ""
done

echo "✅ Rich capture done. Bundles in run_results/rich_capture_*_${TIMESTAMP}/"
echo "   See docs/CAPTURING_RICH_ARCHITECTURE_FROM_REPOS.md for how to use them."
