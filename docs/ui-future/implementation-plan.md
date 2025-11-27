# Implementation Plan

High-level roadmap and phases for the Architecture DSL Platform MVP.

## Overview

**Timeline**: 8-10 weeks  
**Focus**: Bidirectional DSL/diagram editor with LSP support

## Global Composition Engine Enhancements

Enhance the composer to support enterprise boundaries and governance:

- Load domains, contexts, teams from `architecture/` directory
- Bind modules to contexts and enforce exactly one context per module
- Merge cross-module references under their owning contexts
- Generate context map graph from `contextMap` declarations
- Compute ownership and visibility for nodes/components
- Provide qualified identifiers for lookups (`domain.context.module.container.component`)
- Expose boundary-aware validation hooks to the validation engine

## Model Composition Engine

### Core Responsibilities

- Load project artifacts: `domains.sruja`, `teams.sruja`, `contexts.sruja`, `contextmap.sruja`, module DSLs
- Parse DSL → AST → semantic model (Go participle → AST → Go structs)
- Resolve imports (local, relative, GitHub; cache under `.architecture/imports/`)
- Build namespaces: `identity`, `ordering`, `payments`, `identity.auth`, `ordering.checkout`, `payments.corepay`
- Bind modules to contexts and merge containers/components
- Resolve cross-module references (e.g., `checkout.web -> auth.api`)
- Validate IDs, contexts, ownership, boundaries
- Produce `.architecture/global-model.json`

### Architecture (High-Level)

```
            ┌───────────────────────────────────┐
            │        Composition Engine         │
            └───────────────────────────────────┘
                         ▲           ▲
                         │           │
         ┌───────────────┘           └───────────────┐
         │                                             │
┌──────────────────────┐                     ┌───────────────────────┐
│ DSL Parser (Go, participle) │              │    Import Resolver    │
└──────────────────────┘                     └───────────────────────┘
         ▲                                             ▲
         │                                             │
┌──────────────────────┐                     ┌────────────────────────┐
│   AST Transformer    │────→ Go Validate →│   Partial Semantic Models│
└──────────────────────┘                     └────────────────────────┘
                                                    ▲
                                                    │
                                      Merge + Resolve + Validate
                                                    │
                                                    ▼
                             ┌─────────────────────────────────────┐
                             │     Global Architecture Model       │
                             └─────────────────────────────────────┘
                                            │
                                       Write JSON
```

### Folder Structure

```
packages/model-engine/
  src/
    composer/
      composeModel.ts
      loadArtifacts.ts
      importResolver.ts
      moduleComposer.ts
      referenceResolver.ts
      validations/
      mergeStrategies/
      types/
```

### Data Types (Go)

```
type GlobalArchitectureModel struct {
  Version         string
  Domains         map[string]Domain
  Contexts        map[string]BoundedContext
  Teams           map[string]Team
  Modules         map[string]Module
  GlobalRelations []Relationship
  Graph           struct{ Nodes []string; Edges [][2]string }
  ContextMap      []ContextMapEntry
}
```

### Key Components

#### loadArtifacts()

```
async function loadArtifacts(projectRoot: string): Promise<LoadedArtifacts> {
  return {
    domains: await loadDSLFile("shared/domains.sruja"),
    teams: await loadDSLFile("shared/teams.sruja"),
    contexts: await loadDSLFile("shared/contexts.sruja"),
    contextMap: await loadDSLFile("shared/contextmap.sruja"),
    modules: await scanModules(projectRoot)
  };
}
```

#### importResolver()

```
func resolveImport(path, cwd string) (string, error) {
  if isRelative(path) { return resolveLocal(path, cwd), nil }
  if isGithubURL(path) { return resolveGithub(path) }
  return "", fmt.Errorf("unsupported import type")
}
```

#### parseDSL() → AST → model

```
ast := parseWithParticiple(source)
model := buildSemanticModel(ast)
validateModel(model)
```

#### mergeDomainData()

```
function mergeDomains(domains: Domain[]): Record<string, Domain> {
  const out: Record<string, Domain> = {};
  for (const d of domains) {
    if (out[d.id]) throw Error("Duplicate domain");
    out[d.id] = d;
  }
  return out;
}
```

#### bindModulesToContexts()

Each module declares `context: identity.auth`; otherwise inferred.

#### resolveReferences()

```
checkout.web -> auth.api
```

