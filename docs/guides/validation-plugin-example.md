# Architecture Validation Plugin Example

**Production-quality plugin example** that fully implements the official **Architecture Validation Plugin API**.

This includes:
- Plugin structure
- Rule implementation
- Integration with config
- Plugin packaging
- Error handling
- Best practices
- Folder conventions
- Example project usage

You can drop this into your monorepo and it will work.

---

## 🔌 Plugin Overview

**Plugin Name:** `pci-security-plugin`

**Purpose:** Enforce PCI compliance for components handling credit card data.

---

## 📁 Plugin Folder Structure

Recommended structure:

```
plugins/
  pci-security/
    package.json
    index.ts
    rules/
      no-plain-card-data.ts
      gateway-must-be-encrypted.ts
    README.md
    tsconfig.json
```

---

## 📦 `package.json`

```json
{
  "name": "@arch/plugin-pci-security",
  "version": "1.0.0",
  "type": "module",
  "main": "./index.js",
  "exports": {
    ".": "./index.js"
  },
  "dependencies": {
    "@arch/validation": "workspace:*"
  },
  "devDependencies": {
    "typescript": "^5.6.0",
    "@types/node": "^20.0.0"
  }
}
```

---

## 🔧 Plugin Entry (index.ts)

```typescript
import type { ValidationPlugin } from "@arch/validation";
import noPlainCardData from "./rules/no-plain-card-data.js";
import gatewayMustBeEncrypted from "./rules/gateway-must-be-encrypted.js";

const plugin: ValidationPlugin = {
  name: "pci-security",
  version: "1.0.0",
  rules: [
    noPlainCardData,
    gatewayMustBeEncrypted
  ]
};

export default plugin;
```

---

## 📋 Rule 1 — `"pci/no-plain-card-data"`

This rule checks that **components tagged with `credit-card` MUST also include `PCI`** tag.

**File:** `rules/no-plain-card-data.ts`

```typescript
import type { ValidationRule, ValidationContext } from "@arch/validation";

const rule: ValidationRule = {
  id: "pci/no-plain-card-data",
  description: "Components handling card data must be PCI certified.",
  severity: "error",
  apply(ctx: ValidationContext) {
    const issues = [];

    for (const node of ctx.model.nodes) {
      // Check if node handles credit card data
      const handlesCardData = node.metadata?.tags?.includes("credit-card");
      const isPciCertified = node.metadata?.tags?.includes("PCI");

      if (handlesCardData && !isPciCertified) {
        issues.push({
          ruleId: "pci/no-plain-card-data",
          message: `Component '${node.id}' handles card data but is not tagged "PCI".`,
          severity: "error",
          location: ctx.getLocation?.(node.id), // Optional: get source location
          metadata: { 
            nodeId: node.id,
            nodeType: node.type
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

## 📋 Rule 2 — `"pci/gateway-must-be-encrypted"`

Ensures that **any edge connecting a user → payment API has encryption metadata**.

**File:** `rules/gateway-must-be-encrypted.ts`

```typescript
import type { ValidationRule, ValidationContext } from "@arch/validation";

