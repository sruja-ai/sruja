# ADR 003: WASM for Browser Integration

## Status

Accepted

## Context

Sruja's core functionality (parsing, validation, export) is implemented in Go. We needed to make this available in browser-based applications (Designer, Website playground) without:
- Rewriting the Go code in TypeScript
- Maintaining two implementations
- Losing performance
- Compromising on features

## Decision

We compile the Go code to WebAssembly (WASM) and provide TypeScript adapters for browser usage.

Architecture:
- **Go Backend**: Core logic in `pkg/` and `cmd/wasm/`
- **WASM Build**: Compiled to `.wasm` files
- **TypeScript Adapters**: `packages/shared/src/web/wasmAdapter.ts` and `packages/shared/src/node/wasmAdapter.ts`
- **Browser Integration**: Load WASM in browser, call via adapters

## Consequences

### Positive

- **Single Source of Truth**: One implementation in Go
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

- **Development**: Need to rebuild WASM when Go code changes
- **Testing**: Need to test both Go and WASM integration

## Alternatives Considered

1. **Rewrite in TypeScript**: Rejected - too much work, maintenance burden
2. **Server API**: Rejected - requires backend, adds latency
3. **Shared Library**: Rejected - doesn't work in browser
4. **WASM**: Accepted - best balance of performance and maintainability

## References

- WASM specification: https://webassembly.org/
- Go WASM: https://pkg.go.dev/cmd/go#hdr-Environment_variables
- Implementation: `cmd/wasm/`, `packages/shared/src/web/wasmAdapter.ts`

