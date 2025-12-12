---
title: "Lesson 5: Change & Snapshots"
weight: 5
summary: "Track evolution and version states within the architecture."
---

# Lesson 5: Change & Snapshots

## Why Track Changes?
Keeping a history of changes improves communication, auditability, and onboarding.

## Sruja: Change and Snapshot

```sruja
// EXPECTED_FAILURE: course example uses change/snapshot inside architecture
architecture "E-Commerce Platform" {
  system Shop {
    container API
  }

  change "Add caching layer" {
    description "Introduce Redis for hot paths"
    affects [ Shop.API ]
  }

  snapshot v"2025.12" {
    note "Post‑Black Friday stabilization"
  }
}
```

## Practice
- Add a `change` describing an API refactor.
- Create a `snapshot` with the current version tag.
