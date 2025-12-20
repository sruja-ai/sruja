# Sruja Types

Type definitions for Sruja architecture models.

## Using LikeC4 Types (Recommended)

**For TypeScript code, use LikeC4's types directly:**

```typescript
import type {
  Element,
  Relationship,
  ParsedView,
  Specification,
  ParsedLikeC4ModelData,
  BuilderSpecification,
} from "@sruja/shared/types";

// Use LikeC4 types in your code
const element: Element = { ... };
const spec: BuilderSpecification = { ... };
```

## Using Sruja Dump Types (Go Backend Compatibility)

**Only use "Dump" types when interfacing with Go backend:**

```typescript
import type {
  SrujaModelDump,
  ElementDump,
  RelationDump,
  SpecificationDump,
} from "@sruja/shared/types";

// Use Dump types when converting from/to Go backend JSON
const dump: SrujaModelDump = { ... };
```

## Type Mapping

| LikeC4 Type (TypeScript) | Sruja Dump Type (Go Backend) |
|---------------------------|------------------------------|
| `Element` | `ElementDump` |
| `Relationship` | `RelationDump` |
| `ParsedView` | `ViewDump` |
| `Specification` | `SpecificationDump` |
| `ParsedLikeC4ModelData` | `SrujaModelDump` |

**Recommendation:**
- Use LikeC4 types in TypeScript code
- Convert to Dump types only when sending to Go backend
- Use Builder API which handles conversion automatically
