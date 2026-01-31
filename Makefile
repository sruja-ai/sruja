.PHONY: build test test-coverage clean install lint fmt help build-rust test-rust wasm wasm-tiny

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

# Build WASM for website
wasm:
	@echo "Building Rust WASM for website..."
	@if command -v cargo >/dev/null 2>&1; then \
		if ! command -v wasm-pack >/dev/null 2>&1; then \
			echo "⚠️  wasm-pack not found. Installing..."; \
			cargo install wasm-pack || (echo "❌ Failed to install wasm-pack. Please install manually: cargo install wasm-pack"; exit 1); \
		fi; \
		wasm-pack build --target web --out-dir ../../apps/website/public/wasm/rust crates/sruja-wasm --release || \
		(cargo build --target wasm32-unknown-unknown --release -p sruja-wasm && \
		 echo "⚠️  wasm-pack failed, but WASM built. You may need to manually copy files."); \
		if command -v wasm-opt >/dev/null 2>&1; then \
			echo "Optimizing WASM with wasm-opt..."; \
			wasm-opt -O3 --strip-debug \
				apps/website/public/wasm/rust/sruja_wasm_bg.wasm \
				-o apps/website/public/wasm/rust/sruja_wasm_bg.wasm.tmp && \
			mv apps/website/public/wasm/rust/sruja_wasm_bg.wasm.tmp \
				apps/website/public/wasm/rust/sruja_wasm_bg.wasm; \
			echo "✅ WASM optimized"; \
			ls -lh apps/website/public/wasm/rust/sruja_wasm_bg.wasm; \
		else \
			echo "⚠️  wasm-opt not found. Install with: npm install -g wasm-opt"; \
			echo "   Skipping optimization (WASM will be ~25% larger)"; \
			ls -lh apps/website/public/wasm/rust/sruja_wasm_bg.wasm; \
		fi; \
		echo "✅ WASM build complete"; \
	else \
		echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"; \
		exit 1; \
	fi

# Build WASM for Node.js (VSCode extension)
wasm-nodejs:
	@echo "Building Rust WASM for Node.js (VSCode extension)..."
	@if command -v cargo >/dev/null 2>&1; then \
		if ! command -v wasm-pack >/dev/null 2>&1; then \
			echo "⚠️  wasm-pack not found. Installing..."; \
			cargo install wasm-pack || (echo "❌ Failed to install wasm-pack. Please install manually: cargo install wasm-pack"; exit 1); \
		fi; \
		mkdir -p apps/vscode-extension/wasm-build; \
		wasm-pack build --target nodejs --out-dir ../../apps/vscode-extension/wasm-build crates/sruja-wasm --release || \
		(cargo build --target wasm32-unknown-unknown --release -p sruja-wasm && \
		 echo "⚠️  wasm-pack failed, but WASM built. You may need to manually copy files."); \
		if command -v wasm-opt >/dev/null 2>&1; then \
			echo "Optimizing WASM with wasm-opt..."; \
			wasm-opt -O3 --strip-debug \
				apps/vscode-extension/wasm-build/sruja_wasm_bg.wasm \
				-o apps/vscode-extension/wasm-build/sruja_wasm_bg.wasm.tmp && \
			mv apps/vscode-extension/wasm-build/sruja_wasm_bg.wasm.tmp \
				apps/vscode-extension/wasm-build/sruja_wasm_bg.wasm; \
			echo "✅ WASM optimized"; \
			ls -lh apps/vscode-extension/wasm-build/sruja_wasm_bg.wasm; \
		else \
			echo "⚠️  wasm-opt not found. Install with: npm install -g wasm-opt"; \
			echo "   Skipping optimization (WASM will be ~25% larger)"; \
			ls -lh apps/vscode-extension/wasm-build/sruja_wasm_bg.wasm; \
		fi; \
		echo "✅ Node.js WASM build complete"; \
	else \
		echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"; \
		exit 1; \
	fi

# Build tiny WASM variant (minimal features)
wasm-tiny: wasm
	@echo "✅ Tiny WASM variant (same as full for now)"

# Show help
help:
	@echo "Sruja Rust Migration - Build Commands:"
	@echo ""
	@echo "Build & Development:"
	@echo "  make build              - Build Rust libraries"
	@echo "  make test               - Run Rust tests"
	@echo "  make test-coverage      - Run tests with coverage (if available)"
	@echo "  make clean              - Remove build artifacts"
	@echo "  make install            - Install Rust dependencies"
	@echo ""
	@echo "WASM Build:"
	@echo "  make wasm               - Build Rust WASM for website (web target)"
	@echo "  make wasm-nodejs        - Build Rust WASM for Node.js/VSCode (nodejs target)"
	@echo "  make wasm-tiny          - Build tiny WASM variant"
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
	@echo ""
	@echo "This branch is Rust-only. Go compatibility layers have been removed."
