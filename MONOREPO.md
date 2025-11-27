# Monorepo Structure

This project uses **Turborepo** with **Bun** to manage a monorepo containing both Go and Node.js applications.

## Structure

```
sruja/
├── apps/
│   ├── cli/                    # Main CLI tool (Go)
│   │   ├── cmd/
│   │   │   ├── main.go
│   │   │   └── extensions.go
│   │   └── package.json
│   ├── playground-server/      # Playground API server (Go)
│   │   ├── main.go
│   │   └── package.json
│   └── playground-web/         # Playground frontend (React/Vite)
│       ├── src/
│       ├── index.html
│       ├── vite.config.ts
│       └── package.json
├── pkg/                        # Shared Go packages
│   ├── compiler/
│   ├── engine/
│   ├── language/
│   └── ...
├── packages/                   # Shared JS/TS packages (future, currently empty)
├── bin/                        # Build outputs
├── package.json                # Root package.json (Turborepo)
├── turbo.json                  # Turborepo configuration
├── go.mod                      # Go module
└── Makefile                    # Convenience Make targets
```

## Getting Started

### Prerequisites

- Go 1.25+
- Bun 1.3+ (includes Node.js runtime)

### Installation

```bash
# Install Bun if not already installed
curl -fsSL https://bun.sh/install | bash

# Install all dependencies
bun install

# Or use Make
make install
```

## Development

### Using Turborepo (Recommended)

```bash
# Run all apps in development mode
bun dev

# Build all apps
bun build

# Run tests
bun test

# Clean build artifacts
bun clean
```

### Using Make

```bash
# Build all Go applications
make build

# Run development mode (playground server + web)
make dev

# Run tests
make test

# Clean artifacts
make clean
```

### Individual Apps

#### CLI

```bash
cd apps/cli
go run ./cmd compile examples/simple.sruja
```

#### Playground Server

```bash
cd apps/playground-server

# With hot reload (requires air: go install github.com/cosmtrek/air@latest)
bun run dev
# or
air

# Without hot reload
go run main.go
# Server runs on :8080
```

**Hot Reloading**: The playground server supports hot reloading using [Air](https://github.com/air-verse/air). Install it with:
```bash
go install github.com/air-verse/air@latest
```

Then use `bun run dev` or `air` to run with automatic reloading on file changes.

#### Playground Web

```bash
cd apps/playground-web
bun dev
# Frontend runs on :3000
```

## Turborepo Configuration

The `turbo.json` file defines the build pipeline:

- **build**: Builds all apps, respecting dependencies
- **dev**: Development mode (no caching, persistent)
- **test**: Runs tests after build
- **lint**: Linting (no outputs)
- **clean**: Cleans build artifacts

## Workspace Management

Turborepo uses Bun workspaces (compatible with npm/pnpm workspace format) defined in the root `package.json`:

```json
{
  "workspaces": [
    "apps/*"
  ]
}
```

This allows:
- Shared dependencies at the root
- Independent versioning per app
- Efficient dependency hoisting

**Note**: Currently only `apps/*` is used. The `packages/` directory is reserved for future shared JavaScript/TypeScript packages. Go shared code lives in `pkg/` (standard Go convention).

## Build Outputs

- Go binaries: `bin/sruja`, `bin/playground-server`
- Web build: `apps/playground-web/dist/`

## CI/CD

Turborepo provides:
- **Caching**: Build outputs cached locally and remotely
- **Parallel execution**: Tasks run in parallel when possible
- **Dependency graph**: Only rebuilds what changed

Example GitHub Actions:

```yaml
- uses: oven-sh/setup-bun@v1
- uses: actions/setup-go@v4
- run: bun install
- run: bun build
```

## Adding New Apps

1. Create directory in `apps/` or `packages/`
2. Add `package.json` with name `@sruja/<name>`
3. Turborepo will automatically detect it

## Go Module

The Go module (`go.mod`) is at the root. All Go code shares the same module:

```go
module github.com/sruja-ai/sruja
```

Import paths:
```go
import "github.com/sruja-ai/sruja/pkg/language"
```

