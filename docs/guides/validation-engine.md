# Architecture Validation Engine v1 — API Specification

**Version:** `1.0`  
**Status:** Stable for MVP  
**Scope:** Syntax + Semantic + Layer + Custom Rules validation

Production-grade API specification for the core validation module used by CLI, LSP, cloud backend, editor UI, CI pipelines, and plugins.

---

## 🎯 Overview

The Validation Engine is a **pure functional core module**.

**Input:**
- DSL text
- Parsed model
- Project context
- Config (rules + plugins)

**Output:**
- Diagnostics (errors + warnings)
- Structured metadata
- Severity levels
- Rule IDs
- Ranges for editor/highlighting

**Works in:**
- CLI (`sruja validate`)
- LSP (diagnostics)
- Cloud backend (review pipelines)
- Editor UI (live warnings/errors)
- CI pipelines
- Plugins / custom rules

**Design Principle:** No side effects, no DB, no network. Pure functions only.

---

## 📦 Core API Entry Point

```typescript
function validateArchitecture(input: ValidationInput): ValidationResult;
```

---

## 🔧 Type Definitions

### `ValidationInput`

```typescript
interface ValidationInput {
  dsl: string;                     // raw DSL text
  ast: AST;                        // parsed AST from Ohm
  model: ArchitectureModel;        // normalized model (from compiler)
  project: ArchitectureProject;    // APF project info & files
  config?: ValidationConfig;       // optional validation rules
}
```

### `ValidationResult`

```typescript
interface ValidationResult {
  ok: boolean;                     // shorthand: no errors
  errors: ValidationIssue[];       // blocking issues
  warnings: ValidationIssue[];     // non-blocking issues
  summary: ValidationSummary;      // counts, rule categories
}
```

### `ValidationIssue`

```typescript
interface ValidationIssue {
  ruleId: string;                  // e.g. "semantic/no-unknown-reference"
  message: string;                 // human-readable error message
  severity: "error" | "warning";
  location?: SourceLocation;       // for LSP/Monaco
  metadata?: Record<string, any>;  // optional info (nodeId, type, etc.)
}
```

### `SourceLocation`

Matches LSP location spec.

```typescript
interface SourceLocation {
  file: string;     // usually "architecture.sruja"
  start: Position;  
  end: Position;
}

interface Position {
  line: number;     // 1-based
  column: number;   // 1-based
}
```

### `ValidationSummary`

```typescript
interface ValidationSummary {
  errorCount: number;
  warningCount: number;
  rulesExecuted: number;
  ruleBreakdown: Record<string, number>;
}
```

### `ValidationConfig`

```typescript
interface ValidationConfig {
  strictLayers?: boolean;           // require system > container > component
  strictImports?: boolean;          // require all imports to resolve
  warningAsError?: boolean;
  enabledRules?: string[];          // rule IDs
  disabledRules?: string[];
  plugins?: ValidationPlugin[];     // custom rule modules
}
```

### `ValidationPlugin`

Plugins allow external rules to be registered.

```typescript
interface ValidationPlugin {
  name: string;
  version: string;
  rules: ValidationRule[];
}
```

### `ValidationRule`

Rules operate on the architecture model, AST, and DSL.

```typescript
interface ValidationRule {
  id: string;                       // unique, namespaced ("semantic/no-duplicate-id")
  description: string;
  severity: "error" | "warning";
  apply(ctx: ValidationContext): ValidationIssue[] | Promise<ValidationIssue[]>;
}
```

## Go Validation Engine API

### Types

```go
type Severity string

const (
    SeverityError   Severity = "error"
    SeverityWarning Severity = "warning"
)

type ValidationIssue struct {
    RuleID     string
    Severity   Severity
    Message    string
    Location   string
}

type ValidationContext struct {
    Model      *ArchitectureModel
    AST        *Program
    Config     map[string]any
    Enabled    []string
    Disabled   []string
}

type ValidationRule interface {
    ID() string
    Apply(ctx ValidationContext) []ValidationIssue
}

type ValidationPlugin struct {
    Name    string
    Version string
    Rules   []ValidationRule
}

type ValidationResult struct {
    Issues  []ValidationIssue
}

type ValidationEngine struct {
    rules []ValidationRule
}

func (e *ValidationEngine) Register(rules ...ValidationRule) {
    e.rules = append(e.rules, rules...)
}

func (e *ValidationEngine) Execute(ctx ValidationContext) ValidationResult {
    var issues []ValidationIssue
    for _, r := range e.rules {
        if isDisabled(r.ID(), ctx.Disabled) { continue }
        if len(ctx.Enabled) > 0 && !isEnabled(r.ID(), ctx.Enabled) { continue }
        issues = append(issues, r.Apply(ctx)...)
    }
    return ValidationResult{Issues: issues}
}

func isEnabled(id string, enabled []string) bool {
    for _, e := range enabled { if e == id { return true } }
    return false
}

func isDisabled(id string, disabled []string) bool {
    for _, d := range disabled { if d == id { return true } }
    return false
}
```