Resolves to `{ from: "checkout.web", to: "auth.api" }` with validation:
1. Source exists
2. Target exists
3. Boundary rules satisfied
4. Record relationship

#### buildGraph()

Graph nodes: all containers; edges: all validated relations.

### Main Entry: composeModel()

```
export async function composeModel(projectRoot: string): Promise<GlobalArchitectureModel> {
  const artifacts = await loadArtifacts(projectRoot);

  const domainModels = artifacts.domains.map(parseDSL);
  const teamModels = artifacts.teams.map(parseDSL);
  const contextModels = artifacts.contexts.map(parseDSL);
  const mapModels = parseDSL(artifacts.contextMap);

  const domains = mergeDomains(domainModels);
  const teams = mergeTeams(teamModels);
  const contexts = mergeContexts(contextModels, domains, teams);

  const moduleModels: Record<string, Module> = {};
  for (const m of artifacts.modules) {
    const parsed = parseDSL(m.source);
    moduleModels[parsed.id] = parsed;
  }

  bindModulesToContexts(moduleModels, contexts);

  const globalRelations = resolveRelationships(moduleModels);
  const graph = buildGraph(globalRelations);

  return {
    version: "1.0",
    domains,
    contexts,
    teams,
    modules: moduleModels,
    globalRelations,
    contextMap: mapModels.mappings,
    graph
  };
}
```

### Stage Behavior

- Parsing: Go participle → AST → semantic model; Go validation validates shape
- Shared registries: `domains`, `contexts`, `teams`
- Modules: load independently with imports resolved
- Context binding: `module.context = contexts["identity.auth"]`
- Reference resolution: `web -> auth.api` → validated relations
- Graph building: collect container nodes and edges

### Composition-Time Validation

- Target exists (`auth.api` must exist)
- Module context exists and belongs to a valid domain
- Owner is a valid `team.*`
- No duplicate modules across imports
- No circular UIDs
- Forbidden cross-context violations (policy-driven)

### Output

- `.architecture/global-model.json`
- `.architecture/artifacts/*.json` (optional)
- `.architecture/imports/*` (cached remote modules)

### Summary

The composer loads and parses DSL, resolves imports, builds unified semantic registries, binds modules to contexts, validates boundaries and ownership, and emits a single canonical global model.

## Reference Resolution Engine

### Responsibilities

- Normalize identifiers and qualify local names (`<module>.<container>`)
- Resolve cross-module and cross-context references
- Enforce domain/context/team boundaries and topology rules
- Produce global relationship entries and graph edges
- Emit precise LSP diagnostics for unknown IDs, invalid namespaces, and boundary violations
- Integrate with composition’s registries and context map

### Core Data Types

```
interface ContainerRef {
  module: string;
  context: string;
  fullyQualifiedId: string;
  type: string;
  location: SourceLocation;
  team?: string;
}

interface Relationship {
  from: string;
  to: string;
  description?: string;
  sourceModule: string;
  context: string;
  location: SourceLocation;
}

interface ResolutionRegistry {
  containers: Record<string, ContainerRef>;
  modules: Record<string, Module>;
  contexts: Record<string, BoundedContext>;
  domains: Record<string, Domain>;
}
```

### Identifier Rules

- Preferred: `<module>.<container>` (e.g., `auth.api`, `checkout.web`)
- Unqualified names in a module are qualified to the current module (`web` → `checkout.web`)

### Algorithm

1. Normalize `from`/`to` using `qualifyId(raw, currentModuleId)`
2. Lookup in `registry.containers`
3. Validate context boundaries using the context map
4. Validate team boundaries via rule plugins
5. Create `Relationship` and append to global relations

### Implementation

```
function qualifyId(raw: string, currentModuleId: string): string {
  return raw.includes('.') ? raw : `${currentModuleId}.${raw}`;
}

export function resolveRelationships(
  moduleModels: Record<string, Module>,
  registry: ResolutionRegistry,
  contextMap: ContextMap,
  rulePlugins: RulePlugin[]
): Relationship[] {
  const resolved: Relationship[] = [];

  for (const module of Object.values(moduleModels)) {
    const currentModuleId = module.id;
    const currentContextId = module.context;

    for (const rel of module.relations) {
      const fromId = qualifyId(rel.from, currentModuleId);
      const toId = qualifyId(rel.to, currentModuleId);

      const from = registry.containers[fromId];
      const to = registry.containers[toId];

      if (!from) throw new DiagnosticError({ message: `Unknown reference: '${rel.from}'`, location: rel.location });
      if (!to) throw new DiagnosticError({ message: `Unknown target: '${rel.to}'`, location: rel.location });

      validateContextBoundary(from.context, to.context, contextMap, rel.location);
      validateTeamRules(from.team, to.team, rulePlugins, rel.location);

      resolved.push({
        from: from.fullyQualifiedId,
        to: to.fullyQualifiedId,
        description: rel.description,
        context: currentContextId,
        sourceModule: currentModuleId,
        location: rel.location
      });
    }
  }

  return resolved;
}
```

