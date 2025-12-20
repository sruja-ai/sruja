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
specification {
  element system
  element container
  element datastore
  element queue
}

model {
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
    
    Events = datastore "Event Store" {
      technology "Kafka"
      description "Buffers events for processing"
    }
    
    OutputDB = datastore "Output Database" {
      technology "ClickHouse"
      description "Stores processed events"
    }

    Ingest -> Events "Consumes"
    Events -> Processor "Streams"
    Processor -> OutputDB "Writes"
  }
}

views {
  // Complete system view
  view index {
    title "Complete Pipeline"
    include *
  }
  
  // Hot path view: Focus on critical throughput path
  view hotpath {
    title "Hot Path - Throughput Analysis"
    include Pipeline.Ingest Pipeline.Events Pipeline.Processor
    exclude Pipeline.OutputDB
    description "Isolates the critical path for throughput analysis"
  }
  
  // Backpressure view: Components that can cause bottlenecks
  view backpressure {
    title "Backpressure Points"
    include Pipeline.Events Pipeline.Processor
    exclude Pipeline.Ingest Pipeline.OutputDB
    description "Identifies where backpressure can occur"
  }
  
  // Scale view: Components with scaling configuration
  view scale {
    title "Scaling Configuration"
    include Pipeline.Ingest Pipeline.Processor
    exclude Pipeline.Events Pipeline.OutputDB
    description "Shows components with auto-scaling configured"
  }
}
```

## Practice
- Create a view highlighting backpressure points.
- Annotate scale bounds for hot components.
- Use scenarios to model high-volume flows.

