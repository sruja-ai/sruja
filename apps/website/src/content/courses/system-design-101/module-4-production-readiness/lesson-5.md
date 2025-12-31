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
element system
element container
element datastore

Shop = system "E-Commerce Shop" {
API = container "API Service" {
  technology "Go"

  // SLOs tracked over time
  slo {
    availability {
      target "99.9%"
      window "30 days"
      current "99.85%"
      description "Targeting 99.9% after caching improvements"
    }
    latency {
      p95 "200ms"
      p99 "500ms"
      window "7 days"
      current {
        p95 "250ms"
        p99 "600ms"
      }
      description "Latency improved after adding cache"
    }
    errorRate {
      target "< 0.1%"
      window "30 days"
      current "0.12%"
      description "Error rate slightly above target, monitoring"
    }
  }
}

Cache = datastore "Redis Cache" {
  technology "Redis"
  description "Added to improve latency SLO"
}

Database = datastore "PostgreSQL" {
  technology "PostgreSQL"
}

API -> Cache "Reads"
API -> Database "Reads/Writes"
}

// Track changes that affect SLOs
change "Add caching layer" {
description "Introduce Redis for hot paths to improve latency SLO"
affects [ Shop.API Shop.Cache ]
date "2025-01-15"
}

change "Optimize database queries" {
description "Reduce p95 latency from 300ms to 250ms"
affects [ Shop.API Shop.Database ]
date "2025-01-20"
}

// Snapshots capture SLO state at specific points
snapshot v"2025.01" {
note "Post‑Black Friday stabilization - SLOs met"
date "2025-01-31"
}

snapshot v"2025.02" {
note "After caching improvements - latency SLO improved"
date "2025-02-15"
}

view index {
title "Shop System with Change Tracking"
include *
}

// SLO evolution view
view slos {
title "SLO Tracking View"
include Shop.API Shop.Cache Shop.Database
description "Shows components with SLOs and their evolution"
}
```

## Practice

- Add a `change` describing an API refactor.
- Create a `snapshot` with the current version tag.
