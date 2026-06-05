# Ubiquitous Language

This document defines the shared terminology for Sruja development. All conversations, code, and documentation derive from this domain model.

## Core Concepts

| Term | Definition | Example |
|------|-----------|--------|
| Architecture | The structural design of a software system | C4 model with Systems, Containers, Components |
| Context | Evidence extracted from codebase that informs architecture | File imports, function calls, dependencies |
| Drift | Gap between documented architecture and actual code | Missing components, new dependencies |
| Element | A node in the architecture graph | System, Container, Component, Person, Database, Queue |
| Relationship | Connection between two elements | `SystemA -> SystemB "HTTPS"` |
| Scoping | Determining which files to analyze | `.srujaignore`, `.sruja/` directory |

## Sruja DSL Terms

| Term | Definition | Syntax |
|------|-----------|--------|
| System | Top-level boundary in C4 hierarchy | `MySystem = system "Label" { ... }` |
| Container | Deployable unit within a system | `MyContainer = container "Label" { ... }` |
| Component | Code unit within a container | `MyComponent = component "Label" { ... }` |
| Person | Human actor in the system | `User = person "User" { ... }` |
| Database | Data storage element | `DB = database "DB" { technology "PostgreSQL" }` |
| Queue | Async message processing | `Queue = queue "Queue" { technology "RabbitMQ" }` |

## Validation Rules

| Rule | Description |
|------|-------------|
| Orphan | Element with no relationships (should have at least one) |
| Cycle | Circular dependency between elements |
| Layer Violation | Cross-layer relationship (e.g., component bypassing container) |
| Container Nesting | Components must be inside containers/systems |
| Unique ID | Each element must have a unique ID |

## CLI Commands

| Command | Purpose |
|---------|--------|
| `sruja lint` | Validate .sruja file syntax and rules |
| `sruja scan` | Extract architecture evidence from code |
| `sruja export` | Export architecture diagram |
| `sruja drift` | Detect architecture drift |
| `sruja ai-context` | Build context payload for AI editors |
| `sruja doctor` | Validate health signals |
| `sruja mcp` | Start MCP server |

## Quality Signals

| Term | Definition |
|------|-----------|
| Health Score | 0-100 score for architecture quality |
| Context Score | AI-readiness across 5 dimensions |
| Comprehension Debt | Gap between code and human understanding |
| Code Churn | Lines rewritten/deleted within 2 weeks |

## Technology Stack

| Component | Technology |
|-----------|------------|
| Core | Rust |
| CLI | Rust + clap |
| Language | tree-sitter |
| WASM | wasm-bindgen |
| Extension | TypeScript + VS Code API |