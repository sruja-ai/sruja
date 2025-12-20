# Sruja Dependency Graph

This document describes the dependency relationships between packages and modules in the Sruja codebase.

## Overview

Sruja uses a monorepo structure with clear dependency boundaries. Dependencies flow from applications → packages → core utilities.

## Package Dependency Graph

```
┌─────────────────────────────────────────────────────────────┐
│                    Applications Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Designer   │  │   Website    │  │  VS Code     │      │
│  │              │  │              │  │  Extension   │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                 │                  │               │
└─────────┼─────────────────┼──────────────────┼───────────────┘
          │                 │                  │
          ▼                 ▼                  ▼
┌─────────────────────────────────────────────────────────────┐
│                    Packages Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │    Shared   │  │      UI       │  │   Layout    │      │
│  │             │  │              │  │              │      │
│  │  ┌────────┐ │  │              │  │              │      │
│  │  │ Types  │ │  │              │  │              │      │
│  │  │ Utils  │ │  │              │  │              │      │
│  │  │ Builder│ │  │              │  │              │      │
│  │  └────────┘ │  │              │  │              │      │
│  └──────┬──────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                 │                  │               │
│         └─────────────────┴──────────────────┘               │
│                           │                                   │
└───────────────────────────┼───────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Diagram Package                          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Uses: @likec4/diagram, @likec4/core                 │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Detailed Dependencies

### Applications

#### `apps/designer/`
- **Depends on:**
  - `@sruja/shared` - Types, utilities, WASM adapters
  - `@sruja/ui` - UI components
  - `@sruja/diagram` - Diagram rendering
  - `@likec4/diagram` - LikeC4 diagram components

#### `apps/website/`
- **Depends on:**
  - `@sruja/shared` - Types and utilities
  - `@sruja/ui` - UI components
  - Astro framework

#### `apps/vscode-extension/`
- **Depends on:**
  - `@sruja/shared` - Types and utilities
  - VS Code API

### Packages

#### `packages/shared/`
- **Internal Structure:**
  ```
  shared/
  ├── types/          # Type definitions (no dependencies)
  ├── utils/          # Utilities (depends on types)
  ├── builder/        # Builder API (depends on types, utils)
  ├── web/            # WASM adapters (depends on types)
  ├── analytics/      # Analytics (depends on utils)
  └── storage/        # Storage (depends on utils)
  ```
- **External Dependencies:**
  - `@likec4/core` - LikeC4 model types
  - `@likec4/diagram` - LikeC4 diagram components

#### `packages/ui/`
- **Depends on:**
  - `@sruja/shared` - Types and utilities
- **External Dependencies:**
  - React
  - CSS-in-JS or CSS modules

#### `packages/layout/`
- **Depends on:**
  - `@sruja/shared` - Types
- **External Dependencies:**
  - Layout algorithms (dagre, etc.)

#### `packages/diagram/`
- **Depends on:**
  - `@sruja/shared` - Types
  - `@sruja/layout` - Layout algorithms
- **External Dependencies:**
  - `@likec4/diagram` - LikeC4 diagram components
  - `@likec4/core` - LikeC4 model types

## Dependency Rules

### ✅ Allowed

1. **Applications → Packages**: Applications can depend on any package
2. **Packages → Shared**: Packages can depend on `@sruja/shared`
3. **Shared → External**: Shared can depend on external libraries
4. **Types → Nothing**: Types should have minimal dependencies

### ❌ Forbidden

1. **Packages → Applications**: Packages cannot depend on applications
2. **Shared → Packages**: Shared cannot depend on other packages
3. **Circular Dependencies**: No circular dependencies between packages
4. **Types → Utils**: Types should not depend on utilities (use forward references if needed)

## Circular Dependency Prevention

### Current Forward References

In `packages/shared/src/types/core.ts`:
```typescript
// Forward reference to avoid circular dependency
type SrujaExtensions = import("./governance").SrujaExtensions;
```

### Best Practices

1. **Use Type-Only Imports**: `import type { ... }` for type-only dependencies
2. **Forward References**: Use type aliases for cross-module type dependencies
3. **Dependency Inversion**: Use interfaces to break circular dependencies

## External Dependencies

### Go Backend (`pkg/`)
- **No TypeScript dependencies**
- Exports via WASM for browser usage
- Exports via JSON for data exchange

### LikeC4 Integration
- `@likec4/core` - Model types and validation
- `@likec4/diagram` - Diagram rendering components
- Used by: `packages/shared`, `packages/diagram`, `apps/designer`

## Dependency Analysis Tools

### TypeScript
```bash
# Find unused exports
npm run check:unused

# Find unused files
npm run check:unused:files

# Find unused dependencies
npm run check:unused:deps
```

### Dependency Graph Visualization
```bash
# Generate dependency graph (requires madge)
npx madge --extensions ts,tsx --circular packages/
```

## Maintenance

This graph should be updated when:
- New packages are added
- Dependency relationships change
- Circular dependencies are introduced or resolved
- Major refactorings occur

## References

- Monorepo structure: `MONOREPO.md`
- Architecture overview: `ARCHITECTURE.md`
- Package organization: `packages/README.md` (if exists)

