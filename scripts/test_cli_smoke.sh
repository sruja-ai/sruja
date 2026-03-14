#!/bin/bash
# Smoke test for documented CLI command shapes
# Validates that all documented commands are available and produce expected output

set -e

echo "🔍 Running CLI smoke tests..."

# Build the CLI if not already built
if [ ! -f "./target/release/sruja" ]; then
  echo "📦 Building CLI..."
  make build > /dev/null 2>&1
fi

SRUJA="./target/release/sruja"
REPO_ROOT="evaluation/real-world-test/test-repos/express"
FAILED=0
PASSED=0

# Function to test a command
test_command() {
  local name="$1"
  local cmd="$2"
  local expected_exit="$3"
  local output_file="$4"
  
  echo -n "  Testing $name... "
  
  if eval "$cmd > $output_file 2>&1"; then
    actual_exit=0
  else
    actual_exit=$?
  fi
  
  if [ "$actual_exit" -eq "$expected_exit" ]; then
    echo "✓"
    PASSED=$((PASSED + 1))
  else
    echo "✗ (exit $actual_exit, expected $expected_exit)"
    cat "$output_file"
    FAILED=$((FAILED + 1))
  fi
}

# Test repository
if [ ! -d "$REPO_ROOT" ]; then
  echo "⚠️  Test repo not found: $REPO_ROOT"
  echo "   Skipping tests that require a repo"
  REPO_AVAILABLE=false
else
  REPO_AVAILABLE=true
fi

# === P0 Core Commands ===
echo ""
echo "📋 Testing P0 core commands..."

test_command "quickstart text" "$SRUJA quickstart -r . -f text" 0 "/tmp/quickstart.txt"
test_command "quickstart json" "$SRUJA quickstart -r . -f json" 0 "/tmp/quickstart.json"
test_command "drift text" "$SRUJA drift -r . -f text" 0 "/tmp/drift.txt"
test_command "drift json" "$SRUJA drift -r . -f json" 0 "/tmp/drift.json"
test_command "lint text" "$SRUJA lint --format text book/valid-examples/pattern-microservices.sruja" 0 "/tmp/lint.txt"
test_command "lint json" "$SRUJA lint --format json book/valid-examples/pattern-microservices.sruja" 0 "/tmp/lint.json"
test_command "export json" "$SRUJA export json book/valid-examples/pattern-microservices.sruja" 0 "/tmp/export.json"
test_command "export mermaid" "$SRUJA export mermaid book/valid-examples/pattern-microservices.sruja" 0 "/tmp/export.mermaid"
test_command "export markdown" "$SRUJA export markdown book/valid-examples/pattern-microservices.sruja" 0 "/tmp/export.md"
test_command "context json" "$SRUJA context -r . -f json" 0 "/tmp/context.json"

if [ "$REPO_AVAILABLE" = true ]; then
  test_command "quickstart on repo" "$SRUJA quickstart -r $REPO_ROOT -f json" 0 "/tmp/repo-quickstart.json"
  test_command "drift on repo" "$SRUJA drift -r $REPO_ROOT -f json" 0 "/tmp/repo-drift.json"
fi

# === P1 Supplemental Commands ===
echo ""
echo "📋 Testing P1 supplemental commands..."

test_command "scan" "$SRUJA scan . --output /tmp/scan.json" 0 "/tmp/scan-out.txt"
test_command "list" "$SRUJA list book/valid-examples/pattern-microservices.sruja" 0 "/tmp/list.txt"
test_command "tree" "$SRUJA tree book/valid-examples/pattern-microservices.sruja" 0 "/tmp/tree.txt"
test_command "explain" "$SRUJA explain user --file book/valid-examples/pattern-microservices.sruja" 1 "/tmp/explain.txt"
test_command "fmt" "$SRUJA fmt book/valid-examples/pattern-microservices.sruja --check" 1 "/tmp/fmt.txt"
test_command "validate" "$SRUJA validate book/valid-examples/pattern-microservices.sruja" 0 "/tmp/validate.txt"

if [ "$REPO_AVAILABLE" = true ]; then
  test_command "intent check" "$SRUJA intent check -r $REPO_ROOT -f json" 0 "/tmp/intent-check.json"
  test_command "compliance" "$SRUJA compliance -r $REPO_ROOT -f json" 1 "/tmp/compliance.json"
fi

# === Validate JSON Outputs ===
echo ""
echo "📋 Validating JSON outputs..."

validate_json() {
  local json_file="$1"
  if [ -f "$json_file" ]; then
    echo -n "  Validating $json_file... "
    # Some commands output text before JSON, extract JSON from the end
    if grep -q '^{' "$json_file"; then
      # Extract JSON portion (from first { to end)
      local temp_json="/tmp/validate-$(basename $json_file)"
      sed -n '/^{/,$p' "$json_file" > "$temp_json"
      if jq empty "$temp_json" 2>/dev/null; then
        echo "✓"
        PASSED=$((PASSED + 1))
        rm -f "$temp_json"
        return 0
      else
        echo "✗ (invalid JSON)"
        FAILED=$((FAILED + 1))
        rm -f "$temp_json"
        return 1
      fi
    else
      # Try to validate entire file as JSON
      if jq empty "$json_file" 2>/dev/null; then
        echo "✓"
        PASSED=$((PASSED + 1))
        return 0
      else
        echo "✗ (invalid JSON)"
        FAILED=$((FAILED + 1))
        return 1
      fi
    fi
  fi
}

validate_json "/tmp/quickstart.json"
validate_json "/tmp/drift.json"
validate_json "/tmp/lint.json"
validate_json "/tmp/export.json"
validate_json "/tmp/context.json"

# === Summary ===
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "Smoke test complete"
echo "  Passed: $PASSED"
echo "  Failed: $FAILED"
echo "═══════════════════════════════════════════════════════════════"

if [ "$FAILED" -gt 0 ]; then
  echo "❌ Some tests failed"
  exit 1
else
  echo "✅ All tests passed"
  exit 0
fi
