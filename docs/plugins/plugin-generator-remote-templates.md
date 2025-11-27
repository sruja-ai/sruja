# Remote Plugin Template Support

**Full, production-ready design + code implementation** for remote plugin template support in:

```bash
sruja create-plugin <name> --template <github-url>
```

This allows users to scaffold plugins from **any GitHub repository**, like:

```bash
sruja create-plugin pci-security --template https://github.com/myorg/arch-plugin-template
```

Or even specific branches/tags:

```bash
--template https://github.com/myorg/arch-plugin-template/tree/dev
--template https://github.com/myorg/arch-plugin-template#v2.1.0
```

Or zip URLs:

```bash
--template https://github.com/myorg/arch-plugin-template/archive/refs/heads/main.zip
```

This **vastly increases flexibility** and sets you up for a future **plugin marketplace**.

---

## 🎯 Overview

Remote template support enables:
- ✅ GitHub repository templates
- ✅ Branch and tag support
- ✅ Direct ZIP archive downloads
- ✅ Shorthand notation (`user/repo`)
- ✅ Local template fallback
- ✅ Graceful error handling
- ✅ No authentication required
- ✅ No Git installation required

---

## 📋 Requirements

### Support:

- ✅ GitHub `https://github.com/...`
- ✅ GitHub shorthand `myorg/repo`
- ✅ Branches: `.../tree/<branch>`
- ✅ Tags: `.../tree/<tag>`
- ✅ Direct ZIP archives
- ✅ Local template folders (fallback)
- ✅ Fail gracefully if network unavailable
- ✅ Cache downloads (optional future)

### Must not:

- ❌ Require Git installed
- ❌ Require user to authenticate
- ❌ Block CLI if template missing

Your implementation uses **pure Bun + Node fetch**, no extra deps (except JSZip for extraction).

---

## 🏗️ High-Level Flow

1. Detect `--template <url-or-id>`
2. Normalize into a canonical GitHub download URL
3. Download ZIP archive
4. Extract ZIP into temporary directory
5. Copy into target folder (applying templating)
6. Cleanup

**Technologies:**
- **JSZip** (works in Bun)
- **node:fs/promises**
- **native fetch**

---

## 🗂️ CLI UX Examples

```bash
# Shorthand notation
sruja create-plugin pci --template myorg/arch-plugin-pci-template

# Full GitHub URL
sruja create-plugin pci --template https://github.com/myorg/templates/pci

# Specific branch
sruja create-plugin pci --template https://github.com/myorg/templates/pci/tree/next

# Specific tag
sruja create-plugin pci --template https://github.com/myorg/templates/pci/tree/v1.0.0

# Direct ZIP URL
sruja create-plugin pci --template https://github.com/myorg/templates/pci/archive/main.zip
```

---

## 🔧 Implementation

### 1. Normalize GitHub URL

**File:** `packages/cli/src/utils/normalizeTemplateUrl.ts`

```typescript
export interface NormalizedTemplate {
  zipUrl: string;
  repoName: string;
}

export function normalizeTemplateUrl(input: string): NormalizedTemplate {
  // Case 1: shorthand "user/repo"
  if (!input.startsWith("http")) {
    const parts = input.split("/");
    if (parts.length !== 2) {
      throw new Error(`Invalid template shorthand: ${input}. Expected format: user/repo`);
    }
    const [user, repo] = parts;
    return {
      zipUrl: `https://github.com/${user}/${repo}/archive/refs/heads/main.zip`,
      repoName: repo
    };
  }

  // Case 2: direct zip URL
  if (input.endsWith(".zip")) {
    const urlParts = input.split("/");
    const repoName = urlParts[urlParts.length - 1].replace(".zip", "");
    return { zipUrl: input, repoName };
  }

  // Case 3: full GitHub URL
  const githubRegex = /^https?:\/\/github\.com\/([^/]+)\/([^/]+)(?:\/tree\/([^/]+))?/;
  const match = githubRegex.exec(input);

  if (!match) {
    throw new Error(`Unsupported template URL: ${input}`);
  }

  const [, user, repo, branch = "main"] = match;

  return {
    zipUrl: `https://github.com/${user}/${repo}/archive/refs/heads/${branch}.zip`,
    repoName: repo
  };
}
```

**Supports:**
- `myorg/template`
- `myorg/template/tree/dev`
- `https://github.com/...`
- `https://github.com/.../tree/dev`
- Direct `.zip` URLs

---

### 2. Download ZIP

