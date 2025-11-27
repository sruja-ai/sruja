# Validation Engine v1 — Full Implementation Plan

Complete implementation plan for the Architecture Validation Engine v1, broken into phases, tasks, and code modules.

**Status:** Production-grade, aligned with Bun + TypeScript monorepo, ready for OSS contributors

---

## 🎯 Overview

This plan provides:
- ✔ Folder structure
- ✔ Modules
- ✔ APIs
- ✔ Parsing pipeline
- ✔ Rule execution framework
- ✔ Test strategy
- ✔ Integration with CLI + LSP
- ✔ Plugin system
- ✔ Timeline & priorities

---

## 📦 Monorepo Structure

```
/packages
   /dsl-parser        → Ohm grammar + AST builder
   /model             → Zod schema + model normalizer
   /validation        → Validation engine (this plan)
   /cli               → sruja validate command
   /lsp               → editor diagnostics
   /web               → Next.js app
```

Validation engine is `/packages/validation`.

---

## 📋 Validation Engine Requirements

### Must Have:
- ✅ Validate syntax (from parser)
- ✅ Validate semantics (IDs, references)
- ✅ Validate layering rules (strict)
- ✅ Detect cycles
- ✅ Provide LSP-compatible diagnostics
- ✅ Plugin-ready
- ✅ Pure, deterministic functions
- ✅ No DB or I/O

### MVP Rule Set:
1. Duplicate ID
2. Unknown reference
3. Invalid types
4. Strict layering rules
5. Circular dependency detection

---

## 🧠 Boundary-Aware Model Additions

Add structured entities to the model schema used by validation:

```ts
const Team = z.object({
  id: z.string(),
  lead: z.string().optional(),
  slack: z.string().optional(),
  metadata: z.record(z.any()).optional()
});

const Domain = z.object({
  id: z.string(),
  description: z.string().optional(),
  contexts: z.array(z.string())
});

const BoundedContext = z.object({
  id: z.string(),
  domain: z.string(),
  owner: z.string().optional(),
  responsibilities: z.array(z.string()).optional(),
  modules: z.array(z.string()).optional()
});

const Module = z.object({
  id: z.string(),
  context: z.string().optional(),
  owner: z.string().optional()
});
```

Qualified identifiers: `domain.context.module.container.component`.

Context maps store relationships: `customer-supplier`, `conformist`, `shared-kernel`, `open-host-service`, `anticorruption-layer`.

---

## 🛡️ Boundary Rule Set (Implementation)

Add a new rules group under `rules/boundaries/`:

```
rules/
  boundaries/
    forbidden-internal-access.ts
    require-api-access.ts
    team-topologies.ts
    domain-boundaries.ts
    shared-kernel.ts
    anti-corruption-layer.ts
  index.ts
```

### Registry

```typescript
// rules/index.ts
import { boundaryRules } from './boundaries';

export const builtinRules: ValidationRule[] = [
  ...semanticRules,
  ...layerRules,
  ...bestPracticeRules,
  ...boundaryRules,
];
```

### Rule sketches

```typescript
// rules/boundaries/forbidden-internal-access.ts
export const forbiddenInternalAccessRule: ValidationRule = {
  id: 'boundaries/forbidden-internal-access',
  description: 'Disallow direct access to private internals across contexts',
  severity: 'error',
  apply(ctx) {
    const issues = [];
    for (const edge of ctx.model.edges) {
      const from = ctx.findNodeById(edge.from);
      const to = ctx.findNodeById(edge.to);
      if (!from || !to) continue;
      if (from.context && to.context && from.context !== to.context) {
        if (to.visibility === 'private') {
          issues.push({
            ruleId: this.id,
            message: `${from.id} → ${to.id} crosses context boundary to private target`,
            severity: 'error',
            location: ctx.getLocation(edge.id),
            metadata: { from: from.id, to: to.id, fromContext: from.context, toContext: to.context }
          });
        }
      }
    }
    return issues;
  }
};

// rules/boundaries/anti-corruption-layer.ts
export const antiCorruptionLayerRule: ValidationRule = {
  id: 'contexts/require-acl-indirection',
  description: 'Require adapter layer when relation is marked anti-corruption-layer',
  severity: 'error',
  apply(ctx) {
    const issues = [];
    for (const rel of ctx.model.contextMap ?? []) {
      if (rel.type !== 'anticorruption-layer') continue;
      const adapters = ctx.model.nodes.filter(n => n.tags?.includes('acalayer'));
      const hasIndirect = adapters.some(a =>
        ctx.findEdges(rel.from, a.id).length && ctx.findEdges(a.id, rel.to).length
      );
      if (!hasIndirect) {
        issues.push({
          ruleId: this.id,
          message: `ACL required between ${rel.from} and ${rel.to}`,
          severity: 'error'
        });
      }
    }
    return issues;
  }
};
```

