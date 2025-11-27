# Architectural Decisions

Critical design decisions that must be made early to support future Git/cloud/OSS/enterprise use cases.

## 🎯 Overview

**Key Principle**: Make foundational architectural decisions now (takes 1 day), avoid expensive rewrites later (saves months).

### What You MUST Do Now
- ✅ Make 5 critical architectural decisions
- ✅ Design for flexibility (OSS, SaaS, enterprise, self-host)
- ✅ Keep core engine pure and platform-agnostic

### What You DON'T Need in MVP
- ❌ Git integration implementation
- ❌ Cloud backend
- ❌ OAuth/GitHub integration
- ❌ Multi-user collaboration
- ❌ Database schema
- ❌ SaaS subscription model

---

## The 5 Critical Design Decisions

### 1. File-Based Architecture Model ✅

**Decision**: Architecture must always be stored as files, not in databases.

**Structure**: See [Project Format v1](./project-format.md) for complete specification.

**Required Structure**:
```
/project-root/
/architecture.sruja        → Main DSL file (required)
  /.architecture/
    /model.json             → JSON model (required, committed)
    /config.json            → Project configuration (required)
    /visual.json            → Visual layout (optional, user-specific)
    /index.json             → Global index (optional, cacheable)
    /cache/                 → LSP cache (gitignored)
  /adrs/                    → Architecture Decision Records (optional)
  /journeys/                → User journeys (optional, post-MVP)
  /requirements/            → Requirements docs (optional, post-MVP)
  /libraries/               → Component libraries (optional, post-MVP)
```

**Why This Matters**:
- ✅ Works with Git (version control)
- ✅ Works with local filesystem (offline)
- ✅ Works with cloud storage (S3, etc.)
- ✅ Works with self-hosted storage
- ✅ Enables OSS contributions
- ✅ Enables CLI tools
- ✅ Standard format (APF-1.0) ensures interoperability

**Implementation**: See [Project Format v1](./project-format.md) for complete specification.

---

### 2. Project Abstraction Layer

**Decision**: Abstract file operations behind a provider interface.

**Interface** (Go):
```go
type Provider interface {
    LoadFile(path string) ([]byte, error)
    SaveFile(path string, content []byte) error
    ListFiles(pattern string) ([]string, error)
    DeleteFile(path string) error
    Exists(path string) (bool, error)
}
```

**Implementations**:

#### (A) FilesystemProvider (Current)
```go
type FilesystemProvider struct {
    basePath string
}

func (p *FilesystemProvider) LoadFile(path string) ([]byte, error) {
    return os.ReadFile(filepath.Join(p.basePath, path))
}

func (p *FilesystemProvider) SaveFile(path string, content []byte) error {
    fullPath := filepath.Join(p.basePath, path)
    return os.WriteFile(fullPath, content, 0644)
}

// ... other methods
```

#### (B) GitProvider (Planned)
```go
type GitProvider struct {
    repoURL  string
    baseRef  string
}

func (p *GitProvider) LoadFile(path string) ([]byte, error) { /* ... */ return nil, nil }
func (p *GitProvider) SaveFile(path string, content []byte) error { /* ... */ return nil }
func (p *GitProvider) ListFiles(pattern string) ([]string, error) { /* ... */ return nil, nil }
func (p *GitProvider) DeleteFile(path string) error { /* ... */ return nil }
func (p *GitProvider) Exists(path string) (bool, error) { /* ... */ return false, nil }
```

#### (C) WorkspaceProvider (Planned)
```go
type WorkspaceProvider struct {
    roots []string
    cache Cache
}

func (p *WorkspaceProvider) LoadFile(path string) ([]byte, error) { /* ... */ return nil, nil }
func (p *WorkspaceProvider) SaveFile(path string, content []byte) error { /* ... */ return nil }
func (p *WorkspaceProvider) ListFiles(pattern string) ([]string, error) { /* ... */ return nil, nil }
func (p *WorkspaceProvider) DeleteFile(path string) error { /* ... */ return nil }
func (p *WorkspaceProvider) Exists(path string) (bool, error) { /* ... */ return false, nil }
```

#### Caching Strategy

- Content-addressed cache keyed by file path and commit/ref.
- Provider-level caching to avoid repeated I/O across engines.

#### (B) GitProvider (Future)
```go
type GitProvider struct {
    repoURL string
    token   string
}

func (p *GitProvider) LoadFile(path string) ([]byte, error) {
    // Use GitHub/GitLab API or git clone
  }
  
  // ... other methods
}
```

#### (C) CloudStorageProvider (Future)
```go
type CloudStorageProvider struct {
    bucket      string
    credentials string
}

func (p *CloudStorageProvider) LoadFile(path string) ([]byte, error) {
    // Use S3, GCS, Azure Blob, etc.
    // Implementation for future
}
```

**Why This Matters**:
- ✅ Core engine doesn't care about storage backend
- ✅ Easy to add new storage backends later
- ✅ Supports local, Git, cloud, self-hosted
- ✅ Testable (mock providers)

**Implementation**: Add in Phase 0 or early Phase 1.

---

### 3. Pure Functional DSL Engine

**Decision**: DSL parser, model engine, and LSP server must be pure functions with zero platform assumptions.

**Rules**:
- ❌ NO HTTP dependencies
- ❌ NO database dependencies
- ❌ NO authentication dependencies
- ❌ NO Git dependencies
- ❌ NO cloud service dependencies
- ✅ Pure functions only
- ✅ Stateless operations
- ✅ Platform-agnostic

