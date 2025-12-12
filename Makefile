.PHONY: build build-embed-viewer test test-coverage test-coverage-html test-coverage-check clean install lint fmt wasm wasm-tiny wasm-select setup-hooks generate-svgs generate-svgs-all generate-svgs-examples generate-svgs-containers generate-svgs-complete clean-svgs security-scan lint-security check-unused help help-svgs

GOLANGCI_LINT_VERSION = v1.62.2
GOLANGCI = $(shell go env GOPATH)/bin/golangci-lint

# Build CLI
build:
	@echo "Building CLI..."
	@mkdir -p bin
	@go build -o bin/sruja ./cmd/sruja

# Build CLI only (alias for build)
build-cli-only:
	@echo "Building CLI..."
	@mkdir -p bin
	@go build -o bin/sruja ./cmd/sruja

# Build embed-viewer IIFE bundle for HTML export (if HTML export is re-enabled)
# Note: HTML export is currently removed from CLI, but code exists in pkg/export/html
# Only rebuilds if source files have changed
EMBED_VIEWER_OUT := pkg/export/html/embed/embed-viewer.js
VIEWER_SOURCES := $(shell find packages/viewer/src -name '*.ts' -o -name '*.tsx' 2>/dev/null)
EMBED_SOURCES := $(shell find apps/viewer-core/app -name '*.ts' -o -name '*.tsx' 2>/dev/null)

build-embed-viewer: $(EMBED_VIEWER_OUT)

$(EMBED_VIEWER_OUT): $(VIEWER_SOURCES) $(EMBED_SOURCES)
	@echo "Building embed-viewer IIFE bundle (sources changed)..."
	@cd packages/viewer && npm run build
	@cd apps/viewer-core && npm run build:embed
	@mkdir -p pkg/export/html/embed
	@cp apps/viewer-core/dist/embed-viewer.iife.js pkg/export/html/embed/embed-viewer.js 2>/dev/null || echo "Warning: embed-viewer.iife.js not found"
	@cp apps/viewer-core/dist/embed-viewer.css pkg/export/html/embed/embed-viewer.css 2>/dev/null || echo "Warning: embed-viewer.css not found"
	@echo "✅ Embed viewer bundle built"

# Force rebuild of embed-viewer (ignores timestamps) - calls build-embed-viewer after removing dependency
build-embed-viewer-force:
	@echo "Force rebuilding embed-viewer IIFE bundle..."
	@rm -f $(EMBED_VIEWER_OUT)
	@$(MAKE) build-embed-viewer

# Run tests
test:
	@echo "Running Go tests..."
	@go test ./...

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

# Build WASM (full version with parsing, validation, LSP features, and compression)
wasm:
	@echo "Building WASM (full version with compression)..."
	@mkdir -p apps/website/public/wasm
	@GOOS=js GOARCH=wasm go build -ldflags="-s -w" -trimpath -o apps/website/public/wasm/sruja.wasm ./cmd/wasm
	@cp $$(go env GOROOT)/lib/wasm/wasm_exec.js apps/website/public/wasm/
	@if command -v wasm-opt >/dev/null 2>&1; then \
		echo "Optimizing WASM with wasm-opt (bulk-memory enabled)..."; \
		wasm-opt --enable-bulk-memory -Oz apps/website/public/wasm/sruja.wasm -o apps/website/public/wasm/sruja.wasm; \
	fi
	@echo "Creating compressed versions (gzip and brotli)..."
	@gzip -k -f apps/website/public/wasm/sruja.wasm 2>/dev/null || true
	@if command -v brotli >/dev/null 2>&1; then \
		brotli -k -f apps/website/public/wasm/sruja.wasm 2>/dev/null || true; \
	fi
	@SIZE=$$(ls -lh apps/website/public/wasm/sruja.wasm | awk '{print $$5}'); \
	echo "WASM built successfully: apps/website/public/wasm/sruja.wasm ($$SIZE)"; \
	echo ""; \
	ORIG_SIZE=$$(stat -f%z apps/website/public/wasm/sruja.wasm 2>/dev/null || stat -c%s apps/website/public/wasm/sruja.wasm 2>/dev/null); \
	if [ -f apps/website/public/wasm/sruja.wasm.gz ]; then \
		GZ_SIZE=$$(stat -f%z apps/website/public/wasm/sruja.wasm.gz 2>/dev/null || stat -c%s apps/website/public/wasm/sruja.wasm.gz 2>/dev/null); \
		GZ_RATIO=$$(echo "scale=1; (1 - $$GZ_SIZE / $$ORIG_SIZE) * 100" | bc 2>/dev/null || echo "N/A"); \
		echo "Compression ratios:"; \
		echo "  gzip: $$GZ_RATIO% reduction"; \
	fi; \
	if [ -f apps/website/public/wasm/sruja.wasm.br ]; then \
		BR_SIZE=$$(stat -f%z apps/website/public/wasm/sruja.wasm.br 2>/dev/null || stat -c%s apps/website/public/wasm/sruja.wasm.br 2>/dev/null); \
		BR_RATIO=$$(echo "scale=1; (1 - $$BR_SIZE / $$ORIG_SIZE) * 100" | bc 2>/dev/null || echo "N/A"); \
		if [ -z "$$GZ_RATIO" ]; then echo "Compression ratios:"; fi; \
		echo "  brotli: $$BR_RATIO% reduction"; \
	fi; \
	echo ""; \
	echo "Files created:"; \
	ls -lh apps/website/public/wasm/sruja.wasm* 2>/dev/null | awk '{print "  " $$9 " (" $$5 ")"}'


