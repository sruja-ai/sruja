# Sruja

**Sruja** is a next-generation architecture-as-code language with first-class support for requirements, ADRs, and extensible validation.

> **⚠️ Alpha Release (v0.1.0)**: Sruja is under active development. APIs may change. See [ROADMAP.md](ROADMAP.md) for the path to v1.0.0.

## Documentation

https://sruja.ai

## Features

- 🎯 **Architecture DSL**: Define systems, containers, components, and relations
- ✅ **Validation Engine**: Cycle detection, orphan detection, unique IDs, valid references
- 📊 **D2 Export**: Export to D2 diagrams for rendering
- 🎨 **Code Formatter**: Auto-format your architecture with `sruja fmt`
- 🌳 **Tree View**: Visualize hierarchy with `sruja tree`

## Project Structure

```
sruja/
├── cmd/
│   └── sruja/            # Main CLI tool
├── pkg/                  # Shared Go packages
│   ├── engine/           # Validation engine
│   ├── language/         # Parser, AST, lexer
│   └── export/           # Exporters (D2, etc.)
└── examples/             # Example .sruja files
```

## Installation

### Automated Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
```

### Manual Download

Download the latest release from [GitHub Releases](https://github.com/sruja-ai/sruja/releases).

### From Source

```bash
go install github.com/sruja-ai/sruja/cmd/sruja@latest
```

## Quick Start

**Create `example.sruja`:**
```sruja
architecture "My System" {
    system App "My App" {
        container Web "Web Server"
        datastore DB "Database"
    }
    person User "User"

    User -> Web "Visits"
    Web -> DB "Reads/Writes"
}
```

**Export to D2:**
```bash
sruja export d2 example.sruja
```

**Lint your code:**
```bash
sruja lint example.sruja
```

**Format your code:**
```bash
sruja fmt example.sruja
```

**View hierarchy:**
```bash
sruja tree --file example.sruja
```

## Development

### Prerequisites

- Go 1.25+

### Setup

```bash
# Install dependencies
go mod download

# Build CLI
make build
```

## Contributing

We welcome contributions of all sizes — from small doc fixes to core features.

- Start with issues and "good first issues": https://github.com/sruja-ai/sruja/labels/good%20first%20issue
- Read the [contribution guide](./docs/CONTRIBUTING.md)

### Pull Request Checklist

- Run local checks: `make test`, `make fmt`, `make lint`
- Add/update tests for new behavior
- Keep changes focused and well‑scoped
- Use Conventional Commits (e.g., `feat: …`, `fix: …`, `docs: …`)

## License

MIT
