---
title: "Lesson 2: Publishing Bundles"
weight: 2
summary: "Create and distribute repo architecture bundles."
---

# Lesson 2: Publishing Bundles

## Publishing Your First Bundle

```bash
# Navigate to your repo
cd /path/to/user-service

# Publish the architecture bundle
sruja publish -r . -o repo.bundle.json
```

This creates `repo.bundle.json` with:
- Architecture DSL snapshot
- Context graph
- Truth status
- Git commit reference

## Bundle Structure

```json
{
  "schema_version": "1",
  "repo_id": "user-service",
  "repo_path": "/path/to/user-service",
  "git_commit": "a1b2c3d",
  "baseline_path": "repo.sruja",
  "baseline_dsl": "UserService = system \"User Service\" { ... }",
  "context": {
    "components": [...],
    "edges": [...]
  },
  "truth_status": "reviewed",
  "updated_at": "2024-05-12T10:30:00Z"
}
```

## Publishing to a Shared Location

For federation, publish to a shared location:

```bash
# Publish to shared bundles directory
sruja publish -r ./user-service -o ./shared/bundles/user-service.bundle.json
sruja publish -r ./order-service -o ./shared/bundles/order-service.bundle.json
sruja publish -r ./payment-service -o ./shared/bundles/payment-service.bundle.json
```

## Bundle Versioning

Bundles are tied to git commits:

```bash
# Publish with specific version
sruja publish -r . -o repo.bundle.json --version 1.0.0

# Publish from specific commit
git checkout v1.0.0
sruja publish -r . -o repo.bundle.json
```

## Automated Publishing in CI/CD

```yaml
# .github/workflows/publish-bundle.yml
name: Publish Architecture Bundle

on:
  push:
    branches: [main]

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Sruja
        run: curl -fsSL https://sruja.ai/install.sh | bash

      - name: Publish Bundle
        run: |
          sruja publish -r . -o bundle.json

      - name: Upload Bundle
        uses: actions/upload-artifact@v4
        with:
          name: architecture-bundle
          path: bundle.json
```

## Refreshing Bundles

```bash
# Re-publish with latest context
sruja publish -r . -o repo.bundle.json --force

# Or run sruja sync first to update context
sruja sync -r .
sruja publish -r . -o repo.bundle.json
```

## Bundle Contents Deep Dive

### context.components

```json
{
  "id": "UserAPI",
  "kind": "container",
  "label": "User API",
  "description": "REST API for user management",
  "technology": "Node.js",
  "repo_id": "user-service"
}
```

### context.edges

```json
{
  "source": "user-service::UserAPI",
  "target": "user-service::UserDB",
  "label": "SQL"
}
```

## Best Practices

| Practice | Why |
|----------|-----|
| Publish from main branch | Ensures consistency |
| Automate in CI/CD | Keeps bundles fresh |
| Use semantic repo IDs | Avoids collisions |
| Store in shared location | Enables composition |
| Track truth_status | Shows drift |

## Next Steps

Lesson 3 covers composing bundles into a system index.