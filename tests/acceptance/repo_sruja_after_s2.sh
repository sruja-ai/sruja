#!/usr/bin/env bash
# acceptance tests for repo.sruja state after s2 implementation
# runs grep-based assertions to validate the architecture file

set -euo pipefail

FILE="repo.sruja"
PASS=0
FAIL=0
TOTAL=0

assert_match_count() {
  local label="$1"
  local pattern="$2"
  local file="$3"
  local expected="$4"
  local op="$5" # eq or gte
  local actual
  actual=$(grep -c "$pattern" "$file" || true)
  TOTAL=$((TOTAL + 1))
  if [ "$op" = "eq" ]; then
    if [ "$actual" -eq "$expected" ]; then
      echo "  PASS: $label (found $actual match(es), expected $expected)"
      PASS=$((PASS + 1))
    else
      echo "  FAIL: $label (found $actual match(es), expected $expected)"
      FAIL=$((FAIL + 1))
    fi
  elif [ "$op" = "gte" ]; then
    if [ "$actual" -ge "$expected" ]; then
      echo "  PASS: $label (found $actual match(es), expected >= $expected)"
      PASS=$((PASS + 1))
    else
      echo "  FAIL: $label (found $actual match(es), expected >= $expected)"
      FAIL=$((FAIL + 1))
    fi
  fi
}

assert_line_count_increase() {
  local file="$1"
  local expected_increase="$2"
  local baseline
  baseline=$(git show HEAD:"$file" 2>/dev/null | wc -l | tr -d ' ')
  local current
  current=$(wc -l < "$file" | tr -d ' ')
  local actual_increase=$((current - baseline))
  TOTAL=$((TOTAL + 1))
  if [ "$actual_increase" -eq "$expected_increase" ]; then
    echo "  PASS: line count increase is exactly $expected_increase (baseline=$baseline, current=$current)"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: line count increase is $actual_increase (expected $expected_increase; baseline=$baseline, current=$current)"
    FAIL=$((FAIL + 1))
  fi
}

echo "=== Acceptance: repo.sruja post-s2 validation ==="

echo ""
echo "--- Assertions ---"

# Assertion 1: 'autonomous agent' appears (description updated)
assert_match_count \
  "A1: description includes 'autonomous agent'" \
  "autonomous agent" \
  "$FILE" \
  1 \
  "gte"

# Assertion 2: 'autonomous loop' appears (description includes loop)
assert_match_count \
  "A2: description includes 'autonomous loop'" \
  "autonomous loop" \
  "$FILE" \
  1 \
  "gte"

# Assertion 3: 'Drives autonomous loop' appears exactly once (new relationship)
assert_match_count \
  "A3: 'Drives autonomous loop' relationship exists exactly once" \
  "Drives autonomous loop" \
  "$FILE" \
  1 \
  "eq"

# Assertion 4: 'Manages memory and learning' preserved exactly once
assert_match_count \
  "A4: 'Manages memory and learning' preserved exactly once" \
  "Manages memory and learning" \
  "$FILE" \
  1 \
  "eq"

# Assertion 5: line count increased by exactly 1 (one new relationship, nothing else)
assert_line_count_increase "$FILE" 1

# Assertion 6: all existing CLI -> relationships still present
assert_match_count \
  "A6a: CLI -> Core relationship present" \
  "CLI -> Core" \
  "$FILE" \
  1 \
  "eq"

assert_match_count \
  "A6b: CLI -> Context relationship present" \
  "CLI -> Context" \
  "$FILE" \
  1 \
  "eq"

assert_match_count \
  "A6c: CLI -> Export relationship present" \
  "CLI -> Export" \
  "$FILE" \
  1 \
  "eq"

assert_match_count \
  "A6d: CLI -> Book relationship present" \
  "CLI -> Book" \
  "$FILE" \
  1 \
  "eq"

echo ""
echo "=== Results: $PASS/$TOTAL passed, $FAIL failed ==="

if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
