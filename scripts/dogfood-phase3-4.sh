#!/usr/bin/env bash
# Dogfood Phase 3 (drift injector, context prune) + Phase 4 (memory) on the Sruja repo.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

SRUJA="${SRUJA:-./target/release/sruja}"
if [[ ! -x "$SRUJA" ]]; then
  echo "Building release sruja..."
  CARGO_TARGET_DIR="$ROOT/target" cargo build --release -p sruja-cli
  SRUJA="./target/release/sruja"
fi

FAIL=0
pass() { echo "  ✓ $*"; }
fail() { echo "  ✗ $*"; FAIL=1; }

echo "=== Phase 3/4 dogfood ($(basename "$SRUJA")) ==="

# drift_state/v1
if "$SRUJA" drift -r . -f drift-state 2>/dev/null | grep -q 'drift_state/v1'; then
  pass "drift -f drift-state"
else
  fail "drift -f drift-state"
fi

# MCP watch_drift via env (Cursor .cursor/mcp.json uses this)
OUT=$(mktemp)
(
  echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}'
  echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'
) | env SRUJA_MCP_WATCH_DRIFT=1 "$SRUJA" mcp -r . 2>/dev/null >"$OUT" || true
if grep -q 'notifications/drift_state' "$OUT"; then
  pass "MCP SRUJA_MCP_WATCH_DRIFT → notifications/drift_state"
else
  fail "MCP SRUJA_MCP_WATCH_DRIFT"
fi
rm -f "$OUT"

# context prune tool
OUT=$(mktemp)
(
  echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}'
  echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'
  echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"sruja_suggest_context_prune","arguments":{"path":".","active_element_ids":["Sruja"],"session_element_ids":["Sruja","orphan_xyz"],"depth":2}}}'
) | "$SRUJA" mcp -r . 2>/dev/null >"$OUT" || true
if grep -q 'compress_ids' "$OUT" && grep -q 'orphan_xyz' "$OUT"; then
  pass "MCP sruja_suggest_context_prune"
else
  fail "MCP sruja_suggest_context_prune"
fi
rm -f "$OUT"

# memory
"$SRUJA" memory reindex -r . >/dev/null
if "$SRUJA" memory search -r . "Merging" 2>/dev/null | grep -q '"count":'; then
  pass "memory search"
else
  # empty index is ok on fresh clone
  if "$SRUJA" memory search -r . "drift" 2>/dev/null | grep -q '"schema_version"'; then
    pass "memory search (schema ok)"
  else
    fail "memory search"
  fi
fi

# Cursor wiring files
[[ -f .cursor/mcp.json ]] && grep -q SRUJA_MCP_WATCH_DRIFT .cursor/mcp.json && pass ".cursor/mcp.json" || fail ".cursor/mcp.json"
[[ -f .cursor/rules/sruja-context-host.mdc ]] && pass "sruja-context-host rule" || fail "sruja-context-host rule"

echo ""
if [[ $FAIL -eq 0 ]]; then
  echo "✅ Phase 3/4 dogfood passed"
  exit 0
fi
echo "❌ Phase 3/4 dogfood failed"
exit 1
