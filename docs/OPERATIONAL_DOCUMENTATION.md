# Operational Documentation & FAANG-Level Exports

Sruja goes beyond simple diagrams by allowing you to define "Operational Readiness" attributes directly in your architecture-as-code. These attributes are then automatically exported into comprehensive Markdown reports suitable for high-level architectural reviews, audits, and runbooks.

## Enabling FAANG Exports

The Sruja Markdown exporter automatically detects specific **Metadata** and **Properties** to generate advanced sections like "Failure Modes", "Capacity Planning", and "Risk Assessment".

### 1. Executive Summary & KPIs

Define high-level KPIs in the `metadata` block of your `architecture`.

```sruja
architecture "My Platform" {
  metadata {
    "scale" "10M+ users, 1B+ requests/month"
    "availability" "99.99% uptime SLA"
    "performance" "<50ms p95 latency for core APIs"
    "cost" "$15k/month operational cost"
    "highRisk" "Single region dependency (us-east-1)"
  }
}
```

### 2. Failure Modes & Recovery (Runbooks)

Define failure scenarios for your `Systems` or `Containers`. If omitted, Sruja attempts to generate intelligent defaults based on system type.

```sruja
system PaymentService "Payment Processing" {
  metadata {
    "failure.impact" "Users cannot complete checkout. Revenue loss."
    "failure.detection" "Composite alarm: ErrorRate > 1% AND Latency > 1s"
    "failure.mitigation" "Auto-circuit breaker opens. Fallback to cached payment methods."
    "failure.recovery" "RTO: 5 mins. Steps: 1. Verify upstream. 2. Restart pods."
    "failure.fallback" "Queue orders for manual processing."
  }
}
```

### 3. Capacity Planning

Use the `deployment` node to define capacity.

```sruja
deployment Production "Production AWS" {
  node Database "RDS Primary" {
    containerInstance DB
    metadata {
      "capacity" "db.r6g.4xlarge (Storage: 2TB)"
      "scaling" "Vertical scaling allowed. Read replicas handling read traffic."
    }
  }
}
```

### 4. Compliance & Data Lifecycle

Define compliance status and data policies.

```sruja
architecture "My Platform" {
  metadata {
    "pciDss" "Level 1 Certified"
    "gdpr" "Compliant (Data Residency: EU-West-1)"
    "dataRetention" "7 years for financial records (S3 Glacier Lock)"
    "dataDeletion" "Automated pipeline (30 day SLA)"
  }
}
```

## Generated Sections

When you run `sruja export markdown my_arch.sruja`, it will generate:

*   **Executive Summary**: A concise 1-page view for stakeholders.
*   **Failure Modes**: A table or list of how systems fail and recover (Runbook).
*   **Capacity Planning**: Current infrastructure and scaling limits.
*   **Dependency Risks**: Highlighting external vendors and internal SPOFs.
*   **Compliance Matrix**: Quick view of regulatory status.
*   **Cost Analysis**: Breakdown of operational costs.

## Best Practices (Dogfooding)

We recommend treating this metadata as part of your **Definition of Done** for architectural changes.
*   **Architects**: fill in "Risk" and "Scale".
*   **SREs**: fill in "Failure Modes" and "Capacity".
*   **Compliance**: fill in "Data Retention" and "Compliance".
