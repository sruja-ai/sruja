#!/bin/bash
# Automated iterative improvement loop
# This script runs the iterative improvement process automatically

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DESIGNER_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$DESIGNER_DIR"

echo "Starting automated iterative quality improvement loop..."
echo "This will run up to 5 iterations, testing and analyzing each time."
echo ""

# Check if dev server is running
if ! curl -s http://localhost:4321 > /dev/null 2>&1; then
    echo "⚠️  Dev server not detected at http://localhost:4321"
    echo "Please start the dev server first: npm run dev"
    exit 1
fi

# Run iterative improvement
npm run test:quality:iterative

echo ""
echo "✅ Iterative improvement loop completed!"
echo "Check apps/designer/tests/results/iterative-improvement-summary.md for full report"

