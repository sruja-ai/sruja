# LikeC4 Builder Pattern Comparison

## Overview

This document compares LikeC4's programmatic Builder API with Sruja's current implementation, based on the actual LikeC4 reference code in `.likec4-ref/`.

## LikeC4 Builder Pattern (TypeScript)

### Features
- **Programmatic API**: Create models without writing DSL
- **Fluent Interface**: Chainable methods for readability
- **Type-Safe**: TypeScript types ensure correctness at compile-time
- **Specification-Based**: Define element types upfront
- **Two Usage Styles**: Compositional (`forSpecification`) and chainable (`specification`)

### Example Usage (Style 1 - Compositional)
```typescript
const {
  model: { model, system, component, relTo },
  deployment: { env, vm },
  views: { view, $include },
  builder,
} = Builder.forSpecification({
  elements: {
    system: {},
    component: {},
  },
  deployments: ['env', 'vm'],
})

const b = builder
  .with(
    model(
      system('cloud').with(
        component('backend'),
        component('backend.api'),
        component('frontend').with(
          relTo('cloud.backend.api'),
        ),
      ),
    ),
  )
  .toLikeC4Model()
```

### Example Usage (Style 2 - Chainable)
```typescript
const b = Builder
  .specification({
    elements: ['system', 'component'],
    deployments: ['env', 'vm'],
  })
  .model(({ system, component, relTo }, _) =>
    _(
      system('cloud').with(
        component('backend').with(
          component('api'),
        ),
        component('frontend').with(
          relTo('cloud.backend.api'),
        )
      )
    )
  )
  .deployment(({ env, node, instanceOf }, _) =>
    _(
      env('prod').with(
        node('eu').with(
          instanceOf('cloud.ui'),
        ),
      ),
    )
  )
  .views(({ view, viewOf, deploymentView, $include }, _) =>
    _(
      view('index', 'Index').with(
        $include('cloud.*'),
      ),
      viewOf('cloud', 'cloud.ui').with(
        $include('* -> cloud.**'),
      ),
    )
  )
  .toLikeC4Model()
```

### Key Methods
- `Builder.forSpecification(spec)` - Creates builder with destructured helpers (compositional style)
- `Builder.specification(spec)` - Creates builder directly (chainable style)
- `builder.model(callback)` - Adds model elements via callback
- `builder.deployment(callback)` - Adds deployment nodes via callback
- `builder.views(callback)` - Adds views via callback
- `builder.build()` - Returns parsed model (no computation/layout)
- `builder.toLikeC4Model()` - Returns computed model with views
- `builder.clone()` - Creates a copy of the builder
- `builder.with(...ops)` - Applies operations to cloned builder
- `element.with(...ops)` - Adds nested elements/relations

### Implementation Details
- **Location**: `.likec4-ref/packages/core/src/builder/Builder.ts`
- **Internal State**: Uses Maps for elements, views, deployments
- **Validation**: Validates specification (tags, element kinds)
- **Type System**: Complex TypeScript types for type safety
- **Helpers**: Dynamically generates helpers from specification

## Sruja's Current Implementation

### What Sruja Has

#### 1. DSL-Based Model Creation
```sruja
specification {
  element system
  element container
  element component
}

model {
  S = system "My System" {
    C = container "API" {
      Comp = component "Controller"
    }
  }
}

views {
  view index {
    include *
  }
}
```

#### 2. Parser & AST
- **Location**: `pkg/language/ast_likec4.go`
- **Types**: `SpecificationBlock`, `ModelBlock`, `LikeC4ElementDef`, `LikeC4ViewsBlock`
- **Parsing**: Converts DSL to structured AST

#### 3. Export to LikeC4 Format
- **Location**: `pkg/export/likec4/likec4.go`
- **Function**: Converts Sruja AST to LikeC4 JSON
- **DSL Export**: `pkg/export/likec4/likec4_dsl.go` - Exports back to DSL

#### 4. UI Builder (Designer App)
- **Location**: `apps/designer/src/components/Wizard/BuilderWizard.tsx`
- **Type**: Interactive UI wizard
- **Purpose**: Visual model creation
- **Not a programmatic API**

### What Sruja Lacks

❌ **No Programmatic Builder API**
- No Go equivalent of LikeC4's Builder
- No TypeScript/JavaScript Builder API
- Cannot create models programmatically without DSL

## Comparison Table

