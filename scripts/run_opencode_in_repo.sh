#!/bin/bash
# Run automated analysis using task tool

PROJECT_NAME="${1:-}"
REPO_DIR="${2:-}"

if [ -z "$PROJECT_NAME" ]; then
  echo "Usage: $0 PROJECT_NAME [REPO_DIR]"
  echo "Example: $0 express /tmp/sruja_analysis_express"
  exit 1
fi

if [ -z "$REPO_DIR" ]; then
  REPO_DIR="/tmp/sruja_analysis_$PROJECT_NAME"
fi

if [ ! -d "$REPO_DIR" ]; then
  echo "Error: Repository not found: $REPO_DIR"
  echo "Clone it first or run: ./scripts/batch_setup.sh"
  exit 1
fi

echo "═══════════════════════════════════════════════════════════"
echo "  Running AI Analysis: $PROJECT_NAME"
echo "═══════════════════════════════════════════════════════════"
echo "Repository: $REPO_DIR"
echo ""

# Check if opencode is available
if ! command -v opencode &> /dev/null; then
  echo "Error: opencode CLI not found"
  echo ""
  echo "Manual analysis steps:"
  echo "  1. cd $REPO_DIR"
  echo "  2. Read AGENTS.md"
  echo "  3. Explore codebase"
  echo "  4. Generate architecture.sruja"
  echo ""
  echo "Or use the task tool from another AI:"
  echo "  task description='Analyze $PROJECT_NAME' prompt='...' subagent_type=general"
  exit 1
fi

# Run opencode with the analysis task
cd "$REPO_DIR"

opencode << EOF
You are analyzing the $PROJECT_NAME repository to generate architecture.sruja.

Repository: $REPO_DIR

Instructions:
1. Read the AGENTS.md file in this directory
2. Explore the codebase structure using glob and read tools
3. Identify:
   - Systems and subsystems
   - Containers (deployable units)
   - Components (logical modules)
   - Datastores (databases, caches)
   - Technologies used

4. Generate architecture.sruja with:
   - 10-30 top-level elements (NOT 100+)
   - Technology tags for each element
   - Descriptions explaining purpose
   - Relationships between components

5. Save the file to: $REPO_DIR/architecture.sruja

Guidelines:
- Focus on CONTAINER-level architecture
- Group related functionality
- Be specific about technologies
- Show data flows with arrows (->)

After generating, report:
- Number of systems, containers, components, datastores
- Technologies detected
- Architectural pattern identified
- File saved successfully
EOF

echo ""
echo "═══════════════════════════════════════════════════════════"

if [ -f "$REPO_DIR/architecture.sruja" ]; then
  echo "✓ architecture.sruja generated!"
  echo ""
  echo "Next steps:"
  echo "  # Validate"
  echo "  sruja lint $REPO_DIR/architecture.sruja"
  echo ""
  echo "  # Check drift"
  echo "  sruja drift -r $REPO_DIR -a $REPO_DIR/architecture.sruja"
  echo ""
  echo "  # Evaluate"
  echo "  ./scripts/evaluate_agent_output.sh $PROJECT_NAME <results_dir>"
else
  echo "✗ architecture.sruja not found"
  echo "Check the opencode output above for errors"
fi
