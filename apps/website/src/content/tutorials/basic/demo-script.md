---
title: "Demo Script"
weight: 5
summary: "10‑minute walkthrough: model, validate, and export."
tags: ["demo", "getting-started", "walkthrough"]
---

# Demo Script: Quick 10-Minute Walkthrough

This tutorial provides a quick 10-minute walkthrough to demonstrate Sruja's core capabilities: modeling, validation, and export.

## 1) Model (2 minutes)

Create a simple e-commerce architecture:

```sruja
element person
element system
element container
element component
element datastore
element queue

person User
system Shop {
container WebApp
container API
datastore DB
}

User -> Shop.WebApp "Uses"
Shop.WebApp -> Shop.API "Calls"
Shop.API -> Shop.DB "Reads/Writes"

view index {
include *
}
```

## 2) Validate (2 minutes)

Format and validate your model:

```bash
sruja fmt architecture.sruja
sruja lint architecture.sruja
```

## 3) Add Targets (3 minutes)

Add SLOs and scaling configuration:

```sruja
element system
element container

Shop = system "Shop" {
API = container "API" {
  scale {
    metric "req/s"
    min 200
    max 2000
  }

  slo {
    availability {
      target "99.9%"
      window "30 days"
    }
    latency {
      p95 "200ms"
      window "7 days"
    }
    errorRate {
      target "< 0.1%"
      window "30 days"
    }
  }
}
}

view index {
include *
}
```

## 4) Export (3 minutes)

Export to various formats:

```bash
sruja export markdown architecture.sruja
sruja export mermaid architecture.sruja
sruja export svg architecture.sruja
```

**Outcome**: Living docs and diagrams generated from the model.

---

**Note**: Sruja is **free and open source** (Apache 2.0 licensed). Need help with adoption? Professional consulting services are available. Contact the team through [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions) to learn more.