---

## ⚙️ Enforcement Workflow (Composition + Validation)

1. Load domains, contexts, teams, modules
2. Bind each module to exactly one context
3. Index qualified identifiers for lookup
4. Build context map graph
5. Precompute visibility and ownership on nodes
6. Run boundary rules with model and context map
7. Emit diagnostics with rule IDs and locations

## Reference Resolution Integration

### Resolver Contract

```
export function resolveRelationships(
  moduleModels: Record<string, Module>,
  registry: ResolutionRegistry,
  contextMap: ContextMap,
  rulePlugins: RulePlugin[]
): Relationship[]
```

### Identifier Normalization

```
function qualifyId(raw: string, currentModuleId: string): string {
  return raw.includes('.') ? raw : `${currentModuleId}.${raw}`;
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

### Diagnostics & LSP

Resolution failures throw `DiagnosticError { message, location }`. The LSP diagnostics layer converts these to VSCode diagnostics using source ranges.

### Security

- Reject references outside imported namespaces
- Disallow path traversal in imports
- Validate GitHub imports target `.sruja` files only

## 🚀 Implementation Phases

---

## PHASE 1 — Core Engine Skeleton

### 🎯 Goal: Create the scaffolding + core entrypoints

### Tasks

#### 1. Create Package Structure

```
packages/validation/
  index.ts                    → Public API export
  engine/
    validate.ts              → Main validateArchitecture() function
    context.ts               → ValidationContext implementation
    runner.ts                → Rule execution runner
    finalize.ts              → Result finalization
  rules/
    index.ts                 → Rule registry
    semantic/
      duplicate-id.ts
      unknown-reference.ts
      invalid-type.ts
      missing-required-prop.ts
      circular-dependency.ts
    layers/
      no-components-at-root.ts
      containers-must-belong-to-system.ts
      components-must-belong-to-container.ts
      external-boundary.ts
    bestpractice/
      no-direct-db-access.ts
      naming-conventions.ts
      avoid-fat-containers.ts
  types/
    issues.ts                → ValidationIssue, SourceLocation
    config.ts                → ValidationConfig
    plugins.ts                → ValidationPlugin, ValidationRule
    input.ts                 → ValidationInput, ValidationResult
  utils/
    graph.ts                 → Graph traversal (cycle detection)
    ids.ts                   → ID utilities
    locations.ts             → Source location mapping
  __tests__/
    rules/
      semantic.test.ts
      layers.test.ts
      bestpractice.test.ts
    engine.test.ts
    integration.test.ts
```

#### 2. Add Base Interfaces

**File:** `types/input.ts`

Implement from the [Validation Engine API spec](./validation-engine.md):
- `ValidationInput`
- `ValidationResult`
- `ValidationSummary`

**File:** `types/issues.ts`
- `ValidationIssue`
- `SourceLocation`
- `Position`

**File:** `types/config.ts`
- `ValidationConfig`

**File:** `types/plugins.ts`
- `ValidationPlugin`
- `ValidationRule`
- `ValidationContext`

**Time Estimate:** ~1 hour

#### 3. Core Function: `validateArchitecture()`

**File:** `engine/validate.ts`

```typescript
import { ValidationInput, ValidationResult } from '../types/input';
import { createContext } from './context';
import { loadRules } from './runner';
import { finalizeValidation } from './finalize';

