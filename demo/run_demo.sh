#!/usr/bin/env bash
# Architecture Intelligence demo: intent → scan → drift → analyze → why (deterministic)
# Run from repo root: make demo-intel   OR   cd demo && ./run_demo.sh
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
# Reuse E2E helper to find sruja binary (target/release, target/debug, or PATH)
if [ -f "${REPO_ROOT}/evaluation/real-world-test/lib.sh" ]; then
  . "${REPO_ROOT}/evaluation/real-world-test/lib.sh"
else
  find_sruja() {
    [ -f "${REPO_ROOT}/target/release/sruja" ] && echo "${REPO_ROOT}/target/release/sruja" && return
    [ -f "${REPO_ROOT}/target/debug/sruja" ] && echo "${REPO_ROOT}/target/debug/sruja" && return
    command -v sruja >/dev/null 2>&1 && echo "sruja" && return
    echo ""
  }
fi

SRUJA=$(find_sruja)
if [ -z "$SRUJA" ]; then
  echo "❌ sruja CLI not found. From repo root run: make build"
  echo "   Or: cargo build --release -p sruja-cli"
  exit 1
fi

cd "$SCRIPT_DIR"

echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  Sruja Architecture Intelligence Demo                           ║"
echo "║  Intent → Scan → Drift → Analyze → Why (deterministic)           ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

echo "────────────────────────────────────────────────────────────────────"
echo "  [1] The rulebook (intent)"
echo "────────────────────────────────────────────────────────────────────"
echo "architecture.sruja declares: Frontend must not talk to Database."
echo ""
grep -v '^$' architecture.sruja
echo ""

echo "────────────────────────────────────────────────────────────────────"
echo "  [2] The reality (code scan)"
echo "────────────────────────────────────────────────────────────────────"
echo "Scanning demo Python services to build the dependency graph..."
echo "  \$ $SRUJA scan --output sruja.graph.json"
"$SRUJA" scan --output sruja.graph.json
echo "  ✓ Graph written to sruja.graph.json"
echo ""

echo "────────────────────────────────────────────────────────────────────"
echo "  [3] Detecting drift (code vs. intent)"
echo "────────────────────────────────────────────────────────────────────"
echo "Comparing code against architecture.sruja rules..."
echo "  \$ $SRUJA drift -a architecture.sruja"
"$SRUJA" drift -a architecture.sruja
echo ""

echo "────────────────────────────────────────────────────────────────────"
echo "  [4] Runtime intelligence (distributed traces)"
echo "────────────────────────────────────────────────────────────────────"
echo "Merging traces.json into CTO view..."
echo "  \$ $SRUJA analyze --view cto -t traces.json"
"$SRUJA" analyze --view cto -t traces.json 2>/dev/null || echo "  (traces.json not found; step skipped)"
echo ""

echo "────────────────────────────────────────────────────────────────────"
echo "  [5] Deterministic explainability (sruja why)"
echo "────────────────────────────────────────────────────────────────────"
echo "Asking: Why does the Frontend access the database?"
echo "  \$ $SRUJA why \"Why does the Frontend access the database?\" -r . --graph sruja.graph.json"
"$SRUJA" why "Why does the Frontend access the database?" -r . --graph sruja.graph.json 2>/dev/null || true
echo ""
echo "  For natural-language interpretation, use the Sruja skill in your editor (Cursor, Copilot, etc.)."
echo ""

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  ✅ Architecture Intelligence demo complete                      ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Next: sruja quickstart -r .   sruja drift -r .   sruja analyze -r ."
echo ""
