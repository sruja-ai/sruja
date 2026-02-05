# Sruja FAANG Improvements - Quick Start Guide

**Last Updated**: 2025-01-20  
**For**: Frontend Engineers  
**Prerequisites**: TypeScript 5.6+, React 19+

---

## 🚀 Quick Reference

### New Architecture Layers

```
┌─────────────────────────────────────────┐
│           Presentation Layer           │
│        (Components, Hooks)            │
└──────────────┬──────────────────────┘
               │ uses
┌──────────────▼──────────────────────┐
│         Application Layer             │
│       (Use Cases, Commands)        │
└──────────────┬──────────────────────┘
               │ uses
┌──────────────▼──────────────────────┐
│           Domain Layer              │
│   (Aggregates, Value Objects)     │
└──────────────┬──────────────────────┘
               │ uses
┌──────────────▼──────────────────────┐
│       Infrastructure Layer           │
│   (Repositories, Adapters)         │
└─────────────────────────────────────┘
```

---

## 📦 Key Packages & Exports

### Domain Layer

```typescript
// Aggregates
import { ArchitectureAggregate } from "@/domain/aggregates/ArchitectureAggregate";

// Value Objects
import { ElementId, isElementId } from "@/domain/value-objects/ElementId";
import { ElementRelationship, RelationshipKind } from "@/domain/value-objects/ElementRelationship";

// Services
import { ArchitectureValidationService } from "@/domain/services/ArchitectureValidationService";

// Repository Interface
import type { ArchitectureRepository } from "@/domain/repositories/ArchitectureRepository";
```

### Application Layer

```typescript
// Use Cases
import { ArchitectureUseCases } from "@/application/use-cases/ArchitectureUseCases";
```

### Infrastructure Layer

```typescript
// IndexedDB Implementation
import { IndexedDBArchitectureRepository } from "@/infrastructure/adapters/indexeddb/IndexedDBArchitectureRepository";
```

---

## 🎯 Common Patterns

### 1. Creating a New Element

**Old Way** (Direct Store Access - DEPRECATED):

```typescript
// ❌ Don't do this
const element = { id: "user", name: "User", kind: "person" };
useArchitectureStore.setState((s) => ({
  elements: { ...s.elements, [element.id]: element },
}));
```

**New Way** (Domain Use Cases):

```typescript
// ✅ Do this
import { ElementId } from "@/domain/value-objects/ElementId";
import { ArchitectureUseCases } from "@/application/use-cases/ArchitectureUseCases";

const useCreateElement = () => {
  const aggregate = useArchitectureStore(selectAggregate);
  const useCases = useArchitectureUseCases();

  const createElement = async (name: string, kind: string) => {
    // Create type-safe ID
    const idResult = ElementId.fromName(name);
    if (!idResult.ok) {
      showToast("Error", idResult.error.message);
      return;
    }

    const element: Element = {
      id: idResult.value,
      name,
      kind,
      description: `A ${kind} element`,
    };

    // Use use case
    const result = useCases.createElement(aggregate, element);
    if (!result.ok) {
      showToast("Error", result.error.message);
      return;
    }

    // Update store
    useArchitectureStore.setAggregate(result.value.aggregate);
  };

  return { createElement };
};
```

### 2. Handling Errors

**Old Way** (Try-Catch):

```typescript
// ❌ Don't do this
try {
  await saveData(data);
} catch (error) {
  console.error("Failed to save:", error);
  alert("Something went wrong");
}
```

**New Way** (Result Type):

```typescript
// ✅ Do this
import { ValidationError, NetworkError } from "@sruja/shared/utils/errors";

const handleCreateElement = async () => {
  const result = useCases.createElement(aggregate, element);

  if (!result.ok) {
    // Type-safe error handling
    if (result.error instanceof ValidationError) {
      showToast("Validation Error", result.error.message, "error");
    } else if (result.error instanceof NetworkError) {
      showToast("Network Error", "Please check your connection", "error");
    } else {
      showToast("Error", "An unexpected error occurred", "error");
    }
    return;
  }

  // Success
  showToast("Success", "Element created successfully", "success");
};
```