export function validateArchitecture(
  input: ValidationInput
): ValidationResult {
  const ctx = createContext(input);
  const issues: ValidationIssue[] = [];

  // 1. Collect built-in rules
  const rules = loadRules(input.config);

  // 2. Register plugin rules
  if (input.config?.plugins) {
    for (const plugin of input.config.plugins) {
      rules.push(...plugin.rules);
    }
  }

  // 3. Run rules
  for (const rule of rules) {
    try {
      const ruleIssues = rule.apply(ctx);
      // Handle both sync and async rules
      if (ruleIssues instanceof Promise) {
        // In real implementation, use Promise.all for parallel execution
        issues.push(...await ruleIssues);
      } else {
        issues.push(...ruleIssues);
      }
    } catch (error) {
      // Never crash on rule errors
      issues.push({
        ruleId: rule.id,
        message: `Rule execution failed: ${error.message}`,
        severity: 'error',
        metadata: { error: error.toString() }
      });
    }
  }

  // 4. Group into warnings/errors and compute summary
  return finalizeValidation(issues, input.config);
}
```

---

## PHASE 2 — Rule Execution Framework

### 🎯 Goal: Implement the infrastructure that runs validation rules

### Tasks

#### 1. Rule Loader

**File:** `engine/runner.ts`

```typescript
import { ValidationConfig } from '../types/config';
import { ValidationRule } from '../types/plugins';
import { builtinRules } from '../rules';

export function loadRules(config?: ValidationConfig): ValidationRule[] {
  let rules = [...builtinRules];

  // Filter enabled rules
  if (config?.enabledRules) {
    rules = rules.filter(rule => config.enabledRules!.includes(rule.id));
  }

  // Filter disabled rules
  if (config?.disabledRules) {
    rules = rules.filter(rule => !config.disabledRules!.includes(rule.id));
  }

  return rules;
}
```

#### 2. Rule Registry

**File:** `rules/index.ts`

```typescript
import { ValidationRule } from '../types/plugins';
import { semanticRules } from './semantic';
import { layerRules } from './layers';
import { bestPracticeRules } from './bestpractice';

export const builtinRules: ValidationRule[] = [
  ...semanticRules,
  ...layerRules,
  ...bestPracticeRules,
];

// Export individual rule groups for testing
export { semanticRules } from './semantic';
export { layerRules } from './layers';
export { bestPracticeRules } from './bestpractice';
```

#### 3. Create `ValidationContext`

**File:** `engine/context.ts`

```typescript
import { ValidationInput, ValidationConfig } from '../types';
import { ValidationContext, ValidationIssue } from '../types/plugins';
import { ArchitectureModel, AST } from '@arch/model';

export function createContext(input: ValidationInput): ValidationContext {
  const issues: ValidationIssue[] = [];

  return {
    dsl: input.dsl,
    ast: input.ast,
    model: input.model,
    project: input.project,
    config: input.config || {},

    // Helper utilities
    findNodeById(id: string) {
      return input.model.nodes.find(n => n.id === id);
    },

    findEdges(from?: string, to?: string) {
      return input.model.edges.filter(e => {
        if (from && e.from !== from) return false;
        if (to && e.to !== to) return false;
        return true;
      });
    },

    getLayer(nodeId: string) {
      const node = this.findNodeById(nodeId);
      if (!node) return null;
      // Determine layer based on node type and hierarchy
      // Implementation depends on model structure
      return node.type; // Simplified
    },

    report(issue: ValidationIssue) {
      issues.push(issue);
    },

    // Get all reported issues (for internal use)
    getIssues(): ValidationIssue[] {
      return [...issues];
    }
  };
}
```

#### 4. Finalizer

**File:** `engine/finalize.ts`

```typescript
import { ValidationIssue, ValidationConfig, ValidationResult, ValidationSummary } from '../types';

