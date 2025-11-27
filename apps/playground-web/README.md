# Sruja Playground Web

Interactive web playground for trying out the Sruja language.

## Features

- Live code editor with Monaco Editor
- Real-time validation
- Mermaid diagram preview
- Example templates

## Development

### Prerequisites

- Bun 1.3+ (includes Node.js runtime)

### Running Locally

1. Install dependencies:
```bash
bun install
```

2. Start the playground server (from root or apps/playground-server):
```bash
cd ../../apps/playground-server
go run main.go
# Or use turbo
bun --filter @sruja/playground-server dev
```

3. Start the web frontend:
```bash
bun dev
# Or use turbo from root
bun --filter @sruja/playground-web dev
```

4. Open http://localhost:3000

### Using Turborepo

From the root directory:

```bash
# Start both server and web in dev mode
bun dev

# Build everything
bun build

# Run tests
bun test
```

## Architecture

- `apps/playground-server`: Go HTTP server with `/api/compile` and `/api/validate` endpoints
- `apps/playground-web`: React + Vite frontend with Monaco editor and Mermaid rendering
