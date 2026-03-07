#!/usr/bin/env bash
# Evaluate Sruja-generated architecture files
# Uses sruja CLI for validation - Rust-first, no Python dependency
#
# Usage:
#   ./evaluate_architecture.sh <repo-name>
#   ./evaluate_architecture.sh express
#   ./evaluate_architecture.sh /path/to/repo
#   ./evaluate_architecture.sh express --llm   # Add LLM evaluation (requires any LLM API key)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib.sh"
# Load .env from this directory when present (for LLM keys; see .env.example)
[ -f "${SCRIPT_DIR}/.env" ] && set -a && . "${SCRIPT_DIR}/.env" && set +a
REPO_ARG=""
USE_LLM=""

for arg in "$@"; do
  case "$arg" in
    -h|--help)
      echo "Usage: $0 <repo-name|path> [--llm]"
      echo ""
      echo "  repo-name   Name under test-repos/ (e.g. express)"
      echo "  path       Path to repo directory containing architecture.sruja"
      echo "  --llm      Also run sruja eval (requires LLM API key in .env)"
      echo "  -h, --help Show this help"
      echo ""
      echo "Examples:"
      echo "  $0 express"
      echo "  $0 /path/to/repo"
      echo "  $0 express --llm"
      exit 0
      ;;
    --llm) USE_LLM="1" ;;
    *) [ -z "$REPO_ARG" ] && REPO_ARG="$arg" ;;
  esac
done

