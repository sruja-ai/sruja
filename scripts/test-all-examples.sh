#!/bin/bash
# Comprehensive test script to validate all DSL examples
# Tests compilation and export to dot and markdown formats

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track results
TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0
FAILED_FILES=()

# Check if sruja binary exists
SRUJA_BIN="./bin/sruja"
if [ ! -f "$SRUJA_BIN" ]; then
    echo -e "${YELLOW}⚠️  sruja binary not found. Building...${NC}"
    make build
    if [ ! -f "$SRUJA_BIN" ]; then
        echo -e "${RED}❌ Failed to build sruja binary${NC}"
        exit 1
    fi
fi

echo "=== Testing All DSL Examples ==="
echo ""
echo "Binary: $SRUJA_BIN"
echo ""

# Function to test a single file
test_file() {
    local file="$1"
    local rel_path="$2"
    TOTAL=$((TOTAL + 1))
    
    echo -n "Testing: $rel_path ... "
    
    local errors=""
    
    # Test 1: Compile (validate syntax and semantics)
    local compile_output
    compile_output=$("$SRUJA_BIN" compile "$file" 2>&1)
    local compile_exit=$?
    if [ $compile_exit -ne 0 ]; then
        echo -e "${RED}FAILED (compile)${NC}"
        echo "  Error: $compile_output" | head -3
        FAILED=$((FAILED + 1))
        FAILED_FILES+=("$rel_path (compile)")
        return 1
    fi
    
    # Test 2: Export to DOT
    local dot_output
    dot_output=$("$SRUJA_BIN" export dot "$file" 2>&1)
    local dot_exit=$?
    if [ $dot_exit -ne 0 ]; then
        echo -e "${RED}FAILED (dot export)${NC}"
        echo "  Error: $dot_output" | head -3
        FAILED=$((FAILED + 1))
        FAILED_FILES+=("$rel_path (dot export)")
        return 1
    fi
    # Check that DOT output is not empty (a valid model should produce DOT)
    if [ -z "$dot_output" ] || [ "$dot_output" = "" ]; then
        echo -e "${YELLOW}WARNING (empty dot output)${NC}"
        # Don't fail, but warn - some files might have no visualizable elements
    fi
    
    # Test 3: Export to Markdown
    local md_output
    md_output=$("$SRUJA_BIN" export markdown "$file" 2>&1)
    local md_exit=$?
    if [ $md_exit -ne 0 ]; then
        echo -e "${RED}FAILED (markdown export)${NC}"
        echo "  Error: $md_output" | head -3
        FAILED=$((FAILED + 1))
        FAILED_FILES+=("$rel_path (markdown export)")
        return 1
    fi
    # Check that markdown output is not empty (a valid model should produce markdown)
    if [ -z "$md_output" ] || [ "$md_output" = "" ]; then
        echo -e "${YELLOW}WARNING (empty markdown output)${NC}"
        # Don't fail, but warn
    fi
    
    echo -e "${GREEN}PASSED${NC}"
    PASSED=$((PASSED + 1))
    return 0
}

# Find all .sruja files in example directories
find_examples() {
    local dir="$1"
    if [ ! -d "$dir" ]; then
        return 0
    fi
    find "$dir" -type f -name "*.sruja" 2>/dev/null | sort
}

# Test examples directory
if [ -d "examples" ]; then
    echo "=== Testing examples/ ==="
    while IFS= read -r file || [ -n "$file" ]; do
        if [ -n "$file" ] && [ -f "$file" ]; then
            rel_path="${file#examples/}"
            test_file "$file" "examples/$rel_path"
        fi
    done < <(find_examples "examples")
    echo ""
fi

# Test apps/designer/public/examples directory
if [ -d "apps/designer/public/examples" ]; then
    echo "=== Testing apps/designer/public/examples/ ==="
    while IFS= read -r file || [ -n "$file" ]; do
        if [ -n "$file" ] && [ -f "$file" ]; then
            rel_path="${file#apps/designer/public/examples/}"
            test_file "$file" "apps/designer/public/examples/$rel_path"
        fi
    done < <(find_examples "apps/designer/public/examples")
    echo ""
fi

# Test apps/website/public/examples directory
if [ -d "apps/website/public/examples" ]; then
    echo "=== Testing apps/website/public/examples/ ==="
    while IFS= read -r file || [ -n "$file" ]; do
        if [ -n "$file" ] && [ -f "$file" ]; then
            rel_path="${file#apps/website/public/examples/}"
            test_file "$file" "apps/website/public/examples/$rel_path"
        fi
    done < <(find_examples "apps/website/public/examples")
    echo ""
fi

# Test test-examples directory
if [ -d "test-examples" ]; then
    echo "=== Testing test-examples/ ==="
    while IFS= read -r file || [ -n "$file" ]; do
        if [ -n "$file" ] && [ -f "$file" ]; then
            rel_path="${file#test-examples/}"
            test_file "$file" "test-examples/$rel_path"
        fi
    done < <(find_examples "test-examples")
    echo ""
fi

# Summary
echo "=== Summary ==="
echo "Total:  $TOTAL"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo -e "Skipped: ${YELLOW}$SKIPPED${NC}"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}❌ Failed Files:${NC}"
    for failed_file in "${FAILED_FILES[@]}"; do
        echo -e "  ${RED}✗${NC} $failed_file"
    done
    echo ""
    exit 1
else
    echo -e "${GREEN}✅ All examples passed!${NC}"
    echo ""
    exit 0
fi
