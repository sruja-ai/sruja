#!/bin/bash

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

error() { echo -e "${RED}${BOLD}✗ ERROR:${NC} $1" >&2; }
warn() { echo -e "${YELLOW}${BOLD}⚠ WARNING:${NC} $1" >&2; }
success() { echo -e "${GREEN}${BOLD}✓${NC} $1"; }
info() { echo -e "${BLUE}→${NC} $1"; }
header() { echo -e "${CYAN}${BOLD}$1${NC}"; }

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_NAME="${1:-}"
RESULTS_DIR="${2:-$PROJECT_ROOT/evaluation/results}"

if [ -z "$PROJECT_NAME" ]; then
  error "Project name required"
  echo "Usage: $0 <project_name> [results_dir]"
  exit 1
fi

PROJECT_DIR="$RESULTS_DIR/$PROJECT_NAME"

if [ ! -d "$PROJECT_DIR" ]; then
  error "Project directory not found: $PROJECT_DIR"
  exit 1
fi

header "═══════════════════════════════════════════════════════════"
header "  Evaluating Agent Output: $PROJECT_NAME"
header "═══════════════════════════════════════════════════════════"
echo ""

# Check if architecture.sruja exists
ARCH_FILE="$PROJECT_DIR/architecture.sruja"
if [ ! -f "$ARCH_FILE" ]; then
  error "architecture.sruja not found at: $ARCH_FILE"
  echo ""
  echo "Please generate the architecture first using agent analysis."
  echo "See: $PROJECT_DIR/AGENT_INSTRUCTIONS.md"
  exit 1
fi

success "Found architecture.sruja"

# Initialize scores
TOTAL_SCORE=0
MAX_SCORE=100

# 1. Lint Validation (20 points)
header "[1/5] Lint Validation"
if "$PROJECT_ROOT/target/release/sruja" lint "$ARCH_FILE" > "$PROJECT_DIR/lint.log" 2>&1; then
  LINT_SCORE=20
  LINT_STATUS="✅ Pass"
  success "Lint validation passed"
else
  LINT_SCORE=0
  LINT_STATUS="❌ Fail"
  error "Lint validation failed"
  cat "$PROJECT_DIR/lint.log" | head -20
fi
echo ""

# 2. Component Count (20 points)
header "[2/5] Abstraction Level"
COMPONENT_COUNT=$(grep -c "component\|container" "$ARCH_FILE" 2>/dev/null || echo "0")
SYSTEM_COUNT=$(grep -c "^system" "$ARCH_FILE" 2>/dev/null || echo "0")
CONTAINER_COUNT=$(grep -c "container" "$ARCH_FILE" 2>/dev/null || echo "0")

info "Total components/containers: $COMPONENT_COUNT"
info "Systems: $SYSTEM_COUNT"
info "Containers: $CONTAINER_COUNT"

# Score based on abstraction appropriateness
if [ "$COMPONENT_COUNT" -ge 10 ] && [ "$COMPONENT_COUNT" -le 50 ]; then
  ABSTRACTION_SCORE=20
  ABSTRACTION_STATUS="✅ Appropriate (10-50 components)"
elif [ "$COMPONENT_COUNT" -ge 5 ] && [ "$COMPONENT_COUNT" -le 100 ]; then
  ABSTRACTION_SCORE=10
  ABSTRACTION_STATUS="⚠️ Acceptable (5-100 components)"
else
  ABSTRACTION_SCORE=0
  if [ "$COMPONENT_COUNT" -lt 5 ]; then
    ABSTRACTION_STATUS="❌ Too abstract (<5 components)"
  else
    ABSTRACTION_STATUS="❌ Too granular (>100 components)"
  fi
fi

echo "$ABSTRACTION_STATUS"
TOTAL_SCORE=$((TOTAL_SCORE + ABSTRACTION_SCORE))
echo ""

# 3. Drift Comparison (30 points)
header "[3/5] Drift Comparison"
REPO_PATH="/tmp/sruja_test_$PROJECT_NAME"

