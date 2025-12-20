# Next Steps - Go Implementation

## ✅ Recently Completed

1. ✅ **Task 1.0** - DSL Changes (Change blocks, Snapshot blocks, Metadata arrays)
2. ✅ **Task 1.1** - JSON Exporter
3. ✅ **Task 1.5** - Change Commands (create/validate - apply pending)
4. ✅ **Task 1.6** - Markdown Exporter (with all enhancements)
5. ✅ **Architecture Document Enhancements**:
   - Enhanced ADR export (context, decision, consequences)
   - Data Consistency Models section
   - Failure Modes and Recovery section
   - Enhanced deployment architecture export

## 🎯 Immediate Next Steps (High Priority)

### 1. Contract Examples (Quick Win)
**Status**: Pending  
**Priority**: High  
**Estimated Time**: 1-2 hours

**What's needed**:
- Verify contract syntax in DSL
- Add contract examples to `ecommerce_platform.sruja`
- Test contract export in markdown

**Why now**: Contract export code is ready, just needs examples to demonstrate functionality.

**Files to modify**:
- `examples/ecommerce_platform.sruja` - Add contract blocks
- Test with `./bin/sruja export markdown examples/ecommerce_platform.sruja`

---

### 2. Policy & Flow Implementation (Architecture Constructs)
**Status**: Pending  
**Priority**: High  
**Estimated Time**: 2-3 days

**What's needed**:
- Implement `Policy` type in `ast.go`
- Implement `Flow` type in `ast.go`
- Add parser support for `policy` and `flow` blocks
- Add post-processing logic
- Add printer support
- Add to markdown export

**Why now**: User clarified these are architecture constructs (not code constructs like DDD), so they should be implemented.

**Files to modify**:
- `pkg/language/ast.go` - Add Policy and Flow structs
- `pkg/language/parser.go` - Add parsing logic
- `pkg/language/ast_postprocess.go` - Add post-processing
- `pkg/language/printer.go` - Add printing logic
- `pkg/export/markdown/markdown.go` - Add export logic
- `examples/ecommerce_platform.sruja` - Add examples

---

### 3. Task 1.2: JSON to AST Converter
**Status**: Pending  
**Priority**: High  
**Estimated Time**: 3-4 days

**What's needed**:
- Implement JSON → AST conversion
- Ensure round-trip preservation (DSL → JSON → DSL)
- Handle all AST types
- Test with complex examples

**Why now**: Enables round-trip functionality, critical for Studio integration.

**Files to create/modify**:
- `pkg/import/json/json.go` - Main converter
- `pkg/import/json/converter.go` - Conversion logic
- Tests

---

### 4. Task 1.7: Language Server Protocol (LSP)
**Status**: Pending  
**Priority**: High  
**Estimated Time**: 5-7 days

**What's needed**:
- Implement LSP server
- Diagnostics (errors/warnings)
- Hover information
- Code completion
- Go to definition
- Find references
- Code actions (quick fixes)

**Why now**: Enhances developer experience, enables IDE integration.

**Files to create/modify**:
- `pkg/lsp/server.go` - LSP server (partially started)
- `pkg/lsp/diagnostics.go` - Error reporting
- `pkg/lsp/completion.go` - Code completion
- `pkg/lsp/hover.go` - Hover information
- `cmd/lsp/main.go` - LSP entry point

---

## 🔄 Medium Priority

### 5. Task 1.3: Additional CLI Commands
**Status**: Pending  
**Priority**: Medium  
**Estimated Time**: 2-3 days

**What's needed**:
- Additional CLI commands as needed
- Improve existing commands

---

### 6. Task 1.4: Modularization
**Status**: Pending  
**Priority**: Medium  
**Estimated Time**: 2-3 days

**What's needed**:
- Split large files
- Improve code organization
- Reduce complexity

---

### 7. Task 2.1: HTML Exporter
**Status**: Pending  
**Priority**: Medium  
**Estimated Time**: 3-4 days

**What's needed**:
- Implement HTML export
- Use Studio/Viewer for visualization
- Interactive diagrams

**Why later**: Depends on Studio/Viewer being ready.

---

## 📋 P2 Enhancements (Lower Priority)

### 8. Security Architecture Expansion
**Status**: Pending  
**Priority**: Low  
**Estimated Time**: 1-2 days

**What's needed**:
- Expand security details in markdown export
- Document authentication/authorization
- Security best practices

---

### 9. Observability Section
**Status**: Pending  
**Priority**: Low  
**Estimated Time**: 1-2 days

**What's needed**:
- Metrics, logging, tracing strategy
- Add to markdown export

---

### 10. Capacity Planning
**Status**: Pending  
**Priority**: Low  
**Estimated Time**: 1 day

**What's needed**:
- Current capacity documentation
- Growth projections
- Add to markdown export

---

### 11. Testing Strategy
**Status**: Pending  
**Priority**: Low  
**Estimated Time**: 1 day

**What's needed**:
- Test coverage documentation
- Integration test strategy
- Add to markdown export

---

### 12. Trade-offs and Limitations
**Status**: Pending  
**Priority**: Low  
**Estimated Time**: 1 day

**What's needed**:
- Explicit documentation of trade-offs
- Known limitations
- Add to markdown export

---

## Recommended Order

1. **Contract Examples** (1-2 hours) - Quick win, demonstrates functionality
2. **Policy & Flow Implementation** (2-3 days) - Core architecture constructs
3. **JSON to AST Converter** (3-4 days) - Enables round-trip, Studio integration
4. **LSP Implementation** (5-7 days) - Developer experience, IDE integration

**Total Estimated Time**: ~10-15 days for high-priority items

---

## Notes

- **Contract Examples** can be done immediately as a quick win
- **Policy & Flow** are architecture constructs and should be implemented (unlike DDD which is deferred)
- **JSON to AST** is critical for Studio integration
- **LSP** significantly improves developer experience
- P2 enhancements can be done incrementally as needed

