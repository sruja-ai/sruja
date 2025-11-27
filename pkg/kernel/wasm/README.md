# Sruja Kernel WASM

WebAssembly compilation of the Sruja Architecture Kernel for browser-based execution.

## Overview

This package provides the WASM build of the Sruja Kernel, enabling notebook execution in:
- JupyterLab (browser-based)
- VS Code Web
- Custom web applications
- Any browser-based notebook environment

## Build

### Prerequisites

- Go 1.21+ (with WASM support)
- Make (optional, for using Makefile)

### Build WASM Module

Using Make:
```bash
make -f Makefile.wasm wasm
```

Or manually:
```bash
GOOS=js GOARCH=wasm go build -o build/wasm/kernel.wasm ./apps/kernel-wasm
```

### Get wasm_exec.js

The Go WASM runtime file is required. Get it from:

1. **From Go installation:**
   ```bash
   cp $(go env GOROOT)/misc/wasm/wasm_exec.js build/wasm/
   ```

2. **Or download directly:**
   ```bash
   curl -o build/wasm/wasm_exec.js \
     https://raw.githubusercontent.com/golang/go/master/misc/wasm/wasm_exec.js
   ```

## Files

- `apps/kernel-wasm/main.go` - WASM entry point with JS exports
- `pkg/kernel/wasm/wasm_loader.js` - JavaScript loader/wrapper class
- `pkg/kernel/wasm/example.html` - Example HTML page
- `Makefile.wasm` - Build automation

## Usage

### Basic Setup

1. **Include required scripts:**
   ```html
   <script src="wasm_exec.js"></script>
   <script type="module" src="wasm_loader.js"></script>
   ```

2. **Initialize kernel:**
   ```javascript
   import { createKernel } from './wasm_loader.js';

   const go = new Go();
   const kernel = await createKernel('./kernel.wasm', go);
   ```

3. **Execute cells:**
   ```javascript
   const result = await kernel.execute(
     'system Billing {}',
     'cell-1',
     'dsl'
   );
   
   console.log(result);
   // {
   //   success: true,
   //   outputs: [...],
   //   diagnostics: [...],
   //   irChanged: true
   // }
   ```

### API Reference

#### Core Methods

- `execute(code, cellId, cellType)` - Execute a notebook cell
- `query(query)` - Execute a SrujaQL query
- `diagram(target, format)` - Generate a diagram
- `validate(code)` - Validate architecture code
- `exportIR()` - Export architecture IR as JSON
- `importIR(irJSON)` - Import architecture IR from JSON
- `reset()` - Reset kernel state

#### Snapshot Methods

- `createSnapshot(name, description)` - Create a snapshot
- `loadSnapshot(name)` - Load a snapshot
- `listSnapshots()` - List all snapshots

#### Variant Methods

- `createVariant(name, baseSnapshot, description)` - Create a variant
- `applyVariant(name)` - Apply a variant
- `listVariants()` - List all variants

#### LSP Methods

- `autocomplete(code, cursorPos)` - Get completion suggestions
- `inspect(code, cursorPos)` - Get hover information
- `getDiagnostics(cellId)` - Get diagnostics for a cell

### Example

See `example.html` for a complete working example.

## Architecture

### WASM Module Structure

```
Browser
  ↓
wasm_exec.js (Go runtime)
  ↓
kernel.wasm (Sruja Kernel)
  ↓
wasm_loader.js (JS wrapper)
  ↓
Your Application
```

### Memory Management

- Architecture IR is stored in WASM memory
- JSON serialization only for export/import
- Incremental updates for performance
- Memory persists for kernel lifetime

### Performance

- **Startup:** ~100-500ms (depends on WASM size)
- **Cell Execution:** <50ms (typical)
- **Memory:** ~2-20MB (depends on model size)
- **Serialization:** Only when exporting IR

## Integration

### With JupyterLab

1. Build WASM module
2. Serve `kernel.wasm`, `wasm_exec.js`, and `wasm_loader.js`
3. Use JupyterLite kernel adapter
4. Configure kernel spec

### With VS Code Web

1. Build WASM module
2. Create notebook kernel extension
3. Use VS Code Notebook API
4. Initialize WASM kernel

### With Custom App

1. Build WASM module
2. Include loader scripts
3. Initialize kernel
4. Execute cells as needed

## Limitations

### Current Implementation

- ✅ stdio-based (no ZeroMQ/WebSocket in WASM)
- ✅ Single-threaded execution
- ✅ No file system access
- ✅ No network access (by design)

### Browser Compatibility

- ✅ Chrome/Edge (Chromium-based)
- ✅ Firefox
- ✅ Safari (iOS 11+, macOS 10.13+)
- ⚠️ Older browsers may not support WASM

## Troubleshooting

### Kernel Won't Initialize

1. Check browser console for errors
2. Verify `wasm_exec.js` is loaded
3. Check `kernel.wasm` file path
4. Verify CORS settings if loading from different origin

### Execution Errors

1. Check code syntax
2. Review diagnostics in result
3. Verify kernel state (may need reset)

### Performance Issues

1. Consider resetting kernel periodically
2. Use snapshots for large models
3. Minimize JSON serialization

## Development

### Testing WASM Build

```bash
make -f Makefile.wasm test-wasm
```

### Debugging

Enable console logging in browser DevTools. The kernel writes to `console.log` for debugging.

### Building Distribution

```bash
make -f Makefile.wasm dist-wasm
```

This creates a distribution-ready directory with all required files.

## Future Enhancements

- [ ] TinyGo compilation for smaller binary
- [ ] WebWorker support for non-blocking execution
- [ ] Streaming output support
- [ ] Enhanced LSP features
- [ ] Better error messages

## References

- [Go WebAssembly](https://go.dev/doc/wasm/)
- [WebAssembly Documentation](https://webassembly.org/)
- [JupyterLite](https://jupyterlite.readthedocs.io/)

