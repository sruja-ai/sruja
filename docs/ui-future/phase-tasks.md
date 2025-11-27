# Phase Tasks

Detailed task breakdowns for each implementation phase.

[← Back to Documentation Index](../README.md)

## ✅ PHASE 0 — PROJECT SETUP (1–2 days)

### Task 0.1: Initialize Monorepo Structure
- [ ] Create root directory: `architecture-platform`
- [ ] Initialize pnpm workspace: `pnpm init` with `workspaces` config
- [ ] Create `pnpm-workspace.yaml`:
  ```yaml
  packages:
    - 'apps/*'
    - 'packages/*'
  ```
- [ ] Create root `package.json` with workspace scripts
- [ ] Setup `.gitignore` (node_modules, .next, .turbo, dist, etc.)
- [ ] Initialize Git repository

### Task 0.2: Create App Structure
- [ ] Create `/apps/web` directory
- [ ] Create `/apps/backend` directory
- [ ] Create `/packages/dsl-parser` directory
- [ ] Create `/packages/model-engine` directory
- [ ] Create `/packages/schema` directory
- [ ] Create `/packages/shared` directory
- [ ] Create `/infra/git-storage` directory

### Task 0.3: Setup Next.js 16 App
- [ ] Navigate to `/apps/web`
- [ ] Run: `pnpm create next-app@latest . --typescript --tailwind --app --no-src-dir --import-alias "@/*"`
- [ ] Update `next.config.ts`:
  - Enable Turbopack (default in Next.js 16)
  - Enable React Compiler: `experimental.reactCompiler: true`
  - Enable Partial Pre-Rendering: `experimental.ppr: true`
  - Configure path aliases for monorepo packages
- [ ] Update `tsconfig.json` with path mappings for packages
- [ ] Install additional dependencies:
  ```bash
  pnpm add @xyflow/react@12.9.3 zustand@latest @tanstack/react-query@latest
  pnpm add -D @types/node
  ```

### Task 0.4: Setup Elysia Backend
- [ ] Navigate to `/apps/backend`
- [ ] Initialize: `bun init`
- [ ] Install Elysia: `bun add elysia @elysiajs/eden @elysiajs/websocket zod@^4.0.0`
- [ ] Create `src/index.ts` with basic Elysia server
- [ ] Setup TypeScript config for Bun
- [ ] Create `package.json` scripts (dev, build, start)

### Task 0.5: Setup Shared Packages
- [ ] **packages/schema**: Initialize, install `zod@^4.0.0`
- [ ] **packages/shared**: Initialize, create types/constants
- [ ] **packages/dsl-parser**: Initialize, install `ohm-js@latest`
- [ ] **packages/model-engine**: Initialize, link to schema

### Task 0.6: Development Tooling
- [ ] Install root dev dependencies:
  ```bash
  pnpm add -D -w typescript@latest @types/node eslint@latest prettier@latest
  pnpm add -D -w vitest@latest @vitest/ui @testing-library/react
  ```
- [ ] Install Playwright for E2E tests:
  ```bash
  pnpm add -D -w @playwright/test
  npx playwright install
  ```
- [ ] Create root `tsconfig.json` with base config
- [ ] Setup ESLint config (Next.js 16 compatible)
- [ ] Setup Prettier config
- [ ] Create `.prettierrc` and `.prettierignore`
- [ ] Create `vitest.config.ts` for testing

### Task 0.7: GitHub & CI/CD
- [ ] Create GitHub repository
- [ ] Setup GitHub Actions workflow:
  - Lint on PR
  - Type check
  - Test (Vitest)
  - Build check
- [ ] Create PR template in `.github/pull_request_template.md`
- [ ] Setup branch protection rules

### Task 0.8: Documentation
- [ ] Create root `README.md` with setup instructions
- [ ] Create `CONTRIBUTING.md` with development guidelines
- [ ] Document workspace structure

### Task 0.9: Project Format Structure (APF-1.0)
- [ ] Review [Project Format v1](./project-format.md) specification
- [ ] Create example project structure:
  - `architecture.sruja` (placeholder)
  - `.architecture/` directory
  - `.architecture/model.json` (placeholder)
  - `.architecture/config.json` (template)
- [ ] Setup `.gitignore` for project format:
  - Ignore `.architecture/cache/`
  - Ignore `.architecture/index.json` (optional)
  - Ignore `.architecture/visual.json` (optional, user-specific)
- [ ] Create `ProjectProvider` interface in `/packages/providers/src/provider.ts`:
  ```typescript
  interface ProjectProvider {
    loadFile(path: string): Promise<string>;
    saveFile(path: string, content: string): Promise<void>;
    listFiles(pattern?: string): Promise<string[]>;
    deleteFile(path: string): Promise<void>;
    exists(path: string): Promise<boolean>;
  }
  ```
- [ ] Implement `LocalFilesystemProjectProvider` in `/packages/providers/src/filesystem.ts`
- [ ] Document project format in README

