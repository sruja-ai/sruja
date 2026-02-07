#!/usr/bin/env bash
# mdbook preprocessor: copy WASM into build output so every build (including serve) has wasm/rust/.
# First invocation: "supports" <renderer> -> exit 0 if we support it.
# Second: read JSON from stdin, run copy-wasm.sh, pass JSON through.
if [ "$1" = "supports" ]; then
  [ "$2" = "html" ] && exit 0 || exit 1
fi
tmp=$(mktemp -t mdbook-wasm.XXXXXX) || exit 1
trap 'rm -f "$tmp"' EXIT
cat > "$tmp"
# Run copy-wasm from the same directory as this script (book root)
dir="$(cd "$(dirname "$0")" && pwd)"
"$dir/copy-wasm.sh" >/dev/null 2>/dev/null || true
# mdbook expects only the book object (second element), not the full [context, book] array
if command -v jq >/dev/null 2>&1; then
  jq '.[1]' "$tmp"
else
  python3 -c "import json,sys; d=json.load(open(sys.argv[1])); print(json.dumps(d[1]))" "$tmp"
fi
