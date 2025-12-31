---
title: "Validation & Linting"
weight: 30
summary: "Use Sruja's validator to catch errors, orphan elements, and bad references. Includes troubleshooting guide."
tags: ["validation", "linting", "troubleshooting"]
---

# Validation & Linting

Sruja ships with a validation engine that helps keep architectures healthy. This tutorial covers how to use it effectively and troubleshoot common issues.

## Quick Start

```bash
# Lint a single file
sruja lint architecture.sruja

# Lint all .sruja files in a directory
sruja lint ./architectures/

# Get detailed output
sruja lint --verbose architecture.sruja

# Export validation report as JSON (for CI/CD)
sruja lint --json architecture.sruja > lint-report.json
```

## Common Validation Checks

Sruja validates:

1. **Unique IDs**: No duplicate element IDs
2. **Valid references**: Relations must connect existing elements
3. **Cycle detection**: Informational (cycles are valid for many patterns)
4. **Orphan detection**: Elements not used by any relation
5. **Simplicity guidance**: Suggests simpler syntax when appropriate
6. **Constraint violations**: Policy and constraint rule violations

## Real-World Example: E-Commerce Platform

Let's validate a real architecture:

```sruja
element person
element system
element container
element component
element datastore
element queue

Customer = person "Customer"

ECommerce = system "E-Commerce Platform" {
    WebApp = container "Web Application" {
        technology "React"
    }
    API = container "REST API" {
        technology "Go"
    }
    ProductDB = datastore "Product Database" {
        technology "PostgreSQL"
    }
    OrderDB = datastore "Order Database" {
        technology "PostgreSQL"
    }
}

Customer -> ECommerce.WebApp "Browses products"
ECommerce.WebApp -> ECommerce.API "Calls API"
ECommerce.API -> ECommerce.ProductDB "Reads products"
ECommerce.API -> ECommerce.OrderDB "Writes orders"

view index {
include *
}
```

**Validation output:**

```
✅ Valid architecture
✅ All references valid
✅ No orphan elements
ℹ️  Cycle detected: ECommerce.WebApp ↔ ECommerce.API (this is valid for request/response)
```

## Troubleshooting Common Errors

### Error 1: Invalid Reference

**Error message:**

```
❌ Invalid reference: ECommerce.API -> ECommerce.NonExistent "Calls"
   Element 'NonExistent' not found in system 'ECommerce'
```

**Problem**: You're referencing an element that doesn't exist.

**Fix:**

```sruja
// ❌ Wrong
ECommerce.API -> ECommerce.NonExistent "Calls"

// ✅ Correct - element exists
ECommerce.API -> ECommerce.ProductDB "Reads"
```

**Real-world scenario**: You renamed a service but forgot to update all references.

### Error 2: Duplicate ID

**Error message:**

```
❌ Duplicate ID: 'API' found in system 'ECommerce'
   First occurrence: line 5
   Second occurrence: line 12
```

**Problem**: Two elements have the same ID in the same scope.

**Fix:**

```sruja
element system
element container

// EXPECTED_FAILURE: unexpected token
// ❌ Wrong
ECommerce = system "E-Commerce" {
API = container "REST API"
API = container "GraphQL API"  // Duplicate ID!
}

// ✅ Correct - use unique IDs
ECommerce = system "E-Commerce" {
RESTAPI = container "REST API"
GraphQLAPI = container "GraphQL API"
}
```

**Real-world scenario**: You added a new API type but used the same ID.

### Error 3: Orphan Element

**Warning message:**

```
⚠️  Orphan element: ECommerce.Cache
   This element is not referenced by any relation
```

**Problem**: An element exists but nothing connects to it.

**Fix options:**

1. **Add a relation** (if the element should be used):

```sruja
// Add relation to use the cache
ECommerce.API -> ECommerce.Cache "Reads cache"
```

