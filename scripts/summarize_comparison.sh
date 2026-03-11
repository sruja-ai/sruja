#!/usr/bin/env bash
# Summarize an architecture quality comparison: Mermaid (no Sruja) vs Sruja (with skill).
# Extracts metrics so you can compare which captures system details better.
#
# Usage:
#   ./scripts/summarize_comparison.sh evaluation/results/comparison_express_20260309_120000
#
# Supports:
#   - baseline.mmd or baseline.md + sruja.sruja (Mermaid vs Sruja quality comparison)
#   - baseline.sruja + enhanced.sruja (legacy: both Sruja)
#   - baseline/architecture.sruja + enhanced/architecture.sruja (legacy)
set -e

COMPARISON_DIR="${1:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SRUJA="${PROJECT_ROOT}/target/release/sruja"

if [ -z "$COMPARISON_DIR" ] || [ ! -d "$COMPARISON_DIR" ]; then
  echo "Usage: $0 <comparison_dir>"
  echo ""
  echo "Example: $0 evaluation/results/comparison_express_20260309_120000"
  echo ""
  echo "Looks for: baseline.* + sruja.sruja (Mermaid vs Sruja), or baseline.sruja + enhanced.sruja (legacy)."
  exit 1
fi

# Resolve files: prefer Mermaid vs Sruja layout, then legacy
BASELINE_FILE=""
ENHANCED_FILE=""
LAYOUT=""

if [ -f "$COMPARISON_DIR/sruja.sruja" ]; then
  for b in "$COMPARISON_DIR/baseline.mmd" "$COMPARISON_DIR/baseline.md" "$COMPARISON_DIR/baseline.mermaid"; do
    if [ -f "$b" ]; then
      BASELINE_FILE="$b"
      ENHANCED_FILE="$COMPARISON_DIR/sruja.sruja"
      LAYOUT="mermaid_vs_sruja"
      break
    fi
  done
fi

if [ -z "$LAYOUT" ] && [ -f "$COMPARISON_DIR/baseline.sruja" ] && [ -f "$COMPARISON_DIR/enhanced.sruja" ]; then
  BASELINE_FILE="$COMPARISON_DIR/baseline.sruja"
  ENHANCED_FILE="$COMPARISON_DIR/enhanced.sruja"
  LAYOUT="sruja_vs_sruja"
fi

if [ -z "$LAYOUT" ] && [ -f "$COMPARISON_DIR/baseline/architecture.sruja" ] && [ -f "$COMPARISON_DIR/enhanced/architecture.sruja" ]; then
  BASELINE_FILE="$COMPARISON_DIR/baseline/architecture.sruja"
  ENHANCED_FILE="$COMPARISON_DIR/enhanced/architecture.sruja"
  LAYOUT="sruja_vs_sruja"
fi

if [ -z "$BASELINE_FILE" ] || [ -z "$ENHANCED_FILE" ]; then
  echo "No comparison files found in $COMPARISON_DIR"
  echo "Expected: baseline.mmd (or .md) + sruja.sruja, or baseline.sruja + enhanced.sruja"
  exit 1
fi

# Stats for Mermaid file
# Links: count lines containing Mermaid edge arrows (--> --- ==> -.-> etc.); approximate.
stats_mermaid() {
  local f="$1"
  local link_count
  link_count=$(grep -cE -- '-->|---|==>|-\.->' "$f" 2>/dev/null) || link_count=0
  printf "  Lines: %s\n" "$(wc -l < "$f" 2>/dev/null || echo 0)"
  printf "  Subgraphs: %s\n" "$(grep -c -e 'subgraph' "$f" 2>/dev/null || echo 0)"
  printf "  Links (approx, arrow lines): %s\n" "$link_count"
  echo "  Lint: N/A (Mermaid)"
}

# Stats for Sruja file
stats_sruja() {
  local f="$1"
  printf "  Lines: %s\n" "$(wc -l < "$f" 2>/dev/null || echo 0)"
  printf "  Systems: %s\n" "$(grep -c 'system' "$f" 2>/dev/null || echo 0)"
  printf "  Containers: %s\n" "$(grep -c 'container' "$f" 2>/dev/null || echo 0)"
  printf "  Components: %s\n" "$(grep -c 'component' "$f" 2>/dev/null || echo 0)"
  rels=$(grep -c -e '->' "$f" 2>/dev/null) || rels=0
  printf "  Relationships (->): %s\n" "$rels"
  printf "  Descriptions: %s\n" "$(grep -c 'description' "$f" 2>/dev/null || echo 0)"
}

lint_result() {
  local f="$1"
  if [ -x "$SRUJA" ]; then
    if "$SRUJA" lint "$f" >/dev/null 2>&1; then
      echo "  Lint: pass"
    else
      echo "  Lint: fail"
    fi
  else
    echo "  Lint: (sruja not built)"
  fi
}

