.PHONY: build test clean install

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

