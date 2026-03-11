#!/bin/bash
set -euo pipefail

PROJECT_NAME="${1:-}"
PROJECT_URL="${2:-}"

if [ -z "$PROJECT_NAME" ] || [ -z "$PROJECT_URL" ]; then
  echo "Usage: $0 PROJECT_NAME REPO_URL"
  echo "Example: $0 express https://github.com/expressjs/express"
  exit 1
fi

REPO_DIR="/tmp/sruja_analysis_$PROJECT_NAME"
RESULTS_DIR="evaluation/results/agent_automated_$(date +%Y%m%d_%H%M%S)"
PROJECT_DIR="$RESULTS_DIR/$PROJECT_NAME"

echo "═══════════════════════════════════════════════════════════"
echo "  Sruja Automated Agent Testing"
echo "═══════════════════════════════════════════════════════════"
echo "Project: $PROJECT_NAME"
echo "URL: $PROJECT_URL"
echo ""

# Step 1: Clone repository
echo "[1/5] Cloning repository..."
if [ -d "$REPO_DIR" ]; then
  echo "Removing existing clone..."
  rm -rf "$REPO_DIR"
fi
git clone "$PROJECT_URL" "$REPO_DIR" --depth 1
echo "✓ Cloned to: $REPO_DIR"

# Step 2: Setup sruja in the repo
echo ""
echo "[2/5] Setting up sruja in repository..."
cd "$REPO_DIR"

# Initialize sruja if not already
if [ ! -f ".sruja/config.json" ]; then
  mkdir -p .sruja
  cat > .sruja/config.json << EOF
{
  "version": "1.0",
  "project": "$PROJECT_NAME"
}
EOF
  echo "✓ Created .sruja/config.json"
fi

# Copy sruja skills to the repo
if [ -d "$OLDPWD/skills" ]; then
  mkdir -p .sruja/skills
  cp -r "$OLDPWD/skills/sruja-architecture" .sruja/skills/ 2>/dev/null || true
  cp -r "$OLDPWD/skills/sruja-architecture-agent" .sruja/skills/ 2>/dev/null || true
  echo "✓ Installed sruja skills"
fi

# Create AGENTS.md in the repo
cat > AGENTS.md << 'EOF'
---
name: sruja-analysis
description: Generate architecture.sruja from codebase analysis
---

# Architecture Analysis Task

Analyze this codebase and generate a `architecture.sruja` file.

## Instructions

1. **Explore the repository structure**
   - Use `glob` and `read` tools to understand the codebase
   - Find README, package.json, go.mod, Cargo.toml, or similar
   - Scan main directories (src/, lib/, internal/, pkg/, app/)

2. **Identify architectural elements**
   - Systems: What major systems exist?
   - Containers: What deployable units/services?
   - Components: What logical modules/components?
   - Data stores: What databases, caches, queues?
   - Technologies: What languages, frameworks, tools?

3. **Generate architecture.sruja**
   - Target: 10-30 top-level elements
   - Focus on CONTAINERS and major COMPONENTS
   - Do NOT document individual functions or files
   - Add technology tags and descriptions
   - Show relationships with arrows (->)

4. **Save the file**
   - Write to: `architecture.sruja` in the repo root

## Quality Criteria

- ✅ 10-30 components (not 100+)
- ✅ Technology tags present
- ✅ Descriptions for each element
- ✅ Relationships defined
- ✅ Matches actual codebase structure

## After Generation

Report:
- Number of systems, containers, components, datastores
- Technologies detected
- Architectural pattern (monolith, microservices, layered, etc.)
- Any uncertainties or questions
EOF

echo "✓ Created AGENTS.md"

# Create results directory
mkdir -p "$OLDPWD/$PROJECT_DIR"
echo "✓ Results will be saved to: $PROJECT_DIR"

# Step 3: Run opencode analysis
echo ""
echo "[3/5] Running AI agent analysis..."
echo "Working directory: $REPO_DIR"
echo ""

# Create the task for opencode
cd "$OLDPWD"
task description="Analyze $PROJECT_NAME architecture" prompt="You are in the repository at $REPO_DIR.

Your task is to analyze this codebase and generate architecture.sruja.

Steps:
1. Change to the repository directory: $REPO_DIR
2. Read the AGENTS.md file there for detailed instructions
3. Explore the codebase using glob and read tools
4. Generate architecture.sruja with 10-30 top-level components
5. Save it to: $REPO_DIR/architecture.sruja

Guidelines:
- Focus on CONTAINER-level architecture (not individual files)
- Include technology tags
- Add descriptions
- Show relationships

After generating, report:
- Number of elements created
- Technologies found
- Architectural pattern detected" subagent_type="general"