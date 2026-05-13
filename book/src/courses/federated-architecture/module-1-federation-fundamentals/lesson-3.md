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
sruja export repo.sruja --format mermaid

# Export to JSON for custom tooling
sruja export repo.sruja --format json
```

## Validation and Health Checks

```bash
# Check overall health
sruja health -r .

# Check for architectural drift
sruja drift -r . -a repo.sruja

# In CI mode (GitHub Actions format)
sruja drift --ci -r . -a repo.sruja
```

## Using in AI Editors

Load the system index for cross-repo context:

```bash
# Build AI-ready context from multiple repos
sruja ai-context -r repoA -r repoB -r repoC

# Or use context-score to measure AI-readiness
sruja context-score -r .
```

## Basic Workflow

```
1. Team publishes bundle (on every main push)
   └─→ repo.bundle.json in shared location

2. CI collects all bundles
   └─→ ./bundles/*.bundle.json

3. Compose on schedule or on-demand
   └─→ system.index.json

4. Use for:
   └─→ AI editor context (sruja ai-context)
   └─→ Impact analysis (sruja impact)
   └─→ Compliance reporting (sruja compliance)
```

## Module Complete!

You've completed Federation Fundamentals. You now understand:
- ✅ What federation is and when to use it
- ✅ How to publish architecture bundles
- ✅ How to compose bundles into system index
- ✅ Basic cross-repo relationship modeling

## Hands-On: Compose Your First System Index

1. **Create bundles from multiple repos:**
   ```bash
   # Create demo bundles directory
   mkdir -p ./bundles
   
   # Publish from each service
   sruja publish -r ./user-service -o ./bundles/user-service.bundle.json
   sruja publish -r ./order-service -o ./bundles/order-service.bundle.json
   sruja publish -r ./payment-service -o ./bundles/payment-service.bundle.json
   ```

2. **Compose bundles into a system index:**
   ```bash
   sruja compose -i ./bundles -o system.index.json
   ```

3. **Explore the composed system index:**
   ```bash
   cat system.index.json | jq '{repo_count: (.repos | length), node_count: (.nodes | length), edge_count: (.edges | length)}'
   ```

4. **View canonical IDs in the system:**
   ```bash
   cat system.index.json | jq '.nodes[].canonical_id'
   ```

## Learning Outcomes

- ✅ Compose multiple bundles into a unified system index
- ✅ Understand the system index structure (repos, nodes, edges, conflicts)
- ✅ Use canonical IDs to reference components across repositories
- ✅ Export system index to different formats (Mermaid, JSON)
- ✅ Apply federation workflow in CI/CD pipelines

## Quiz: Test Your Understanding

### Q1: What does the `conflicts` array in a system index contain?

A) Git merge conflicts between repos
B) Duplicate elements across repos that need resolution
C) Network connectivity errors
D) Authentication failures

### Q2: What is a canonical ID in federation?

A) A Git commit hash
B) A Docker container identifier
C) A unique identifier in the format `repo_id::local_id` that prevents naming collisions
D) A UUID for each user

### Q3: Which command is used to compose multiple bundles into a system index?

A) `sruja merge`
B) `sruja compose -i <input> -o <output>`
C) `sruja index`
D) `sruja aggregate`

Next, Module 2 covers advanced cross-repo relationships.