if [ -d "$REPO_PATH" ]; then
  if "$PROJECT_ROOT/target/release/sruja" drift -r "$REPO_PATH" -a "$ARCH_FILE" > "$PROJECT_DIR/agent_drift.log" 2>&1; then
    DRIFT_VIOLATIONS=0
    DRIFT_SCORE=30
    DRIFT_STATUS="✅ No violations"
    success "No drift violations found"
  else
    DRIFT_VIOLATIONS=$(grep -c "violation\|error" "$PROJECT_DIR/agent_drift.log" 2>/dev/null || echo "1")
    
    if [ "$DRIFT_VIOLATIONS" -le 3 ]; then
      DRIFT_SCORE=20
      DRIFT_STATUS="⚠️ Minor violations ($DRIFT_VIOLATIONS)"
    elif [ "$DRIFT_VIOLATIONS" -le 10 ]; then
      DRIFT_SCORE=10
      DRIFT_STATUS="⚠️ Some violations ($DRIFT_VIOLATIONS)"
    else
      DRIFT_SCORE=0
      DRIFT_STATUS="❌ Many violations ($DRIFT_VIOLATIONS)"
    fi
    
    warn "$DRIFT_STATUS"
    tail -20 "$PROJECT_DIR/agent_drift.log"
  fi
else
  DRIFT_SCORE=0
  DRIFT_STATUS="⏭️ Skipped (repo not found)"
  warn "Repository not found, skipping drift comparison"
fi

TOTAL_SCORE=$((TOTAL_SCORE + DRIFT_SCORE))
echo ""

# 4. Completeness Check (15 points)
header "[4/5] Completeness Check"

# Check for essential elements
HAS_DESCRIPTION=$(grep -c "description" "$ARCH_FILE" 2>/dev/null || echo "0")
HAS_TECHNOLOGY=$(grep -c "technology" "$ARCH_FILE" 2>/dev/null || echo "0")
HAS_RELATIONSHIPS=$(grep -c "->" "$ARCH_FILE" 2>/dev/null || echo "0")

COMPLETENESS_SCORE=0
COMPLETENESS_ITEMS=()

if [ "$HAS_DESCRIPTION" -gt 0 ]; then
  COMPLETENESS_ITEMS+=("✓ Descriptions")
  COMPLETENESS_SCORE=$((COMPLETENESS_SCORE + 5))
else
  COMPLETENESS_ITEMS+=("✗ No descriptions")
fi

if [ "$HAS_TECHNOLOGY" -gt 0 ]; then
  COMPLETENESS_ITEMS+=("✓ Technology tags")
  COMPLETENESS_SCORE=$((COMPLETENESS_SCORE + 5))
else
  COMPLETENESS_ITEMS+=("✗ No technology tags")
fi

if [ "$HAS_RELATIONSHIPS" -gt 0 ]; then
  COMPLETENESS_ITEMS+=("✓ Relationships")
  COMPLETENESS_SCORE=$((COMPLETENESS_SCORE + 5))
else
  COMPLETENESS_ITEMS+=("✗ No relationships")
fi

for item in "${COMPLETENESS_ITEMS[@]}"; do
  info "$item"
done

TOTAL_SCORE=$((TOTAL_SCORE + COMPLETENESS_SCORE))
echo ""

# 5. Code Quality (15 points)
header "[5/5] Code Quality"

# Check for proper formatting and structure
HAS_DUPLICATES=$(sort "$ARCH_FILE" | uniq -d | wc -l | tr -d ' ')
LINE_COUNT=$(wc -l < "$ARCH_FILE" | tr -d ' ')
AVG_LINE_LENGTH=$(awk '{ total += length($0); count++ } END { if (count > 0) print int(total/count); else print 0 }' "$ARCH_FILE")

QUALITY_SCORE=0

if [ "$HAS_DUPLICATES" -eq 0 ]; then
  QUALITY_SCORE=$((QUALITY_SCORE + 5))
  info "✓ No duplicate lines"
else
  info "✗ Found $HAS_DUPLICATES duplicate lines"
fi

if [ "$LINE_COUNT" -ge 50 ] && [ "$LINE_COUNT" -le 500 ]; then
  QUALITY_SCORE=$((QUALITY_SCORE + 5))
  info "✓ Appropriate file size ($LINE_COUNT lines)"
else
  info "⚠ File size: $LINE_COUNT lines"
fi

if [ "$AVG_LINE_LENGTH" -lt 120 ]; then
  QUALITY_SCORE=$((QUALITY_SCORE + 5))
  info "✓ Good line length (avg: $AVG_LINE_LENGTH chars)"
else
  info "✗ Long lines (avg: $AVG_LINE_LENGTH chars)"