const rule: ValidationRule = {
  id: "pci/gateway-must-be-encrypted",
  description: "Payment gateway connections must specify encryption metadata.",
  severity: "warning",
  apply(ctx: ValidationContext) {
    const issues = [];

    for (const edge of ctx.model.edges) {
      const isPaymentFlow = edge.metadata?.tags?.includes("payment");
      
      if (isPaymentFlow && !edge.metadata?.encryption) {
        issues.push({
          ruleId: "pci/gateway-must-be-encrypted",
          message: `Payment flow '${edge.from} → ${edge.to}' must specify encryption metadata.`,
          severity: "warning",
          location: ctx.getLocation?.(edge.id), // Optional: get source location
          metadata: {
            from: edge.from,
            to: edge.to,
            required: "metadata.encryption"
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

## ⚙️ Plugin API Usage in a Project

Users enable plugins in:

### `.architecture/config.json`

```json
{
  "validation": {
    "strictLayers": true,
    "plugins": [
      "./validators/pci-security-plugin/index.js"
    ]
  }
}
```

Or use absolute paths or npm packages:

```json
{
  "validation": {
    "plugins": [
      "@arch/plugin-pci-security",
      "/absolute/path/to/plugin/index.js"
    ]
  }
}
```

---

## 🔗 CLI Integration

When the CLI calls:

```bash
arch validate
```

It loads the plugin automatically:

```typescript
// In packages/cli/commands/validate.ts
import { validateArchitecture } from "@arch/validation";
import { loadProject } from "../utils/project";

export async function validateCommand(options: ValidateOptions) {
  const project = loadProject(options.projectPath);
  const config = project.config.validation;
  
  // Load plugins from config
  const plugins = await loadPlugins(config.plugins || []);
  
  const result = validateArchitecture({
    dsl: project.dsl,
    ast: project.ast,
    model: project.model,
    project,
    config: {
      ...config,
      plugins
    }
  });

  // Output results...
}
```

**File:** `packages/validation/engine/plugins.ts` (plugin loader)

```typescript
import { ValidationPlugin } from "../types/plugins";
import { z } from "zod";

const PluginSchema = z.object({
  name: z.string(),
  version: z.string(),
  rules: z.array(z.any()) // ValidationRule schema
});

export async function loadPlugins(
  pluginPaths: string[]
): Promise<ValidationPlugin[]> {
  const plugins: ValidationPlugin[] = [];

  for (const path of pluginPaths) {
    try {
      // Load plugin module
      const module = await import(path);
      const plugin = module.default || module;

      // Validate plugin structure
      const validated = PluginSchema.parse(plugin);
      plugins.push(validated);
    } catch (error) {
      console.warn(`Failed to load plugin from ${path}: ${error.message}`);
      // Continue loading other plugins
    }
  }

  return plugins;
}
```

---

## 📝 Example DSL That Triggers Plugin Errors

```dsl
system "Checkout" {
  container web: Frontend "Web App" {
    component cart: Component "Cart"
    component paymentApi: Component "Payment API" {
      tags: ["credit-card"]  // missing PCI tag → error
    }
  }
}

cart -> paymentApi: "send payment" {
  tags: ["payment"]         // but missing metadata.encryption → warning
}
```

**Output:**

```
✖ Component 'paymentApi' handles card data but is not tagged "PCI".
⚠ Payment flow 'cart → paymentApi' must specify encryption metadata.

Summary: 1 errors, 1 warnings
```

---

## 📦 Packaging & Publishing

See [Plugin Build System](./plugin-build-system.md) for complete build and publishing guide.

### Quick Build

```bash
# Using Bun (recommended)
bun run build

# Development with watch
bun run dev
```

### Publish to npm

```bash
npm publish
```

### Use in Project

```json
{
  "validation": {
    "plugins": [
      "@arch/plugin-pci-security"
    ]
  }
}
```

### Build System Features

- ✅ Bun-native builds (fastest)
- ✅ TypeScript support
- ✅ Tree-shaking
- ✅ Source maps
- ✅ Safe ESM output
- ✅ Works on Node & Bun

---

## 🚀 Future Plugin Capabilities

You can evolve the plugin ecosystem with:

### ✔ Cloud Security Plugins

- AWS Well-Architected Framework rules
- CIS Benchmarks compliance
- Zero Trust Models
- SOC 2 requirements

### ✔ Domain Plugins

- Banking/FinTech (PCI-DSS, GLBA)
- Healthcare (HIPAA, HITECH)
- Government (FedRAMP, FISMA)
- GDPR/Privacy regulations

### ✔ Organization-Level Rules

- Naming conventions
- Folder structure enforcement
- Required metadata for auditing
- ADR compliance rules
- Architecture pattern enforcement

### ✔ Community Plugin Marketplace

(Inspired by ESLint, Prettier, Terraform providers)

- Plugin registry
- Plugin ratings
- Version management
- Organization-specific plugin stores

---

## 🧪 Testing Your Plugin

### Unit Test Example

**File:** `rules/__tests__/no-plain-card-data.test.ts`

```typescript
import { describe, it, expect } from 'vitest';
import noPlainCardData from '../no-plain-card-data';
import { createMockContext } from '@arch/validation/testing';

describe('PCI: No Plain Card Data', () => {
  it('reports error when component handles card data without PCI tag', () => {
    const ctx = createMockContext({
      model: {
        nodes: [
          {
            id: 'paymentApi',
            type: 'component',
            metadata: {
              tags: ['credit-card'] // Missing PCI tag
            }
          }
        ]
      }
    });

    const issues = noPlainCardData.apply(ctx);
    
    expect(issues).toHaveLength(1);
    expect(issues[0].ruleId).toBe('pci/no-plain-card-data');
    expect(issues[0].severity).toBe('error');
    expect(issues[0].message).toContain('paymentApi');
  });

  it('passes when component has both credit-card and PCI tags', () => {
    const ctx = createMockContext({
      model: {
        nodes: [
          {
            id: 'paymentApi',
            type: 'component',
            metadata: {
              tags: ['credit-card', 'PCI'] // Has both
            }
          }
        ]
      }
    });

    const issues = noPlainCardData.apply(ctx);
    expect(issues).toHaveLength(0);
  });
});
```

---

## 📚 Plugin Development Best Practices

### 1. Rule IDs

Use namespaced IDs:
- ✅ `pci/no-plain-card-data`
- ✅ `aws/well-architected-cost`
- ❌ `no-plain-card-data` (not namespaced)

### 2. Error Messages

Make messages actionable:
- ✅ `Component 'paymentApi' handles card data but is not tagged "PCI". Add the "PCI" tag to comply with PCI-DSS requirements.`
- ❌ `Invalid component`

### 3. Metadata

Include helpful metadata:
```typescript
metadata: {
  nodeId: node.id,
  nodeType: node.type,
  requiredTags: ['PCI'],
  documentation: 'https://example.com/pci-compliance'
}
```

### 4. Severity

Choose appropriate severity:
- `error`: Blocks deployment, violates compliance
- `warning`: Best practice, should be addressed

### 5. Performance

- Keep rules fast (avoid heavy computations)
- Use early returns when possible
- Cache expensive lookups

### 6. Documentation

Always include:
- Rule description
- Examples of violations
- How to fix violations
- References to standards/requirements

---

## 🏁 Final Summary

You now have:

- ✅ A **complete plugin API example**
- ✅ 2 real PCI rules
- ✅ Correct structure for bundling
- ✅ LSP-compatible location reporting
- ✅ CLI-compatible validation output
- ✅ Ability for organizations to create their own rule sets
- ✅ Plugin loading via config
- ✅ Testing examples
- ✅ Best practices

This design is **simple**, **powerful**, and **future-proof** — exactly what your architecture modeling ecosystem needs.

---

[← Back to Documentation Index](../README.md)

