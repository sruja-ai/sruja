# Sruja

**Sruja** is a next-generation architecture-as-code language with first-class support for requirements, ADRs, and extensible validation.

> **⚠️ Alpha Release (v0.1.0)**: Sruja is under active development. APIs may change. See [ROADMAP.md](ROADMAP.md) for the path to v1.0.0.

## Features

- 🎯 **Architecture DSL**: Define systems, containers, components, and relations
- ✅ **Validation Engine**: Cycle detection, orphan detection, unique IDs, valid references
- 📝 **Requirements & ADRs**: First-class language support
- 📊 **Compiler**: Transpile to Mermaid C4 diagrams (extensible)
- 🎨 **Code Formatter**: Auto-format your architecture with `sruja fmt`
- 📓 **Notebook Experience**: Jupyter-like Markdown integration
- 🤖 **MCP Integration**: AI agents can read and validate your architecture
- 📦 **Extension System**: Git-based packages (like Deno/Go)
- 🎮 **Playground**: Interactive web playground for trying Sruja

## Monorepo Structure

This is a Turborepo monorepo with the following structure:

```
sruja/
├── apps/
│   ├── cli/              # Main CLI tool (Go)
│   ├── playground-server/ # Playground API server (Go)
│   └── playground-web/    # Playground frontend (React/Vite)
├── pkg/                   # Shared Go packages
│   ├── compiler/          # Compiler backends
│   ├── engine/            # Validation engine
│   ├── language/          # Parser, AST, lexer
│   └── ...
└── packages/              # Shared packages (future)
```

## Installation

### Download Pre-built Binary

Download the latest release for your platform from [GitHub Releases](https://github.com/sruja-ai/sruja/releases):

```bash
# Linux (AMD64)
curl -L https://github.com/sruja-ai/sruja/releases/latest/download/sruja-v0.1.0-linux-amd64.tar.gz | tar xz

# macOS (Apple Silicon)
curl -L https://github.com/sruja-ai/sruja/releases/latest/download/sruja-v0.1.0-darwin-arm64.tar.gz | tar xz

# Move to PATH
sudo mv sruja /usr/local/bin/
```

### From Source

```bash
go install github.com/sruja-ai/sruja/cmd/sruja@latest
```

## Quick Start

**Create `example.sruja`:**
```sruja
workspace {
  model {
    system User "User"
    system API "API Service" {
      container WebApp "Web Application"
      container Database "Database"
    }
    
    User -> WebApp "Uses"
    WebApp -> Database "Reads/Writes"
  }
  
  requirements {
    R1: functional "Must handle 10k RPS"
    R2: constraint "Must use PostgreSQL"
  }
  
  adrs {
    ADR001: "Use microservices architecture"
  }
}
```

**Compile to Mermaid:**
```bash
sruja compile example.sruja
```

**Lint your code:**
```bash
sruja lint example.sruja
```

**Format your code:**
```bash
sruja fmt example.sruja
```

### Initialize a project

```
sruja init
```

- Scaffolds `architecture.sruja`, `.architecture/` with `config.json`, and `adrs/`.
- Adds `.gitignore` entries for `.architecture/cache/`, `index.json`, and optional `visual.json`.

## Development

### Prerequisites

- Go 1.25+
- Bun 1.3+ (includes Node.js runtime)

### Setup

```bash
# Install Bun if not already installed
curl -fsSL https://bun.sh/install | bash

# Install dependencies
bun install

# Build all apps
bun build

# Run development mode
bun dev
```

### Individual Apps

```bash
# CLI
cd apps/cli
go run ./cmd

# Playground Server
cd apps/playground-server
go run main.go

# Playground Web
cd apps/playground-web
bun dev
```

### Using Turborepo

```bash
# Run all apps in dev mode
bun dev

# Build all apps
bun build

# Run tests
bun test

# Clean build artifacts
bun clean
```

## Extension System

Create custom validation rules and compilers as Git packages.

**Create `sruja.json`:**
```json
{
  "name": "my-architecture",
  "version": "1.0.0",
  "type": "project",
  "imports": {
    "aws-rules": "github.com/myorg/sruja-aws-rules@v1.0.0"
  }
}
```

**Install dependencies:**
```bash
sruja install
```

Works with both public and private repositories (uses your git credentials/SSH keys).

### Supported Extension Modalities (Go CLI)

- Go-native plugins loaded in-process via stable interfaces.
- Optional JS plugins executed via subprocess (Bun/Node) with JSON I/O.
- Optional WASM modules with a stable ABI for rule evaluation.

Use `sruja.json` to declare imports and pin versions. The Go CLI resolves and executes plugins according to modality.

## MCP Integration

Start the MCP server for AI integration:
```bash
sruja mcp
```

This exposes tools (`validate`, `compile`) and resources (your `.sruja` files) to AI agents.

## VS Code Extension

Install syntax highlighting:
```bash
cp -r .vscode-extension ~/.vscode/extensions/sruja
# Reload VS Code
```

## Project Structure

- `apps/cli`: CLI tool
- `apps/playground-server`: Playground API server
- `apps/playground-web`: Playground web UI
- `pkg/language`: Lexer, Parser, AST
- `pkg/compiler`: Compiler backends (Mermaid, etc.)
- `pkg/engine`: Validation rules
- `pkg/extensions`: Extension/package system
- `pkg/notebook`: Markdown notebook runner
- `pkg/mcp`: Model Context Protocol server

## License

MIT