export function finalizeValidation(
  issues: ValidationIssue[],
  config?: ValidationConfig
): ValidationResult {
  // Apply warningAsError if configured
  const processedIssues = config?.warningAsError
    ? issues.map(issue => ({
        ...issue,
        severity: issue.severity === 'warning' ? 'error' : issue.severity
      }))
    : issues;

  // Split into errors and warnings
  const errors = processedIssues.filter(i => i.severity === 'error');
  const warnings = processedIssues.filter(i => i.severity === 'warning');

  // Compute summary
  const ruleBreakdown: Record<string, number> = {};
  for (const issue of processedIssues) {
    ruleBreakdown[issue.ruleId] = (ruleBreakdown[issue.ruleId] || 0) + 1;
  }

  const summary: ValidationSummary = {
    errorCount: errors.length,
    warningCount: warnings.length,
    rulesExecuted: new Set(processedIssues.map(i => i.ruleId)).size,
    ruleBreakdown
  };

  return {
    ok: errors.length === 0,
    errors,
    warnings,
    summary
  };
}
```

---

## PHASE 3 — Implement Built-in Rules

This is where you deliver real value.

### 3.1 Semantic Rules

#### 3.1.1 Duplicate IDs

**File:** `rules/semantic/duplicate-id.ts`

```typescript
import { ValidationRule, ValidationContext } from '../../types/plugins';

export const duplicateIdRule: ValidationRule = {
  id: 'semantic/duplicate-id',
  description: 'Detect duplicate node IDs',
  severity: 'error',
  apply(ctx: ValidationContext) {
    const issues = [];
    const seenIds = new Map<string, number>();

    for (const node of ctx.model.nodes) {
      if (seenIds.has(node.id)) {
        issues.push({
          ruleId: this.id,
          message: `Duplicate node ID: '${node.id}'`,
          severity: 'error',
          location: ctx.getLocation(node.id), // Helper to get source location
          metadata: { nodeId: node.id, firstOccurrence: seenIds.get(node.id) }
        });
      } else {
        seenIds.set(node.id, node.line || 0);
      }
    }

    return issues;
  }
};
```

#### 3.1.2 Unknown References

**File:** `rules/semantic/unknown-reference.ts`

```typescript
export const unknownReferenceRule: ValidationRule = {
  id: 'semantic/unknown-reference',
  description: 'Detect references to undefined nodes',
  severity: 'error',
  apply(ctx: ValidationContext) {
    const issues = [];
    const nodeIds = new Set(ctx.model.nodes.map(n => n.id));

    for (const edge of ctx.model.edges) {
      if (!nodeIds.has(edge.from)) {
        issues.push({
          ruleId: this.id,
          message: `Reference to undefined node: '${edge.from}'`,
          severity: 'error',
          location: ctx.getLocation(edge.from),
          metadata: { referencedId: edge.from, edgeId: edge.id }
        });
      }
      if (!nodeIds.has(edge.to)) {
        issues.push({
          ruleId: this.id,
          message: `Reference to undefined node: '${edge.to}'`,
          severity: 'error',
          location: ctx.getLocation(edge.to),
          metadata: { referencedId: edge.to, edgeId: edge.id }
        });
      }
    }

    return issues;
  }
};
```

#### 3.1.3 Invalid Types

**File:** `rules/semantic/invalid-type.ts`

```typescript
export const invalidTypeRule: ValidationRule = {
  id: 'semantic/invalid-type',
  description: 'Detect invalid node type usage',
  severity: 'error',
  apply(ctx: ValidationContext) {
    const issues = [];
    // Check parent-child relationships
    // E.g., component cannot be direct child of system
    // Implementation depends on model structure
    return issues;
  }
};
```

#### 3.1.4 Missing Required Properties

**File:** `rules/semantic/missing-required-prop.ts`

```typescript
export const missingRequiredPropRule: ValidationRule = {
  id: 'semantic/missing-required-prop',
  description: 'Detect missing required properties',
  severity: 'error',
  apply(ctx: ValidationContext) {
    const issues = [];
    // Check required fields based on node type
    // E.g., system must have name
    return issues;
  }
};
```

#### 3.1.5 Circular Dependencies

**File:** `rules/semantic/circular-dependency.ts`

```typescript
import { detectCycles } from '../../utils/graph';

