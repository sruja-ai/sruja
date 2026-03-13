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
RESULTS_DIR="${1:-$PROJECT_ROOT/evaluation/local-artifacts/testing}"

if [ ! -d "$RESULTS_DIR" ]; then
  error "Results directory not found: $RESULTS_DIR"
  exit 1
fi

header "═══════════════════════════════════════════════════════════"
header "  Aggregating Sruja Agent Skill Test Results"
header "═══════════════════════════════════════════════════════════"
echo ""

# Find all evaluation files
EVAL_FILES=$(find "$RESULTS_DIR" -name "evaluation.json" -type f 2>/dev/null | sort)

if [ -z "$EVAL_FILES" ]; then
  error "No evaluation files found"
  echo ""
  echo "Run agent analysis and evaluation first:"
  echo "  1. ./scripts/testing/agent_skill_benchmark.sh <tier>"
  echo "  2. Follow AGENT_INSTRUCTIONS.md for each project"
  echo "  3. ./scripts/evaluate_agent_output.sh <project_name>"
  exit 1
fi

info "Found evaluation files:"
echo "$EVAL_FILES" | while read -r file; do
  echo "  • $(dirname "$file" | xargs basename)"
done
echo ""

# Aggregate data
TOTAL_PROJECTS=0
SUCCESSFUL_PROJECTS=0
TOTAL_SCORE=0
AVG_SCORE=0

GRADES_A=0
GRADES_B=0
GRADES_C=0
GRADES_D=0
GRADES_F=0

# Create aggregated JSON
AGGREGATED_JSON="$RESULTS_DIR/AGGREGATED_RESULTS.json"
echo "[" > "$AGGREGATED_JSON"

FIRST=true
while IFS= read -r eval_file; do
  PROJECT_DIR=$(dirname "$eval_file")
  PROJECT_NAME=$(basename "$PROJECT_DIR")
  
  if [ ! -f "$eval_file" ]; then
    continue
  fi
  
  TOTAL_PROJECTS=$((TOTAL_PROJECTS + 1))
  
  # Extract data from evaluation.json
  SCORE=$(jq -r '.total_score // 0' "$eval_file")
  GRADE=$(jq -r '.grade // "F"' "$eval_file")
  
  TOTAL_SCORE=$((TOTAL_SCORE + SCORE))
  
  # Count grades
  case "$GRADE" in
    A) GRADES_A=$((GRADES_A + 1)) ;;
    B) GRADES_B=$((GRADES_B + 1)) ;;
    C) GRADES_C=$((GRADES_C + 1)) ;;
    D) GRADES_D=$((GRADES_D + 1)) ;;
    F) GRADES_F=$((GRADES_F + 1)) ;;
  esac
  
  if [ "$SCORE" -ge 60 ]; then
    SUCCESSFUL_PROJECTS=$((SUCCESSFUL_PROJECTS + 1))
  fi
  
  # Add to JSON array
  if [ "$FIRST" = true ]; then
    FIRST=false
  else
    echo "," >> "$AGGREGATED_JSON"
  fi
  
  cat "$eval_file" >> "$AGGREGATED_JSON"
  
done <<< "$EVAL_FILES"

echo "]" >> "$AGGREGATED_JSON"

# Calculate averages
if [ "$TOTAL_PROJECTS" -gt 0 ]; then
  AVG_SCORE=$((TOTAL_SCORE / TOTAL_PROJECTS))
fi

# Generate markdown report
REPORT="$RESULTS_DIR/FINAL_REPORT.md"
cat > "$REPORT" << EOF
# Sruja Agent Skill Test Results - Final Report

**Generated:** $(date)
**Results Directory:** $RESULTS_DIR

---

## Executive Summary

- **Total Projects Evaluated:** $TOTAL_PROJECTS
- **Successful Projects (≥60%):** $SUCCESSFUL_PROJECTS / $TOTAL_PROJECTS
- **Average Score:** $AVG_SCORE / 100
- **Success Rate:** $((SUCCESSFUL_PROJECTS * 100 / TOTAL_PROJECTS))%

---

## Grade Distribution

