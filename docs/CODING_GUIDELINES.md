# Coding Guidelines

This document outlines coding standards and conventions for the Sruja codebase to ensure consistency, maintainability, and code quality.

**Primary stack:** The Sruja CLI and core libraries are implemented in **Rust**. Rust modules live under `crates/`; CLI commands are split by domain in `crates/sruja-cli/src/commands/` (see `crates/sruja-cli/REFACTORING_PLAN.md`). The VS Code extension is implemented in **TypeScript** under `extension/`.

## Table of Contents

1. [CLI Commands](#cli-commands)
2. [TypeScript Exports](#typescript-exports)
3. [Error Handling](#error-handling)
4. [Logging](#logging)
5. [Type Safety](#type-safety)
6. [Test File Naming](#test-file-naming)
7. [Configuration Files](#configuration-files)

## CLI Commands

### Structure

CLI subcommands are defined via Clap in [main.rs](file:///Users/dilipkola/Workspace/sruja/crates/sruja-cli/src/main.rs) and implemented as functions under [commands/](file:///Users/dilipkola/Workspace/sruja/crates/sruja-cli/src/commands).

**Rules:**

- Add new command logic under `crates/sruja-cli/src/commands/<domain>.rs` (or a new module if needed).
- Prefer command functions that return `Result<(), CliError>` (sync) or `async fn ... -> Result<(), CliError>` (async).
- Export the function from [commands/mod.rs](file:///Users/dilipkola/Workspace/sruja/crates/sruja-cli/src/commands/mod.rs) and wire it into Clap in `main.rs`.
- Use `println!` for user-facing command output and `eprintln!` for error output; use `log::{debug,info,warn,error}` for internal diagnostics.

## TypeScript Exports

### Preferred Patterns

**2. Named Exports (Always Preferred)**

```typescript
// ✅ Good - named export
export function myFunction() { ... }
export type MyType = ...
export interface MyInterface { ... }

// ❌ Avoid - default export (except for entry points required by tooling)
export default function entryPoint() { ... }
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
- **Avoid default exports** except for:
  - Main entry points
  - When required by tooling conventions
- **Use `export const`** for constants and simple utilities
- **Use `export type`** for type-only exports

## Error Handling

### Rust Error Handling

**Rules:**

- Prefer `Result<T, E>` with domain-specific error types (e.g. `CliError`).
- Use `?` for propagation and add context at the call site where it’s most meaningful.
- Avoid `unwrap()`/`expect()` outside tests.

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

**Use VS Code primitives (extension):**

```typescript
import * as vscode from "vscode";

const channel = vscode.window.createOutputChannel("Sruja");
channel.appendLine("Running sruja drift -r . ...");

try {
  // ...
} catch (err) {
  const msg = err instanceof Error ? err.message : String(err);
  channel.appendLine(`Error: ${msg}`);
  vscode.window.showErrorMessage("Sruja command failed: " + msg);
}
```

**Rules:**

- ❌ **Never use** `console.log`, `console.error`, `console.warn`, `console.debug`
- ✅ **Use** `vscode.window.createOutputChannel("Sruja")` for command/session logs
- ✅ **Use** `vscode.window.showErrorMessage` / `showWarningMessage` for user-visible notifications

### Rust Logging

Rust code uses the `log` crate (`log::{debug, info, warn, error}`), with the CLI initializing `env_logger` where appropriate.

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

### Rust Tests

- Unit tests live in `#[cfg(test)]` modules within crates.
- Integration tests live under `crates/*/tests/` (for example, the CLI has tests under `crates/sruja-cli/tests/`).

### TypeScript Tests (extension)

- **Pattern:** `*.test.ts` or `*.test.tsx`
- **Location:** Existing extension tests live under `extension/src/test/`.

## Configuration Files

### File Extensions

**Config files should match their toolchain:**

- Rust: `Cargo.toml`, `rustfmt.toml` (if present)
- Node/TypeScript (extension and e2e): `package.json`, `tsconfig.json`

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

- [ ] CLI command logic lives under `crates/sruja-cli/src/commands/` and returns `Result<(), CliError>`
- [ ] TypeScript uses named exports (avoid default exports)
- [ ] Extension logging uses VS Code OutputChannel (never `console.*`)
- [ ] No `any` types - use proper types or `unknown` with type guards
- [ ] Test files follow naming conventions (`crates/*/tests` for Rust integration, `*.test.ts` for TypeScript)
- [ ] Configuration files use appropriate extensions (`.ts` when possible)
- [ ] Errors include actionable context and avoid `unwrap()` outside tests

## Additional Resources

- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
