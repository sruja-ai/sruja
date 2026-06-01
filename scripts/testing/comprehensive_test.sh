#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/_common.sh"

ROOT="$(project_root)"
SRUJA_BIN="$(ensure_sruja_cli "$ROOT")"

# Load environment from repo root if present
if [ -f "$ROOT/.env" ]; then
  set -a
  # shellcheck disable=SC1090
  source "$ROOT/.env"
  set +a
  export SRUJA_LLM_PROVIDER="${SRUJA_LLM_PROVIDER:-openrouter}"
fi

# Test configuration
TIMEOUT="${TIMEOUT:-600}"  # 10 minutes per project

# Results tracking (under evaluation/local-artifacts/testing/, git-ignored)
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
RESULTS_ROOT="$(artifacts_root "$ROOT")"
RESULTS_DIR="$RESULTS_ROOT/comprehensive_test_$TIMESTAMP"
mkdir -p "$RESULTS_DIR"

header "══════════════════════════════════════════════════════════════════"
header "  Sruja Comprehensive Test Suite - Complex GitHub Projects        "
header "══════════════════════════════════════════════════════════════════"
echo ""
info "Results directory: $RESULTS_DIR"
info "Timeout per project: ${TIMEOUT}s"
echo ""

test_project() {
  local PROJECT=$1
  local URL=$2
  local OUTPUT_DIR="$RESULTS_DIR/$PROJECT"
  local LOG_FILE="$OUTPUT_DIR/test.log"
  local METRICS_FILE="$OUTPUT_DIR/metrics.json"

  mkdir -p "$OUTPUT_DIR"

  header "────────────────────────────────────────────────────────────────"
  header "  Testing: $PROJECT"
  header "────────────────────────────────────────────────────────────────"
  echo "URL: $URL"
  echo ""

  # Clone or update repository
  local TARGET_REPO="/tmp/sruja_test_$PROJECT"
  if [ -d "$TARGET_REPO" ]; then
    info "Repository exists, updating..."
    git -C "$TARGET_REPO" pull --depth=1 2>&1 | head -3 || warn "update skipped"
  else
    info "Cloning $URL (shallow)..."
    if ! git clone --depth=1 "$URL" "$TARGET_REPO" 2>&1 | head -5; then
      error "Clone failed"
      cat >"$METRICS_FILE" <<EOF
{"project":"$PROJECT","status":"clone_failed"}
EOF
      return 1
    fi
  fi

  # Run Sruja quickstart
  info "Running Sruja quickstart..."
  local START_TIME
  START_TIME=$(date +%s)

  if "$SRUJA_BIN" quickstart -r "$TARGET_REPO" >"$LOG_FILE" 2>&1; then
    local END_TIME
    END_TIME=$(date +%s)
    local DURATION=$((END_TIME - START_TIME))

    # Extract metrics from output
    local HEALTH
    local COMPONENTS
    local MODULES
    HEALTH=$(grep -o 'Health Score: [0-9]*' "$LOG_FILE" | grep -o '[0-9]*' || echo "N/A")
    COMPONENTS=$(grep -o 'Found [0-9]* components' "$LOG_FILE" | grep -o '[0-9]*' | head -1 || echo "N/A")
    MODULES=$(grep -o '[0-9]* modules' "$LOG_FILE" | grep -o '[0-9]*' | head -1 || echo "N/A")

    # Generate graph for analysis
    info "Generating architecture graph..."
    if ! "$SRUJA_BIN" scan "$TARGET_REPO" --output "$OUTPUT_DIR/graph.json" 2>&1 >>"$LOG_FILE"; then
      warn "Graph generation skipped"
    fi

    # Collect file sizes and graph stats
    local GRAPH_SIZE="0"
    local NODE_COUNT="0"
    local EDGE_COUNT="0"

    if [ -f "$OUTPUT_DIR/graph.json" ]; then
      GRAPH_SIZE=$(stat -f%z "$OUTPUT_DIR/graph.json" 2>/dev/null || stat -c%s "$OUTPUT_DIR/graph.json" 2>/dev/null || echo "0")
      NODE_COUNT=$(jq '.nodes | length' "$OUTPUT_DIR/graph.json" 2>/dev/null || echo "0")
      EDGE_COUNT=$(jq '.edges | length' "$OUTPUT_DIR/graph.json" 2>/dev/null || echo "0")
    fi

    success "Analysis complete (${DURATION}s)"
    echo "  Health: $HEALTH/100"
    echo "  Components: $COMPONENTS"
    echo "  Graph size: $(echo "$GRAPH_SIZE" | awk '{printf "%.1f MB", $1/1024/1024}')"
    echo "  Nodes: $NODE_COUNT, Edges: $EDGE_COUNT"

    # Save metrics
    cat >"$METRICS_FILE" <<EOF
{
  "project": "$PROJECT",
  "url": "$URL",
  "status": "success",
  "duration_seconds": $DURATION,
  "health_score": $HEALTH,
  "components": $COMPONENTS,
  "modules": $MODULES,
  "graph_size_bytes": $GRAPH_SIZE,
  "node_count": $NODE_COUNT,
  "edge_count": $EDGE_COUNT
}
EOF
    return 0
  else
    local EXIT_CODE=$?
    error "Analysis failed (exit code: $EXIT_CODE)"
    cat >"$METRICS_FILE" <<EOF
{"project":"$PROJECT","status":"failed","exit_code":$EXIT_CODE}
EOF
    return 1
  fi
}

# Determine which projects to test
if [ "${1:-}" = "--distributed" ]; then
  PROJECTS=(
    "kubernetes:https://github.com/kubernetes/kubernetes"
    "grafana:https://github.com/grafana/grafana"
    "cilium:https://github.com/cilium/cilium"
    "vitess:https://github.com/vitessio/vitess"
  )