fi

TOTAL_SCORE=$((TOTAL_SCORE + QUALITY_SCORE))
echo ""

# Generate evaluation report
header "═══════════════════════════════════════════════════════════"
header "  Evaluation Summary"
header "═══════════════════════════════════════════════════════════"
echo ""

GRADE="F"
if [ "$TOTAL_SCORE" -ge 90 ]; then
  GRADE="A"
elif [ "$TOTAL_SCORE" -ge 80 ]; then
  GRADE="B"
elif [ "$TOTAL_SCORE" -ge 70 ]; then
  GRADE="C"
elif [ "$TOTAL_SCORE" -ge 60 ]; then
  GRADE="D"
fi

echo "Total Score: $TOTAL_SCORE / $MAX_SCORE"
echo "Grade: $GRADE"
echo ""
echo "Breakdown:"
echo "  • Lint Validation:    $LINT_SCORE/20  $LINT_STATUS"
echo "  • Abstraction Level:  $ABSTRACTION_SCORE/20  $ABSTRACTION_STATUS"
echo "  • Drift Comparison:   $DRIFT_SCORE/30  $DRIFT_STATUS"
echo "  • Completeness:       $COMPLETENESS_SCORE/15"
echo "  • Code Quality:       $QUALITY_SCORE/15"
echo ""

# Save evaluation to JSON
cat > "$PROJECT_DIR/evaluation.json" << EOF
{
  "project": "$PROJECT_NAME",
  "timestamp": "$(date -Iseconds)",
  "total_score": $TOTAL_SCORE,
  "max_score": $MAX_SCORE,
  "grade": "$GRADE",
  "breakdown": {
    "lint_validation": {
      "score": $LINT_SCORE,
      "max": 20,
      "status": "$LINT_STATUS"
    },
    "abstraction_level": {
      "score": $ABSTRACTION_SCORE,
      "max": 20,
      "status": "$ABSTRACTION_STATUS",
      "component_count": $COMPONENT_COUNT,
      "system_count": $SYSTEM_COUNT,
      "container_count": $CONTAINER_COUNT
    },
    "drift_comparison": {
      "score": $DRIFT_SCORE,
      "max": 30,
      "status": "$DRIFT_STATUS",
      "violations": ${DRIFT_VIOLATIONS:-0}
    },
    "completeness": {
      "score": $COMPLETENESS_SCORE,
      "max": 15,
      "has_descriptions": $([ "$HAS_DESCRIPTION" -gt 0 ] && echo "true" || echo "false"),
      "has_technology": $([ "$HAS_TECHNOLOGY" -gt 0 ] && echo "true" || echo "false"),
      "has_relationships": $([ "$HAS_RELATIONSHIPS" -gt 0 ] && echo "true" || echo "false")
    },
    "code_quality": {
      "score": $QUALITY_SCORE,
      "max": 15,
      "line_count": $LINE_COUNT,
      "avg_line_length": $AVG_LINE_LENGTH,
      "duplicate_lines": $HAS_DUPLICATES
    }
  },
  "files": {
    "architecture": "architecture.sruja",
    "lint_log": "lint.log",
    "drift_log": "agent_drift.log"
  }
}
EOF

success "Evaluation saved to: $PROJECT_DIR/evaluation.json"

# Provide recommendations
echo ""
header "Recommendations:"
echo ""

if [ "$LINT_SCORE" -lt 20 ]; then
  echo "  • Fix lint errors in architecture.sruja"
fi

if [ "$ABSTRACTION_SCORE" -lt 20 ]; then
  if [ "$COMPONENT_COUNT" -lt 10 ]; then
    echo "  • Add more detail: break down high-level components"
  else
    echo "  • Reduce granularity: group related components into containers"
  fi
fi

if [ "$DRIFT_SCORE" -lt 30 ]; then
  echo "  • Review drift violations and adjust architecture to match code"
fi

if [ "$COMPLETENESS_SCORE" -lt 15 ]; then
  [ "$HAS_DESCRIPTION" -eq 0 ] && echo "  • Add descriptions to components"
  [ "$HAS_TECHNOLOGY" -eq 0 ] && echo "  • Add technology tags"
  [ "$HAS_RELATIONSHIPS" -eq 0 ] && echo "  • Add relationships between components"
fi

echo ""
success "Evaluation complete!"
