# AST Transformer Engine

**Status**: Core Engine  
**Pillars**: Core (Transformation)

[← Back to Engines](../README.md)

## Overview

The AST Transformer Engine transforms Abstract Syntax Tree (AST) nodes into typed Intermediate Representation (IR), converting parsed DSL into semantic models.

**This bridges the gap between syntax (AST) and semantics (IR).**

## Purpose

The AST Transformer Engine:

- ✅ Transforms AST → IR
- ✅ Maps syntax to semantics
- ✅ Validates semantic rules
- ✅ Builds typed models
- ✅ Resolves identifiers
- ✅ Validates structure
- ✅ Produces IR for composition

## Architecture

```
AST (from Parser)
   ↓
AST Transformer
   ↓
IR (Intermediate Representation)
   ↓
Zod Validation
   ↓
Typed IR Model
```

## Transformation Process

### Step 1 — AST Traversal
- Walk AST tree
- Visit each node
- Extract semantic information

### Step 2 — Semantic Mapping
- Map AST nodes to IR constructs
- Build domain models
- Build context models
- Build module models
- Build relationship models

### Step 3 — Validation
- Validate IR structure
- Check semantic rules
- Report validation errors

## IR Structure

The transformer produces IR with:

```ts
interface IRModel {
  domains: Domain[];
  contexts: BoundedContext[];
  teams: Team[];
  modules: Module[];
  relations: Relation[];
  journeys?: UserJourney[];
  adrs?: ADR[];
  imports?: Import[];
}
```

## Transformation Mappings

### Domain Mapping
```ts
function mapDomain(ast: ASTNode): Domain {
  return {
    id: ast.identifier,
    name: ast.label,
    location: ast.location
  };
}
```

### Context Mapping
```ts
function mapContext(ast: ASTNode): BoundedContext {
  return {
    id: ast.identifier,
    domain: ast.domainId,
    name: ast.label,
    location: ast.location
  };
}
```

### Module Mapping
```ts
function mapModule(ast: ASTNode): Module {
  return {
    id: ast.identifier,
    context: ast.contextId,
    owner: ast.teamId,
    containers: mapContainers(ast.containers),
    location: ast.location
  };
}
```

### Relation Mapping
```ts
function mapRelation(ast: ASTNode): Relation {
  return {
    from: ast.fromId,
    to: ast.toId,
    description: ast.label,
    location: ast.location
  };
}
```

## Validation

Uses **Zod** for validation:

```ts
const IRModelSchema = z.object({
  domains: z.array(DomainSchema),
  contexts: z.array(ContextSchema),
  modules: z.array(ModuleSchema),
  relations: z.array(RelationSchema)
});

function validateIR(ir: IRModel): ValidationResult {
  return IRModelSchema.safeParse(ir);
}
```

## Error Handling

The transformer provides:

- Semantic error detection
- Type validation
- Structure validation
- Detailed error messages

Example error:

```
Semantic error at line 10:
  Module 'checkout' must belong to a context
  module checkout { ... }
         ^
```

## Integration Points

### DSL Parser
- Consumes AST
- Transforms to IR

### Model Composition Engine
- Consumes IR
- Composes global model

### Validation Engine
- Validates IR structure
- Checks semantic rules

## Transformation Rules

### Identifier Resolution
- Resolve unqualified names
- Build qualified identifiers
- Validate identifier existence

### Type Validation
- Validate component types
- Validate relation types
- Check type compatibility

### Structure Validation
- Validate module structure
- Validate context structure
- Validate domain structure

## MCP API

```
transformer.transform(ast)
transformer.validate(ir)
transformer.toIR(ast)
```

## Strategic Value

The AST Transformer Engine provides:

- ✅ AST to IR conversion
- ✅ Semantic validation
- ✅ Type safety
- ✅ Structure validation
- ✅ Foundation for composition

**This is critical for semantic model building.**

## Implementation Status

✅ Architecture designed  
✅ Transformation mappings specified  
✅ Validation defined  
📋 Implementation in progress

---

*The AST Transformer Engine transforms AST into typed IR models.*

