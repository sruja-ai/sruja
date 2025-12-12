# WASM Size Analysis and Reduction Strategies

## Current Size: 8.0MB (unoptimized), 7.3MB (with wasm-opt)

## Size Contributors

### 1. Go Runtime (~3-4MB)
- **Unavoidable**: Go's WASM runtime includes garbage collector, scheduler, and standard library
- **Impact**: Largest single contributor
- **Reduction**: Use TinyGo (can reduce to ~1-2MB) but may have compatibility issues

### 2. Parser Library (~1-2MB)
- **Package**: `github.com/alecthomas/participle/v2`
- **Impact**: Large parser library with lookahead, lexer, and grammar support
- **Reduction**: 
  - Consider lighter parser alternatives (but would require rewrite)
  - Use build tags to exclude parser if not needed

### 3. Engine/Validation (~1MB)
- **Packages**: `pkg/engine/*` (all validation rules)
- **Impact**: Includes all validation rules even if not used
- **Reduction**:
  - Use build tags to exclude unused rules
  - Create minimal validation build

### 4. Exporters (~500KB-1MB)
- **Packages**: `pkg/export/markdown`, `pkg/export/mermaid`, `pkg/export/json`
- **Impact**: All exporters included even if only one is needed
- **Reduction**:
  - Use build tags to exclude unused exporters
  - Create feature-specific builds

### 5. LSP Functions (~500KB-1MB)
- **Location**: `cmd/wasm/main.go` (not in `wasm/main.go`)
- **Impact**: Diagnostics, symbols, hover, completion, go-to-definition, format, score
- **Reduction**: 
  - Already excluded in `wasm/` build (viewer-only)
  - Use `wasm/` instead of `cmd/wasm` for viewer use cases

### 6. Dependencies
- `gonum.org/v1/gonum` - Large math library (may not be needed for WASM)
- `github.com/tdewolff/minify/v2` - Minification library (may not be needed)
- Standard library packages (encoding/json, strings, etc.)

## Size Comparison

| Build | Size | Features |
|-------|------|----------|
| `cmd/wasm` (full) | 8.0MB | Parser + Engine + Exporters + LSP |
| `wasm/` (viewer) | 8.0MB | Parser + Engine + Exporters (no LSP) |
| After wasm-opt | 7.3MB | ~9% reduction |

**Note**: Both builds are same size, suggesting LSP functions don't add much. The bulk is Go runtime + parser + engine.

## Reduction Strategies

### Strategy 1: Use Build Tags (Recommended)

Create feature-specific builds using build tags:

```go
//go:build js && wasm && !wasm_no_markdown
// Include markdown exporter

//go:build js && wasm && !wasm_no_mermaid  
// Include mermaid exporter

//go:build js && wasm && !wasm_minimal
// Include full validation engine
```

**Expected reduction**: 20-30% (1.5-2MB)

### Strategy 2: Split into Multiple WASM Modules

Create separate WASM files for different features:
- `sruja-core.wasm` - Parser only (~3-4MB)
- `sruja-validator.wasm` - Validation rules (~1MB)
- `sruja-exporters.wasm` - Exporters (~1MB)
- Load only what's needed

**Expected reduction**: 40-50% per module (but requires multiple downloads)

### Strategy 3: Use TinyGo

TinyGo produces much smaller binaries:
- Current: 8.0MB
- TinyGo: ~1-2MB (estimated)

**Trade-offs**:
- May have compatibility issues
- Some Go features not supported
- Requires testing

**Command**: `make wasm-tiny` or `make wasm-viewer-tiny`

### Strategy 4: Remove Unused Dependencies

Check if these are actually used in WASM:
- `gonum.org/v1/gonum` - Math library (likely not needed)
- `github.com/tdewolff/minify/v2` - Minification (may not be needed)

**Expected reduction**: 500KB-1MB

### Strategy 5: Optimize Go Build

Additional build flags:
```bash
GOOS=js GOARCH=wasm go build \
  -ldflags="-s -w -X main.version=..." \
  -trimpath \
  -buildmode=exe \
  -o sruja.wasm ./cmd/wasm
```

**Expected reduction**: Minimal (already applied)

### Strategy 6: Compression (Already Applied)

- Gzip: ~2-3MB (60-70% reduction)
- Brotli: ~1.5-2MB (75-80% reduction)

**Note**: Compression is most effective but requires server support.

## Recommended Approach

### Phase 1: Quick Wins (Immediate)
1. ✅ Remove unused dependencies (`gonum`, `minify` if not used)
2. ✅ Use `wasm/` build instead of `cmd/wasm` for viewer (already done)
3. ✅ Ensure wasm-opt is applied (already done)
4. ✅ Use compression (already done)

**Expected**: 500KB-1MB reduction

### Phase 2: Build Tags (Short-term)
1. Create build tags for exporters
2. Create build tags for validation rules
3. Create minimal parser-only build

**Expected**: 1.5-2MB reduction (20-30%)

### Phase 3: TinyGo Evaluation (Medium-term)
1. Test TinyGo compatibility
2. Fix any compatibility issues
3. Use TinyGo for production if stable

**Expected**: 5-6MB reduction (60-75%)

### Phase 4: Code Splitting (Long-term)
1. Split into multiple WASM modules
2. Implement dynamic loading
3. Load only required features

**Expected**: 40-50% per module

## Implementation Priority

1. **High Priority**: Remove unused dependencies, use build tags
2. **Medium Priority**: Evaluate TinyGo
3. **Low Priority**: Code splitting (complex, may not be worth it)

## Measurement

After each optimization, measure:
```bash
# Build and measure
GOOS=js GOARCH=wasm go build -ldflags="-s -w" -trimpath -o sruja.wasm ./cmd/wasm
ls -lh sruja.wasm

# With wasm-opt
wasm-opt --enable-bulk-memory -Oz sruja.wasm -o sruja-opt.wasm
ls -lh sruja-opt.wasm

# Compressed
gzip -k sruja-opt.wasm
brotli -k sruja-opt.wasm
ls -lh sruja-opt.wasm*
```

## Target Sizes

| Build | Current | Target | Method |
|-------|---------|--------|--------|
| Full (unoptimized) | 8.0MB | 6.0MB | Build tags + dependency removal |
| Full (wasm-opt) | 7.3MB | 5.5MB | Build tags + dependency removal |
| Full (compressed) | ~2MB | ~1.5MB | All optimizations |
| TinyGo | N/A | ~1-2MB | TinyGo compiler |

## Notes

- Go's WASM runtime is inherently large (~3-4MB)
- Most size comes from Go runtime, not application code
- Compression is most effective (75-80% reduction)
- TinyGo is best option for size reduction but requires testing
- Build tags allow feature-specific builds without major refactoring
