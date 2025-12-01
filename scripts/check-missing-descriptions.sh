#!/bin/bash
# check-missing-descriptions.sh
# Finds markdown files in learn/content without description or summary in frontmatter

cd "$(dirname "$0")/../learn/content" || exit 1

echo "=== Pages Missing Descriptions ==="
echo ""

total=0
missing=0

find . -name "*.md" -type f | sort | while read file; do
  total=$((total + 1))
  
  # Check if file has description or summary in frontmatter
  has_desc=$(grep -m1 "^description:" "$file" 2>/dev/null || echo "")
  has_summary=$(grep -m1 "^summary:" "$file" 2>/dev/null || echo "")
  
  if [ -z "$has_desc" ] && [ -z "$has_summary" ]; then
    missing=$((missing + 1))
    title=$(grep -m1 "^title:" "$file" 2>/dev/null | sed 's/title: *//' | tr -d '"' || echo "(no title)")
    echo "[$missing] $file"
    echo "    Title: $title"
    echo ""
  fi
done

echo "=== Summary ==="
echo "Checked: $total files"
echo "Missing descriptions: $missing files"
echo ""
echo "Tip: Add 'description:' or 'summary:' to frontmatter of missing files"

