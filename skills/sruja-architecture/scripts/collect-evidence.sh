#!/bin/bash
# Evidence collection script for sruja-architecture skill
# Usage: ./collect-evidence.sh [path] [output_file]

set -e

# Default values
REPO_PATH="${1:-.}"
OUTPUT_FILE="${2:-evidence.json}"

echo "🔍 Collecting evidence from $REPO_PATH..."

# Run sruja discover with context and JSON output
if ! command -v sruja &> /dev/null; then
    echo "❌ Error: sruja CLI not found"
    echo "Install it with: curl -fsSL https://sruja.ai/install.sh | bash"
    exit 1
fi

sruja discover --context -r "$REPO_PATH" --format json > "$OUTPUT_FILE"

if [ $? -eq 0 ]; then
    echo "✅ Evidence collected successfully"
    echo "📁 Output: $OUTPUT_FILE"
    echo ""
    echo "📊 Evidence summary:"
    echo "   - Repository structure"
    echo "   - Detected technologies"
    echo "   - Module boundaries"
    echo "   - Entry points"
    echo "   - Dependencies"
    echo "   - Scan scope"
    echo ""
    echo "🎯 Next: Use this evidence with the sruja-architecture skill in your AI editor"
else
    echo "❌ Error collecting evidence"
    exit 1
fi
