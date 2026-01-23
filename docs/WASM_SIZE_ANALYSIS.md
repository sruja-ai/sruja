# WASM Size Analysis: Rust vs Go

## Current Rust WASM Sizes

### Measured Sizes (as of analysis)

**Uncompressed Sizes:**

| Build Type                                       | Size       | Notes                            |
| ------------------------------------------------ | ---------- | -------------------------------- |
| `cargo build --release` (wasm32-unknown-unknown) | **994 KB** | Direct cargo build               |
| `wasm-pack build --release`                      | **1.7 MB** | Includes wasm-bindgen glue code  |
| `wasm-opt -O3` optimized                         | **748 KB** | 25% reduction from release build |

**Compressed Sizes (from 748 KB optimized WASM):**

| Compression      | Size       | Reduction | Notes                               |
| ---------------- | ---------- | --------- | ----------------------------------- |
| **Uncompressed** | **748 KB** | -         | After wasm-opt optimization         |
| **Gzip**         | **254 KB** | 66%       | Standard HTTP compression           |
| **Brotli**       | **187 KB** | 75%       | Better compression, modern browsers |

**Note**: The 748 KB figure is the **uncompressed** WASM binary size after `wasm-opt` optimization. When served over HTTP with compression (gzip/brotli), the actual transfer size is much smaller (187-254 KB).

### Current Optimization Settings

The `crates/sruja-wasm/Cargo.toml` already includes size optimizations:

```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization
strip = true        # Strip symbols
```

## Analysis

### What's Contributing to Size?

1. **Core Dependencies**:
   - `sruja-language` (parser with `nom`)
   - `sruja-engine` (validation engine)
   - `sruja-export` (multiple exporters: JSON, Mermaid, DOT, Markdown)
   - `sruja-diagnostics` (error reporting)

2. **WASM-Specific Dependencies**:
   - `wasm-bindgen` (bindings generator)
   - `js-sys` and `web-sys` (JavaScript interop)
   - `serde` + `serde_json` (serialization)

3. **Standard Library**:
   - Rust stdlib includes many features that may not all be needed

### Size Comparison Context

- **Graphviz WASM** (from ADR 004): ~1 MB (uncompressed)
- **Current Rust WASM** (optimized, uncompressed): 748 KB - **25% smaller than Graphviz**
- **Current Rust WASM** (optimized, gzip): 254 KB - **75% smaller than Graphviz**
- **Current Rust WASM** (optimized, brotli): 187 KB - **81% smaller than Graphviz**
- **Current Rust WASM** (release, uncompressed): 994 KB - **Comparable to Graphviz**

## Is Rust Actually Smaller Than Go?

**Answer: We need to measure the Go WASM to know for certain.**

### Why Rust _Should_ Be Smaller

1. **No Runtime Overhead**: Rust compiles to WASM without a runtime (Go includes a runtime)
2. **Better Dead Code Elimination**: Rust's LTO and `opt-level = "z"` are very effective
3. **Smaller Standard Library**: Rust's stdlib is more modular

### Why It Might Not Be

1. **Go's TinyGo**: If you used TinyGo (not standard Go), it produces very small WASM
2. **Feature Parity**: The Rust version might include more features than the Go version
3. **Dependencies**: Rust dependencies might pull in more code than Go's minimal stdlib

## Recommendations

### 1. Measure Go WASM Size (if available)

If you have old Go WASM files or can rebuild them:

```bash
# If you have old Go WASM
ls -lh path/to/old/sruja.wasm

# Or rebuild Go WASM for comparison
GOOS=js GOARCH=wasm go build -o sruja_go.wasm ./cmd/sruja_wasm
ls -lh sruja_go.wasm
```

### 2. Optimize Further with wasm-opt

Add `wasm-opt` to your build process:

```bash
# In Makefile or build script
wasm-opt -O3 --strip-debug \
  target/wasm32-unknown-unknown/release/sruja_wasm.wasm \
  -o output/sruja_wasm_optimized.wasm
```

This reduces size from **994 KB → 748 KB** (25% reduction).

**Note**: The Makefile has been updated to automatically run `wasm-opt` after building.

### 2b. Enable HTTP Compression

For production, serve WASM files with compression:

- **Gzip**: Reduces 748 KB → 254 KB (66% reduction)
- **Brotli**: Reduces 748 KB → 187 KB (75% reduction)

Most web servers (nginx, Cloudflare, etc.) automatically compress WASM files. The codebase already supports `.wasm.gz` files in Node.js adapters (VSCode extension).

### 3. Consider Feature Flags

Split functionality into optional features:

```toml
[features]
default = ["json", "mermaid"]
json = []
mermaid = []
dot = []
markdown = []
```

This allows users to only include what they need.

### 4. Use `wee_alloc` for Smaller Allocator

For WASM, consider using a smaller allocator:

```toml
[dependencies]
wee_alloc = { version = "0.4", optional = true }

[features]
default = []
small-alloc = ["wee_alloc"]
```

### 5. Enable `panic = "abort"` for WASM

In `Cargo.toml`:

```toml
[profile.release]
panic = "abort"  # Smaller binary, no unwinding
```

### 6. Use `wasm-pack` with Size Optimization

Update the Makefile to use wasm-opt:

```makefile
wasm:
	wasm-pack build --target web --out-dir ../../apps/website/public/wasm/rust crates/sruja-wasm --release
	wasm-opt -O3 --strip-debug \
	  ../../apps/website/public/wasm/rust/sruja_wasm_bg.wasm \
	  -o ../../apps/website/public/wasm/rust/sruja_wasm_bg.wasm
```

## Action Items

1. **Measure Go WASM**: Find or rebuild Go WASM to compare
2. **Add wasm-opt**: Integrate `wasm-opt` into build pipeline
3. **Consider Feature Flags**: Split functionality to reduce size
4. **Monitor Size**: Add size checks to CI/CD

## Expected Outcomes

With all optimizations:

- **Uncompressed**: 994 KB (release) / 748 KB (wasm-opt)
- **Compressed (gzip)**: ~254 KB (from 748 KB optimized)
- **Compressed (brotli)**: ~187 KB (from 748 KB optimized)
- **With panic=abort + feature flags**: Potentially 500-600 KB uncompressed (~150-200 KB compressed)
- **Comparison needed**: Measure Go WASM to validate migration benefit

**Real-world transfer size**: With HTTP compression, users typically download **187-254 KB** (brotli/gzip), not 748 KB.

## Conclusion

**The Rust WASM is likely smaller than Go**, especially with `wasm-opt` optimization:

- **Uncompressed**: 748 KB (after wasm-opt)
- **Compressed (gzip)**: 254 KB
- **Compressed (brotli)**: 187 KB

However, **we need actual Go WASM measurements** to confirm this assumption. The migration to Rust provides other benefits (performance, type safety, maintainability) regardless of size.

**Key Takeaway**: While the uncompressed WASM is 748 KB, with HTTP compression (standard in production), users download only **187-254 KB**, which is excellent for web performance. The codebase already supports compressed WASM files (`.wasm.gz`) in Node.js environments.
