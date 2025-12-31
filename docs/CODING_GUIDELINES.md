# Coding Guidelines

This document outlines coding standards and conventions for the Sruja codebase to ensure consistency, maintainability, and code quality.

## Table of Contents

1. [CLI Commands](#cli-commands)
2. [TypeScript Exports](#typescript-exports)
3. [Error Handling](#error-handling)
4. [Logging](#logging)
5. [Type Safety](#type-safety)
6. [Test File Naming](#test-file-naming)
7. [Configuration Files](#configuration-files)

## CLI Commands

### Function Signatures

All CLI command functions must follow this standard signature:

```go
func runXxx(args []string, stdout, stderr io.Writer) int
```

**Rules:**

- Function name: `run` + PascalCase command name (e.g., `runCompile`, `runLint`)
- Parameters: `args []string`, `stdout io.Writer`, `stderr io.Writer`
- Return: `int` (0 for success, non-zero for failure)
- Use `stdout` for normal output, `stderr` for errors
- Wrap in Cobra command with `RunE` that converts int return to error

**Example:**

```go
func runInit(args []string, stdout, stderr io.Writer) int {
    projectName := "my-sruja-project"
    if len(args) > 0 {
        projectName = args[0]
    }

    if err := os.MkdirAll(projectName, 0o750); err != nil {
        _, _ = fmt.Fprintf(stderr, "Error: failed to create project directory: %v\n", err)
        return 1
    }

    _, _ = fmt.Fprintf(stdout, "Initialized project in %s\n", projectName)
    return 0
}
```

**Cobra Integration:**

```go
var initCmd = &cobra.Command{
    Use:   "init [project-name]",
    Short: "Initialize a new Sruja project",
    RunE: func(cmd *cobra.Command, args []string) error {
        if runInit(args, cmd.OutOrStdout(), cmd.ErrOrStderr()) != 0 {
            return fmt.Errorf("init failed")
        }
        return nil
    },
}
```

## TypeScript Exports

### Preferred Patterns

**1. Function Declarations (Preferred for Components)**

```typescript
// ✅ Preferred for React components
export function MyComponent() {
  return <div>Hello</div>;
}
```

**2. Named Exports (Always Preferred)**

```typescript
// ✅ Good - named export
export function myFunction() { ... }
export type MyType = ...
export interface MyInterface { ... }

// ❌ Avoid - default export (except for entry points)
export default function MyComponent() { ... }
```

**3. Const Arrow Functions (Acceptable but Less Preferred)**

```typescript
// ⚠️ Acceptable but less preferred for components
export const MyComponent = () => {
  return <div>Hello</div>;
};
```

**4. Re-exports**

```typescript
// ✅ Good - re-export with type
export type { SomeType } from "./other";
export { someFunction } from "./other";
```

### Rules

- **Always use named exports** for functions, types, and interfaces
- **Prefer function declarations** over const arrow functions for React components (better for debugging)
- **Avoid default exports** except for:
  - Main entry points (e.g., `index.tsx`)
  - When required by framework conventions
- **Use `export const`** for constants and simple utilities
- **Use `export type`** for type-only exports

## Error Handling

### Go Error Handling

**Standard Pattern:**

```go
// ✅ Good - wrap errors with context
if err != nil {
    return fmt.Errorf("operation failed: %w", err)
}

// ✅ Good - simple errors
if condition {
    return errors.New("invalid argument")
}
```

**Rules:**

- Always use `fmt.Errorf` with `%w` verb for error wrapping (Go 1.13+)
- Use `errors.New` for simple errors without wrapping
- Include context in error messages
- Use descriptive error messages

### TypeScript Error Handling

**Standard Pattern:**

```typescript
// ✅ Good - proper error handling
try {
  const result = await someAsyncOperation();
  return result;
} catch (error) {
  const message = error instanceof Error ? error.message : "Unknown error";
  throw new Error(`Operation failed: ${message}`);
}
```

## Logging

### TypeScript Logging

**Always use the centralized logger:**

```typescript
import { logger } from "@sruja/shared";

// ✅ Good - structured logging
logger.error("DSL parse failed", {
  component: "DSLPanel",
  action: "parse_dsl",
  error: error instanceof Error ? error.message : String(error),
});

logger.warn("Score calculation skipped", {
  component: "GovernancePanel",
  action: "calculateScore",
  error: error instanceof Error ? error.message : String(error),
});

logger.info("Model loaded successfully", {
  component: "ArchitectureStore",
  action: "loadFromDSL",
  elementCount: model.elements?.length || 0,
});

logger.debug("Cache hit", {
  component: "Cache",
  key: cacheKey,
});
```

**Rules:**

- ❌ **Never use** `console.log`, `console.error`, `console.warn`, `console.debug`
- ✅ **Always use** `logger` from `@sruja/shared`
- Use structured logging with context objects
- Use appropriate log levels: `error`, `warn`, `info`, `debug`
- Include component and action in context for traceability

### Go Logging

Go code should use standard library logging or structured logging libraries as appropriate for the context.

## Type Safety

### TypeScript Type Safety

**Avoid `any` Type:**

```typescript
// ❌ Bad - using any
const editor: any = null;
function handleEditor(_monaco: any, editor: any) { ... }

// ✅ Good - proper types
import type * as monacoTypes from "monaco-editor";
const editor: monacoTypes.editor.IStandaloneCodeEditor | null = null;
function handleEditor(
  _monaco: typeof monacoTypes,
  editor: monacoTypes.editor.IStandaloneCodeEditor
) { ... }
```

**Rules:**

- ❌ **Never use `any`** - always use proper types
- ✅ **Use `unknown`** when types are truly unknown, then narrow with type guards
- ✅ **Use proper type imports** from libraries (e.g., `monaco-editor`)
- ✅ **Create type guards** for runtime type checking
- ✅ **Use type assertions** only when necessary and safe (`as Type`)

**Type Guards Example:**

```typescript
function isSrujaModel(data: unknown): data is SrujaModelDump {
  return typeof data === "object" && data !== null && "elements" in data && "relations" in data;
}
```

## Test File Naming

### Go Tests

- **Pattern:** `*_test.go`
- **Location:** Same directory as source files
- **Example:** `parser.go` → `parser_test.go`

### TypeScript Tests

- **Pattern:** `*.test.ts` or `*.test.tsx`
- **Location:**
  - **Preferred:** Co-located with source files (e.g., `utils.ts` → `utils.test.ts`)
  - **Alternative:** `__tests__` subdirectory for complex test suites
- **Example:** `architectureStore.ts` → `architectureStore.test.ts` or `__tests__/architectureStore.test.ts`

## Configuration Files

### File Extensions

**ESLint Configs:**

- ✅ Use `.ts` extension when TypeScript is available
- ⚠️ Use `.js` only when TypeScript is not available
- ⚠️ Use `.mjs` only when required by tooling (e.g., Astro)

**Other Configs:**

- `tsconfig.json` - TypeScript configuration
- `package.json` - Package configuration
- `vite.config.ts` - Vite configuration
- `vitest.config.ts` - Vitest configuration

### Package.json Structure

**Field Ordering (Recommended):**

1. `name`
2. `version`
3. `description` (if present)
4. `license`
5. `private`
6. `type` (if present)
7. `main` / `exports` (if present)
8. `scripts`
9. `dependencies`
10. `devDependencies`
11. `peerDependencies` (if present)
12. `engines` (if present)
13. `packageManager` (if present)

## Summary Checklist

When writing code, ensure:

- [ ] CLI commands use standard `runXxx(args []string, stdout, stderr io.Writer) int` signature
- [ ] TypeScript uses named exports (avoid default exports)
- [ ] React components use function declarations when possible
- [ ] All logging uses `logger` from `@sruja/shared` (never `console.*`)
- [ ] No `any` types - use proper types or `unknown` with type guards
- [ ] Test files follow naming conventions (`*_test.go` for Go, `*.test.ts` for TypeScript)
- [ ] Configuration files use appropriate extensions (`.ts` when possible)
- [ ] Errors are wrapped with context using `fmt.Errorf` with `%w` verb

## Additional Resources

- [Go Error Handling Best Practices](https://go.dev/blog/error-handling-and-go)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [React Component Patterns](https://react.dev/learn/your-first-component)