## 🎯 Developer Experience Focus — LSP & CLI

### Phase 1 — LSP Foundation (Weeks 1–4)
- [ ] Initialize Go LSP server skeleton (JSON-RPC over stdio/TCP)
- [ ] Hook participle parser errors into LSP diagnostics (red squiggles)
- [ ] Map AST positions to `TextDocument` ranges
- [ ] Generate `.architecture/index.json` (nodes, links) per `project-format.md` on open/save
- [ ] Implement completion sources (keywords, identifiers from index.json)
- [ ] Implement hover with node/edge metadata

### Phase 2 — CLI & VS Code (Weeks 5–6)
- [ ] Connect VS Code extension to Go LSP server (Monaco/VS Code client)
- [ ] Add `sruja init` to scaffold `architecture.sruja` and `adrs/`
- [ ] Ensure `sruja init` writes `.architecture/config.json` and `.gitignore`
- [ ] Validate project format on `sruja validate` and emit diagnostics

## ✅ PHASE 1 — DSL + MODEL FOUNDATION (7–10 days)

### Task 1.1: Project Format Implementation
- [ ] Ensure project follows APF-1.0 format:
  - `architecture.sruja` at root
  - `.architecture/model.json` for compiled model
  - `.architecture/config.json` for project config
- [ ] Implement model.json generation:
  - After parsing DSL, generate canonical JSON model
  - Store in `.architecture/model.json`
  - Include version, nodes, edges, meta
- [ ] Implement config.json template:
  - DSL version
  - Validation rules
  - Library paths
- [ ] Ensure deterministic serialization:
  - Nodes sorted by ID
  - Edges sorted by (from, to)
  - Metadata keys sorted alphabetically
- [ ] Write tests for project format structure

### Task 1.2: DSL Grammar Definition (Ohm)
- [ ] Create `/packages/dsl-parser/src/grammar.ohm` with v1 grammar
- [ ] Define grammar rules:
  - `Program` → sequence of statements
  - `Statement` → Import | NodeDeclaration | EdgeDeclaration | Comment
  - `NodeDeclaration` → identifier: type "label"? {metadata}?
  - `EdgeDeclaration` → identifier -> identifier "label"?
  - `NodeType` → "service" | "database" | "component"
  - `MetadataBlock` → { key: value, ... }
  - `Value` → StringLiteral | NumberLiteral | BooleanLiteral
- [ ] Test grammar with Ohm playground
- [ ] Document grammar rules in comments

### Task 1.3: Parser Implementation
- [ ] Install `ohm-js@^17.2.1` in dsl-parser package
- [ ] Create `src/parser.ts`:
  - Load grammar file
  - Create parser instance
  - Implement `parse(dsl: string): AST` function
  - Handle parse errors with line numbers
- [ ] Create `src/ast-types.ts` with TypeScript interfaces:
  - `ASTNode`, `NodeDeclaration`, `EdgeDeclaration`, `ImportStatement`
- [ ] Implement error reporting:
  - Syntax error messages
  - Line/column positions
  - Suggestions for common mistakes

### Task 1.4: Schema Package (Zod)
- [ ] Create `/packages/schema/src/model.ts`:
  - Define `NodeSchema` (id, type, label, metadata, position?)
  - Define `EdgeSchema` (from, to, label, metadata?)
  - Define `ModelSchema` (nodes[], edges[], meta?)
- [ ] Create `/packages/schema/src/ast.ts`:
  - Define AST node schemas using Zod
  - Validation for AST structure
- [ ] Export all schemas from `index.ts`
- [ ] Write schema validation tests

### Task 1.5: AST → JSON Model Transformer
- [ ] Create `src/transformer.ts` in model-engine:
  - Implement `astToModel(ast: AST): Model` function
  - Map NodeDeclarations to Model nodes
  - Map EdgeDeclarations to Model edges
  - Extract metadata blocks
  - Generate default positions for nodes
- [ ] Handle edge cases:
  - Duplicate node IDs (error)
  - Edges referencing non-existent nodes (warning)
  - Missing labels (use ID as fallback)
- [ ] Write transformer tests with sample DSL

### Task 1.6: JSON Model → DSL Serializer
- [ ] Create `src/serializer.ts` in model-engine:
  - Implement `modelToDSL(model: Model): string` function
  - Canonical formatting rules:
    - Nodes sorted alphabetically by ID
    - Edges sorted by (from, to)
    - Metadata keys sorted alphabetically
    - One statement per line
    - Consistent indentation
- [ ] Preserve comments (if stored in model)
- [ ] Handle special characters in labels/IDs
- [ ] Write serializer tests

### Task 1.7: Round-Trip Testing
- [ ] Create `src/__tests__/roundtrip.test.ts`:
  - Test DSL → JSON → DSL preserves structure
  - Test with various DSL examples
  - Verify canonical formatting
  - Test edge cases (empty model, single node, etc.)