| Grade | Count | Percentage |
|-------|-------|------------|
| A (90-100) | $GRADES_A | $([ "$TOTAL_PROJECTS" -gt 0 ] && echo "$((GRADES_A * 100 / TOTAL_PROJECTS))" || echo "0")% |
| B (80-89) | $GRADES_B | $([ "$TOTAL_PROJECTS" -gt 0 ] && echo "$((GRADES_B * 100 / TOTAL_PROJECTS))" || echo "0")% |
| C (70-79) | $GRADES_C | $([ "$TOTAL_PROJECTS" -gt 0 ] && echo "$((GRADES_C * 100 / TOTAL_PROJECTS))" || echo "0")% |
| D (60-69) | $GRADES_D | $([ "$TOTAL_PROJECTS" -gt 0 ] && echo "$((GRADES_D * 100 / TOTAL_PROJECTS))" || echo "0")% |
| F (<60) | $GRADES_F | $([ "$TOTAL_PROJECTS" -gt 0 ] && echo "$((GRADES_F * 100 / TOTAL_PROJECTS))" || echo "0")% |

---

## Project Results

| Project | Score | Grade | Components | Violations | Status |
|---------|-------|-------|------------|------------|--------|
EOF

# Add project rows
while IFS= read -r eval_file; do
  PROJECT_DIR=$(dirname "$eval_file")
  PROJECT_NAME=$(basename "$PROJECT_DIR")
  
  if [ ! -f "$eval_file" ]; then
    continue
  fi
  
  SCORE=$(jq -r '.total_score // 0' "$eval_file")
  GRADE=$(jq -r '.grade // "F"' "$eval_file")
  COMPONENTS=$(jq -r '.breakdown.abstraction_level.component_count // 0' "$eval_file")
  VIOLATIONS=$(jq -r '.breakdown.drift_comparison.violations // 0' "$eval_file")
  
  if [ "$SCORE" -ge 80 ]; then
    STATUS="✅ Excellent"
  elif [ "$SCORE" -ge 60 ]; then
    STATUS="✅ Good"
  elif [ "$SCORE" -ge 40 ]; then
    STATUS="⚠️ Needs Work"
  else
    STATUS="❌ Poor"
  fi
  
  echo "| $PROJECT_NAME | $SCORE | $GRADE | $COMPONENTS | $VIOLATIONS | $STATUS |" >> "$REPORT"
  
done <<< "$EVAL_FILES"

cat >> "$REPORT" << EOF

---

## Detailed Analysis

### Lint Validation

EOF

# Analyze lint results
LINT_PASS=0
LINT_FAIL=0
while IFS= read -r eval_file; do
  if [ -f "$eval_file" ]; then
    LINT_SCORE=$(jq -r '.breakdown.lint_validation.score // 0' "$eval_file")
    if [ "$LINT_SCORE" -ge 20 ]; then
      LINT_PASS=$((LINT_PASS + 1))
    else
      LINT_FAIL=$((LINT_FAIL + 1))
    fi
  fi
done <<< "$EVAL_FILES"

cat >> "$REPORT" << EOF
- **Passed:** $LINT_PASS / $TOTAL_PROJECTS
- **Failed:** $LINT_FAIL / $TOTAL_PROJECTS

### Abstraction Level

EOF

# Analyze abstraction
TOO_GRANULAR=0
TOO_ABSTRACT=0
APPROPRIATE=0
while IFS= read -r eval_file; do
  if [ -f "$eval_file" ]; then
    ABSTRACTION_SCORE=$(jq -r '.breakdown.abstraction_level.score // 0' "$eval_file")
    if [ "$ABSTRACTION_SCORE" -ge 20 ]; then
      APPROPRIATE=$((APPROPRIATE + 1))
    elif [ "$ABSTRACTION_SCORE" -ge 10 ]; then
      TOO_ABSTRACT=$((TOO_ABSTRACT + 1))
    else
      TOO_GRANULAR=$((TOO_GRANULAR + 1))
    fi
  fi
done <<< "$EVAL_FILES"

cat >> "$REPORT" << EOF
- **Appropriate (10-50 components):** $APPROPRIATE
- **Too Abstract (<10 components):** $TOO_ABSTRACT
- **Too Granular (>50 components):** $TOO_GRANULAR

### Drift Comparison

EOF