**Example** (Go):
```go
// ✅ GOOD: Pure function
func ParseDSL(dsl string) (*Model, error) {
    ast, err := parser.Parse(dsl)
    if err != nil {
        return nil, err
    }
    return transformer.ASTToModel(ast)
}

// ❌ BAD: Platform-dependent
func ParseDSL(dsl string) (*Model, error) {
    ast, err := parser.Parse(dsl)
    if err != nil {
        return nil, err
    }
    user, err := getCurrentUser() // ❌ HTTP/DB dependency
    if err != nil {
        return nil, err
    }
    return transformer.ASTToModel(ast)
}
```

**Why This Matters**:
- ✅ Can run in CLI tools
- ✅ Can run in VSCode extension (via LSP)
- ✅ Can run in GitHub Actions
- ✅ Can run in cloud backend (future)
- ✅ Easy to test
- ✅ Easy to embed anywhere

**Implementation**: Already aligned - DSL parser and model engine are pure.

---

### 4. Stateless LSP Server

**Decision**: LSP server must be stateless and never persist documents to cloud DB.

**Architecture**:
```
✅ CORRECT:
Editor → LSP → Parse → Model → Diagnostics → Editor
         (stateless, ephemeral)

❌ WRONG:
Editor → LSP → Database → Model → Editor
         (stateful, locked to cloud)
```

**Rules**:
- LSP server receives document text via WebSocket
- LSP server parses and validates in memory
- LSP server returns diagnostics/completions
- LSP server does NOT persist to database
- LSP server does NOT store document state
- Document state lives in editor (Monaco)

**Why This Matters**:
- ✅ User controls source of truth (Git, local files)
- ✅ Works offline
- ✅ Works self-hosted
- ✅ Works in cloud (stateless)
- ✅ No vendor lock-in
- ✅ Can run LSP anywhere (local, cloud, edge)

**Implementation**: LSP server design already follows this pattern.

---

### 5. Modular Repository Structure

**Decision**: Organize monorepo to support plugins, libraries, and extensions.

**Structure** (Go):
```
/pkg
  /language            → DSL parser (pure, platform-agnostic)
  /compiler            → Model compiler (pure, platform-agnostic)
  /engine              → Validation and processing engines
  /model               → Model types and structures
  /composition         → Multi-module composition
  /providers           → Storage provider abstraction
    /filesystem        → FilesystemProvider (current)
    /git               → GitProvider (future)
    /cloud             → CloudStorageProvider (future)

/cmd
  /sruja               → CLI tool (current focus)

/internal
  /graph               → Graph operations
  /utils               → Internal utilities
```

**Why This Matters**:
- ✅ Clean separation of concerns
- ✅ Easy to add new providers
- ✅ Easy to create CLI tools
- ✅ Easy to create VSCode extension
- ✅ OSS contributions welcome
- ✅ Enterprise can extend

**Implementation**: Current structure already aligns, but ensure providers are in separate package.

---

## Implementation Checklist

### Phase 0: Foundation (Go)
- [ ] Design `Provider` interface (Go)
- [ ] Implement `FilesystemProvider`
- [ ] Ensure DSL parser is pure (no dependencies)
- [ ] Ensure compiler is pure (no dependencies)
- [ ] Organize Go packages with provider abstraction

### Phase 1: Core Engine (Go)
- [ ] Verify all core functions are pure
- [ ] Add provider abstraction to compiler
- [ ] Test with FilesystemProvider
- [ ] Document provider interface

### Future: Extensions
- [ ] Implement `GitProvider` (when needed)
- [ ] Implement `CloudStorageProvider` (when needed)
- [ ] Create LSP server using core packages (see [LSP Architecture](../ui-future/lsp-architecture.md))
- [ ] Create VSCode extension using core packages

---

## Why These Decisions Matter

These 5 decisions enable:

### ✅ OSS Usage
- Users can run locally
- Users can self-host
- Users can contribute plugins

### ✅ SaaS Cloud Usage
- Stateless LSP can run in cloud
- Provider abstraction allows cloud storage
- No vendor lock-in

### ✅ Enterprise Usage
- Can self-host everything
- Can integrate with internal Git
- Can extend with custom providers

### ✅ Developer Experience
- CLI tools possible
- VSCode extension possible
- GitHub Actions integration possible

---

## Anti-Patterns to Avoid

### ❌ Don't: Tie Core Engine to HTTP
```go
// ❌ BAD
func ParseDSL(dsl string) (*Model, error) {
    user, err := http.Get("/api/user") // ❌ HTTP dependency
  return parser.parse(dsl);
}
```

### ❌ Don't: Persist State to Database (Future LSP)
```go
// ❌ BAD (for future LSP)
func OnDidChangeTextDocument(doc *TextDocument) error {
    return db.SaveDocument(doc) // ❌ Database dependency
}
```

### ❌ Don't: Hardcode Storage Backend
```go
// ❌ BAD
type ModelService struct{}

func (s *ModelService) Save(model *Model) error {
    return git.Commit(model) // ❌ Hardcoded to Git
}
```

### ✅ Do: Use Provider Abstraction
```go
// ✅ GOOD
type ModelService struct {
    provider Provider
}

func (s *ModelService) Save(model *Model) error {
    dsl := serializer.ModelToDSL(model)
    return s.provider.SaveFile("architecture.sruja", []byte(dsl))
}
```

---

## Summary

**Make these 5 decisions now** (takes 1 day):
1. ✅ File-based architecture (already decided)
2. ✅ Project abstraction layer (add in Phase 0)
3. ✅ Pure functional DSL engine (already aligned)
4. ✅ Stateless LSP server (already aligned)
5. ✅ Modular repository structure (already aligned)

**Don't implement these in MVP**:
- ❌ Git integration
- ❌ Cloud backend
- ❌ OAuth
- ❌ Multi-user collaboration
- ❌ Database persistence

**Result**: Future-proof architecture that supports OSS, SaaS, enterprise, and self-hosted use cases from the same core.

---

[← Back to Documentation Index](../README.md)
