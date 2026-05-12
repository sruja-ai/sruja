---
title: "Lesson 2: Resolution Strategies"
weight: 2
summary: "Approaches to resolving architectural conflicts."
---

# Lesson 2: Resolution Strategies

## Strategy 1: Manual Resolution

Human decides which element to use:

```bash
# Mark one as canonical
sruja resolve --canonical "user-service::API" \
  --alias "order-service::API" \
  --to "user-service::API"
```

This updates the system index:

```json
{
  "resolved": {
    "order-service::API": "user-service::API"
  }
}
```

## Strategy 2: Aliasing

Keep both but disambiguate:

```bash
# Add repo prefix to one
sruja resolve --alias "order-service::API" \
  --rename "order-service-order-api"
```

Result in DSL:

```sruja
// Both preserved, now distinct
user-service::API
order-service::order-api
```

## Strategy 3: Merging

Combine two elements:

```bash
# Merge into single canonical
sruja resolve --merge "user-service::API" \
  --with "order-service::API" \
  --canonical "unified::API" \
  --ownership "shared-team"
```

This requires:
- Merging DSL definitions
- Updating all references
- Defining new ownership

## Strategy 4: Deprecation

Mark one as deprecated:

```bash
# Deprecate duplicate
sruja resolve --deprecate "legacy-auth::AuthAPI" \
  --reason "Replaced by auth-service::AuthAPI" \
  --migrate-to "auth-service::AuthAPI"
```

Deprecated elements:
- Still visible in history
- Flagged in reports
- Block new usage

## Resolution Workflow

```
1. Detect conflict
   └─→ sruja compose --check-conflicts

2. Analyze options
   └─→ Is this same service? → MERGE
   └─→ Different services with same name? → ALIAS
   └─→ One is legacy? → DEPRECATE
   └─→ Unclear? → MANUAL REVIEW

3. Apply resolution
   └─→ sruja resolve [strategy]

4. Verify composition
   └─→ sruja compose -i ./bundles -o system.index.json

5. Update affected repos
   └─→ Update references in DSL
```

## Handling Blocked Composition

If composition fails:

```bash
# Show blocking conflicts
sruja compose -i ./bundles --show-blockers

# Blockers must be resolved before proceeding
```

## Next Steps

Lesson 3 covers preventing future conflicts.