### Registry Population

```
function registerContainers(module: Module, registry: ResolutionRegistry) {
  for (const c of module.containers) {
    const fqid = `${module.id}.${c.id}`;
    registry.containers[fqid] = {
      module: module.id,
      context: module.context,
      type: c.type,
      fullyQualifiedId: fqid,
      location: c.location,
      team: module.owner
    };
  }
}
```

### Diagnostics & LSP Integration

- Throw `DiagnosticError { message, location }` on resolution failures
- LSP layer maps errors to VSCode diagnostics using source ranges

### Security Considerations

- Reject references outside imported namespaces
- Disallow path traversal and non-DSL GitHub imports

### Plugins Integration

- Team topology rules executed via `RulePlugin.validateTeamBoundary(fromTeam, toTeam, location)`

### Summary

The resolver guarantees cross-module correctness, boundary-safe relationships, and developer-friendly diagnostics integrated with the composition pipeline.

## MCP Server: Full Backend Implementation Plan

### Overall Architecture

- Goal: Stateless, git-backed, plugin-enabled MCP server exposing tools to LLMs and UI
- Stack: Bun + Elysia HTTP/JSON, Zod schemas, file system adapters, DSL compiler, model composer, graph engine, validation engine, Git adapter, MCP endpoints

### Runtime & Framework Setup (Bun + Elysia)

- Components: Elysia server, `/mcp/*` routes, Zod request/response, middleware (timings, logs, error boundary)
- Tasks:
  - Bootstrap: `bun create elysia mcp-server`
  - Add deps: `bun add zod @elysiajs/static`
  - Register routes: `POST /mcp/read_model`, `POST /mcp/query`, `POST /mcp/update`, `POST /mcp/validate`, `POST /mcp/simulate_failure`
  - Attach `context.projectRoot` to requests

### File System Layer (FS Adapter)

- Goal: Load project from disk/git
- Components: `listFiles`, `readFile`, `exists`, `glob("**/*.sruja")`, `.archconfig.json` loader
- Tasks: Implement `ProjectFileSystem`, detect project root (domains/contexts/teams DSL), normalize paths, cache listing, watcher (later)

### DSL Parser Layer (Ohm + TS)

- Goal: DSL → AST → typed IR
- Components: Ohm grammar, `createAST`, DSL → IR transformer, diagnostics
- Tasks: Implement grammar (v2), `parseDSL(source): DSLNode[]`, error handling, AST → IR mapping, Zod validation on IR

### Import Resolver + Multi-Module Composition

- Goal: Load all `.sruja`, resolve imports, produce single global model
- Components: `ModuleIndex`, `ImportResolver`, `ReferenceResolutionEngine`, module dependency graph, collision detection
- Tasks: Build module index (modules/contexts/domains/teams), resolve imports (relative/root, cycles), merge IRs → GlobalModel, validate (missing context/domain, duplicate modules, missing targets, unresolvable imports)

### Graph Engine (Architecture DAG)

- Goal: Build complete graph of nodes/edges across modules
- Components: node/edge builders, DAG validator, topo sort, cycle detector
- Nodes: domains, contexts, modules, containers, externals
- Edges: dependencies, context-map relations, data flows, journey steps
- Tasks: Build representation `{ nodes: Map; edges: Set }`, add edges from modules/context maps/journeys/imports, cycle detection (Tarjan/DFS), layer mapping (domain/context/module/container)

### Validation Engine

- Goal: Run built-in + plugin rules on global model
- Components: rule registry/executor, plugin sandbox, diagnostics formatter
- Built-in rules: graph structure, domain boundaries, cross-context policies, data storage, ADR status, container constraints
- Tasks: `runRules(model, rules): Diagnostic[]`, implement built-ins (`no-cycles`, `no-cross-domain-without-boundary`, `all-containers-must-have-id`, `no-empty-context`, `all-services-must-be-owned-by-team`), plugin executor (load from `.arch/plugins/*`, Bun `vm`, Zod manifest), diagnostics output