export const circularDependencyRule: ValidationRule = {
  id: 'semantic/circular-dependency',
  description: 'Detect circular dependencies',
  severity: 'error',
  apply(ctx: ValidationContext) {
    const issues = [];
    const cycles = detectCycles(ctx.model);

    for (const cycle of cycles) {
      issues.push({
        ruleId: this.id,
        message: `Circular dependency detected: ${cycle.join(' → ')}`,
        severity: 'error',
        metadata: { cycle }
      });
    }

    return issues;
  }
};
```

**File:** `utils/graph.ts`

```typescript
export function detectCycles(model: ArchitectureModel): string[][] {
  const cycles: string[][] = [];
  const visited = new Set<string>();
  const recStack = new Set<string>();

  function dfs(nodeId: string, path: string[]): void {
    if (recStack.has(nodeId)) {
      // Found cycle
      const cycleStart = path.indexOf(nodeId);
      cycles.push([...path.slice(cycleStart), nodeId]);
      return;
    }

    if (visited.has(nodeId)) return;

    visited.add(nodeId);
    recStack.add(nodeId);

    const edges = model.edges.filter(e => e.from === nodeId);
    for (const edge of edges) {
      dfs(edge.to, [...path, nodeId]);
    }

    recStack.delete(nodeId);
  }

  for (const node of model.nodes) {
    if (!visited.has(node.id)) {
      dfs(node.id, []);
    }
  }

  return cycles;
}
```

**File:** `rules/semantic/index.ts`

```typescript
import { ValidationRule } from '../../types/plugins';
import { duplicateIdRule } from './duplicate-id';
import { unknownReferenceRule } from './unknown-reference';
import { invalidTypeRule } from './invalid-type';
import { missingRequiredPropRule } from './missing-required-prop';
import { circularDependencyRule } from './circular-dependency';

export const semanticRules: ValidationRule[] = [
  duplicateIdRule,
  unknownReferenceRule,
  invalidTypeRule,
  missingRequiredPropRule,
  circularDependencyRule
];
```

### 3.2 Layer Rules

Enabled via config: `strictLayers: true`

#### 3.2.1 No Components at System Root

**File:** `rules/layers/no-components-at-root.ts`

```typescript
export const noComponentsAtRootRule: ValidationRule = {
  id: 'layers/no-components-at-root',
  description: 'Components must not be defined at system root',
  severity: 'error',
  apply(ctx: ValidationContext) {
    if (!ctx.config.strictLayers) return [];

    const issues = [];
    // Check if any components are direct children of system
    // Implementation depends on model hierarchy
    return issues;
  }
};
```

#### 3.2.2 Containers Must Belong to System

**File:** `rules/layers/containers-must-belong-to-system.ts`

```typescript
export const containersMustBelongToSystemRule: ValidationRule = {
  id: 'layers/containers-must-belong-to-system',
  description: 'Containers must belong to a system',
  severity: 'error',
  apply(ctx: ValidationContext) {
    if (!ctx.config.strictLayers) return [];
    // Implementation
    return [];
  }
};
```

#### 3.2.3 Components Must Belong to Container

**File:** `rules/layers/components-must-belong-to-container.ts`

```typescript
export const componentsMustBelongToContainerRule: ValidationRule = {
  id: 'layers/components-must-belong-to-container',
  description: 'Components must belong to a container',
  severity: 'error',
  apply(ctx: ValidationContext) {
    if (!ctx.config.strictLayers) return [];
    // Implementation
    return [];
  }
};
```

#### 3.2.4 External Boundary

**File:** `rules/layers/external-boundary.ts`

```typescript
export const externalBoundaryRule: ValidationRule = {
  id: 'layers/external-boundary',
  description: 'External systems cannot have inbound internal links',
  severity: 'error',
  apply(ctx: ValidationContext) {
    if (!ctx.config.strictLayers) return [];
    // Implementation
    return [];
  }
};
```

**File:** `rules/layers/index.ts`

```typescript
import { ValidationRule } from '../../types/plugins';
import { noComponentsAtRootRule } from './no-components-at-root';
import { containersMustBelongToSystemRule } from './containers-must-belong-to-system';
import { componentsMustBelongToContainerRule } from './components-must-belong-to-container';
import { externalBoundaryRule } from './external-boundary';

