# Sruja

[![TypeScript Coverage](https://codecov.io/gh/sruja-ai/sruja/branch/main/graph/badge.svg?flag=typescript)](https://codecov.io/gh/sruja-ai/sruja)

**Sruja** is a next-generation architecture-as-code language with first-class support for requirements, ADRs, and extensible validation.

> **⚠️ Alpha Release**: Sruja is under active development. APIs may change.

## Documentation

https://sruja.ai

## Features

- 🎯 **Architecture DSL**: Define systems, containers, components, and relations with first-class support for requirements and ADRs
- ✅ **Validation Engine**: Cycle detection, orphan detection, unique IDs, valid references
- 📊 **Multiple Export Formats**: JSON, Markdown, LikeC4, and more for integration with other tools
- 🎨 **Code Formatter**: Auto-format your architecture with `sruja fmt`
- 🌳 **Tree View**: Visualize hierarchy with `sruja tree`
- 🔍 **LSP Support**: Language Server Protocol for IDE integration (VS Code extension available)
- 🎨 **Interactive Designer**: Web-based visual designer for creating and editing architecture diagrams

## Project Structure

```
sruja/
├── cmd/
│   └── sruja/            # Main CLI tool
├── pkg/                  # Shared Go packages
│   ├── engine/           # Validation engine
│   ├── language/         # Parser, AST, lexer
│   └── export/           # Exporters (JSON, views)
├── apps/                 # Frontend applications
│   ├── website/          # Astro website (docs, courses, tutorials)
│   ├── designer/         # Interactive designer application (Sruja Designer)
│   ├── vscode-extension/ # VS Code language support
│   ├── social-publish/   # Social media publishing tools
│   └── storybook/        # Component documentation
├── packages/             # Shared TypeScript packages
│   ├── shared/           # Shared utilities and types
│   ├── ui/               # UI component library
│   ├── layout/           # Layout algorithms for diagrams
│   └── diagram/          # Diagram rendering utilities
└── examples/             # Example .sruja files
```

### Developer Documentation

**Essential Guides:**
- [Contributing Guide](docs/CONTRIBUTING.md) - How to contribute to Sruja
- [First Contribution](docs/FIRST_CONTRIBUTION.md) - Step-by-step guide for your first contribution
- [Development Guide](docs/DEVELOPMENT.md) - Development practices and tooling
- [Architecture Guide](docs/ARCHITECTURE.md) - Code organization and structure

**Content Creation:**
- [Content Contribution Guide](docs/CONTENT_CONTRIBUTION_GUIDE.md) - Creating courses, tutorials, and docs
- [Content Style Guide](docs/CONTENT_STYLE_GUIDE.md) - Writing style and best practices

**Reference:**
- [Language Specification](docs/LANGUAGE_SPECIFICATION.md) - Complete DSL reference
- [Design Philosophy](docs/DESIGN_PHILOSOPHY.md) - Language design principles

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
specification {
  element person
  element system
  element container
  element database
}

model {
  user = person "User" {
    description "End user of the application"
  }

  app = system "My App" {
    web = container "Web Server" {
      technology "Node.js"
    }
    db = database "Database" {
      technology "PostgreSQL"
    }
  }

  user -> app.web "visits"
  app.web -> app.db "reads/writes"
}
```

**Export to JSON:**

```bash
sruja export json example.sruja
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

- **Go >= 1.25** (CI uses `1.25.5`)
- **Node.js >= 18** (CI uses `24` for most workflows)

### Setup

```bash
# Install dependencies
go mod download

# Setup git hooks (recommended)
make setup-hooks

# Build CLI
make build
```

### Git Hooks

A pre-commit hook automatically tests code compilation when you commit changes to the `examples/` directory. This prevents broken code from being committed.

**Setup:**

```bash
make setup-hooks
```

The hook will:

- Test designer examples compile correctly
- Test course code blocks compile correctly
- Test docs code blocks compile correctly
- Block commits if any code fails to compile

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for more details.

## Contributing

We welcome contributions of all sizes — from small doc fixes to core features.

### 🎯 New to Contributing?

**Start here:** [First Contribution Guide](docs/FIRST_CONTRIBUTION.md)

This guide walks you through making your first contribution, even if you're new to the project.

### Quick Links

- 💡 **Contribution Ideas**: [What Can I Contribute?](docs/CONTRIBUTION_IDEAS.md)
- 🐛 **Find Issues**: [Good First Issues](https://github.com/sruja-ai/sruja/labels/good%20first%20issue) (may be limited for new projects)
- 📖 **Full Guide**: [Contribution Guide](docs/CONTRIBUTING.md)
- 💬 **Get Help**: [Discord](https://discord.gg/VNrvHPV5) | [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions)

### Ways to Contribute

**No Code Required:**

- Fix typos in documentation
- Add examples
- Test and report bugs
- Write tutorials or blog posts

**Code Contributions:**

- Fix bugs
- Add features
- Improve tests
- Enhance tooling

### Pull Request Checklist

- Run local checks: `make test`, `make fmt`, `make lint`
- Add/update tests for new behavior
- Keep changes focused and well‑scoped
- Use Conventional Commits (e.g., `feat: …`, `fix: …`, `docs: …`)

Apache 2.0