- [ ] Create test fixtures in `fixtures/`:
  - `simple.sruja` (minimal example)
  - `complex.sruja` (with metadata, multiple edges)
  - `edge-cases.sruja` (special characters, etc.)

### Task 1.8: Model Engine Core Operations
- [ ] Create `/packages/model-engine/src/engine.ts`:
  - `addNode(model, node): Model`
  - `updateNode(model, nodeId, updates): Model`
  - `removeNode(model, nodeId): Model` (also removes connected edges)
  - `addEdge(model, edge): Model`
  - `removeEdge(model, from, to): Model`
  - `normalizeModel(model): Model` (dedupe, validate structure)
- [ ] Implement immutability (return new model instances)
- [ ] Add validation (check node/edge references)
- [ ] Write unit tests for each operation

### Task 1.9: Error Handling & Validation
- [ ] Create `src/errors.ts` with custom error types:
  - `ParseError` (syntax errors)
  - `ValidationError` (semantic errors)
  - `TransformError` (transformation failures)
- [ ] Implement error formatting for UI display
- [ ] Add error recovery suggestions
- [ ] Write error handling tests

### Task 1.10: Validation Engine Implementation
See [Validation Engine Implementation Plan](./validation-engine-implementation.md) for complete details.

**Phase 1: Core Engine Skeleton**
- [ ] Create `/packages/validation/` package structure:
  - `index.ts` - Public API export
  - `engine/` - Core validation engine
  - `rules/` - Built-in validation rules
  - `types/` - Type definitions
  - `utils/` - Helper utilities
- [ ] Implement base interfaces (ValidationInput, ValidationResult, ValidationIssue, etc.)
- [ ] Implement `validateArchitecture()` core function
- [ ] Create ValidationContext with helper utilities

**Phase 2: Rule Execution Framework**
- [ ] Implement rule loader (`loadRules()`)
- [ ] Create rule registry (`builtinRules`)
- [ ] Implement finalizer (`finalizeValidation()`)

**Phase 3: Built-in Rules**
- [ ] Implement semantic rules:
  - `semantic/duplicate-id`
  - `semantic/unknown-reference`
  - `semantic/invalid-type`
  - `semantic/missing-required-prop`
  - `semantic/circular-dependency` (with graph utils)
- [ ] Implement layer rules (if strictLayers enabled):
  - `layers/no-components-at-root`
  - `layers/containers-must-belong-to-system`
  - `layers/components-must-belong-to-container`
  - `layers/external-boundary`
- [ ] Implement best-practice rules (warnings):
  - `bestpractice/no-direct-db-access`
  - `bestpractice/naming-conventions`
  - `bestpractice/avoid-fat-containers`

**Phase 4: Plugin System**
- [ ] Implement plugin loader
- [ ] Add Zod validation for plugin structure
- [ ] Add error handling for plugin exceptions

**Phase 5: Source Mapping**
- [ ] Implement AST → source location mapping
- [ ] Add location helpers to ValidationContext

**Phase 6: Test Suite**
- [ ] Write unit tests for each rule
- [ ] Write integration tests for full pipeline
- [ ] Create golden file tests

**Phase 7: Integration**
- [ ] Integrate with LSP diagnostics (see Task 2.4)
- [ ] Integrate with CLI validate command (post-MVP)
- [ ] Export for editor UI use

### Task 1.11: Testing Suite (DSL Foundation)
- [ ] Setup Vitest in packages:
  - `dsl-parser/__tests__/`
  - `model-engine/__tests__/`
  - `schema/__tests__/`
- [ ] **Grammar Tests** (Ohm):
  - Test valid DSL syntax
  - Test invalid syntax (should fail gracefully)
  - Test edge cases (empty, whitespace, special chars)
  - Use snapshot testing for AST structure
- [ ] **AST Builder Tests**:
  - Test node declarations parsing
  - Test edge declarations parsing
  - Test metadata parsing
  - Test import statements
- [ ] **Model Validation Tests** (Zod):
  - Test schema validation
  - Test invalid models are rejected
  - Test type coercion
- [ ] **Round-Trip Tests** (Critical):
  - `parse → model → serialize → parse → model`
  - Test with various DSL examples
  - Verify canonical formatting
  - Test idempotency
- [ ] **Validation Engine Tests**:
  - Test validation API (`validateArchitecture`)
  - Test semantic rules (unknown reference, duplicate ID, etc.)
  - Test layer rules (if enabled)
  - Test best-practice rules
  - Test plugin loading and execution
  - Test validation config (enabled/disabled rules)
  - Test validation result format (errors, warnings, summary)
  - Test source location mapping
- [ ] **Error Handling Tests**:
  - Test parse errors provide helpful messages
  - Test validation errors include line numbers
  - Test graceful degradation on invalid input
- [ ] Create test fixtures in `__tests__/fixtures/`:
  - `simple.sruja` (minimal example)
  - `complex.sruja` (with metadata, multiple edges)
  - `invalid.sruja` (syntax errors)
  - `edge-cases.sruja` (special characters, etc.)

