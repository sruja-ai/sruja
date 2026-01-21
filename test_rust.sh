#!/bin/bash
# Test script for Rust migration
# Run: bash test_rust.sh

set -e

echo "🔍 Testing Rust Migration..."
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust toolchain first."
    echo "   Visit: https://rustup.rs/"
    exit 1
fi

echo "📦 Checking workspace..."
cargo check --workspace

echo ""
echo "🧪 Running tests..."
cargo test --workspace

echo ""
echo "✅ All tests passed!"
echo ""
echo "📊 Test Summary:"
echo "   - Language parsing tests"
echo "   - Validation engine tests"
echo "   - Export functionality tests"
echo "   - LSP feature tests"
