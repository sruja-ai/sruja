# WASM Functionality Analysis

## Question: Does having too many functionalities cause higher WASM size?

**Short Answer**: **Partially yes**, but the main culprit is the **Go runtime** (~3-4MB). However, including entire packages when only small parts are used does contribute significantly.

## What's Actually Included in WASM

### Currently Imported Packages

```go
// cmd/wasm/main.go imports:
- pkg/language          // Parser, AST, lexer (entire package)
- pkg/engine            // ALL validation rules (entire package)
- pkg/export/markdown   // ALL markdown export code (entire package)
- pkg/export/mermaid    // ALL mermaid export code (entire package)
- pkg/diagnostics       // Diagnostics (entire package)
- internal/converter     // JSON converter (entire package)
```

### The Problem: Package-Level Inclusion

**Go's linker includes entire packages**, even if you only use one function. This means:

1. **pkg/export/markdown** (~20,000+ lines)
   - Used: Only `NewExporter()` and `Export()`
   - Included: ALL helper functions, templates, test code (if not excluded)
   - Impact: ~500KB-1MB

2. **pkg/export/mermaid** (~5,000+ lines)
   - Used: Only `NewExporter()` and `Export()`
   - Included: ALL generators, renderers, config, styles
   - Impact: ~300-500KB

3. **pkg/engine** (~40 files, 20,000+ lines)
   - Used: Only 4-5 validation rules
   - Included: ALL validation rules (15+ rules), scorer, helpers
   - Impact: ~1MB

4. **pkg/language** (~74 files)
   - Used: Parser, AST, printer
   - Included: Entire language package
   - Impact: ~1-2MB (but this is core, needed)

## Size Breakdown (Estimated)

| Component | Size | Can Reduce? |
|-----------|------|-------------|
| Go Runtime | 3-4MB | Only with TinyGo |
| Parser (pkg/language) | 1-2MB | Core, needed |
| Engine (all rules) | ~1MB | Yes - use build tags |
| Markdown Export | ~500KB-1MB | Yes - use build tags |
| Mermaid Export | ~300-500KB | Yes - use build tags |
| Diagnostics | ~100-200KB | Minimal |
| Converter | ~100-200KB | Minimal |
| **Total** | **8.0MB** | |

## The Real Issue: Dead Code Not Eliminated

### Example: Markdown Exporter

**What you use**:
```go
exporter := markdown.NewExporter()
md, err := exporter.Export(program.Architecture)
```

**What gets included** (even if unused):
- All helper functions (20+ files)
- All template processing
- Executive summary helpers
- Deployment helpers
- SLO helpers
- Quality helpers
- Contract helpers
- Failure modes helpers
- Operational helpers
- Consistency helpers
- TOC helpers
- Mermaid config helpers
- All test code (if not excluded)

**Why?** Go's linker can't eliminate unused code within a package if any part of the package is used.

## Validation Rules: Only 4-5 Used, But All Included

**Used in WASM**:
- `UniqueIDRule`
- `ValidReferenceRule`
- `ScenarioFQNRule`
- `RelationTagRule` (only in LSP)
- `CompletenessRule` (only in LSP)

**But ALL rules are compiled**:
- `BestPracticesRule`
- `CycleRule`
- `OrphanRule`
- `LayerRule`
- `SloRule`
- `SloEnforcementRule`
- `SimplicityRule`
- `ExternalDependencyRule`
- `PropertiesValidationRule`
- `Scorer` (entire scoring system)

**Impact**: ~500KB-1MB of unused code

## Solutions

### 1. Build Tags (Recommended)

Exclude entire packages when not needed:

```go
// pkg/export/markdown/markdown.go
//go:build !wasm_no_markdown

// pkg/export/mermaid/exporter.go
//go:build !wasm_no_mermaid

// pkg/engine/scorer.go
//go:build !wasm_no_scorer
```

**Build commands**:
```bash
# Parser only (no exports)
GOOS=js GOARCH=wasm go build -tags wasm_no_markdown,wasm_no_mermaid ./cmd/wasm

# Parser + Markdown only
GOOS=js GOARCH=wasm go build -tags wasm_no_mermaid ./cmd/wasm

# Parser + Mermaid only
GOOS=js GOARCH=wasm go build -tags wasm_no_markdown ./cmd/wasm
```

**Expected reduction**: 500KB-1MB per excluded package

### 2. Split Packages

Split large packages into smaller ones:

```
pkg/export/markdown/
  - core.go          // NewExporter, Export (always included)
  - helpers.go       // Build tag: !wasm_no_markdown_helpers
  - templates.go     // Build tag: !wasm_no_markdown_templates
```

**Trade-off**: More complex package structure

### 3. Minimal Builds

Create separate minimal builds:
- `cmd/wasm-parser` - Parser only (~4-5MB)
- `cmd/wasm-viewer` - Parser + JSON export (~5-6MB)
- `cmd/wasm-full` - Everything (~8MB)

### 4. TinyGo (Best for Size)

TinyGo does better dead code elimination:
- Current: 8.0MB
- TinyGo: ~1-2MB (estimated)
- **75-87% reduction**

## Recommendations

### Immediate (This Week)
1. ✅ **Use build tags for exporters**
   - Exclude markdown if not needed
   - Exclude mermaid if not needed
   - **Expected**: 500KB-1MB reduction

2. ✅ **Use build tags for validation rules**
   - Exclude unused rules (scorer, best practices, etc.)
   - **Expected**: 200-500KB reduction

### Short-term (Next 2 Weeks)
1. Create minimal parser build
2. Test TinyGo compatibility
3. **Target**: 6-7MB (15-25% reduction)

### Medium-term (Next Month)
1. Evaluate TinyGo for production
2. **Target**: 1-2MB with TinyGo (75-87% reduction)

## Conclusion

**Yes, having many functionalities does contribute to size**, but:

1. **Go runtime is the biggest contributor** (~3-4MB, 50% of size)
2. **Package-level inclusion** means entire packages are included even if only one function is used
3. **Build tags can help** exclude unused packages (20-30% reduction)
4. **TinyGo is the best solution** for size (75-87% reduction)

The functionality itself isn't the problem - it's that Go's linker includes entire packages. Build tags and TinyGo are the solutions.
