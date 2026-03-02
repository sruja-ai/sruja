#!/usr/bin/env bash
# Generate timeline report (markdown + JSON) from captured snapshots.
# Reads manifest.json, runs drift-diff for each consecutive ref pair.
#
# Usage: ./timeline_report.sh [REPO_OR_TIMELINE_DIR] [-f text|json|both]
#   REPO_OR_TIMELINE_DIR: repo name (resolved to timelines/REPO) or path to timeline dir. Omit if only one dir under timelines/.
#
# Requires: sruja CLI, jq.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TIMELINES_DIR="${SCRIPT_DIR}/timelines"
[ -f "${SCRIPT_DIR}/.env" ] && set -a && . "${SCRIPT_DIR}/.env" && set +a
. "${SCRIPT_DIR}/lib.sh"

FORMAT="both"
TARGET=""
while [ $# -gt 0 ]; do
  case "$1" in
    -f) FORMAT="$2"; shift 2 ;;
    -*) echo "Unknown option: $1" >&2; exit 1 ;;
    *)  TARGET="$1"; shift ;;
  esac
done

# Resolve timeline dir (Section 3.6: optional arg)
if [ -z "$TARGET" ]; then
  if [ -d "$TIMELINES_DIR" ]; then
    N=$(find "$TIMELINES_DIR" -maxdepth 1 -type d ! -path "$TIMELINES_DIR" | wc -l | tr -d ' ')
    if [ "$N" -eq 1 ]; then
      TARGET="$(basename "$(find "$TIMELINES_DIR" -maxdepth 1 -type d ! -path "$TIMELINES_DIR" | head -1)")"
    elif [ "$N" -gt 1 ]; then
      echo "Which timeline? (name under timelines/):" >&2
      ls -1 "$TIMELINES_DIR" 2>/dev/null | cat -n >&2
      read -r TARGET
    fi
  fi
fi

if [ -z "$TARGET" ]; then
  echo "Usage: $0 [REPO_OR_TIMELINE_DIR] [-f text|json|both]" >&2
  echo "  REPO_OR_TIMELINE_DIR: repo name or path to timeline dir. Omit if only one timeline exists." >&2
  exit 1
fi

if [ -d "$TARGET" ] && [ -f "$TARGET/manifest.json" ]; then
  TIMELINE_DIR="$(cd "$TARGET" && pwd)"
  REPO_NAME="$(basename "$TIMELINE_DIR")"
elif [ -f "${TIMELINES_DIR}/${TARGET}/manifest.json" ]; then
  TIMELINE_DIR="${TIMELINES_DIR}/${TARGET}"
  REPO_NAME="$TARGET"
else
  echo "Error: timeline dir or manifest not found: $TARGET" >&2
  exit 1
fi

SRUJA="$(find_sruja)"
if [ -z "$SRUJA" ]; then
  echo "Error: sruja CLI not found." >&2
  exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is required." >&2
  exit 1
fi

REPO=$(jq -r '.repo // ""' "$TIMELINE_DIR/manifest.json")
[ -z "$REPO" ] && REPO="$REPO_NAME"
GRAPH_FILES=()
while IFS= read -r line; do
  [ -n "$line" ] && GRAPH_FILES+=("$line")
done < <(jq -r '.graph_files[]?' "$TIMELINE_DIR/manifest.json")
REFS_JSON=$(jq -c '.refs | map(.ref)' "$TIMELINE_DIR/manifest.json")
[ -z "$REFS_JSON" ] || [ "$REFS_JSON" = "null" ] && REFS_JSON="[]"

