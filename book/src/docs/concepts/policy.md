---
title: "Policy"
weight: 50
summary: "Define architectural rules and constraints that must be followed."
---

# Policy

Policies define architectural rules, standards, and constraints that your system must follow. They help enforce best practices, compliance requirements, and organizational standards directly in your architecture model.

## Syntax

```sruja
PolicyID = policy "Short policy statement" {
  category "security"
  enforcement "required"
  description "Explain intent, scope, and rationale. Rules below are what get evaluated."
  rule deny edge from { kind "external_api" } to { kind "database" }
}
```

## Writing strong policies

- Make the title a testable statement ("DBs must be encrypted at rest")
- Set `category` so policies can be grouped (security, compliance, performance, data, observability)
- Pick `enforcement` based on impact (`required` for must-fix, `recommended` for should-fix, `optional` for guidance)
- Encode enforcement using `rule ...` entries so `sruja lint` can evaluate them
- Use `except ...` to record deliberate exceptions instead of relying on tribal knowledge

## Simple Policy

```sruja
import { * } from 'sruja.ai/stdlib'


SecurityPolicy = policy "Enforce TLS 1.3 for all external communications"

view index {
  include *
}
```

## Policy Fields

- **`ID`**: Unique identifier for the policy (e.g., `SecurityPolicy`, `GDPR_Compliance`)
- **`Description`**: Human-readable description of the policy
- **`category`** (optional): Policy category (e.g., "security", "compliance", "performance")
- **`enforcement`** (optional): Enforcement level (`required` | `recommended` | `optional`). Synonyms are accepted: `error`/`warn`/`info`.
- **`description`** (optional): Detailed description within the policy body
- **Defaults**: `category` defaults to `general`, `enforcement` defaults to `warn`

## Editor and CI integration

- The VS Code extension surfaces policy nodes in the outline and shows lint violations as diagnostics.
- CI and CLI checks (lint/drift/compliance) treat `enforcement` as severity (`required` → error, `recommended` → warning, `optional` → info).

## Example: Security Policies

```sruja
// partial
import { * } from 'sruja.ai/stdlib'


TLSEnforcement = policy "All external communications must use TLS 1.3" {
  category "security"
  enforcement "required"
}

EncryptionAtRest = policy "Sensitive data must be encrypted at rest" {
  category "security"
  enforcement "required"
}

BankingApp = system "Banking App" {
  API = container "API Service"
  CustomerDB = database "Customer Database"
}

view index {
  include *
}
```

## Example: Compliance Policies

```sruja
import { * } from 'sruja.ai/stdlib'


HIPAACompliance = policy "Must comply with HIPAA regulations" {
  category "compliance"
  enforcement "required"
  description "All patient data must be encrypted and access logged"
}

DataRetention = policy "Medical records retained for 10 years" {
  category "compliance"
  enforcement "required"
}

view index {
  include *
}
```

## Policy Categories

Common policy categories include:

- **`security`**: Security standards and practices
- **`compliance`**: Regulatory and legal requirements
- **`performance`**: Performance standards and SLAs
- **`observability`**: Monitoring, logging, and metrics requirements
- **`architecture`**: Architectural patterns and principles
- **`data`**: Data handling and privacy requirements

## Enforcement Levels

- **`required`**: Policy must be followed (non-negotiable)
- **`recommended`**: Policy should be followed (best practice)
- **`optional`**: Policy is a guideline (suggested)

## Benefits

- **Documentation**: Policies are part of your architecture, not separate documents
- **Validation**: Can be validated against actual implementations
- **Communication**: Clear standards for development teams
- **Compliance**: Track regulatory and organizational requirements
- **Governance**: Enforce architectural decisions and patterns

## Policy Rules

Policies can include **structured rules** that are evaluated by the rule engine (deterministic; no ad-hoc phrase parsing).

### Selectors

Policy rules match elements using **selectors**:

```sruja
// partial
{ 
  kind "database"
  id "Shop.DB"
  tag "production"
  technology "PostgreSQL"
  meta "owner" "team-payments"
}
```

Supported selector fields:

- `kind "..."` — element kind (normalized: lowercase; spaces/hyphens become underscores)
- `id "..."` — element id (fully-qualified like `Shop.DB` or leaf id like `DB`)
- `tag "..."` — required tag (can also be written as `#tag` / `@tag` on elements)
- `technology "..."` — matches `technology "..."` on elements
- `meta "key" "value"` — matches `metadata { key "value" }` on elements (value match is case-insensitive)

### `deny edge`

Deny a class of edges by matching source and target selectors:

```sruja
NoExtToDb = policy "External APIs must not call databases" {
  category "security"
  enforcement "required"
  rule deny edge from { kind "external_api" } to { kind "database" }
}
```

When you run `sruja lint` or drift/compliance checks, the rule engine evaluates each edge in the graph; if an edge matches (source selector → target selector), a policy violation is reported.

You can also add:

- `except from { ... } to { ... }` — allow a specific edge pattern
- `message "..."` — override the default violation message
- `suggest "..."` — add one or more suggestions (repeatable)

```sruja
NoServiceToDb = policy "Services must not call DBs" {
  enforcement "required"
  rule deny edge
    from { kind "service" }
    to { kind "database" }
    except from { id "Checkout" } to { id "PaymentsDb" }
    message "Direct DB access is forbidden"
    suggest "Route via an API"
    suggest "If intentional, add an exception"
}
```

### `require tags`

Require one or more tags to exist on matched elements:

```sruja
EncryptionPolicy = policy "DBs must be encrypted" {
  enforcement "required"
  rule require tags on { kind "database" } tags ["encrypted"]
}
```

Optional tail items:

- `except { ... }` — exclude elements matching a selector
- `message "..."` / `suggest "..."` — same as `deny edge`

### `require metadata`

Require a metadata key (and optionally a specific value) on matched elements:

```sruja
OwnershipPolicy = policy "Containers must declare owners" {
  enforcement "recommended"
  rule require metadata on { kind "container" } key "owner"
}
```

With a required value:

```sruja
EnvPolicy = policy "Production containers must declare environment" {
  enforcement "required"
  rule require metadata on { kind "container" tag "production" } key "environment" value "prod"
}
```

### `require slo`

Require an `slo { ... }` block on matched elements:

```sruja
SloPolicy = policy "Production containers must have SLOs" {
  enforcement "required"
  rule require slo on { kind "container" tag "production" }
}
```

### Evaluation

Policy evaluation is **rule-based**: the DSL is the way you author rules, and the evaluator runs them deterministically over the architecture graph. Policy titles/descriptions document intent; only `rule ...` entries are evaluated.

## See Also

- [Architecture](docs/concepts/architecture.md)
- [System](docs/concepts/system.md)
- [Requirements](docs/concepts/requirements.md)
- [ADR](docs/concepts/adr.md)
