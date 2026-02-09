#!/usr/bin/env bash
# Copy WASM shim (Node) and LICENSE into the extension for packaging.
# Lint and export use WASM by default; set sruja.lsp.path to use the Sruja CLI instead.
# Run from repo root: extension/scripts/copy-assets.sh
# Or from extension/: scripts/copy-assets.sh (REPO_ROOT = ..)
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$EXT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# 1. WASM (Node target) for in-extension lint and markdown export
WASM_SRC="crates/sruja-wasm/pkg-nodejs"
if [ ! -f "$WASM_SRC/sruja_wasm.js" ] || [ ! -f "$WASM_SRC/sruja_wasm_bg.wasm" ]; then
  echo "Building Node WASM (wasm-pack nodejs)..."
  if ! command -v wasm-pack >/dev/null 2>&1; then
    echo "⚠️  wasm-pack not found. Install: cargo install wasm-pack"
    exit 1
  fi
  wasm-pack build --target nodejs --out-dir pkg-nodejs crates/sruja-wasm --release
fi
mkdir -p "$EXT_DIR/wasm"
cp "$WASM_SRC/sruja_wasm.js" "$WASM_SRC/sruja_wasm_bg.wasm" "$EXT_DIR/wasm/"
echo "✅ Copied WASM shim to extension/wasm/"

# 2. LICENSE (vsce expects LICENSE, LICENSE.md, or LICENSE.txt)
if [ -f "LICENSE-APACHE" ]; then
  cp LICENSE-APACHE "$EXT_DIR/LICENSE.txt"
  echo "✅ Copied LICENSE.txt"
else
  echo "⚠️  LICENSE-APACHE not found at repo root"
fi