### 3. Type-Safe IDs

**Old Way** (Plain String):

```typescript
// ❌ Don't do this
let elementId = "user-service"; // Could be invalid
elementId = "invalid$id!"; // No validation
```

**New Way** (ElementId Value Object):

```typescript
// ✅ Do this
import { ElementId } from "@/domain/value-objects/ElementId";

// Create with validation
const idResult = ElementId.create("user-service");
if (!idResult.ok) {
  console.error("Invalid ID:", idResult.error);
  return;
}
const elementId = idResult.value;

// Generate unique ID
const uniqueId = ElementId.generate("component");
// => 'component-abc123-xyz789'

// Convert from name
const nameId = ElementId.fromName("My User Service");
if (nameId.ok) {
  const safeId = nameId.value; // 'my-user-service'
}

// Use in code
console.log(elementId.value); // 'user-service'
console.log(elementId.getPrefix()); // 'user'
console.log(elementId.getSuffix()); // 'service'
console.log(elementId.startsWith("user")); // true
```

### 4. Creating Relationships

```typescript
import { ElementRelationship, RelationshipKind } from "@/domain/value-objects/ElementRelationship";

const createRelationship = async (sourceId: string, targetId: string, description: string) => {
  // Create with validation
  const result = ElementRelationship.create(sourceId, targetId, description, {
    kind: RelationshipKind.USES,
    technology: "HTTP/REST",
    direction: "forward",
  });

  if (!result.ok) {
    showToast("Error", result.error.message);
    return;
  }

  // Add to architecture
  const addResult = useCases.addRelationship(aggregate, sourceId, targetId, description, {
    kind: RelationshipKind.USES,
  });

  if (addResult.ok) {
    useArchitectureStore.setAggregate(addResult.value.aggregate);
  }
};
```

### 5. Using Repository

```typescript
import { IndexedDBArchitectureRepository } from "@/infrastructure/adapters/indexeddb/IndexedDBArchitectureRepository";

// Initialize repository
const repository = new IndexedDBArchitectureRepository({
  cacheEnabled: true,
  cacheTTL: 5 * 60 * 1000, // 5 minutes
  compressionEnabled: true,
});

await repository.initialize();

// Save architecture
const saveResult = await repository.save(aggregate);
if (saveResult.ok) {
  console.log("Saved with ID:", saveResult.value);
}

// Load architecture
const loadResult = await repository.findById("my-architecture");
if (loadResult.ok) {
  console.log("Loaded:", loadResult.value.metadata.name);
}

// Search architectures
const searchResult = await repository.search("user service");
if (searchResult.ok) {
  console.log(`Found ${searchResult.value.items.length} results`);
}

// Cleanup
await repository.close();
```

### 6. Validating Architecture

```typescript
import { ArchitectureValidationService } from "@/domain/services/ArchitectureValidationService";

const validateArchitecture = (aggregate: ArchitectureAggregate) => {
  const validator = new ArchitectureValidationService({
    detectCycles: true,
    detectOrphans: true,
    requireDescriptions: true,
    maxNestingDepth: 10,
  });

  const report = validator.validateModel(aggregate.toDump());

  if (!report.isValid) {
    console.error("Validation failed:");
    report.issues.forEach((issue) => {
      console.error(`  [${issue.severity}] ${issue.code}: ${issue.message}`);
      if (issue.suggestion) {
        console.error(`    Suggestion: ${issue.suggestion}`);
      }
    });
  }

  if (report.hasWarnings) {
    console.warn("Warnings:");
    report.issues
      .filter((i) => i.severity === "warning")
      .forEach((issue) => {
        console.warn(`  [${issue.code}] ${issue.message}`);
      });
  }

  console.log(`Quality Score: ${report.qualityScore}/100`);

  return report;
};
```