# Analyze drift
NO_DRIFT=0
MINOR_DRIFT=0
MAJOR_DRIFT=0
while IFS= read -r eval_file; do
  if [ -f "$eval_file" ]; then
    DRIFT_SCORE=$(jq -r '.breakdown.drift_comparison.score // 0' "$eval_file")
    if [ "$DRIFT_SCORE" -ge 30 ]; then
      NO_DRIFT=$((NO_DRIFT + 1))
    elif [ "$DRIFT_SCORE" -ge 20 ]; then
      MINOR_DRIFT=$((MINOR_DRIFT + 1))
    else
      MAJOR_DRIFT=$((MAJOR_DRIFT + 1))
    fi
  fi
done <<< "$EVAL_FILES"

cat >> "$REPORT" << EOF
- **No Violations:** $NO_DRIFT
- **Minor Violations (1-3):** $MINOR_DRIFT
- **Major Violations (>3):** $MAJOR_DRIFT

---

## Key Findings

### Strengths ✅

EOF

# Add strengths
if [ "$LINT_PASS" -gt $((TOTAL_PROJECTS * 70 / 100)) ]; then
  echo "- High lint pass rate ($LINT_PASS / $TOTAL_PROJECTS)" >> "$REPORT"
fi

if [ "$APPROPRIATE" -gt $((TOTAL_PROJECTS * 70 / 100)) ]; then
  echo "- Appropriate abstraction levels in most projects" >> "$REPORT"
fi

if [ "$NO_DRIFT" -gt $((TOTAL_PROJECTS * 50 / 100)) ]; then
  echo "- Good alignment between generated architecture and code" >> "$REPORT"
fi

cat >> "$REPORT" << EOF

### Issues Identified 🔍

EOF

# Add issues
if [ "$LINT_FAIL" -gt 0 ]; then
  echo "- $LINT_FAIL project(s) failed lint validation" >> "$REPORT"
fi

if [ "$TOO_GRANULAR" -gt 0 ]; then
  echo "- $TOO_GRANULAR project(s) too granular (need higher abstraction)" >> "$REPORT"
fi

if [ "$MAJOR_DRIFT" -gt 0 ]; then
  echo "- $MAJOR_DRIFT project(s) have significant drift violations" >> "$REPORT"
fi

cat >> "$REPORT" << EOF

---

## Recommendations

### Immediate Actions

1. **Fix Lint Errors:** Review and fix lint failures in failing projects
2. **Adjust Abstraction:** Refine granularity in projects with inappropriate component counts
3. **Reduce Drift:** Update architectures to better match actual code structure

### Future Improvements

1. **Better Pattern Recognition:** Improve detection of common architectural patterns
2. **Technology Detection:** Enhance automatic technology identification
3. **Relationship Inference:** Better inference of component relationships from code
4. **Abstraction Heuristics:** Smarter defaults for component grouping

---

## Individual Project Reports

EOF

# Add links to individual reports
while IFS= read -r eval_file; do
  PROJECT_DIR=$(dirname "$eval_file")
  PROJECT_NAME=$(basename "$PROJECT_DIR")
  echo "- [$PROJECT_NAME](./$PROJECT_NAME/evaluation.json)" >> "$REPORT"
done <<< "$EVAL_FILES"

cat >> "$REPORT" << EOF

---

## Files Generated

- **AGGREGATED_RESULTS.json** - All evaluation data in JSON format
- **FINAL_REPORT.md** - This comprehensive report
- **PROJECT_NAME/evaluation.json** - Individual project evaluations
- **PROJECT_NAME/architecture.sruja** - Generated architecture files
- **PROJECT_NAME/AGENT_INSTRUCTIONS.md** - Agent analysis instructions

---

**Test Framework Version:** 1.0
**Generated by:** Sruja Agent Skill Testing Framework
EOF

success "Results aggregated!"
echo ""
echo "Summary:"
echo "  • Total Projects: $TOTAL_PROJECTS"
echo "  • Successful: $SUCCESSFUL_PROJECTS / $TOTAL_PROJECTS"
echo "  • Average Score: $AVG_SCORE / 100"
echo "  • Success Rate: $((SUCCESSFUL_PROJECTS * 100 / TOTAL_PROJECTS))%"
echo ""
echo "Grade Distribution:"
echo "  • A: $GRADES_A"
echo "  • B: $GRADES_B"
echo "  • C: $GRADES_C"
echo "  • D: $GRADES_D"
echo "  • F: $GRADES_F"
echo ""
echo "Reports saved to:"
echo "  • $AGGREGATED_JSON"
echo "  • $REPORT"
