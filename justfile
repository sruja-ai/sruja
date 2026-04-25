# Sruja - Cross-platform Build Commands
# Required: just (https://github.com/casey/just)

set shell := ["bash", "-c"]

# Default command
default: help

# --- Core Development ---

# Run all checks (fmt, lint, test)
check: fmt lint test
    @echo "✅ All checks passed!"

# Build Rust libraries
build:
    @echo "Building Rust libraries..."
    cargo build --release
    @echo "✅ Build complete!"

# Run Rust tests
test:
    @echo "Testing Rust code..."
    cargo test --workspace
    @echo "✅ Tests complete!"

# Setup development environment
setup:
    ./scripts/setup.sh

# --- Code Quality ---

# Format Rust code
fmt:
    @echo "Formatting Rust code..."
    cargo fmt
    @echo "✅ Formatting complete!"

# Lint Rust code
lint:
    @echo "Linting Rust code..."
    cargo clippy --workspace -- -D warnings
    @echo "✅ Linting complete!"

# Run tests with coverage
test-coverage:
    @echo "Running Rust tests with coverage..."
    cargo llvm-cov --workspace

# --- Specialized Tests ---

# Run WASM unit tests
test-wasm:
    @echo "Testing WASM (sruja-wasm)..."
    cd crates/sruja-wasm && wasm-pack test --node
    @echo "✅ WASM tests passed"

# Run Playwright E2E test
test-e2e:
    @echo "Running E2E (Playwright)..."
    npm run e2e
    @echo "✅ E2E tests passed"

# Run CLI smoke tests
test-cli-smoke:
    @echo "Running CLI smoke tests..."
    ./scripts/test_cli_smoke.sh

# --- Build Targets ---

# Build WASM (web target)
wasm:
    @echo "Building Rust WASM (web)..."
    wasm-pack build --target web --out-dir crates/sruja-wasm/pkg crates/sruja-wasm --release
    @if command -v wasm-opt >/dev/null 2>&1; then \
        wasm-opt --enable-bulk-memory --enable-sign-ext --enable-nontrapping-float-to-int -Oz --strip-debug crates/sruja-wasm/pkg/sruja_wasm_bg.wasm -o crates/sruja-wasm/pkg/sruja_wasm_bg.wasm.tmp && mv crates/sruja-wasm/pkg/sruja_wasm_bg.wasm.tmp crates/sruja-wasm/pkg/sruja_wasm_bg.wasm; \
    fi
    @echo "✅ WASM build complete"

# Build WASM for Node.js
wasm-nodejs:
    @echo "Building Rust WASM (nodejs target)..."
    wasm-pack build --target nodejs --out-dir crates/sruja-wasm/pkg-nodejs crates/sruja-wasm --release
    @echo "✅ Node.js WASM build complete"

# Build VS Code extension package
build-extension:
    @echo "Building Sruja VS Code extension..."
    cd extension && npm install --silent
    cd extension && npm run copy:assets
    cd extension && npm run compile
    cd extension && npx --yes @vscode/vsce package --no-dependencies
    @echo "✅ Extension built"

# Install VS Code extension into VS Code or Cursor
install-extension: build-extension
    @VSIX=$$(ls -t extension/sruja-*.vsix | head -1); \
    if [ -z "$$VSIX" ]; then echo "❌ No .vsix found"; exit 1; fi; \
    INSTALLED=0; \
    if command -v cursor >/dev/null 2>&1; then \
        echo "  🖱️  Installing into Cursor..."; \
        cursor --install-extension "$$VSIX" && INSTALLED=1; \
    fi; \
    if command -v code >/dev/null 2>&1; then \
        echo "  💻 Installing into VS Code..."; \
        code --install-extension "$$VSIX" && INSTALLED=1; \
    fi; \
    if [ "$$INSTALLED" -eq 0 ]; then \
        echo "⚠️  No editor CLI found. Install manually: $$VSIX"; \
    else \
        echo "✅ Extension installed!"; \
    fi

# --- Workflows ---

# Daily sync: setup, check, federate, and update AI context
daily: setup check federate context-sync
    @echo "Checking for architecture drift..."
    ./target/release/sruja drift -r . -a repo.sruja || true
    @echo "✅ Daily setup complete. AI editors are now context-aware!"

# Update AI editor context files
context-sync:
    @echo "Updating AI editor context..."
    @./target/release/sruja context -r . -f cursor-rules -o .cursorrules
    @echo -e "\n# Global AI Agent Guidelines\nYou MUST read and strictly adhere to the instructions located in \`AGENTS.md\` before proceeding with any task." >> .cursorrules
    @./target/release/sruja context -r . -f copilot-instructions -o .github/copilot-instructions.md
    @echo -e "\n# Global AI Agent Guidelines\nYou MUST read and strictly adhere to the instructions located in \`AGENTS.md\` before proceeding with any task." >> .github/copilot-instructions.md
    @./target/release/sruja context -r . -f cursor-rules -o CLAUDE.md
    @echo -e "\n# Global AI Agent Guidelines\nYou MUST read and strictly adhere to the instructions located in \`AGENTS.md\` before proceeding with any task." >> CLAUDE.md
    @./target/release/sruja context -r . -f cursor-rules -o .gemini/AGENTS.md
    @echo -e "\n# Global AI Agent Guidelines\nYou MUST read and strictly adhere to the instructions located in \`AGENTS.md\` before proceeding with any task." >> .gemini/AGENTS.md
    @echo "✅ AI context synchronized"

# Federated Architecture
federate: build
    @echo "Composing system index..."
    @./target/release/sruja publish -r crates/sruja-cli --repo-id sruja-cli -o crates/sruja-cli/repo.bundle.json
    @./target/release/sruja publish -r crates/sruja-scan --repo-id sruja-scan -o crates/sruja-scan/repo.bundle.json
    @./target/release/sruja publish -r crates/sruja-language --repo-id sruja-language -o crates/sruja-language/repo.bundle.json
    @./target/release/sruja publish -r crates/sruja-diff --repo-id sruja-diff -o crates/sruja-diff/repo.bundle.json
    @./target/release/sruja compose \
        -i crates/sruja-cli/repo.bundle.json \
        -i crates/sruja-scan/repo.bundle.json \
        -i crates/sruja-language/repo.bundle.json \
        -i crates/sruja-diff/repo.bundle.json \
        -o system.index.json
    @echo "✅ Federated architecture composed"

# Clean build artifacts
clean:
    @echo "Cleaning build artifacts..."
    rm -rf target/ Cargo.lock bin/
    @echo "✅ Clean complete!"

# Show help
help:
    @just --list