elif [ "${1:-}" = "--enterprise" ]; then
  PROJECTS=(
    "mattermost:https://github.com/mattermost/mattermost"
    "gitea:https://github.com/go-gitea/gitea"
    "discourse:https://github.com/discourse/discourse"
  )
elif [ "${1:-}" = "--ecommerce" ]; then
  PROJECTS=(
    "saleor:https://github.com/saleor/saleor"
    "medusa:https://github.com/medusajs/medusa"
  )
elif [ "${1:-}" = "--infrastructure" ]; then
  PROJECTS=(
    "terraform:https://github.com/hashicorp/terraform"
    "pulumi:https://github.com/pulumi/pulumi"
    "caddy:https://github.com/caddyserver/caddy"
  )
elif [ "${1:-}" != "" ]; then
  PROJECTS=("$1") # Single project specified as "name:url"
else
  PROJECTS=(
    "grafana:https://github.com/grafana/grafana"
    "mattermost:https://github.com/mattermost/mattermost"
    "saleor:https://github.com/saleor/saleor"
    "terraform:https://github.com/hashicorp/terraform"
    "cilium:https://github.com/cilium/cilium"
  )
  info "Testing representative sample of complex projects"
  echo ""
fi

# Execute tests
TOTAL=0
SUCCESS=0
FAILED=0

for PROJECT_SPEC in "${PROJECTS[@]}"; do
  PROJECT=$(echo "$PROJECT_SPEC" | cut -d: -f1)
  URL=$(echo "$PROJECT_SPEC" | cut -d: -f2-)

  TOTAL=$((TOTAL + 1))
  if test_project "$PROJECT" "$URL"; then
    SUCCESS=$((SUCCESS + 1))
  else
    FAILED=$((FAILED + 1))
  fi
  echo ""
done

# Summary
header "══════════════════════════════════════════════════════════════════"
header "  Test Summary                                                     "
header "══════════════════════════════════════════════════════════════════"
echo ""
echo "Total:  $TOTAL"
echo "✓ Pass: $SUCCESS"
echo "✗ Fail: $FAILED"
if [ $TOTAL -gt 0 ]; then
  SUCCESS_RATE=$(awk "BEGIN {printf \"%.1f\", $SUCCESS * 100.0 / $TOTAL}")
  echo "Success Rate: ${SUCCESS_RATE}%"
fi
echo ""
echo "Results saved to: $RESULTS_DIR"
echo ""

# Generate aggregate summary.json
cat >"$RESULTS_DIR/summary.json" <<EOF
{
  "timestamp": "$(date -Iseconds 2>/dev/null || date +%Y-%m-%dT%H:%M:%S)",
  "total_projects": $TOTAL,
  "successful": $SUCCESS,
  "failed": $FAILED,
  "success_rate": $(awk "BEGIN {printf \"%.2f\", $SUCCESS * 100.0 / $TOTAL}")
}
EOF

# Generate markdown report
cat >"$RESULTS_DIR/REPORT.md" <<EOF
# Sruja Test Results - Complex GitHub Projects

**Date:** $(date)
**Total Projects:** $TOTAL
**Success Rate:** $(awk "BEGIN {printf \"%.1f\", $SUCCESS * 100.0 / $TOTAL}")%

## Results by Project

EOF

for PROJECT_SPEC in "${PROJECTS[@]}"; do
  PROJECT=$(echo "$PROJECT_SPEC" | cut -d: -f1)
  METRICS_FILE="$RESULTS_DIR/$PROJECT/metrics.json"

  if [ -f "$METRICS_FILE" ]; then
    STATUS=$(jq -r '.status' "$METRICS_FILE")
    if [ "$STATUS" = "success" ]; then
      HEALTH=$(jq -r '.health_score // "N/A"' "$METRICS_FILE")
      COMPONENTS=$(jq -r '.components // "N/A"' "$METRICS_FILE")
      DURATION=$(jq -r '.duration_seconds // "N/A"' "$METRICS_FILE")
      NODES=$(jq -r '.node_count // "N/A"' "$METRICS_FILE")
      EDGES=$(jq -r '.edge_count // "N/A"' "$METRICS_FILE")
      GRAPH_SIZE=$(jq -r '.graph_size_bytes // 0' "$METRICS_FILE")
      GRAPH_SIZE_MB=$(awk "BEGIN {printf \"%.1f\", $GRAPH_SIZE / 1024 / 1024}")

      cat >>"$RESULTS_DIR/REPORT.md" <<EOF
### $PROJECT ✓

- **Status:** Success
- **Health Score:** $HEALTH/100
- **Components:** $COMPONENTS
- **Graph:** $NODES nodes, $EDGES edges ($GRAPH_SIZE_MB MB)
- **Duration:** ${DURATION}s
- **Details:** [metrics.json](./$PROJECT/metrics.json) | [test.log](./$PROJECT/test.log)

EOF
    else
      EXIT_CODE=$(jq -r '.exit_code // "N/A"' "$METRICS_FILE")
      cat >>"$RESULTS_DIR/REPORT.md" <<EOF
### $PROJECT ✗

- **Status:** Failed (exit code: $EXIT_CODE)
- **Details:** [metrics.json](./$PROJECT/metrics.json) | [test.log](./$PROJECT/test.log)

EOF
    fi
  fi
done

cat >>"$RESULTS_DIR/REPORT.md" <<EOF

## Analysis & Recommendations

Generated from comprehensive testing on $(date).

EOF

success "Report generated: $RESULTS_DIR/REPORT.md"
echo ""
info "Next steps:"
echo "  1. Review REPORT.md for detailed results"
echo "  2. Check individual test.log files for detailed output"
echo "  3. Analyze graph.json files for architecture patterns"

