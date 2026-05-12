---
title: "Lesson 3: Basic Composition"
weight: 3
summary: "Compose bundles into a unified system index."
---

# Lesson 3: Basic Composition

## Composing Bundles

Once you have bundles published, compose them:

```bash
# Compose from a directory
sruja compose -i ./bundles -o system.index.json

# Or from individual bundles
sruja compose -i ./user.bundle.json -i ./order.bundle.json -o system.index.json
```

## System Index Structure

```json
{
  "schema_version": "1",
  "repos": [
    {
      "repo_id": "user-service",
      "repo_path": "/path/to/user-service",
      "truth_status": "reviewed",
      "git_commit": "a1b2c3d"
    },
    {
      "repo_id": "order-service",
      "repo_path": "/path/to/order-service",
      "truth_status": "reviewed",
      "git_commit": "x9y8z7"
    }
  ],
  "nodes": [
    {
      "canonical_id": "user-service::UserAPI",
      "repo_id": "user-service",
      "local_id": "UserAPI",
      "kind": "container",
      "label": "User API"
    },
    {
      "canonical_id": "order-service::OrderProcessor",
      "repo_id": "order-service",
      "local_id": "OrderProcessor",
      "kind": "container",
      "label": "Order Processor"
    }
  ],
  "edges": [
    {
      "source": "order-service::OrderProcessor",
      "target": "user-service::UserAPI",
      "label": "REST",
      "repo_id": "order-service"
    }
  ],
  "conflicts": []
}
```

## Cross-Repo Relationships

In your Sruja DSL, reference other repos using canonical IDs:

```sruja
// In order-service/repo.sruja
import { user-service } from "https://bundles.company.com/user-service.bundle.json"

OrderService = system "Order Service" {
  OrderProcessor = container "Order Processor"

  // Reference user-service component
  OrderProcessor -> user-service.UserAPI "Calls for customer info"
}
```

## Visualizing the System Index

```bash
# Export to Mermaid
sruja export -i system.index.json --format mermaid

# Export to JSON for custom tooling
sruja export -i system.index.json --format json
```

## Validation and Health Checks

```bash
# Check overall federated health
sruja health -i system.index.json

# Check for conflicts
sruja compose -i ./bundles -o system.index.json --check-conflicts

# Show drift across repos
sruja drift -i system.index.json
```

## Using in AI Editors

Load the system index for cross-repo context:

```bash
# Build AI-ready context from index
sruja context -i system.index.json -o ai-context.json

# Or use directly with focus
sruja focus --component order-service::OrderProcessor -i system.index.json
```

## Basic Workflow

```
1. Team publishes bundle (on every main push)
   └─→ repo.bundle.json in shared location

2. CI collects all bundles
   └─→ ./bundles/*.bundle.json

3. Compose on schedule or on-demand
   └─→ system.index.json

4. Use system index for:
   └─→ AI editor context
   └─→ Cross-repo diagrams
   └─→ Impact analysis
   └─→ Compliance reporting
```

## Module Complete!

You've completed Federation Fundamentals. You now understand:
- ✅ What federation is and when to use it
- ✅ How to publish architecture bundles
- ✅ How to compose bundles into system index
- ✅ Basic cross-repo relationship modeling

Next, Module 2 covers advanced cross-repo relationships.