### Plugin Registration

```go
engine := &ValidationEngine{}
engine.Register(plugin.Rules...)
res := engine.Execute(ValidationContext{Model: model, AST: ast})
```

### `ValidationContext`

The context passed into each rule.

```typescript
interface ValidationContext {
  dsl: string;
  ast: AST;
  model: ArchitectureModel;
  project: ArchitectureProject;
  config: ValidationConfig;
  report(issue: ValidationIssue): void;
}
```

---

## 📋 Built-In Validation Rules (v1 Standard)

These rules ship with the core engine.

### 4.1 Syntax Rules

Provided by parser → used by validator.

- `syntax/invalid-token`
- `syntax/unexpected-structure`
- `syntax/missing-closing-brace`

The validation engine **receives** them, does not generate them.

### 4.2 Semantic Rules

#### **1. Unknown reference**
- `semantic/unknown-reference`
- Edge references unknown node
- Journeys referencing nonexistent components
- ADR links missing

#### **2. Duplicate ID**
- `semantic/duplicate-id`

#### **3. Invalid type**
- `semantic/invalid-type`
- (e.g., container defined inside component)

#### **4. Missing required properties**
- `semantic/missing-required-prop`
- Required props in DSL v2 (like `title` in ADR references)

#### **5. Circular dependencies**
- `semantic/circular-dependency`

### 4.3 Layer Rules (if strictLayers = true)

- `layers/no-components-at-root`
- `layers/containers-must-belong-to-system`
- `layers/components-must-belong-to-container`
- `layers/externals-cannot-have-inbound-internal-links`

### 4.4 Best-Practice Rules (optional)

- `bestpractice/avoid-direct-db-calls`
- `bestpractice/external-boundary-check`
- `bestpractice/service-has-api`
- `bestpractice/naming-conventions`

These are warnings by default.

---

## 🛡️ Boundary-Aware Governance Rules (DDD)

These rules enforce organizational boundaries, context ownership, and interaction contracts.

### Cross-Context Access

- `boundaries/forbidden-internal-access`
- Components in context A cannot reference private internals of context B
- Allowed via published API: `boundaries/require-api-access`

### Team Topologies

- `teams/frontend-cannot-own-databases`
- `teams/platform-cannot-depend-on-feature-teams`
- `teams/no-direct-modification-of-external-contexts`

### Domain Boundaries

- `domains/no-direct-db-cross-access` (e.g., `payments.db -> orders.db`)
- `domains/allow-api-cross-access` (e.g., `payments.api -> orders.api`)

### Shared Kernel

- `contexts/shared-kernel-allow-shared-models`

### Anti-Corruption Layer (ACL)

- `contexts/require-acl-indirection` for relations marked `anticorruption-layer`
- References must route via `acalayer.*` or designated adapter nodes

### Example Diagnostics

```
❌ checkout.api → identity.jwtSigner (forbidden)
✅ checkout.web → identity.auth.api (allowed)
```

Enable via config or per-project policy.

## 🔄 Rule Execution Flow

```
Input → Syntax Check → Semantic Check → Layer Validation → Best Practice Rules → Plugins
```

Each rule runs independently with access to the same context.

You can short-circuit at "syntax failure" if desired.

---

## 💻 Example Usage (Programmatic)

```typescript
import { validateArchitecture } from "@arch/core/validation";

const result = validateArchitecture({
  dsl: fs.readFileSync("architecture.sruja", "utf8"),
  ast,
  model,
  project: loadProject("."),
  config: {
    strictLayers: true
  }
});

if (!result.ok) {
  console.error(`${result.summary.errorCount} errors found`);
  result.errors.forEach(err => {
    console.error(`${err.ruleId}: ${err.message}`);
  });
}
```

---

## 📤 Example Output

