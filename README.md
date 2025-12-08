# Sruja

[![TypeScript Coverage](https://codecov.io/gh/sruja-ai/sruja/branch/main/graph/badge.svg?flag=typescript)](https://codecov.io/gh/sruja-ai/sruja)

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
│   └── export/           # Exporters (D2, HTML, Markdown, SVG, JSON)
├── apps/                 # Frontend applications
│   ├── website/          # Astro website (docs, courses, tutorials)
│   ├── studio-core/      # Studio app (diagram editor)
│   ├── viewer-core/     # Viewer app (architecture visualization)
│   └── vscode-extension/ # VS Code language support
├── packages/             # Shared TypeScript packages
│   ├── shared/           # Shared utilities and types
│   ├── ui/               # UI component library
│   └── viewer/           # Viewer library
└── examples/             # Example .sruja files
```

### Developer Documentation

- Architecture & Code Organization: `docs/ARCHITECTURE.md`
- Internal Guide: `internal-docs/guide.md`

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

    User -> App.Web "Visits"
    App.Web -> App.DB "Reads/Writes"
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
- Test playground examples compile correctly
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
- 💬 **Get Help**: [Discord](https://discord.gg/QMCsquJq) | [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions)

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

## License

MIT
