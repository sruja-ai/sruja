#!/usr/bin/env bash
# Capture architecture snapshots at multiple refs and write manifest + graph JSONs.
# When no refs given, uses LLM to suggest architecture-significant commits (or tags fallback).
#
# Usage: ./capture_timeline.sh [REPO] [ref1 ref2 ...] [--no-llm] [--max-refs N] [--commits N] [--force] [--overwrite] [--adr] [--no-adr]
#   REPO: name under test-repos/ or path to git repo. Omit to auto-detect (CWD if git repo, or only dir in test-repos).
#   refs: optional; if omitted, LLM suggests refs (when API key set) or tags/--commits fallback.
#
# Requires: sruja CLI built, jq for parsing suggest-refs output. Optional: .env with LLM key for smart ref selection.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPOS_DIR="${SCRIPT_DIR}/test-repos"
[ -f "${SCRIPT_DIR}/.env" ] && set -a && . "${SCRIPT_DIR}/.env" && set +a
. "${SCRIPT_DIR}/lib.sh"

# Parse flags and positional args
REPO_NAME=""
REFS=()
NO_LLM=""
MAX_REFS=30
COMMITS=""
FORCE=""
OVERWRITE=""
CAPTURE_ADR=""
while [ $# -gt 0 ]; do
  case "$1" in
    --no-llm)    NO_LLM=1; shift ;;
    --max-refs)  MAX_REFS="$2"; shift 2 ;;
    --commits)   COMMITS="$2"; shift 2 ;;
    --force)     FORCE=1; shift ;;
    --overwrite) OVERWRITE=1; shift ;;
    --adr)       CAPTURE_ADR=1; shift ;;
    --no-adr)    CAPTURE_ADR=0; shift ;;
    -*)          echo "Unknown option: $1" >&2; exit 1 ;;
    *)
      if [ -z "$REPO_NAME" ]; then
        REPO_NAME="$1"
      else
        REFS+=("$1")
      fi
      shift
      ;;
  esac
done