---

## 🔄 Migration Checklist

### Component Migration

- [ ] Extract business logic to use cases
- [ ] Replace direct store access with use cases
- [ ] Use Result type for error handling
- [ ] Replace string IDs with ElementId
- [ ] Add proper TypeScript types
- [ ] Write tests for the use cases

### Data Layer Migration

- [ ] Implement repository interface
- [ ] Use value objects for IDs and relationships
- [ ] Add validation at boundaries
- [ ] Implement caching strategy
- [ ] Add error handling and recovery
- [ ] Write integration tests

### Testing Migration

- [ ] Write unit tests for domain logic
- [ ] Write tests for use cases
- [ ] Write tests for adapters
- [ ] Add integration tests for flows
- [ ] Add E2E tests for critical paths
- [ ] Achieve 85%+ coverage

---

## 🎨 Best Practices

### 1. Type Safety

- ✅ Use `ElementId` instead of `string` for IDs
- ✅ Use `Result<T, E>` instead of try-catch for recoverable errors
- ✅ Use custom error types (`ValidationError`, `NetworkError`, etc.)
- ✅ Avoid `any` types
- ✅ Use `isElementId()` type guards

### 2. Error Handling

- ✅ Use `Result` type for recoverable errors
- ✅ Throw only for unrecoverable errors
- ✅ Provide context and suggestions in errors
- ✅ Handle errors at appropriate layer
- ✅ Log errors with structured data

### 3. State Management

- ✅ Use selectors to avoid unnecessary re-renders
- ✅ Batch state updates when possible
- ✅ Compute derived state with `useMemo`
- ✅ Keep store slices focused and small
- ✅ Use use cases for state mutations

### 4. Component Design

- ✅ Keep components < 300 lines
- ✅ Use composition over inheritance
- ✅ Extract business logic to hooks/use cases
- ✅ Use `React.memo` for expensive components
- ✅ Use `useCallback` for event handlers

### 5. Testing

- ✅ Follow AAA pattern (Arrange-Act-Assert)
- ✅ Test behavior, not implementation
- ✅ Use test doubles for dependencies
- ✅ Write descriptive test names
- ✅ Test error cases

---

## 🔧 Common Tasks

### Adding a New Use Case

1. **Create use case file**:

   ```typescript
   // src/application/use-cases/MyUseCase.ts
   import { ArchitectureAggregate } from "@/domain/aggregates/ArchitectureAggregate";
   import { ValidationError, ok, err, type Result } from "@sruja/shared/utils";

   export interface MyUseCaseOptions {
     // Options here
   }

   export interface MyUseCaseResult {
     // Result structure
   }

   export function myUseCase(
     aggregate: ArchitectureAggregate,
     options: MyUseCaseOptions
   ): Result<MyUseCaseResult, ValidationError> {
     // Implementation
     return ok(result);
   }
   ```

2. **Add to ArchitectureUseCases**:

   ```typescript
   export class ArchitectureUseCases {
     // Existing methods...

     myUseCase(options: MyUseCaseOptions): Result<MyUseCaseResult, ValidationError> {
       const aggregate = useArchitectureStore(selectAggregate);
       return myUseCase(aggregate, options);
     }
   }
   ```

3. **Add tests**:
   ```typescript
   // src/application/use-cases/MyUseCase.test.ts
   describe("myUseCase", () => {
     it("should handle valid input", () => {
       const aggregate = ArchitectureAggregate.createEmpty();
       const result = myUseCase(aggregate.value, {
         /* options */
       });
       expect(result.ok).toBe(true);
     });
   });
   ```

### Creating a Custom Repository Adapter

1. **Implement interface**:

   ```typescript
   export class CustomRepository implements ArchitectureRepository {
     async initialize() {
       /* ... */
     }
     async findById(id: string) {
       /* ... */
     }
     async save(aggregate: ArchitectureAggregate) {
       /* ... */
     }
     // Implement all interface methods
   }
   ```

