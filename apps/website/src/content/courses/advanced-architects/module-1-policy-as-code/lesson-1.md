---
title: "Lesson 1: Writing Constraints and Conventions"
weight: 1
summary: "Codify architectural rules as executable constraints that prevent violations."
---

# Lesson 1: Writing Constraints and Conventions

## The Problem: Architectural Drift

As teams grow, architectural standards drift. Services violate boundaries, dependencies become circular, and compliance requirements are missed. Manual reviews don't scale.

**Example violations:**
- Frontend directly accessing database (violates layer boundaries)
- Services in wrong layers (business logic in presentation layer)
- Circular dependencies between services
- Missing compliance controls (HIPAA, SOC 2)

## Solution: Policy as Code

Sruja lets you codify architectural standards as **constraints** and **conventions** that are:
- ✅ Version-controlled with your code
- ✅ Validated automatically in CI/CD
- ✅ Enforced consistently across teams
- ✅ Tracked and reported on

## Writing Constraints

Constraints define **hard rules** that must be followed. Violations block CI/CD.

```sruja
architecture "E-Commerce Platform" {
  // Constraint: Presentation layer cannot access datastores directly
  constraint C1 {
    description "Presentation layer must not access datastores"
    rule "containers in layer 'presentation' must not have relations to datastores"
  }

  // Constraint: No circular dependencies
  constraint C2 {
    description "No circular dependencies between services"
    rule "no cycles in service dependencies"
  }

  // Constraint: Compliance requirement
  constraint C3 {
    description "Payment services must have encryption"
    rule "containers with tag 'payment' must have property 'encryption' = 'AES-256'"
  }

  layering {
    layer Presentation "Presentation Layer" {
      description "User-facing interfaces"
    }
    layer Business "Business Logic Layer" {
      description "Core business logic"
    }
    layer Data "Data Access Layer" {
      description "Data persistence"
    }
  }

  system Shop "E-Commerce System" {
    container WebApp "Web Application" {
      layer Presentation
      // This would violate C1 if it accessed DB directly
    }
    
    container PaymentService "Payment Service" {
      layer Business
      tags ["payment"]
      properties {
        encryption "AES-256"  // Required by C3
      }
    }
    
    datastore DB "Database" {
      layer Data
    }

    // Correct: WebApp -> PaymentService -> DB (respects layers)
    WebApp -> PaymentService "Processes payments"
    PaymentService -> DB "Stores transactions"
  }
}
```

## Writing Conventions

Conventions define **best practices** and **naming standards**. They're warnings, not blockers.

```sruja
architecture "Microservices Platform" {
  // Convention: Naming standards
  convention N1 {
    description "Service names should follow pattern: <domain>-<function>"
    rule "container names should match pattern /^[a-z]+-[a-z]+$/"
  }

  // Convention: Technology standards
  convention T1 {
    description "API services should use REST or gRPC"
    rule "containers with tag 'api' must have technology matching /REST|gRPC/"
  }

  system Platform "Microservices Platform" {
    container user-service "User Service" {  // ✅ Follows N1
      tags ["api"]
      technology "REST"  // ✅ Follows T1
    }
    
    container authService "Auth Service" {  // ⚠️ Violates N1 (should be auth-service)
      tags ["api"]
      technology "GraphQL"  // ⚠️ Violates T1 (should be REST or gRPC)
    }
  }
}
```

## Real-World Example: Multi-Team Governance

Here's how a large organization enforces standards across teams:

```sruja
architecture "Organization Standards" {
  // Global constraint: All services must have SLOs
  constraint Global1 {
    description "All production services must define SLOs"
    rule "containers with tag 'production' must have slo block"
  }

  // Team-specific constraint: Payment team standards
  constraint Payment1 {
    description "Payment services must be in payment layer"
    rule "containers with tag 'payment' must have layer 'payment'"
  }

  // Compliance constraint: HIPAA requirements
  constraint Compliance1 {
    description "Healthcare data must be encrypted"
    rule "datastores with tag 'healthcare' must have property 'encryption' = 'AES-256'"
  }

  layering {
    layer payment "Payment Layer"
    layer healthcare "Healthcare Layer"
  }

  system PaymentSystem "Payment System" {
    container PaymentAPI "Payment API" {
      layer payment
      tags ["payment", "production"]
      slo {
        availability { target "99.9%" window "30 days" }
        latency { p95 "200ms" window "7 days" }
      }
    }
  }

  system HealthcareSystem "Healthcare System" {
    datastore PatientDB "Patient Database" {
      layer healthcare
      tags ["healthcare"]
      properties {
        encryption "AES-256"
      }
    }
  }
}
```

## Enforcing in CI/CD

Add validation to your CI/CD pipeline:

```yaml
# .github/workflows/architecture.yml
name: Architecture Validation
on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Sruja
        run: curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
      - name: Validate Architecture
        run: sruja lint architecture.sruja
      - name: Check Constraints
        run: sruja validate --constraints architecture.sruja
```

**Result:** Violations block merges automatically.

## Key Takeaways

1. **Constraints** = Hard rules that block CI/CD
2. **Conventions** = Best practices that warn
3. **Version control** policies with code
4. **Automate enforcement** in CI/CD
5. **Scale governance** across teams

## Next Steps

- Try writing constraints for your organization
- Integrate validation into your CI/CD pipeline
- Track compliance across services
- Iterate based on team feedback

**You now know how to codify architectural policies. Let's enforce them automatically!**
