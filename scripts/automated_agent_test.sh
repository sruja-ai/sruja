#!/bin/bash
set -euo pipefail

PROJECT_NAME="${1:-}"
TIER="${2:-tier3}"

if [ -z "$PROJECT_NAME" ]; then
  echo "Usage: $0 PROJECT_NAME [tier]"
  echo "Example: $0 react tier3"
  exit 1
fi

echo "═══════════════════════════════════════════════════════════"
echo "  Automated AI Agent Testing: $PROJECT_NAME"
echo "═══════════════════════════════════════════════════════════"

# Step 1: Prepare test repository
echo "[1/6] Preparing test repository..."
./test_agent_skills.sh "$TIER" "$PROJECT_NAME"

# Find the results directory
RESULTS_DIR=$(find evaluation/results -name "agent_skill_test_*" -type d | sort | tail -1)
PROJECT_DIR="$RESULTS_DIR/$PROJECT_NAME"
REPO_DIR="/tmp/sruja_test_$PROJECT_NAME"

echo "Results directory: $PROJECT_DIR"
echo "Repository: $REPO_DIR"

# Step 2: Invoke AI agent via opencode task
echo ""
echo "[2/6] Invoking AI agent for architecture analysis..."

# Create a task file for the agent
TASK_FILE="/tmp/sruja_task_${PROJECT_NAME}.md"
cat > "$TASK_FILE" << EOF
# Architecture Analysis Task

You are analyzing the **$PROJECT_NAME** repository to generate a Sruja architecture DSL file.

## Repository Location
$REPO_DIR

## Your Task

1. **Explore the codebase**:
   - Read README.md, package.json, go.mod, or similar config files
   - Scan the directory structure (src/, lib/, internal/, pkg/)
   - Identify main systems, services, and components
   - Detect technologies (languages, frameworks, databases)

2. **Understand the architecture**:
   - Identify the architectural pattern (monolith, microservices, layered, etc.)
   - Group related functionality into logical containers
   - Note key dependencies and data flows

3. **Generate architecture.sruja**:
   - Create 10-30 top-level components/containers (NOT individual functions)
   - Include technology tags
   - Add descriptions for each element
   - Show relationships between components
   - Save to: $PROJECT_DIR/architecture.sruja

## Guidelines

- Focus on SYSTEM and CONTAINER level abstraction
- Do NOT create components for individual functions or files
- Group related modules into logical containers
- Use the Sruja DSL syntax (see examples in examples/ directory)
- Be specific about technologies (e.g., "PostgreSQL" not just "Database")

## Expected Output

Generate a complete architecture.sruja file and write it to:
$PROJECT_DIR/architecture.sruja

After generating, report:
- Number of systems, containers, components
- Technologies detected
- Architectural pattern identified
- Any issues or uncertainties
EOF

# Use opencode to run the analysis (this will be manual for now, but shows the pattern)
echo ""
echo "Task file created: $TASK_FILE"
echo ""
echo "To run automated analysis, use:"
echo "  opencode --task $TASK_FILE"
echo ""
echo "Or manually invoke with:"
echo "  # In another terminal or via API"
echo "  # The agent will analyze $REPO_DIR and generate architecture.sruja"
echo ""

# For now, let's check if architecture.sruja exists
if [ -f "$PROJECT_DIR/architecture.sruja" ]; then
  echo "[3/6] Found architecture.sruja"
  
  # Step 3: Validate
  echo "[4/6] Validating architecture..."
  sruja lint "$PROJECT_DIR/architecture.sruja"
  
  # Step 4: Check drift
  echo "[5/6] Checking drift..."
  sruja drift -r "$REPO_DIR" -a "$PROJECT_DIR/architecture.sruja" || true
  
  # Step 5: Evaluate
  echo "[6/6] Evaluating results..."
  ./scripts/evaluate_agent_output.sh "$PROJECT_NAME" "$RESULTS_DIR"
  
  # Show results
  echo ""
  echo "═══════════════════════════════════════════════════════════"
  echo "  Results"
  echo "═══════════════════════════════════════════════════════════"
  cat "$PROJECT_DIR/evaluation.json"
else
  echo "[3/6] No architecture.sruja found yet"
  echo "      Waiting for AI agent to generate it..."
  echo ""
  echo "Next steps:"
  echo "  1. Run AI analysis (see task file above)"
  echo "  2. Re-run this script to validate and evaluate"
fi
