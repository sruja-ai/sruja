#!/usr/bin/env bash
# Build book, copy WASM into output, then serve. Use this so "Show diagram" works.
# mdbook's render step wipes the build dir on every build (including live reload),
# so we run a loop that re-copies WASM into the output every 0.5s. That way
# wasm/rust/ is restored quickly after each render.
# Run from book/: ./serve.sh
# Or from repo root: book/serve.sh

set -e
BOOK_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$BOOK_ROOT"
REPO_ROOT="$(cd "$BOOK_ROOT/.." && pwd)"
# mdbook output (build-dir = "book" in book.toml); don't require dir to exist yet
OUTPUT_DIR="$BOOK_ROOT/book"

# Ensure WASM is built (so copy-wasm.sh can find files)
if ! [ -f "$REPO_ROOT/crates/sruja-wasm/pkg/sruja_wasm.js" ] && ! [ -f "$REPO_ROOT/book/wasm/rust/sruja_wasm.js" ]; then
  echo "⚠️  WASM not built. From repo root run: make wasm"
  echo "    Then run make book-serve again."
  exit 1
fi

mdbook build
"$REPO_ROOT/book/copy-wasm.sh" "$OUTPUT_DIR" || { echo "⚠️  copy-wasm failed"; exit 1; }

# mdbook serve in background (its initial build wipes the output dir)
mdbook serve "$@" &
SERVE_PID=$!
trap 'kill $SERVE_PID 2>/dev/null' EXIT

# Re-copy WASM every 0.5s so it reappears after each render
while true; do
  "$REPO_ROOT/book/copy-wasm.sh" "$OUTPUT_DIR" >/dev/null 2>/dev/null || true
  sleep 0.5
done