### MCP Endpoints (Tools)

- `/mcp/read_model`: returns global model, paths, metadata, hash
- `/mcp/query`: filters by module/context/domain/relationship/journey/container type
- `/mcp/update`: add module/relationship/container, update props, remove element, add ADR/journey; writes DSL, optional Git commit
- `/mcp/validate`: runs built-in + plugin rules; returns diagnostics[]
- `/mcp/simulate_failure`: graph traversal to find downstream dependents

### End-to-End Data Flow

- Example: Add FraudService and link PaymentService → FraudService
- Flow: read_model → update model (parse, update IR, rewrite DSL, rebuild graph, validate) → validate → diagnostics → summary

### Implementation Timeline (6 Weeks)

- Week 1: Elysia server + FS loader + routing
- Week 2: Ohm parser + AST + IR
- Week 3: Global composer + import resolver
- Week 4: Graph engine + built-in validations
- Week 5: MCP tools + auth + JSON schemas
- Week 6: Plugins + simulation + performance + docs

## Architecture Compiler Pipeline

### Overview

Project → File Loader → Lexer/Parser → AST → IR → Module Graph → Global Model → DAG → Validation → Output

### Stage 1 — Project Loader

- Goal: Collect `.sruja` files, normalize paths, package as `LoadedFiles`
- Output: `{ root: string; files: Map<FilePath, string> }`
- Subcomponents: `detectProjectRoot`, `readProjectFiles`, `glob("**/*.sruja")`, `loadCommonFiles`, `FileNormaliser`

### Stage 2 — Parser (Ohm)

- Goal: DSL source → AST nodes with diagnostics
- Output: `Map<FilePath, ASTNode[]>`
- Components: Ohm grammar, `createParser`, `parseFile(content)`, syntax error reporter

### Stage 3 — AST → IR Transformer

- Goal: AST → typed IR (domains, contexts, teams, modules, relations, journeys, ADRs, imports)
- Output: `IRModel`
- Components: `mapDomain`, `mapContext`, `mapModule`, `mapRelation`, `mapJourney`, `mapADR`, `mapImport`

### Stage 4 — Module Index + Import Resolution

- Goal: Build multi-module knowledge graph with resolved imports
- Output: `ModuleIndex` (modules/contexts/domains/teams/imports/sourceFiles)
- Components: `ModuleCollector`, `ImportResolver`, `SymbolTable`, `IdentifierResolver`

### Stage 5 — Global Model Composer

- Goal: Merge IRs into single Global Model (matches `GlobalModelSchema`)
- Output: `{ domains, contexts, teams, modules, relations, journeys, adrs, projectRoot, version, hash }`
- Components: `ContextBinder`, `DomainBinder`, `OwnerBinder`, `JourneyBinder`, `ADRMapper`, `MetadataMerger`

### Stage 6 — Graph Builder (DAG + Context Graph)

- Goal: Build domain/context/module/container/journey graphs; detect cycles
- Output: `{ nodes: Map<id, NodeInfo>; edges: EdgeInfo[]; cycles: string[][] }`
- Components: `GraphNodeBuilder`, `GraphEdgeBuilder`, `CycleDetector` (Tarjan/DFS), `LayerMapper`

### Stage 7 — Validation Engine

- Goal: Apply built-in and plugin rules over model + graph
- Built-ins: `no-cycles`, `no-cross-domain-violation`, `context-must-belong-to-domain`, `team-must-own-modules`, `service-must-have-container(api)`, `no-missing-dependencies`, `no-duplicate-modules`, `all-modules-must-have-context`, `invalid-import-target`
- Output: `Diagnostic[]` (Zod schema)
- Components: `RuleEngine`, `PluginLoader`, `PluginSandbox`, `DiagnosticsMapper`

### Stage 8 — Output / MCP Integration

- Goal: Package results for MCP endpoints
- Outputs: Global Model, Query results, Updates, Diagnostics, Failure simulations
- Endpoints: `/mcp/read_model`, `/mcp/query`, `/mcp/update`, `/mcp/validate`, `/mcp/simulate_failure`

### Compiler Entrypoint