## ✅ PHASE 2 — LSP SERVER (7–10 days)

**Focus**: Build web-based LSP server for DSL with diagnostics, completion, and hover.

### Task 2.1: LSP Server Setup
- [ ] Install LSP dependencies in apps/backend:
  ```bash
  bun add vscode-languageserver vscode-jsonrpc
  bun add -d @types/vscode-jsonrpc
  ```
- [ ] Create `/apps/backend/src/lsp/server.ts`:
  - Import `vscode-languageserver`
  - Create `LanguageServer` instance
  - Setup connection (stdio or WebSocket)
  - Initialize server capabilities
- [ ] Create `/apps/backend/src/lsp/document.ts`:
  - `TextDocument` class to track document state
  - `updateDocument(uri, text, version)`
  - `getDocument(uri): TextDocument | null`
- [ ] Setup LSP lifecycle handlers:
  - `onInitialize`
  - `onInitialized`
  - `onShutdown`
  - `onExit`

### Task 2.2: WebSocket LSP Transport
- [ ] Install `vscode-ws-jsonrpc` in apps/backend:
  ```bash
  bun add vscode-ws-jsonrpc
  ```
- [ ] Create `/apps/backend/src/lsp/transport.ts`:
  - Setup WebSocket server for LSP
  - Create JSON-RPC connection over WebSocket
  - Handle connection lifecycle
  - Map WebSocket messages to LSP protocol
- [ ] Create Elysia route `/lsp`:
  - Accept WebSocket connections
  - Route to LSP server
  - Handle reconnection logic

### Task 2.3: TextDocument Synchronization
- [ ] Implement `onDidOpenTextDocument`:
  - Store document in memory
  - Parse initial document
  - Send initial diagnostics
- [ ] Implement `onDidChangeTextDocument`:
  - Update document text
  - Increment version
  - Trigger re-validation
- [ ] Implement `onDidCloseTextDocument`:
  - Remove document from memory
  - Cleanup resources
- [ ] Handle incremental updates (LSP change events)

### Task 2.4: Diagnostics Feature
- [ ] Create `/apps/backend/src/lsp/features/diagnostics.ts`:
  - `computeDiagnostics(document): Diagnostic[]`
  - Parse DSL using parser package
  - Collect syntax errors from parser
  - **Use Validation Engine** for semantic validation:
    ```typescript
    import { validateArchitecture } from '@packages/validation-engine';
    
    const result = validateArchitecture({
      dsl: document.getText(),
      ast: parsedAST,
      model: compiledModel,
      project: projectContext,
      config: validationConfig
    });
    ```
- [ ] Map validation results to LSP Diagnostic format:
  - Convert `ValidationIssue[]` to LSP `Diagnostic[]`
  - Map `SourceLocation` to LSP `Range`
  - Map severity (error/warning) to LSP `DiagnosticSeverity`
  - Include rule ID in diagnostic source
- [ ] Implement `onDidChangeTextDocument` handler:
  - Compute diagnostics on change
  - Send `textDocument/publishDiagnostics` notification
- [ ] Handle debouncing (avoid too frequent updates)
- [ ] Write diagnostics tests:
  - Test validation engine integration
  - Test LSP diagnostic mapping
  - Test error/warning display

### Task 2.5: Completion Feature
- [ ] Create `/apps/backend/src/lsp/features/completion.ts`:
  - `provideCompletionItems(document, position): CompletionItem[]`
- [ ] Implement keyword completion:
  - DSL keywords: `service`, `database`, `component`, `context`, `containers`, etc.
  - Node types: `service`, `database`, `component`, `actor`, `system`
  - Edge syntax: `->`
  - Metadata keys: common keys from model
- [ ] Implement identifier completion:
  - Existing node IDs (from parsed model)
  - Suggest based on context (after `->` suggest target nodes)
- [ ] Create completion item details:
  - `label`, `kind`, `detail`, `documentation`
  - Insert text with proper formatting
- [ ] Handle completion triggers:
  - On typing identifiers
  - On typing `:` (for node types)
  - On typing `->` (for edges)
- [ ] Write completion tests

### Task 2.6: Hover Feature
- [ ] Create `/apps/backend/src/lsp/features/hover.ts`:
  - `provideHover(document, position): Hover | null`
- [ ] Detect hover target:
  - Node identifier
  - Node type
  - Edge label
  - Metadata key
- [ ] Build hover content:
  - Node: type, label, metadata summary
  - Edge: source → target, label
  - Type: description of node type
  - Metadata: value and description
- [ ] Format as Markdown:
  - Use LSP `MarkupContent` with markdown
  - Pretty-print metadata
  - Show related information
- [ ] Write hover tests

### Task 2.7: Formatting Feature (Optional)
- [ ] Create `/apps/backend/src/lsp/features/formatting.ts`:
  - `formatDocument(document, options): TextEdit[]`
- [ ] Use model serializer to format:
  - Serialize model to canonical DSL
  - Compute diff between original and formatted
  - Return TextEdit array
