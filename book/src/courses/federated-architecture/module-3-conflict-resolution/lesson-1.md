---
title: "Lesson 1: Detecting Conflicts"
weight: 1
summary: "Find duplicate services and elements in federated architecture."
---

# Lesson 1: Detecting Conflicts

## What Causes Conflicts?

Conflicts occur when:
- Two repos model the same logical element
- Teams use overlapping names
- Services get duplicated during acquisitions
- Monoliths are split inconsistently

## Conflict Types

| Type | Example | Impact |
|------|---------|--------|
| **Same ID** | `UserService::API` in two repos | Cannot compose |
| **Same Label** | Two containers labeled "API" | Ambiguous in diagrams |
| **Same Kind+Label** | Two systems called "Payment" | Merge confusion |
| **Circular Import** | A imports B, B imports A | Composition fails |

## Conflict Detection in Compose

```bash
# Compose and detect conflicts
sruja compose -i ./bundles -o system.index.json --check-conflicts
```

The compose command outputs:

```json
{
  "conflicts": [
    {
      "type": "duplicate_kind_label",
      "kind": "container",
      "label": "API",
      "repos": ["user-service", "order-service"],
      "canonical_ids": [
        "user-service::API",
        "order-service::API"
      ]
    },
    {
      "type": "duplicate_component",
      "canonical_id": "auth-service::AuthAPI",
      "also_in": ["legacy-auth::AuthAPI"],
      "reason": "Same logical service"
    }
  ]
}
```

## When Conflicts Block Composition

Some conflicts are blocking:
- Same canonical ID
- Circular dependencies

Others are warnings:
- Same label (can disambiguate with repo prefix)
- Similar components (may need human review)

## Running Pre-Composition Check

```bash
# Check before composing
sruja compose -i ./bundles --dry-run

# Shows what conflicts would occur
# without actually creating system.index.json
```

## Conflict Report

```bash
# Generate detailed conflict report
sruja compose -i ./bundles -o system.index.json --conflict-report

# View report
cat conflicts-2024-05-12.json
```

## Next Steps

Lesson 2 covers resolution strategies.