# Generate playground examples from .sruja files
generate-examples:
	@echo "Generating playground examples..."
	@go run scripts/generate-playground-examples.go

# Build WASM for website (compression included by default in wasm target)
build-wasm-compressed: wasm
	@echo "WASM ready for website (compression already included)"

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

# Content management commands
content-course:
	@go run scripts/content-generator/main.go course $(NAME)

content-module:
	@go run scripts/content-generator/main.go module $(COURSE) $(NAME)

content-lesson:
	@go run scripts/content-generator/main.go lesson $(COURSE) $(MODULE) $(NAME)

content-tutorial:
	@go run scripts/content-generator/main.go tutorial $(NAME)

content-blog:
	@go run scripts/content-generator/main.go blog $(NAME)

content-doc:
	@go run scripts/content-generator/main.go doc $(NAME)

content-validate:
	@echo "Validating content..."
	@go run scripts/content-validator/main.go

# Check test coverage meets threshold (80%)
test-coverage-check: test-coverage
	@echo "Checking coverage threshold..."
	@COVERAGE=$$(go tool cover -func=coverage.out | tail -1 | awk '{print $$3}' | sed 's/%//'); \
	THRESHOLD=80; \
	if [ $$(echo "$$COVERAGE < $$THRESHOLD" | bc) -eq 1 ]; then \
		echo "❌ Coverage $$COVERAGE% is below threshold $$THRESHOLD%"; \
		exit 1; \
	else \
		echo "✅ Coverage $$COVERAGE% meets threshold $$THRESHOLD%"; \
	fi

# Run security vulnerability scan
security-scan:
	@echo "Running security vulnerability scan..."
	@command -v govulncheck > /dev/null 2>&1 || { \
		echo "govulncheck not found. Installing..."; \
		go install golang.org/x/vuln/cmd/govulncheck@latest; \
	}
	@govulncheck ./...

# Run security linter (gosec)
GOSEC = $(shell go env GOPATH)/bin/gosec
lint-security:
	@echo "Running security linter..."
	@command -v $(GOSEC) > /dev/null 2>&1 || { \
		echo "gosec not found. Installing..."; \
		go install github.com/securego/gosec/v2/cmd/gosec@latest; \
	}
	@$(GOSEC) -quiet ./...

# Check for unused code
check-unused:
	@echo "Checking for unused code..."
	@./scripts/check-unused-code.sh

# Run all quality checks
quality: lint test-coverage-check security-scan check-unused
	@echo "✅ All quality checks passed!"

