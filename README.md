# Sruja

**Sruja** is a next-generation architecture-as-code language with first-class support for requirements, ADRs, and extensible validation.

> **⚠️ Alpha Release (v0.1.0)**: Sruja is under active development. APIs may change. See [ROADMAP.md](ROADMAP.md) for the path to v1.0.0.

## Features

- 🎯 **Architecture DSL**: Define systems, containers, components, and relations
- ✅ **Validation Engine**: Cycle detection, orphan detection, unique IDs, valid references
- 📝 **Requirements & ADRs**: First-class language support
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

## License

MIT
