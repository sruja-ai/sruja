#!/bin/bash
# scripts/check-unused-code.sh
# Comprehensive unused code detection script

set -e

echo "🔍 Checking for unused code in codebase..."
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if tools are installed
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${YELLOW}⚠️  $1 not found. Install with: $2${NC}"
        return 1
    fi
    return 0
}

# TypeScript/JavaScript checks
echo "📦 TypeScript/JavaScript Analysis"
echo "=================================="

# Check for unused exports (ts-prune)
if check_tool "npx" "npm install"; then
    echo ""
    echo "1. Checking for unused TypeScript exports (ts-prune)..."
    if npx ts-prune --project tsconfig.json 2>/dev/null | head -50; then
        echo -e "${GREEN}✓ ts-prune check complete${NC}"
    else
        echo -e "${YELLOW}⚠️  ts-prune not configured. Run: npm install --save-dev ts-prune${NC}"
    fi
fi

# Check for unused files and dependencies (unimported)
if check_tool "npx" "npm install"; then
    echo ""
    echo "2. Checking for unused files and dependencies (unimported)..."
    if npx unimported 2>/dev/null | head -50; then
        echo -e "${GREEN}✓ unimported check complete${NC}"
    else
        echo -e "${YELLOW}⚠️  unimported not configured. Run: npm install --save-dev unimported${NC}"
    fi
fi

# Check for unused npm packages (depcheck)
if check_tool "npx" "npm install"; then
    echo ""
    echo "3. Checking for unused npm packages (depcheck)..."
    if npx depcheck 2>/dev/null; then
        echo -e "${GREEN}✓ depcheck complete${NC}"
    else
        echo -e "${YELLOW}⚠️  depcheck not configured. Run: npm install --save-dev depcheck${NC}"
    fi
fi

# Go checks
echo ""
echo "🔷 Go Analysis"
echo "=============="

# Check with staticcheck
if check_tool "staticcheck" "go install honnef.co/go/tools/cmd/staticcheck@latest"; then
    echo ""
    echo "1. Running staticcheck (includes unused code detection)..."
    if staticcheck ./... 2>&1 | grep -E "(unused|deadcode)" | head -30; then
        echo -e "${GREEN}✓ staticcheck complete${NC}"
    else
        echo -e "${GREEN}✓ No unused code found by staticcheck${NC}"
    fi
fi

# Check with go vet
if check_tool "go" "Go is required"; then
    echo ""
    echo "2. Running go vet (unused variables)..."
    if go vet ./... 2>&1 | head -30; then
        echo -e "${GREEN}✓ go vet complete${NC}"
    fi
fi

# Manual checks
echo ""
echo "🔎 Manual Checks"
echo "================"
echo ""
echo "Checking for common unused code patterns..."

# Find potentially unused TypeScript files (no imports)
echo "1. TypeScript files with no imports found:"
find apps packages -name "*.ts" -o -name "*.tsx" | while read file; do
    if ! grep -q "import.*from" "$file" 2>/dev/null && ! grep -q "export" "$file" 2>/dev/null; then
        # Skip test files and index files
        if [[ ! "$file" =~ (test|spec|index) ]]; then
            echo "   $file"
        fi
    fi
done | head -10

# Find Go files with no imports
echo ""
echo "2. Go files with minimal usage:"
find cmd pkg internal -name "*.go" -type f | while read file; do
    # Count exports
    exports=$(grep -c "^func [A-Z]" "$file" 2>/dev/null || echo "0")
    if [ "$exports" -eq 0 ] && [[ ! "$file" =~ (test|_test) ]]; then
        echo "   $file (no exported functions)"
    fi
done | head -10

echo ""
echo -e "${GREEN}✅ Unused code check complete!${NC}"
echo ""
echo "💡 Tips:"
echo "   - Review the output above for unused code"
echo "   - Use 'Find All References' in your IDE to verify"
echo "   - Some code may be used dynamically (runtime, reflection)"
echo "   - Test files are intentionally excluded"