2. **Remove the element** (if it's not needed):

```sruja
// Remove if not part of current architecture
// datastore Cache "Cache" { ... }
```

3. **Document why it's isolated** (if intentional):

```sruja
datastore Cache "Cache" {
    description "Future: Will be used for product catalog caching"
    metadata {
        status "planned"
    }
}
```

**Real-world scenario**: You added a component for future use but haven't integrated it yet.

### Error 4: Constraint Violation

**Error message:**

```
❌ Constraint violation: 'NoDirectDB' violated
   ECommerce.WebApp -> ECommerce.ProductDB "Direct database access"
   Constraint: Frontend containers cannot access databases directly
```

**Problem**: A constraint rule is being violated.

**Fix:**

```sruja
// EXPECTED_FAILURE: Invalid reference
// ❌ Wrong - violates constraint
ECommerce.WebApp -> ECommerce.ProductDB "Direct access"

// ✅ Correct - go through API
ECommerce.WebApp -> ECommerce.API "Calls API"
ECommerce.API -> ECommerce.ProductDB "Reads products"
```

**Real-world scenario**: Enforcing architectural standards (e.g., "no direct database access from frontend").

## Understanding Validation Messages

### Cycles Are Valid

Sruja detects cycles but **doesn't block them** - cycles are valid architectural patterns:

- **Feedback loops**: User ↔ System interactions
- **Event-driven**: Service A ↔ Service B via events
- **Mutual dependencies**: Microservices that call each other
- **Bidirectional flows**: API ↔ Database (read/write)

```sruja
element person
element system

// ✅ Valid - feedback loop
User = person "User"
Platform = system "Platform"
User -> Platform "Makes request"
Platform -> User "Sends response"

// ✅ Valid - event-driven pattern
ServiceA = system "Service A"
ServiceB = system "Service B"
ServiceA -> ServiceB "Publishes event"
ServiceB -> ServiceA "Publishes response event"

// ✅ Valid - mutual dependencies
PaymentService = system "Payment Service"
OrderService = system "Order Service"
PaymentService -> OrderService "Updates order status"
OrderService -> PaymentService "Requests payment"

view index {
include *
}
```

The validator will **inform** you about cycles but won't prevent compilation, as they're often intentional.

### Simplicity Guidance

Sruja suggests simpler syntax when appropriate:

**Example:**

```
ℹ️  Simplicity suggestion: Consider using 'system' instead of nested 'container'
   Current: system App { container Web { ... } }
   Simpler: system Web { ... }
```

This is **informational only** - use the level of detail that matches your modeling goal.

## CI/CD Integration

### GitHub Actions Example

Add validation to your CI pipeline:

```yaml
name: Validate Architecture

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Sruja
        run: |
          curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
          export PATH="$HOME/go/bin:$PATH"

      - name: Lint Architecture
        run: |
          sruja lint architecture.sruja

      - name: Export Validation Report
        if: always()
        run: |
          sruja lint --json architecture.sruja > lint-report.json

      - name: Upload Report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: lint-report
          path: lint-report.json
```

### GitLab CI Example

```yaml
validate-architecture:
  image: golang:1.21
  script:
    - curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
    - export PATH="$HOME/go/bin:$PATH"
    - sruja lint architecture.sruja
  only:
    - merge_requests
    - main
```

### Pre-commit Hook

Validate before every commit:

```bash
#!/bin/sh
# .git/hooks/pre-commit

sruja lint architecture.sruja
if [ $? -ne 0 ]; then
    echo "❌ Architecture validation failed. Fix errors before committing."
    exit 1
fi
```

## Advanced: Custom Validation Rules

Use constraints and conventions for custom validation:

```sruja
element person
element system
element container
element component
element datastore
element queue

// Define constraint
constraint NoDirectDB "Frontend cannot access databases directly" {
    description "All database access must go through API layer"
}

// Apply convention
convention LayeredArchitecture {
    rule "Frontend → API → Database"
}

system Platform {
    Frontend = container "React App" {
        // This will be validated
    }
    API = container "REST API"
    DB = datastore "PostgreSQL"

    // ✅ Valid
    Frontend -> API "Calls API"
    API -> DB "Reads/Writes"

    // ❌ Will be caught by validator
    // Frontend -> DB "Direct access"  // Violates constraint
}

view index {
include *
}
```

## Real-World Workflow

### Step 1: Write Architecture

```sruja
element person
element system
element container
element component
element datastore
element queue

system App {
    container Web
    datastore DB
}

view index {
include *
}
```

### Step 2: Validate

```bash
sruja lint architecture.sruja
```

### Step 3: Fix Errors

Address any validation errors or warnings.

### Step 4: Commit to CI/CD

Once validation passes locally, commit. CI/CD will validate again.

### Step 5: Monitor in Production

Use validation in CI/CD to catch issues before they reach production.

## Key Takeaways

1. **Validate early and often**: Run `sruja lint` frequently during development
2. **Fix errors immediately**: Don't accumulate validation debt
3. **Integrate with CI/CD**: Catch issues before they reach production
4. **Understand cycles**: They're often valid patterns, not errors
5. **Use constraints**: Enforce architectural standards automatically

## Exercise: Fix Validation Errors

**Scenario**: You have an architecture file with several validation errors.

**Tasks:**

1. Run `sruja lint` on a file
2. Identify all errors and warnings
3. Fix each error
4. Re-validate to confirm fixes

**Time**: 10 minutes

## Further Reading

- Docs: [Validation Concepts](/docs/concepts/validation)
- Tutorial: [CLI Basics](/tutorials/basic/cli-basics)
- Course: [System Design 101 - Module 4: Production Readiness](/courses/system-design-101/module-4-production-readiness)