if [ -z "$REPO_ARG" ]; then
  echo "Usage: $0 <repo-name|path>"
  echo ""
  echo "Examples:"
  echo "  $0 express"
  echo "  $0 /path/to/repo"
  echo ""
  echo "Available repos in test-repos/:"
  if [ -d "${SCRIPT_DIR}/test-repos" ]; then
    for repo in "${SCRIPT_DIR}"/test-repos/*/; do
      [ -d "$repo" ] || continue
      name=$(basename "$repo")
      arch_file="${repo}architecture.sruja"
      status="⬜"
      [ -f "$arch_file" ] && status="✅"
      echo "  $status $name"
    done
  else
    echo "  (run ./setup_repos.sh first)"
  fi
  exit 1
fi

# Resolve repo path
if [ -d "$REPO_ARG" ]; then
  REPO_PATH="$REPO_ARG"
  REPO_NAME=$(basename "$REPO_PATH")
else
  REPO_PATH="${SCRIPT_DIR}/test-repos/${REPO_ARG}"
  REPO_NAME="$REPO_ARG"
fi

ARCH_FILE="${REPO_PATH}/architecture.sruja"

if [ ! -d "$REPO_PATH" ]; then
  echo "❌ Repository not found: $REPO_PATH"
  exit 1
fi

if [ ! -f "$ARCH_FILE" ]; then
  echo "❌ No architecture.sruja found in $REPO_PATH"
  echo ""
  echo "To generate one:"
  echo "  cd $REPO_PATH"
  echo "  # Use Sruja AI skills: npx skills add sruja-ai/sruja --skill sruja-architecture-agent"
  echo "  # Then ask your AI: 'Analyze this codebase and create a Sruja architecture DSL'"
  exit 1
fi

echo ""
echo "============================================================"
echo "Evaluating: $REPO_NAME"
echo "============================================================"
echo ""

# Gather statistics (normalize to digits for report)
echo "📊 Gathering statistics..."
norm() { echo "$1" | tr -d ' \n'; }
LINES=$(norm "$(wc -l < "$ARCH_FILE")")
CHARS=$(norm "$(wc -c < "$ARCH_FILE")")
SYSTEMS=$(norm "$(grep -c "= system" "$ARCH_FILE" 2>/dev/null || echo "0")")
CONTAINERS=$(norm "$(grep -c "= container" "$ARCH_FILE" 2>/dev/null || echo "0")")
DATABASES=$(norm "$(grep -E "= database|= datastore" "$ARCH_FILE" 2>/dev/null | wc -l || echo "0")")
QUEUES=$(norm "$(grep -c "= queue" "$ARCH_FILE" 2>/dev/null || echo "0")")
PERSONS=$(norm "$(grep -c "= person" "$ARCH_FILE" 2>/dev/null || echo "0")")
RELATIONSHIPS=$(norm "$(grep -c -e "->" "$ARCH_FILE" 2>/dev/null || echo "0")")

echo "  Lines: $LINES"
echo "  Components: $SYSTEMS systems, $CONTAINERS containers, $DATABASES databases"
echo "  Relationships: $RELATIONSHIPS"
echo ""

# Run validation
echo "🔍 Running validation (sruja lint)..."
if command -v sruja >/dev/null 2>&1; then
  if sruja lint "$ARCH_FILE" 2>&1; then
    echo "✅ Validation passed"
    VALID="true"
  else
    echo "⚠️  Validation issues found (see above)"
    VALID="false"
  fi
else
  echo "⚠️  sruja CLI not found, skipping validation"
  echo "   Install with: curl -fsSL https://sruja.ai/install.sh | bash"
  VALID="unknown"
fi
echo ""

# Manual checklist
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          Manual Evaluation Checklist for $REPO_NAME"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Please review the generated architecture.sruja and answer:"
echo ""
echo "COMPLETENESS (Are main parts captured?)"
echo "  [ ] Main entry points identified"
echo "  [ ] Core modules/components documented"
echo "  [ ] Key data flows shown"
echo "  [ ] External dependencies included"
echo "  [ ] Important subsystems represented"
echo "  Score: ___/10"
echo ""
echo "ACCURACY (Does it match the codebase?)"
echo "  [ ] Component names are correct"
echo "  [ ] Relationships reflect actual dependencies"
echo "  [ ] No fabricated/hallucinated components"
echo "  [ ] Technology choices are accurate"
echo "  [ ] Architecture patterns are correct"
echo "  Score: ___/10"
echo ""
echo "CLARITY (Is it understandable?)"
echo "  [ ] Easy to see high-level structure"
echo "  [ ] Component purposes are clear"
echo "  [ ] Relationships are well-labeled"
echo "  [ ] Hierarchy makes sense"
echo "  [ ] Not overly complex"
echo "  Score: ___/10"
echo ""
echo "USEFULNESS (Would it help developers?)"
echo "  [ ] Would speed up onboarding"
echo "  [ ] Reveals important design decisions"
echo "  [ ] Helps understand complexity"
echo "  [ ] Better than README alone"
echo "  [ ] Could guide architectural changes"
echo "  Score: ___/10"
echo ""
echo "AVERAGE SCORE: ___/10"
echo ""
echo "VERDICT:"
echo "  [ ] Useful (≥7/10)"
echo "  [ ] Partially Useful (5-6/10)"
echo "  [ ] Not Useful (<5/10)"
echo ""
if [ -n "$USE_LLM" ]; then
  echo "🤖 Running LLM evaluation (sruja eval)..."
  SRUJA_CMD=$(find_sruja)
  HAS_LLM_KEY=$(has_llm_key)
  if [ -n "$HAS_LLM_KEY" ] && [ -n "$SRUJA_CMD" ]; then
    if $SRUJA_CMD eval "$REPO_PATH" 2>&1; then
      echo ""
      echo "✅ LLM evaluation complete"
    else
      echo "⚠️  LLM evaluation failed (check API key or SRUJA_LLM_PROVIDER)"
    fi
  else
    echo "⚠️  No LLM API key found. Skipping LLM eval."
    echo ""
    echo "   To enable: cp .env.example .env  (in this directory) and add a key"
    echo "   Or: export OPENAI_API_KEY=sk-...  (or OPENROUTER, ANTHROPIC, GEMINI)"
    echo "   Or: SRUJA_LLM_PROVIDER=ollama for local models"
    echo ""
  fi
  echo ""
else
  echo "Tip: Run with --llm for LLM evaluation (requires any LLM API key)"
  echo "     ./evaluate_architecture.sh $REPO_NAME --llm"
  echo ""
fi

# Generate report
REPORT_DIR="${SCRIPT_DIR}/results"
mkdir -p "$REPORT_DIR"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="${REPORT_DIR}/evaluation_${REPO_NAME}_${TIMESTAMP}.md"

cat > "$REPORT_FILE" << EOF
# Architecture Evaluation Report: $REPO_NAME

**Date**: $(date '+%Y-%m-%d %H:%M:%S')

## File Statistics

- **Lines**: $LINES
- **Characters**: $CHARS
- **Systems**: $SYSTEMS
- **Containers**: $CONTAINERS
- **Databases**: $DATABASES
- **Queues**: $QUEUES
- **Persons**: $PERSONS
- **Relationships**: $RELATIONSHIPS

## Validation

- **Status**: $([ "$VALID" = "true" ] && echo "✅ Valid" || echo "❌ Issues found")
- **File**: $ARCH_FILE

## Manual Evaluation

See checklist above for manual evaluation.

## Next Steps

1. Review generated architecture in context of codebase
2. Compare with existing documentation (if any)
3. Identify gaps and inaccuracies
4. Provide feedback to improve Sruja

---
*Generated by Sruja Real-World Test (shell + sruja CLI)*
EOF

echo "📄 Report saved to: $REPORT_FILE"
echo ""
echo "============================================================"
echo "✅ Evaluation complete!"
echo "============================================================"
echo ""
