# Multi-Module Architecture Linking & Composition

**Complete, production-grade system** for linking architecture definitions across services, modules, and repos.

**Status**: ✅ **Core concepts are relevant for Go CLI**  
**Note**: Code examples in this document are TypeScript. They need to be updated to Go during implementation, but the concepts and design are valid.

Real-world architectures have many modules, many services, many teams, and many repos—and the architecture model must:
- ✅ Reference elements across modules
- ✅ Compose distributed systems into a single logical view
- ✅ Allow reuse of shared components
- ✅ Support boundaries between teams
- ✅ Allow "partial architecture" per repo
- ✅ Combine into a global architecture graph

---

## 🎯 The Problem to Solve

Large systems typically look like:

```
/backend/auth-service
/backend/payments-service
/backend/order-service
/frontend/storefront
/frontend/admin-portal
/shared/libs
/infra
```

Each project might contain **partial architecture**:
- Auth Service defines: tokens, sessions, auth API
- Payments Service defines: payment API, events, payment DB
- Storefront defines: consumers of these APIs

But the **global architecture** must connect:

```
storefront → auth-service.api
storefront → payments-service.api
payments-service → order-service → inventory-service
```

**How do we model these connections?**

This requires:
- ✅ Namespacing
- ✅ Referencing
- ✅ Importing
- ✅ Project linking
- ✅ Dependency resolution
- ✅ Global composition

---

## 🎯 Design Goals

1. Model each service/module independently
2. Compose them into a global model
3. Allow cross-project referencing
4. Support Git-based linking
5. Support multiple levels (system → container → component)
6. Avoid global ID collisions
7. Support partial definitions without circular imports
8. Minimal overhead for teams

---

## 🏗️ Solution: Namespaced Architecture Model + Imports

Every project will be a **namespaced architecture module**.

### Example:

```dsl
module auth {
  container api: Service "Auth API"
  container db: Database "Users DB"
}
```

Another service can reference those elements:

```dsl
import "github.com/acme/auth-service/architecture.sruja" as auth

system "Storefront" {
  container web: Frontend "Web"
  web -> auth.api: "Login"
}
```

This accomplishes:
- ✅ Cross-project linking
- ✅ Namespace safety
- ✅ Architectural composition

---

## 🧩 Detailed Linking System Design

### 1. Every DSL file defines a **module ID**

```dsl
module <id> { ... }
```

**Examples:**

```dsl
module payments
module auth
module storefront
module shared
```

### 2. Elements inside module get **hierarchical IDs**

```
auth.api
auth.db
payments.stripeGateway
payments.db
```

### 3. Cross-project references are always **fully qualified**

```dsl
storefront.web -> auth.api
payments.worker -> orders.api
```

This prevents collisions.

---

## 📥 Import System

The import system lets one architecture DSL file pull in others:

```dsl
import "./local.sruja" as local
import "github.com/acme/auth-service/architecture.sruja" as auth
import "../payments/architecture.sruja" as pay
```

### Import Types

| Type               | Example                          | Description            |
| ------------------ | -------------------------------- | ---------------------- |
| **Relative path**  | `./auth.sruja`                   | same repo              |
| **GitHub URL**     | `"github.com/acme/auth"`         | remote repo            |
| **Registry**       | `"@acme/auth"`                   | enterprise registry    |
| **Local monorepo** | `"../auth-service"`              | multi-service monorepo |
| **Versioned**      | `"github.com/acme/auth#v1.2.0"` | specific version       |

### Import Resolution

Your CLI resolves them:

```bash
sruja compile
sruja build
```

The compiler downloads remote imports and caches them in:

```
.architecture/imports/
```

Which keeps things reproducible + CI-friendly.

---

## 🧠 Model Composition Engine

The compiler builds a **global architecture model** by merging:
- The current module
- All imported modules
- All levels: system, container, component
- All relationships, references, metadata

### Composition Steps

#### **1. Parse current module**

Build module: `storefront`

#### **2. Resolve imports**

For each import:
- Fetch file
- Compile it (recursively)
- Register namespace
- Attach model under namespace root

