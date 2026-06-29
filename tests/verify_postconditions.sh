#!/usr/bin/env bash
# Post-condition verification script for the autonomous agent subtask.
# Validates all acceptance criteria. Exits non-zero on any failure.
set -euo pipefail

REPO_SRUJA="repo.sruja"
PASS=0
FAIL=0

pass() {
  echo "  ✅ PASS: $1"
  PASS=$((PASS + 1))
}

fail() {
  echo "  ❌ FAIL: $1"
  FAIL=$((FAIL + 1))
}

echo "============================================"
echo "  Post-condition verification: repo.sruja"
echo "============================================"
echo ""

# ---------- Criterion 1: Agent description contains 'autonomous agent' ----------
echo "Criterion 1: Agent description contains 'autonomous agent'"
if grep -q 'autonomous agent' "$REPO_SRUJA"; then
  pass "Found 'autonomous agent' in repo.sruja"
else
  fail "Did not find 'autonomous agent' in repo.sruja"
fi
echo ""

# ---------- Criterion 2: Agent description contains the observe-act-verify-critique-replan loop ----------
echo "Criterion 2: Agent description contains 'observe-act-verify-critique-replan loop'"
if grep -q 'observe-act-verify-critique-replan loop' "$REPO_SRUJA"; then
  pass "Found 'observe-act-verify-critique-replan loop' in repo.sruja"
else
  fail "Did not find 'observe-act-verify-critique-replan loop' in repo.sruja"
fi
echo ""

# ---------- Criterion 3: Relationship 'CLI -> Agent "Drives autonomous loop"' exists ----------
echo "Criterion 3: Relationship 'CLI -> Agent \"Drives autonomous loop\"' exists"
if grep -q 'CLI -> Agent "Drives autonomous loop"' "$REPO_SRUJA"; then
  pass "Found 'CLI -> Agent \"Drives autonomous loop\"'"
else
  fail "Did not find 'CLI -> Agent \"Drives autonomous loop\"'"
fi
echo ""

# ---------- Criterion 4: Count of 'CLI -> Agent' lines is exactly 2 ----------
echo "Criterion 4: Count of 'CLI -> Agent' lines is exactly 2"
CLI_AGENT_COUNT=$(grep -c 'CLI -> Agent' "$REPO_SRUJA" || true)
if [ "$CLI_AGENT_COUNT" -eq 2 ]; then
  pass "Count of 'CLI -> Agent' lines is exactly 2 (found $CLI_AGENT_COUNT)"
else
  fail "Count of 'CLI -> Agent' lines is $CLI_AGENT_COUNT, expected 2"
fi
echo ""

# ---------- Criterion 5: Other container definitions are unchanged ----------
echo "Criterion 5: Other container definitions (Core, Context, Export, Book, IDE) unchanged"

# Expected line ranges extracted from the baseline (pre-change) repo.sruja
# These were captured before any implementation changes.
EXPECTED_CORE=$(cat <<'EOF'
  Core = container "Core (Language + Validation + Diagnostics)" {
    technology "Rust"
    description "Parsing, validation rules, and diagnostic formatting"
    tags ["core"]
  }
EOF
)

EXPECTED_CONTEXT=$(cat <<'EOF'
  Context = container "Context Engineering (Scan + Graph + Drift + Intent)" {
    technology "Rust, Tree-sitter"
    description "Evidence capture (scan + graph), drift detection, intent checks, and context that grounds AI suggestions in repo truth"
    tags ["context"]
  }
EOF
)

EXPECTED_EXPORT=$(cat <<'EOF'
  Export = container "Export System" {
    technology "Rust"
    description "Exports validated architecture models to multiple formats"
    tags ["export"]
  }
EOF
)

EXPECTED_BOOK=$(cat <<'EOF'
  Book = container "Documentation Site (mdBook)" {
    technology "Rust, mdBook"
    description "Docs site and courses built from markdown; published as a static site"
    tags ["docs"]
  }
EOF
)

EXPECTED_IDE=$(cat <<'EOF'
  IDE = container "IDE Support (VS Code + WASM)" {
    technology "TypeScript, WASM"
    description "Editor validation, diagnostics, and preview powered by the WASM build"
    tags ["ide"]
  }
EOF
)

# We extract the container block from the current file and compare.
# Helper: extract a container block between its opening '{' and closing '}' at the right indent level.
extract_block() {
  local label="$1"
  local file="$2"
  # Use awk to find the container definition and extract the block.
  awk -v label="$label" '
    $0 ~ label && /container/ { found=1; depth=0 }
    found {
      # count braces
      n = gsub(/{/, "{")
      depth += n
      n = gsub(/}/, "}")
      depth -= n
      print
      if (depth == 0) found=0
    }
  ' "$file"
}

CURRENT_CORE=$(extract_block 'Core' "$REPO_SRUJA")
CURRENT_CONTEXT=$(extract_block 'Context' "$REPO_SRUJA")
CURRENT_EXPORT=$(extract_block 'Export' "$REPO_SRUJA")
CURRENT_BOOK=$(extract_block 'Book' "$REPO_SRUJA")
CURRENT_IDE=$(extract_block 'IDE' "$REPO_SRUJA")

unchanged() {
  local name="$1"
  local expected="$2"
  local current="$3"
  if [ "$expected" = "$current" ]; then
    pass "$name container definition is unchanged"
  else
    fail "$name container definition was modified"
    echo "    Expected:"
    echo "$expected" | sed 's/^/      /'
    echo "    Actual:"
    echo "$current" | sed 's/^/      /'
  fi
}

unchanged "Core" "$EXPECTED_CORE" "$CURRENT_CORE"
unchanged "Context" "$EXPECTED_CONTEXT" "$CURRENT_CONTEXT"
unchanged "Export" "$EXPECTED_EXPORT" "$CURRENT_EXPORT"
unchanged "Book" "$EXPECTED_BOOK" "$CURRENT_BOOK"
unchanged "IDE" "$EXPECTED_IDE" "$CURRENT_IDE"
echo ""

# ---------- Summary ----------
echo "============================================"
TOTAL=$((PASS + FAIL))
echo "  Results: $PASS/$TOTAL passed, $FAIL/$TOTAL failed"
echo "============================================"

if [ "$FAIL" -gt 0 ]; then
  echo "  ❌ VERIFICATION FAILED"
  exit 1
fi

echo "  ✅ ALL CHECKS PASSED"
exit 0
