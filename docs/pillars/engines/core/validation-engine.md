# Validation Engine

**Status**: Core Engine  
**Pillar**: All (foundational)

[← Back to Engines](../README.md)

## Overview

The Validation Engine is a **pure functional core module** that validates architecture models for syntax, semantics, layering, and custom rules.

## Purpose

The Validation Engine:
- Validates DSL syntax (from parser)
- Validates semantics (IDs, references, types)
- Validates layering rules (strict hierarchy)
- Detects cycles and anti-patterns
- Provides LSP-compatible diagnostics
- Supports plugin-based custom rules
- Is pure, deterministic, and side-effect-free

## API Specification

### Core Entry Point

```typescript
function validateArchitecture(input: ValidationInput): ValidationResult;
```

### Types

#### `ValidationInput`

```typescript
interface ValidationInput {
  dsl: string;                     // raw DSL text
  ast: AST;                        // parsed AST from Ohm
  model: ArchitectureModel;        // normalized model (from compiler)
  project: ArchitectureProject;    // APF project info & files
  config?: ValidationConfig;       // optional validation rules
}
```

#### `ValidationResult`

```typescript
interface ValidationResult {
  ok: boolean;                     // shorthand: no errors
  errors: ValidationIssue[];       // blocking issues
  warnings: ValidationIssue[];     // non-blocking issues
  summary: ValidationSummary;      // counts, rule categories
}
```

#### `ValidationIssue`

```typescript
interface ValidationIssue {
  ruleId: string;                  // e.g. "semantic/no-unknown-reference"
  message: string;                 // human-readable error message
  severity: "error" | "warning";
  location?: SourceLocation;       // for LSP/Monaco
  metadata?: Record<string, any>;  // optional info (nodeId, type, etc.)
}
```

#### `ValidationConfig`

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

## Built-In Rules

### Syntax Rules
- `syntax/invalid-token`
- `syntax/unexpected-structure`
- `syntax/missing-closing-brace`

### Semantic Rules
- `semantic/unknown-reference` - Edge references unknown node
- `semantic/duplicate-id` - Duplicate identifiers
- `semantic/invalid-type` - Invalid type (e.g., container inside component)
- `semantic/missing-required-prop` - Missing required properties
- `semantic/circular-dependency` - Circular dependencies detected

### Layer Rules (if strictLayers = true)
- `layers/no-components-at-root`
- `layers/containers-must-belong-to-system`
- `layers/components-must-belong-to-container`
- `layers/externals-cannot-have-inbound-internal-links`

### Best-Practice Rules (optional)
- `bestpractice/avoid-direct-db-calls`
- `bestpractice/external-boundary-check`
- `bestpractice/service-has-api`
- `bestpractice/naming-conventions`

## Rule Execution Flow

```
Input → Syntax Check → Semantic Check → Layer Validation → Best Practice Rules → Plugins
```

Each rule runs independently with access to the same context.

## Plugin System

Users can create custom validation plugins:

```typescript
interface ValidationPlugin {
  name: string;
  version: string;
  rules: ValidationRule[];
}

interface ValidationRule {
  id: string;                       // unique, namespaced
  description: string;
  severity: "error" | "warning";
  apply(ctx: ValidationContext): ValidationIssue[] | Promise<ValidationIssue[]>;
}
```

### Example Plugin

```typescript
export default {
  name: "pci-validator",
  version: "1.0.0",
  rules: [
    {
      id: "pci/no-plain-card-data",
      description: "Components handling card data must be PCI compliant",
      severity: "error",
      apply({ model }) {
        const issues = [];
        for (const node of model.components) {
          if (node.tags.includes("credit-card") && !node.tags.includes("PCI")) {
            issues.push({
              ruleId: "pci/no-plain-card-data",
              message: `Component '${node.id}' handles card data but is not tagged PCI.`,
              severity: "error"
            });
          }
        }
        return issues;
      }
    }
  ]
}
```

## Usage Example

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
```

## Output Example

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
      }
    }
  ],
  "warnings": [],
  "summary": {
    "errorCount": 1,
    "warningCount": 0,
    "rulesExecuted": 18,
    "ruleBreakdown": {
      "semantic/unknown-reference": 1
    }
  }
}
```

## Integration Points

- **CLI** - `sruja validate` command
- **LSP** - Real-time diagnostics in editors
- **CI/CD** - Pre-commit and pipeline validation
- **Cloud Backend** - Architecture review API

## Implementation Status

✅ API Specification defined  
✅ Type definitions complete  
✅ Plugin system designed  
📋 Implementation in progress

---

*The Validation Engine is foundational - all architecture models must pass validation before use.*
