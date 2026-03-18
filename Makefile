.PHONY: build test test-coverage test-wasm test-e2e clean install lint fmt help build-rust test-rust wasm wasm-tiny book book-build book-wasm book-serve book-deps book-clean assets demo build-extension install-extension test-cli-smoke

# Build Rust libraries
build-rust:
	@echo "Building Rust libraries..."
	@if command -v cargo >/dev/null 2>&1; then \
		cargo build --release --manifest-path Cargo.toml; \
		echo "✅ Rust libraries built successfully"; \
	else \
		echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"; \
		exit 1; \
	fi

# Test Rust code (all workspace packages)
test-rust:
	@echo "Testing Rust code..."
	@if command -v cargo >/dev/null 2>&1; then \
		cargo test --workspace --manifest-path Cargo.toml; \
	else \
		echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"; \
		exit 1; \
	fi

# Test architecture intelligence (why command E2E)
test-arch-intel:
	@echo "Testing architecture intelligence (why E2E)..."
	@cargo test -p sruja-cli --test why_e2e && \
	echo "✅ Architecture intelligence tests passed"

# Test extraction CLI (lint --format json, discover --format json)
test-extraction:
	@echo "Testing extraction CLI (lint/discover JSON)..."
	@cargo test -p sruja-cli --test extraction_cli && \
	echo "✅ Extraction CLI tests passed"

# Test CLI smoke tests (validates documented command shapes)
test-cli-smoke:
	@echo "Running CLI smoke tests..."
	@./scripts/test_cli_smoke.sh

# Build (default: Rust)
build: build-rust
	@echo "✅ Build complete!"

# Run tests (default: Rust)
test: test-rust
	@echo "✅ Tests complete!"

# Run WASM unit tests (requires: wasm-pack, rustup target add wasm32-unknown-unknown)
test-wasm:
	@echo "Testing WASM (sruja-wasm)..."
	@if command -v wasm-pack >/dev/null 2>&1; then \
		(cd crates/sruja-wasm && wasm-pack test --node) && echo "✅ WASM tests passed"; \
	else \
		echo "⚠️  wasm-pack not found. Install: cargo install wasm-pack"; exit 1; \
	fi

# Run Playwright E2E test (book Show diagram). Prerequisite: make book-serve in another terminal.
test-e2e:
	@echo "Running E2E (Playwright)..."
	@if [ -f package.json ] && command -v npm >/dev/null 2>&1; then \
		npm run e2e && echo "✅ E2E tests passed"; \
	else \
		echo "⚠️  Run from repo root with Node/npm installed. Start book first: make book-serve"; exit 1; \
	fi

# Run tests with coverage (requires: cargo install cargo-llvm-cov)
test-coverage:
	@echo "Running Rust tests with coverage..."
	@if command -v cargo >/dev/null 2>&1; then \
		if cargo llvm-cov --version >/dev/null 2>&1; then \
			cargo llvm-cov --manifest-path Cargo.toml; \
		else \
			cargo test --manifest-path Cargo.toml; \
			echo ""; \
			echo "Note: Install cargo-llvm-cov for coverage reports:"; \
			echo "  rustup component add llvm-tools-preview"; \
			echo "  cargo install cargo-llvm-cov"; \
		fi; \
	else \
		echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"; \
		exit 1; \
	fi

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf target/ Cargo.lock bin/
	@echo "✅ Clean complete!"

# Install dependencies
install:
	@echo "Installing Rust dependencies..."
	@if command -v cargo >/dev/null 2>&1; then \
		cargo fetch --manifest-path Cargo.toml; \
		echo "✅ Dependencies installed"; \
	else \
		echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"; \
		exit 1; \
	fi

# Format Rust code
fmt:
	@echo "Formatting Rust code..."
	@if command -v cargo >/dev/null 2>&1; then \
		cargo fmt --manifest-path Cargo.toml; \
		echo "✅ Formatting complete!"; \
	else \
		echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"; \
		exit 1; \
	fi

# Lint Rust code
lint:
	@echo "Linting Rust code..."
	@if command -v cargo >/dev/null 2>&1; then \
		cargo clippy --manifest-path Cargo.toml -- -D warnings || \
		(cargo install clippy 2>/dev/null || echo "Note: Install clippy for linting"); \
	else \
		echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"; \
		exit 1; \
	fi