#### **3. Validate cross-module references**

```dsl
web -> auth.api           ✓
orders.worker -> pay.db   ✓
unknown → abc.xyz         ✗ error
```

#### **4. Compose global graph**

Build graph:

```
Nodes: all containers & components across modules
Edges: all relations
Namespaces: module → container → component
```

#### **5. Publish global model (JSON)**

```
.architecture/model.json
```

---

## 🔗 Example — Full Multi-Module Linking

### **auth-service/architecture.sruja**

```dsl
module auth {
  container api: Service "Auth API" {
    component tokenService: Component "Token Service"
    component sessionService: Component "Session Service"
  }
  container db: Database "Users DB"
  
  api -> db: "Store sessions"
}
```

### **payments-service/architecture.sruja**

```dsl
module payments {
  container api: Service "Payments API" {
    component processor: Component "Payment Processor"
    component webhook: Component "Webhook Handler"
  }
  container db: Database "Payments DB"
  
  api -> db: "Store transactions"
}
```

### **storefront/architecture.sruja**

```dsl
import "../auth-service/architecture.sruja" as auth
import "../payments-service/architecture.sruja" as pay

module storefront {
  container web: Frontend "Storefront Web" {
    component cart: Component "Shopping Cart"
    component checkout: Component "Checkout Flow"
  }
  
  web -> auth.api: "Login"
  web -> pay.api: "Checkout"
  checkout -> pay.api.processor: "Process payment"
}
```

### **Global Composed Model**

The global composed model includes:

**Nodes:**
```
auth.api
auth.api.tokenService
auth.api.sessionService
auth.db
payments.api
payments.api.processor
payments.api.webhook
payments.db
storefront.web
storefront.web.cart
storefront.web.checkout
```

**Relationships:**
```
auth.api → auth.db
payments.api → payments.db
storefront.web → auth.api
storefront.web → payments.api
storefront.web.checkout → payments.api.processor
```

---

## 📘 Validation Support

The validation engine must support:

### ✅ Cross-Module Reference Resolution

Check unknown IDs across boundaries:

```typescript
// In validation engine
function validateCrossModuleReference(
  fromModule: string,
  toModule: string,
  elementId: string,
  globalModel: ComposedModel
): ValidationIssue[] {
  const qualifiedId = `${toModule}.${elementId}`;
  const exists = globalModel.hasNode(qualifiedId);
  
  if (!exists) {
    return [{
      ruleId: "semantic/unknown-cross-module-reference",
      message: `Reference to unknown element: ${qualifiedId}`,
      severity: "error"
    }];
  }
  
  return [];
}
```

### ✅ Namespace Collision Detection

Two modules cannot both be `module orders`:

```typescript
function detectNamespaceCollision(
  modules: Module[]
): ValidationIssue[] {
  const seen = new Map<string, string>();
  const issues = [];
  
  for (const module of modules) {
    if (seen.has(module.id)) {
      issues.push({
        ruleId: "semantic/namespace-collision",
        message: `Module ID '${module.id}' is already used by ${seen.get(module.id)}`,
        severity: "error"
      });
    } else {
      seen.set(module.id, module.source);
    }
  }
  
  return issues;
}
```

### ✅ Import Cycle Detection

Detect circular imports (auth → storefront → auth):

```typescript
function detectImportCycles(
  module: Module,
  visited: Set<string> = new Set(),
  path: string[] = []
): ValidationIssue[] {
  if (visited.has(module.id)) {
    const cycle = [...path, module.id];
    return [{
      ruleId: "semantic/import-cycle",
      message: `Circular import detected: ${cycle.join(" → ")}`,
      severity: "error",
      metadata: { cycle }
    }];
  }
  
  visited.add(module.id);
  path.push(module.id);
  
  const issues = [];
  for (const import of module.imports) {
    issues.push(...detectImportCycles(import.module, visited, path));
  }
  
  path.pop();
  visited.delete(module.id);
  
  return issues;
}
```

### ✅ Origin Mapping

For LSP diagnostics, track where each element comes from:

