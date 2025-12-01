# Repository Organization Analysis & Recommendations

## Current Structure

```
sruja-lang/
├── cmd/
│   ├── sruja/          # CLI commands (existing)
│   ├── wasm/           # WASM compilation (existing)
│   └── investigate_d2/ # Investigation tool
├── pkg/
│   ├── language/       # Parser, AST, Printer (existing)
│   ├── export/         # Exporters (json, d2, svg) (existing)
│   ├── engine/         # Validation engine (existing)
│   ├── dx/             # Developer experience (existing)
│   └── config/         # Configuration (existing)
├── learn/              # Hugo documentation site (existing)
│   ├── assets/js/      # Documentation site components (Playground, etc.)
│   ├── static/         # Static assets (WASM files for playground)
│   └── content/        # Hugo content
├── docs/               # Documentation (existing)
│   └── implementation/ # Implementation plans
├── examples/           # Example .sruja files (existing)
├── scripts/            # Utility scripts (existing)
└── tests/              # Tests (existing)
```

**Note**: `learn/` is a Hugo documentation site, not an application. Studio should be separate.

## Analysis: What's Missing

### 1. Go Implementation Gaps

**Missing Packages**:
- ❌ `pkg/export/json/converter.go` - JSON to AST converter (Task 1.2)
- ❌ `pkg/changes/` - Change management (Task 1.5)
  - `changes.go` - Change model
  - `apply.go` - Change application
  - `conflicts.go` - Conflict detection
  - `snapshots.go` - Snapshot generation
- ❌ `pkg/studio/` - Studio API server (Task 4.3)
  - `server.go` - HTTP server
  - `api.go` - API handlers
  - `files.go` - File operations
- ❌ `pkg/wasm/` - WASM compilation (for Public Studio)
  - `parser.go` - Parser WASM
  - `printer.go` - Printer WASM

**Missing CLI Commands**:
- ❌ `cmd/sruja/change.go` - Change commands
- ❌ `cmd/sruja/snapshot.go` - Snapshot commands
- ❌ `cmd/sruja/studio.go` - Studio server command
- ❌ `cmd/sruja/json_to_dsl.go` - JSON to DSL command

### 2. TypeScript Implementation Gaps

**Missing Directories**:
- ❌ `learn/assets/js/viewer/` - Viewer library (Task 3.1-3.8)
  - `core.ts` - Core viewer
  - `layout.ts` - Layout configuration
  - `styling.ts` - Styling
  - `views.ts` - View management
  - `interactivity.ts` - Interactions
  - `export.ts` - Export functionality
  - `file-visualization.ts` - File boundaries
  - `change-visualization.ts` - Change visualization
- ❌ `local-studio/` - Local CLI Studio (Task 4.1-4.6)
  - `src/Studio.tsx` - Main Studio component
  - `src/components/` - Studio components
- ❌ `public-studio/` - Public Studio (Task 4.7)
  - `src/` - React app
  - `wasm/` - WASM modules
  - `public/` - Static files
- ❌ `self-hosted-studio/` - Self-hosted Studio
  - `server/` - Go server
  - `frontend/` - React app

### 3. Infrastructure Gaps

**Missing**:
- ❌ `.github/workflows/` - CI/CD workflows
  - `sruja-validate.yml` - PR validation
  - `sruja-preview.yml` - Preview generation
  - `sruja-main.yml` - Main branch validation
  - `release.yml` - Release workflow
- ❌ `testdata/` - Test data for comprehensive testing
  - `simple/` - Simple architectures
  - `medium/` - Medium architectures
  - `complex/` - Complex architectures
  - `changes/` - Change test data
  - `invalid/` - Invalid input test data

## Recommended Repository Structure

