# Website and Rust Backend Integration

This document describes how to integrate the Rust backend with the website.

## Current Status

The website currently uses **Go WASM** compiled from `cmd/wasm/main.go`. We need to migrate to **Rust WASM** using `wasm-bindgen`.

## Architecture

### Go WASM (Current)

- Uses Go's `syscall/js` package
- Requires `wasm_exec.js` runtime
- Functions registered on `window` object
- Functions: `sruja_dsl_to_model`, `sruja_dsl_to_mermaid`, etc.

### Rust WASM (Target)

- Uses `wasm-bindgen` for bindings
- No runtime needed (pure WASM)
- Functions exported via `wasm-bindgen`
- Same function names for compatibility

## Migration Steps

### 1. Build Rust WASM

```bash
# Install wasm-pack if needed
cargo install wasm-pack

# Build WASM
cd crates/sruja-wasm
wasm-pack build --target web --out-dir ../../apps/website/public/wasm/rust --release
```

Or use the Makefile:

```bash
make wasm
```

### 2. Update Website Scripts

The website's `ensure-wasm.mjs` script needs to:

- Check for Rust WASM first
- Fall back to Go WASM if Rust not available
- Or use a new `ensure-wasm-rust.mjs` script

### 3. Update TypeScript Adapter

The `packages/shared/src/web/wasmAdapter.ts` needs to:

- Support both Go WASM (current) and Rust WASM (new)
- Detect which WASM is available
- Use appropriate initialization

## Implementation

### Rust WASM Functions

The Rust WASM crate (`crates/sruja-wasm`) exports:

- `sruja_dsl_to_model(dsl: string, filename?: string) -> string`
- `sruja_dsl_to_mermaid(dsl: string, config?: string) -> string`
- `sruja_dsl_to_dot(dsl: string, view_level?: u8, target_id?: string) -> string`
- `sruja_dsl_to_markdown(dsl: string) -> string`
- `sruja_model_to_dsl(model_json: string) -> string`
- `sruja_get_diagnostics(dsl: string, filename?: string) -> string`
- `sruja_calculate_architecture_score(dsl: string) -> string`

### TypeScript Integration

Create a new adapter `packages/shared/src/web/wasmAdapterRust.ts` that:

1. Loads Rust WASM module via `wasm-bindgen` generated code
2. Wraps functions to match Go WASM API
3. Provides same interface as Go WASM adapter

## Testing

### Test WASM Build

```bash
# Build Rust WASM
make wasm

# Check output
ls -la apps/website/public/wasm/rust/
```

### Test Website Integration

```bash
cd apps/website
npm run dev
# Open browser and test playground
```

## Migration Strategy

### Option 1: Parallel Support (Recommended)

- Keep Go WASM as fallback
- Add Rust WASM as primary
- Website tries Rust first, falls back to Go
- Gradual migration

### Option 2: Direct Replacement

- Remove Go WASM completely
- Use only Rust WASM
- Requires full TypeScript adapter rewrite

## Current Implementation

- ✅ Rust WASM crate created (`crates/sruja-wasm`)
- ✅ WASM functions implemented
- ✅ Makefile targets added
- ⏳ TypeScript adapter (needs implementation)
- ⏳ Website script updates (needs implementation)

## Next Steps

1. **Complete Rust WASM build**: Ensure all functions work
2. **Create TypeScript adapter**: Match Go WASM API
3. **Update website scripts**: Use Rust WASM
4. **Test integration**: Verify website works
5. **Remove Go WASM**: Once Rust is stable

## Troubleshooting

### WASM Build Fails

- Check `wasm-pack` is installed: `cargo install wasm-pack`
- Check Rust target: `rustup target add wasm32-unknown-unknown`
- Check dependencies in `Cargo.toml`

### Website Can't Load WASM

- Check file paths in `public/wasm/rust/`
- Check browser console for errors
- Verify WASM module exports match TypeScript expectations

### Function Not Found

- Check `wasm-bindgen` exports match function names
- Verify TypeScript adapter loads module correctly
- Check browser console for registration errors
