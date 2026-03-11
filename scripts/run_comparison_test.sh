#!/bin/bash
# Compare architecture quality: LLM generates in Mermaid (no Sruja) vs Sruja (with skill).
# Baseline = popular format (Mermaid), no Sruja. Enhanced = Sruja DSL with skill.
# Goal: compare which output captures system details better (quality of architecture).

PROJECT_NAME="${1:-express}"
PROJECT_URL="${2:-https://github.com/expressjs/express}"

echo "═══════════════════════════════════════════════════════════"
echo "  Architecture quality: Mermaid (no Sruja) vs Sruja (skill)"
echo "═══════════════════════════════════════════════════════════"
echo "Project: $PROJECT_NAME"
echo "URL: $PROJECT_URL"
echo ""

BASELINE_DIR="/tmp/sruja_comparison_${PROJECT_NAME}_baseline"
ENHANCED_DIR="/tmp/sruja_comparison_${PROJECT_NAME}_enhanced"

# Clone repositories
echo "[1/6] Cloning repositories..."
if [ -d "$BASELINE_DIR" ]; then rm -rf "$BASELINE_DIR"; fi
if [ -d "$ENHANCED_DIR" ]; then rm -rf "$ENHANCED_DIR"; fi

git clone --depth 1 "$PROJECT_URL" "$BASELINE_DIR" 2>&1 | grep -E "(Cloning|done)" || true
git clone --depth 1 "$PROJECT_URL" "$ENHANCED_DIR" 2>&1 | grep -E "(Cloning|done)" || true
echo "✓ Repositories cloned"
echo ""

# Baseline: ask for Mermaid (no Sruja — Sruja is not popular; use a format the LLM knows well)
echo "[2/6] Running BASELINE (architecture in Mermaid, no Sruja skill)..."
echo "This will run Cursor CLI agent..."
echo ""

if ! command -v agent >/dev/null 2>&1; then
  echo "❌ Cursor CLI agent not found in PATH (expected 'agent')."
  echo "   See: evaluation/real-world-test/LOCAL_CURSOR_CLI_TESTING.md"
  exit 1
fi