```
sruja-lang/
├── .github/
│   └── workflows/              # CI/CD workflows
│       ├── sruja-validate.yml
│       ├── sruja-preview.yml
│       ├── sruja-main.yml
│       └── release.yml
│
├── cmd/
│   ├── sruja/                  # Main CLI
│   │   ├── main.go
│   │   ├── change.go           # Change commands (NEW)
│   │   ├── snapshot.go         # Snapshot commands (NEW)
│   │   ├── studio.go           # Studio server (NEW)
│   │   ├── lsp.go              # LSP server (NEW)
│   │   ├── json_to_dsl.go      # JSON to DSL (NEW)
│   │   └── ... (existing)
│   ├── wasm/                   # WASM compilation (existing)
│   └── studio-server/          # Self-hosted Studio server (NEW)
│       └── main.go
│
├── pkg/
│   ├── language/               # Parser, AST, Printer (existing)
│   ├── export/
│   │   ├── json/
│   │   │   ├── json.go        # JSON exporter (existing)
│   │   │   ├── converter.go   # JSON to AST (NEW)
│   │   │   └── json_types.go  # JSON types (existing)
│   │   ├── d2/                # D2 exporter (existing)
│   │   ├── svg/               # SVG exporter (existing)
│   │   └── html/              # HTML exporter (NEW)
│   ├── changes/               # Change management (NEW)
│   │   ├── changes.go         # Change model
│   │   ├── apply.go           # Change application
│   │   ├── conflicts.go       # Conflict detection
│   │   ├── snapshots.go       # Snapshot generation
│   │   └── metadata.go        # Change metadata
│   ├── studio/                # Studio API server (NEW)
│   │   ├── server.go          # HTTP server
│   │   ├── api.go             # API handlers
│   │   └── files.go           # File operations
│   ├── lsp/                   # Language Server Protocol (NEW)
│   │   ├── server.go          # LSP server
│   │   ├── handlers.go        # Request handlers
│   │   ├── diagnostics.go    # Error diagnostics
│   │   ├── completion.go     # Code completion
│   │   ├── hover.go          # Hover information
│   │   ├── definition.go     # Go to definition
│   │   ├── references.go     # Find references
│   │   ├── actions.go        # Code actions
│   │   ├── symbols.go        # Document/workspace symbols
│   │   ├── formatting.go    # Formatting
│   │   └── workspace.go     # Workspace management
│   ├── wasm/                  # WASM compilation (NEW)
│   │   ├── parser.go          # Parser WASM
│   │   └── printer.go         # Printer WASM
│   ├── engine/                # Validation engine (existing)
│   ├── dx/                    # Developer experience (existing)
│   └── config/                # Configuration (existing)
│
├── learn/                      # Hugo documentation site (existing)
│   ├── assets/js/              # Documentation site components
│   │   └── ... (existing: Playground, code blocks, etc.)
│   ├── static/
│   │   └── ... (existing)
│   └── ... (existing Hugo content)
│
├── viewer/                     # Viewer library (NEW - separate package)
│   ├── src/
│   │   ├── core.ts
│   │   ├── layout.ts
│   │   ├── styling.ts
│   │   ├── views.ts
│   │   ├── interactivity.ts
│   │   ├── export.ts
│   │   ├── file-visualization.ts
│   │   └── change-visualization.ts
│   ├── dist/
│   │   └── sruja-viewer.js     # Compiled viewer (CDN)
│   ├── package.json
│   └── tsconfig.json
│
├── local-studio/                # Local CLI Studio (NEW - separate app)
│   ├── src/
│   │   ├── components/
│   │   │   ├── Studio.tsx
│   │   │   └── ...
│   │   ├── lib/
│   │   │   └── viewer.ts       # Import from viewer package
│   │   ├── App.tsx
│   │   └── main.tsx
│   ├── public/
│   ├── package.json
│   └── tsconfig.json
│
├── public-studio/              # Public Studio (NEW)
│   ├── src/
│   │   ├── components/
│   │   ├── lib/
│   │   ├── wasm/
│   │   ├── App.tsx
│   │   └── main.tsx
│   ├── public/
│   ├── package.json
│   └── tsconfig.json
│
├── self-hosted-studio/         # Self-hosted Studio (NEW)
│   ├── server/                 # Go server
│   │   └── main.go
│   ├── frontend/               # React app
│   │   ├── src/
│   │   └── package.json
│   └── Dockerfile
│
├── testdata/                   # Test data (NEW)
│   ├── simple/
│   ├── medium/
│   ├── complex/
│   ├── changes/
│   └── invalid/
│
├── tests/                      # Tests (existing)
│   ├── integration/           # Integration tests (NEW)
│   ├── validation/            # Validation tests (NEW)
│   └── ... (existing)
│
├── docs/                       # Documentation (existing)
│   └── implementation/        # Implementation plans
│
├── examples/                   # Example files (existing)
│
└── scripts/                    # Utility scripts (existing)
```