```json
{
  "ok": false,
  "errors": [
    {
      "ruleId": "semantic/unknown-reference",
      "message": "Reference to undefined component: 'billingApi'",
      "severity": "error",
      "location": {
        "file": "architecture.sruja",
        "start": { "line": 14, "column": 12 },
        "end": { "line": 14, "column": 23 }
      },
      "metadata": {
        "nodeId": "api",
        "referencedId": "billingApi"
      }
    }
  ],
  "warnings": [
    {
      "ruleId": "bestpractice/naming-conventions",
      "message": "Component name 'api' should be more descriptive",
      "severity": "warning",
      "location": {
        "file": "architecture.sruja",
        "start": { "line": 5, "column": 1 },
        "end": { "line": 5, "column": 10 }
      }
    }
  ],
  "summary": {
    "errorCount": 1,
    "warningCount": 1,
    "rulesExecuted": 18,
    "ruleBreakdown": {
      "semantic/unknown-reference": 1,
      "bestpractice/naming-conventions": 1
    }
  }
}
```

Perfect for:
- CLI JSON mode
- LSP diagnostics
- Cloud architecture reviews
- Pull request comments

---

## 🔌 Plugin API Example (User-Defined Validation Rule)

Users can create custom validation plugins.

See [Validation Plugin Example](./validation-plugin-example.md) for a complete, production-ready plugin implementation including:
- Full plugin structure
- Multiple rule examples
- Testing examples
- Packaging and publishing
- Best practices

### Quick Example

```typescript
import type { ValidationPlugin } from "@arch/validation";

export default {
  name: "pci-validator",
  version: "1.0.0",
  rules: [
    {
      id: "pci/no-plain-card-data",
      description: "Components handling card data must be PCI compliant",
      severity: "error",
      apply({ model, getLocation }) {
        const issues = [];
        for (const node of model.nodes) {
          if (node.metadata?.tags?.includes("credit-card") && 
              !node.metadata?.tags?.includes("PCI")) {
            issues.push({
              ruleId: "pci/no-plain-card-data",
              message: `Component '${node.id}' handles card data but is not tagged PCI.`,
              severity: "error",
              location: getLocation?.(node.id),
              metadata: { nodeId: node.id }
            });
          }
        }
        return issues;
      }
    }
  ]
} as ValidationPlugin;
```

### Load via config:

```json
{
  "validation": {
    "plugins": ["./validators/pci-validator.js"]
  }
}
```

---

## 🎯 MVP Rule Set

For MVP, include:

### Required:
- ✅ Duplicate IDs
- ✅ Unknown references
- ✅ Invalid types
- ✅ Layer rules (if strictLayers enabled)
- ✅ Circular dependencies

### Optional:
- ⚠️ Minimal best-practice warnings

### Future:
- 🔮 Security rules
- 🔮 Cloud vendor rules
- 🔮 Domain-specific rules

---

## 🏗 Implementation Structure

```
/packages/validation-engine/
  /src/
    /index.ts              → Main API (validateArchitecture)
    /rules/
      /syntax.ts           → Syntax rule wrappers
      /semantic.ts         → Semantic rules
      /layers.ts           → Layer validation rules
      /bestpractice.ts     → Best practice rules
    /plugins/
      /loader.ts           → Plugin loading
      /registry.ts         → Rule registry
    /context.ts            → ValidationContext
    /types.ts              → Type definitions
  /__tests__/
    /rules.test.ts
    /plugins.test.ts
```

---

## 🔗 Integration Points

### LSP Integration
```typescript
// In LSP diagnostics feature
const result = validateArchitecture({ dsl, ast, model, project });
const diagnostics = result.errors.concat(result.warnings).map(issue => ({
  range: issue.location,
  severity: issue.severity === "error" ? DiagnosticSeverity.Error : DiagnosticSeverity.Warning,
  message: issue.message,
  source: issue.ruleId
}));
```

### CLI Integration
```typescript
// In sruja validate command
const result = validateArchitecture({ dsl, ast, model, project, config });
if (!result.ok) {
  process.exit(2); // Exit code 2 = validation errors
}
```

### Editor UI Integration
```typescript
// In React component
const result = validateArchitecture({ dsl, ast, model, project });
setValidationErrors(result.errors);
setValidationWarnings(result.warnings);
```

---

## 🧠 Why This API Design Matters

Your **Validation Engine v1 API** is:

- ✅ **Pure functions** - No side effects, works anywhere
- ✅ **Extensible** - Plugin system for custom rules
- ✅ **CI-friendly** - JSON output, exit codes
- ✅ **Plugin-ready** - Community can contribute rules
- ✅ **Universal** - Works in CLI, LSP, Cloud, VSCode
- ✅ **Stable and predictable** - Same input = same output

This makes your DSL & architecture platform ready for:

- Enterprise-grade rules
- Community plugins
- AI-driven validation
- Architecture scoring
- Automation in CI/CD

---

[← Back to Documentation Index](../README.md)
