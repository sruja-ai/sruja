# Architecture Documentation

Internal architecture of the Sruja language and CLI tool.

[← Back to Documentation Index](../README.md)

## Contents

- [System Architecture](./architecture.md) - Repository structure and organization (Go CLI focus)
- [Architectural Decisions](./architectural-decisions.md) - Critical design decisions (updated for Go)
- [Multi-Module Architecture](./multi-module-architecture.md) - Cross-project linking and composition (concepts relevant, examples need Go update)

## Current Focus

**Go CLI Implementation:**
- All architecture docs updated to focus on Go CLI tool
- Code examples updated to Go where applicable
- UI/TypeScript references moved to [UI & Future Features](../ui-future/README.md)

## Key Architectural Principles

1. **File-Based Architecture** - All models stored as files (`.sruja`, `.json`)
2. **Provider Abstraction** - Storage operations abstracted via `Provider` interface
3. **Pure Core Engine** - Parser and compiler are pure functions
4. **CLI-First** - Command-line tool is primary interface
5. **Extensible** - Plugin system for custom engines

---

*See [System Architecture](./architecture.md) for Go repository structure.*

