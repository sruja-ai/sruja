# Sruja - Architecture Editor with Live Code Sync

[![TypeScript Coverage](https://codecov.io/gh/sruja-ai/sruja/branch/main/graph/badge.svg?flag=typescript)](https://codecov.io/gh/sruja-ai/sruja)

**Visual architecture editing with live code sync** - edit diagrams or code, changes sync both ways automatically.

> **⚠️ Alpha Release**: Sruja is under active development. APIs may change.

---

## Why Sruja?

### 🔄 **Bidirectional Sync** - Like Notion for Architecture

Most architecture tools make you choose:

- ❌ Visual editor (Draw.io) - no code, no version control
- ❌ Code-only (Mermaid, PlantUML) - no visual editing
- ❌ One-way sync (Structurizr) - code → view only

**Sruja does both:**

- ✅ Edit visually → Code updates in real-time
- ✅ Edit code → Diagram updates automatically
- ✅ Version-controlled `.sruja` files in Git
- ✅ Works for your entire team (designers, developers, PMs)

**Try it now**: [Sruja Designer](https://designer.sruja.ai) (no signup required)

---

## Quick Start

### Option 1: Visual Editor (Fastest)

1. Open [Sruja Designer](https://designer.sruja.ai)
2. Create architecture visually (drag & drop)
3. Switch to **Code tab** - see the DSL
4. Edit either way - they sync automatically
5. Export `.sruja` file → Commit to Git

### Option 2: CLI (For Developers)

**Install:**

```bash
curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
```

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

### 🎨 Visual + Code, Together

- **Interactive Designer**: Web-based visual editor
- **Live Code Sync**: Changes sync bidirectionally (visual ↔ code)
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

## Comparison

| Feature                | Draw.io | Mermaid | Structurizr  | PlantUML | **Sruja** |
| ---------------------- | ------- | ------- | ------------ | -------- | --------- |
| Visual editor          | ✅      | ❌      | ⚠️ View only | ❌       | ✅        |
| Code-backed            | ❌      | ✅      | ✅           | ✅       | ✅        |
| **Bidirectional sync** | ❌      | ❌      | ❌           | ❌       | **✅**    |
| Real-time feedback     | ❌      | ❌      | ❌           | ❌       | **✅**    |
| Version control        | ❌      | ✅      | ✅           | ✅       | ✅        |
| Export/Import          | ✅      | ❌      | ⚠️ Limited   | ❌       | ✅        |

**Unique advantage**: True bidirectional visual ↔ code sync

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
├── cmd/
│   └── sruja/            # CLI tool
├── pkg/                  # Go packages
│   ├── engine/           # Validation engine
│   ├── language/         # Parser, AST, lexer
│   └── export/           # Exporters (JSON, Markdown, etc.)
├── apps/                 # Frontend applications
│   ├── designer/         # 🎨 Interactive visual designer
│   ├── website/          # Documentation site
│   └── vscode-extension/ # VS Code support
├── packages/             # TypeScript packages
│   ├── shared/           # Shared utilities
│   └── ui/               # UI components
└── examples/             # Example .sruja files
```

---

## Development

### Prerequisites

- **Go >= 1.25**
- **Node.js >= 18**

### Setup

```bash
# Install dependencies
go mod download
npm install

# Setup git hooks (recommended)
make setup-hooks

# Build CLI
make build

# Run designer locally
cd apps/designer
npm run dev
```

### Testing

```bash
make test          # Run all tests
make test-go       # Go tests only
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
- **Designer**: https://designer.sruja.ai
- **Discord**: https://discord.gg/VNrvHPV5
- **GitHub**: https://github.com/sruja-ai/sruja
