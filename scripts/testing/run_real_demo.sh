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

INPUT_ARG="${1:-.}"

is_url() {
  local s="$1"
  [[ "$s" == http* ]] || [[ "$s" == git@* ]] || [[ "$s" == *github.com* ]]
}

run_sruja_cmd() {
  local description="$1"
  shift
  info "$description"
  echo -e "  ${CYAN}\$ sruja $*${NC}"
  local start_time
  start_time=$(date +%s)
  local exit_code=0
  if ! "$SRUJA_BIN" "$@" 2>&1; then
    exit_code=$?
  fi
  local duration
  duration=$(($(date +%s) - start_time))
  if [ $exit_code -eq 0 ]; then
    success "Completed in ${duration}s"
  else
    error "Command failed with exit code $exit_code (took ${duration}s)"
  fi
  return $exit_code
}

TARGET_REPO=""
REPO_NAME=""
if is_url "$INPUT_ARG"; then
  REPO_NAME="$(basename "$INPUT_ARG" .git)"
  TARGET_REPO="/tmp/sruja_demo_${REPO_NAME}"
  info "GitHub URL detected. Using $TARGET_REPO"

  if [ -d "$TARGET_REPO" ]; then
    info "Directory exists, pulling latest..."
    git -C "$TARGET_REPO" pull 2>&1 | head -3 || warn "Pull failed; using existing checkout"
  else
    info "Cloning (timeout: 5min)..."
    if ! run_with_timeout 300 git clone --depth=1 "$INPUT_ARG" "$TARGET_REPO" 2>&1 | head -10; then
      error "Clone failed or timed out"
      exit 1
    fi
  fi
else
  TARGET_REPO="$INPUT_ARG"
  REPO_NAME="$(basename "$(cd "$TARGET_REPO" 2>/dev/null && pwd || echo "$TARGET_REPO")")"
  REPO_NAME="${REPO_NAME#sruja_demo_}"
fi

if [ ! -d "$TARGET_REPO" ]; then
  error "Repository not found: $TARGET_REPO"
  exit 1
fi

TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
OUTPUT_DIR="$(artifacts_root "$ROOT")/real_demo_${REPO_NAME}_${TIMESTAMP}"
mkdir -p "$OUTPUT_DIR"

IS_SYSTEM=false
for SIGNAL in Dockerfile docker-compose.yml docker-compose.yaml Procfile kubernetes k8s helm skaffold.yaml; do
  if [ -e "$TARGET_REPO/$SIGNAL" ]; then
    IS_SYSTEM=true
    break
  fi
done

if [ "$IS_SYSTEM" = false ]; then
  for SUBDIR in "$TARGET_REPO"/*/; do
    [ -d "$SUBDIR" ] || continue
    DNAME=$(basename "$SUBDIR")
    [[ "$DNAME" == .* ]] && continue
    [[ "$DNAME" == "src" ]] && continue
    [[ "$DNAME" == "crates" ]] && continue
    [[ "$DNAME" == "packages" ]] && continue

    if [ -e "$SUBDIR/Dockerfile" ] || [ -e "$SUBDIR/docker-compose.yml" ]; then
      IS_SYSTEM=true
      break
    fi
  done
fi

header "════════════════════════════════════════════════════"
header "     SRUJA DYNAMIC CODEBASE INTELLIGENCE DEMO      "
header "════════════════════════════════════════════════════"
echo ""

if [ "$IS_SYSTEM" = true ]; then
  success "Project Type: SYSTEM/APPLICATION"
  echo "  Full suite analysis: services, drift, timeline, report"
else
  warn "Project Type: LIBRARY (heuristic)"
  echo "  Library-focused: module coupling, god-module detection"
fi

echo ""
header "[1/6] SCANNING REPOSITORY"
if ! "$SRUJA_BIN" scan "$TARGET_REPO" --output "$OUTPUT_DIR/real.graph.json" >/dev/null 2>&1; then
  warn "Graph generation had issues, continuing..."
else
  success "Graph saved to $OUTPUT_DIR/real.graph.json"
fi

echo ""
header "[2/6] STRUCTURAL COMPLEXITY ANALYSIS"
run_sruja_cmd "Analyzing complexity..." complexity -r "$TARGET_REPO" --centrality --coupling || true

echo ""
header "[3/6] GENERATING ARCHITECTURE BASELINE"
run_sruja_cmd "Generating baseline..." quickstart -r "$TARGET_REPO" --generate-baseline || true
if [ -f "$TARGET_REPO/architecture.sruja" ]; then
  mv "$TARGET_REPO/architecture.sruja" "$OUTPUT_DIR/" || true
  success "Baseline saved to $OUTPUT_DIR/architecture.sruja"
else
  warn "No baseline file created"
fi

echo ""
header "[4/6] ARCHITECTURE DRIFT DETECTION"
if [ -f "$OUTPUT_DIR/architecture.sruja" ]; then
  run_sruja_cmd "Detecting drift vs baseline..." drift -r "$TARGET_REPO" -a "$OUTPUT_DIR/architecture.sruja" || true
else
  run_sruja_cmd "Detecting drift (no baseline)..." drift -r "$TARGET_REPO" || true
fi

echo ""
header "[5/6] ARCHITECTURE TIMELINE"
run_sruja_cmd "Analyzing git history..." timeline explain -r "$TARGET_REPO" || true

echo ""
header "[6/6] FINAL REPORT"
LOG_FILE="$OUTPUT_DIR/analysis.log"
if [ -n "${SRUJA_LLM_PROVIDER:-}" ] || [ -n "${OPENAI_API_KEY:-}" ]; then
  info "LLM enabled. Generating richer report..."
  if "$SRUJA_BIN" analyze -r "$TARGET_REPO" --intent "$OUTPUT_DIR" --view cto --llm >"$LOG_FILE" 2>&1; then
    success "Report saved to $OUTPUT_DIR/final_cto_report.txt"
  else
    warn "Report generation had issues (see $LOG_FILE)"
  fi
else
  info "LLM disabled. Generating basic report..."
  if "$SRUJA_BIN" analyze -r "$TARGET_REPO" --intent "$OUTPUT_DIR" --view cto >"$LOG_FILE" 2>&1; then
    success "Report saved to $OUTPUT_DIR/final_cto_report.txt"
  else
    warn "Report generation had issues (see $LOG_FILE)"
  fi
fi

echo ""
header "════════════════════════════════════════════════════"
header "                     DEMO COMPLETE                 "
header "════════════════════════════════════════════════════"
echo ""
echo "Output files in $OUTPUT_DIR/:"
ls -lh "$OUTPUT_DIR" 2>/dev/null | tail -n +2 | awk '{printf "  • %s (%s)\n", $NF, $5}'

