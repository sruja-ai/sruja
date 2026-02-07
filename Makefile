.PHONY: build test test-coverage clean install lint fmt help build-rust test-rust wasm wasm-tiny book book-build book-wasm book-serve book-deps book-clean

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

# Test Rust code
test-rust:
	@echo "Testing Rust code..."
	@if command -v cargo >/dev/null 2>&1; then \
		cargo test --manifest-path Cargo.toml; \
	else \
		echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"; \
		exit 1; \
	fi

# Build (default: Rust)
build: build-rust
	@echo "✅ Build complete!"

# Run tests (default: Rust)
test: test-rust
	@echo "✅ Tests complete!"

# Run tests with coverage
test-coverage:
	@echo "Running Rust tests with coverage..."
	@if command -v cargo >/dev/null 2>&1; then \
		cargo test --manifest-path Cargo.toml -- --nocapture; \
		cargo test --manifest-path Cargo.toml --features test-coverage 2>/dev/null || echo "Note: Install cargo-llvm-cov for coverage reports"; \
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
			wasm-opt -O3 --strip-debug $(WASM_PKG)/sruja_wasm_bg.wasm -o $(WASM_PKG)/sruja_wasm_bg.wasm.tmp && mv $(WASM_PKG)/sruja_wasm_bg.wasm.tmp $(WASM_PKG)/sruja_wasm_bg.wasm; \
		fi; \
		echo "✅ WASM build complete ($(WASM_PKG)/)"; \
	else \
		echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"; exit 1; \
	fi

# Build WASM for Node.js (for future VS Code extension / LSP integration)
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

# Show help
help:
	@echo "Sruja - Build Commands"
	@echo ""
	@echo "Build & Development:"
	@echo "  make build              - Build Rust libraries"
	@echo "  make test               - Run Rust tests"
	@echo "  make test-coverage      - Run tests with coverage (if available)"
	@echo "  make clean              - Remove build artifacts"
	@echo "  make install            - Install Rust dependencies"
	@echo ""
	@echo "Book (mdBook):"
	@echo "  make book-deps          - Install mdbook, mdbook-mermaid, copy Mermaid assets"
	@echo "  make book               - Build book + copy WASM (run 'make wasm' once for diagrams)"
	@echo "  make book-serve         - Serve book at http://localhost:3000 (live reload)"
	@echo "  make book-clean         - Remove book/book/ output"
	@echo ""
	@echo "WASM Build:"
	@echo "  make wasm               - Build Rust WASM (web target, crates/sruja-wasm/pkg/)"
	@echo "  make wasm-nodejs        - Build Rust WASM for Node (nodejs target, for future LSP/extension)"
	@echo ""
	@echo "Code Quality:"
	@echo "  make lint               - Run Rust linter (clippy)"
	@echo "  make fmt                - Format Rust code"
	@echo ""
	@echo "Direct Cargo Commands:"
	@echo "  cargo build --release   - Build release version"
	@echo "  cargo test              - Run all tests"
	@echo "  cargo test --lib        - Run library tests only"
	@echo "  cargo clippy            - Run linter"
	@echo "  cargo fmt --check       - Check formatting"
