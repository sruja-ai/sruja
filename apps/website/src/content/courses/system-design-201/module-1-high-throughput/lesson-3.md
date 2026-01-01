---
title: "Lesson 3: Views for Critical Throughput Paths"
weight: 3
summary: "Use views to isolate and analyze high‑volume flows."
---

# Lesson 3: Views for Critical Throughput Paths

## Why Views for Throughput?

Focus on hot paths to reason about scaling, backpressure, and caching. High-throughput systems have critical paths that need isolation for analysis.

## Sruja: High‑Throughput View

```sruja
import { * } from 'sruja.ai/stdlib'


Pipeline = system "Data Pipeline" {
Ingest = container "Ingestion Service" {
  technology "Kafka Consumer"
  scale {
    min 5
    max 50
    metric "lag > 1000"
  }
}

Processor = container "Processing Service" {
  technology "Go Workers"
  scale {
    min 10
    max 200
    metric "queue_depth > 5000"
  }
}

Events = database "Event Store" {
  technology "Kafka"
  description "Buffers events for processing"
}

OutputDB = database "Output Database" {
  technology "ClickHouse"
  description "Stores processed events"
}

Ingest -> Events "Consumes"
Events -> Processor "Streams"
Processor -> OutputDB "Writes"
}

// Complete system view
view index {
title "Complete Pipeline"
include *
}

// Hot path view: Focus on critical throughput path
view hotpath {
title "Hot Path - Throughput Analysis"
include Pipeline.Ingest
include Pipeline.Events
include Pipeline.Processor
exclude Pipeline.OutputDB
}

// Backpressure view: Components that can cause bottlenecks
view backpressure {
title "Backpressure Points"
include Pipeline.Events
include Pipeline.Processor
exclude Pipeline.Ingest
exclude Pipeline.OutputDB
}

// Scale view: Components with scaling configuration
view scale {
title "Scaling Configuration"
include Pipeline.Ingest
include Pipeline.Processor
exclude Pipeline.Events
exclude Pipeline.OutputDB
}
```

## Practice

- Create a view highlighting backpressure points.
- Annotate scale bounds for hot components.
- Use scenarios to model high-volume flows.
