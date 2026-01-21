# Website and Rust Backend Integration - COMPLETE ✅

## Summary

The website now uses the **Rust WASM backend by default**, with automatic fallback to Go WASM if Rust is unavailable.

## What Was Done

### 1. Rust WASM Crate (`crates/sruja-wasm`)

- ✅ Created WASM bindings using `wasm-bindgen`
- ✅ Implemented all required functions:
  - `sruja_dsl_to_model`
  - `sruja_dsl_to_mermaid`
  - `sruja_dsl_to_dot`
  - `sruja_dsl_to_markdown`
  - `sruja_model_to_dsl`
  - `sruja_get_diagnostics`
  - `sruja_calculate_architecture_score`

### 2. Build Infrastructure

- ✅ Updated `Makefile` with `wasm` and `wasm-tiny` targets
- ✅ Updated `apps/website/scripts/ensure-wasm.mjs` to build Rust WASM
- ✅ Created `apps/website/scripts/ensure-wasm-rust.mjs` for Rust-specific build
- ✅ Updated `apps/designer/scripts/ensure-wasm-exists.mjs` to check Rust WASM path

### 3. TypeScript Integration

- ✅ Created `packages/shared/src/web/wasmAdapterRust.ts`
  - Loads Rust WASM module via `wasm-bindgen` generated code
  - Wraps functions to match Go WASM API
  - Handles error wrapping for compatibility
  - Builds `DotResult` from model JSON and DOT string

- ✅ Updated `packages/shared/src/web/wasmAdapter.ts`
  - `initWasmAuto()` now tries Rust WASM first
  - Falls back to Go WASM if Rust fails
  - Maintains backward compatibility

## How It Works

1. **Build Time**: `npm run ensure:wasm` builds Rust WASM to `apps/website/public/wasm/rust/`
2. **Runtime**: `initWasmAuto()` tries to load Rust WASM first
3. **Fallback**: If Rust WASM fails, automatically falls back to Go WASM
4. **Transparency**: Website code doesn't need changes - same API

## Testing

### Build Rust WASM

```bash
# Install wasm-pack if needed
cargo install wasm-pack

# Build WASM
make wasm

# Or via npm script
cd apps/website
npm run ensure:wasm
```

### Verify Build Output

```bash
ls -la apps/website/public/wasm/rust/
# Should see:
# - sruja_wasm.js (wasm-bindgen loader)
# - sruja_wasm_bg.wasm (WASM binary)
# - sruja_wasm.d.ts (TypeScript definitions)
```

### Test Website

```bash
cd apps/website
npm run dev

# Open browser console and check:
# [WASM] ✅ Loaded from /wasm/rust/sruja_wasm_bg.wasm
# or
# [WASM] Rust WASM init failed, falling back to Go WASM
```

### Test Playground

1. Navigate to playground page
2. Enter DSL code
3. Verify diagram renders
4. Check browser console for WASM backend used

## Architecture

```
┌─────────────────┐
│   Website App   │
│  (Astro/React)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  wasmAdapter.ts  │
│  (initWasmAuto)  │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌────────┐ ┌──────────┐
│  Rust  │ │   Go     │
│  WASM  │ │   WASM    │
│ (new)  │ │ (fallback)│
└────────┘ └──────────┘
```

## Benefits

1. **Performance**: Rust WASM is typically faster and smaller
2. **Type Safety**: Better compile-time guarantees
3. **Maintainability**: Single codebase (Rust) for backend
4. **Gradual Migration**: Go fallback ensures no breaking changes
5. **Developer Experience**: Same API, transparent switching

## Migration Path

- **Phase 1** (Current): Rust-first with Go fallback ✅
- **Phase 2** (Future): Remove Go WASM once Rust is stable
- **Phase 3** (Future): Remove Go codebase entirely

## Known Limitations

1. **DotResult Building**: Currently builds from model JSON + DOT string
   - May need refinement for complex layouts
   - Node sizes are defaulted based on element kind

2. **Error Handling**: Rust WASM throws exceptions
   - Wrapped in TypeScript to match Go API
   - Error codes may differ (RUST_5001 vs Go codes)

3. **Function Signatures**: Some functions have different parameter orders
   - `dslToDot` in Rust takes `(dsl, viewLevel?, targetId?)`
   - Go version uses config object
   - Adapter handles conversion

## Next Steps

1. **Test thoroughly** with real DSL files
2. **Monitor performance** vs Go WASM
3. **Gather feedback** from users
4. **Remove Go WASM** once confident in Rust
5. **Update documentation** for Rust-only setup

## Troubleshooting

### Rust WASM Not Loading

- Check `apps/website/public/wasm/rust/` exists
- Verify `wasm-pack` is installed
- Check browser console for errors
- Try hard refresh (Ctrl+Shift+R)

### Fallback to Go

- This is expected if Rust WASM fails
- Check build logs for errors
- Verify Rust toolchain is installed
- Check `make wasm` output

### Type Errors

- Run `npm run typecheck` in website
- Check `sruja_wasm.d.ts` is generated
- Verify TypeScript can find types

---

**Status**: ✅ **INTEGRATION COMPLETE**

The website is now fully integrated with the Rust backend. All features should work with Rust WASM, with automatic fallback to Go if needed.
