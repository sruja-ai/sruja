---
title: "Change and Snapshot"
weight: 60
summary: "Track architecture evolution and capture versioned snapshots."
---

# Change and Snapshot

Use `change` to describe modifications; use `snapshot` to capture versioned states.

## Syntax

```sruja
// EXPECTED_FAILURE: change/snapshot inside architecture parsing pending harmonization
architecture "Shop" {
  system App {
    container API
    datastore DB
  }

  change "Add caching layer" {
    description "Introduce Redis for hot paths"
    affects [ App.API ]
  }

  snapshot v"2025.12" {
    note "Post‑Black Friday stabilization"
  }
}
```

## Guidance
- Keep `change` titles action‑oriented; include `affects` where relevant.
- Use `snapshot` with clear version strings and a short note.
- Link ADRs to `change` items for rationale.

## Related
- `adr` for decisions
- `deployment` when changes affect runtime topology
