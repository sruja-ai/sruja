.PHONY: build test test-coverage test-coverage-html clean install lint fmt wasm build-docs setup-hooks

GOLANGCI_LINT_VERSION = v2.6.2
GOLANGCI = $(shell go env GOPATH)/bin/golangci-lint

# Build CLI
build:
	@echo "Building CLI..."
	@mkdir -p bin
	@go build -o bin/sruja ./cmd/sruja

# Run tests
test:
	@echo "Running Go tests..."
	@go test ./pkg/... ./cmd/...

# Run tests with coverage
test-coverage:
	@echo "Running Go tests with coverage..."
	@go test -coverprofile=coverage.out ./pkg/... ./cmd/...
	@echo ""
	@echo "Coverage summary:"
	@go tool cover -func=coverage.out | tail -1

# Generate HTML coverage report
test-coverage-html: test-coverage
	@echo "Generating HTML coverage report..."
	@go tool cover -html=coverage.out -o coverage.html
	@echo "Coverage report saved to coverage.html"

# Clean build artifacts
clean:
	@rm -rf bin/

# Install dependencies
install:
	@echo "Installing Go dependencies..."
	@go mod download

# Build WASM
wasm:
	@echo "Building WASM..."
	@mkdir -p learn/static
	@GOOS=js GOARCH=wasm go build -ldflags="-s -w" -trimpath -o learn/static/sruja.wasm ./cmd/wasm
	@cp $$(go env GOROOT)/lib/wasm/wasm_exec.js learn/static/
	@if command -v wasm-opt >/dev/null 2>&1; then \
		echo "Optimizing WASM with wasm-opt..."; \
		wasm-opt -Oz learn/static/sruja.wasm -o learn/static/sruja.wasm; \
	fi
	@SIZE=$$(ls -lh learn/static/sruja.wasm | awk '{print $$5}'); \
	echo "WASM built successfully: learn/static/sruja.wasm ($$SIZE)"; \
	echo ""; \
	echo "Note: Large file size (~23MB) is due to D2 rendering library dependencies."; \
	echo "With gzip compression: ~7MB (70% reduction)"; \
	echo "Enable compression on your web server for better performance."

# Test playground, course, and docs code compilation
test-learn-code:
	@echo "Testing playground, course, and docs code compilation..."
	@go test -v -run "TestPlaygroundExamples|TestCourseCodeBlocks|TestDocsCodeBlocks" || (echo "❌ Code compilation tests failed! Fix errors before deploying." && exit 1)
	@echo "✅ All code examples compile successfully"

# Build WASM for docs website (includes compression)
build-docs: test-learn-code wasm
	@echo "Creating compressed versions..."
	@gzip -k -f learn/static/sruja.wasm 2>/dev/null || true
	@if command -v brotli >/dev/null 2>&1; then \
		brotli -k -f learn/static/sruja.wasm 2>/dev/null || true; \
	fi
	@echo "WASM ready for docs website"
	@echo ""
	@echo "Files created:"
	@ls -lh learn/static/sruja.wasm* 2>/dev/null | awk '{print "  " $$9 " (" $$5 ")"}'
	@echo ""
	@echo "Optimization tips:"
	@echo "  - Configure web server to serve .wasm.gz with Content-Encoding: gzip"
	@echo "  - Consider lazy-loading WASM only when playground is used"
	@echo "  - The large size is primarily from D2 rendering library"

# Run linter
lint: $(GOLANGCI)
	@echo "Running linter..."
	@$(GOLANGCI) run

$(GOLANGCI):
	@command -v $(GOLANGCI) >/dev/null 2>&1 || { \
		echo "golangci-lint not found. Installing..."; \
		curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh \
			| sh -s -- -b $$(go env GOPATH)/bin $(GOLANGCI_LINT_VERSION); \
	}

# Format code
fmt:
	@echo "Formatting code..."
	@go fmt ./...

# Setup git hooks
setup-hooks:
	@echo "Setting up git hooks..."
	@./scripts/setup-git-hooks.sh
