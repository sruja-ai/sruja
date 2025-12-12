# WASM Size Reduction Plan

## Current State
- **Size**: 8.0MB (unoptimized), 7.3MB (with wasm-opt)
- **Compressed**: ~2MB (gzip), ~1.5MB (brotli)
- **Both builds same size**: Full (`cmd/wasm`) and minimal (`wasm/`) are both 8.0MB

## Root Causes

### 1. Go Runtime (~3-4MB) - **Largest Contributor**
- Go's WASM runtime includes GC, scheduler, standard library
- **Unavoidable** with standard Go compiler
- **Solution**: Use TinyGo (reduces to ~1-2MB)

### 2. Parser Library (~1-2MB)
- `github.com/alecthomas/participle/v2` - Large parser library
- **Needed**: Core functionality
- **Solution**: Build tags to exclude if not needed

### 3. Engine/Validation (~1MB)
- All validation rules included
- **Solution**: Build tags to exclude unused rules

### 4. Exporters (~500KB-1MB)
- Markdown, Mermaid, JSON exporters
- **Solution**: Build tags to exclude unused exporters

### 5. Dependencies (Not Used in WASM)
- `gonum.org/v1/gonum` - **Not imported in WASM** ✅
- `github.com/tdewolff/minify/v2` - **Not imported in WASM** ✅
- These are in `go.mod` but not actually linked into WASM

## Immediate Actions (Quick Wins)

### ✅ Already Done
1. Removed debug output (`fmt.Println`)
2. Applied wasm-opt optimization
3. Using compression (gzip/brotli)
4. Minimal build exists (`wasm/` without LSP)

### 🔧 Can Do Now

#### 1. Verify Dependencies
```bash
# Check what's actually linked
go list -deps ./cmd/wasm | grep -E "gonum|tdewolff"
```
If not found, they're not contributing to size.

#### 2. Test TinyGo
```bash
make wasm-tiny
# Compare sizes
ls -lh apps/website/public/wasm/sruja.wasm
```
**Expected**: 1-2MB (60-75% reduction)

#### 3. Use Compression
Already done, but verify server serves compressed files:
- `.wasm.gz` with `Content-Encoding: gzip`
- `.wasm.br` with `Content-Encoding: br`

## Short-term Optimizations (1-2 weeks)

### 1. Build Tags for Exporters

Create feature-specific builds:

```go
// cmd/wasm/main.go
//go:build js && wasm

// pkg/export/markdown/markdown.go
//go:build js && wasm && !wasm_no_markdown

// pkg/export/mermaid/mermaid.go  
//go:build js && wasm && !wasm_no_mermaid
```

**Build commands**:
```bash
# Parser only
GOOS=js GOARCH=wasm go build -tags wasm_no_markdown,wasm_no_mermaid ./cmd/wasm

# Parser + Markdown only
GOOS=js GOARCH=wasm go build -tags wasm_no_mermaid ./cmd/wasm

# Parser + Mermaid only
GOOS=js GOARCH=wasm go build -tags wasm_no_markdown ./cmd/wasm
```

**Expected reduction**: 500KB-1MB per excluded exporter

### 2. Build Tags for Validation Rules

```go
// pkg/engine/scorer.go
//go:build js && wasm && !wasm_no_scorer

// pkg/engine/best_practices_rule.go
//go:build js && wasm && !wasm_no_best_practices
```

**Expected reduction**: 200-500KB per excluded rule

### 3. Minimal Parser Build

Create `cmd/wasm-minimal` with only parser:
- No validation
- No exporters
- Just parse DSL to JSON

**Expected size**: ~4-5MB (50% reduction)

## Medium-term Optimizations (1-2 months)

### 1. Evaluate TinyGo

**Pros**:
- 60-75% size reduction (8MB → 1-2MB)
- Faster startup
- Smaller memory footprint

**Cons**:
- Compatibility issues possible
- Some Go features not supported
- Requires testing

**Action**:
1. Test with `make wasm-tiny`
2. Run full test suite
3. Fix any compatibility issues
4. Use for production if stable

### 2. Code Splitting (Multiple WASM Modules)

Split into separate modules:
- `sruja-core.wasm` - Parser only (~3-4MB)
- `sruja-validator.wasm` - Validation (~1MB)
- `sruja-markdown.wasm` - Markdown export (~500KB)
- `sruja-mermaid.wasm` - Mermaid export (~500KB)

Load only what's needed.

**Expected**: 40-50% reduction per module, but requires multiple downloads

## Long-term Optimizations (3+ months)

### 1. Alternative Parser

Consider lighter parser alternatives:
- Hand-written recursive descent
- Smaller parser generator
- Trade-off: More maintenance

**Expected**: 1-2MB reduction

### 2. WASM Component Model

Use WASI/Component Model for better tree-shaking:
- Better dead code elimination
- Smaller runtime
- Future standard

## Recommended Priority

### Phase 1: Now (This Week)
1. ✅ Test TinyGo: `make wasm-tiny`
2. ✅ Verify compression is working
3. ✅ Document current state

### Phase 2: Short-term (Next 2 Weeks)
1. Add build tags for exporters
2. Add build tags for validation rules
3. Create minimal parser build
4. **Target**: 6-7MB (15-25% reduction)

### Phase 3: Medium-term (Next Month)
1. Evaluate TinyGo for production
2. Fix any compatibility issues
3. **Target**: 1-2MB with TinyGo (75-87% reduction)

### Phase 4: Long-term (Future)
1. Consider code splitting
2. Evaluate alternative parsers
3. Monitor WASM Component Model

## Measurement

Track size after each change:

```bash
# Build
make wasm

# Measure
ls -lh apps/website/public/wasm/sruja.wasm

# With wasm-opt (should be automatic)
wasm-opt --enable-bulk-memory -Oz apps/website/public/wasm/sruja.wasm -o /tmp/test.wasm
ls -lh /tmp/test.wasm

# Compressed
gzip -k apps/website/public/wasm/sruja.wasm
brotli -k apps/website/public/wasm/sruja.wasm
ls -lh apps/website/public/wasm/sruja.wasm*
```

## Target Sizes

| Build Type | Current | Target | Method |
|------------|---------|--------|--------|
| Full (unoptimized) | 8.0MB | 6.0MB | Build tags |
| Full (wasm-opt) | 7.3MB | 5.5MB | Build tags |
| Full (compressed) | ~2MB | ~1.5MB | All optimizations |
| TinyGo | N/A | ~1-2MB | TinyGo compiler |
| Minimal (parser only) | 8.0MB | 4-5MB | Build tags |

## Notes

- **Go runtime is the biggest contributor** (~3-4MB)
- **Compression is most effective** (75-80% reduction)
- **TinyGo is best option** for size but requires testing
- **Build tags** allow feature-specific builds without major refactoring
- **Both builds are same size** - LSP functions don't add much
