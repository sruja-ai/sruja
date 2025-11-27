# Architecture Plugin Generator (`sruja create-plugin`)

**Full design + implementation plan + code scaffold** for `sruja create-plugin` — Architecture Plugin Generator.

This will let users generate new validation plugins instantly:

```bash
sruja create-plugin pci-security
```

Creates a complete plugin structure ready for development.

This is a **major DX win**, like:
- `npm create vite@latest`
- `bun create react-app`
- `eslint --init`
- `yo generator`

---

## 🎯 Overview

The plugin generator creates a complete plugin structure with:
- ✅ All configuration files (`package.json`, `tsconfig.json`, `bunfig.toml`)
- ✅ Source files (`src/index.ts`, example rules)
- ✅ Test files
- ✅ README documentation
- ✅ Ready to build and publish

---

## 📋 CLI Command Specification

### Command

```bash
sruja create-plugin <name>
```

### Flags

| Flag                      | Purpose                           |
| ------------------------- | --------------------------------- |
| `--dir <path>`            | Generate in custom directory      |
| `--yes`                   | Skip all prompts                  |
| `--template <github-url>` | Use remote template (GitHub repo, branch, tag, or ZIP) |
| `--git`                   | Auto init git repo                |
| `--install`               | Auto install deps                 |

See [Remote Plugin Template Support](./plugin-generator-remote-templates.md) for complete remote template documentation.

### Prompts (if no `--yes`)

```
Plugin name: pci-security
Description: PCI DSS compliance rules
Author: Dilip Kola
Version: 0.1.0
Add example rule? (Y/n)
Add tests? (Y/n)
Initialize Git repo? (Y/n)
```

---

## 🏗️ Implementation Location

Inside your monorepo:

```
/packages/cli/src/commands/create-plugin.ts
```

**Entry in CLI index:**

```typescript
import createPlugin from "./commands/create-plugin";

program
  .command("create-plugin")
  .argument("<name>", "Plugin name")
  .option("--dir <path>", "Output directory")
  .option("--yes", "Skip prompts")
  .option("--git", "Initialize git repo")
  .option("--install", "Install dependencies")
  .action((name, options) => createPlugin(name, options));
```

---

## 🧩 Implementation Plan

### PHASE 1 — Scaffolding System

Implement:

#### 1. File Copier Utility

```typescript
copyTemplateFiles(fromDir, toDir, vars)
```

#### 2. Templating Engine (Simple)

Replace mustache-like tags:

```
{{name}}
{{description}}
{{author}}
```

#### 3. Directory Generator

```typescript
mkdirp(pluginDir)
writeFiles()
```

---

### PHASE 2 — CLI Logic

Steps executed by command:

1. Parse inputs
2. Resolve plugin directory
3. Prompt for metadata (unless `--yes`)
4. Load template files
5. Write them into destination
6. Run post-install steps (optional)
7. Print success banner

---

### PHASE 3 — Templates

Create internal template folder:

```
packages/cli/templates/plugin/
```

Contents (raw templates):

```
package.json.hbs
tsconfig.json.hbs
bunfig.toml.hbs
arch-plugin.json.hbs
src/index.ts.hbs
src/rules/example.ts.hbs
tests/example.test.ts.hbs
README.md.hbs
.gitignore.hbs
```

---

## 📄 Template Files

Below are the actual template files with placeholders.

### `package.json.hbs`

```json
{
  "name": "@arch/plugin-{{name}}",
  "version": "{{version}}",
  "description": "{{description}}",
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
    "{{name}}"
  ]
}
```

### `tsconfig.json.hbs`

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

### `bunfig.toml.hbs`

```toml
[build]
entrypoints = ["src/index.ts"]
outdir = "dist"
target = "node"
minify = true
sourcemap = "linked"
```

### `arch-plugin.json.hbs`

```json
{
  "title": "{{name}} Plugin",
  "author": "{{author}}",
  "description": "{{description}}",
  "tags": ["validation"],
  "category": "custom",
  "version": "{{version}}"
}
```

### `src/index.ts.hbs`

```typescript
import type { ValidationPlugin } from "@arch/validation";

import exampleRule from "./rules/example.js";

const plugin: ValidationPlugin = {
  name: "{{name}}",
  version: "{{version}}",
  rules: [
    exampleRule
  ]
};

export default plugin;
```

### `src/rules/example.ts.hbs`

```typescript
import type { ValidationRule, ValidationContext } from "@arch/validation";

const rule: ValidationRule = {
  id: "{{name}}/example-rule",
  description: "Example rule generated by sruja create-plugin.",
  severity: "warning",
  apply(ctx: ValidationContext) {
    const issues = [];

    // Simple demo: warn for components with tag "example"
    for (const node of ctx.model.nodes ?? []) {
      if (node.metadata?.tags?.includes("example")) {
        issues.push({
          ruleId: "{{name}}/example-rule",
          message: `Component '${node.id}' uses tag "example".`,
          severity: "warning",
          location: ctx.getLocation?.(node.id),
          metadata: { id: node.id }
        });
      }
    }

    return issues;
  }
};

export default rule;
```

