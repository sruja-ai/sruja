# WASM Optimizations

This document describes the optimizations applied to the Sruja WASM build for improved performance and reduced file size.

## Current Optimizations

### 1. Build Flags
- `-ldflags="-s -w"`: Strips debug symbols and DWARF information
- `-trimpath`: Removes file system paths from the binary
- `GOOS=js GOARCH=wasm`: Targets WebAssembly platform

### 2. wasm-opt Optimization
- Uses `wasm-opt -Oz` for maximum size optimization
- Enables `--enable-bulk-memory` for better memory operations
- Reduces WASM binary size significantly

### 3. Code-Level Optimizations

#### Removed Debug Output
- All `fmt.Println` statements removed from production WASM builds
- Silent panic recovery (no console output)
- Reduces binary size and improves performance

#### String Operations
- Use `strings.Builder` instead of string concatenation
- Pre-allocate slices and maps with known capacity
- Example: `make([]string, 0, 20)` instead of `[]string{}`

#### Memory Pre-allocation
- Pre-allocate slices with estimated capacity:
  ```go
  estimatedSize := len(arch.Systems) + len(arch.Containers) + ...
  symbols := make([]map[string]interface{}, 0, estimatedSize)
  ```
- Pre-allocate maps with known capacity:
  ```go
  res := make(map[string]interface{}, 3) // Known capacity
  ```

#### Improved String Building
- Use `strings.Builder` with `Grow()` for error messages:
  ```go
  var msgBuilder strings.Builder
  msgBuilder.Grow(len(errs) * 50) // Pre-allocate
  ```

#### Better Identifier Extraction
- Efficient character-by-character parsing instead of `strings.Fields()`
- `isIdentChar()` helper for fast character checks
- Reduces allocations and improves performance

### 4. Compression
- Gzip compression: `sruja.wasm.gz`
- Brotli compression: `sruja.wasm.br` (better compression ratio)
- Web server should serve compressed versions with proper headers

## Future Optimization Opportunities

### 1. Parser/Validator Pooling
- Reuse parser and validator instances using `sync.Pool`
- Reduces allocation overhead for repeated calls
- **Status**: Considered but not implemented (parsers may not be thread-safe)

### 2. Build Tags for Feature Selection
- Use build tags to exclude unused features:
  - `//go:build !wasm_lsp` to exclude LSP functions
  - `//go:build !wasm_viewer` to exclude viewer-specific code
- Create minimal builds for specific use cases

### 3. TinyGo Alternative
- TinyGo produces smaller binaries but may have compatibility issues
- Already available via `make wasm-tiny` and `make wasm-viewer-tiny`
- Test thoroughly before using in production

### 4. Lazy Loading
- Load WASM only when needed (e.g., when playground is opened)
- Reduces initial page load time
- Implemented in TypeScript adapter layer

### 5. Code Splitting
- Separate WASM builds for different features:
  - Core parser only
  - Parser + validator
  - Full build with LSP
- Load appropriate build based on use case

## Performance Metrics

### Before Optimizations
- Full WASM: ~7.3MB
- LSP WASM: ~5.0MB

### After Optimizations
- Reduced debug overhead
- Improved string operation performance
- Better memory allocation patterns

## Build Commands

```bash
# Standard optimized build
make wasm

# Minimal viewer build (JSON, Markdown, SVG only)
make wasm-viewer

# With compression
make wasm-viewer-compressed

# TinyGo builds (smaller but may have issues)
make wasm-tiny
make wasm-viewer-tiny
```

## Best Practices

1. **Always use compression**: Serve `.wasm.gz` or `.wasm.br` with proper headers
2. **Lazy load**: Only load WASM when needed
3. **Monitor size**: Check WASM file size in CI/CD
4. **Test thoroughly**: Optimizations may affect functionality
5. **Use viewer build**: For viewer-only use cases, use `wasm-viewer` instead of full build

## References

- [WebAssembly Optimization Guide](https://web.dev/webassembly/)
- [wasm-opt Documentation](https://github.com/WebAssembly/binaryen)
- [Go WASM Documentation](https://pkg.go.dev/syscall/js)
