# Implementation Documentation

Implementation plans and technical details for building Sruja in Go.

[← Back to Documentation Index](../README.md)

## Current Focus: Go CLI Tool

**Goal**: Build Sruja language and CLI tooling in Go for software systems design.

## Implementation Plans

### Before Starting
- ⚠️ **[Pre-Implementation Checklist](./pre-implementation-checklist.md)** - **Review this first!**

### Core Implementation
- [Phase 1: Core Engine](./phase1-core.md) - DSL parser, model compiler, Mermaid generation
- [Technology Stack](./technology-stack.md) - Go-specific technology recommendations
- [Parser Decision](./parser-decision.md) - Parser technology choice (participle)

## Implementation Phases

### Phase 1: Core Language & CLI (Current)
- DSL Parser (Go)
- Model Compiler (DSL → JSON)
- Mermaid Compiler (JSON → Mermaid)
- Validation Engine
- CLI Tool

**Timeline**: 6-8 weeks

### Phase 2: Basic Engines (Ongoing)
- Basic pillar engines
- Extension support
- Plugin system

### Phase 3: Advanced Features (Future)
- Advanced engines
- MCP integration
- Code generation

### Phase 4: UI & Collaboration (Future)
- Web UI for viewing
- Visual editor
- Real-time collaboration

See [UI & Future Features](../ui-future/README.md) for UI plans.

---

## Key Principles

1. **Go-First** - All core tooling in Go
2. **CLI-First** - Command-line tool is primary interface
3. **File-Based** - Git-native, text-based architecture
4. **Extensible** - Plugin system for custom engines
5. **Incremental** - Build features incrementally

---

*Focus on building a solid Go CLI tool first, then add UI and advanced features later.*