echo "============================================================"
echo "  Comparison summary: $(basename "$COMPARISON_DIR") ($LAYOUT)"
echo "============================================================"
echo ""

if [ "$LAYOUT" = "mermaid_vs_sruja" ]; then
  echo "Baseline (Mermaid — no Sruja skill):"
  stats_mermaid "$BASELINE_FILE"
  echo ""
  echo "Sruja (with Sruja skill):"
  stats_sruja "$ENHANCED_FILE"
  lint_result "$ENHANCED_FILE"
else
  echo "Baseline (no Sruja guidelines):"
  stats_sruja "$BASELINE_FILE"
  lint_result "$BASELINE_FILE"
  echo ""
  echo "Enhanced (with Sruja guidelines):"
  stats_sruja "$ENHANCED_FILE"
  lint_result "$ENHANCED_FILE"
fi

echo ""
echo "============================================================"

# Write summary file
SUMMARY_FILE="$COMPARISON_DIR/HELPFUL_SUMMARY.md"
if [ "$LAYOUT" = "mermaid_vs_sruja" ]; then
  BL=$(wc -l < "$BASELINE_FILE" 2>/dev/null || echo 0)
  BSUB=$(grep -c -e 'subgraph' "$BASELINE_FILE" 2>/dev/null || echo 0)
  BLINK=$(grep -cE -- '-->|---|==>|-\.->' "$BASELINE_FILE" 2>/dev/null || echo 0)
  EL=$(wc -l < "$ENHANCED_FILE" 2>/dev/null || echo 0)
  ESYS=$(grep -c 'system' "$ENHANCED_FILE" 2>/dev/null || echo 0)
  ECON=$(grep -c 'container' "$ENHANCED_FILE" 2>/dev/null || echo 0)
  EREL=$(grep -c -e '->' "$ENHANCED_FILE" 2>/dev/null || echo 0)
  EDESC=$(grep -c 'description' "$ENHANCED_FILE" 2>/dev/null || echo 0)
  "$SRUJA" lint "$ENHANCED_FILE" >/dev/null 2>&1 && E_LINT="pass" || E_LINT="fail"
  {
    echo "# Architecture quality: Mermaid vs Sruja — summary"
    echo ""
    echo "Compare **which captures system details better** (see QUALITY_COMPARISON.md)."
    echo ""
    echo "| Metric | Baseline (Mermaid) | Sruja (with skill) |"
    echo "|--------|--------------------|--------------------|"
    echo "| Lines | $BL | $EL |"
    echo "| Subgraphs / Systems | $BSUB | $ESYS |"
    echo "| Containers | (varies) | $ECON |"
    echo "| Relationships/links (Mermaid: approx) | $BLINK | $EREL |"
    echo "| Descriptions | (in labels) | $EDESC |"
    echo "| Lint | N/A | $E_LINT |"
    echo ""
    echo "See \`IS_SRUJA_HELPFUL.md\` in evaluation/real-world-test/run_results/."
  } > "$SUMMARY_FILE"
else
  BL=$(wc -l < "$BASELINE_FILE" 2>/dev/null || echo 0)
  EL=$(wc -l < "$ENHANCED_FILE" 2>/dev/null || echo 0)
  BS=$(grep -c 'system' "$BASELINE_FILE" 2>/dev/null || echo 0)
  ES=$(grep -c 'system' "$ENHANCED_FILE" 2>/dev/null || echo 0)
  BC=$(grep -c 'container' "$BASELINE_FILE" 2>/dev/null || echo 0)
  EC=$(grep -c 'container' "$ENHANCED_FILE" 2>/dev/null || echo 0)
  br=$(grep -c -e '->' "$BASELINE_FILE" 2>/dev/null) || br=0
  er=$(grep -c -e '->' "$ENHANCED_FILE" 2>/dev/null) || er=0
  "$SRUJA" lint "$BASELINE_FILE" >/dev/null 2>&1 && B_LINT="pass" || B_LINT="fail"
  "$SRUJA" lint "$ENHANCED_FILE" >/dev/null 2>&1 && E_LINT="pass" || E_LINT="fail"
  {
    echo "# With vs without Sruja — summary"
    echo ""
    echo "| Metric | Baseline | Enhanced |"
    echo "|--------|----------|----------|"
    echo "| Lines | $BL | $EL |"
    echo "| Systems | $BS | $ES |"
    echo "| Containers | $BC | $EC |"
    echo "| Relationships | $br | $er |"
    echo "| Lint | $B_LINT | $E_LINT |"
    echo ""
    echo "See \`IS_SRUJA_HELPFUL.md\` in evaluation/real-world-test/run_results/."
  } > "$SUMMARY_FILE"
fi

echo "Summary written to: $SUMMARY_FILE"
