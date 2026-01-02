#!/bin/bash
# Fix common DSL syntax errors
# Usage: ./scripts/fix-dsl-syntax.sh <file.sruja>

if [ $# -eq 0 ]; then
    echo "Usage: $0 <file.sruja>"
    exit 1
fi

FILE="$1"

if [ ! -f "$FILE" ]; then
    echo "Error: File $FILE not found"
    exit 1
fi

# Create backup
cp "$FILE" "$FILE.bak"

# Fix common issues:
# 1. Relations missing "->" operator (pattern: Element1 Element2)
# 2. Relations with REQ/ADR identifiers that should be requirements
# 3. Missing "=" in requirement/ADR definitions

# Pattern 1: Fix relations missing "->" when followed by an identifier
# This handles cases like: "Element1 Element2" -> "Element1 -> Element2"
# But be careful not to break requirement definitions

# Use sed to fix common patterns
# Note: This is a simple fix - manual review is recommended

echo "Fixing common DSL syntax errors in $FILE..."
echo "Backup created at $FILE.bak"

# Fix pattern: Element1 Element2 "label" (missing ->)
# This is tricky because we need to distinguish from requirement definitions
# We'll look for patterns that look like relations but are missing ->

# For now, just report potential issues
echo ""
echo "Potential issues found:"
grep -n "^[A-Za-z][A-Za-z0-9_.]* [A-Za-z][A-Za-z0-9_.]*" "$FILE" | grep -v "=" | grep -v "->" | head -10

echo ""
echo "Please review the file manually. Common fixes:"
echo "1. Add '->' between element names in relations"
echo "2. Ensure requirements use '=' not '->'"
echo "3. Check for incomplete relation statements"

