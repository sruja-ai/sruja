#!/usr/bin/env bash
# Sruja E2E Value Demo - Smooth, fast, zero-config by default
#
# Run this to experience Sruja's full value in ~2 minutes.
# No API keys or config required for the fast path.
#
# Usage:
#   ./run_demo.sh              # Fast path only (quickstart, drift, scan)
#   ./run_demo.sh --baseline   # + drift vs example architecture
#   ./run_demo.sh --llm        # + LLM eval (requires any LLM API key)
#   ./run_demo.sh --all        # baseline + llm

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib.sh"
# Optional: load .env for LLM API keys (eval with --llm)
[ -f "${SCRIPT_DIR}/.env" ] && set -a && . "${SCRIPT_DIR}/.env" && set +a
REPOS_DIR="${SCRIPT_DIR}/test-repos"
DEMO_REPO="express"  # Small, fast to clone and scan

# Parse flags
WITH_BASELINE=""
WITH_LLM=""
for arg in "$@"; do
  case "$arg" in
    -h|--help)
      echo "Usage: $0 [OPTIONS]"
      echo ""
      echo "  (none)       Fast path: quickstart + drift (no config required)"
      echo "  --baseline   Also run drift vs example architecture"
      echo "  --llm        Also run LLM eval (requires any LLM API key in .env)"
      echo "  --all        Same as --baseline + --llm"
      echo "  -h, --help   Show this help"
      echo ""
      echo "From this directory: ./run_demo.sh"
      exit 0
      ;;
    --baseline) WITH_BASELINE="1" ;;
    --llm)      WITH_LLM="1" ;;
    --all)      WITH_BASELINE="1"; WITH_LLM="1" ;;
  esac
done

SRUJA=$(find_sruja)
if [ -z "$SRUJA" ]; then
  echo "❌ sruja CLI not found. Install first:"
  echo "   make build    # from sruja repo root"
  echo "   # or: curl -fsSL https://sruja.ai/install.sh | bash"
  exit 1
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  Sruja E2E Value Demo - Fast path (no config required)           ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

# ─── Phase 1: Setup (clone if needed) ───
REPO_PATH="${REPOS_DIR}/${DEMO_REPO}"
if [ ! -d "$REPO_PATH" ]; then
  echo "📂 Cloning ${DEMO_REPO} (one-time setup)..."
  mkdir -p "$REPOS_DIR"
  git clone --depth 1 "https://github.com/expressjs/express.git" "$REPO_PATH" 2>/dev/null || {
    echo "❌ Failed to clone. Check network."
    exit 1
  }
  echo "   ✓ Done"
  echo ""
else
  echo "📂 Using existing ${DEMO_REPO} repo"
  echo ""
fi

# ─── Phase 2: Fast path - Quickstart (zero config) ───
echo "════════════════════════════════════════════════════════════════════"
echo "  1. Quickstart - Architecture health snapshot (no keys)"
echo "════════════════════════════════════════════════════════════════════"
$SRUJA quickstart -r "$REPO_PATH" | tail -50
echo ""

# ─── Phase 3: Drift (scan-only, no baseline) ───
echo "════════════════════════════════════════════════════════════════════"
echo "  2. Drift - Structural analysis (cycles, orphans, layers)"
echo "════════════════════════════════════════════════════════════════════"
$SRUJA drift -r "$REPO_PATH" | tail -30
echo ""

# ─── Phase 4: Optional baseline ───
if [ -n "$WITH_BASELINE" ]; then
  echo "════════════════════════════════════════════════════════════════════"
  echo "  3. Drift vs baseline (example architecture)"
  echo "════════════════════════════════════════════════════════════════════"
  EXAMPLE_ARCH="${SCRIPT_DIR}/examples/example_generated_express.sruja"
  if [ -f "$EXAMPLE_ARCH" ]; then
    cp "$EXAMPLE_ARCH" "${REPO_PATH}/architecture.sruja"
    $SRUJA drift -r "$REPO_PATH" -a "${REPO_PATH}/architecture.sruja" 2>&1 | tail -40 || true
  else
    echo "   ⚠ Example architecture not found. Run without --baseline."
  fi
  echo ""
fi

# ─── Phase 5: Optional LLM eval ───
if [ -n "$WITH_LLM" ]; then
  echo "════════════════════════════════════════════════════════════════════"
  echo "  4. LLM evaluation (optional - requires any LLM API key)"
  echo "════════════════════════════════════════════════════════════════════"
  if [ -n "$(has_llm_key)" ]; then
    ARCH_FILE="${REPO_PATH}/architecture.sruja"
    EXAMPLE_ARCH="${SCRIPT_DIR}/examples/example_generated_express.sruja"
    if [ ! -f "$ARCH_FILE" ] && [ -f "$EXAMPLE_ARCH" ]; then
      cp "$EXAMPLE_ARCH" "$ARCH_FILE"
    fi
    if [ -f "$ARCH_FILE" ]; then
      $SRUJA eval "$REPO_PATH" 2>&1 || echo "   ⚠ LLM eval failed"
    else
      echo "   Example architecture not found. Run setup_repos.sh first."
    fi
  else
    echo "   ⚠ No LLM API key found. Skipping LLM eval."
    echo ""
    echo "   To enable: copy .env.example to .env and add a key"
    echo "   Or: export OPENAI_API_KEY=sk-...  (or OPENROUTER, ANTHROPIC, GEMINI)"
    echo "   Or: SRUJA_LLM_PROVIDER=ollama for local models"
    echo ""
  fi
  echo ""
fi

# ─── Summary ───
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  ✅ Demo complete                                               ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Next steps:"
echo "  • sruja quickstart -r .     # Try on your own repo"
echo "  • sruja drift -r .           # Structural drift"
echo "  • sruja analyze -r .         # Full analysis"
echo ""
if [ -z "$WITH_LLM" ]; then
  echo "Optional: ./run_demo.sh --llm        # Add LLM eval (set any LLM API key)"
fi
if [ -z "$WITH_BASELINE" ]; then
  echo "Optional: ./run_demo.sh --baseline   # Drift vs example architecture"
fi
echo ""
