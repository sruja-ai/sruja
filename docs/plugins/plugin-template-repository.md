# Architecture Plugin Template Repository v1

**Full, production-ready plugin template repository** for the Architecture Validation Engine.

This template includes:
- ✅ Full folder structure
- ✅ Full source files
- ✅ Boilerplate rules
- ✅ Types
- ✅ Build scripts
- ✅ Tests
- ✅ README
- ✅ Ready for `npm publish` or `bun publish`
- ✅ Ready to be consumed by `sruja validate`

You can literally drop this into GitHub as-is.

---

## 🗂️ Repository Structure

```
arch-plugin-template/
│
├── package.json
├── tsconfig.json
├── bunfig.toml
├── arch-plugin.json            # plugin metadata (future marketplace)
├── .gitignore
│
├── src/
│   ├── index.ts
│   └── rules/
│        ├── example-rule.ts
│        └── naming-convention.ts
│
├── dist/                       # build output (generated)
│
├── tests/
│   ├── example-rule.test.ts
│   └── naming-convention.test.ts
│
└── README.md
```

---

## 📄 `package.json`

```json
{
  "name": "@arch/plugin-template",
  "version": "0.1.0",
  "description": "A template plugin for the Architecture Validation Engine.",
  "license": "MIT",
  "type": "module",
  "main": "dist/index.js",
  "exports": {
    ".": "./dist/index.js"
  },
  "files": [
    "dist",
    "arch-plugin.json",
    "README.md"
  ],
  "scripts": {
    "build": "bun build ./src/index.ts --outdir dist --target node",
    "dev": "bun build ./src/index.ts --outdir dist --target node --watch",
    "test": "bun test",
    "prepublishOnly": "bun run build"
  },
  "peerDependencies": {
    "@arch/validation": ">=1.0.0"
  },
  "devDependencies": {
    "bun-types": "latest",
    "typescript": "^5.6.0",
    "vitest": "^2.0.0"
  },
  "keywords": [
    "architecture",
    "validation",
    "plugin",
    "template"
  ],
  "repository": {
    "type": "git",
    "url": "https://github.com/yourorg/arch-plugin-template.git"
  }
}
```

---

## 🧪 `bunfig.toml`

```toml
[build]
entrypoints = ["src/index.ts"]
outdir = "dist"
target = "node"
minify = true
sourcemap = "linked"
```

---

## 📘 `tsconfig.json`

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "allowJs": false,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "outDir": "dist",
    "declaration": true,
    "declarationMap": true
  },
  "include": ["src"],
  "exclude": ["node_modules", "dist", "tests"]
}
```

---

## 🧩 `arch-plugin.json`

*(Marketplace metadata — optional but future-ready)*

```json
{
  "title": "Architecture Plugin Template",
  "author": "Your Name",
  "description": "Starter template for writing plugins.",
  "tags": ["template", "validation"],
  "category": "general",
  "logo": "./logo.png",
  "version": "0.1.0",
  "homepage": "https://github.com/yourorg/arch-plugin-template",
  "repository": "https://github.com/yourorg/arch-plugin-template"
}
```

---

## 📝 `.gitignore`

```gitignore
# Dependencies
node_modules/

# Build output
dist/

# TypeScript
*.tsbuildinfo

# Testing
coverage/
.nyc_output/

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
npm-debug.log*
yarn-debug.log*
yarn-error.log*
```

---

## 🧠 `src/index.ts`

The plugin entrypoint.

```typescript
import type { ValidationPlugin } from "@arch/validation";

import exampleRule from "./rules/example-rule.js";
import namingConventionRule from "./rules/naming-convention.js";

const plugin: ValidationPlugin = {
  name: "plugin-template",
  version: "0.1.0",
  rules: [
    exampleRule,
    namingConventionRule
  ]
};

export default plugin;
```

---

## 🔍 `src/rules/example-rule.ts`

This is a simple rule that catches components with ID length < 3.

```typescript
import type { ValidationRule, ValidationContext } from "@arch/validation";

const rule: ValidationRule = {
  id: "template/id-too-short",
  description: "Component IDs must be at least 3 characters long.",
  severity: "warning",
  apply(ctx: ValidationContext) {
    const issues = [];

    for (const node of ctx.model.nodes ?? []) {
      if (node.id.length < 3) {
        issues.push({
          ruleId: "template/id-too-short",
          message: `Component ID '${node.id}' is too short (minimum 3 characters).`,
          severity: "warning",
          location: ctx.getLocation?.(node.id),
          metadata: { 
            nodeId: node.id,
            length: node.id.length,
            minimum: 3
          }
        });
      }
    }

    return issues;
  }
};

