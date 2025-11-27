# Architecture Linting & Style Engine

**Status**: Advanced Engine  
**Pillars**: Cross-Pillar (Governance, Operational Excellence)

[← Back to Engines](../README.md)

## Overview

The Architecture Linting & Style Engine ensures clean, consistent, readable, and idiomatic architecture DSL and diagrams across entire organizations.

This engine acts like:
- **ESLint** (for code)
- **Prettier** (for formatting)
- **Stylelint** (for consistency)
- **SonarLint** (for quality)
- **Rustfmt / GoFmt** (for canonical formatting)

...but built **for your Architecture DSL**.

## Purpose

The engine enforces:

- ✅ Naming conventions
- ✅ Component structure rules
- ✅ Domain-driven naming
- ✅ Dependency ordering
- ✅ File organization
- ✅ Formatting & whitespace
- ✅ Import ordering
- ✅ Grouping & hierarchy
- ✅ Event naming rules
- ✅ Consistency across modules
- ✅ Organizational style guides

Includes:

- **lint errors**
- **lint warnings**
- **fix suggestions**
- **auto-fix patches**
- **prettier-like formatting**
- **rulesets via plugins**

## Architecture

```
Lint Engine
 ├── Style Rule Engine
 │     ├── Naming Rules
 │     ├── File Style Rules
 │     ├── Formatting Rules
 ├── Structure Rule Engine
 │     ├── AQL checks
 │     ├── Graph rules
 ├── AI Semantic Engine
 │     ├── Domain linguistic analysis
 │     ├── DDD vocabulary validation
 │     ├── Naming sanity checks
 ├── Auto-Fix Generator
 └── Reporter
```

## Rule Categories

### 1. Naming Rules

Built-in rules:

- camelCase for IDs
- PascalCase for domains
- no underscores
- component names must be descriptive
- no duplicate names in same layer
- event naming validation (past tense)

Customizable via plugin:

```yaml
naming:
  contexts: PascalCase
  modules: kebab-case
  components: PascalCase
  events:
    mustEndWith: "Event"
```

### 2. Style Rules

Examples:

- require newlines between sections
- require descriptions for each component
- require tags for each service
- enforce sorted properties

### 3. Structural Rules

Example rule:

```yaml
StructureRule:
  forbid:
    - from: Component
      to: Database
      unless: ["Repository"]
```

Another:

```yaml
Layering:
  Domain -> Application -> Infra
```

### 4. File Organization Rules

Lint:

- folder structure
- file naming
- import ordering
- module grouping

### 5. Anti-Smell Rules

Detect:

- ambiguous naming
- deep graph chains
- too many responsibilities
- misaligned bounded contexts
- oversized domains
- naming conflicts

### 6. "Dead Architecture" Rules

Detect unused:

- events
- commands
- modules
- user journeys
- unused services
- orphan components

### 7. Evolution Consistency Rules

Lint history:

- components renamed without ADR
- boundaries changed without justification
- event contract breaks
- drift in domain terms

## Integration with Other Engines

The linter reuses:

### Global Model
Checks naming + structure.

### AQL Engine
Supports structural constraints.

### Policy Engine
Allows soft rules (lint) vs hard rules (policy).

### Documentation Engine
Enforces style before generating docs.

### Code Gen Engine
Warns about style issues in generated code.

## UI & Editor Integration

### In the DSL Editor (Monaco/CodeMirror):
- inline warnings
- inline auto-fix suggestions
- quick-fix popup ("Fix naming")
- formatting on save
- lint diagnostic panel

### Diagram View
- misnamed nodes show red/yellow outline
- click → show explanation
- neon highlight for naming inconsistencies

## Command Line Interface

```
sruja lint                       # run all linters
sruja lint --fix                 # auto-fix
sruja lint --ruleset companyX    # custom styleset
sruja format                     # apply canonical formatting
sruja style-check                # check style rules only
```

CI friendly.

## Configuration (lint.json)

Example:

```json
{
  "rules": {
    "naming": {
      "components": "PascalCase",
      "modules": "kebab-case"
    },
    "formatting": {
      "indent": 2,
      "sortProperties": true
    },
    "structure": {
      "requireDescriptions": true,
      "maxComponentsPerModule": 20
    }
  },
  "plugins": ["./company-style-rules"]
}
```

## Auto-Fix Engine

Lint violations can produce:

- inline fixes
- DSL patches
- refactoring suggestions

### Example:

**Before:**
```sruja
component user_svc "User"
```

**After fix:**
```sruja
Component UserService {
  description: "Handles user profiles and authentication"
}
```

## AI-Semantic Linter

AI detects semantic style problems:

- ambiguous naming
- incorrect domain mapping
- overlapping responsibilities
- inconsistent domain language
- misaligned bounded contexts
- event names lacking domain vocabulary
- unidiomatic architectural wording

Example:

> "PaymentHandler" is ambiguous; rename to "PaymentAuthorizationService" based on its behavior and dependencies.

## Plugin-Based Rule System

Organizations can define:

- naming rules
- grouping rules
- custom lint checks
- refactoring templates
- semantic naming constraints
- domain naming dictionary
- layer style guides

Plugins register:

```typescript
lintRules: { ... }
styleRules: { ... }
formatRules: { ... }
autoFixers: { ... }
```

## Formatting Engine (like Prettier)

Apply canonical formatting:

- indentation
- block collapsing
- sorting
- grouping
- blank line rules
- attribute ordering
- deterministic serialization

Output is **100% stable** → ideal for Git diffs.

## Implementation Status

✅ Architecture designed  
✅ Rule categories defined  
✅ Auto-fix engine specified  
📋 Implementation in progress

---

*The Architecture Linting & Style Engine dramatically improves readability, team alignment, and consistency across architecture repos.*

