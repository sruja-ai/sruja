---
title: "Lesson 1: Fitness Functions & Health Metrics"
weight: 1
summary: "Define and measure architectural fitness using Sruja fitness functions and health metrics."
---

# Lesson 1: Fitness Functions & Health Metrics

## What Are Fitness Functions?

A **fitness function** is a quantitative measure of how well a system achieves its architectural outcomes. Just as evolutionary biology uses fitness to measure adaptation, architectural fitness functions measure how well your system meets its non-functional requirements.

## Why Fitness Functions?

Traditional architecture focuses on *design*—but what matters is *outcomes*. Fitness functions shift the conversation from "is the design correct?" to "does the system actually achieve what it set out to achieve?"

**Example fitness functions:**
- `availability > 99.9%`
- `latency_p99 < 200ms`
- `error_rate < 0.1%`
- `cpu_utilization < 80%`

## Defining Fitness in Sruja

A **fitness** is a top-level declaration that defines an outcome you want to measure:

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

fitness LatencyTarget {
  target "p99_response_time < 200ms"
  measure "scripts/measure_latency.sh"
}

fitness AvailabilityTarget {
  target "uptime > 99.9%"
  measure "scripts/check_availability.sh"
}

fitness ThroughputTarget {
  target "requests_per_second > 10000"
  measure "scripts/measure_throughput.sh"
}
```

Fitness functions can also be defined inside elements:

```sruja
<!-- partial -->
EcommercePlatform = system "E-Commerce Platform" {
  fitness checkout_success_rate {
    target "success_rate > 99.5%"
    measure "scripts/evaluate_checkout.sh"
  }
}
```

## Health Metrics Dashboard

From v0.42.0, Sruja provides integrated health metrics:

```bash
sruja health -r .
```

This command displays:
- **Graph Health Score**: Composite score based on architectural quality
- **Community Detection**: Identifies tightly coupled components
- **Trend Analysis**: Health over time
- **Threshold Violations**: Areas requiring attention

## Hands-On: Define Fitness for a Sample System

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

fitness response_time {
  target "p99_response_time < 150ms"
  measure "scripts/measure_api_latency.sh"
}

fitness checkout_success_rate {
  target "success_rate > 99.5%"
  measure "scripts/evaluate_checkout.sh"
}

fitness cache_hit_ratio {
  target "cache_hit_ratio > 90%"
  measure "scripts/check_redis_stats.sh"
}

EcommercePlatform = system "E-Commerce Platform" {
  API = container "REST API" {
    technology "Node.js"
    description "Handles all API requests"
  }

  DB = database "PostgreSQL" {
    technology "PostgreSQL"
    description "Primary data store"
  }

  Cache = container "Redis Cache" {
    technology "Redis"
    description "In-memory cache for hot data"
  }

  PaymentProcessor = container "Payment Processor" {
    technology "Stripe"
    description "Handles payment processing"
  }
}

User = person "Customer"
User -> EcommercePlatform.API "Browses products"
EcommercePlatform.API -> EcommercePlatform.Cache "Cache lookups"
EcommercePlatform.API -> EcommercePlatform.DB "Persistent storage"
EcommercePlatform.API -> EcommercePlatform.PaymentProcessor "Process payments"
```

## Learning Outcomes

- ✅ Understand what fitness functions are and why they matter
- ✅ Define fitness functions in Sruja DSL
- ✅ Use `sruja health` to view health metrics
- ✅ Apply fitness functions to real architectural scenarios

## Quiz: Test Your Understanding

### Q1: What is a fitness function in architecture?

A) A function that optimizes code performance
B) A quantitative measure of how well a system achieves its architectural outcomes
C) A database query function
D) A network monitoring tool

### Q2: What does `sruja health` display?

A) Server uptime statistics
B) Graph Health Score, community detection, and threshold violations
C) Network latency metrics
D) Database query performance

### Q3: Why are fitness functions important?

A) They make code run faster
B) They measure if the system actually achieves its intended outcomes, not just if the design looks correct
C) They encrypt data
D) They manage users

## Next Steps

In Lesson 2, we'll explore how to interpret health scores and configure thresholds.