if [ ${#GRAPH_FILES[@]} -lt 2 ]; then
  echo "One snapshot only; no steps to compare."
  MD_FILE="${TIMELINE_DIR}/timeline_${REPO}.md"
  JSON_FILE="${TIMELINE_DIR}/timeline_${REPO}.json"
  echo "# Architecture timeline: ${REPO}" > "$MD_FILE"
  echo "" >> "$MD_FILE"
  echo "One snapshot only; no steps to compare." >> "$MD_FILE"
  jq -n --arg repo "$REPO" --argjson refs "$REFS_JSON" '{repo: $repo, refs: $refs, steps: []}' > "$JSON_FILE"
  echo "Wrote $MD_FILE and $JSON_FILE"
  exit 0
fi

steps_json="[]"
steps_md=""
refs_display=""
for ((i=0; i < ${#GRAPH_FILES[@]} - 1; i++)); do
  base_file="${GRAPH_FILES[$i]}"
  head_file="${GRAPH_FILES[$i+1]}"
  base_path="${TIMELINE_DIR}/${base_file}"
  head_path="${TIMELINE_DIR}/${head_file}"
  base_ref="${base_file#graph_}"
  base_ref="${base_ref%.json}"
  head_ref="${head_file#graph_}"
  head_ref="${head_ref%.json}"
  if [ -z "$refs_display" ]; then
    refs_display="${base_ref}"
  fi
  refs_display="${refs_display} → ${head_ref}"

  diff_out=$("$SRUJA" drift-diff -b "$base_path" -h "$head_path" -f json 2>/dev/null) || true
  if [ -z "$diff_out" ]; then
    new_components=0
    removed_components=0
    new_edges=0
    removed_edges=0
    errors=0
    warnings=0
  else
    new_components=$(echo "$diff_out" | jq -r '.summary.missing_components // 0')
    removed_components=$(echo "$diff_out" | jq -r '.summary.new_components // 0')
    new_edges=$(echo "$diff_out" | jq -r '.summary.new_dependencies // 0')
    removed_edges=$(echo "$diff_out" | jq -r '.summary.removed_dependencies // 0')
    errors=$(echo "$diff_out" | jq -r '[.violations[]? | select(.severity == "Error")] | length')
    warnings=$(echo "$diff_out" | jq -r '[.violations[]? | select(.severity == "Warning")] | length')
  fi

  adr_base=""
  adr_head=""
  if [ -f "${TIMELINE_DIR}/adr_${base_ref}.json" ]; then
    adr_base=$(jq -r '.adrs | length' "${TIMELINE_DIR}/adr_${base_ref}.json" 2>/dev/null || echo "0")
  fi
  if [ -f "${TIMELINE_DIR}/adr_${head_ref}.json" ]; then
    adr_head=$(jq -r '.adrs | length' "${TIMELINE_DIR}/adr_${head_ref}.json" 2>/dev/null || echo "0")
  fi
  [ -z "$adr_base" ] && adr_base=0
  [ -z "$adr_head" ] && adr_head=0

  step=$(jq -n \
    --arg base "$base_ref" --arg head "$head_ref" \
    --argjson nc "$new_components" --argjson rc "$removed_components" \
    --argjson ne "$new_edges" --argjson re "$removed_edges" \
    --argjson ab "$adr_base" --argjson ah "$adr_head" \
    --argjson err "$errors" --argjson warn "$warnings" \
    '{base_ref: $base, head_ref: $head, new_components: $nc, removed_components: $rc, new_edges: $ne, removed_edges: $re, adrs_at_base: $ab, adrs_at_head: $ah, violations_summary: {errors: $err, warnings: $warn}}')
  steps_json=$(echo "$steps_json" | jq --argjson s "$step" '. + [$s]')

  if [ "$FORMAT" != "json" ]; then
    steps_md="${steps_md}## ${base_ref} → ${head_ref}
- New components: ${new_components}
- Removed components: ${removed_components}
- New edges: ${new_edges}
- Removed edges: ${removed_edges}
- ADRs at base: ${adr_base} | at head: ${adr_head}

"
  fi
done

refs_array="$REFS_JSON"
if [ "$FORMAT" != "json" ]; then
  MD_FILE="${TIMELINE_DIR}/timeline_${REPO}.md"
  {
    echo "# Architecture timeline: ${REPO}"
    echo ""
    echo "Refs: ${refs_display}"
    echo ""
    echo "$steps_md"
  } > "$MD_FILE"
  [ "$FORMAT" = "both" ] && echo "Wrote $MD_FILE"
fi

if [ "$FORMAT" != "text" ]; then
  JSON_FILE="${TIMELINE_DIR}/timeline_${REPO}.json"
  jq -n \
    --arg repo "$REPO" \
    --argjson refs "$refs_array" \
    --argjson steps "$steps_json" \
    '{repo: $repo, refs: $refs, steps: $steps}' > "$JSON_FILE"
  [ "$FORMAT" = "both" ] && echo "Wrote $JSON_FILE"
fi