```
export async function compileProject(projectRoot: string): Promise<{
  files: LoadedFiles;
  ast: Map<FilePath, ASTNode[]>;
  ir: IRModel;
  index: ModuleIndex;
  model: GlobalModel;
  graph: ArchitectureGraph;
  diagnostics: Diagnostic[];
  ok: boolean;
}> { /* impl */ }
```

### Roadmap (6 Weeks)

- Week 1: Loader + Parser
- Week 2: IR Mapper + Import Resolver
- Week 3: Global Composer
- Week 4: Graph Engine
- Week 5: Validation Engine
- Week 6: Integration (MCP tools, performance, caching)

## Onboarding & Friction Reduction Strategy

### Modes

- Simple Mode: D2/C4-style modeling with ~15 keywords, no domains/contexts/imports, zero configuration
- Pro Mode: Unlock domains, contexts, bounded contexts, teams, layers, multi-module imports, plugins, MCP, validations

### Auto-Generated DSL

- Visual editor actions generate DSL blocks and relationships automatically
- Relations and container additions create canonical container-level links

### Smart Defaults

- Auto-create API containers for modules lacking containers
- Auto-link service ↔ database with sensible labels when both exist
- Journey steps resolve to module API containers when targets are module IDs

### Architecture Wizards

- Templates for microservices, event-driven, serverless, mobile+backend, data platforms, CRM/ERP, fintech/e-commerce
- One-click generation populates domains/contexts/modules/relations

### AI-First Onboarding

- Sidebar assistant: “Generate architecture for X” → domains/contexts/modules/containers/relations/journeys
- Immediate diagram rendering and DSL output

### DSL v0.1 (MVP Simplified)

- Minimal syntax: `service`, `db`, basic `A -> B` relations; advanced modeling optionally enabled later

### In-Editor Hints

- Autocomplete for object types, snippets, inline warnings, quick fixes, hover docs

### Zero-Install Options

- Web playground (no login), GitHub integration (sync DSL files), local IndexedDB storage for beginners

### Architecture Libraries

- Component palettes for AWS/GCP/Azure/Kubernetes/Kafka/Redis/PostgreSQL/OpenAPI/DDD/C4; drag-and-drop generates DSL

### Incremental Complexity

- Allow modeling services first; add containers/contexts/domains/teams/ADRs/journeys progressively
## Development Phases

### 🟦 PHASE 1 — FOUNDATION (Week 1–2)

**Focus**: Basic environment + core packages + minimal DSL

#### Deliverables
- Working monorepo with all packages
- Project Format v1 (APF-1.0) structure implemented
- `ProjectProvider` abstraction layer
- DSL parser (Ohm) that can parse v1 grammar
- Model engine with core operations
- **Validation Engine** with semantic and layer rules
- Round-trip tests (DSL → JSON → DSL)

#### Key Technologies
- pnpm workspaces
- Next.js 16 (App Router, Turbopack, React Compiler)
- React 19
- Ohm v17.2.1+ (DSL parser)
- Zod 4.x (validation)
- Bun 1.1+ (backend runtime)

#### Success Criteria
- ✅ Project follows APF-1.0 format (architecture.sruja, .architecture/)
- ✅ `ProjectProvider` interface implemented
- ✅ `LocalFilesystemProjectProvider` working
- ✅ Parse minimal DSL syntax
- ✅ Convert DSL to JSON model
- ✅ Generate `.architecture/model.json`
- ✅ Convert JSON model back to DSL (canonical format)
- ✅ **Validation Engine** validates semantic rules (unknown refs, duplicates, etc.)
- ✅ **Validation Engine** validates layer rules (if enabled)
- ✅ All tests passing

---

### 🟩 PHASE 2 — LSP SERVER (Week 3–4)

**Focus**: Build web-based LSP server for DSL with core language features.

#### Deliverables
- LSP server (Bun/Elysia)
- WebSocket JSON-RPC transport
- Diagnostics (syntax + semantic)
- Completion (keywords + identifiers)
- Hover (node/edge information)
- Optional: Formatting

#### Key Technologies
- vscode-languageserver (LSP protocol)
- vscode-ws-jsonrpc (WebSocket transport)
- Ohm parser (syntax validation)
- Semantic validation engine

#### Success Criteria
- ✅ LSP server responds to diagnostics requests
- ✅ Completion suggests keywords and node names
- ✅ Hover shows node/edge information
- ✅ WebSocket connection stable
- ✅ Handles errors gracefully

---

### 🟩 PHASE 3 — MVP DIAGRAM + SYNC ENGINE WITH LSP (Week 5–7)

