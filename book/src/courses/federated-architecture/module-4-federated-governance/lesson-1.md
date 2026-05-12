---
title: "Lesson 1: Federated Policies"
weight: 1
summary: "Define policies that apply across all repositories."
---

# Lesson 1: Federated Policies

## The Challenge: Org-Wide Governance

You need to enforce:
- All services use HTTPS
- PII must be encrypted
- No direct database access from frontend
- All APIs must have SLAs defined

But you don't control all repos!

## Solution: Federated Policies

Define policies once, apply everywhere:

```sruja
// In federation-root/policies/global.sruja
policy "Global Security" {
  description "Security policies for all services"

  constraint "All external endpoints must use HTTPS" {
    applies_to: container
    where: container.external == true
  }

  constraint "All databases must have encryption" {
    applies_to: database
  }

  constraint "No PII in logs" {
    applies_to: container
    rule: "container must have pii_handling policy"
  }
}

policy "Global Observability" {
  description "Observability requirements"

  constraint "All services must expose /health" {
    applies_to: container
  }

  constraint "All services must expose /metrics" {
    applies_to: container
  }
}
```

## Policy Inheritance

Repos inherit federated policies:

```bash
# In repo, inherit from federation
sruja policy inherit ../federation/policies/global.sruja
```

Repos can still add local policies:

```sruja
// In user-service/policies/local.sruja
policy "User Service Specific" {
  description "User service specific policies"

  constraint "User emails must be verified" {
    applies_to: container
    where: container.type == "user-management"
  }
}
```

## Policy Composition

Combine federated and local:

```bash
# Validate with both
sruja validate -r . --policies \
  ../federation/policies/global.sruja \
  ./policies/local.sruja
```

## Policy Overrides

Repos can override (with approval):

```sruja
// In user-service/policies/override.sruja
policy "Override: Extended Timeout" {
  description "Override: Allow 60s timeout for batch jobs"
  inherits: "Global Observability.timeout"

  overrides: true
  approved_by: "architecture-board"

  constraint "Batch containers may have 60s timeout" {
    applies_to: container
    where: container.type == "batch"
  }
}
```

## Next Steps

Lesson 2 covers generating compliance reports.