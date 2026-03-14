#!/bin/bash
# Batch testing with opencode automation

PROJECTS=(
  "express|https://github.com/expressjs/express"
  "react|https://github.com/facebook/react"
  "vue|https://github.com/vuejs/vue"
)

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="evaluation/results/automated_batch_$TIMESTAMP"

echo "═══════════════════════════════════════════════════════════"
echo "  Batch Automated Testing"
echo "═══════════════════════════════════════════════════════════"
echo "Projects: ${#PROJECTS[@]}"
echo "Results: $RESULTS_DIR"
echo ""

mkdir -p "$RESULTS_DIR"

for project_info in "${PROJECTS[@]}"; do
  IFS='|' read -r name url <<< "$project_info"
  
  echo "───────────────────────────────────────────────────────────"
  echo "Testing: $name"
  echo "───────────────────────────────────────────────────────────"
  
  REPO_DIR="/tmp/sruja_analysis_$name"
  
  # Clone
  if [ -d "$REPO_DIR" ]; then
    rm -rf "$REPO_DIR"
  fi
  git clone --depth 1 "$url" "$REPO_DIR"
  
  # Setup
  mkdir -p "$REPO_DIR/.sruja"
  cp -r skills/sruja-architecture "$REPO_DIR/.sruja/" 2>/dev/null || true
  
  # Create task instructions
  cat > "$REPO_DIR/AGENTS.md" << EOF
---
name: sruja-architecture-analysis
---

# Architecture Analysis: $name

Analyze this codebase and generate architecture.sruja.

## Steps

1. Explore the repository structure
2. Read README, package.json, configs
3. Identify systems, containers, components
4. Generate 10-30 top-level elements
5. Save to: architecture.sruja

## Focus

- Container-level architecture
- Technology tags
- Descriptions
- Relationships
EOF
  
  echo "✓ Repository prepared with the core Sruja skill: $REPO_DIR"
  echo ""
  echo "Next: Run opencode analysis"
  echo "  cd $REPO_DIR"
  echo "  opencode --prompt 'Read AGENTS.md and generate architecture.sruja'"
  echo ""
  
  # Save metadata
  mkdir -p "$RESULTS_DIR/$name"
  echo "{\"project\": \"$name\", \"url\": \"$url\", \"repo\": \"$REPO_DIR\"}" > "$RESULTS_DIR/$name/metadata.json"
  
  sleep 2
done

echo "═══════════════════════════════════════════════════════════"
echo "  Setup Complete"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Repositories ready for analysis in /tmp/sruja_analysis_*"
echo ""
echo "To run automated analysis:"
echo "  ./scripts/run_batch_analysis.sh"
