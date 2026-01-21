# E2E Testing Guide for Rust WASM Backend

## Current Status

The website integration with Rust WASM backend is **complete**, but the first-time WASM build can take 5-10 minutes because it needs to:

1. Install `wasm-pack` (if not present)
2. Install `wasm32-unknown-unknown` Rust target (first time only)
3. Compile the Rust WASM crate
4. Start the Astro dev server

## Quick Test Steps

### 1. Build Rust WASM (First Time - Takes ~5-10 minutes)

```bash
cd apps/website
npm run ensure:wasm
```

This will:

- Install `wasm-pack` if needed
- Install `wasm32-unknown-unknown` target if needed
- Build Rust WASM to `public/wasm/rust/`

### 2. Start Dev Server

```bash
cd apps/website
npm run dev:full
```

Or if WASM is already built:

```bash
npm run dev
```

### 3. Run E2E Tests

Once the server is running on `http://localhost:4321`:

```bash
# Run all E2E tests
npm run test:e2e

# Run Rust WASM specific tests
npm run test:e2e -- rust-wasm-backend.spec.ts

# Run with UI mode (interactive)
npm run test:e2e:ui
```

## Manual Browser Testing

### Test Rust WASM Loading

1. Open browser to `http://localhost:4321/playground`
2. Open browser console (F12)
3. Look for messages:
   - ✅ `[WASM] ✅ Loaded from /wasm/rust/sruja_wasm_bg.wasm` (Rust WASM)
   - ⚠️ `Rust WASM init failed, falling back to Go WASM` (Fallback)
   - ❌ Any WASM errors

### Test Diagram Rendering

1. Navigate to playground
2. Enter test DSL:
   ```sruja
   system TestSystem "Test System" {
     description "Testing Rust WASM backend"
   }
   ```
3. Verify diagram renders (canvas or SVG appears)
4. Check console for any errors

### Test Export Functions

1. Navigate to any example page
2. Look for export buttons
3. Try exporting to different formats (JSON, Mermaid, etc.)
4. Verify exports work without errors

## Automated E2E Test Coverage

The `rust-wasm-backend.spec.ts` test covers:

1. **WASM Loading**: Verifies Rust WASM loads or falls back gracefully
2. **Diagram Rendering**: Tests that diagrams render with WASM backend
3. **Error Handling**: Ensures invalid DSL doesn't crash the app
4. **Export Functionality**: Tests export features work
5. **Function Availability**: Checks WASM functions are registered

## Troubleshooting

### Server Won't Start

- Check if port 4321 is in use: `lsof -ti:4321`
- Check build logs: `tail -f /tmp/website-dev.log`
- Verify Rust toolchain: `rustc --version` and `cargo --version`

### WASM Build Fails

- Install wasm-pack: `cargo install wasm-pack`
- Install wasm target: `rustup target add wasm32-unknown-unknown`
- Check Rust version: `rustc --version` (should be 1.70+)

### WASM Not Loading in Browser

- Check browser console for errors
- Verify files exist: `ls -la apps/website/public/wasm/rust/`
- Try hard refresh: Ctrl+Shift+R (Cmd+Shift+R on Mac)
- Check network tab for failed requests

### Fallback to Go WASM

This is expected if:

- Rust WASM build failed
- Rust WASM files are missing
- Initialization error occurred

The adapter automatically falls back to Go WASM, so the website should still work.

## Expected Behavior

### ✅ Success Indicators

- Browser console shows: `[WASM] ✅ Loaded from /wasm/rust/sruja_wasm_bg.wasm`
- Diagrams render correctly
- No WASM-related errors in console
- Export functions work
- Playground is interactive

### ⚠️ Fallback Indicators

- Console shows: `Rust WASM init failed, falling back to Go WASM`
- Website still works (using Go WASM)
- This is acceptable during migration period

### ❌ Failure Indicators

- Multiple WASM errors in console
- Diagrams don't render
- Export functions fail
- Page crashes or becomes unresponsive

## Next Steps

1. **Wait for WASM build to complete** (first time only)
2. **Run automated tests**: `npm run test:e2e`
3. **Manual verification**: Test playground and examples
4. **Monitor performance**: Compare Rust vs Go WASM load times
5. **Remove Go fallback**: Once Rust is stable

---

**Note**: The first WASM build can take 5-10 minutes. Subsequent builds are much faster (~30 seconds) because dependencies are cached.
