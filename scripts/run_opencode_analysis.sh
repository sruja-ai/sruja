#!/bin/bash
# Simple wrapper to run opencode analysis in a repository

REPO_DIR="${1:-}"
OUTPUT_DIR="${2:-.}"

if [ -z "$REPO_DIR" ]; then
  echo "Usage: $0 REPO_DIR [OUTPUT_DIR]"
  echo "Example: $0 /tmp/my-repo ./results"
  exit 1
fi

if [ ! -d "$REPO_DIR" ]; then
  echo "Error: Repository not found: $REPO_DIR"
  exit 1
fi

echo "Repository: $REPO_DIR"
echo "Output: $OUTPUT_DIR/architecture.sruja"
echo ""
echo "Starting opencode analysis..."
echo ""

# Run opencode with the analysis task
# This will use the task tool internally
opencode << 'OPENERVATE_MSG'
You are analyzing the repository to generate architecture.sruja.

Repository: $REPO_DIR

Instructions:
1. Explore the codebase structure
2. Read key files (README, package.json, configs)
3. Identify systems, containers, components
4. Generate architecture.sruja with 10-30 elements
5. Save to: $OUTPUT_DIR/architecture.sruja

Focus on CONTAINER-level architecture, not individual files.
Include technologies, descriptions, and relationships.
OPENERVATE_MSG
