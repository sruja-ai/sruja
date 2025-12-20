# Monorepo Structure for TypeScript Implementation

This document describes the monorepo structure for the TypeScript/React implementation.

## Overview

The TypeScript implementation is organized as a **Turborepo monorepo** with the following structure:

```
sruja-lang/
├── apps/
│   ├── learn/          # Hugo documentation site
│   │   ├── assets/     # React components (uses @sruja/viewer)
│   │   └── ...
│   └── studio/         # React Studio application
│       ├── src/
│       └── ...
├── packages/
│   ├── shared/         # Shared utilities (@sruja/shared)
│   │   └── src/
│   └── viewer/         # Viewer library (@sruja/viewer)
│       └── src/
├── package.json        # Root with Turborepo
└── turbo.json          # Turborepo configuration
```

## Package Details

### `@sruja/viewer` (packages/viewer/)

**Purpose**: Core library for rendering architecture diagrams using Cytoscape.js

**Location**: `packages/viewer/`

**Exports**:
```typescript
import { createViewer, SrujaViewer } from '@sruja/viewer';
import type { ArchitectureJSON, ViewerOptions } from '@sruja/viewer';
```

**Build**:
```bash
npm run build --filter=@sruja/viewer
# Output: packages/viewer/dist/index.js
```

**Usage**:
- Used by both `@sruja/studio` and `@sruja/learn` apps
- Provides `SrujaViewer` class for Cytoscape.js integration
- Converts ArchitectureJSON to Cytoscape elements
- Handles rendering, styling, and basic interactions

### `@sruja/shared` (packages/shared/)

**Purpose**: Shared TypeScript utilities and types

**Location**: `packages/shared/`

**Exports**:
```typescript
import { formatVersion } from '@sruja/shared';
import type { SrujaConfig } from '@sruja/shared';
```

**Build**:
```bash
npm run build --filter=@sruja/shared
```

### `@sruja/studio` (apps/studio/)

**Purpose**: React application for visual architecture editing

**Location**: `apps/studio/`

**Tech Stack**:
- React 19
- TypeScript
- Vite
- Tailwind CSS (future)

**Dependencies**:
- `@sruja/viewer` - For diagram rendering
- `@sruja/shared` - For shared utilities

**Build**:
```bash
npm run build --filter=@sruja/studio
# Output: apps/studio/dist/
```

**Deployment**: `https://sruja.ai/studio/` (GitHub Pages)

**Development**:
```bash
npm run dev --filter=@sruja/studio
# Runs on http://localhost:5173
```

### `@sruja/learn` (apps/learn/)

**Purpose**: Hugo documentation site with React components

**Location**: `apps/learn/`

**Dependencies**:
- `@sruja/viewer` - For Playground component
- `@sruja/shared` - For shared utilities

**Build**:
```bash
npm run build --filter=@sruja/learn
# Output: apps/learn/public/
```

**Deployment**: `https://sruja.ai/` (GitHub Pages root)

## Development Workflow

### Setup

```bash
# Install all dependencies
npm install
```

### Build All

```bash
# Build all packages and apps
npm run build
```

### Build Specific Package/App

```bash
# Build viewer library
npm run build --filter=@sruja/viewer

# Build studio app
npm run build --filter=@sruja/studio

# Build learn app
npm run build --filter=@sruja/learn
```

### Development Mode

```bash
# Start studio in dev mode
npm run dev --filter=@sruja/studio

# Start learn in dev mode
npm run dev --filter=@sruja/learn
```

## Import Paths

All packages use workspace imports:

```typescript
// In apps/studio/src/App.tsx
import { createViewer } from '@sruja/viewer';
import { formatVersion } from '@sruja/shared';

// In apps/learn/assets/js/components/Playground.tsx
import { createViewer } from '@sruja/viewer';
```

## Path Aliases

Path aliases are configured in `vite.config.ts` and `tsconfig.json`:

```typescript
// apps/studio/vite.config.ts
resolve: {
  alias: {
    '@sruja/shared': path.resolve(__dirname, '../../packages/shared/src'),
    '@sruja/viewer': path.resolve(__dirname, '../../packages/viewer/src'),
  },
}
```

## GitHub Pages Deployment

Both apps are deployed together via GitHub Actions:

- **Workflow**: `.github/workflows/deploy-pages.yml`
- **Learn**: Deployed to root (`https://sruja.ai/`)
- **Studio**: Deployed to `/studio/` (`https://sruja.ai/studio/`)

See [GITHUB_PAGES_SETUP.md](../../GITHUB_PAGES_SETUP.md) for details.

## Adding New Features

### Adding to Viewer Library

1. Edit files in `packages/viewer/src/`
2. Export from `packages/viewer/src/index.ts`
3. Build: `npm run build --filter=@sruja/viewer`
4. Import in apps: `import { newFeature } from '@sruja/viewer'`

### Adding to Studio

1. Edit files in `apps/studio/src/`
2. Import from packages: `import { ... } from '@sruja/viewer'`
3. Build: `npm run build --filter=@sruja/studio`
4. Test: `npm run dev --filter=@sruja/studio`

## Task Implementation Notes

When implementing tasks from `docs/implementation/typescript/`:

- **Viewer tasks (3.x)**: Implement in `packages/viewer/src/`
- **Studio tasks (4.x)**: Implement in `apps/studio/src/`
- **Shared utilities**: Add to `packages/shared/src/`
- **Learn components**: Add to `apps/learn/assets/js/components/`

All tasks should use the package import paths (`@sruja/viewer`, `@sruja/shared`) rather than relative paths.