# Resolve REPO if not given (Section 3.6: auto-detect)
if [ -z "$REPO_NAME" ]; then
  if [ -d "$(pwd)/.git" ] && [[ "$(pwd)" == "${REPOS_DIR}"/* ]]; then
    REPO_NAME="$(basename "$(pwd)")"
    REPO_PATH="$(pwd)"
  else
    if [ -d "$REPOS_DIR" ]; then
      N=$(find "$REPOS_DIR" -maxdepth 1 -type d ! -name ".*" ! -path "$REPOS_DIR" | wc -l | tr -d ' ')
      if [ "$N" -eq 1 ]; then
        REPO_NAME="$(basename "$(find "$REPOS_DIR" -maxdepth 1 -type d ! -path "$REPOS_DIR" | head -1)")"
        REPO_PATH="${REPOS_DIR}/${REPO_NAME}"
      elif [ "$N" -gt 1 ]; then
        HAS_LLM=$(has_llm_key)
        if [ -n "$HAS_LLM" ]; then
          echo "Multiple repos in test-repos; using first (run with REPO name to choose): $(ls -1 "$REPOS_DIR" | head -1)" >&2
          REPO_NAME="$(ls -1 "$REPOS_DIR" | head -1)"
          REPO_PATH="${REPOS_DIR}/${REPO_NAME}"
        else
          echo "Which repo? (name under test-repos/):" >&2
          ls -1 "$REPOS_DIR" 2>/dev/null | cat -n >&2
          read -r REPO_NAME
          REPO_PATH="${REPOS_DIR}/${REPO_NAME}"
        fi
      fi
    fi
  fi
fi
[ -z "$REPO_PATH" ] && REPO_PATH="${REPOS_DIR}/${REPO_NAME}"

# Allow REPO to be an absolute or relative path to a git repo
if [ -d "$REPO_NAME" ] && [ -d "$REPO_NAME/.git" ]; then
  REPO_PATH="$(cd "$REPO_NAME" && pwd)"
  REPO_NAME="$(basename "$REPO_PATH")"
fi

if [ -z "$REPO_NAME" ] || [ ! -d "$REPO_PATH" ]; then
  echo "Usage: $0 [REPO] [ref1 ref2 ...] [--no-llm] [--max-refs N] [--commits N] [--force] [--adr] [--no-adr]" >&2
  echo "  REPO: name under test-repos/ or path to git repo. Omit to auto-detect." >&2
  exit 1
fi

if [ ! -d "$REPO_PATH/.git" ]; then
  echo "Error: not a git repo: ${REPO_PATH}" >&2
  exit 1
fi

SRUJA="$(find_sruja)"
if [ -z "$SRUJA" ]; then
  echo "Error: sruja CLI not found. Run 'make build' from repo root or set PATH." >&2
  exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is required (install with: brew install jq or apt install jq)." >&2
  exit 1
fi

OUT_DIR="${SCRIPT_DIR}/timelines/${REPO_NAME}"
if [ -d "$OUT_DIR" ] && [ -n "$OVERWRITE" ]; then
  rm -rf "$OUT_DIR"
fi
mkdir -p "$OUT_DIR"

# Default branch: main or master
default_branch() {
  (cd "$REPO_PATH" && git rev-parse -q --verify main 2>/dev/null) && echo "main" && return
  (cd "$REPO_PATH" && git rev-parse -q --verify master 2>/dev/null) && echo "master" && return
  (cd "$REPO_PATH" && git symbolic-ref -q refs/remotes/origin/HEAD 2>/dev/null) | sed 's|^refs/remotes/origin/||' && return
  echo "main"
}

# Ref to safe filename: drop refs/heads and refs/tags, replace / with -, SHA -> 7 char
sanitize_ref() {
  local r="$1"
  r="${r#refs/heads/}"
  r="${r#refs/tags/}"
  r="${r//\//-}"
  if [[ "$r" =~ ^[0-9a-fA-F]+$ ]]; then
    r="${r:0:7}"
  fi
  echo "$r"
}

# Build ref list if not provided
if [ ${#REFS[@]} -eq 0 ]; then
  BRANCH="$(default_branch)"
  if [ -n "$COMMITS" ]; then
    mapfile -t REFS < <(cd "$REPO_PATH" && git rev-list -n "$COMMITS" "$BRANCH" 2>/dev/null)
    [ ${#REFS[@]} -eq 0 ] && REFS=("$BRANCH" "HEAD")
  elif [ -z "$NO_LLM" ] && [ -n "$(has_llm_key)" ]; then
    if command -v jq >/dev/null 2>&1; then
      RAW=$("$SRUJA" timeline suggest-refs -r "$REPO_PATH" 2>/dev/null) || true
      if [ -n "$RAW" ]; then
        while IFS= read -r line; do
          [ -n "$line" ] && REFS+=("$line")
        done < <(echo "$RAW" | jq -r '.[]?' 2>/dev/null)
      fi
    fi
    if [ ${#REFS[@]} -eq 0 ]; then
      echo "LLM suggest-refs failed or returned nothing; falling back to tags + ${BRANCH}" >&2
    fi
  fi
  if [ ${#REFS[@]} -eq 0 ]; then
    REFS=("$BRANCH")
    while IFS= read -r tag; do
      tag=$(echo "$tag" | tr -d '\r' | xargs)
      [ -z "$tag" ] && continue
      [[ "$tag" == *" "* ]] && continue
      [[ "$tag" == +* ]] && continue
      REFS+=("$tag")
    done < <(cd "$REPO_PATH" && git tag -l 2>/dev/null | while read -r t; do git log -1 --format="%ci %(refname:short)" "$t" 2>/dev/null; done | sort | awk '{print $NF}')
    # Dedupe and cap (keep order)
    declare -a seen=()
    declare -a out=()
    for r in "${REFS[@]}"; do
      if [[ " ${seen[*]} " != *" $r "* ]]; then
        seen+=("$r")
        out+=("$r")
      fi
    done
    REFS=("${out[@]}")
    REFS=("${REFS[@]:0:$MAX_REFS}")
  fi
fi

# Dirty tree: abort unless interactive and user says yes, or --force
cd "$REPO_PATH"
SAVED_REF="$(git rev-parse -q --abbrev-ref HEAD 2>/dev/null || git rev-parse HEAD)"
if ! git diff-index --quiet HEAD -- 2>/dev/null; then
  if [ -n "$FORCE" ]; then
    echo "Warning: uncommitted changes; continuing (--force)." >&2
  elif [ ! -t 0 ]; then
    echo "Error: working tree has uncommitted changes. Commit, stash, or run with --force." >&2
    exit 1
  else
    echo "Warning: working tree has uncommitted changes. Checking out refs may overwrite them." >&2
    read -r -p "Continue anyway? [y/N] " resp
    case "$resp" in
      [yY]) ;;
      *) exit 1 ;;
    esac
  fi
fi

# Auto-enable ADR capture when ADR dirs exist (unless --no-adr)
if [ -z "$CAPTURE_ADR" ]; then
  for d in docs/architecture docs/adr doc/adr; do
    if [ -d "$REPO_PATH/$d" ]; then
      CAPTURE_ADR=1
      break
    fi
  done
  [ -z "$CAPTURE_ADR" ] && CAPTURE_ADR=0
fi

echo "Capturing timeline for ${REPO_NAME} (${#REFS[@]} refs)"
echo "Output: ${OUT_DIR}"
echo ""

MANIFEST_REFS="[]"
MANIFEST_FILES="[]"
FAILED_REFS="[]"
CAPTURED_AT=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
idx=0
for ref in "${REFS[@]}"; do
  ref_checkout=$(echo "$ref" | tr '\n' ' ' | xargs | awk '{print $NF}')
  [ -z "$ref_checkout" ] && ref_checkout="$ref"
  if ! git checkout -q "$ref_checkout" 2>/dev/null; then
    echo "Warning: could not checkout ${ref_checkout}, skipping" >&2
    FAILED_REFS=$(echo "$FAILED_REFS" | jq --arg r "$ref_checkout" '. + [$r]')
    continue
  fi
  sha=$(git rev-parse -q --short HEAD 2>/dev/null || echo "$ref_checkout")
  safe=$(sanitize_ref "$ref_checkout")
  graph_file="graph_${safe}.json"
  graph_path="${OUT_DIR}/${graph_file}"
  if ! "$SRUJA" scan . --output "$graph_path" 2>/dev/null; then
    echo "Warning: scan failed for ${ref_checkout}, skipping" >&2
    FAILED_REFS=$(echo "$FAILED_REFS" | jq --arg r "$ref_checkout" '. + [$r]')
    continue
  fi
  if [ -n "$CAPTURE_ADR" ] && [ "$CAPTURE_ADR" -eq 1 ]; then
    adr_file="${OUT_DIR}/adr_${safe}.json"
    "$SRUJA" intent adr-index -r . -o "$adr_file" --ref-name "$ref_checkout" --sha "$sha" --captured-at "$CAPTURED_AT" 2>/dev/null || true
  fi
  entry=$(jq -n --arg ref "$ref_checkout" --arg sha "$sha" --arg at "$CAPTURED_AT" '{ref: $ref, sha: $sha, captured_at: $at}')
  MANIFEST_REFS=$(echo "$MANIFEST_REFS" | jq --argjson e "$entry" '. + [$e]')
  MANIFEST_FILES=$(echo "$MANIFEST_FILES" | jq --arg f "$graph_file" '. + [$f]')
  echo "  [$((idx+1))/${#REFS[@]}] $ref_checkout -> $graph_file"
  idx=$((idx+1))
done

git checkout -q "$SAVED_REF" 2>/dev/null || true

# Write manifest (repo_path: relative from script dir or just name)
if [[ "$REPO_PATH" == "${REPOS_DIR}"/* ]]; then
  repo_path_rel="test-repos/${REPO_NAME}"
else
  repo_path_rel="$REPO_NAME"
fi
jq -n \
  --arg repo "$REPO_NAME" \
  --arg repo_path "$repo_path_rel" \
  --argjson refs "$MANIFEST_REFS" \
  --argjson graph_files "$MANIFEST_FILES" \
  --argjson adr_capture "${CAPTURE_ADR:-false}" \
  --argjson failed_refs "${FAILED_REFS:-[]}" \
  '{repo: $repo, repo_path: $repo_path, refs: $refs, graph_files: $graph_files, adr_capture: $adr_capture, failed_refs: $failed_refs}' \
  > "${OUT_DIR}/manifest.json"

echo ""
echo "Done. Manifest: ${OUT_DIR}/manifest.json"
echo "Generate report: ./timeline_report.sh ${REPO_NAME}"