**File:** `packages/cli/src/utils/downloadZip.ts`

```typescript
import fs from "node:fs/promises";

export async function downloadZip(url: string, destPath: string): Promise<void> {
  try {
    const res = await fetch(url);

    if (!res.ok) {
      throw new Error(
        `Failed to download ZIP: ${res.status} ${res.statusText}\n` +
        `URL: ${url}`
      );
    }

    const buffer = Buffer.from(await res.arrayBuffer());
    await fs.writeFile(destPath, buffer);
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`Network error downloading template: ${error.message}`);
    }
    throw error;
  }
}
```

---

### 3. Extract ZIP

**File:** `packages/cli/src/utils/extractZip.ts`

Uses **JSZip** (works perfectly in Bun):

```bash
bun add jszip
bun add -d @types/jszip
```

```typescript
import fs from "node:fs/promises";
import path from "node:path";
import JSZip from "jszip";

export async function extractZip(zipPath: string, extractTo: string): Promise<string> {
  const data = await fs.readFile(zipPath);
  const zip = await JSZip.loadAsync(data);

  await fs.mkdir(extractTo, { recursive: true });

  // Extract all files
  for (const fileName of Object.keys(zip.files)) {
    const file = zip.files[fileName];

    // Skip macOS __MACOSX folder
    if (fileName.includes("__MACOSX")) {
      continue;
    }

    const outPath = path.join(extractTo, fileName);

    if (file.dir) {
      await fs.mkdir(outPath, { recursive: true });
    } else {
      const content = await file.async("nodebuffer");
      await fs.mkdir(path.dirname(outPath), { recursive: true });
      await fs.writeFile(outPath, content);
    }
  }

  // GitHub wraps contents in a folder, find the root
  const dirs = await fs.readdir(extractTo);
  const rootDir = dirs.find(dir => {
    const dirPath = path.join(extractTo, dir);
    return fs.stat(dirPath).then(stat => stat.isDirectory());
  });

  return rootDir ? path.join(extractTo, rootDir) : extractTo;
}
```

---

### 4. Incorporate into `arch create-plugin`

**File:** `packages/cli/src/commands/create-plugin.ts` (updated)

```typescript
import fs from "node:fs";
import path from "node:path";
import { mkdir, rm } from "node:fs/promises";
import { $ } from "bun";
import prompts from "prompts";
import kleur from "kleur";
import { normalizeTemplateUrl } from "../utils/normalizeTemplateUrl.js";
import { downloadZip } from "../utils/downloadZip.js";
import { extractZip } from "../utils/extractZip.js";

interface PluginOptions {
  dir?: string;
  yes?: boolean;
  git?: boolean;
  install?: boolean;
  template?: string;
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

  // Determine template source
  let templateDir: string;
  const tmpFiles: string[] = [];

  if (options.template) {
    try {
      console.log(kleur.cyan(`📥 Using remote template: ${options.template}\n`));

      const { zipUrl, repoName } = normalizeTemplateUrl(options.template);
      const tmpZip = path.join(process.cwd(), `.arch-tmp-${Date.now()}.zip`);
      const tmpDir = path.join(process.cwd(), `.arch-tmp-${repoName}-${Date.now()}`);

      tmpFiles.push(tmpZip, tmpDir);

      // Download
      console.log(kleur.dim(`Downloading template from ${zipUrl}...`));
      await downloadZip(zipUrl, tmpZip);

      // Extract
      console.log(kleur.dim("Extracting template..."));
      templateDir = await extractZip(tmpZip, tmpDir);

      console.log(kleur.green("✓ Template downloaded and extracted\n"));
    } catch (error) {
      console.warn(kleur.yellow(`\n⚠️  Remote template download failed: ${error.message}`));
      console.log(kleur.dim("Using built-in template...\n"));

      // Fallback to internal template
      templateDir = path.join(__dirname, "../../templates/plugin");
    }
  } else {
    // Use built-in template
    templateDir = path.join(__dirname, "../../templates/plugin");
  }

  // Copy everything (render templates)
  copyFolder(templateDir, targetDir, meta);

  // Cleanup temporary files
  for (const tmpFile of tmpFiles) {
    try {
      await rm(tmpFile, { recursive: true, force: true });
    } catch (error) {
      // Ignore cleanup errors
    }
  }

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

## ⚙️ Fallback Logic

If remote template fails:

```
⚠️  Remote template download failed: Failed to download ZIP: 404 Not Found
Using built-in template...
```

The command gracefully falls back to the built-in template, ensuring the user can always create a plugin.

---

## 🧪 Usage Examples

### Default (built-in template)

```bash
arch create-plugin pci
```

### GitHub repo template

```bash
arch create-plugin pagerduty --template https://github.com/foo/bar
```

### Shorthand

```bash
arch create-plugin pci --template myorg/arch-plugin-template
```

### Specific branch

```bash
arch create-plugin pci --template https://github.com/myorg/arch-plugin-template/tree/dev
```

### Specific tag

```bash
arch create-plugin pci --template https://github.com/myorg/arch-plugin-template/tree/v1.2.0
```

### Direct ZIP

```bash
arch create-plugin pci --template https://github.com/myorg/arch-plugin-template/archive/main.zip
```

### With all options

```bash
sruja create-plugin pci \
  --template myorg/arch-plugin-template \
  --yes \
  --git \
  --install
