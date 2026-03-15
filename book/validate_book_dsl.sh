#!/usr/bin/env bash
# Extract Sruja DSL blocks from book markdown and run sruja lint on each.

BOOK_SRC="$(cd "$(dirname "$0")/src" && pwd)"
TMP_DIR="$(cd "$(dirname "$0")" && pwd)/tmp_validate"
SRUJA="${SRUJA:-sruja}"

mkdir -p "$TMP_DIR"
trap "rm -rf $TMP_DIR" EXIT

total=0
passed=0
failed=0
skipped=0

for md_file in $(find "$BOOK_SRC" -name "*.md" -type f); do
  rel="${md_file#$BOOK_SRC/}"
  block_num=0
  
  # Extract sruja code blocks
  while IFS= read -r line; do
    start="${line%%:*}"
    
    # Get content until closing ```
    content=$(awk -v start=$((start + 1)) 'NR>=start && /^```$/ {exit} NR>=start' "$md_file")
    
    # Skip empty or very short blocks
    if [ -z "$content" ] || [ ${#content} -lt 10 ]; then
      continue
    fi
    
    block_num=$((block_num + 1))
    total=$((total + 1))
    
    # Check if marked as partial or expected failure
    if echo "$content" | grep -qiE "(<!--\s*partial\s*-->|#\s*partial|//\s*partial|EXPECTED_FAILURE)"; then
      skipped=$((skipped + 1))
      echo "SKIP $rel (block $block_num) - marked as partial"
      continue
    fi
    
    # Write to temp file
    tmp_name="${rel//\//_}"
    tmp_name="${tmp_name%.md}_${block_num}.sruja"
    echo "$content" > "$TMP_DIR/$tmp_name"
    
    # Run sruja lint
    if $SRUJA lint "$TMP_DIR/$tmp_name" > /dev/null 2>&1; then
      passed=$((passed + 1))
    else
      failed=$((failed + 1))
      echo "FAIL $rel (block $block_num)"
      $SRUJA lint "$TMP_DIR/$tmp_name" 2>&1 | head -3 | sed 's/^/  /'
    fi
  done < <(grep -n '^```sruja$' "$md_file")
done

echo ""
echo "Validated $total DSL blocks: $passed passed, $failed failed, $skipped skipped"

[ $failed -eq 0 ]
