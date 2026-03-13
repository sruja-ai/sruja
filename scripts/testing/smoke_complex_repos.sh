#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/_common.sh"

ROOT="$(project_root)"
SRUJA_BIN="$(ensure_sruja_cli "$ROOT")"

usage() {
  cat <<'EOF'
smoke_complex_repos.sh

Deterministic smoke test against a curated list of large OSS repos:
- clones/updates repos under /tmp/sruja_test_<name>
- runs: sruja quickstart, sruja scan
- writes logs + metrics under evaluation/local-artifacts/testing/

Usage:
  ./scripts/testing/smoke_complex_repos.sh

Environment:
  SRUJA_SMOKE_REPOS   Optional. Comma-separated repo specs "name=https://github.com/org/repo,..."

EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

require_cmd git
require_cmd jq

TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
RESULTS_DIR="$(artifacts_root "$ROOT")/smoke_complex_repos_$TIMESTAMP"
mkdir -p "$RESULTS_DIR"

DEFAULT_PROJECTS=(
  "terraform:https://github.com/hashicorp/terraform"
  "grafana:https://github.com/grafana/grafana"
  "moby:https://github.com/moby/moby"
  "kubernetes:https://github.com/kubernetes/kubernetes"
  "elasticsearch:https://github.com/elastic/elasticsearch"
  "vscode:https://github.com/microsoft/vscode"
)

PROJECTS=("${DEFAULT_PROJECTS[@]}")
if [ -n "${SRUJA_SMOKE_REPOS:-}" ]; then
  IFS=',' read -r -a PROJECTS <<<"$(echo "$SRUJA_SMOKE_REPOS" | sed 's/=/\:/g')"
fi

extract_number() {
    local file="$1"
    local pattern="$2"
    local default="${3:-0}"
    local value
    
    value=$(grep -E "$pattern" "$file" 2>/dev/null | grep -oE '[0-9,]+' | head -1 | tr -d ',' || echo "")
    
    if [ -z "$value" ] || ! [[ "$value" =~ ^[0-9]+$ ]]; then
        echo "$default"
    else
        echo "$value"
    fi
}

run_with_timeout() {
    local duration=$1
    shift
    if command -v timeout &>/dev/null; then
        timeout "$duration" "$@"
    elif command -v gtimeout &>/dev/null; then
        gtimeout "$duration" "$@"
    else
        "$@"
    fi
}

header "══════════════════════════════════════════════════════════════════"
header "  Sruja Smoke Test - Complex GitHub Projects"
header "══════════════════════════════════════════════════════════════════"
echo ""
info "Writing artifacts to: $RESULTS_DIR"
success "Prerequisites OK"
echo ""

SUCCESS_COUNT=0
FAIL_COUNT=0
TIMEOUT_COUNT=0

for PROJECT_SPEC in "${PROJECTS[@]}"; do
    PROJECT=$(echo "$PROJECT_SPEC" | cut -d: -f1)
    URL=$(echo "$PROJECT_SPEC" | cut -d: -f2-)
    
    header "────────────────────────────────────────────────────────────────"
    header "  Testing: $PROJECT"
    header "────────────────────────────────────────────────────────────────"
    
    OUTPUT_DIR="$RESULTS_DIR/$PROJECT"
    mkdir -p "$OUTPUT_DIR"
    
    TARGET_REPO="/tmp/sruja_test_$PROJECT"
    
    if [ -d "$TARGET_REPO" ]; then
        info "Updating existing repository..."
        if ! git -C "$TARGET_REPO" fetch --depth=1 2>/dev/null; then
            warn "Could not update, using existing checkout"
        fi
    else
        info "Cloning $PROJECT (timeout: 10min)..."
        if ! run_with_timeout 600 git clone --depth=1 "$URL" "$TARGET_REPO" 2>&1 | tee "$OUTPUT_DIR/clone.log"; then
            error "Clone failed or timed out"
            ((TIMEOUT_COUNT++))
            cat > "$OUTPUT_DIR/metrics.json" <<EOF
{
  "project": "$PROJECT",
  "url": "$URL",
  "status": "clone_timeout",
  "health_score": "N/A",
  "modules": 0,
  "services": 0,
  "circular_deps": 0,
  "graph_nodes": 0,
  "graph_edges": 0,
  "graph_size_mb": 0.0,
  "duration_seconds": 0,
  "timestamp": "$(date -Iseconds)"
}
EOF
            echo ""
            continue
        fi
    fi
    success "Repository ready"
    
    info "Running Sruja analysis..."
    START=$(date +%s)
    
    if ! "$SRUJA_BIN" quickstart -r "$TARGET_REPO" > "$OUTPUT_DIR/quickstart.log" 2>&1; then
        DURATION=$(($(date +%s) - START))
        error "Analysis failed after ${DURATION}s"
        ((FAIL_COUNT++))
        
        warn "Last 30 lines of output:"
        tail -30 "$OUTPUT_DIR/quickstart.log" | sed 's/^/    /'
        
        cat > "$OUTPUT_DIR/metrics.json" <<EOF
{
  "project": "$PROJECT",
  "url": "$URL",
  "status": "analysis_failed",
  "health_score": "N/A",
  "modules": 0,
  "services": 0,
  "circular_deps": 0,
  "graph_nodes": 0,
  "graph_edges": 0,
  "graph_size_mb": 0.0,
  "duration_seconds": $DURATION,
  "timestamp": "$(date -Iseconds)"
}
EOF
        echo ""
        continue
    fi
    
    DURATION=$(($(date +%s) - START))
    ((SUCCESS_COUNT++))
    
    HEALTH=$(grep -E "Health Score:" "$OUTPUT_DIR/quickstart.log" | grep -oE '[0-9]+' | head -1 || echo "N/A")
    MODULES=$(extract_number "$OUTPUT_DIR/quickstart.log" "modules" 0)
    SERVICES=$(extract_number "$OUTPUT_DIR/quickstart.log" "services" 0)
    DATABASES=$(extract_number "$OUTPUT_DIR/quickstart.log" "databases" 0)
    CYCLES=$(extract_number "$OUTPUT_DIR/quickstart.log" "circular|cycle" 0)
    
    info "Generating dependency graph..."
    if ! "$SRUJA_BIN" scan "$TARGET_REPO" --output "$OUTPUT_DIR/graph.json" 2>"$OUTPUT_DIR/scan_error.log"; then
        warn "Graph generation failed"
        cat "$OUTPUT_DIR/scan_error.log" | head -20 | sed 's/^/    /'
    fi
    
    if [ -f "$OUTPUT_DIR/graph.json" ]; then
        if ! jq '.' "$OUTPUT_DIR/graph.json" >/dev/null 2>&1; then
            warn "Generated graph.json is not valid JSON"
            NODES=0
            EDGES=0
            GRAPH_SIZE_MB=0
        else
            GRAPH_SIZE=$(stat -f%z "$OUTPUT_DIR/graph.json" 2>/dev/null || stat -c%s "$OUTPUT_DIR/graph.json" 2>/dev/null || echo "0")
            NODES=$(jq '.nodes | length' "$OUTPUT_DIR/graph.json" 2>/dev/null || echo "0")
            EDGES=$(jq '.edges | length' "$OUTPUT_DIR/graph.json" 2>/dev/null || echo "0")
            IMPORT_EDGES=$(jq '[.edges[] | select(.evidence[0].rule == "imports")] | length' "$OUTPUT_DIR/graph.json" 2>/dev/null || echo "0")
            MODULE_EDGES=$(jq '[.edges[] | select(.evidence[0].rule == "module_imports")] | length' "$OUTPUT_DIR/graph.json" 2>/dev/null || echo "0")
            GRAPH_SIZE_MB=$(echo "$GRAPH_SIZE" | awk '{printf "%.2f", $1/1024/1024}')
        fi
        
        if [[ "$HEALTH" =~ ^[0-9]+$ ]]; then
            if [ "$HEALTH" -ge 70 ]; then
                STATUS="✅ Good"
            elif [ "$HEALTH" -ge 50 ]; then
                STATUS="⚠️  Fair"
            else
                STATUS="🔴 Needs Work"
            fi
        else
            STATUS="❓ Unknown"
        fi
        
        success "Analysis complete (${DURATION}s)"
        echo "  Health Score: $HEALTH/100"
        echo "  Components: $MODULES modules, $SERVICES services, $DATABASES databases"
        echo "  Graph: $NODES nodes, $EDGES edges (${GRAPH_SIZE_MB}MB)"
        echo "  Dependencies: $IMPORT_EDGES imports, $MODULE_EDGES module-level"
        echo "  Status: $STATUS"
        
        if [ "$CYCLES" -eq 0 ] && [ "$MODULES" -gt 100 ]; then
            warn "No cycles detected in large codebase ($MODULES modules) - detection may need improvement"
        fi
        
        cat > "$OUTPUT_DIR/metrics.json" <<EOF
{
  "project": "$PROJECT",
  "url": "$URL",
  "status": "success",
  "health_score": "$HEALTH",
  "modules": $MODULES,
  "services": $SERVICES,
  "databases": $DATABASES,
  "circular_deps": $CYCLES,
  "graph_nodes": $NODES,
  "graph_edges": $EDGES,
  "import_edges": $IMPORT_EDGES,
  "module_edges": $MODULE_EDGES,
  "graph_size_mb": $GRAPH_SIZE_MB,
  "duration_seconds": $DURATION,
  "timestamp": "$(date -Iseconds)"
}
EOF
    else
        warn "Analysis completed but graph generation failed"
        cat > "$OUTPUT_DIR/metrics.json" <<EOF
{
  "project": "$PROJECT",
  "url": "$URL",
  "status": "graph_failed",
  "health_score": "$HEALTH",
  "modules": $MODULES,
  "services": $SERVICES,
  "circular_deps": $CYCLES,
  "graph_nodes": 0,
  "graph_edges": 0,
  "graph_size_mb": 0.0,
  "duration_seconds": $DURATION,
  "timestamp": "$(date -Iseconds)"
}
EOF
    fi
    
    echo ""
done

header "══════════════════════════════════════════════════════════════════"
header "  Test Summary"
header "══════════════════════════════════════════════════════════════════"
echo ""
echo "  Total Projects: ${#PROJECTS[@]}"
success "Successful: $SUCCESS_COUNT"
[ $FAIL_COUNT -gt 0 ] && error "Failed: $FAIL_COUNT"
[ $TIMEOUT_COUNT -gt 0 ] && warn "Timeouts: $TIMEOUT_COUNT"
echo ""

TOTAL_MODULES=$(find "$RESULTS_DIR" -name "metrics.json" -exec jq -r 'select(.status == "success") | .modules // 0' {} \; 2>/dev/null | awk '{sum+=$1} END {print sum+0}')
TOTAL_NODES=$(find "$RESULTS_DIR" -name "metrics.json" -exec jq -r 'select(.status == "success") | .graph_nodes // 0' {} \; 2>/dev/null | awk '{sum+=$1} END {print sum+0}')
TOTAL_EDGES=$(find "$RESULTS_DIR" -name "metrics.json" -exec jq -r 'select(.status == "success") | .graph_edges // 0' {} \; 2>/dev/null | awk '{sum+=$1} END {print sum+0}')

echo "  Total Modules Scanned: $TOTAL_MODULES"
echo "  Total Graph Nodes: $TOTAL_NODES"
echo "  Total Graph Edges: $TOTAL_EDGES"
echo ""
echo "Results saved to: $RESULTS_DIR"
echo ""

SUMMARY_FILE="$RESULTS_DIR/SUMMARY.md"

cat > "$SUMMARY_FILE" <<EOF
# Sruja Test Results

**Date:** $(date)  
**Projects:** ${#PROJECTS[@]} | **Success:** $SUCCESS_COUNT | **Failed:** $FAIL_COUNT | **Timeouts:** $TIMEOUT_COUNT

## Results

| Project | Health | Modules | Services | Edges | Duration | Status |
|---------|--------|---------|----------|-------|----------|--------|
EOF

for PROJECT_SPEC in "${PROJECTS[@]}"; do
    PROJECT=$(echo "$PROJECT_SPEC" | cut -d: -f1)
    METRICS="$RESULTS_DIR/$PROJECT/metrics.json"
    
    if [ -f "$METRICS" ]; then
        HEALTH=$(jq -r '.health_score // "N/A"' "$METRICS" 2>/dev/null || echo "N/A")
        MODULES=$(jq -r '.modules // 0' "$METRICS" 2>/dev/null || echo "0")
        SERVICES=$(jq -r '.services // 0' "$METRICS" 2>/dev/null || echo "0")
        EDGES=$(jq -r '.graph_edges // 0' "$METRICS" 2>/dev/null || echo "0")
        DURATION=$(jq -r '.duration_seconds // 0' "$METRICS" 2>/dev/null || echo "0")
        STATUS=$(jq -r '.status // "unknown"' "$METRICS" 2>/dev/null || echo "unknown")
        
        if [ "$STATUS" = "success" ]; then
            STATUS_ICON="✅"
        elif [ "$STATUS" = "clone_timeout" ]; then
            STATUS_ICON="⏭️"
        else
            STATUS_ICON="❌"
        fi
        
        echo "| $PROJECT | $HEALTH | $MODULES | $SERVICES | $EDGES | ${DURATION}s | $STATUS_ICON |" >> "$SUMMARY_FILE"
    fi
done

cat >> "$SUMMARY_FILE" <<EOF

## Issues Found

$(if [ $FAIL_COUNT -gt 0 ]; then echo "- **$FAIL_COUNT** projects failed analysis"; fi)
$(if [ $TIMEOUT_COUNT -gt 0 ]; then echo "- **$TIMEOUT_COUNT** projects timed out during clone"; fi)
$(if [ $FAIL_COUNT -eq 0 ] && [ $TIMEOUT_COUNT -eq 0 ]; then echo "- All projects analyzed successfully!"; fi)

## Files

- \`SUMMARY.md\` - This file
- \`*/metrics.json\` - Per-project metrics
- \`*/graph.json\` - Dependency graphs
- \`*/quickstart.log\` - Full analysis output
EOF

success "Summary saved to $SUMMARY_FILE"