# Build WASM (web target). Output: crates/sruja-wasm/pkg/
WASM_PKG := crates/sruja-wasm/pkg
wasm:
	@echo "Building Rust WASM (web)..."
	@if command -v cargo >/dev/null 2>&1; then \
		if ! command -v wasm-pack >/dev/null 2>&1; then \
			echo "⚠️  wasm-pack not found. Install: cargo install wasm-pack"; \
			exit 1; \
		fi; \
		wasm-pack build --target web --out-dir $(WASM_PKG) crates/sruja-wasm --release; \
		if command -v wasm-opt >/dev/null 2>&1; then \
			wasm-opt --enable-bulk-memory --enable-sign-ext -Oz --strip-debug $(WASM_PKG)/sruja_wasm_bg.wasm -o $(WASM_PKG)/sruja_wasm_bg.wasm.tmp && mv $(WASM_PKG)/sruja_wasm_bg.wasm.tmp $(WASM_PKG)/sruja_wasm_bg.wasm; \
		fi; \
		gzip -9 -k -f $(WASM_PKG)/sruja_wasm_bg.wasm 2>/dev/null || true; \
		if command -v brotli >/dev/null 2>&1; then brotli -q 11 -k -f $(WASM_PKG)/sruja_wasm_bg.wasm; fi; \
		echo "✅ WASM build complete ($(WASM_PKG)/)"; \
	else \
		echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"; exit 1; \
	fi

# Build WASM for Node.js (used by VS Code extension for in-editor lint and markdown export)
wasm-nodejs:
	@echo "Building Rust WASM (nodejs target)..."
	@if command -v cargo >/dev/null 2>&1; then \
		if ! command -v wasm-pack >/dev/null 2>&1; then \
			echo "⚠️  wasm-pack not found. Install: cargo install wasm-pack"; exit 1; \
		fi; \
		wasm-pack build --target nodejs --out-dir crates/sruja-wasm/pkg-nodejs crates/sruja-wasm --release; \
		echo "✅ Node.js WASM build complete (crates/sruja-wasm/pkg-nodejs/)"; \
	else \
		echo "❌ Cargo not found."; exit 1; \
	fi

# Build VS Code extension VSIX package (WASM + TypeScript compile)
# Output: extension/sruja-*.vsix
build-extension:
	@echo "Building Sruja VS Code extension..."
	@if ! command -v npm >/dev/null 2>&1; then \
		echo "❌ npm not found. Please install Node.js: https://nodejs.org/"; exit 1; \
	fi
	@echo "  📦 Installing extension npm deps..."
	@cd extension && npm install --silent
	@echo "  🔧 Building Node.js WASM + copying assets..."
	@cd extension && npm run copy:assets
	@echo "  🔨 Compiling TypeScript..."
	@cd extension && npm run compile
	@echo "  📦 Packaging VSIX..."
	@cd extension && npx --yes @vscode/vsce package --no-dependencies
	@echo "✅ Extension built: $$(ls extension/sruja-*.vsix 2>/dev/null | tail -1)"

# Build and install VS Code extension into VS Code and/or Cursor
# Detects available editors automatically.
install-extension: build-extension
	@VSIX="$$(ls extension/sruja-*.vsix 2>/dev/null | tail -1)"; \
	if [ -z "$$VSIX" ]; then \
		echo "❌ No .vsix found. Run 'make build-extension' first."; exit 1; \
	fi; \
	INSTALLED=0; \
	if command -v cursor >/dev/null 2>&1; then \
		echo "  🖱️  Installing into Cursor..."; \
		cursor --install-extension "$$VSIX" && INSTALLED=1 && echo "  ✅ Installed in Cursor"; \
	fi; \
	if command -v code >/dev/null 2>&1; then \
		echo "  💻 Installing into VS Code..."; \
		code --install-extension "$$VSIX" && INSTALLED=1 && echo "  ✅ Installed in VS Code"; \
	fi; \
	if [ "$$INSTALLED" -eq 0 ]; then \
		echo ""; \
		echo "⚠️  Neither 'cursor' nor 'code' CLI found in PATH."; \
		echo "   Install manually: Extensions → ⋯ → Install from VSIX → select $$VSIX"; \
	else \
		echo ""; \
		echo "✅ Extension installed! Reload your editor window to activate it."; \
		echo "   Open any .sruja file to see diagnostics, hover docs, and diagram preview."; \
	fi

# --- Book (mdBook) ---
BOOK_DIR := book

book-build:
	@echo "Building book..."
	@if command -v mdbook >/dev/null 2>&1; then \
		(cd $(BOOK_DIR) && mdbook build); \
		echo "✅ Book built ($(BOOK_DIR)/book/)"; \
	else \
		echo "❌ mdbook not found. Run: make book-deps"; exit 1; \
	fi

book-wasm:
	@echo "Copying WASM into book output..."
	@$(BOOK_DIR)/copy-wasm.sh || echo "⚠️  Run 'make wasm' first if you need Sruja diagrams"
	@echo "✅ WASM copied"

