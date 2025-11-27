# Pre-Implementation Checklist

**Purpose**: Verify all critical decisions and requirements are finalized before starting implementation.

[← Back to Implementation Guide](./README.md)

## ✅ Critical Decisions

### 1. Language & Technology Stack
- [x] **Language**: Go (decided)
- [x] **Parser**: `participle` (decided - handles all features)
- [x] **CLI Framework**: `cobra` (decided)
- [x] **Git Integration**: `go-git` (decided)
- [x] **Testing**: Standard library + `testify` (decided)
- [x] **Build Tool**: `goreleaser` (decided)

**Status**: ✅ **All decisions made**

---

### 2. Architecture Decisions
- [x] **File-based architecture** (decided - Git-friendly)
- [x] **Provider abstraction** (decided - filesystem/Git providers)
- [x] **Pure core engine** (decided - no HTTP/DB dependencies)
- [x] **CLI-first approach** (decided - UI later)
- [x] **Multi-format compilation** (decided - Mermaid, D2, PlantUML)

**Status**: ✅ **All decisions made**

**Reference**: [Architectural Decisions](../architecture/architectural-decisions.md)

---

### 3. DSL Design
- [x] **Grammar approach**: `participle` struct-based (decided)
- [x] **Core syntax**: Defined (workspace, model, systems, containers, components)
- [x] **Extensions**: Designed (resilience, performance, security, cost, sustainability)
- [x] **Requirements & ADRs**: Designed
- [x] **Systems thinking**: Designed (future)

**Status**: ✅ **Design complete**

**Reference**: [DSL Overview](../specs/dsl-overview.md), [DSL Index](../specs/dsl-index.md)

---

### 4. Compilation Targets
- [x] **Primary**: Mermaid (decided)
- [x] **Secondary**: D2 (decided - for beautiful themes)
- [x] **Future**: PlantUML, SVG, PNG (planned)

**Status**: ✅ **Targets defined**

**Reference**: [D2 vs Sruja Analysis](../architecture/d2-vs-sruja-analysis.md)

---

### 5. Project Structure
- [x] **Go module structure**: Defined
- [x] **Package organization**: Defined (language, compiler, engine, model, etc.)
- [x] **CLI structure**: Defined (`cmd/sruja/`)

**Status**: ✅ **Structure defined**

**Reference**: [System Architecture](../architecture/architecture.md)

---

## ⚠️ Items to Verify

### 1. DSL Grammar Specification
- [ ] **Formal grammar**: Is the complete grammar documented?
- [ ] **Edge cases**: Are edge cases defined? (empty files, malformed syntax, etc.)
- [ ] **Error messages**: Are error message formats defined?
- [ ] **Source location tracking**: Is position tracking strategy clear?

**Action**: Review [DSL Specification](../specs/dsl-specification.md) - ensure complete

---

### 2. AST Structure
- [ ] **AST types**: Are all AST node types defined?
- [ ] **Source locations**: Is position tracking in AST nodes defined?
- [ ] **Serialization**: Can AST be serialized/deserialized? (for caching, LSP)

**Action**: Define AST structure in `pkg/language/ast.go`

---

### 3. Model Structure
- [ ] **JSON schema**: Is the model JSON schema defined?
- [ ] **Versioning**: Is model versioning strategy defined?
- [ ] **Backward compatibility**: How will we handle model evolution?

**Action**: Define model structure in `pkg/model/model.go`

---

### 4. Validation Rules
- [ ] **Core rules**: Are the 4 core validation rules defined?
  - [ ] Unique ID validation
  - [ ] Valid reference checking
  - [ ] Cycle detection
  - [ ] Orphan detection
- [ ] **Error formats**: Are validation error formats defined?
- [ ] **Severity levels**: Error, warning, info?

**Action**: Review [Validation Engine](../pillars/engines/core/validation-engine.md)

---

### 5. CLI Commands
- [ ] **Command list**: Are all CLI commands defined?
  - [ ] `sruja compile` - Compile to Mermaid/D2
  - [ ] `sruja validate` - Validate architecture
  - [ ] `sruja format` - Format DSL file
  - [ ] `sruja hover` - Get element info (for VS Code)
  - [ ] `sruja complete` - Get completions (for VS Code)
  - [ ] `sruja definition` - Get definition location (for VS Code)
- [ ] **Flags**: Are command flags defined? (`--format`, `--output`, etc.)
- [ ] **Error handling**: How are CLI errors handled?

**Action**: Review [CLI Specification](../specs/cli-specification.md) if exists

---

### 6. Testing Strategy
- [ ] **Unit tests**: What needs unit tests? (parser, compiler, validation)
- [ ] **Integration tests**: What needs integration tests? (end-to-end compilation)
- [ ] **Test data**: Do we have example `.sruja` files for testing?
- [ ] **Test coverage goal**: What's the target coverage?

**Action**: Review [Testing Strategy](../guides/testing-strategy.md) if exists

---

### 7. Error Handling
- [ ] **Parser errors**: How are syntax errors reported?
- [ ] **Validation errors**: How are validation errors reported?
- [ ] **Compilation errors**: How are compilation errors reported?
- [ ] **User-friendly messages**: Are error messages user-friendly?

**Action**: Define error types and message formats

---