export const layerRules: ValidationRule[] = [
  noComponentsAtRootRule,
  containersMustBelongToSystemRule,
  componentsMustBelongToContainerRule,
  externalBoundaryRule
];
```

### 3.3 Best Practices (MVP minimal)

Only warnings in v1.

#### 3.3.1 No Direct DB Access

**File:** `rules/bestpractice/no-direct-db-access.ts`

```typescript
export const noDirectDbAccessRule: ValidationRule = {
  id: 'bestpractice/no-direct-db-access',
  description: 'Components should not directly access databases',
  severity: 'warning',
  apply(ctx: ValidationContext) {
    const issues = [];
    // Check for direct edges from components to database nodes
    return issues;
  }
};
```

#### 3.3.2 Naming Conventions

**File:** `rules/bestpractice/naming-conventions.ts`

```typescript
export const namingConventionsRule: ValidationRule = {
  id: 'bestpractice/naming-conventions',
  description: 'Enforce naming conventions',
  severity: 'warning',
  apply(ctx: ValidationContext) {
    const issues = [];
    // Check node names against conventions (e.g., camelCase, PascalCase)
    return issues;
  }
};
```

#### 3.3.3 Avoid Fat Containers

**File:** `rules/bestpractice/avoid-fat-containers.ts`

```typescript
export const avoidFatContainersRule: ValidationRule = {
  id: 'bestpractice/avoid-fat-containers',
  description: 'Containers should not have too many components',
  severity: 'warning',
  apply(ctx: ValidationContext) {
    const issues = [];
    const MAX_COMPONENTS = 10; // Configurable
    // Check container component count
    return issues;
  }
};
```

**File:** `rules/bestpractice/index.ts`

```typescript
import { ValidationRule } from '../../types/plugins';
import { noDirectDbAccessRule } from './no-direct-db-access';
import { namingConventionsRule } from './naming-conventions';
import { avoidFatContainersRule } from './avoid-fat-containers';

export const bestPracticeRules: ValidationRule[] = [
  noDirectDbAccessRule,
  namingConventionsRule,
  avoidFatContainersRule
];
```

---

## PHASE 4 — Plugin System

### 🎯 Goal: External validators

### Tasks

#### 1. Define Plugin Loading API

**File:** `engine/plugins.ts`

```typescript
import { ValidationConfig, ValidationPlugin, ValidationRule } from '../types';
import { z } from 'zod';

const PluginSchema = z.object({
  name: z.string(),
  version: z.string(),
  rules: z.array(z.any()) // ValidationRule schema
});

export async function loadPlugins(
  pluginPaths: string[]
): Promise<ValidationRule[]> {
  const rules: ValidationRule[] = [];

  for (const path of pluginPaths) {
    try {
      // Load plugin module (supports npm packages, relative paths, absolute paths)
      const module = await import(path);
      const plugin = module.default || module;

      // Validate plugin structure
      const validated = PluginSchema.parse(plugin);
      rules.push(...validated.rules);
    } catch (error) {
      // Never crash on invalid plugins
      console.warn(`Failed to load plugin from ${path}: ${error.message}`);
    }
  }

  return rules;
}
```

#### 2. Validate Plugin Structure Using Zod

Already included in step 1.

#### 3. Add Error Handling for Plugin Exceptions

Already included in `validateArchitecture()` - rules are wrapped in try/catch.

#### 4. Plugin Example

See [Validation Plugin Example](./validation-plugin-example.md) for a complete, production-ready plugin implementation.

---

## PHASE 5 — Source Mapping & Locations (for LSP + editor)

### Tasks

#### 1. Ohm CST → Model Node → Source Range

**File:** `utils/locations.ts`

```typescript
import { AST, CSTNode } from '@arch/dsl-parser';
import { SourceLocation, Position } from '../types/issues';