(cd "$BASELINE_DIR" && agent --trust -p "Analyze the repository at $BASELINE_DIR and generate an architecture diagram in Mermaid format.

Instructions:
1. Explore the codebase (README, package.json or equivalent, main modules).
2. Identify the main systems, components, and their relationships.
3. Generate a Mermaid diagram that captures the architecture (e.g. flowchart or C4-style subgraphs: systems, containers, components, and how they connect).
4. Save the Mermaid code to: $BASELINE_DIR/architecture.mmd
   (If you use a markdown file with a mermaid code block, save as $BASELINE_DIR/architecture.mmd with only the mermaid content, or keep the file path as given.)

Focus on capturing the system accurately: main entry points, core modules, key dependencies, and data/control flow. Use a format the LLM is familiar with (Mermaid) — no Sruja or other niche DSLs.

IMPORTANT: Return a short summary of what you captured (systems, components, relationships).")

echo ""

# Enhanced: Sruja with skill
echo "[3/6] Running ENHANCED (architecture in Sruja DSL with Sruja skill)..."
echo "This will run Cursor CLI agent..."
echo ""

(cd "$ENHANCED_DIR" && agent --trust -p "Analyze the repository at $ENHANCED_DIR and generate architecture in Sruja DSL using sruja-architecture guidelines.

Guidelines:
- Target: 10-30 components (NOT 100+)
- Focus on CONTAINER-level architecture (C4: system, container, component)
- Include technology tags and descriptions for each element
- Define relationships with clear labels (e.g. \"uses\", \"calls\", \"reads from\")
- Use standard Sruja DSL: Id = kind \"Label\" { description \"...\" technology \"...\" }, relationship A -> B \"label\"

Instructions:
1. Explore the codebase
2. Identify systems, containers, components
3. Generate architecture.sruja with proper abstraction
4. Save to: $ENHANCED_DIR/architecture.sruja

IMPORTANT: Return a short summary of what you captured.")

echo ""

# Locate baseline file (may be .mmd or .md with mermaid)
BASELINE_FILE=""
for f in "$BASELINE_DIR/architecture.mmd" "$BASELINE_DIR/architecture.md" "$BASELINE_DIR/architecture.mermaid"; do
  if [ -f "$f" ]; then BASELINE_FILE="$f"; break; fi
done
if [ -z "$BASELINE_FILE" ]; then
  echo "⚠ Baseline output not found (expected architecture.mmd or architecture.md in $BASELINE_DIR)"
  BASELINE_FILE="$BASELINE_DIR/architecture.mmd"
fi

ENHANCED_FILE="$ENHANCED_DIR/architecture.sruja"

# Compare results (quality-oriented metrics)
echo "[4/6] Comparing architecture quality..."
echo ""

echo "BASELINE (Mermaid — no Sruja):"
if [ -f "$BASELINE_FILE" ]; then
  BLINES=$(wc -l < "$BASELINE_FILE" 2>/dev/null || echo "0")
  echo "  Lines: $BLINES"
  # Mermaid heuristics: subgraphs, links
  BSUB=$(grep -c -e 'subgraph' "$BASELINE_FILE" 2>/dev/null || echo "0")
  BLINKS=$(grep -cE '-->|---' "$BASELINE_FILE" 2>/dev/null || echo "0")
  echo "  Subgraphs: $BSUB"
  echo "  Links (--> or ---): $BLINKS"
else
  echo "  (file not found)"
fi
echo ""

echo "ENHANCED (Sruja with skill):"
if [ -f "$ENHANCED_FILE" ]; then
  ELINES=$(wc -l < "$ENHANCED_FILE" 2>/dev/null || echo "0")
  ESYS=$(grep -c 'system' "$ENHANCED_FILE" 2>/dev/null || echo "0")
  ECON=$(grep -c 'container' "$ENHANCED_FILE" 2>/dev/null || echo "0")
  EREL=$(grep -c -e '->' "$ENHANCED_FILE" 2>/dev/null || echo "0")
  EDESC=$(grep -c 'description' "$ENHANCED_FILE" 2>/dev/null || echo "0")
  echo "  Lines: $ELINES"
  echo "  Systems: $ESYS  Containers: $ECON  Relationships: $EREL  Descriptions: $EDESC"
else
  echo "  (file not found)"
fi
echo ""

# Validate Sruja only; capture lint pass/fail for records
echo "[5/6] Validating Sruja output..."
SRUJA_BIN=""
if command -v sruja >/dev/null 2>&1; then
  SRUJA_BIN="sruja"
elif [ -x "$(cd "$(dirname "$0")/.." && pwd)/target/release/sruja" ]; then
  SRUJA_BIN="$(cd "$(dirname "$0")/.." && pwd)/target/release/sruja"
fi

SRUJA_LINT_STATUS="(skipped: file missing or sruja not found)"
if [ -f "$ENHANCED_FILE" ] && [ -n "$SRUJA_BIN" ]; then
  if "$SRUJA_BIN" lint "$ENHANCED_FILE" >/dev/null 2>&1; then
    SRUJA_LINT_STATUS="pass"
    echo "  Sruja lint: pass"
  else
    SRUJA_LINT_STATUS="fail"
    echo "  Sruja lint: fail"
    "$SRUJA_BIN" lint "$ENHANCED_FILE" 2>&1 | head -5
  fi
else
  echo "  $SRUJA_LINT_STATUS"
fi
echo ""

# Save comparison
TIMESTAMP=$(date +%Y%m%d_%H%S)
COMPARISON_DIR="evaluation/results/comparison_${PROJECT_NAME}_$TIMESTAMP"
mkdir -p "$COMPARISON_DIR"

# Copy baseline (preserve extension)
if [ -f "$BASELINE_FILE" ]; then
  cp "$BASELINE_FILE" "$COMPARISON_DIR/baseline.${BASELINE_FILE##*.}"
fi
if [ -f "$ENHANCED_FILE" ]; then
  cp "$ENHANCED_FILE" "$COMPARISON_DIR/sruja.sruja"
fi

# Quality comparison note
cat > "$COMPARISON_DIR/QUALITY_COMPARISON.md" << 'QUALITY_EOF'
# Architecture quality: Mermaid (no Sruja) vs Sruja (with skill)

Compare **which output captures the system details better**.

## What to compare

| Criterion | Baseline (Mermaid) | Sruja (with skill) |
|-----------|--------------------|--------------------|
| **Completeness** | Are main systems, components, and entry points captured? | Same. |
| **Relationships** | Are key dependencies and data/control flows shown? | Same. |
| **Clarity** | Is the structure easy to follow? | Same. |
| **Technologies** | Are tech stack and runtime noted? | Sruja encourages `technology` tags. |
| **Descriptions** | Are component roles explained? | Sruja encourages `description` on every element. |
| **Level of detail** | Appropriate (not too flat, not too noisy)? | Sruja skill targets 10–30 components. |

## Files

- **baseline.mmd** (or .md) — Architecture generated in Mermaid, no Sruja skill.
- **sruja.sruja** — Architecture generated in Sruja DSL with the Sruja skill.

## Verdict

After reviewing both, which captures the system better? Note your assessment (e.g. "Sruja: better structure and descriptions" or "Mermaid: simpler but missed X"). Run `./scripts/summarize_comparison.sh` on this dir for metric summary.
QUALITY_EOF

# Add run-specific metrics and lint status to README
cat > "$COMPARISON_DIR/README.md" << EOF
# Architecture quality comparison: $PROJECT_NAME

**Date:** $(date)
**URL:** $PROJECT_URL

## Design

- **Baseline:** LLM asked to generate architecture in **Mermaid** (no Sruja; popular format).
- **Enhanced:** LLM asked to generate architecture in **Sruja DSL** (with Sruja skill).

**Goal:** Compare **quality of architecture** — which output captures system details better?

## Metrics (this run)

| Metric | Baseline (Mermaid) | Sruja (with skill) |
|--------|--------------------|--------------------|
| Lines | $([ -f "$BASELINE_FILE" ] && wc -l < "$BASELINE_FILE" || echo "N/A") | $([ -f "$ENHANCED_FILE" ] && wc -l < "$ENHANCED_FILE" || echo "N/A") |
| Subgraphs / Systems | $([ -f "$BASELINE_FILE" ] && grep -c -e 'subgraph' "$BASELINE_FILE" 2>/dev/null || echo "?") | $([ -f "$ENHANCED_FILE" ] && grep -c 'system' "$ENHANCED_FILE" 2>/dev/null || echo "?") |
| Containers / structure | (Mermaid varies) | $([ -f "$ENHANCED_FILE" ] && grep -c 'container' "$ENHANCED_FILE" 2>/dev/null || echo "?") |
| Relationships/links | $([ -f "$BASELINE_FILE" ] && grep -cE -- '-->|---|==>|-\.->' "$BASELINE_FILE" 2>/dev/null || echo "?") | $([ -f "$ENHANCED_FILE" ] && grep -c -e '->' "$ENHANCED_FILE" 2>/dev/null || echo "?") |
| Descriptions | (Mermaid: in labels) | $([ -f "$ENHANCED_FILE" ] && grep -c 'description' "$ENHANCED_FILE" 2>/dev/null || echo "?") |
| **Sruja lint** | N/A | **$SRUJA_LINT_STATUS** |

## Files

- baseline.* — Mermaid diagram (no Sruja)
- sruja.sruja — Sruja DSL (with skill)
- QUALITY_COMPARISON.md — Rubric and how to judge which captures the system better
EOF

# Record lint status for scripts/CI
echo "$SRUJA_LINT_STATUS" > "$COMPARISON_DIR/LINT_STATUS.txt"

echo "[6/6] Results saved to: $COMPARISON_DIR"
echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  Comparison complete"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo "  1. Review quality: diff which captures system details better (see QUALITY_COMPARISON.md)"
echo "  2. Baseline: cat $COMPARISON_DIR/baseline.*"
echo "  3. Sruja:     cat $COMPARISON_DIR/sruja.sruja"
echo "  4. Summary:   ./scripts/summarize_comparison.sh $COMPARISON_DIR"
echo ""