| Feature | LikeC4 Builder | Sruja |
|---------|---------------|-------|
| Programmatic API | ✅ TypeScript Builder | ❌ None |
| DSL Support | ✅ Can export to DSL | ✅ Primary method |
| Fluent Interface | ✅ Chainable methods | ❌ N/A |
| Type Safety | ✅ TypeScript types | ⚠️ Go types (compile-time) |
| Specification-Based | ✅ Required | ✅ Supported |
| UI Builder | ❌ None | ✅ BuilderWizard |
| Export to LikeC4 JSON | ✅ Yes | ✅ Yes |
| Nested Elements | ✅ `.with()` method | ✅ DSL syntax |

## Recommendations

### Option 1: Implement Go Builder API (Recommended for Backend)

Create a programmatic Builder in Go similar to LikeC4's TypeScript version:

```go
// pkg/builder/builder.go
package builder

type Builder struct {
    spec *Specification
    elements map[string]*Element
    relations []*Relation
    views []*View
}

func ForSpecification(spec *Specification) *Builder {
    return &Builder{
        spec: spec,
        elements: make(map[string]*Element),
        relations: []*Relation{},
        views: []*View{},
    }
}

func (b *Builder) Model(fn func(*ModelBuilder)) *Builder {
    // Implementation
    return b
}

func (b *Builder) Build() *language.Program {
    // Convert to language.Program
}
```

**Pros:**
- Enables programmatic model generation in Go
- Useful for code generation, testing, migrations
- Maintains consistency with DSL approach

**Cons:**
- Requires significant implementation effort
- Need to maintain parity with DSL features

### Option 2: Implement TypeScript Builder API (Recommended for Frontend)

Create a TypeScript Builder API in the designer app:

```typescript
// apps/designer/src/builder/Builder.ts
export class Builder {
  static forSpecification(spec: Specification) {
    // Implementation similar to LikeC4
  }
  
  model(callback: ModelBuilderFunction) {
    // Implementation
  }
  
  toSrujaDsl(): string {
    // Convert to DSL string
  }
}
```

**Pros:**
- Enables programmatic model creation in frontend
- Can generate DSL from programmatic API
- Useful for UI-driven model creation

**Cons:**
- Duplicates some logic from Go parser
- Need to sync with DSL syntax

### Option 3: Keep DSL-First Approach (Current)

**Pros:**
- Simpler architecture
- Single source of truth (DSL)
- No API maintenance overhead
- Users learn DSL syntax

**Cons:**
- Cannot programmatically generate models
- Harder for code generation use cases
- Less flexible for dynamic model creation

## Conclusion

**Current Status**: Sruja does NOT implement LikeC4's Builder pattern. It uses a DSL-first approach with a UI wizard.

**Key Differences**:

1. **LikeC4 Builder**:
   - Programmatic TypeScript API
   - Creates models in memory (Maps, arrays)
   - Returns `ParsedLikeC4ModelData` or `LikeC4Model.Computed`
   - Type-safe with complex TypeScript types
   - Two usage styles (compositional vs chainable)

2. **Sruja Current**:
   - DSL-first (`.sruja` files)
   - Parser converts DSL → AST
   - Exporter converts AST → LikeC4 JSON
   - UI wizard for visual editing
   - No programmatic API

**Recommendation**: 
- If you need programmatic model creation, implement Option 1 (Go) or Option 2 (TypeScript)
- If DSL-first approach works for your use case, Option 3 is fine

The choice depends on your use cases:
- **Code generation**: Need Builder API
- **Interactive editing**: Current UI wizard is sufficient
- **Testing/migrations**: Builder API would be helpful
- **Dynamic model creation**: Builder API essential

## Reference Implementation

The LikeC4 Builder implementation can be found in:
- Main file: `.likec4-ref/packages/core/src/builder/Builder.ts` (976 lines)
- Supporting files:
  - `Builder.model.ts` - Model building logic
  - `Builder.element.ts` - Element creation with `.with()` nesting
  - `Builder.views.ts` - View creation
  - `Builder.deployment.ts` - Deployment model
  - `Builder.view-common.ts` - View helpers ($include, $exclude, $style)
  - `_types.ts` - Complex TypeScript type system

**Test Examples**:
- `.likec4-ref/packages/core/src/builder/Builder-style1.spec.ts` - Compositional style
- `.likec4-ref/packages/core/src/builder/Builder-style2.spec.ts` - Chainable style
