# Architecture Plugin Build System v1

**Production-grade, simple, modern, Bun-first build system and bundling guide** for Architecture Validation Plugins.

This design:
- ✅ Works for **OSS plugins**, organization plugins, and marketplace plugins
- ✅ Works for **Node**, **Bun**, and **Browser**
- ✅ Ensures **safe ESM**, no file path hacks
- ✅ Supports **TypeScript**, **tree-shaking**, **side-effect-free rules**, and **hot loading**
- ✅ Works for **CLI + LSP + Cloud backend**
- ✅ Enables contributors to publish plugins easily
- ✅ Supports **version pinning** (like ESLint plugins or Terraform providers)

---

## 🎯 Overview

Your tool supports plugins that users can install:
- Locally in project
- Globally (optional)
- From GitHub
- From marketplace (future)
- From npm

Therefore the build & packaging **must be clean, minimal, and consistent**.

---

## 📁 Plugin Project Structure

This is what *every plugin* should look like.

```
my-plugin/
  src/
    index.ts
    rules/
      ruleA.ts
      ruleB.ts
  package.json
  tsconfig.json
  bunfig.toml
  README.md
  .gitignore
```

Simple. Clear. Universal.

---

## 📦 `package.json` Template (Official Plugin Standard)

```json
{
  "name": "@arch/plugin-pci-security",
  "version": "1.0.0",
  "description": "PCI DSS architectural validation rules.",
  "type": "module",
  "main": "./dist/index.js",
  "exports": {
    ".": "./dist/index.js"
  },
  "files": [
    "dist"
  ],
  "scripts": {
    "build": "bun build ./src/index.ts --outdir dist --target node",
    "dev": "bun build ./src/index.ts --outdir dist --target node --watch",
    "test": "vitest",
    "prepublishOnly": "bun run build"
  },
  "peerDependencies": {
    "@arch/validation": ">=1.0.0"
  },
  "devDependencies": {
    "@arch/validation": "workspace:*",
    "typescript": "^5.6.0",
    "vitest": "^2.0.0"
  },
  "keywords": [
    "architecture",
    "validation",
    "plugin",
    "pci",
    "compliance"
  ]
}
```

### Why `peerDependencies`?

Because your plugin extends your validation engine, but should **not** bundle it.

Same pattern as:
- ESLint plugins
- Prettier plugins
- Vite plugins

---

## ⚙️ `tsconfig.json`

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "outDir": "dist",
    "declaration": true,
    "declarationMap": true
  },
  "include": ["src"],
  "exclude": ["node_modules", "dist", "**/*.test.ts"]
}
```

---

## 🔧 `bunfig.toml`

Optional but recommended.

```toml
[build]
entrypoints = ["src/index.ts"]
outdir = "dist"
target = "node"
minify = true
sourcemap = "linked"
```

This ensures consistent builds even if contributors don't use the npm script.

---

## 🏗️ Build Command

Plugins build with:

```bash
bun run build
```

**Output:**

```
dist/
  index.js
  rules/
    ruleA.js
    ruleB.js
  index.js.map
```

**Build Options:**

```bash
# Production build (minified)
bun run build

# Development build (with watch)
bun run dev

# Build with sourcemaps
bun build ./src/index.ts --outdir dist --target node --sourcemap
```

---

## 📝 Standard Plugin Entry File (`src/index.ts`)

```typescript
import type { ValidationPlugin } from "@arch/validation";

import ruleA from "./rules/ruleA.js";
import ruleB from "./rules/ruleB.js";

const plugin: ValidationPlugin = {
  name: "pci-security",
  version: "1.0.0",
  rules: [ruleA, ruleB]
};

export default plugin;
```

Everything else stays isolated.

---

## Go Extension Modalities

### Primary: Go-Native Plugins

```go
package validation

type Rule interface {
    ID() string
    Apply(ctx Context) []Issue
}

