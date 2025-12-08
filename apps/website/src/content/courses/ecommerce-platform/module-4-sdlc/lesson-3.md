---
title: "Lesson 3: CI/CD Pipeline (Validation)"
weight: 3
summary: "Automating architectural checks in your pipeline."
---

# Lesson 3: CI/CD Pipeline

Architecture compliance shouldn't be a manual review process. It should be a build step.

## The Pipeline
In your GitHub Actions or Jenkins pipeline, add a step to install and run Sruja.

```yaml
steps:
  - name: Checkout
    uses: actions/checkout@v3

  - name: Install Sruja
    run: go install github.com/sruja-ai/sruja/cmd/sruja@latest

  - name: Validate Architecture
    run: sruja validate architecture/
```

## Breaking the Build
If a developer introduces a violation (e.g., "Frontend talks directly to Database"), `sruja validate` will exit with a non-zero code, failing the build.

This is **Governance as Code**. You stop architectural drift before it merges.
