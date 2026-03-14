#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/_common.sh"

ROOT="$(project_root)"
SRUJA_BIN="$(ensure_sruja_cli "$ROOT")"

usage() {
  cat <<'EOF'
agent_skill_benchmark.sh

Maintainer harness for running Sruja CLI analysis across tiers of repos and
emitting structured artifacts for evaluating the agent/skill workflows.

Usage:
  ./scripts/testing/agent_skill_benchmark.sh [tier] [project_filter]

Args:
  tier            all|tier1|tier2|tier3 (default: all)
  project_filter  optional substring to select a single project (e.g. "grafana")

Artifacts:
  evaluation/local-artifacts/testing/agent_skill_benchmark_<timestamp>/

EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

# Configuration
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
RESULTS_DIR="$(artifacts_root "$ROOT")/agent_skill_benchmark_$TIMESTAMP"

# Project definitions (name:url format)
TIER1_PROJECTS=(
  "kubernetes:https://github.com/kubernetes/kubernetes"
  "prometheus:https://github.com/prometheus/prometheus"
  "grafana:https://github.com/grafana/grafana"
  "istio:https://github.com/istio/istio"
  "temporal:https://github.com/temporalio/temporal"
)

TIER2_PROJECTS=(
  "vscode:https://github.com/microsoft/vscode"
  "elasticsearch:https://github.com/elastic/elasticsearch"
  "moby:https://github.com/moby/moby"
  "terraform:https://github.com/hashicorp/terraform"
)

TIER3_PROJECTS=(
  "fastapi:https://github.com/tiangolo/fastapi"
  "express:https://github.com/expressjs/express"
  "react-admin:https://github.com/marmelab/react-admin"
)

# Parse arguments
TIER="${1:-all}"
PROJECT_FILTER="${2:-}"

# Check prerequisites
require_cmd git
require_cmd jq

# Clone repository
clone_repo() {
  local name="$1"
  local url="$2"
  local target="/tmp/sruja_test_$name"
  
  if [ -d "$target" ]; then
    info "Repository $name already cloned, updating..."
    if ! git -C "$target" pull 2>&1; then
      warn "Pull failed, using existing checkout"
    fi
  else
    info "Cloning $name from $url..."
    if ! run_with_timeout 600 git clone --depth=1 "$url" "$target" 2>&1; then
        error "Clone failed or timed out after 10 minutes"
        return 1
    else
      :
    fi
  fi
  
  echo "$target"
}

# Run deterministic analysis
run_cli_analysis() {
  local name="$1"
  local repo_path="$2"
  local output_dir="$RESULTS_DIR/$name/cli"
  
  mkdir -p "$output_dir"
  
  info "Running CLI analysis for $name..."
  
  # 1. Quickstart
  header "[1/4] Quickstart Analysis"
  if ! "$SRUJA_BIN" quickstart -r "$repo_path" > "$output_dir/quickstart.log" 2>&1; then
    warn "Quickstart had issues"
  fi
  
  # 2. Scan
  header "[2/4] Scanning Repository"
  if ! "$SRUJA_BIN" scan "$repo_path" --output "$output_dir/graph.json" > "$output_dir/scan.log" 2>&1; then
    warn "Scan had issues"
  fi
  
  # 3. Complexity
  header "[3/4] Complexity Analysis"
  if ! "$SRUJA_BIN" complexity -r "$repo_path" --centrality --coupling > "$output_dir/complexity.log" 2>&1; then
    warn "Complexity analysis had issues"
  fi
  
  # 4. Drift
  header "[4/4] Drift Detection"
  if ! "$SRUJA_BIN" drift -r "$repo_path" > "$output_dir/drift.log" 2>&1; then
    warn "Drift detection found violations (expected)"
  fi
  
  success "CLI analysis complete for $name"
}

# Extract metrics
extract_metrics() {
  local name="$1"
  local output_dir="$RESULTS_DIR/$name/cli"
  local metrics_file="$RESULTS_DIR/$name/metrics.json"
  
  info "Extracting metrics for $name..."
  
  # Parse quickstart output
  local health_score="N/A"
  local modules=0
  local services=0
  
  if [ -f "$output_dir/quickstart.log" ]; then
    health_score=$(grep -oE "Health Score: [0-9]+/[0-9]+" "$output_dir/quickstart.log" | head -1 || echo "N/A")
    modules=$(grep -oE "Found [0-9,]+ components" "$output_dir/quickstart.log" | grep -oE '[0-9,]+' | tr -d ',' | head -1 || echo "0")
    services=$(grep -E "services" "$output_dir/quickstart.log" | grep -oE '[0-9,]+' | tr -d ',' | head -1 || echo "0")
  fi
  
  # Parse graph size
  local graph_size=0
  if [ -f "$output_dir/graph.json" ]; then
    graph_size=$(du -m "$output_dir/graph.json" | cut -f1)
  fi
  
  # Generate metrics JSON
  cat > "$metrics_file" << EOF
{
  "project": "$name",
  "timestamp": "$(date -Iseconds)",
  "cli_analysis": {
    "health_score": "$health_score",
    "modules": $modules,
    "services": $services,
    "graph_size_mb": $graph_size,
    "quickstart_log": "cli/quickstart.log",
    "graph_file": "cli/graph.json",
    "complexity_log": "cli/complexity.log",
    "drift_log": "cli/drift.log"
  },
  "agent_analysis": {
    "status": "pending",
    "architecture_file": "architecture.sruja",
    "evaluation_file": "evaluation.json"
  }
}
EOF
  
  success "Metrics extracted for $name"
}

# Generate agent instructions
generate_agent_instructions() {
  local name="$1"
  local repo_path="$2"
  local output_dir="$RESULTS_DIR/$name"
  
  cat > "$output_dir/AGENT_INSTRUCTIONS.md" << EOF
# Agent Analysis Instructions for $name

## Context

You are analyzing the **$name** repository using the sruja-architecture skill.

## Files Available

- Repository: \`$repo_path\`
- Graph: \`$output_dir/cli/graph.json\`
- Quickstart findings: \`$output_dir/cli/quickstart.log\`
- Complexity metrics: \`$output_dir/cli/complexity.log\`
- Drift findings: \`$output_dir/cli/drift.log\`

## Your Task

Use the \`@sruja-architecture\` skill to:

1. **Analyze the codebase**
   - Read README.md and key documentation
   - Scan package.json, go.mod, Cargo.toml, or other dependency files
   - Identify main systems, services, and containers
   - Detect technologies (languages, frameworks, databases)

2. **Review the graph**
   - Examine \`cli/graph.json\` for dependency patterns
   - Identify architectural layers and boundaries
   - Note any god modules or bottlenecks

3. **Generate architecture.sruja**
   - Use appropriate abstraction level (aim for 10-30 top-level components)
   - Focus on major architectural elements, NOT individual functions
   - Include technology tags and descriptions
   - Show key relationships and data flows
   - Save to: \`$output_dir/architecture.sruja\`

4. **Validate**
   - Run: \`sruja lint $output_dir/architecture.sruja\`
   - Fix any errors

5. **Compare to code**
   - Run: \`sruja drift -r $repo_path -a $output_dir/architecture.sruja\`
   - Review violations and adjust if needed

## Expected Output Structure

\`\`\`sruja
system "$name" {
  description "Brief description of the system"
  
  container "Main Service" {
    technology "Language/Framework"
    description "What this service does"
    
    component "API Layer" {
      technology "REST/GraphQL/gRPC"
    }
    
    component "Business Logic" {
      technology "Framework"
    }
  }
  
  datastore "Primary Database" {
    technology "PostgreSQL/MongoDB/etc"
  }
  
  // Relationships
  container "Main Service" -> datastore "Primary Database" "queries"
}
\`\`\`

## Quality Criteria

- **Abstraction**: 10-30 top-level components (NOT 100+)
- **Completeness**: All major systems included
- **Accuracy**: Technologies and relationships correct
- **Clarity**: Readable and understandable

## After Generation

Run the evaluation script:
\`\`\`bash
./scripts/evaluate_agent_output.sh $name
\`\`\`
EOF
  
  success "Agent instructions generated for $name"
}

# Test a single project
test_project() {
  local project_def="$1"
  local name="${project_def%%:*}"
  local url="${project_def#*:}"
  
  header "═══════════════════════════════════════════════════════════"
  header "Testing Project: $name"
  header "═══════════════════════════════════════════════════════════"
  
  local output_dir="$RESULTS_DIR/$name"
  mkdir -p "$output_dir"
  
  # Step 1: Clone
  local repo_path
  if ! repo_path=$(clone_repo "$name" "$url"); then
    error "Failed to clone $name"
    echo "{\"project\": \"$name\", \"status\": \"clone_failed\"}" > "$output_dir/metrics.json"
    return 1
  fi
  
  # Step 2: Run CLI analysis
  run_cli_analysis "$name" "$repo_path"
  
  # Step 3: Extract metrics
  extract_metrics "$name"
  
  # Step 4: Generate agent instructions
  generate_agent_instructions "$name" "$repo_path"
  
  success "Completed testing for $name"
  echo ""
  echo "Next step: Run agent analysis manually"
  echo "  1. Open your AI assistant (Cursor, Claude, etc.)"
  echo "  2. Read: $output_dir/AGENT_INSTRUCTIONS.md"
  echo "  3. Follow instructions to generate architecture.sruja"
  echo ""
}

# Main execution
main() {
  header "═══════════════════════════════════════════════════════════"
  header "  Sruja Agent Skill Testing Framework"
  header "═══════════════════════════════════════════════════════════"
  echo ""
  
mkdir -p "$RESULTS_DIR"
success "Artifacts will be written to: $RESULTS_DIR"
  
  mkdir -p "$RESULTS_DIR"
  
  info "Results will be saved to: $RESULTS_DIR"
  echo ""
  
  # Select projects to test
  local projects=()
  
  case "$TIER" in
    tier1)
      projects=("${TIER1_PROJECTS[@]}")
      info "Testing Tier 1 projects (Microservices & Distributed Systems)"
      ;;
    tier2)
      projects=("${TIER2_PROJECTS[@]}")
      info "Testing Tier 2 projects (Large-Scale Applications)"
      ;;
    tier3)
      projects=("${TIER3_PROJECTS[@]}")
      info "Testing Tier 3 projects (Frameworks & Libraries)"
      ;;
    all)
      info "Testing all tiers"
      projects=(
        "${TIER1_PROJECTS[@]}"
        "${TIER2_PROJECTS[@]}"
        "${TIER3_PROJECTS[@]}"
      )
      ;;
    *)
      error "Invalid tier: $TIER"
      echo "Usage: $0 [tier1|tier2|tier3|all] [project_name]"
      exit 1
      ;;
  esac
  
  # Filter by project name if specified
  if [ -n "$PROJECT_FILTER" ]; then
    local found=0
    for project_def in "${projects[@]}"; do
      local name="${project_def%%:*}"
      if [ "$name" = "$PROJECT_FILTER" ]; then
        test_project "$project_def"
        found=1
        break
      fi
    done
    
    if [ $found -eq 0 ]; then
      error "Project '$PROJECT_FILTER' not found in tier $TIER"
      exit 1
    fi
  else
    # Test all projects in the tier
    for project_def in "${projects[@]}"; do
      test_project "$project_def"
    done
  fi
  
  # Generate summary
  header "═══════════════════════════════════════════════════════════"
  header "  Test Summary"
  header "═══════════════════════════════════════════════════════════"
  echo ""
  
  cat > "$RESULTS_DIR/SUMMARY.md" << EOF
# Sruja Agent Skill Test Results

**Date:** $(date)
**Tier:** $TIER
**Results Directory:** $RESULTS_DIR

## Projects Tested

EOF
  
  for project_def in "${projects[@]}"; do
    local name="${project_def%%:*}"
    if [ -f "$RESULTS_DIR/$name/metrics.json" ]; then
      echo "- $name" >> "$RESULTS_DIR/SUMMARY.md"
      echo "  - CLI Analysis: ✅ Complete" >> "$RESULTS_DIR/SUMMARY.md"
      echo "  - Agent Analysis: ⏳ Pending (see AGENT_INSTRUCTIONS.md)" >> "$RESULTS_DIR/SUMMARY.md"
    fi
  done
  
  cat >> "$RESULTS_DIR/SUMMARY.md" << EOF

## Next Steps

1. For each project, run agent analysis:
   - Open AI assistant
   - Read \`PROJECT_NAME/AGENT_INSTRUCTIONS.md\`
   - Generate \`architecture.sruja\`

2. After agent analysis, run evaluation:
   \`\`\`bash
   ./scripts/evaluate_agent_output.sh PROJECT_NAME
   \`\`\`

3. Aggregate results:
   \`\`\`bash
   ./scripts/aggregate_results.sh
   \`\`\`
EOF
  
  success "Testing complete!"
  echo ""
  echo "Results saved to: $RESULTS_DIR"
  echo "See: $RESULTS_DIR/SUMMARY.md"
}

main "$@"