type Plugin interface {
    Name() string
    Version() string
    Rules() []Rule
}
```

- Discovery: declared in `sruja.json` imports as Go module paths.
- Loading: resolved by the Go CLI and linked at build or fetched via Git.
- Execution: rules run in-process via the Validation Engine.

### Optional: JS Plugins via Subprocess

- Runtime: Bun or Node invoked by the Go CLI as a subprocess.
- Contract: JSON stdin/stdout for rule inputs and issues.
- Isolation: subprocess boundaries for safety; disabled by default.

### Optional: WASM Plugins

- ABI: stable interfaces for rule evaluation compiled to WASM.
- Loading: Go CLI loads `.wasm` modules and calls exported functions.
### Versioning and Security

- Version pinning via `sruja.json` import strings.
- Signature verification recommended for remote templates and plugins.
## ✅ Rules Must Be Pure Functions

Rules must:
- ✅ Take context
- ✅ Return issues
- ✅ Be deterministic
- ✅ Have no side effects
- ✅ Have no I/O (cloud-safe)
- ✅ Be fully serializable during analysis

This ensures:
- ✅ Cloud cluster consistency
- ✅ Reproducible validation
- ✅ Caching compatibility
- ✅ Safe plugin hot-reloading

**Example Pure Rule:**

```typescript
// ✅ Good: Pure function
export const rule: ValidationRule = {
  id: "pci/no-plain-card-data",
  apply(ctx: ValidationContext) {
    const issues = [];
    // Only reads from ctx, no I/O, no side effects
    for (const node of ctx.model.nodes) {
      // ... validation logic
    }
    return issues;
  }
};

