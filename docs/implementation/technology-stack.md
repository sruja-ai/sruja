# Technology Stack for Go Implementation

This document outlines the recommended technology stack for building Sruja in Go.

[← Back to Documentation Index](../README.md)

## Overview

Since Sruja is being built in Go (as a solo founder project), this document provides Go-specific technology recommendations based on the conversation analysis.

## Core Stack

### Language: Go

**Why Go:**
- ✅ Fast compilation and execution
- ✅ Excellent for CLI tools
- ✅ Strong standard library
- ✅ Good concurrency primitives
- ✅ Easy to distribute (single binary)
- ✅ Growing ecosystem
- ✅ Good for solo developers (simpler than Rust, more performant than TypeScript for backend)

### Parser & DSL Processing

**Decision:** Use **participle** for all parsing needs.

**Why `participle`:**
- ✅ Struct-based parser combinators, very Go-idiomatic
- ✅ Handles all DSL features (core + extensions + systems thinking)
- ✅ Built-in error reporting and source location tracking
- ✅ Fast development velocity
- ✅ Sufficient for all engines and extensions

**No migration needed** - `participle` can handle the full language complexity.

```go
// Example with participle
type Architecture struct {
    Systems []System `@@*`
}

type System struct {
    Name string `@Ident`
    // ...
}
```

### JSON Model

**Standard library:** `encoding/json`

Go's standard JSON library is excellent. Use struct tags for mapping:

```go
type ArchitectureModel struct {
    Version    string      `json:"version"`
    Elements   []Element   `json:"elements"`
    Relations  []Relation  `json:"relations"`
}
```

### Diagram Compilers

**Approach:** Template-based code generation for multiple formats

**Primary Target: Mermaid**
Use Go's `text/template` or `html/template` to generate Mermaid code:

```go
const mermaidTemplate = `
C4Context
{{range .Elements}}
    {{.Type}}({{.Id}}, "{{.Name}}")
{{end}}
{{range .Relations}}
    Rel({{.From}}, {{.To}}, "{{.Label}}")
{{end}}
`
```

**Additional Targets: D2, PlantUML**

Add compilers for other diagram formats:
- **D2**: For beautiful themes and animations (see [D2 vs Sruja Analysis](../architecture/d2-vs-sruja-analysis.md))
- **PlantUML**: For UML-style diagrams
- **SVG/PNG**: Direct export (future)

**Usage**:
```bash
sruja compile --format mermaid example.sruja  # Default
sruja compile --format d2 example.sruja      # D2 format
sruja compile --format plantuml example.sruja # PlantUML format
```

### Validation Engine

**Approach:** Custom rule engine in Go

Build a simple, extensible rule system:

```go
type Rule interface {
    ID() string
    Validate(model *ArchitectureModel) []ValidationIssue
}

type ValidationEngine struct {
    rules []Rule
}
```

### CLI Framework

**Options:**
1. **cobra** - Most popular, feature-rich
2. **urfave/cli** - Simpler, more minimal
3. **Standard library flag** - Built-in, minimal

**Recommendation:** **cobra** for better UX and extensibility.

### File I/O & Git Integration

**Options:**
1. **go-git** - Pure Go Git implementation
2. **libgit2 bindings** - More features, C dependency
3. **Command-line git** - Simple but less control

**Recommendation:** **go-git** for MVP (pure Go, no C dependencies).

### Testing

**Standard library:** `testing` package

Go's built-in testing is excellent. Also consider:
- **testify** - Assertions and test suites
- **ginkgo/gomega** - BDD-style testing (optional)

### Documentation

**Standard:** `godoc` (built into Go)

Generate documentation from code comments.

## Project Structure

```
sruja/
├── cmd/
│   └── sruja/          # CLI entry point
├── pkg/
│   ├── language/        # Lexer, Parser, AST
│   ├── compiler/        # Mermaid compiler
│   ├── engine/          # Validation engine
│   ├── model/           # JSON model types
│   ├── mcp/             # MCP server
│   └── extensions/      # Extension system
├── internal/            # Private packages
├── docs/                # Documentation
└── examples/            # Example .sruja files
```

## Development Tools

### Build & Release
- **goreleaser** - Automated releases, cross-compilation
- **Make** - Build automation

### Linting & Formatting
- **gofmt** - Built-in formatter
- **golangci-lint** - Comprehensive linter
- **goimports** - Auto-import management

### Dependency Management
- **Go Modules** - Built-in (go.mod)

## Performance Considerations

### When to Optimize

**Start simple, optimize later:**
- Go is already fast for most use cases
- Parser performance is rarely a bottleneck
- Focus on correctness first
- Profile if needed (`go tool pprof`)

### Potential Optimizations (Future)

- **Parallel parsing** for large files
- **Caching** compiled models
- **Incremental parsing** for editor integration
- **WASM compilation** for browser (if needed)

## Comparison with TypeScript Approach

The conversation also discussed a TypeScript approach. Here's why Go is better for this project:

| Aspect | Go | TypeScript |
|--------|----|-----------|
| **CLI Tool** | ✅ Excellent | ⚠️ Requires Node.js |
| **Distribution** | ✅ Single binary | ⚠️ npm/node dependencies |
| **Performance** | ✅ Fast | ⚠️ Slower (but acceptable) |
| **Parser** | ✅ Good libraries | ✅ Excellent (Ohm, Chevrotain) |
| **Learning Curve** | ✅ Moderate | ✅ Easy (if you know JS) |
| **Ecosystem** | ✅ Growing | ✅ Huge |
| **Solo Developer** | ✅ Good fit | ✅ Also good |

**For Sruja (Go-first):**
- CLI tool is primary interface
- Single binary distribution is valuable
- Go's simplicity helps solo development
- Can add TypeScript UI later if needed

## Future: UI Layer (Optional)

If building a web UI later:

**Options:**
1. **Separate TypeScript/React app** - Call Go backend via API
2. **Go web server** - Serve HTML/JS, use Go templates
3. **WASM** - Compile Go to WASM for browser (experimental)

**Recommendation:** Start with CLI, add web UI later as separate service.

## Summary

**Core Stack:**
- **Language**: Go
- **Parser**: participle (MVP) or hand-written (later)
- **JSON**: standard library
- **CLI**: cobra
- **Git**: go-git
- **Testing**: standard library + testify
- **Build**: goreleaser

**This stack provides:**
- Fast development velocity
- Easy distribution (single binary)
- Good performance
- Simple deployment
- Strong tooling ecosystem

Perfect for a solo founder building a CLI-first architecture tool.

