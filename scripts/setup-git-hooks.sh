#!/bin/bash
# scripts/setup-git-hooks.sh
# Setup git hooks for Sruja development

set -e

HOOKS_DIR=".git/hooks"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Setting up git hooks..."

# Create hooks directory if it doesn't exist
mkdir -p "$HOOKS_DIR"

# Create pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/sh
#
# Pre-commit hook to test Sruja code compilation
# This ensures playground examples and course/docs code blocks compile correctly
#

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "${YELLOW}Running Sruja code compilation tests...${NC}"

# Check if learn files are being committed
if git diff --cached --name-only | grep -qE '^learn/'; then
    echo "Detected changes in learn/ directory"
    
    # Run the compilation tests
    if ! go test -run "TestPlaygroundExamples|TestCourseCodeBlocks|TestDocsCodeBlocks" 2>&1; then
        echo ""
        echo "${RED}❌ Code compilation tests failed!${NC}"
        echo "${RED}Please fix compilation errors before committing.${NC}"
        echo ""
        echo "To see detailed errors, run:"
        echo "  go test -v -run 'TestPlaygroundExamples|TestCourseCodeBlocks|TestDocsCodeBlocks'"
        echo ""
        exit 1
    fi
    
    echo "${GREEN}✅ All code examples compile successfully${NC}"
fi

exit 0
EOF

chmod +x "$HOOKS_DIR/pre-commit"

echo "✅ Pre-commit hook installed successfully!"
echo ""
echo "The hook will run compilation tests when you commit changes to learn/ files."
echo "To bypass the hook (not recommended), use: git commit --no-verify"