// ❌ Bad: Has side effects
export const badRule: ValidationRule = {
  id: "bad/rule",
  apply(ctx: ValidationContext) {
    console.log("Validating..."); // ❌ Side effect
    fetch("https://api.example.com/check"); // ❌ I/O
    return [];
  }
};
```

---

## 🔍 Plugin Resolution in Your Engine

Your engine uses 3 resolution strategies:

### 1. Local Project Plugins

```
./validators/* 
.architecture/config.json → plugins: ["./validators/pci.js"]
```

### 2. Node/Bun Module Resolution

```
@arch/plugin-pci-security
```

### 3. Absolute Paths

```
/home/plugin-dir/my.js
```

All loaded via:

```typescript
const plugin = await import(pluginPathOrPackageId);
ValidationPluginSchema.parse(plugin.default ?? plugin);
```

---

## 🔒 Safe Loading (Production Hardened)

I recommend a **safe dynamic import wrapper**.

**File:** `packages/validation/engine/safeImport.ts`

```typescript
export async function safeImport(path: string) {
  try {
    const mod = await import(path);
    return mod.default ?? mod;
  } catch (err) {
    return {
      __invalid: true,
      __error: `Failed to load plugin at ${path}: ${err.message}`
    };
  }
}
```

**Used in plugin loader:**

**File:** `packages/validation/engine/plugins.ts`

```typescript
import { ValidationPluginSchema } from "../types/plugin-schema.js";
import { safeImport } from "./safeImport.js";

export async function loadPlugins(
  pluginPaths: string[]
): Promise<{ plugins: ValidationPlugin[]; errors: PluginLoadError[] }> {
  const plugins: ValidationPlugin[] = [];
  const errors: PluginLoadError[] = [];

  for (const pluginPath of pluginPaths) {
    try {
      const raw = await safeImport(pluginPath);

      if (raw.__invalid) {
        errors.push({ 
          pluginPath, 
          reason: raw.__error 
        });
        continue;
      }

      const parsed = ValidationPluginSchema.safeParse(raw);
      
      if (!parsed.success) {
        errors.push({ 
          pluginPath, 
          reason: parsed.error.message 
        });
        continue;
      }

      plugins.push(parsed.data);
    } catch (error) {
      errors.push({ 
        pluginPath, 
        reason: error.message 
      });
    }
  }

  return { plugins, errors };
}
```

This protects your cloud backend from:
- ✅ Malformed plugins
- ✅ User errors
- ✅ Malicious packages

---

## 📤 Publishing Workflow

User runs:

```bash
bun publish
# or
npm publish
```

Because you output:

```
dist/index.js
```

Everything stays clean.

**Pre-publish Checklist:**

1. ✅ Run `bun run build`
2. ✅ Run `bun test`
3. ✅ Verify `dist/` contains all necessary files
4. ✅ Check `package.json` `files` field includes `dist`
5. ✅ Verify `peerDependencies` are correct

---

## 🛒 Marketplace Compatibility (Future)

Your plugin build system is **marketplace-ready**.

Later you can add:

### 🌐 Plugin Metadata Manifest

**File:** `arch-plugin.json`

```json
{
  "title": "PCI Compliance Rules",
  "author": "Your Org",
  "tags": ["pci", "security", "compliance"],
  "category": "security",
  "license": "MIT",
  "logo": "./logo.png",
  "homepage": "https://github.com/yourorg/pci-plugin",
  "repository": "https://github.com/yourorg/pci-plugin"
}
```

**Bundled via:**

```json
{
  "files": ["dist", "arch-plugin.json", "README.md"]
}
```

### 💵 Premium/Freemium Support

Plugins distributed from your cloud:
- ✅ Verified authors
- ✅ Paid plans
- ✅ Custom tokens
- ✅ Org-level rulesets

Your build system already supports this.

---

## 🛠️ Plugin Development DX (Optional Enhancements)

I recommend adding:

### Hot-Reload Dev Server

```json
{
  "scripts": {
    "dev": "bun build ./src/index.ts --outdir dist --target node --watch"
  }
}
```

Automatically rebuilds plugin on file change.

### TypeScript Type Declarations

```bash
npm i -D @arch/validation
```

Types are automatically available when using `@arch/validation` as a peer dependency.

### Plugin Testing Scaffold

Use Vitest:

**File:** `tests/ruleA.test.ts`

```typescript
import { describe, it, expect } from 'vitest';
import ruleA from "../src/rules/ruleA.js";
import { createMockContext } from '@arch/validation/testing';

describe('Rule A', () => {
  it('rejects missing PCI tag', () => {
    const ctx = createMockContext({ 
      model: { 
        nodes: [{ 
          id: "a", 
          metadata: { tags: ["credit-card"] } 
        }] 
      }
    });

    const issues = ruleA.apply(ctx);
    expect(issues.length).toBe(1);
    expect(issues[0].ruleId).toBe('pci/no-plain-card-data');
  });
});
```

**File:** `vitest.config.ts`

```typescript
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node'
  }
});
```

---

## 📋 `.gitignore` Template

```gitignore
# Dependencies
node_modules/

# Build output
dist/

# TypeScript
*.tsbuildinfo

# Testing
coverage/

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db
```

---

## 🎁 Full Plugin Starter Template

See [Plugin Template Repository](./plugin-template-repository.md) for a complete, copy-paste-ready template with:
- ✅ Full folder structure
- ✅ Complete source files
- ✅ Example rules
- ✅ Test files
- ✅ All configuration files
- ✅ README documentation
- ✅ Ready for GitHub and npm publishing

A complete plugin template includes:

```
plugin-template/
  src/
    index.ts              → Plugin entry
    rules/
      example-rule.ts      → Example rule
  tests/
    example-rule.test.ts   → Example test
  package.json             → With all scripts
  tsconfig.json            → TypeScript config
  bunfig.toml              → Bun build config
  arch-plugin.json         → Marketplace metadata
  README.md                → Plugin documentation
  .gitignore               → Git ignore rules
  LICENSE                  → License file
```

---

## 🚀 Build System Benefits

This build system is:

- ✅ **Bun-native** - Fastest builds
- ✅ **Works on Node & Bun** - Universal compatibility
- ✅ **Portable for CLI / Cloud / LSP** - Same code everywhere
- ✅ **Secure + deterministic** - Safe loading, no side effects
- ✅ **Marketplace-ready** - Future-proof architecture
- ✅ **Easy for OSS contributions** - Simple structure
- ✅ **Fastest possible DX** - Hot reload, fast builds
- ✅ **Zero configuration for plugin authors** - Just `bun run build`

---

## 📚 Integration with Validation Engine

The validation engine loads plugins using this build system:

**File:** `packages/validation/engine/plugins.ts`

```typescript
import { loadPlugins } from './plugins.js';
import { ValidationConfig } from '../types/config.js';

export async function loadUserPlugins(
  config?: ValidationConfig
): Promise<ValidationPlugin[]> {
  if (!config?.plugins || config.plugins.length === 0) {
    return [];
  }

  const { plugins, errors } = await loadPlugins(config.plugins);

  // Log errors but don't fail
  if (errors.length > 0) {
    console.warn('Some plugins failed to load:');
    for (const error of errors) {
      console.warn(`  - ${error.pluginPath}: ${error.reason}`);
    }
  }

  return plugins;
}
```

---

## 🔄 Version Pinning

Plugins support version pinning (like ESLint plugins or Terraform providers):

```json
{
  "validation": {
    "plugins": [
      "@arch/plugin-pci-security@1.0.0",
      "./validators/custom-plugin.js"
    ]
  }
}
```

The engine resolves versions using standard npm/Bun resolution.

---

## 🎯 Next Steps

1. **Create plugin template repository** - Starter template for plugin authors
2. **Add plugin marketplace architecture** - Business model + APIs
3. **Add plugin testing framework** - Vitest integration
4. **Add build performance optimizations** - Caching, parallel builds
5. **Add plugin validation CLI** - `sruja plugin validate` command

---

[← Back to Documentation Index](../README.md)