export function locationOf(astNode: CSTNode): SourceLocation | undefined {
  if (!astNode.source) return undefined;

  return {
    file: astNode.source.source.name || 'architecture.sruja',
    start: {
      line: astNode.source.startLine,
      column: astNode.source.startCol
    },
    end: {
      line: astNode.source.endLine,
      column: astNode.source.endCol
    }
  };
}

export function getLocationForNode(
  nodeId: string,
  ast: AST
): SourceLocation | undefined {
  // Find AST node by ID
  const astNode = findNodeById(ast, nodeId);
  if (!astNode) return undefined;
  return locationOf(astNode);
}
```

#### 2. Add Location Helper to Context

**File:** `engine/context.ts` (update)

```typescript
getLocation(nodeId: string): SourceLocation | undefined {
  return getLocationForNode(nodeId, this.ast);
}
```

---

## PHASE 6 — Test Suite

### Testing Strategy

Use **Vitest**.

### 6.1 Unit Tests (Rules)

**File:** `__tests__/rules/semantic.test.ts`

```typescript
import { describe, it, expect } from 'vitest';
import { duplicateIdRule } from '../../rules/semantic/duplicate-id';
import { createMockContext } from '../helpers';

describe('Semantic Rules', () => {
  it('detects duplicate IDs', () => {
    const ctx = createMockContext({
      model: {
        nodes: [
          { id: 'api', type: 'service' },
          { id: 'api', type: 'service' } // Duplicate!
        ]
      }
    });

    const issues = duplicateIdRule.apply(ctx);
    expect(issues).toHaveLength(1);
    expect(issues[0].ruleId).toBe('semantic/duplicate-id');
  });
});
```

**File:** `__tests__/rules/layers.test.ts`

```typescript
// Test layer rules
```

**File:** `__tests__/rules/bestpractice.test.ts`

```typescript
// Test best practice rules
```

**File:** `__tests__/rules/circular-deps.test.ts`

```typescript
import { circularDependencyRule } from '../../rules/semantic/circular-dependency';

describe('Circular Dependency Detection', () => {
  it('detects simple cycle', () => {
    const ctx = createMockContext({
      model: {
        nodes: [{ id: 'a' }, { id: 'b' }],
        edges: [
          { from: 'a', to: 'b' },
          { from: 'b', to: 'a' } // Cycle!
        ]
      }
    });

    const issues = circularDependencyRule.apply(ctx);
    expect(issues.length).toBeGreaterThan(0);
  });
});
```

### 6.2 Integration Tests

**File:** `__tests__/integration.test.ts`

```typescript
import { validateArchitecture } from '../engine/validate';
import { parse } from '@arch/dsl-parser';
import { transform } from '@arch/model';

describe('Full Validation Pipeline', () => {
  it('validates complete architecture', () => {
    const dsl = `
      system "Test" {
        container api: Service "API"
      }
    `;

    const ast = parse(dsl);
    const model = transform(ast);

    const result = validateArchitecture({
      dsl,
      ast,
      model,
      project: { root: '.' }
    });

    expect(result.ok).toBe(true);
  });
});
```

### 6.3 Golden File Tests

**File:** `__tests__/golden.test.ts`

```typescript
import { readdirSync, readFileSync } from 'fs';
import { join } from 'path';

describe('Golden File Tests', () => {
  const testCases = readdirSync(join(__dirname, 'fixtures'));

  for (const testCase of testCases) {
    it(`validates ${testCase}`, () => {
      const input = readFileSync(join(__dirname, 'fixtures', testCase, 'input.sruja'), 'utf8');
      const expected = JSON.parse(
        readFileSync(join(__dirname, 'fixtures', testCase, 'expected.json'), 'utf8')
      );

      const result = validateArchitecture(/* ... */);
      expect(result).toMatchObject(expected);
    });
  }
});
```

---

## PHASE 7 — Integration (CLI + LSP)

### 🎯 Goal: Hook validation engine into real clients

### 7.1 CLI: `sruja validate`

**File:** `packages/cli/commands/validate.ts`

```typescript
import { validateArchitecture } from '@arch/validation';
import { parse } from '@arch/dsl-parser';
import { transform } from '@arch/model';
import { loadProject } from '../utils/project';