export default rule;
```

---

## 🔎 `src/rules/naming-convention.ts`

Detects invalid naming patterns.

```typescript
import type { ValidationRule, ValidationContext } from "@arch/validation";

const rule: ValidationRule = {
  id: "template/naming-convention",
  description: "Components must use kebab-case naming convention.",
  severity: "error",
  apply(ctx: ValidationContext) {
    const issues = [];
    const kebabCaseRegex = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

    for (const node of ctx.model.nodes ?? []) {
      if (!kebabCaseRegex.test(node.id)) {
        issues.push({
          ruleId: "template/naming-convention",
          message: `Component ID '${node.id}' is not kebab-case. Use lowercase letters, numbers, and hyphens only.`,
          severity: "error",
          location: ctx.getLocation?.(node.id),
          metadata: { 
            nodeId: node.id,
            expectedFormat: "kebab-case",
            example: "my-component-name"
          }
        });
      }
    }

    return issues;
  }
};

export default rule;
```

---

## 🧪 `tests/example-rule.test.ts`

```typescript
import { describe, it, expect } from "bun:test";
import exampleRule from "../src/rules/example-rule.ts";
import type { ValidationContext } from "@arch/validation";

function createMockContext(overrides: Partial<ValidationContext>): ValidationContext {
  return {
    dsl: "",
    ast: {} as any,
    model: { nodes: [] },
    project: {} as any,
    config: {},
    getLocation: () => undefined,
    ...overrides
  } as ValidationContext;
}

describe("Example Rule: ID Too Short", () => {
  it("flags short component IDs", () => {
    const ctx = createMockContext({
      model: {
        nodes: [
          { id: "a", type: "component" },
          { id: "ab", type: "component" },
          { id: "abc", type: "component" },
          { id: "abcd", type: "component" }
        ]
      }
    });

    const issues = exampleRule.apply(ctx);

    expect(issues.length).toBe(2); // "a" and "ab" are too short
    expect(issues[0].severity).toBe("warning");
    expect(issues[0].ruleId).toBe("template/id-too-short");
    expect(issues[0].message).toContain("'a'");
  });

  it("passes for valid IDs", () => {
    const ctx = createMockContext({
      model: {
        nodes: [
          { id: "abc", type: "component" },
          { id: "my-component", type: "component" }
        ]
      }
    });

    const issues = exampleRule.apply(ctx);
    expect(issues.length).toBe(0);
  });
});
```

---

## 🧪 `tests/naming-convention.test.ts`

```typescript
import { describe, it, expect } from "bun:test";
import namingConventionRule from "../src/rules/naming-convention.ts";
import type { ValidationContext } from "@arch/validation";

function createMockContext(overrides: Partial<ValidationContext>): ValidationContext {
  return {
    dsl: "",
    ast: {} as any,
    model: { nodes: [] },
    project: {} as any,
    config: {},
    getLocation: () => undefined,
    ...overrides
  } as ValidationContext;
}

describe("Naming Convention Rule", () => {
  it("identifies naming convention problems", () => {
    const ctx = createMockContext({
      model: {
        nodes: [
          { id: "valid-name", type: "component" },
          { id: "InvalidName", type: "component" },
          { id: "snake_case", type: "component" },
          { id: "camelCase", type: "component" }
        ]
      }
    });

    const issues = namingConventionRule.apply(ctx);

    expect(issues.length).toBe(3); // InvalidName, snake_case, camelCase
    expect(issues[0].severity).toBe("error");
    expect(issues[0].ruleId).toBe("template/naming-convention");
  });

  it("passes for valid kebab-case names", () => {
    const ctx = createMockContext({
      model: {
        nodes: [
          { id: "valid-name", type: "component" },
          { id: "another-valid-name", type: "component" },
          { id: "name-with-123", type: "component" }
        ]
      }
    });

    const issues = namingConventionRule.apply(ctx);
    expect(issues.length).toBe(0);
  });
});
```

---

## 📘 `README.md`

```markdown
# @arch/plugin-template

A starter template for building plugins for the Architecture Validation Engine.

## 📦 Install

