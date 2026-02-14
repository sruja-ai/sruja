# ADR 003: WASM for Browser Integration

## Status

Accepted. Implementation is **Rust → WASM** (previously Go; repo migrated to Rust).

## Context

Sruja's core functionality (parsing, validation, export) is implemented in Rust. We need this available in the VS Code extension (and any browser/Node usage) without:
- Rewriting the Rust code in TypeScript
- Maintaining two implementations
- Losing performance
- Compromising on features

## Decision

We compile the Rust crates to WebAssembly (WASM) and use the generated JS/WASM in the extension and browser.

Architecture:
- **Rust core**: `crates/sruja-language`, `crates/sruja-export`, etc.
- **WASM crate**: `crates/sruja-wasm` (wasm-pack build)
- **Output**: `sruja_wasm.js` + `sruja_wasm_bg.wasm` (consumed by `extension/`)
- **Integration**: Extension loads WASM for preview/export in Node; can be used in browser if needed

## Consequences

### Positive

- **Single Source of Truth**: One implementation in Rust
- **Performance**: WASM is fast, near-native performance
- **Type Safety**: TypeScript adapters provide type safety
- **Consistency**: Same behavior in CLI and browser
- **Maintainability**: No duplicate logic

### Negative

- **Bundle Size**: WASM files add to bundle size
- **Loading Time**: WASM must be loaded before use
- **Debugging**: WASM debugging is more complex
- **Browser Support**: Requires modern browsers (all major browsers support WASM)

### Neutral

- **Development**: Need to rebuild WASM when Rust code changes
- **Testing**: Need to test both Go and WASM integration

## Alternatives Considered

1. **Rewrite in TypeScript**: Rejected - too much work, maintenance burden
2. **Server API**: Rejected - requires backend, adds latency
3. **Shared Library**: Rejected - doesn't work in browser
4. **WASM**: Accepted - best balance of performance and maintainability

## References

- WASM specification: https://webassembly.org/
- wasm-pack: https://rustwasm.github.io/docs/wasm-pack/
- Implementation: `crates/sruja-wasm/`, `extension/src/wasm.ts`

