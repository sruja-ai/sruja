# Sruja Builder API

Programmatic API for creating Sruja architecture models in TypeScript.

## SrujaBuilder - Using LikeC4's Builder (Recommended) ✅

Uses LikeC4's tested and proven Builder implementation, adapted for Sruja.
**Uses LikeC4's types directly** - no type conversion needed for input.

```typescript
import { SrujaBuilder, type BuilderSpecification } from "@sruja/shared/builder";

// Use LikeC4's BuilderSpecification type directly
const spec: BuilderSpecification = {
  elements: {
    system: {},
    container: {},
    component: {},
  },
  tags: ['frontend', 'backend'],
};

const { builder, model, views } = SrujaBuilder.forSpecification(spec);

const result = builder
  .with(
    model(
      system('cloud', 'Cloud System').with(
        component('api', 'API Server', { technology: 'Node.js' }),
        component('frontend', 'Web Frontend', { technology: 'React' })
          .with(
            relTo('cloud.api', 'API calls')
          )
      )
    )
  )
  .with(
    views(
      view('index', 'Overview').with($include('*'))
    )
  )
  .build(); // Returns SrujaModelDump (converted from LikeC4 format)
```

**Benefits:**
- ✅ Uses LikeC4's proven implementation
- ✅ **Uses LikeC4's types directly** (`BuilderSpecification`, etc.)
- ✅ Full feature support (deployment, views, etc.)
- ✅ Well-tested
- ✅ Type-safe
- ✅ Automatic conversion to `SrujaModelDump` format

**Note:** 
- Requires `@likec4/core` as a dependency (already included in Sruja projects)
- Input types use LikeC4's types (no conversion needed)
- Output is converted to `SrujaModelDump` format (for Go backend compatibility)

## Type Usage

### Input Types (Use LikeC4's Types Directly)

```typescript
import type { BuilderSpecification } from "@sruja/shared/builder";

// Use LikeC4's BuilderSpecification - no conversion needed
const spec: BuilderSpecification = {
  elements: ['system', 'container'], // Array form
  // OR
  elements: {                        // Object form with config
    system: { style: { color: 'blue' } },
    container: {},
  },
  tags: ['frontend', 'backend'],
  deployments: ['env', 'node'],
};
```

### Output Type (SrujaModelDump)

The `.build()` method automatically converts LikeC4's `ParsedLikeC4ModelData` to `SrujaModelDump`:

```typescript
const result: SrujaModelDump = builder.build();
// result is compatible with Sruja's architecture store and Go backend
```

### Advanced: Access LikeC4 Model Directly

If you need LikeC4-specific features (computed views, etc.):

```typescript
const likec4Model = builder.buildLikeC4(); // Returns ParsedLikeC4ModelData
const computed = builder.toLikeC4Model();   // Returns computed model with views
```

## Examples

See `example-usage.ts` for complete usage examples.
