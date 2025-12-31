# Sruja Types

Type definitions for Sruja architecture models.

## Using Core Types (Recommended)

**For TypeScript code, use core types directly:**

```typescript
import type {
  Element,
  Relationship,
  ParsedView,
  Specification,
  ParsedModelData,
  BuilderSpecification,
} from "@sruja/shared/types";

// Use core types in your code
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

| Core Type (TypeScript) | Sruja Dump Type (Go Backend) |
| ---------------------- | ---------------------------- |
| `Element`              | `ElementDump`                |
| `Relationship`         | `RelationDump`               |
| `ParsedView`           | `ViewDump`                   |
| `Specification`        | `SpecificationDump`          |
| `ParsedModelData`      | `SrujaModelDump`             |

**Recommendation:**

- Use core types in TypeScript code
- Convert to Dump types only when sending to Go backend
- Use Builder API which handles conversion automatically