- [ ] Handle formatting options:
  - Indentation size
  - Line endings
  - Trailing newlines
- [ ] Register `onDocumentFormatting` handler

### Task 2.8: LSP Server Integration
- [ ] Connect LSP server to Elysia:
  - Create `/lsp` WebSocket endpoint
  - Route WebSocket messages to LSP
  - Handle connection lifecycle
- [ ] Add LSP health check endpoint:
  - `GET /lsp/health` - Check if LSP is running
- [ ] Add logging:
  - Log LSP requests/responses
  - Log errors
  - Debug mode for development
- [ ] Write integration tests:
  - Test WebSocket connection
  - Test diagnostics flow
  - Test completion flow
  - Test hover flow

### Task 2.9: Error Handling & Recovery
- [ ] Handle parser errors gracefully:
  - Don't crash on invalid DSL
  - Return partial diagnostics
  - Suggest fixes
- [ ] Handle connection errors:
  - Reconnection logic
  - Graceful degradation
- [ ] Add error logging:
  - Log to file or console
  - Include stack traces in dev mode
- [ ] Write error handling tests

### Task 2.10: LSP Testing Suite
- [ ] Setup LSP test infrastructure:
  - Mock LSP client using `vscode-languageserver-protocol`
  - Test helpers for sending requests
  - Test fixtures with sample DSL
- [ ] **Diagnostics Tests** (Critical):
  - Test syntax error detection
  - Test semantic error detection (unknown nodes, etc.)
  - Test error messages include line/column
  - Test error severity (error, warning, info)
  - Test diagnostics update on document change
- [ ] **Completion Tests** (Critical):
  - Test keyword completion (service, database, etc.)
  - Test identifier completion (existing node names)
  - Test context-aware completion (after `->` suggests targets)
  - Test completion item details (label, kind, documentation)
- [ ] **Hover Tests** (High Value):
  - Test hover on node identifiers
  - Test hover on node types
  - Test hover on edges
  - Test hover content format (Markdown)
- [ ] **Position Mapping Tests**:
  - Test offset → line/column conversion
  - Test line/column → offset conversion
  - Test multi-byte character handling
- [ ] **Model Syncing Tests**:
  - Test TextDocument sync on open
  - Test TextDocument sync on change
  - Test TextDocument sync on close
- [ ] **Error Handling Tests**:
  - Test graceful handling of invalid DSL
  - Test connection errors
  - Test reconnection logic
- [ ] **Integration Tests**:
  - Test full LSP protocol flow
  - Test WebSocket transport
  - Test with real DSL examples from fixtures

## ✅ PHASE 3 — EDITOR MVP WITH LSP (10–14 days)

### Task 2.1: React Flow Setup
- [ ] Install `@xyflow/react@12.9.3` in apps/web
- [ ] Create `/apps/web/app/editor/page.tsx` (Next.js App Router)
- [ ] Setup React Flow provider:
  - Import `ReactFlow` and `ReactFlowProvider`
  - Configure default viewport
  - Setup background grid
- [ ] Create basic canvas layout (split view: diagram + editor)
- [ ] Add React Flow CSS imports

### Task 2.2: Model Store (Zustand)
- [ ] Create `/apps/web/stores/model-store.ts`:
  - Define Zustand store with model state
  - Actions: `setModel`, `updateNode`, `updateEdge`, etc.
  - Selectors for nodes/edges
- [ ] Create `/apps/web/stores/editor-store.ts`:
  - UI state (selected nodes, viewport, etc.)
  - Editor mode (diagram/text)
- [ ] Type store with TypeScript using schema package types
- [ ] Write store tests

### Task 2.3: Custom Node Components
- [ ] Create `/apps/web/components/nodes/BaseNode.tsx`:
  - Generic node wrapper
  - Handle selection, dragging
  - Display label, type icon
- [ ] Create specific node types:
  - `ServiceNode.tsx` (service icon, styling)
  - `DatabaseNode.tsx` (database icon, styling)
  - `ComponentNode.tsx` (component icon, styling)
- [ ] Create `/apps/web/components/nodes/node-types.ts`:
  - Map node types to components
  - Default node configuration
- [ ] Style nodes with TailwindCSS (modern, clean design)

### Task 2.4: Render Nodes/Edges from Model
- [ ] Create `useModelToFlow()` hook:
  - Convert model nodes to React Flow nodes
  - Convert model edges to React Flow edges
  - Handle positions (use model positions or auto-layout)
- [ ] Connect store to React Flow:
  - Subscribe to model changes
  - Update React Flow nodes/edges reactively
- [ ] Handle initial layout (use ELK.js or simple grid)

### Task 2.5: Diagram Interactions
- [ ] Enable node dragging:
  - Update model positions on drag end
  - Sync back to model store
- [ ] Add node selection:
  - Highlight selected nodes
  - Show selection in UI
- [ ] Implement edge creation:
  - Use React Flow edge creation handlers
  - Validate edge (no self-loops, valid node refs)
  - Add edge to model