**Focus**: Basic UI that can edit both text + diagram with sync, powered by LSP.

#### Deliverables
- React Flow diagram editor with custom nodes
- Monaco DSL editor with LSP integration
- LSP-powered diagnostics, completion, hover
- Bidirectional sync (text ⇄ diagram)
- Undo/redo system
- Auto layout (ELK.js)
- Local storage persistence

#### Key Technologies
- React Flow 12.9.3+ (@xyflow/react)
- Monaco Editor 0.55.1+ with LSP client
- monaco-languageclient (LSP integration)
- Zustand 5.x (state management)
- ELK.js 0.9+ (auto layout)
- React 19 (Server Components, Suspense)

#### Success Criteria
- ✅ Write DSL → see diagram update
- ✅ **LSP shows diagnostics in Monaco gutter**
- ✅ **LSP autocomplete suggests keywords/nodes**
- ✅ **LSP hover shows node information**
- ✅ Drag node → DSL updates
- ✅ Create edge in diagram → DSL updates
- ✅ Undo/redo works
- ✅ Auto layout generates readable diagrams
- ✅ Local storage saves/loads models

---

### 🟧 PHASE 4 — BACKEND + REAL-TIME SYNC (Week 8–10)

**Focus**: Persist model + real-time multi-tab editing.

#### Deliverables
- Elysia REST API (GET/POST model)
- WebSocket server for real-time sync
- Eden Treaty typed client
- Git storage service (save/load from Git)
- Project management API
- Conflict resolution (last write wins)

#### Key Technologies
- Elysia 1.x (backend framework)
- Eden Treaty (type-safe APIs)
- WebSockets (real-time sync)
- simple-git (Git operations)
- Bun 1.1+ (runtime)

#### Success Criteria
- ✅ Multiple tabs can edit same model in real-time
- ✅ Changes persist to Git repository
- ✅ Load model from Git
- ✅ View commit history
- ✅ Type-safe API calls (Eden Treaty)

---

## Post-MVP Phases

### 🟥 PHASE 5 — MULTI-LAYER MODELING (Post-MVP)

Focus: VHLD / HLD / LLD + UI layer switching.

### 🟪 PHASE 6 — VALIDATION ENGINE (Post-MVP)

Focus: Advanced rules engine with extensible validation.

### 🟫 PHASE 7 — GIT WORKSPACES + VERSIONING (Post-MVP)

Focus: Manage multiple architecture proposals with version control UI.

### 🟩 PHASE 8 — CUSTOM LIBRARIES (Post-MVP)

Focus: Importable component libraries.

### 🟦 PHASE 9 — CLI TOOL (Post-MVP)

Focus: Command-line tool for architecture management.

**Commands:**
- `sruja init` - Create new architecture project
- `sruja compile` - Compile DSL to JSON model
- `sruja validate` - Validate architecture

**Why Post-MVP:**
- Core engine (parser, model engine) must be pure functions (enables CLI)
- CLI uses same packages as web/backend
- Can be built after MVP without architectural changes
- See [CLI Specification](./cli-specification.md) for details

### 🟦 PHASE 10 — AI/MCP INTEGRATION (Post-MVP)

Focus: Model querying + AI helpers.

---

## Key Milestones

- **Milestone 1** (Day 10): DSL parser + model engine working
- **Milestone 2** (Day 17): LSP server with diagnostics/completion/hover
- **Milestone 3** (Day 30): Local editor MVP with LSP (text + diagram sync)
- **Milestone 4** (Day 45): Full-stack MVP with real-time sync + Git
- **Milestone 5** (Day 60+): Multi-layer modeling
- **Milestone 6** (Day 90+): Validation engine + versioning

---

## Critical Path

1. Monorepo setup
2. DSL grammar + parser
3. Model schema (Zod)
4. AST → Model transformer
5. Model → DSL serializer
6. Semantic validation engine
7. **DSL testing suite (parser, round-trip, validation)**
8. **LSP server (diagnostics, completion, hover)**
9. **LSP testing suite**
10. **LSP client integration (Monaco)**
11. React Flow diagram rendering
12. Monaco DSL editor with LSP
13. Bidirectional sync (local)
14. Backend REST API
15. WebSocket real-time sync
16. **E2E tests (1-2 critical scenarios)**

---

For detailed task breakdowns, see [Phase Tasks](./phase-tasks.md).

[← Back to Documentation Index](./README.md)
