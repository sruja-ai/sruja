#!/bin/bash
set -e

echo "🚀 Setting up Sruja development environment..."

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo (Rust) not found. Please install: https://rustup.rs/"
    exit 1
fi

# Check for Node.js
if ! command -v npm &> /dev/null; then
    echo "❌ Node.js not found. Please install: https://nodejs.org/"
    exit 1
fi

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "⚠️  wasm-pack not found. Installing..."
    cargo install wasm-pack || echo "❌ Failed to install wasm-pack. WASM builds will fail."
fi

# Install dependencies
echo "📦 Installing Rust dependencies..."
make install

# Install Git hooks
echo "🏗️  Installing Git hooks..."
mkdir -p .git/hooks
cp scripts/pre-commit .git/hooks/pre-commit 2>/dev/null || echo "⚠️  scripts/pre-commit not found; skipping hook installation."
[ -f .git/hooks/pre-commit ] && chmod +x .git/hooks/pre-commit && echo "✅ Pre-commit hook installed."

# Build project
echo "🔨 Building Sruja CLI..."
make build

echo ""
echo "✅ Setup complete! You're ready to contribute to Sruja."
echo "💡 To verify your setup, run: make test"
echo "💡 To keep your context synced across AI editors, run: make daily"
