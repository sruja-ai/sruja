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

# Test context engineering (why command E2E)
test-arch-intel:
    @echo "Testing context engineering (why E2E)..."
    cargo test -p sruja-cli --test why_e2e
    @echo "✅ Context engineering tests passed"

# Test extraction CLI (lint --format json, discover --format json)
test-extraction:
    @echo "Testing extraction CLI (lint/discover JSON)..."
    cargo test -p sruja-cli --test extraction_cli
    @echo "✅ Extraction CLI tests passed"

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

# Install Rust dependencies (fetch only)
install:
    @echo "Installing Rust dependencies..."
    cargo fetch
    @echo "✅ Dependencies installed"

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

# Build extension and reveal VSIX (for Trae manual install)
deploy-trae: build-extension
    @VSIX=$$(ls -t extension/sruja-*.vsix 2>/dev/null | head -1); \
    if [ -z "$$VSIX" ]; then \
        echo "❌ No .vsix found. Run 'just build-extension' first."; exit 1; \
    fi; \
    if command -v open >/dev/null 2>&1; then open -R "$$VSIX" || true; fi; \
    echo ""; \
    echo "✅ VSIX ready: $$VSIX"; \
    echo ""; \
    echo "Trae: Extensions → ... → Install from VSIX → select $$VSIX"; \
    echo "Or drag-and-drop the VSIX into the Extensions panel."

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

# --- Book (mdBook) ---

book-build:
    @echo "Building book..."
    @if command -v mdbook >/dev/null 2>&1; then \
        (cd book && mdbook build); \
        echo "✅ Book built (book/book/)"; \
    else \
        echo "❌ mdbook not found. Run: just book-deps"; exit 1; \
    fi

book-wasm:
    @echo "Copying WASM into book output..."
    @book/copy-wasm.sh || echo "⚠️  Run 'just wasm' first if you need Sruja diagrams"
    @echo "✅ WASM copied"

book: book-build book-wasm
    @echo "✅ Book ready (output: book/book/)"

book-serve: wasm
    @echo "Serving book at http://localhost:3000 (live reload)..."
    @book/serve.sh

book-deps:
    @echo "Installing mdbook and mdbook-mermaid..."
    cargo install mdbook mdbook-mermaid
    (cd book && mdbook-mermaid install .)
    @echo "✅ Book dependencies installed"

book-clean:
    rm -rf book/book
    @echo "✅ Book output removed"

book-lint-examples:
    @echo "Linting book/valid-examples/*.sruja..."
    @for f in book/valid-examples/*.sruja; do \
        sruja lint "$$f" || exit 1; \
    done
    @echo "✅ All valid-examples pass sruja lint"

# --- Assets ---

assets:
    @echo "Copying assets to correct locations..."
    @if [ -f "extension/sruja-logo.png" ]; then \
        if [ -d "crates/sruja-wasm/pkg" ]; then \
            cp extension/sruja-logo.png crates/sruja-wasm/pkg/; \
            echo "  ✅ sruja-logo.png → crates/sruja-wasm/pkg/"; \
        fi; \
    fi
    @echo "✅ Assets copied"

# --- Demos ---

demo:
    @echo "Running Sruja E2E demo..."
    @if [ -f "evaluation/real-world-test/run_demo.sh" ]; then \
        cd evaluation/real-world-test && ./run_demo.sh; \
    else \
        echo "❌ evaluation/real-world-test/run_demo.sh not found"; exit 1; \
    fi

demo-intel:
    @echo "Running Context Engineering demo..."
    @if [ -f "demo/run_demo.sh" ]; then \
        cd demo && ./run_demo.sh; \
    else \
        echo "❌ demo/run_demo.sh not found"; exit 1; \
    fi

# Clean build artifacts
clean:
    @echo "Cleaning build artifacts..."
    rm -rf target/ Cargo.lock bin/
    @echo "✅ Clean complete!"

# Show help
help:
    @just --list