### `tests/example.test.ts.hbs`

```typescript
import { describe, it, expect } from "bun:test";
import plugin from "../src/index.ts";

describe("{{name}} Plugin", () => {
  it("plugin loads rules", () => {
    expect(plugin.rules.length).toBeGreaterThan(0);
  });

  it("plugin has correct name", () => {
    expect(plugin.name).toBe("{{name}}");
    expect(plugin.version).toBe("{{version}}");
  });
});
```

### `README.md.hbs`

```markdown
# @arch/plugin-{{name}}

{{description}}

## 📦 Install

\`\`\`sh
bun add @arch/plugin-{{name}}

# or

npm install @arch/plugin-{{name}}
\`\`\`

## 🧩 Usage

Add to `.architecture/config.json`:

\`\`\`json
{
  "validation": {
    "plugins": [
      "@arch/plugin-{{name}}"
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
  id: "{{name}}/example",
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
```

### `.gitignore.hbs`

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

## 🧠 Generator Code

**File:** `packages/cli/src/commands/create-plugin.ts`

```typescript
import fs from "node:fs";
import path from "node:path";
import { mkdir } from "node:fs/promises";
import { $ } from "bun";
import prompts from "prompts";
import kleur from "kleur";

interface PluginOptions {
  dir?: string;
  yes?: boolean;
  git?: boolean;
  install?: boolean;
}

interface PluginMetadata {
  name: string;
  description: string;
  author: string;
  version: string;
}

export default async function createPlugin(
  name: string,
  options: PluginOptions = {}
): Promise<void> {
  const targetDir = path.resolve(options.dir ?? name);

  console.log(kleur.cyan(`\n🧩 Creating plugin: ${name}\n`));

  // Check if directory exists
  if (fs.existsSync(targetDir)) {
    console.error(kleur.red(`Error: Directory ${targetDir} already exists`));
    process.exit(1);
  }

  const defaults: PluginMetadata = {
    name,
    description: `${name} validation plugin`,
    author: "",
    version: "0.1.0"
  };

  let meta: PluginMetadata = defaults;

  if (!options.yes) {
    const response = await prompts([
      {
        type: "text",
        name: "description",
        message: "Description:",
        initial: defaults.description
      },
      {
        type: "text",
        name: "author",
        message: "Author:"
      },
      {
        type: "text",
        name: "version",
        message: "Version:",
        initial: "0.1.0"
      }
    ]);

    meta = {
      name,
      description: response.description || defaults.description,
      author: response.author || defaults.author,
      version: response.version || defaults.version
    };
  }

  // Create target directory
  await mkdir(targetDir, { recursive: true });

  // Copy template files
  const templateRoot = path.join(__dirname, "../../templates/plugin");
  copyFolder(templateRoot, targetDir, meta);

  console.log(kleur.green(`\n✅ Plugin created at ${targetDir}\n`));

  // Post-install steps
  if (options.git) {
    console.log(kleur.cyan("Initializing git repository..."));
    await $`cd ${targetDir} && git init`.quiet();
  }

  if (options.install) {
    console.log(kleur.cyan("Installing dependencies..."));
    await $`cd ${targetDir} && bun install`.quiet();
  }

  // Print next steps
  console.log(kleur.bold("\nNext steps:\n"));
  console.log(`  ${kleur.cyan(`cd ${targetDir}`)}`);
  if (!options.install) {
    console.log(`  ${kleur.cyan("bun install")}`);
  }
  console.log(`  ${kleur.cyan("bun run dev")}\n`);
}

function copyFolder(
  src: string,
  dest: string,
  vars: PluginMetadata
): void {
  if (!fs.existsSync(src)) {
    throw new Error(`Template directory not found: ${src}`);
  }

  for (const file of fs.readdirSync(src)) {
    const srcPath = path.join(src, file);
    const destPath = path.join(dest, file.replace(".hbs", ""));

    if (fs.statSync(srcPath).isDirectory()) {
      fs.mkdirSync(destPath, { recursive: true });
      copyFolder(srcPath, destPath, vars);
    } else {
      const content = fs.readFileSync(srcPath, "utf8");
      const rendered = renderTemplate(content, vars);
      fs.writeFileSync(destPath, rendered);
    }
  }
}

function renderTemplate(str: string, vars: PluginMetadata): string {
  return str.replace(/\{\{(\w+)\}\}/g, (_, key) => {
    return vars[key as keyof PluginMetadata] ?? "";
  });
}
```

