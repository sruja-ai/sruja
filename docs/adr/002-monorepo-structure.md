# ADR 002: Monorepo Structure with Turbo

## Status

Accepted

## Context

Sruja is a multi-language project (Go backend, TypeScript frontend) with multiple applications and shared packages. We needed a monorepo structure that:
- Supports both Go and TypeScript/JavaScript
- Enables code sharing between applications
- Provides fast builds and testing
- Maintains clear dependency boundaries
- Supports independent versioning if needed

## Decision

We use a monorepo structure with:
- **Turbo**: For build orchestration and caching
- **npm workspaces**: For TypeScript/JavaScript package management
- **Go modules**: For Go package management
- **Clear separation**: `cmd/` for executables, `pkg/` for Go packages, `packages/` for TypeScript packages, `apps/` for applications

Structure:
```
sruja/
├── cmd/          # Go executables (CLI, WASM)
├── pkg/          # Go packages (language, engine, export, etc.)
├── packages/     # TypeScript packages (shared, ui, layout, diagram)
├── apps/         # Applications (designer, website, vscode-extension)
└── examples/     # Example .sruja files
```

## Consequences

### Positive

- **Code Sharing**: Easy to share code between applications
- **Fast Builds**: Turbo caching speeds up CI and local development
- **Consistent Tooling**: Single lint, test, and build setup
- **Dependency Management**: Clear dependency graph
- **Developer Experience**: Single repository, single clone

### Negative

- **Complexity**: More complex than single-package repos
- **Tooling Overhead**: Need to understand Turbo and workspaces
- **CI Complexity**: More complex CI workflows

### Neutral

- **Versioning**: Can still version packages independently if needed
- **Deployment**: Each app can be deployed independently

## Alternatives Considered

1. **Separate Repositories**: Rejected - too much overhead for code sharing
2. **Lerna**: Rejected - Turbo is faster and simpler for our use case
3. **Nx**: Rejected - overkill for our needs, Turbo is sufficient

## References

- Turbo documentation: https://turbo.build/repo/docs
- npm workspaces: https://docs.npmjs.com/cli/v7/using-npm/workspaces
- Monorepo structure: `MONOREPO.md`

