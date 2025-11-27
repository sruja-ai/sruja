.PHONY: build test clean install lint fmt

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

# Clean build artifacts
clean:
	@rm -rf bin/

# Install dependencies
install:
	@echo "Installing Go dependencies..."
	@go mod download


# Run linter
lint:
	@echo "Running linter..."
	@command -v $(GOLANGCI) >/dev/null 2>&1 || { \
		echo "golangci-lint not found. Installing..."; \
		curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh \
			| sh -s -- -b $$(go env GOPATH)/bin $(GOLANGCI_LINT_VERSION); \
	}
	@$(GOLANGCI) run

# Format code
fmt:
	@echo "Formatting code..."
	@go fmt ./...