## Implementation Priority

### Phase 1: Core Structure (Week 1)

1. **Create missing Go packages**:
   ```bash
   mkdir -p pkg/export/json
   mkdir -p pkg/changes
   mkdir -p pkg/studio
   mkdir -p pkg/lsp
   mkdir -p pkg/wasm
   mkdir -p pkg/export/html
   ```

2. **Create missing CLI commands**:
   ```bash
   touch cmd/sruja/change.go
   touch cmd/sruja/snapshot.go
   touch cmd/sruja/studio.go
   touch cmd/sruja/lsp.go
   touch cmd/sruja/json_to_dsl.go
   ```

3. **Create test data structure**:
   ```bash
   mkdir -p testdata/{simple,medium,complex,changes,invalid}
   ```

### Phase 2: TypeScript Structure (Week 2-3)

1. **Create viewer library** (separate package):
   ```bash
   mkdir -p viewer/{src,dist}
   cd viewer && npm init
   ```

2. **Create Local Studio** (separate app):
   ```bash
   mkdir -p local-studio/{src/components,public}
   cd local-studio && npm init
   ```

### Phase 3: Additional Studios (Week 8+)

1. **Create Public Studio**:
   ```bash
   mkdir -p public-studio/{src/{components,lib,wasm},public}
   ```

2. **Create Self-hosted Studio**:
   ```bash
   mkdir -p self-hosted-studio/{server,frontend/src}
   ```

### Phase 5: IDE Extensions (Week 11+)

1. **Create VS Code Extension**:
   ```bash
   mkdir -p vscode-extension/{src/{lsp,studio,commands},syntaxes}
   cd vscode-extension && npm init
   ```

2. **Create JetBrains Plugin**:
   ```bash
   mkdir -p jetbrains-plugin/src/{main/{kotlin,resources},test}
   ```

### Phase 5: IDE Extensions (Week 11+)

1. **Create VS Code Extension**:
   ```bash
   mkdir -p vscode-extension/{src/{lsp,studio,commands},syntaxes}
   cd vscode-extension && npm init
   ```

2. **Create JetBrains Plugin**:
   ```bash
   mkdir -p jetbrains-plugin/src/{main/{kotlin,resources},test}
   ```

### Phase 4: Infrastructure (Ongoing)

1. **Create CI/CD workflows**:
   ```bash
   mkdir -p .github/workflows
   # Copy from docs/implementation/CI_CD_WORKFLOWS.md
   ```

## Current Structure Assessment

### ✅ What's Good

1. **Clear separation**: `cmd/` for executables, `pkg/` for packages
2. **Existing packages**: Language, export, engine well-organized
3. **Documentation**: Comprehensive implementation docs
4. **Examples**: Good example files for testing
5. **WASM support**: Already has `cmd/wasm/` structure

### ⚠️ What Needs Improvement

1. **Missing packages**: Changes, Studio API, LSP, WASM compilation
2. **Missing commands**: Change, snapshot, studio, lsp, json-to-dsl
3. **TypeScript organization**: Viewer and Studio need clear structure
4. **Test data**: Need organized test data directory
5. **CI/CD**: No workflows yet
6. **Public Studio**: Separate directory needed
7. **Self-hosted Studio**: Separate directory needed
8. **IDE Extensions**: VS Code and JetBrains plugins needed

## Recommendations

