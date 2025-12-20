#!/bin/bash
# scripts/test-likec4-format.sh
# Test script to compare LikeC4 JSON export format with Sruja's export

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 Testing LikeC4 JSON Format Compatibility"
echo "============================================"
echo ""

# Check if likec4 CLI is available
if ! command -v likec4 &> /dev/null; then
    echo -e "${YELLOW}⚠️  likec4 CLI not found. Installing...${NC}"
    npm install -g likec4 || {
        echo -e "${RED}❌ Failed to install likec4 CLI${NC}"
        echo "Please install manually: npm install -g likec4"
        exit 1
    }
fi

# Create temp directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

echo "📝 Step 1: Create a minimal LikeC4 test file"
cat > "$TEMP_DIR/test.c4" << 'EOF'
specification {
  element system
  element container
  element component
}

model {
  sys1 = system "System 1" {
    cont1 = container "Container 1" {
      comp1 = component "Component 1"
    }
  }
  
  sys2 = system "System 2"
  
  sys1 -> sys2 "communicates"
}

views {
  view index {
    include *
  }
}
EOF

echo "✅ Created test.c4"
echo ""

echo "📦 Step 2: Export to JSON using LikeC4 CLI"
LIKEC4_JSON="$TEMP_DIR/likec4-export.json"
likec4 export json --output "$LIKEC4_JSON" "$TEMP_DIR/test.c4" || {
    echo -e "${RED}❌ Failed to export LikeC4 JSON${NC}"
    exit 1
}

echo "✅ Exported LikeC4 JSON to: $LIKEC4_JSON"
echo ""

echo "📦 Step 3: Convert same file using Sruja (if available)"
# Convert .c4 to .sruja format (they should be compatible)
SRUJA_JSON="$TEMP_DIR/sruja-export.json"
if command -v sruja &> /dev/null; then
    # Note: Sruja uses .sruja extension, but format should be similar
    # You may need to adjust this based on your Sruja CLI
    sruja export json -o "$SRUJA_JSON" "$TEMP_DIR/test.c4" 2>/dev/null || {
        echo -e "${YELLOW}⚠️  Sruja CLI not available or failed. Skipping...${NC}"
    }
else
    echo -e "${YELLOW}⚠️  Sruja CLI not found. Skipping comparison...${NC}"
fi

echo ""
echo "📊 Step 4: Analyze JSON structure"
echo "=================================="
echo ""

# Use jq if available to pretty print and analyze
if command -v jq &> /dev/null; then
    echo "LikeC4 Export Structure:"
    echo "------------------------"
    jq '{
        has_specification: (.specification != null),
        elements_count: (.elements | length),
        relations_type: (.relations | type),
        relations_count: (if (.relations | type) == "array" then (.relations | length) else (.relations | length) end),
        views_count: (.views | length),
        sample_element: (.elements | to_entries | .[0].value | {id, kind, title}),
        sample_relation: (if (.relations | type) == "array" then .relations[0] else (.relations | to_entries | .[0].value) end | {from, to, source, target, id})
    }' "$LIKEC4_JSON"
    
    echo ""
    echo "Key Observations:"
    echo "-----------------"
    echo "1. Relations format:"
    if jq -e '.relations | type == "array"' "$LIKEC4_JSON" > /dev/null 2>&1; then
        echo -e "   ${GREEN}✅ Relations is an ARRAY${NC}"
        echo "   Sample relation structure:"
        jq '.relations[0]' "$LIKEC4_JSON" | head -20
    else
        echo -e "   ${GREEN}✅ Relations is a RECORD/OBJECT${NC}"
        echo "   Sample relation structure:"
        jq '.relations | to_entries | .[0].value' "$LIKEC4_JSON" | head -20
    fi
    
    echo ""
    echo "2. Relation fields (checking first relation):"
    FIRST_REL=$(jq 'if (.relations | type) == "array" then .relations[0] else (.relations | to_entries | .[0].value) end' "$LIKEC4_JSON")
    echo "   Has 'from' field: $(echo "$FIRST_REL" | jq -r 'has("from")')"
    echo "   Has 'to' field: $(echo "$FIRST_REL" | jq -r 'has("to")')"
    echo "   Has 'source' field: $(echo "$FIRST_REL" | jq -r 'has("source")')"
    echo "   Has 'target' field: $(echo "$FIRST_REL" | jq -r 'has("target")')"
    
    echo ""
    echo "3. Full LikeC4 JSON structure preview:"
    jq '.' "$LIKEC4_JSON" | head -100
    
else
    echo "⚠️  jq not found. Install it for detailed analysis:"
    echo "   macOS: brew install jq"
    echo "   Linux: apt-get install jq"
    echo ""
    echo "Raw JSON preview (first 50 lines):"
    head -50 "$LIKEC4_JSON"
fi

echo ""
echo "📋 Next Steps:"
echo "=============="
echo "1. Review the LikeC4 JSON structure above"
echo "2. Compare with Sruja's export format in: pkg/export/json/likec4_types.go"
echo "3. Check if LikeC4Model.fromDump() accepts this format"
echo "4. Update Sruja's export if format differs"
echo ""
echo "✅ LikeC4 JSON export saved to: $LIKEC4_JSON"