- [ ] Add node/edge deletion:
  - Delete key handler
  - Context menu for delete
  - Update model accordingly

### Task 3.6: Monaco Editor Setup
- [ ] Install Monaco dependencies in apps/web:
  ```bash
  pnpm add @monaco-editor/react@latest monaco-editor@0.55.1
  pnpm add monaco-languageclient@latest vscode-ws-jsonrpc@latest
  ```
- [ ] Create `/apps/web/components/editor/DSLEditor.tsx`:
  - Setup Monaco editor instance
  - Configure editor options (theme, font, etc.)
  - Handle editor lifecycle
- [ ] Register DSL language with Monaco:
  ```typescript
  monaco.languages.register({ id: "architecture-dsl" });
  ```
- [ ] Create `/apps/web/lib/monaco/grammar.ts`:
  - Define Monarch grammar for DSL syntax highlighting
  - Basic keywords, strings, identifiers
  - Register with Monaco (fallback if LSP unavailable)
- [ ] Setup language configuration:
  - Auto-indentation
  - Bracket matching
  - Word-based suggestions
  - Comments configuration

### Task 3.7: LSP Client Integration
- [ ] Create `/apps/web/lib/lsp/client.ts`:
  - Setup Monaco Language Client
  - Configure WebSocket connection to LSP server
  - Handle connection lifecycle
- [ ] Create LSP client wrapper:
  ```typescript
  const client = new MonacoLanguageClient({
    name: "Architecture DSL",
    clientOptions: {
      documentSelector: [{ language: "architecture-dsl" }],
    },
    connectionProvider: {
      get: () => createWebSocketConnection("ws://localhost:3001/lsp"),
    },
  });
  ```
- [ ] Initialize LSP client:
  - Start client on editor mount
  - Stop client on unmount
  - Handle reconnection
- [ ] Connect Monaco editor to LSP:
  - Register language client with Monaco
  - Enable LSP features (diagnostics, completion, hover)

### Task 3.8: DSL Editor Integration
- [ ] Connect editor to model store:
  - Load DSL from model (serialize)
  - Display in Monaco editor
- [ ] Implement text change handler:
  - Debounce text changes (300ms)
  - Parse DSL on change
  - Update model on successful parse
  - LSP handles diagnostics automatically
- [ ] Error display:
  - LSP diagnostics shown in Monaco gutter (automatic)
  - Display error messages below editor (optional)
  - Highlight error lines (automatic via LSP)
- [ ] Preserve cursor position:
  - Save cursor position before update
  - Restore after DSL regeneration
- [ ] Handle LSP diagnostics:
  - Show inline errors/warnings
  - Update error count in UI
  - Prevent sync if critical errors exist

### Task 3.9: Bidirectional Sync Engine
- [ ] Create `/apps/web/lib/sync/sync-engine.ts`:
  - `syncTextToModel(text: string): Model | Error`
  - `syncModelToText(model: Model): string`
  - Track sync direction to prevent loops
- [ ] Implement sync state machine:
  - IDLE → PARSING → UPDATING → IDLE
  - Handle concurrent updates
- [ ] Add sync debouncing:
  - Debounce text changes
  - Batch model updates
- [ ] Write sync engine tests

### Task 3.10: Undo/Redo System
- [ ] Create `/apps/web/lib/history/history.ts`:
  - Implement history stack (undo/redo)
  - Store model snapshots
  - Limit history size (50 entries)
- [ ] Integrate with model store:
  - Save snapshot before each mutation
  - Implement `undo()` and `redo()` actions
- [ ] Add keyboard shortcuts:
  - Cmd/Ctrl+Z for undo
  - Cmd/Ctrl+Shift+Z for redo
- [ ] Add undo/redo buttons to UI

### Task 3.11: Auto Layout (ELK.js)
- [ ] Install `elkjs@latest` in apps/web
- [ ] Create `/apps/web/lib/layout/auto-layout.ts`:
  - Convert model to ELK graph format
  - Run ELK layout algorithm
  - Convert back to model positions
- [ ] Add "Auto Layout" button to UI
- [ ] Handle layout options:
  - Direction (top-to-bottom, left-to-right)
  - Spacing configuration
- [ ] Preserve manual positions when possible

### Task 3.12: Local Storage Persistence
- [ ] Create `/apps/web/lib/storage/local-storage.ts`:
  - `saveModel(model: Model, key: string): void`
  - `loadModel(key: string): Model | null`
- [ ] Auto-save on model changes (debounced)
- [ ] Add "Save" and "Load" buttons
- [ ] Handle storage errors gracefully
- [ ] Add storage quota warnings

### Task 3.13: UI Polish & Error Handling
- [ ] Create error boundary component
- [ ] Add loading states for async operations
- [ ] Implement toast notifications for errors/success
- [ ] Add keyboard shortcuts help modal
- [ ] Responsive design (mobile-friendly)
- [ ] Accessibility improvements (ARIA labels, keyboard nav)