```typescript
interface NodeOrigin {
  module: string;
  file: string;
  line: number;
  column: number;
}

interface ComposedNode {
  id: string;
  qualifiedId: string; // e.g., "auth.api"
  origin: NodeOrigin;
  // ... node data
}
```

### ✅ Imported Rule Injection

Plugins can add module-specific rules:

```typescript
// Plugin can validate cross-module relationships
const rule: ValidationRule = {
  id: "custom/cross-module-boundary",
  apply(ctx: ValidationContext) {
    const issues = [];
    
    for (const edge of ctx.model.edges) {
      const fromModule = extractModule(edge.from);
      const toModule = extractModule(edge.to);
      
      if (fromModule !== toModule) {
        // Validate cross-module boundary rules
        // e.g., frontend cannot directly access database
      }
    }
    
    return issues;
  }
};
```

---

## ⚙️ DSL Additions Required

Add new grammar constructs to the DSL:

### Module Declaration

```ohm
ModuleDeclaration
  = "module" identifier "{" ModuleBody "}"
  
ModuleBody
  = (ContainerDeclaration | SystemDeclaration | ComponentDeclaration | ImportDeclaration)*
```

### Import Declaration

```ohm
ImportDeclaration
  = "import" StringLiteral "as" identifier
```

### Qualified Identifier

```ohm
QualifiedIdentifier
  = identifier "." identifier
  | identifier
```

This powers `auth.api` style linking.

### Updated Grammar Example

```ohm
Architecture
  = ModuleDeclaration+
  
ModuleDeclaration
  = "module" identifier "{" ModuleContent* "}"
  
ModuleContent
  = ImportDeclaration
  | ContainerDeclaration
  | SystemDeclaration
  | ComponentDeclaration
  | EdgeDeclaration
  
ImportDeclaration
  = "import" StringLiteral "as" identifier
  
QualifiedIdentifier
  = identifier ("." identifier)*
  
EdgeDeclaration
  = QualifiedIdentifier "->" QualifiedIdentifier (":" StringLiteral)?
```

---

## 📦 Project Format Support (APF v1)

We add to the Architecture Project Format:

### `.architecture/imports/`

Resolved remote modules (versioned by commit hash or tag):

```
.architecture/imports/
  github.com_acme_auth-service_v1.2.0/
    architecture.sruja
    model.json
  github.com_acme_payments-service_main/
    architecture.sruja
    model.json
```

### `.architecture/modules.json`

List of all imported modules + versions:

```json
{
  "modules": [
    {
      "id": "auth",
      "source": "github.com/acme/auth-service",
      "version": "v1.2.0",
      "commit": "abc123",
      "importedAt": "2024-01-01T00:00:00Z"
    },
    {
      "id": "payments",
      "source": "../payments-service",
      "version": "local",
      "importedAt": "2024-01-01T00:00:00Z"
    }
  ]
}
```

### `.architecture/compose.json`

Full composed graph per module:

```json
{
  "modules": {
    "auth": {
      "nodes": ["auth.api", "auth.db"],
      "edges": [{"from": "auth.api", "to": "auth.db"}]
    },
    "storefront": {
      "nodes": ["storefront.web"],
      "edges": [
        {"from": "storefront.web", "to": "auth.api"}
      ]
    }
  },
  "global": {
    "nodes": [
      {"id": "auth.api", "module": "auth"},
      {"id": "auth.db", "module": "auth"},
      {"id": "storefront.web", "module": "storefront"}
    ],
    "edges": [
      {"from": "auth.api", "to": "auth.db"},
      {"from": "storefront.web", "to": "auth.api"}
    ]
  }
}
```

---

## 🔧 Implementation Structure

### Module Resolver

**File:** `packages/model-engine/src/resolver.ts`

```typescript
export interface ModuleResolver {
  resolve(importPath: string): Promise<Module>;
}

export class GitHubModuleResolver implements ModuleResolver {
  async resolve(importPath: string): Promise<Module> {
    // Parse GitHub URL
    // Download and cache
    // Return Module
  }
}

export class LocalModuleResolver implements ModuleResolver {
  async resolve(importPath: string): Promise<Module> {
    // Resolve relative path
    // Load from filesystem
    // Return Module
  }
}
```

