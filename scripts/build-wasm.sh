#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")"/.. && pwd)"
OUT_DIR="$ROOT_DIR/apps/website/public/wasm"

mkdir -p "$OUT_DIR"

# Try new path first (Go 1.16+), fall back to old path
WASM_EXEC_JS="$(go env GOROOT)/lib/wasm/wasm_exec.js"
if [ ! -f "$WASM_EXEC_JS" ]; then
    WASM_EXEC_JS="$(go env GOROOT)/misc/wasm/wasm_exec.js"
fi
cp "$WASM_EXEC_JS" "$OUT_DIR/wasm_exec.js"

GOOS=js GOARCH=wasm go build -o "$OUT_DIR/sruja.wasm" "$ROOT_DIR/wasm"

echo "Built WASM to $OUT_DIR/sruja.wasm"
