# Model Composition Engine

**Status**: Core Engine  
**Pillar**: All (foundational)

[← Back to Engines](../README.md)

## Overview

The Model Composition Engine is a **core subsystem** that takes multiple DSL files (modules, domains, contexts, teams) and produces a unified global architecture model.

## Purpose

The engine:
- Loads all project artifacts (domains, teams, contexts, modules)
- Parses DSL → AST → semantic model
- Resolves imports (local, relative, GitHub, registry)
- Builds namespaces
- Binds modules to contexts
- Merges all containers & components
- Resolves cross-module references
- Validates all IDs, contexts, ownership
- Builds final JSON global model

## Architecture

```
            ┌───────────────────────────────────┐
            │        Composition Engine         │
            └───────────────────────────────────┘
                         ▲           ▲
                         │           │
         ┌───────────────┘           └───────────────┐
         │                                             │
┌──────────────────────┐                     ┌───────────────────────┐
│   DSL Parser (Ohm)   │                     │    Import Resolver    │
└──────────────────────┘                     └───────────────────────┘
         ▲                                             ▲
         │                                             │
┌──────────────────────┐                     ┌────────────────────────┐
│   AST Transformer    │────→ Zod Validate →│   Partial Semantic Models│
└──────────────────────┘                     └────────────────────────┘
                                                    ▲
                                                    │
                                      Merge + Resolve + Validate
                                                    │
                                                    ▼
                             ┌─────────────────────────────────────┐
                             │     Global Architecture Model       │
                             └─────────────────────────────────────┘
                                            |
                                       Write JSON
```

## Data Types

### Global Model

```typescript
export interface GlobalArchitectureModel {
  version: string;

  domains: Record<string, Domain>;
  contexts: Record<string, BoundedContext>;
  teams: Record<string, Team>;

  modules: Record<string, Module>;

  globalRelations: Relationship[];

  graph: {
    nodes: string[];
    edges: [string, string][];
  };

  contextMap: ContextMapEntry[];
}
```

## Key Components

### 1. loadArtifacts()

Loads all DSL files (local and imported):

```typescript
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

### 2. importResolver()

Handles:
- `import "./local.sruja"`
- `import "github.com/org/repo"`
- Caching to `.architecture/imports/`

```typescript
async function resolveImport(path: string, cwd: string): Promise<string> {
  if (isRelative(path)) return resolveLocal(path, cwd);
  if (isGithubUrl(path)) return await resolveGithub(path);
  throw new Error("Unsupported import type");
}
```

### 3. parseDSL() → AST → model

Uses **Ohm grammar**:

```typescript
const ast = parseWithOhm(source);
const model = buildSemanticModel(ast); // team, domain, context, module
ZodModel.parse(model);
```

### 4. mergeDomainData()

Collect all `domain {...}` blocks:

```typescript
function mergeDomains(domains: Domain[]): Record<string, Domain> {
  const out = {};
  for (const d of domains) {
    if (out[d.id]) throw Error("Duplicate domain");
    out[d.id] = d;
  }
  return out;
}
```

### 5. bindModulesToContexts()

Every module must declare:

```sruja
context: identity.auth
```

Or engine tries to infer.

### 6. resolveReferences()

Convert:

```sruja
checkout.web -> auth.api
```

into:

```typescript
{ from: "checkout.web", to: "auth.api" }
```

**Steps:**
1. Validate source exists
2. Validate target exists
3. Validate context boundary rules
4. Record relationship

### 7. buildGraph()

```typescript
nodes: all containers
edges: all relations
```

## Main Function: composeModel()

**This is the central entry point:**

```typescript
export async function composeModel(projectRoot: string): Promise<GlobalArchitectureModel> {
  const artifacts = await loadArtifacts(projectRoot);

  // 1. Parse & validate shared definitions
  const domainModels = artifacts.domains.map(parseDSL);
  const teamModels = artifacts.teams.map(parseDSL);
  const contextModels = artifacts.contexts.map(parseDSL);
  const mapModels = parseDSL(artifacts.contextMap);

  // 2. Build shared registries
  const domains = mergeDomains(domainModels);
  const teams = mergeTeams(teamModels);
  const contexts = mergeContexts(contextModels, domains, teams);

  // 3. Parse modules
  const moduleModels = {};
  for (const m of artifacts.modules) {
    const parsed = parseDSL(m.source);
    moduleModels[parsed.id] = parsed;
  }

  // 4. Bind modules to contexts
  bindModulesToContexts(moduleModels, contexts);

  // 5. Resolve references
  const globalRelations = resolveRelationships(moduleModels);

  // 6. Build graph
  const graph = buildGraph(globalRelations);

  // 7. Assemble final global model
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

## Processing Stages

### Stage 1 — Parsing
- Ohm grammar converts DSL → AST
- AST → semantic model
- Zod to validate shape

### Stage 2 — Shared Registries
Builds:

```typescript
domains: { identity, ordering, payments }
contexts: { identity.auth, ordering.checkout }
teams:    { platform, checkoutTeam }
```

### Stage 3 — Parsing Modules
Each module is loaded independently but imports resolved before parsing.

**Universal model structure:**

```sruja
module {
  context
  owner
  containers
  components
  relationships
  metadata
}
```

### Stage 4 — Context Binding
For each module:

```typescript
module.context = contexts["identity.auth"]
```

### Stage 5 — Reference Resolution
Transforms:

```sruja
web -> auth.api
```

into validated relationships.

### Stage 6 — Graph Building
Simply collects container IDs & pairings.

## Validation Rules

The composer enforces:

### Reference Target Exists
```sruja
checkout.web -> auth.api
```
`auth.api` must exist in the global model.

### Module belongs to a valid context
```sruja
context: identity.auth
```
must exist.

### Context belongs to a valid domain

### Ownership must be valid team
```sruja
owner: team.platform
```

### No duplicate modules across imports

### No circular UIDs

## Output

Produces:

```
.architecture/global-model.json
```

This is the **single source of truth** for the entire architecture.

## Integration

Used by:
- **CLI** - `sruja compile` command
- **LSP** - Global model for diagnostics
- **Editor** - Full architecture context
- **Validation Engine** - Cross-module validation
- **Visual Editor** - Complete architecture graph

## Implementation Status

✅ Architecture designed  
✅ Processing stages defined  
✅ Validation rules specified  
📋 Implementation in progress

---

*The Model Composition Engine is the foundation that enables multi-module architecture modeling.*
