.PHONY: build dev test clean install

# Build all Go applications
build:
	@echo "Building CLI..."
	@cd apps/cli && go build -o ../../bin/sruja ./cmd
	@echo "Building playground server..."
	@cd apps/playground-server && go build -o ../../bin/playground-server ./main.go
	@echo "Building kernel..."
	@cd apps/kernel && go build -o ../../bin/sruja-kernel ./main.go

# Run development mode
dev:
	@echo "Starting playground server..."
	@cd apps/playground-server && go run ./main.go &
	@echo "Starting playground web..."
	@cd apps/playground-web && bun dev

# Run tests
test:
	@echo "Running Go tests..."
	@go test ./pkg/...
	@cd apps/cli && go test ./...
	@cd apps/playground-server && go test ./...

# Clean build artifacts
clean:
	@rm -rf bin/
	@rm -rf apps/playground-web/dist
	@rm -rf apps/playground-web/.vite
	@rm -rf .turbo

# Install dependencies
install:
	@echo "Installing Bun dependencies..."
	@bun install
	@echo "Installing Go dependencies..."
	@go mod download

