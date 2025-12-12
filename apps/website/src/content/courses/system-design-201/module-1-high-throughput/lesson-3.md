---
title: "Lesson 3: Views for Critical Throughput Paths"
weight: 3
summary: "Use views to isolate and analyze high‑volume flows."
---

# Lesson 3: Views for Critical Throughput Paths

## Why Views for Throughput?
Focus on hot paths to reason about scaling, backpressure, and caching.

## Sruja: High‑Throughput View

```sruja
architecture "Streaming" {
  system Pipeline {
    container Ingest
    container Processor
    datastore Events
  }

  Ingest -> Processor "Process"
  Processor -> Events "Store"

  views {
    container Pipeline "Hot Path" {
      include Ingest Processor Events
    }
  }
}
```

## Practice
- Create a view highlighting backpressure points.
- Annotate scale bounds for hot components.