### Composition Engine

**File:** `packages/model-engine/src/composer.ts`

```typescript
export class ArchitectureComposer {
  async compose(
    rootModule: Module,
    resolver: ModuleResolver
  ): Promise<ComposedModel> {
    const modules = new Map<string, Module>();
    const visited = new Set<string>();
    
    // Recursively resolve all imports
    await this.resolveImports(rootModule, resolver, modules, visited);
    
    // Build global graph
    const globalModel = this.buildGlobalModel(modules);
    
    // Validate cross-module references
    const issues = this.validateCrossModuleReferences(globalModel);
    
    return {
      modules,
      global: globalModel,
      issues
    };
  }
  
  private async resolveImports(
    module: Module,
    resolver: ModuleResolver,
    modules: Map<string, Module>,
    visited: Set<string>
  ): Promise<void> {
    if (visited.has(module.id)) {
      return; // Already resolved
    }
    
    visited.add(module.id);
    modules.set(module.id, module);
    
    for (const import of module.imports) {
      const importedModule = await resolver.resolve(import.path);
      await this.resolveImports(importedModule, resolver, modules, visited);
    }
  }
}
```

---

## 🚀 Advanced Features (Optional, Future)

### 1. Multi-Team Contract Enforcement

Teams can publish "API contracts" for their services:

```dsl
module auth {
  contract api {
    endpoint POST /login
    endpoint POST /logout
    endpoint GET /user
  }
  
  container api: Service "Auth API" {
    implements contract api
  }
}
```

Other modules can validate against contracts:

```dsl
import "github.com/acme/auth-service" as auth

module storefront {
  container web: Frontend "Web" {
    consumes auth.api.contract
  }
}
```

### 2. Version Pinning

```dsl
import "github.com/acme/auth#v1.2.0"
import "github.com/acme/payments#main"
import "github.com/acme/orders#abc123" // commit hash
```

### 3. Monorepo Linking

Use workspace metadata to auto-detect sibling modules:

```json
// package.json or pnpm-workspace.yaml
{
  "workspaces": [
    "packages/*",
    "services/*"
  ]
}
```

Auto-import sibling modules:

```dsl
// In storefront/architecture.sruja
import "@workspace/auth-service" as auth
import "@workspace/payments-service" as pay
```

### 4. Distributed Architecture Validation

Each module validates independently + globally:

```bash
# Validate single module
sruja validate --module auth

# Validate all modules
sruja validate --all

# Validate with cross-module rules
sruja validate --cross-module
```

### 5. GraphQL-Style Introspection

Query the composed architecture model:

```graphql
query {
  modules {
    id
    nodes {
      id
      type
    }
    edges {
      from
      to
    }
  }
  
  global {
    nodes {
      id
      module
    }
    edges {
      from
      to
    }
  }
}
```

### 6. Architecture Dependency Graph

Visualize module dependencies:

```bash
sruja graph --modules
```

Outputs:

```
auth (no dependencies)
payments (depends on: auth)
storefront (depends on: auth, payments)
```

---

## 🏁 Summary — How You Link Large Systems

Your modeling platform supports:

- ✅ Namespaced modules
- ✅ Fully-qualified identifiers
- ✅ Cross-module references
- ✅ Import system (local, git, registry)
- ✅ Global model composition
- ✅ Cross-project validation
- ✅ Reusable shared libraries
- ✅ Future cloud marketplace integration

This is exactly how:
- Bazel WORKSPACE + BUILD compositions
- Rust crates linking
- Node monorepos
- Terraform modules
- Protobuf packages

work at scale.

You now have a **world-class architecture linking system**.

---

## 📚 Related Documentation

- [Project Format](./project-format.md) - Architecture Project Format v1
- [DSL Specification](./dsl-specification.md) - Complete DSL grammar
- [Validation Engine](./validation-engine.md) - Validation API
- [CLI Specification](./cli-specification.md) - Command-line tool

---

[← Back to Documentation Index](../README.md)