2. **Register with dependency injection** (if applicable):
   ```typescript
   const repository = new CustomRepository(config);
   await repository.initialize();
   ```

---

## 📚 Additional Resources

### Documentation

- [Full FAANG Improvements](./FAANG_IMPROVEMENTS.md)
- [Architecture Decisions](./docs/adr/)
- [API Reference](./docs/api/)
- [Component Library](./docs/components/)

### Learning

- [Clean Architecture Guide](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Domain-Driven Design](https://domainlanguage.com/ddd/)
- [TypeScript Best Practices](https://typescript-eslint.io/rules/)
- [React Testing Library](https://testing-library.com/docs/react-testing-library/intro)

### Support

- **Slack**: #sruja-architecture
- **GitHub Issues**: [Create Issue](https://github.com/sruja-ai/sruja/issues)
- **Office Hours**: Thursdays 2-3pm PT

---

## ✅ Quick Wins

### Immediate (Today)

1. ✅ Start using `ElementId` for new IDs
2. ✅ Replace try-catch with `Result` type in new code
3. ✅ Add type annotations to existing code
4. ✅ Write tests for critical paths

### This Week

1. ✅ Migrate one component to use use cases
2. ✅ Implement repository pattern for data layer
3. ✅ Add integration tests for a critical flow
4. ✅ Set up performance monitoring

### This Month

1. ✅ Migrate all components to new architecture
2. ✅ Achieve 85% test coverage
3. ✅ Optimize bundle sizes
4. ✅ Create architecture documentation

---

## 🆘 Troubleshooting

### Common Issues

**Q: How do I convert existing string IDs to ElementId?**

```typescript
// Option 1: Create (validates)
const idResult = ElementId.create(existingId);
if (idResult.ok) {
  const elementId = idResult.value;
}

// Option 2: Unsafe (if you know it's valid)
const elementId = ElementId.unsafe(existingId);
```

**Q: How do I handle async operations with Result type?**

```typescript
const result = await someAsyncOperation();
if (!result.ok) {
  // Handle error
  return;
}
// Use result.value
```

**Q: How do I access the repository from a component?**

```typescript
const useRepository = () => {
  const repository = useMemo(() => {
    const repo = new IndexedDBArchitectureRepository();
    repo.initialize();
    return repo;
  }, []);

  useEffect(() => {
    return () => repository.close();
  }, [repository]);

  return repository;
};
```

**Q: How do I optimize re-renders with selectors?**

```typescript
// Bad: Re-renders on any store change
const model = useArchitectureStore((s) => s.model);

// Good: Only re-renders when model changes
const selectModel = (state: ArchitectureState) => state.model;
const model = useArchitectureStore(selectModel);

// Even better: Memoize complex selections
const selectElementsByKind = (kind: string) => (state: ArchitectureState) => {
  return Object.values(state.model.elements).filter((el) => el.kind === kind);
};
const components = useArchitectureStore(selectElementsByKind("component"));
```

**Q: WebSocket / HMR errors (ws://localhost:5173 failed, chunk 404s, invalid hook call)?**

- The website (Astro) runs on **port 4321**; the designer (Vite) runs on **5173**. If you run `npm run dev` from the repo root, both start and the browser may try to connect HMR or load assets from the wrong port.
- **Fix**: Use the website only when working on the site: `npm run dev:website` (or `cd apps/website && npm run dev`). Then open **http://localhost:4321**.
- The Astro config sets `server.origin` and `server.hmr` so that when you use the website dev server, all asset URLs and the HMR WebSocket use `localhost:4321`. If you still see 5173 in errors, hard-refresh (Ctrl+Shift+R / Cmd+Shift+R) or clear site data for localhost.

---

**Start building FAANG-quality code today! 🚀**
