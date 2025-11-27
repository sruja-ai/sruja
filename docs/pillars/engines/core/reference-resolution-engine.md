# Reference Resolution Engine

**Status**: Core Engine  
**Pillars**: Core (Cross-Module Linking)

[← Back to Engines](../README.md)

## Overview

The Reference Resolution Engine is a critical subsystem that handles cross-module linking, cross-context linking, namespace-based identifier resolution, relationship resolution, and boundary validation.

**This is one of the most critical subsystems of the compiler.**

## Purpose

The Reference Resolution Engine:

- ✅ Resolves cross-module references
- ✅ Resolves cross-context references
- ✅ Normalizes identifiers and qualifies local names
- ✅ Validates domain/context/team boundaries
- ✅ Produces global relationship entries
- ✅ Emits precise LSP diagnostics
- ✅ Integrates with composition engine

## What the Reference Resolver Must Do

Given a relationship in DSL:

```
checkout.web -> auth.api: "login"
```

It must:

### Parse source & target identifiers
→ (`checkout.web`, `auth.api`)

### Validate existence
- Does `checkout.web` exist?
- Does `auth.api` exist?

### Resolve across imports
- checkout-web imports auth-service
- IDs must merge correctly

### Validate boundaries (optional)
- Are checkout → auth allowed via context map?
- Do team/topology rules allow this?

### Generate global relationship entry
And add to the global model.

### Produce IDE/LSP diagnostics
- Unknown ID
- Cross-context violation
- Unknown import
- Wrong namespace
- Invalid reference path

## Core Data Types

### ContainerRef

```ts
interface ContainerRef {
  module: string;
  context: string;
  fullyQualifiedId: string; // e.g. "auth.api"
  type: "Service" | "Database" | "Frontend" | string;
  location: SourceLocation; // for LSP
}
```

### Relationship

```ts
interface Relationship {
  from: string; // "checkout.web"
  to: string;   // "auth.api"
  description?: string;
  sourceModule: string;
  context: string;
  location: SourceLocation;
}
```

### ResolutionRegistry

```ts
interface ResolutionRegistry {
  containers: Record<string, ContainerRef>;
  modules: Record<string, Module>;
  contexts: Record<string, BoundedContext>;
  domains: Record<string, Domain>;
}
```

## Identifier Rules

Identifiers follow:

```
<module>.<container>
```

Where:

```
auth.api
payments.api
checkout.web
orders.queue
inventory.db
```

Internal modules use implicit module name:

```
web -> auth.api
```

`web` is assumed to be `checkout.web`  
(target belongs to imported namespace)

## Resolution Algorithm

Given a relationship AST node:

```
from = "checkout.web"
to = "auth.api"
```

### Step 1 — Normalize identifiers

If no module prefix:

```
web → checkout.web   (local module)
```

Else keep fully qualified id.

Implementation:

```ts
function qualifyId(raw: string, currentModuleId: string): string {
  return raw.includes(".") ? raw : `${currentModuleId}.${raw}`;
}
```

### Step 2 — Lookup in container registry

```ts
function lookupContainer(id: string, registry: ResolutionRegistry): ContainerRef | null {
  return registry.containers[id] ?? null;
}
```

Raise error if null:

```
❌ Unknown reference: auth.ap
```

### Step 3 — Validate cross-context rules

Using the context graph:

```
contextMap: identity.auth -> ordering.checkout: "customer-supplier"
```

Examples:

✔ checkout → auth is allowed  
✔ checkout → payments is allowed  
✘ payments → checkout (violates "conformist")  
✘ identity → payments (no declared relation)

We implement:

```ts
function validateContextBoundary(
  fromContext: string,
  toContext: string,
  contextMap: ContextMap,
  location: SourceLocation
) {
  // 1. Allow same context
  if (fromContext === toContext) return;

  // 2. Allow declared mapping
  if (contextMap.allows(fromContext, toContext)) return;

  // 3. Otherwise error
  throw new DiagnosticError({
    message: `Invalid cross-context dependency: ${fromContext} → ${toContext}`,
    location
  });
}
```

### Step 4 — Validate team boundaries

Team topology rules (plugins) enforce constraints:

Example rules:

- frontend teams cannot reference databases directly
- payments team cannot access ordering internals
- platform team must expose public APIs only

Framework:

```ts
function validateTeamRules(
  fromTeam: string,
  toTeam: string,
  rulePlugins: RulePlugin[],
  location: SourceLocation
) {
  for (const plugin of rulePlugins) {
    plugin.validateTeamBoundary(fromTeam, toTeam, location);
  }
}
```

### Step 5 — Create relationship object

```ts
function createRelationship(
  from: ContainerRef,
  to: ContainerRef,
  context: string,
  description: string,
  sourceModule: string,
  location: SourceLocation
): Relationship {

  return {
    from: from.fullyQualifiedId,
    to: to.fullyQualifiedId,
    description,
    context,
    sourceModule,
    location
  };
}
```

## Main Implementation

```ts
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

      // 1. normalize
      const fromId = qualifyId(rel.from, currentModuleId);
      const toId = qualifyId(rel.to, currentModuleId);

      // 2. lookup
      const from = registry.containers[fromId];
      const to = registry.containers[toId];

      if (!from) throw new DiagnosticError({
        message: `Unknown reference: '${rel.from}'`,
        location: rel.location
      });

      if (!to) throw new DiagnosticError({
        message: `Unknown target: '${rel.to}'`,
        location: rel.location
      });

      // 3. validate context boundaries
      validateContextBoundary(
        from.context,
        to.context,
        contextMap,
        rel.location
      );

      // 4. validate team boundaries
      validateTeamRules(
        from.team,
        to.team,
        rulePlugins,
        rel.location
      );

      // 5. build relationship model
      resolved.push(createRelationship(
        from,
        to,
        currentContextId,
        rel.description,
        currentModuleId,
        rel.location
      ));
    }
  }

  return resolved;
}
```

## Registry Population

Before resolution, the composer builds:

```
registry.containers = {
  "auth.api": {...}
  "auth.db": {...}
  "checkout.web": {...}
  "payments.api": {...}
}
```

Done during module parsing:

```ts
function registerContainers(module, registry) {
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

## Diagnostics & LSP Integration

Every validation error becomes:

```ts
throw new DiagnosticError({
  message: "...",
  location: { line, column, file }
});
```

The LSP layer catches and converts into VSCode diagnostics.

## Security Considerations

The resolver must:

- Prevent referencing outside imported namespaces
- Reject references that implicitly use unknown modules
- Reject traversal attempts like: `../../../secret.sruja`
- Validate GitHub imports only point to `.sruja` files

## MCP API

```
reference.resolve(from, to)
reference.validate(from, to)
reference.lookup(id)
reference.registry()
```

## Strategic Value

The Reference Resolution Engine provides:

- ✅ Cross-module linking
- ✅ Boundary validation
- ✅ LSP diagnostics
- ✅ Namespace resolution
- ✅ Relationship validation

**This is critical for multi-module architecture support.**

## Implementation Status

✅ Architecture designed  
✅ Resolution algorithm specified  
✅ Boundary validation defined  
📋 Implementation in progress

---

*The Reference Resolution Engine handles cross-module and cross-context reference resolution.*
