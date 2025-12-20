# Getting Started with TypeScript Implementation

This guide helps new developers get started with the TypeScript/React implementation.

## Prerequisites

- Node.js >= 18.0.0
- npm >= 10.2.4
- Go >= 1.25 (for CLI and JSON export)
- Basic knowledge of TypeScript, React, and Cytoscape.js

## Setup

### 1. Clone and Install

```bash
# Clone the repository
git clone <repo-url>
cd sruja-lang

# Install all dependencies
npm install

# Verify setup
npm run build
```

### 2. Understand the Structure

See [MONOREPO.md](MONOREPO.md) for detailed monorepo structure.

**Quick Overview**:
- `packages/viewer/` - Core visualization library (Cytoscape.js)
- `packages/shared/` - Shared utilities
- `apps/studio/` - React Studio application
- `apps/learn/` - Hugo documentation site

### 3. Development Workflow

```bash
# Start Studio in dev mode
npm run dev --filter=@sruja/studio

# Start Learn in dev mode
npm run dev --filter=@sruja/learn

# Build specific package
npm run build --filter=@sruja/viewer
```

## JSON Export Format

The Go CLI exports architecture to JSON. The format is:

```json
{
  "metadata": {
    "name": "Architecture Name",
    "version": "1.0.0",
    "generated": "2025-12-01T15:32:27+05:30"
  },
  "architecture": {
    "systems": [...],
    "persons": [...],
    "relations": [...],
    "containers": [...],
    "components": [...],
    "datastores": [...],
    "queues": [...],
    "scenarios": [...],
    "requirements": [...],
    "adrs": [...],
    "deployment": [...]
  },
  "navigation": {
    "levels": ["level1", "level2", "level3"],
    "scenarios": [...]
  }
}
```

**Generate test JSON**:
```bash
./bin/sruja export json examples/c4_full.sruja > test.json
```

## Current Status

### ✅ Ready to Use

1. **Monorepo Structure** - Fully set up with Turborepo
2. **Viewer Package** - Basic structure exists (`packages/viewer/`)
3. **Studio App** - Scaffold exists (`apps/studio/`)
4. **Shared Package** - Basic utilities (`packages/shared/`)
5. **JSON Export** - Working and tested
6. **Documentation** - All tasks documented with updated paths

### ⚠️ Known Issues / TODOs

1. **Type Mismatch**: The TypeScript types in `packages/viewer/src/types.ts` are simplified and don't fully match the Go JSON export structure. They need to be updated to match `pkg/export/json/json_types.go`.

2. **Viewer Implementation**: The viewer class exists but needs:
   - Complete type definitions matching Go JSON
   - Full conversion logic for all element types
   - Layout and styling implementation
   - Interaction handlers

3. **Studio App**: Currently just a scaffold. Needs full implementation.

## Starting a Task

### Step 1: Choose a Task

See [README.md](README.md) for all available tasks. Recommended starting points:
- **Task 3.1** (Viewer Core) - If you want to work on visualization
- **Task 4.1** (Studio Core) - If you want to work on the UI
- **Task 3.2** (Layout) - If you want to work on graph layouts

### Step 2: Read the Task File

Each task has a detailed file in `docs/implementation/typescript/`:
- `task-3.1-viewer-core.md` - Viewer library
- `task-4.1-studio-core.md` - Studio app
- etc.

### Step 3: Update Types First

**Important**: Before implementing features, update the TypeScript types to match the Go JSON export:

1. Check `pkg/export/json/json_types.go` for the actual structure
2. Update `packages/viewer/src/types.ts` to match
3. Ensure all fields are properly typed

### Step 4: Implement

Follow the task documentation and implement the feature.

### Step 5: Test

```bash
# Build and test
npm run build --filter=@sruja/viewer
npm run build --filter=@sruja/studio

# Test in browser
npm run dev --filter=@sruja/studio
# Open http://localhost:5173
```

## Import Paths

Always use workspace package imports:

```typescript
// ✅ Correct
import { createViewer } from '@sruja/viewer';
import { formatVersion } from '@sruja/shared';

// ❌ Wrong
import { createViewer } from '../../packages/viewer/src';
```

## Testing with Real Data

1. Generate JSON from a `.sruja` file:
   ```bash
   ./bin/sruja export json examples/c4_full.sruja > test-data.json
   ```

2. Load in your component:
   ```typescript
   import testData from './test-data.json';
   const viewer = createViewer({ container: '#app', data: testData });
   await viewer.init();
   ```

## Common Issues

### Type Errors

If you see type errors, check that:
1. Types match Go JSON export structure
2. You're using workspace imports (`@sruja/viewer`)
3. Packages are built (`npm run build`)

### Build Errors

```bash
# Clean and rebuild
npm run clean
npm install
npm run build
```

### Import Errors

Make sure path aliases are configured in:
- `apps/studio/vite.config.ts`
- `apps/studio/tsconfig.json`

## Next Steps

1. Read [MONOREPO.md](MONOREPO.md) for detailed structure
2. Read [README.md](README.md) for task overview
3. Pick a task and read its detailed file
4. Start implementing!

## Questions?

- Check existing code in `packages/viewer/src/` for examples
- Review Go JSON export in `pkg/export/json/`
- See task documentation files for detailed requirements

