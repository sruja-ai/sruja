# Shared helpers for evaluation scripts (sourced, not executed).
# Usage: SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)" && . "${SCRIPT_DIR}/lib.sh"

# Finds sruja CLI: prefer repo target/debug or target/release, then PATH.
# Output: path to binary or empty if not found.
find_sruja() {
  local root
  root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
  [ -f "${root}/target/debug/sruja" ] && echo "${root}/target/debug/sruja" && return
  [ -f "${root}/target/release/sruja" ] && echo "${root}/target/release/sruja" && return
  command -v sruja >/dev/null 2>&1 && echo "sruja" && return
  echo ""
}

# Returns "1" if any LLM API key or ollama is configured, else "".
# Call after loading .env (e.g. [ -f "${SCRIPT_DIR}/.env" ] && set -a && . "${SCRIPT_DIR}/.env" && set +a).
has_llm_key() {
  [ -n "$OPENROUTER_API_KEY" ] && echo "1" && return
  [ -n "$OPENAI_API_KEY" ] && echo "1" && return
  [ -n "$ANTHROPIC_API_KEY" ] && echo "1" && return
  [ -n "$GEMINI_API_KEY" ] && echo "1" && return
  [ -n "$GOOGLE_API_KEY" ] && echo "1" && return
  [ -n "$SRUJA_LLM_API_KEY" ] && echo "1" && return
  [ "$SRUJA_LLM_PROVIDER" = "ollama" ] && echo "1" && return
  echo ""
}
