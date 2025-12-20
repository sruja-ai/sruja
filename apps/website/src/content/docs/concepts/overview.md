---
title: "Overview"
weight: 12
summary: "Summarize systems with high‑level context for readers and diagrams."
---

# Overview

Use `overview` to provide a concise system description shown in docs/exports.

## Syntax

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  overview {
    title "E‑Commerce Platform"
    summary "Web, API, and DB supporting browse, cart, and checkout"
  }
}

views {
  view index {
    include *
  }
}
```

## Guidance
- Keep summary short and practical; avoid marketing language.
- Use `overview` at architecture root; prefer `description` inside elements for details.