export async function validateCommand(options: ValidateOptions) {
  const project = loadProject(options.projectPath);
  const dsl = await project.loadFile('architecture.sruja');
  
  const ast = parse(dsl);
  const model = transform(ast);

  const result = validateArchitecture({
    dsl,
    ast,
    model,
    project,
    config: project.config.validation
  });

  if (options.json) {
    console.log(JSON.stringify(result, null, 2));
  } else {
    // Human-readable output
    printValidationResults(result);
  }

  process.exit(result.ok ? 0 : 2);
}
```

### 7.2 LSP Diagnostics

**File:** `apps/backend/src/lsp/features/diagnostics.ts` (update)

```typescript
import { validateArchitecture } from '@arch/validation';
import { Diagnostic, DiagnosticSeverity } from 'vscode-languageserver';

export function computeDiagnostics(
  document: TextDocument,
  model: ArchitectureModel,
  ast: AST
): Diagnostic[] {
  const result = validateArchitecture({
    dsl: document.getText(),
    ast,
    model,
    project: getProject(document.uri),
    config: getValidationConfig(document.uri)
  });

  // Map ValidationIssue to LSP Diagnostic
  const diagnostics: Diagnostic[] = [];

  for (const issue of [...result.errors, ...result.warnings]) {
    if (!issue.location) continue;

    diagnostics.push({
      range: {
        start: {
          line: issue.location.start.line - 1, // LSP is 0-based
          character: issue.location.start.column - 1
        },
        end: {
          line: issue.location.end.line - 1,
          character: issue.location.end.column - 1
        }
      },
      message: issue.message,
      severity: issue.severity === 'error' 
        ? DiagnosticSeverity.Error 
        : DiagnosticSeverity.Warning,
      source: issue.ruleId
    });
  }

  return diagnostics;
}
```

---

## 📅 Detailed Timeline & Priorities

### Week 1: Core Foundation

**🚀 CORE FOUNDATION**

- [ ] Create `/packages/validation`
- [ ] Types & interfaces
- [ ] Context + runner + finalizer
- [ ] Rule loader

**Result:** Validation engine compiles but does nothing.

---

### Week 2: Core Semantic Rules

**🔍 CORE SEMANTIC RULES**

- [ ] Duplicate ID
- [ ] Unknown reference
- [ ] Invalid parent type
- [ ] Missing required props
- [ ] Circular dependencies

**Result:** Can validate basic DSL.

---

### Week 3: Layer & Best Practices

**🏛 LAYER & BEST PRACTICES**

- [ ] Strict layer rules
- [ ] 3 basic best-practice checks

---

### Week 4: Plugins + Source Mapping

**🔌 PLUGINS + SOURCE MAPPING**

- [ ] Plugin loader
- [ ] Zod plugin validation
- [ ] AST → source range mapping
- [ ] LSP compatibility

---

### Week 5: Complete Test Suite

**🧪 COMPLETE TEST SUITE**

- [ ] 20+ rule tests
- [ ] Full pipeline tests
- [ ] Golden files

---

### Week 6: Integration

**🧩 INTEGRATION**

- [ ] Hook into CLI
- [ ] Hook into LSP
- [ ] Release v1.0 of validation engine

---

## 🎉 Final Result

At Week 6 you have:

- ✅ A full validation engine
- ✅ Strong, predictable API
- ✅ Built-in rules
- ✅ Plugin system
- ✅ CI-friendly integration
- ✅ LSP diagnostics
- ✅ Real errors highlighted in editor
- ✅ 100% deterministic pure-core

This becomes a foundational pillar for your architecture modeling system.

---

[← Back to Documentation Index](../README.md)