\`\`\`sh
bun add @arch/plugin-template

# or

npm install @arch/plugin-template
\`\`\`

## 🧩 Usage (in .architecture/config.json)

\`\`\`json
{
  "validation": {
    "plugins": [
      "@arch/plugin-template"
    ]
  }
}
\`\`\`

## 🛠 Development

\`\`\`sh
bun run dev    # watch mode
bun run build  # builds to dist/
bun test       # run tests
\`\`\`

## 🧱 Creating Rules

Rules live in:

\`\`\`
src/rules/
\`\`\`

A rule looks like:

\`\`\`typescript
import type { ValidationRule, ValidationContext } from "@arch/validation";

const rule: ValidationRule = {
  id: "template/example",
  description: "Example rule description",
  severity: "error",
  apply(ctx: ValidationContext) {
    const issues = [];
    // Your validation logic here
    return issues;
  }
};

export default rule;
\`\`\`

## 🧪 Testing

Uses Bun's built-in test runner.

\`\`\`sh
bun test
\`\`\`

## 📚 Documentation

- [Plugin API Documentation](https://docs.arch.example.com/plugins)
- [Validation Engine API](https://docs.arch.example.com/validation)
- [Plugin Examples](https://docs.arch.example.com/plugins/examples)

## 📄 License

MIT

## 🤝 Contributing

Contributions welcome! Please read our contributing guidelines first.

## 🔗 Links

- [Architecture Validation Engine](https://github.com/yourorg/arch-validation)
- [Plugin Marketplace](https://plugins.arch.example.com)
```

---

## 🚀 Quick Start Guide

### 1. Create New Plugin from Template

```bash
# Clone or copy this template
git clone https://github.com/yourorg/arch-plugin-template.git my-plugin
cd my-plugin

# Update package.json with your plugin name
# Edit src/index.ts with your plugin name
# Add your rules to src/rules/
```

### 2. Develop Your Plugin

```bash
# Install dependencies
bun install

# Start development (watch mode)
bun run dev

# Run tests
bun test

# Build for production
bun run build
```

### 3. Test Locally

```bash
# In your architecture project
# Add to .architecture/config.json:
{
  "validation": {
    "plugins": [
      "../my-plugin/dist/index.js"
    ]
  }
}

# Run validation
sruja validate
```

### 4. Publish

```bash
# Build first
bun run build

# Publish to npm
npm publish

# Or publish to GitHub Packages
npm publish --registry=https://npm.pkg.github.com
```

---

## 🎯 Customization Checklist

When creating a new plugin from this template:

- [ ] Update `package.json`:
  - [ ] Change `name` to your plugin name
  - [ ] Update `description`
  - [ ] Update `repository` URL
  - [ ] Add relevant `keywords`

- [ ] Update `src/index.ts`:
  - [ ] Change plugin `name`
  - [ ] Update `version`
  - [ ] Import your rules

- [ ] Create your rules in `src/rules/`:
  - [ ] Remove example rules
  - [ ] Add your custom rules
  - [ ] Follow naming convention: `your-rule-name.ts`

- [ ] Update tests in `tests/`:
  - [ ] Remove example tests
  - [ ] Add tests for your rules

- [ ] Update `README.md`:
  - [ ] Change title and description
  - [ ] Add usage examples
  - [ ] Add documentation links

- [ ] Update `arch-plugin.json`:
  - [ ] Change title, author, description
  - [ ] Add relevant tags
  - [ ] Update category

---

## 📦 Template Features

This template includes:

- ✅ **TypeScript** - Full type safety
- ✅ **Bun** - Fast builds and tests
- ✅ **Vitest** - Optional test framework
- ✅ **Example Rules** - Two working examples
- ✅ **Tests** - Complete test examples
- ✅ **Build System** - Ready for production
- ✅ **Marketplace Ready** - Includes metadata
- ✅ **Documentation** - Complete README

---

## 🔗 Integration with Validation Engine

This template is designed to work seamlessly with:

- ✅ **CLI** (`sruja validate`)
- ✅ **LSP** (Language Server Protocol)
- ✅ **Cloud Backend** (API validation)
- ✅ **Editor UI** (Live validation)

---

## 🎁 Next Steps

1. **Customize the template** - Replace example rules with your own
2. **Add more rules** - Create additional validation rules
3. **Write tests** - Ensure your rules work correctly
4. **Document your plugin** - Update README with usage examples
5. **Publish** - Share with the community or use internally

---

## 📚 Related Documentation

- [Plugin Build System](./plugin-build-system.md) - Build and bundling guide
- [Validation Plugin Example](./validation-plugin-example.md) - Production plugin example
- [Validation Engine API](./validation-engine.md) - Complete API reference

---

[← Back to Documentation Index](../README.md)
