# Vision & Requirements

This document outlines the core vision and requirements for the Sruja language and platform.

[← Back to Documentation Index](../README.md)

## Core Vision

Sruja is a **bidirectional, code-driven architecture modeling system** that serves as the single source of truth for software architecture—from abstract intent to concrete implementation—and exposes this context via MCP (Model Context Protocol) so AI code assistants can generate architecturally compliant code in large codebases.

## Key Requirements

### 1. True Two-Way Sync

Edits in UI (diagram) ⇄ spec (code) must stay perfectly synchronized.

- UI changes → DSL updates
- DSL changes → diagram updates
- No information loss during round-trip
- Deterministic formatting for minimal diffs

### 2. Layered Abstraction Support

Support multiple levels of architectural abstraction:

- **VHLD (Very High-Level Design)**: Logical blocks (e.g., "Auth Service", "File Storage")
- **HLD (High-Level Design)**: Concrete tech (e.g., "Auth0", "S3")
- **LLD (Low-Level Design)**: Code-level constructs (modules, classes, interfaces)

Plus:
- User journeys
- ADRs (Architecture Decision Records)
- Requirements (functional, non-functional, constraints)
- Evolution history

### 3. Organization-Wide Reusability

Define and share custom component templates (e.g., `GDPRDataStore`, `EventSourcedAggregate`) as versioned libraries.

- Versioned component libraries
- Dependency management (like NPM/Go modules)
- Template marketplace
- Organizational standards enforcement

### 4. Git-Native & Versionable

Specs must be plain-text (DSL), diffable, and mergeable.

- All architecture definitions in text files
- Git-friendly format
- Merge conflict resolution
- Version history tracking
- Branch-based architecture proposals

### 5. MCP-Ready Output

The model must be queryable via a standardized API so LLMs or agents can:

- Validate design decisions
- Auto-generate scaffolding code
- Enforce architectural rules during coding
- Understand context (what, why, how, where)

### 6. Tooling Flexibility

Leverage or extend open-source tools (e.g., Structurizr Lite, Kroki, MPS, Backstage) but must support bidirectional editing—not just rendering from code.

## Goal

Enable AI assistants to understand:
- **What** to build (requirements)
- **Why** (ADRs)
- **How** (HLD/LLD)
- **Where** (ownership, boundaries)

So generated code aligns with organizational architecture.

## Use Cases

### AI Code Generation
- Generate architecturally compliant code
- Validate generated code against architecture rules
- Suggest architectural improvements

### Design Decision Validation
- Check design decisions against ADRs
- Validate pattern usage
- Enforce organizational standards

### Auto-Generate Scaffolding
- Generate code templates from architecture models
- Create project structure from DSL
- Generate API contracts from models

### Architectural Governance
- Enforce rules during coding
- Track architecture evolution
- Manage approval workflows
- Version and compare architecture proposals

## Success Criteria

1. ✅ Perfect bidirectional sync (UI ⇄ DSL)
2. ✅ Multi-layer abstraction support
3. ✅ Component library system
4. ✅ Git-native workflow
5. ✅ MCP integration for AI assistants
6. ✅ Extensible validation engine
7. ✅ Compilation to Mermaid (initial target)
8. ✅ Future: Custom UI rendering