### 1. Immediate Actions

**Create missing directories**:
```bash
# Go packages
mkdir -p pkg/{changes,studio,wasm,export/html}

# CLI commands (files will be created during implementation)
# cmd/sruja/change.go
# cmd/sruja/snapshot.go
# cmd/sruja/studio.go
# cmd/sruja/json_to_dsl.go

# Test data
mkdir -p testdata/{simple,medium,complex,changes,invalid}

# CI/CD
mkdir -p .github/workflows
```

### 2. TypeScript Organization

**Viewer Library** (separate `viewer/` package):
- Standalone npm package
- Can be used by Studio, Public Studio, Self-hosted Studio, and HTML exports
- Exported as `sruja-viewer.js` to CDN (`learn/static/js/` or separate CDN)
- Imported by Studio apps: `import { SrujaViewer } from '@sruja/viewer'` or via CDN

**Local Studio** (separate `local-studio/` app):
- Standalone React app
- Uses viewer library (npm package or CDN)
- Integrates with Go API server (`sruja studio` command)
- Bundled into Go binary via `embed` directive
- **Also called**: CLI Studio (launched via CLI)

**Public Studio** (separate `public-studio/`):
- Standalone React app
- Uses WASM for DSL import/export
- Deployed separately (GitHub Pages, Netlify, etc.)

### 3. Package Organization

**`pkg/changes/`**:
- All change management logic
- Independent of export/engine
- Can be tested separately

**`pkg/studio/`**:
- Studio API server
- File operations
- DSL ↔ JSON conversions
- Separate from CLI commands

**`pkg/wasm/`**:
- WASM compilation targets
- Separate from main CLI
- Used by Public Studio

### 4. Build Process

**Go Build**:
- Main CLI: `go build ./cmd/sruja`
- WASM Parser: `GOOS=js GOARCH=wasm go build -o public-studio/wasm/sruja-parser.wasm ./pkg/wasm/parser`
- WASM Printer: `GOOS=js GOARCH=wasm go build -o public-studio/wasm/sruja-printer.wasm ./pkg/wasm/printer`

**TypeScript Build**:
- Viewer: `cd viewer && npm run build` (outputs to `dist/sruja-viewer.js`)
- Local Studio: `cd local-studio && npm run build` (outputs to `dist/`, embedded in Go binary)
- Public Studio: `cd public-studio && npm run build` (outputs to `dist/`, deployed to static hosting)
- Self-Hosted Studio: `cd self-hosted-studio/frontend && npm run build` (outputs to `dist/`)

## Studio Naming Convention

**Clear distinction between Studio types**:

1. **`local-studio/`** - Local CLI Studio
   - Launched via `sruja studio` command
   - Connected to local Git repo
   - Reads/writes `.sruja` files directly
   - Embedded in Go binary

2. **`public-studio/`** - Public Studio
   - Zero-friction web app
   - No installation, no auth
   - Uses WASM for DSL import/export
   - Deployed to static hosting (sruja.ai/studio)

3. **`self-hosted-studio/`** - Self-Hosted Studio
   - Standalone server + frontend
   - For team sharing and previews
   - No Git connection
   - Customer deploys on their infrastructure

**Benefits of naming**:
- ✅ Clear distinction between different Studio types
- ✅ No confusion about which Studio is which
- ✅ Matches deployment model (local vs. public vs. self-hosted)

## Summary

**Current Status**: ✅ **Well-organized foundation** with clear separation

**Missing**: 
- Change management package (`pkg/changes/`)
- Studio API server package (`pkg/studio/`)
- WASM compilation package (`pkg/wasm/`)
- Viewer library (`viewer/`)
- Local Studio (`local-studio/`)
- Public Studio (`public-studio/`)
- Self-Hosted Studio (`self-hosted-studio/`)
- CI/CD workflows (`.github/workflows/`)
- Test data organization (`testdata/`)

**Recommendation**: Create missing directories now, implement features incrementally. The current structure is solid and can accommodate all planned features with the additions above.

