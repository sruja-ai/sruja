# Sruja (beta) – Architecture-as-Code Tool

[![TypeScript Coverage](https://codecov.io/gh/sruja-ai/sruja/branch/main/graph/badge.svg?flag=typescript)](https://codecov.io/gh/sruja-ai/sruja)

**Architecture-as-code for the AI SDLC process** – define architecture in `.sruja` files; validate and export to Markdown and Mermaid diagrams.

> **Beta** – Sruja is under active development. We welcome feedback; APIs may still evolve.

---

## Why Sruja?

### 🔄 **Architecture-as-Code** – Version controlled, validated, exported

- Define architecture in `.sruja` files – version-controlled in Git
- Built-in validation – catch issues before they reach production
- Export to Markdown and Mermaid – integrate into your docs
- Works for developers and CI/CD pipelines

---

## Quick Start

### Install CLI

**Option A – install script (downloads binary from [GitHub Releases](https://github.com/sruja-ai/sruja/releases)):**

```bash
curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
```

**Option B – from Git (requires Rust):**

```bash
cargo install sruja-cli --git https://github.com/sruja-ai/sruja
```

**Option C – build from source:**

```bash
git clone https://github.com/sruja-ai/sruja.git && cd sruja && make build
```

Then ensure the install directory is on your `PATH` (install script uses `~/.local/bin` by default; Option B uses `~/.cargo/bin`; Option C uses `target/release`).

**Create `example.sruja`:**

```sruja
person = kind "Person"
system = kind "System"
container = kind "Container"

user = person "User" {
  description "End user of the application"
}

app = system "My App" {
  web = container "Web Server" {
    technology "Node.js"
  }
}

user -> app.web "visits"
```

**Export & validate:**

```bash
sruja lint example.sruja
sruja export json example.sruja
sruja export markdown example.sruja
```

---

## Features

### 🎨 Code-First Architecture Tool

- **DSL**: Simple architecture definition language
- **Export/Import**: `.sruja` files work with Git

### ✅ Built-in Validation

- Cycle detection
- Orphan detection
- Unique ID enforcement
- Valid reference checking

### 📊 Multiple Outputs

- **JSON**: Full model with metadata
- **DSL**: Text format for Git
- **Markdown**: Documentation generation
- **Mermaid**: Diagram export

### 🔍 Developer Tools

- **LSP Support**: VS Code extension with autocomplete
- **Code Formatter**: `sruja fmt`
- **Tree View**: `sruja tree`
- **CLI**: Full command-line interface

---

## Documentation

**Website**: https://sruja.ai

**Essential Guides:**

- [Contributing Guide](docs/CONTRIBUTING.md) - How to contribute
- [First Contribution](docs/FIRST_CONTRIBUTION.md) - Step-by-step guide
- [Language Specification](docs/LANGUAGE_SPECIFICATION.md) - Complete DSL reference
- [Design Philosophy](docs/DESIGN_PHILOSOPHY.md) - Language design principles

**Content Creation:**

- [Content Contribution](docs/CONTENT_CONTRIBUTION_GUIDE.md) - Creating courses & tutorials
- [Content Style Guide](docs/CONTENT_STYLE_GUIDE.md) - Writing best practices

---

## Project Structure

```
sruja/
├── crates/               # Rust crates
│   ├── sruja-core/       # Core parsing and validation engine
│   ├── sruja-wasm/       # WebAssembly bindings
│   └── sruja-lsp/        # Language Server Protocol
├── extension/            # VS Code extension (syntax highlighting, diagnostics)
└── examples/             # Example .sruja files
```

---

## Development

### Prerequisites

- **Rust >= 1.70**
- **Node.js >= 18**

### Setup

```bash
# Install dependencies
cargo fetch
npm install

# Setup git hooks (recommended)
make setup-hooks

# Build CLI
make build
```

### Testing

```bash
make test          # Run all tests
make test-rust     # Rust tests only
make test-coverage # With coverage report
```

---

## Contributing

We welcome contributions of all sizes!

### 🎯 New Contributors

**Start here**: [First Contribution Guide](docs/FIRST_CONTRIBUTION.md)

### Quick Links

- 💡 [Contribution Ideas](docs/CONTRIBUTION_IDEAS.md)
- 🐛 [Good First Issues](https://github.com/sruja-ai/sruja/labels/good%20first%20issue)
- 📖 [Full Contribution Guide](docs/CONTRIBUTING.md)
- 💬 [Discord](https://discord.gg/VNrvHPV5) | [Discussions](https://github.com/sruja-ai/sruja/discussions)

### Ways to Contribute

**No code required:**

- Fix typos in docs
- Add examples
- Report bugs
- Write tutorials

**Code contributions:**

- Fix bugs
- Add features
- Improve tests
- Enhance tooling

### Pull Request Checklist

- ✅ Run `make test`, `make fmt`, `make lint`
- ✅ Add/update tests
- ✅ Keep changes focused
- ✅ Use Conventional Commits (`feat:`, `fix:`, `docs:`)

---

## License

Apache 2.0

---

## Links

- **Website**: https://sruja.ai
- **Discord**: https://discord.gg/VNrvHPV5
- **GitHub**: https://github.com/sruja-ai/sruja