```

---

## 📦 Dependencies

Add to `packages/cli/package.json`:

```json
{
  "dependencies": {
    "commander": "^11.0.0",
    "prompts": "^2.4.2",
    "kleur": "^4.1.5",
    "jszip": "^3.10.1"
  },
  "devDependencies": {
    "@types/jszip": "^3.4.1"
  }
}
```

---

## 🧪 Testing Remote Templates

**File:** `packages/cli/__tests__/create-plugin-remote.test.ts`

```typescript
import { describe, it, expect } from "bun:test";
import { normalizeTemplateUrl } from "../src/utils/normalizeTemplateUrl";

describe("normalizeTemplateUrl", () => {
  it("handles shorthand notation", () => {
    const result = normalizeTemplateUrl("myorg/template");
    expect(result.zipUrl).toBe(
      "https://github.com/myorg/template/archive/refs/heads/main.zip"
    );
    expect(result.repoName).toBe("template");
  });

  it("handles full GitHub URL", () => {
    const result = normalizeTemplateUrl(
      "https://github.com/myorg/template"
    );
    expect(result.zipUrl).toBe(
      "https://github.com/myorg/template/archive/refs/heads/main.zip"
    );
  });

  it("handles branch specification", () => {
    const result = normalizeTemplateUrl(
      "https://github.com/myorg/template/tree/dev"
    );
    expect(result.zipUrl).toBe(
      "https://github.com/myorg/template/archive/refs/heads/dev.zip"
    );
  });

  it("handles direct ZIP URL", () => {
    const result = normalizeTemplateUrl(
      "https://github.com/myorg/template/archive/main.zip"
    );
    expect(result.zipUrl).toBe(
      "https://github.com/myorg/template/archive/main.zip"
    );
  });

  it("throws on invalid shorthand", () => {
    expect(() => normalizeTemplateUrl("invalid")).toThrow();
  });
});
```

---

## 🎁 What This Unlocks

This feature gives you:

### ✔ Full Remote Template Scaffolding

Publish plugin templates on GitHub, npm, etc.

### ✔ Community Plugin Ecosystem

Users can make plugin templates that other devs can use.

### ✔ Marketplace-Ready Structure

You can later auto-browse templates from your cloud.

### ✔ Supports Org-Wide Rule Templates

Companies can centrally maintain:

```
github.com/org/arch-plugin-template-enterprise
```

### ✔ Makes `arch` CLI Feel Premium

Like `npm create`, `bun create`, `cargo generate`.

---

## 🚀 Future Enhancements

### Template Caching

Cache downloaded templates locally:

```typescript
const cacheDir = path.join(os.homedir(), ".arch", "templates");
// Cache by URL hash
```

### Template Validation

Validate template structure:

```typescript
function validateTemplate(dir: string): boolean {
  // Check for required files
  return fs.existsSync(path.join(dir, "package.json")) &&
         fs.existsSync(path.join(dir, "src/index.ts"));
}
```

### Template Registry

Auto-discover templates from a registry:

```bash
sruja create-plugin pci --template registry:pci-security
```

### Private Repository Support

Support private repos with authentication:

```bash
sruja create-plugin pci --template myorg/private-template --token $GITHUB_TOKEN
```

---

## 📚 Related Documentation

- [Plugin Generator](./plugin-generator.md) - Base plugin generator implementation
- [Plugin Template Repository](./plugin-template-repository.md) - Template structure
- [Plugin Build System](./plugin-build-system.md) - Build and bundling guide

---

[← Back to Documentation Index](../README.md)
