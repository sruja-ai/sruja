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
   └─→ sruja compose -i ./bundles -o system.index.json

2. Analyze options
   └─→ Same service in multiple repos? → MERGE or RENAME
   └─→ Different services with same name? → USE ALIASES
   └─→ One is legacy? → DEPRECATE and remove

3. Update DSL in affected repos
   └─→ Rename conflicting components
   └─→ Update references

4. Re-compose and verify
   └─→ sruja compose -i ./bundles -o system.index.json

5. Validate no new conflicts
   └─→ sruja lint repo.sruja
```

## Handling Blocked Composition

If composition detects conflicts:

```bash
# View the conflicts in the system index
cat system.index.json | jq '.conflicts'

# Resolve by updating DSL to use unique names
# Then republish and recompose
sruja publish -r . -o updated-bundle.json
```

## Hands-On: Resolve a Conflict

1. **Compose and detect conflicts:**
   ```bash
   sruja compose -i ./bundles -o system.index.json
   cat system.index.json | jq '.conflicts'
   ```

2. **Identify the conflicting components:**
   ```bash
   # Look for duplicate kind+label
   cat system.index.json | jq '.nodes[] | select(.label == "API")'
   ```

3. **Rename to resolve (example):**
   In `order-service/repo.sruja`, rename `API` to `OrderAPI`:
   ```sruja
   OrderAPI = container "Order API" {
     description "API for order management"
   }
   ```

4. **Republish and recompose:**
   ```bash
   sruja publish -r ./order-service -o ./bundles/order-service.bundle.json
   sruja compose -i ./bundles -o system.index.json
   ```

## Learning Outcomes

- ✅ Understand different conflict resolution strategies
- ✅ Detect conflicts during bundle composition
- ✅ Apply renaming or aliasing to resolve conflicts
- ✅ Validate resolution by recomposing bundles

## Quiz: Test Your Understanding

### Q1: What typically causes conflicts in federated architecture?

A) Network timeouts
B) Two repos modeling the same logical element or using the same name
C) Git merge conflicts
D) Database lock contention

### Q2: What is the first step when you detect a conflict in composition?

A) Delete one of the bundles
B) Analyze whether the elements are actually the same or just have similar names
C) Contact the cloud provider
D) Disable CI/CD

### Q3: After resolving a conflict, what should you do?

A) Nothing - conflicts auto-resolve
B) Republish affected bundles and recompose
C) Delete the system index
D) Restart all services

## Next Steps

Lesson 3 covers preventing future conflicts.