---

## 🎯 Example Run

### Interactive Mode

```bash
sruja create-plugin pci-security
```

**Output:**

```
🧩 Creating plugin: pci-security

Description: PCI DSS compliance rules
Author: Dilip Kola
Version: 0.1.0

✅ Plugin created at /path/to/pci-security

Next steps:

  cd /path/to/pci-security
  bun install
  bun run dev
```

### Non-Interactive Mode

```bash
sruja create-plugin pci-security --yes --git --install
```

**Output:**

```
🧩 Creating plugin: pci-security

✅ Plugin created at /path/to/pci-security
Initializing git repository...
Installing dependencies...

Next steps:

  cd /path/to/pci-security
  bun run dev
```

---

## 🚀 Integration with CLI

**File:** `packages/cli/src/index.ts`

```typescript
import { program } from "commander";
import createPlugin from "./commands/create-plugin";

program
  .name("arch")
  .description("Architecture Validation CLI")
  .version("1.0.0");

program
  .command("create-plugin")
  .description("Create a new validation plugin")
  .argument("<name>", "Plugin name (e.g., pci-security)")
  .option("--dir <path>", "Output directory (default: plugin name)")
  .option("--yes", "Skip prompts and use defaults")
  .option("--git", "Initialize git repository")
  .option("--install", "Install dependencies after creation")
  .action((name: string, options) => {
    createPlugin(name, options).catch((error) => {
      console.error("Error creating plugin:", error.message);
      process.exit(1);
    });
  });

program.parse();
```

---

## 📦 Dependencies

Add to `packages/cli/package.json`:

```json
{
  "dependencies": {
    "commander": "^11.0.0",
    "prompts": "^2.4.2",
    "kleur": "^4.1.5"
  }
}
```

---

## 🧪 Testing the Generator

**File:** `packages/cli/__tests__/create-plugin.test.ts`

```typescript
import { describe, it, expect, beforeAll, afterAll } from "bun:test";
import { mkdir, rm } from "node:fs/promises";
import { existsSync } from "node:fs";
import path from "node:path";
import createPlugin from "../src/commands/create-plugin";

describe("create-plugin", () => {
  const testDir = path.join(process.cwd(), "test-plugin");

  beforeAll(async () => {
    if (existsSync(testDir)) {
      await rm(testDir, { recursive: true });
    }
  });

  afterAll(async () => {
    if (existsSync(testDir)) {
      await rm(testDir, { recursive: true });
    }
  });

  it("creates plugin structure", async () => {
    await createPlugin("test-plugin", {
      yes: true,
      dir: testDir
    });

    expect(existsSync(testDir)).toBe(true);
    expect(existsSync(path.join(testDir, "package.json"))).toBe(true);
    expect(existsSync(path.join(testDir, "src/index.ts"))).toBe(true);
    expect(existsSync(path.join(testDir, "src/rules/example.ts"))).toBe(true);
    expect(existsSync(path.join(testDir, "tests/example.test.ts"))).toBe(true);
  });

  it("renders template variables", async () => {
    const packageJson = JSON.parse(
      await Bun.file(path.join(testDir, "package.json")).text()
    );

    expect(packageJson.name).toBe("@arch/plugin-test-plugin");
    expect(packageJson.version).toBe("0.1.0");
  });
});
```

---

## 🎁 Future Enhancements

### Remote Templates

✅ **Implemented!** See [Remote Plugin Template Support](./plugin-generator-remote-templates.md) for complete implementation.

Support GitHub templates:

```bash
sruja create-plugin my-plugin --template github:user/template
sruja create-plugin my-plugin --template user/repo
sruja create-plugin my-plugin --template https://github.com/user/repo/tree/branch
```

### Custom Templates

Allow users to specify custom template directories:

```bash
sruja create-plugin my-plugin --template ./my-templates
```

### Plugin Marketplace Integration

Auto-register created plugins:

```bash
sruja create-plugin my-plugin --publish
```

---

## 🚀 Your `arch` CLI Now Creates Plugins Instantly

This is your **Vite-style DX moment** for validation plugins.

### Benefits:

- ✅ **Instant plugin creation** - No manual setup
- ✅ **Consistent structure** - All plugins follow same pattern
- ✅ **Best practices** - Templates include best practices
- ✅ **Type-safe** - Full TypeScript support from start
- ✅ **Test-ready** - Includes test examples
- ✅ **Documentation-ready** - Includes README template

---

## 📚 Related Documentation

- [Plugin Template Repository](./plugin-template-repository.md) - Complete template structure
- [Plugin Build System](./plugin-build-system.md) - Build and bundling guide
- [Validation Plugin Example](./validation-plugin-example.md) - Production plugin example

---

[← Back to Documentation Index](../README.md)