### 8. Documentation
- [ ] **User docs**: Is there a getting started guide?
- [ ] **Developer docs**: Is the developer guide complete?
- [ ] **API docs**: Will we generate Go docs? (`godoc`)
- [ ] **Examples**: Do we have example `.sruja` files?

**Action**: Review documentation completeness

---

## 🔍 Pre-Implementation Review

### Grammar Completeness Check

**Verify**:
1. Can the grammar parse all example `.sruja` files?
2. Are all keywords and operators defined?
3. Are comments handled?
4. Are whitespace rules clear?
5. Are string escaping rules defined?

**Action**: Create test `.sruja` files and verify grammar can parse them

---

### Parser Implementation Check

**Verify**:
1. Does `participle` support all grammar features we need?
2. Can we get source locations from `participle`?
3. Can we handle partial parsing (for completion)?
4. Are error messages from `participle` sufficient?

**Action**: Create a small proof-of-concept parser

---

### Model Design Check

**Verify**:
1. Can the model represent all DSL features?
2. Is the model JSON-serializable?
3. Can we round-trip (DSL → Model → DSL)?
4. Is the model extensible (for extensions)?

**Action**: Design model structure and verify it can represent all features

---

### Compiler Design Check

**Verify**:
1. Can we compile to Mermaid? (test with simple example)
2. Can we compile to D2? (test with simple example)
3. Are compilation outputs deterministic?
4. Can we handle compilation errors gracefully?

**Action**: Create proof-of-concept compilers

---

## 📋 Implementation Readiness Score

| Category | Status | Notes |
|----------|--------|-------|
| **Technology Stack** | ✅ Complete | All decisions made |
| **Architecture** | ✅ Complete | All decisions made |
| **DSL Design** | ✅ Complete | Grammar designed |
| **AST Structure** | ⚠️ Needs Definition | Define in code |
| **Model Structure** | ⚠️ Needs Definition | Define in code |
| **Validation Rules** | ✅ Complete | Rules defined |
| **CLI Commands** | ⚠️ Needs Review | Verify all commands |
| **Testing Strategy** | ⚠️ Needs Review | Define approach |
| **Error Handling** | ⚠️ Needs Definition | Define error types |
| **Documentation** | ✅ Mostly Complete | Review examples |

**Overall Readiness**: ~80% - Ready to start, but define AST/Model structures first

---

## 🚀 Recommended Pre-Implementation Steps

### Step 1: Define Core Structures (1-2 days)

**Create**:
1. `pkg/language/ast.go` - AST node types
2. `pkg/model/model.go` - Model structure
3. `pkg/language/errors.go` - Error types

**Verify**:
- AST can represent all DSL features
- Model can represent all architecture concepts
- Errors are user-friendly

---

### Step 2: Proof-of-Concept Parser (1 day)

**Create**:
1. Simple `participle` parser for basic DSL
2. Parse a simple `.sruja` file
3. Generate AST

**Verify**:
- `participle` works for our grammar
- Source locations are tracked
- Error messages are good

---

### Step 3: Proof-of-Concept Compiler (1 day)

**Create**:
1. Simple Mermaid compiler
2. Compile AST → Mermaid
3. Test with simple example

**Verify**:
- Compilation works
- Output is correct
- Errors are handled

---

### Step 4: Review & Finalize (1 day)

**Review**:
1. Proof-of-concept code
2. Identify any issues
3. Adjust design if needed
4. Finalize implementation plan

---

## ✅ Final Checklist Before Starting

- [ ] **AST structure defined** (`pkg/language/ast.go`)
- [ ] **Model structure defined** (`pkg/model/model.go`)
- [ ] **Error types defined** (`pkg/language/errors.go`)
- [ ] **Proof-of-concept parser** (basic grammar works)
- [ ] **Proof-of-concept compiler** (AST → Mermaid works)
- [ ] **Test `.sruja` files** (examples for testing)
- [ ] **Go module initialized** (`go mod init`)
- [ ] **Project structure created** (directories)
- [ ] **Dependencies identified** (participle, cobra, etc.)

**Once all checked**: ✅ **Ready to start full implementation**

---

## 🎯 Implementation Order

Once pre-implementation is complete:

1. **Phase 1**: Core Parser (Week 1)
   - Lexer
   - Parser (participle)
   - AST generation
   - Basic error handling

2. **Phase 2**: Model Compiler (Week 2)
   - AST → Model transformation
   - Model structure
   - JSON serialization

3. **Phase 3**: Mermaid Compiler (Week 2-3)
   - Model → Mermaid
   - Basic diagram generation

4. **Phase 4**: Validation Engine (Week 3)
   - Core validation rules
   - Error reporting

5. **Phase 5**: CLI Tool (Week 4)
   - Command structure
   - File I/O
   - Error handling

6. **Phase 6**: VS Code Extension (Week 5)
   - Syntax highlighting
   - Hover, completion, go-to-definition
   - Mermaid preview

---

## 📚 References

- [Technology Stack](./technology-stack.md) - Go stack decisions
- [Architectural Decisions](../architecture/architectural-decisions.md) - Architecture decisions
- [DSL Overview](../specs/dsl-overview.md) - DSL design
- [System Architecture](../architecture/architecture.md) - Project structure
- [Developer Guide](../DEVELOPER-GUIDE.md) - Implementation guide

---

*Review this checklist before starting implementation. Address any ⚠️ items first.*

