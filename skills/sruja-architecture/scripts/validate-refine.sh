#!/bin/bash
# Validation and refinement script for sruja-architecture skill
# Usage: ./validate-refine.sh [architecture_file] [repo_path]

set -e

# Default values
ARCH_FILE="${1:-architecture.sruja}"
REPO_PATH="${2:-.}"

echo "🔍 Validating and refining architecture..."

# Check if architecture file exists
if [ ! -f "$ARCH_FILE" ]; then
    echo "❌ Error: Architecture file not found: $ARCH_FILE"
    exit 1
fi

# Check if sruja CLI is available
if ! command -v sruja &> /dev/null; then
    echo "❌ Error: sruja CLI not found"
    echo "Install it with: curl -fsSL https://sruja.ai/install.sh | bash"
    exit 1
fi

echo "📋 Step 1: Linting architecture..."
sruja lint "$ARCH_FILE"
if [ $? -ne 0 ]; then
    echo "❌ Linting failed. Fix errors before proceeding."
    exit 1
fi

echo "✅ Linting passed"

echo ""
echo "📋 Step 2: Checking for drift..."
if [ -f "$ARCH_FILE" ]; then
    sruja drift -r "$REPO_PATH" -a "$ARCH_FILE" --format json > drift-results.json 2>&1
    if [ $? -eq 0 ]; then
        echo "✅ Drift check complete"
        echo "📁 Output: drift-results.json"
    else
        echo "⚠️  Drift detected. Review drift-results.json for details."
    fi
else
    echo "ℹ️  No drift check (baseline not found)"
fi

echo ""
echo "📋 Step 3: Formatting architecture..."
sruja fmt "$ARCH_FILE"
echo "✅ Formatting complete"

echo ""
echo "🎉 Validation and refinement complete!"
echo ""
echo "📊 Summary:"
echo "   - Architecture: $ARCH_FILE"
echo "   - Repo path: $REPO_PATH"
echo "   - Linting: ✅ Passed"
echo "   - Formatting: ✅ Complete"
echo "   - Drift check: ✅ Complete"
echo ""
echo "🎯 Next steps:"
echo "   1. Review drift-results.json if drift was detected"
echo "   2. Use sruja-architecture skill to address drift"
echo "   3. Export documentation: sruja export markdown $ARCH_FILE"