### Task 3.14: Component & Sync Testing (Optional for MVP)
- [ ] Setup `@testing-library/react` in apps/web
- [ ] **Component Tests** (Optional):
  - Test custom node components render correctly
  - Test node metadata editing
  - Test diagram pan/zoom doesn't break state
  - Skip testing React Flow itself (mature library)
- [ ] **Sync Engine Tests**:
  - Test text → model → diagram flow
  - Test diagram → model → text flow
  - Test debouncing prevents excessive updates
  - Test sync direction tracking (prevents loops)
- [ ] **Undo/Redo Tests**:
  - Test history stack operations
  - Test undo/redo limits
  - Test snapshot creation

## ✅ PHASE 4 — BACKEND + GIT STORAGE (7–10 days)

### Task 4.1: Elysia Server Setup
- [ ] Create `/apps/backend/src/index.ts`:
  - Initialize Elysia server
  - Setup CORS for Next.js frontend
  - Configure error handling
  - Add request logging
- [ ] Create `/apps/backend/src/routes/` directory structure
- [ ] Setup environment variables:
  - `PORT` (default: 3001)
  - `NODE_ENV`
  - `GIT_REPO_PATH` (for Git storage)
- [ ] Create `package.json` scripts (dev, build, start)

### Task 4.2: REST API Endpoints
- [ ] Create `/apps/backend/src/routes/model.ts`:
  - `GET /api/model/:projectId` - Get model by project ID
  - `POST /api/model/:projectId` - Update model
  - Use Zod schemas for validation
  - Return typed responses with Eden Treaty
- [ ] Create `/apps/backend/src/routes/projects.ts`:
  - `GET /api/projects` - List all projects
  - `POST /api/projects` - Create new project
  - `DELETE /api/projects/:id` - Delete project
- [ ] Add error handling middleware
- [ ] Write API tests with Vitest

### Task 4.3: Eden Treaty + TanStack Query Integration
- [ ] Install `@elysiajs/eden` in apps/web:
  ```bash
  pnpm add @elysiajs/eden
  ```
- [ ] Create `/apps/web/lib/api/client.ts`:
  - Initialize Eden Treaty client with backend URL
  - Type-safe API calls with automatic type inference
  - Error handling wrapper (throws errors for TanStack Query)
  ```typescript
  import { treaty } from '@elysiajs/eden'
  import type { App } from '@apps/backend/src/index'
  
  export const api = treaty<App>('http://localhost:3001')
  
  // Wrapper to throw errors for TanStack Query
  export async function apiCall<T>(
    fn: () => Promise<{ data: T; error?: any }>
  ): Promise<T> {
    const result = await fn()
    if (result.error) throw result.error
    return result.data
  }
  ```
- [ ] Create `/apps/web/lib/api/hooks.ts`:
  - TanStack Query hooks using Eden Treaty client
  - Proper error handling for TanStack Query's `onError`
  ```typescript
  import { useQuery, useMutation } from '@tanstack/react-query'
  import { api, apiCall } from './client'
  
  export function useModel(projectId: string) {
    return useQuery({
      queryKey: ['model', projectId],
      queryFn: () => apiCall(() => 
        api.api.model[projectId].get()
      ),
    })
  }
  
  export function useUpdateModel() {
    return useMutation({
      mutationFn: ({ projectId, model }: { projectId: string; model: Model }) =>
        apiCall(() => 
          api.api.model[projectId].post(model)
        ),
    })
  }
  ```
- [ ] Export types from backend:
  - Eden Treaty automatically infers types from Elysia backend
  - No manual type definitions needed
  - Full type safety across client/server boundary

### Task 4.4: WebSocket Server (Separate from LSP)
- [ ] Install `@elysiajs/websocket` in apps/backend
- [ ] Create `/apps/backend/src/websocket/index.ts`:
  - Setup WebSocket plugin
  - Handle connection lifecycle
  - Store active connections per project
- [ ] Implement message types:
  ```typescript
  type WSMessage = 
    | { type: 'MODEL_UPDATED', payload: Model }
    | { type: 'NODE_CHANGED', payload: { nodeId, updates } }
    | { type: 'EDGE_CHANGED', payload: { from, to, updates } }
    | { type: 'SET_MODEL', payload: Model }
  ```
- [ ] Broadcast to all clients in same project
- [ ] Handle client disconnection

### Task 4.5: WebSocket Client Hook
- [ ] Create `/apps/web/hooks/useWebSocket.ts`:
  - Connect to WebSocket server
  - Handle reconnection logic
  - Subscribe to project room
  - Emit messages to server
- [ ] Integrate with model store:
  - Apply incoming model updates
  - Emit local changes to server
  - Handle conflicts (last write wins for MVP)
- [ ] Add connection status indicator in UI
- [ ] Handle offline/online states

### Task 4.6: Project Format Integration
- [ ] Use `ProjectProvider` interface for all file operations:
  - Load `architecture.sruja` via provider
  - Save `architecture.sruja` via provider
  - Load `.architecture/model.json` via provider
  - Save `.architecture/model.json` via provider