book: book-build book-wasm
	@echo "✅ Book ready (output: $(BOOK_DIR)/book/)"

book-serve: wasm
	@echo "Serving book at http://localhost:3000 (live reload)..."
	@$(BOOK_DIR)/serve.sh

book-deps:
	@echo "Installing mdbook and mdbook-mermaid..."
	@cargo install mdbook mdbook-mermaid
	@(cd $(BOOK_DIR) && mdbook-mermaid install .)
	@echo "✅ Book dependencies installed"

book-clean:
	@rm -rf $(BOOK_DIR)/book
	@echo "✅ Book output removed"

book-lint-examples:
	@echo "Linting book/valid-examples/*.sruja..."
	@for f in $(BOOK_DIR)/valid-examples/*.sruja; do \
		sruja lint "$$f" || exit 1; \
	done
	@echo "✅ All valid-examples pass sruja lint"

# --- Assets ---
# Copy assets to correct locations (logo, icons, etc.)
assets:
	@echo "Copying assets to correct locations..."
	@if [ -f "extension/sruja-logo.png" ]; then \
		if [ -d "crates/sruja-wasm/pkg" ]; then \
			cp extension/sruja-logo.png crates/sruja-wasm/pkg/; \
			echo "  ✅ sruja-logo.png → crates/sruja-wasm/pkg/"; \
		fi; \
	fi
	@echo "✅ Assets copied"

# Run E2E value demo (quickstart + drift on Express; optional --baseline, --llm)
demo:
	@echo "Running Sruja E2E demo..."
	@if [ -f "evaluation/real-world-test/run_demo.sh" ]; then \
		cd evaluation/real-world-test && ./run_demo.sh; \
	else \
		echo "❌ evaluation/real-world-test/run_demo.sh not found"; exit 1; \
	fi

# Run Architecture Intelligence demo (intent → scan → drift → analyze → AI ask)
# Uses demo/ microservices + architecture.sruja. Optional: set LLM key for AI step.
demo-intel:
	@echo "Running Architecture Intelligence demo..."
	@if [ -f "demo/run_demo.sh" ]; then \
		cd demo && ./run_demo.sh; \
	else \
		echo "❌ demo/run_demo.sh not found"; exit 1; \
	fi

# Show help
help:
	@echo "Sruja - Build Commands"
	@echo ""
	@echo "Build & Development:"
	@echo "  make build              - Build Rust libraries"
	@echo "  make test               - Run Rust tests"
	@echo "  make test-wasm          - Run WASM unit tests (wasm-pack test --node)"
	@echo "  make test-e2e           - Run Playwright E2E (book Show diagram); start book-serve first"
	@echo "  make test-coverage      - Run tests with coverage (if available)"
	@echo "  make clean              - Remove build artifacts"
	@echo "  make install            - Install Rust dependencies"
	@echo "  make assets             - Copy assets (logos, icons) to correct locations"
	@echo ""
	@echo "Book (mdBook):"
	@echo "  make book-deps          - Install mdbook, mdbook-mermaid, copy Mermaid assets"
	@echo "  make book               - Build book + copy WASM (run 'make wasm' once for diagrams)"
	@echo "  make book-serve         - Serve book at http://localhost:3000 (live reload)"
	@echo "  make book-clean         - Remove book/book/ output"
	@echo ""
	@echo "WASM Build:"
	@echo "  make wasm               - Build Rust WASM (web target, crates/sruja-wasm/pkg/)"
	@echo "  make wasm-nodejs        - Build Rust WASM for Node (nodejs target, used by extension)"
	@echo ""
	@echo "VS Code / Cursor Extension:"
	@echo "  make build-extension    - Build WASM + compile TS + package .vsix"
	@echo "  make install-extension  - Build and install into VS Code / Cursor (auto-detected)"
	@echo ""
	@echo "Code Quality:"
	@echo "  make lint               - Run Rust linter (clippy)"
	@echo "  make fmt                - Format Rust code"
	@echo ""
	@echo "Architecture Intelligence:"
	@echo "  make test-arch-intel    - Run architecture intelligence E2E (why command)"
	@echo "  make test-cli-smoke     - Run CLI smoke tests (validates documented command shapes)"
	@echo "  make demo               - Run E2E value demo (quickstart + drift on sample repo)"
	@echo "  make demo-intel         - Run Architecture Intelligence demo (intent → scan → drift → analyze → AI)"
	@echo ""
	@echo "Direct Cargo Commands:"
	@echo "  cargo build --release   - Build release version"
	@echo "  cargo test              - Run all tests"
	@echo "  cargo test --lib        - Run library tests only"
	@echo "  cargo clippy            - Run linter"
	@echo "  cargo fmt --check       - Check formatting"
