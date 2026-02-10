#!/usr/bin/env bash
# Copy WASM files into the mdBook build output so "Show diagram" works for ```sruja blocks.
# Run from repo root: book/copy-wasm.sh [output-dir]
# Or from book/: ./copy-wasm.sh
# If output-dir is given (e.g. book/book), copy there; else use default BOOK_ROOT/book.

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BOOK_ROOT="$SCRIPT_DIR"
# In CI, use GITHUB_WORKSPACE so path resolution is unambiguous regardless of cwd
if [ -n "${GITHUB_WORKSPACE:-}" ] && [ -d "${GITHUB_WORKSPACE}" ]; then
  REPO_ROOT="$GITHUB_WORKSPACE"
else
  REPO_ROOT="$(cd "$BOOK_ROOT/.." && pwd)"
fi
# Resolve output dir to absolute path so copy works regardless of cwd
if [ -n "$1" ]; then
  case "$1" in
    /*) OUTPUT_DIR="$1" ;;
    *)  OUTPUT_DIR="$BOOK_ROOT/$1" ;;
  esac
  [ -d "$OUTPUT_DIR" ] || mkdir -p "$OUTPUT_DIR"
  OUTPUT_DIR="$(cd "$OUTPUT_DIR" && pwd)"
else
  OUTPUT_DIR="$BOOK_ROOT/book"
fi
WASM_DEST="$BOOK_ROOT/wasm/rust"

# Prefer book/wasm/rust (after make wasm); fallback to crate pkg
WASM_SRC=""
if [ -f "$REPO_ROOT/book/wasm/rust/sruja_wasm.js" ] && [ -f "$REPO_ROOT/book/wasm/rust/sruja_wasm_bg.wasm" ]; then
  WASM_SRC="$REPO_ROOT/book/wasm/rust"
elif [ -f "$REPO_ROOT/crates/sruja-wasm/pkg/sruja_wasm.js" ] && [ -f "$REPO_ROOT/crates/sruja-wasm/pkg/sruja_wasm_bg.wasm" ]; then
  WASM_SRC="$REPO_ROOT/crates/sruja-wasm/pkg"
fi

mkdir -p "$WASM_DEST"
if [ -n "$WASM_SRC" ]; then
  # Only copy to WASM_DEST if different from source (avoid cp "identical" error)
  WASM_SRC_ABS="$(cd "$WASM_SRC" && pwd)"
  WASM_DEST_ABS="$(cd "$WASM_DEST" && pwd)"
  if [ "$WASM_SRC_ABS" != "$WASM_DEST_ABS" ]; then
    cp "$WASM_SRC/sruja_wasm.js" "$WASM_SRC/sruja_wasm_bg.wasm" "$WASM_DEST/"
  fi
  mkdir -p "$OUTPUT_DIR/wasm/rust"
  cp "$WASM_SRC/sruja_wasm.js" "$WASM_SRC/sruja_wasm_bg.wasm" "$OUTPUT_DIR/wasm/rust/"
  echo "Copied WASM into book output ($OUTPUT_DIR/wasm/rust/)"
fi
# Copy Sruja logo into book output (for sidebar and pages)
if [ -f "$BOOK_ROOT/sruja-logo.png" ]; then
  mkdir -p "$OUTPUT_DIR"
  cp "$BOOK_ROOT/sruja-logo.png" "$OUTPUT_DIR/sruja-logo.png"
  echo "Copied sruja-logo.png into book output"
fi
if [ -z "$WASM_SRC" ]; then
  echo "WASM not found. Run from repo root: make wasm"
  echo "  Then rebuild/serve the book so wasm/rust/ is copied into the output."
  exit 1
fi