# Generate SVG files for all example .sruja files
generate-svgs: build
	@echo "Generating SVGs for all example files..."
	@mkdir -p test-outputs
	@for file in examples/*.sruja; do \
		if [ -f "$$file" ]; then \
			name=$$(basename "$$file" .sruja); \
			echo "Generating SVG for $$name..."; \
			./bin/sruja export svg "$$file" > "test-outputs/$$name.svg" 2>&1 || echo "Failed: $$name"; \
		fi \
	done
	@echo "SVG generation complete. Check test-outputs/ directory."

# Generate SVGs for all examples (system view)
generate-svgs-all: build
	@echo "Generating SVGs for all examples (flat output: C4 L1/L2/L3, All, Deployment)..."
	@mkdir -p examples-svg
	@for file in examples/*.sruja; do \
		if [ -f "$$file" ]; then \
			name=$$(basename "$$file" .sruja); \
			echo "  [$$name] C4:L1"; \
			./bin/sruja export -view=c4:l1 svg "$$file" > "examples-svg/$${name}.c4-l1.svg" 2>/dev/null || echo "    ⚠️  L1 failed"; \
			echo "  [$$name] All"; \
			./bin/sruja export -view=all svg "$$file" > "examples-svg/$${name}.all.svg" 2>/dev/null || echo "    ⚠️  All failed"; \
			sys_ids=$$(awk '/^[[:space:]]*system[[:space:]]+/ {print $$2}' "$$file" | tr -d '"'); \
			for sys in $$sys_ids; do \
				echo "    [$$name] C4:L2 $$sys"; \
				./bin/sruja export -view=c4:l2:$$sys svg "$$file" > "examples-svg/$${name}.c4-l2.$${sys}.svg" 2>/dev/null || echo "      ⚠️  L2 failed: $$sys"; \
				cont_ids=$$(awk '/^[[:space:]]*container[[:space:]]+/ {print $$2}' "$$file" | tr -d '"'); \
				for cont in $$cont_ids; do \
					./bin/sruja export -view=c4:l3:$$sys/$$cont svg "$$file" >/dev/null 2>&1 && \
					{ echo "      [$$name] C4:L3 $$sys/$$cont"; ./bin/sruja export -view=c4:l3:$$sys/$$cont svg "$$file" > "examples-svg/$${name}.c4-l3.$${sys}.$${cont}.svg" 2>/dev/null; } || \
					true; \
				done; \
			done; \
			dep_ids=$$(awk '/^[[:space:]]*(deployment|deploymentNode)[[:space:]]+/ {print $$2}' "$$file" | tr -d '"'); \
			for dep in $$dep_ids; do \
				echo "    [$$name] Deployment $$dep"; \
				./bin/sruja export -view=deployment:$$dep svg "$$file" > "examples-svg/$${name}.deployment.$${dep}.svg" 2>/dev/null || echo "      ⚠️  Deployment failed: $$dep"; \
			done; \
		fi; \
		done
	@echo ""
	@echo "✅ SVG generation complete!"
	@echo "   Output directory: examples-svg/ (flat files: *.c4-l1.svg, *.c4-l2.<sys>.svg, *.c4-l3.<sys>.<cont>.svg, *.all.svg, *.deployment.<id>.svg)"

# Generate SVGs for examples (alias)
generate-svgs-examples: generate-svgs-all

# Generate SVGs with container views (for files with systems)
generate-svgs-containers: build
	@echo "Generating SVGs with container views..."
	@mkdir -p examples-svg/containers
	@for file in examples/*.sruja; do \
		if [ -f "$$file" ]; then \
			name=$$(basename "$$file" .sruja); \
			# Try to extract first system ID (basic approach) \
			sys_id=$$(grep -m 1 "^system" "$$file" | sed 's/system \([^ ]*\).*/\1/' | tr -d '"' || echo ""); \
			if [ -n "$$sys_id" ]; then \
				echo "  Generating container view for $$name (system: $$sys_id)..."; \
				./bin/sruja export -view=container:$$sys_id svg "$$file" > "examples-svg/containers/$$name.svg" 2>&1 || echo "    ⚠️  Failed: $$name"; \
			fi \
		fi \
	done
	@echo ""
	@echo "✅ Container view SVG generation complete!"

# Clean generated SVG files
clean-svgs:
	@echo "Cleaning generated SVG files..."
	@rm -rf examples-svg test-outputs/*.svg
	@echo "✅ Cleaned SVG files"

# Generate all SVGs (system + container views)
generate-svgs-complete: generate-svgs-all generate-svgs-containers
	@echo ""
	@echo "🎉 All SVG generation complete!"
	@echo "   System views: examples-svg/*.svg"
	@echo "   Container views: examples-svg/containers/*.svg"

# Show help for SVG generation commands
help-svgs:
	@echo "SVG Generation Commands:"
	@echo ""
	@echo "  make generate-svgs           - Generate SVGs to test-outputs/ (legacy)"
	@echo "  make generate-svgs-all       - Generate system view SVGs for all examples"
	@echo "  make generate-svgs-examples  - Alias for generate-svgs-all"
	@echo "  make generate-svgs-containers - Generate container view SVGs"
	@echo "  make generate-svgs-complete  - Generate all views (system + container)"
	@echo "  make clean-svgs              - Remove generated SVG files"
	@echo ""
	@echo "Output directories:"
	@echo "  - examples-svg/              - System view SVGs"
	@echo "  - examples-svg/containers/   - Container view SVGs"
	@echo "  - test-outputs/             - Legacy output directory"

# Show general help
help:
	@echo "Sruja Makefile Commands:"
	@echo ""
	@echo "Build & Development:"
	@echo "  make build              - Build CLI binary"
	@echo "  make test               - Run tests"
	@echo "  make test-coverage      - Run tests with coverage"
	@echo "  make test-coverage-html - Generate HTML coverage report"
	@echo "  make clean              - Remove build artifacts"
	@echo ""
	@echo "Code Quality:"
	@echo "  make lint               - Run linter"
	@echo "  make fmt                - Format code"
	@echo "  make quality            - Run all quality checks"
	@echo "  make security-scan      - Run security vulnerability scan"
	@echo ""
	@echo "SVG Generation:"
	@echo "  make help-svgs          - Show SVG generation commands"
	@echo "  make generate-svgs-all  - Generate SVGs for all examples"
	@echo ""
	@echo "WASM (all targets include compression by default):"
	@echo "  make wasm                    - Build WASM (full version with compression)"
	@echo ""
	@echo "Note: All WASM targets include gzip and brotli compression by default."
	@echo ""
	@echo "For more SVG commands, run: make help-svgs"
