# WASM Setup for VSCode Extension

This document explains how to build and set up the Rust WASM module for the VSCode extension.

## Prerequisites

- Rust toolchain installed (`cargo`, `rustc`)
- `wasm-pack` installed: `cargo install wasm-pack`

## Building WASM for Node.js

The VSCode extension requires WASM built with the `nodejs` target (not `web` target used for the browser).

### Quick Build

```bash
# From repository root
make wasm-nodejs
```

This will:
1. Build Rust WASM with `wasm-pack --target nodejs`
2. Output files to `apps/vscode-extension/wasm-build/`
3. Optimize WASM with `wasm-opt` (if available)

### Manual Build

```bash
# From repository root
cd crates/sruja-wasm
wasm-pack build --target nodejs --out-dir ../../apps/vscode-extension/wasm-build --release
```

## Building the Extension

After building WASM, build the extension:

```bash
cd apps/vscode-extension
npm run vscode:prepublish
```

This will:
1. Compile TypeScript
2. Bundle the extension
3. Copy WASM files from `wasm-build/` to `wasm/rust/`

## File Structure

After building, the extension should have:

```
apps/vscode-extension/
├── wasm/
│   └── rust/
│       ├── sruja_wasm.js          # CommonJS module (Node.js bindings)
│       └── sruja_wasm_bg.wasm     # WASM binary
└── dist/
    └── extension.js               # Bundled extension
```

## Verification

The extension will verify WASM files exist on startup. If files are missing, you'll see an error in the "Sruja WASM LSP" output channel.

To verify manually:

```bash
cd apps/vscode-extension
test -f wasm/rust/sruja_wasm.js && test -f wasm/rust/sruja_wasm_bg.wasm && echo "✅ WASM files present" || echo "❌ WASM files missing"
```

## Troubleshooting

### "Rust WASM files not found" error

1. Ensure you built with `make wasm-nodejs` (not `make wasm`)
2. Check that files exist in `wasm-build/` directory
3. Run `npm run copy-wasm` manually to copy files

### "Failed to load Rust WASM module" error

1. Check that WASM files are in `wasm/rust/` directory
2. Verify files are not corrupted (check file sizes)
3. Check the "Sruja WASM LSP" output channel for detailed error messages

### Extension doesn't activate

1. Check VSCode Developer Console (Help > Toggle Developer Tools)
2. Check "Sruja" output channel for errors
3. Verify TypeScript compiled without errors: `npm run compile`

## Development Workflow

1. Make changes to Rust code in `crates/sruja-wasm/`
2. Rebuild WASM: `make wasm-nodejs`
3. Rebuild extension: `npm run build`
4. Reload VSCode window (Cmd+R / Ctrl+R)

## Differences from Browser WASM

- **Target**: `nodejs` instead of `web`
- **Module format**: CommonJS (`require()`) instead of ES modules
- **File location**: `wasm/rust/` in extension directory
- **Loading**: Uses Node.js `fs` and `require()` instead of `fetch()`
