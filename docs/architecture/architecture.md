# System Architecture

System structure and organization for the Sruja language and CLI tool (Go implementation).

[← Back to Documentation Index](../README.md)

## Repository Structure (Go CLI)

```
/sruja
│
├── cmd/
│   └── sruja/                  → CLI entry point
│       └── main.go
│
├── pkg/
│   ├── language/              → DSL parser and language processing
│   │   ├── parser.go          → DSL parser (participle or hand-written)
│   │   ├── lexer.go           → Lexical analysis
│   │   ├── ast.go             → AST types and structures
│   │   └── grammar.go         → Grammar definition
│   │
│   ├── compiler/              → Model compilation
│   │   ├── transformer.go    → AST → IR transformation
│   │   ├── model.go           → Model building
│   │   ├── mermaid.go         → Mermaid compiler
│   │   └── serializer.go      → Model → DSL serialization
│   │
│   ├── engine/                → Validation and processing engines
│   │   ├── validation.go     → Validation engine
│   │   ├── registry.go       → Engine registry
│   │   └── engines/          → Individual engine implementations
│   │
│   ├── model/                 → Architecture model types
│   │   ├── model.go          → Core model types
│   │   ├── elements.go       → Architecture elements
│   │   └── relations.go     → Relationships
│   │
│   ├── composition/           → Multi-module composition
│   │   ├── composer.go       → Model composition engine
│   │   ├── resolver.go       → Import and reference resolution
│   │   └── namespace.go      → Namespace management
│   │
│   ├── providers/             → Storage provider abstraction
│   │   ├── provider.go       → Provider interface
│   │   ├── filesystem.go     → Local filesystem provider
│   │   └── git.go            → Git provider (future)
│   │
│   └── mcp/                   → MCP integration (future)
│       └── server.go
│
├── internal/                 → Private packages
│   ├── graph/                → Graph operations
│   └── utils/                → Internal utilities
│
├── docs/                      → Documentation
├── examples/                  → Example DSL files
├── go.mod                     → Go module definition
├── go.sum                     → Dependency checksums
└── README.md
```

## Architecture Principles

### Key Design Decisions

1. **Go-First**: All core tooling implemented in Go
2. **CLI-First**: Command-line tool is primary interface
3. **Package Separation**: Clear boundaries between parser, compiler, engines
4. **Pure Core Engine**: Parser and compiler are pure functions (no HTTP/DB dependencies)
5. **Provider Abstraction**: Storage operations abstracted via `Provider` interface
6. **File-Based Architecture**: All models stored as files (`.sruja`, `.json`) for Git compatibility
7. **Extensible Engines**: Plugin system for custom engines

See [Architectural Decisions](./architectural-decisions.md) for detailed rationale.

## Data Flow

```
DSL File (.sruja)
    ↓
CLI Command (sruja compile)
    ↓
DSL Parser (Go) → AST
    ↓
AST Transformer → IR Model
    ↓
Model Composer → Global Model (JSON)
    ↓
Validation Engine → Validation Results
    ↓
Mermaid Compiler → Mermaid Diagram (.mmd)
    ↓
Output Files
```

## Enterprise Boundaries

The platform models enterprise architecture with explicit boundaries and governance.

- **Domains**: Top-level business areas (e.g., `identity`, `ordering`)
- **Bounded Contexts**: Domain-specific conceptual models owned by teams
- **Teams**: Ownership metadata and responsibilities
- **Context Maps**: Relationship types between contexts
- **Ownership**: Qualified identifiers (e.g., `identity.auth.api`)
- **Boundary Policies**: Cross-boundary rules enforced by validation engine

### Governance Outcomes

- Enforce domain boundaries and contract-based interactions
- Prevent direct access across contexts except through published APIs
- Support Team Topologies constraints
- Provide structured metadata for queries and generation

## Module System

### Single Module (Current)

```
project/
├── architecture.sruja        → Main DSL file
└── .architecture/
    ├── model.json            → Compiled JSON model
    └── config.json           → Project configuration
```

### Multi-Module (Future)

See [Multi-Module Architecture](./multi-module-architecture.md) for cross-project linking and composition.

## Provider Abstraction

Storage operations are abstracted behind a provider interface:

```go
type Provider interface {
    LoadFile(path string) ([]byte, error)
    SaveFile(path string, content []byte) error
    ListFiles(pattern string) ([]string, error)
    Exists(path string) (bool, error)
}
```

**Implementations:**
- `FilesystemProvider` - Local filesystem (current)
- `GitProvider` - Git repository (future)

## CLI Commands

```
sruja compile [file]          → Compile DSL to Mermaid
sruja validate [file]         → Validate architecture
sruja format [file]           → Format DSL file
sruja export [format]         → Export to various formats
```

## Future: UI Integration

When building UI later, the Go CLI will serve as the backend:
- CLI tool exposes API (REST or gRPC)
- UI calls CLI commands or API
- Same core engine used by both CLI and UI

See [UI & Future Features](../ui-future/README.md) for UI plans.

---

[← Back to Documentation Index](../README.md)
