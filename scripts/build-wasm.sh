#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")"/.. && pwd)"
OUT_DIR="$ROOT_DIR/apps/website/public/wasm"
COMPRESS="${1:-}"

mkdir -p "$OUT_DIR"

# Try new path first (Go 1.16+), fall back to old path
WASM_EXEC_JS="$(go env GOROOT)/lib/wasm/wasm_exec.js"
if [ ! -f "$WASM_EXEC_JS" ]; then
    WASM_EXEC_JS="$(go env GOROOT)/misc/wasm/wasm_exec.js"
fi
cp "$WASM_EXEC_JS" "$OUT_DIR/wasm_exec.js"

# Build with optimization flags
GOOS=js GOARCH=wasm go build -ldflags="-s -w" -trimpath -o "$OUT_DIR/sruja.wasm" "$ROOT_DIR/wasm"

# Optimize with wasm-opt if available
if command -v wasm-opt >/dev/null 2>&1; then
    echo "Optimizing WASM with wasm-opt..."
    wasm-opt -Oz "$OUT_DIR/sruja.wasm" -o "$OUT_DIR/sruja.wasm"
fi

echo "Built WASM to $OUT_DIR/sruja.wasm"

# Compress if requested
if [ "$COMPRESS" = "compress" ] || [ "$COMPRESS" = "--compress" ]; then
    echo "Creating compressed versions..."
    gzip -k -f "$OUT_DIR/sruja.wasm" 2>/dev/null || true
    if command -v brotli >/dev/null 2>&1; then
        brotli -k -f "$OUT_DIR/sruja.wasm" 2>/dev/null || true
    fi
    echo "Compressed WASM files created:"
    ls -lh "$OUT_DIR"/sruja.wasm* 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'
fi