- [ ] Implement project initialization:
  - Create `architecture.sruja` template
  - Create `.architecture/` directory
  - Create `.architecture/config.json` with defaults
  - Generate initial `model.json`
- [ ] Ensure project format compliance:
  - Validate APF-1.0 structure
  - Check required files exist
  - Verify file locations

### Task 4.7: Git Storage Service (Post-MVP, Optional for MVP)
- [ ] Install `simple-git@latest` in apps/backend (optional for MVP)
- [ ] Create `/packages/providers/src/git.ts`:
  - Implement `GitProjectProvider` (post-MVP)
  - Use GitHub/GitLab APIs or git clone
  - Support read/write operations
- [ ] Note: For MVP, use `LocalFilesystemProjectProvider` only
- [ ] Git integration can be added post-MVP without changing core engine

### Task 4.8: Git Integration API (Post-MVP)
- [ ] Create `/apps/backend/src/routes/git.ts`:
  - `POST /api/git/init` - Initialize Git repo
  - `GET /api/git/history/:projectId` - Get commit history
  - `GET /api/git/diff/:projectId/:commit1/:commit2` - Get diff
  - `POST /api/git/restore/:projectId/:commit` - Restore version
- [ ] Add Git configuration:
  - User name/email for commits
  - Remote repository URL
  - Branch management
- [ ] Handle merge conflicts:
  - Detect conflicts on pull
  - Simple "last write wins" strategy for MVP
  - Return conflict info to client

### Task 4.9: Project Management
- [ ] Create `/apps/backend/src/services/project-service.ts`:
  - Create project directory structure
  - Manage project metadata
  - List all projects
- [ ] Store project config in `project.json`:
  ```json
  {
    "id": "project-1",
    "name": "My Architecture",
    "createdAt": "...",
    "updatedAt": "..."
  }
  ```
- [ ] Add project validation
- [ ] Implement project cleanup (delete)

### Task 4.10: Real-time Sync Logic
- [ ] Create `/apps/backend/src/services/sync-service.ts`:
  - Track model versions (optimistic locking)
  - Handle concurrent updates
  - Merge strategies
- [ ] Implement delta updates:
  - Calculate diff between model versions
  - Send only changed nodes/edges
  - Reduce WebSocket payload size
- [ ] Add conflict resolution:
  - Detect conflicts
  - Apply last-write-wins
  - Log conflicts for debugging

### Task 4.11: Error Handling & Logging
- [ ] Setup structured logging:
  - Use Bun's built-in logger or pino
  - Log API requests, WebSocket events
  - Log Git operations
- [ ] Create error types:
  - `GitError`, `ValidationError`, `SyncError`
- [ ] Add error recovery:
  - Retry failed Git operations
  - Handle corrupted model files
  - Graceful degradation

### Task 4.12: Security & Validation
- [ ] Add input validation:
  - Validate project IDs (prevent path traversal)
  - Sanitize file paths
  - Validate model structure
- [ ] Add rate limiting:
  - Limit API requests per IP
  - Limit WebSocket messages
- [ ] Add authentication (future):
  - JWT tokens
  - Project access control
- [ ] Add request size limits

### Task 4.13: Backend Testing Suite
- [ ] **API Endpoint Tests**:
  - Test GET /api/model/:projectId
  - Test POST /api/model/:projectId
  - Test project management endpoints
  - Test error responses (404, 400, 500)
  - Test validation (Zod schemas)
- [ ] **WebSocket Sync Tests**:
  - Test connection lifecycle
  - Test message broadcasting
  - Test conflict resolution (last write wins)
  - Test reconnection logic
  - Test multiple clients per project
- [ ] **Git Storage Tests**:
  - Test repository initialization
  - Test model save/load
  - Test commit creation
  - Test history retrieval
  - Test error handling (corrupted files, etc.)
- [ ] **Integration Tests**:
  - Test full workflow: create → edit → save → load
  - Test concurrent edits
  - Test Git operations with real repos

### Task 4.14: E2E Testing (1-2 Critical Scenarios)
- [ ] Setup Playwright in apps/web:
  - Install Playwright
  - Configure for Next.js
  - Create E2E test directory
- [ ] **Critical E2E Test 1**: Simple DSL → Diagram
  - Load simple DSL file
  - Verify diagram renders correctly
  - Verify nodes and edges appear
- [ ] **Critical E2E Test 2**: Diagram Edit → DSL Update
  - Load model with diagram
  - Drag a node
  - Verify DSL updates automatically
  - Verify LSP diagnostics appear
- [ ] (Optional) Additional E2E tests:
  - LSP autocomplete works
  - LSP hover shows information
  - Git save/load workflow

### Task 4.15: Documentation
- [ ] Create API documentation:
  - OpenAPI/Swagger spec
  - WebSocket protocol docs
- [ ] Add example requests/responses
- [ ] Document testing strategy
- [ ] Create test coverage report
