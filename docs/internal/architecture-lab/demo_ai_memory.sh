#!/usr/bin/env bash
# Demo script for Architecture Explainer + Memory (plan §12).
# Run from repo root. Set LLM env (e.g. OPENAI_API_KEY) for full explain/ask; otherwise fallback is used.
set -e
REPO="${1:-.}"
SRUJA="${SRUJA:-cargo run -p sruja-cli --}"

echo "=== 1. sruja ai explain -r $REPO --topic \"request flow\" ==="
EXPLAIN_OUT=$($SRUJA ai explain -r "$REPO" --topic "request flow" -f json 2>/dev/null || true)
echo "$EXPLAIN_OUT" | head -20
if command -v jq &>/dev/null; then
  ANSWER_ID=$(echo "$EXPLAIN_OUT" | jq -r '.answer_id // empty')
  FACT_ID=$(echo "$EXPLAIN_OUT" | jq -r '.new_fact_ids[0] // empty')
else
  ANSWER_ID=""
  FACT_ID=""
  echo "(jq not found; step 3 will be skipped unless you set ANSWER_ID/FACT_ID manually)"
fi

echo ""
echo "=== 2. sruja ai ask -r $REPO \"Where are architecture boundary risks?\" ==="
$SRUJA ai ask -r "$REPO" "Where are architecture boundary risks?" 2>/dev/null || true

echo ""
echo "=== 3. sruja ai feedback (answer_id=$ANSWER_ID fact_id=$FACT_ID) ==="
if [[ -n "$ANSWER_ID" && -n "$FACT_ID" ]]; then
  $SRUJA ai feedback -r "$REPO" --answer-id "$ANSWER_ID" --fact-id "$FACT_ID" --verdict wrong --comment "Demo: marking one fact wrong" 2>/dev/null || true
else
  echo "Skipped (no answer_id/fact_id from step 1; run with LLM for full demo)"
fi

echo ""
echo "=== 4. sruja ai ask again (after feedback) ==="
$SRUJA ai ask -r "$REPO" "Where are architecture boundary risks?" 2>/dev/null || true

echo ""
echo "=== 5. sruja timeline explain -r $REPO --max-commits 5 ==="
$SRUJA timeline explain -r "$REPO" --max-commits 5 2>/dev/null || true

echo ""
echo "=== Done ==="
