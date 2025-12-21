---
title: "Lesson 3: Governance as Code"
weight: 3
summary: "Automating architectural compliance with Policies and Rules."
---

# Lesson 3: Governance as Code

As your organization scales, manually reviewing every architectural change becomes impossible. You need automated guardrails to ensure consistency and security.

## What is Governance as Code?

Governance as Code treats architectural policies (e.g., "All databases must be encrypted", "No circular dependencies") as executable code that can be validated automatically in your CI/CD pipeline.

## Built-in Validation Rules

Sruja validates common architectural concerns automatically:

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
    PaymentService = system "Payment Service" {
        API = container "Payment API" {
            technology "Go"
            tags ["encrypted", "pci-compliant"]
            
            // SLOs for payment processing
            slo {
                availability {
                    target "99.99%"
                    window "30 days"
                    current "99.98%"
                    description "Payment processing must be highly available"
                }
                latency {
                    p95 "100ms"
                    p99 "200ms"
                    window "7 days"
                    current {
                        p95 "95ms"
                        p99 "190ms"
                    }
                    description "Fast payment processing critical for UX"
                }
                errorRate {
                    target "< 0.01%"
                    window "30 days"
                    current "0.008%"
                    description "Payment errors must be extremely rare"
                }
                throughput {
                    target "500 req/s"
                    window "1 hour"
                    current "480 req/s"
                    description "Handle peak payment volumes"
                }
            }
        }
        
        DB = datastore "Payment Database" {
            technology "PostgreSQL"
            tags ["encrypted", "backed-up"]
            
            // Database SLOs
            slo {
                availability {
                    target "99.95%"
                    window "30 days"
                    current "99.92%"
                }
                latency {
                    p95 "20ms"
                    p99 "50ms"
                    window "7 days"
                }
            }
        }
    }

    Auditor = person "Security Auditor"
    Auditor -> PaymentService.API "Reviews"
    PaymentService.API -> PaymentService.DB "Reads/Writes"
}

views {
  view index {
    title "Payment Service with Governance"
    include *
  }
  
  // SLO monitoring view
  view slos {
    title "SLO Monitoring View"
    include PaymentService.API PaymentService.DB
    exclude Auditor
    description "Focuses on components with SLOs defined"
  }
  
  // Compliance view
  view compliance {
    title "Compliance View"
    include PaymentService.API PaymentService.DB
    exclude Auditor
    description "Shows components with compliance tags"
  }
}
```

## Automated Validation

The real power comes when you run the Sruja CLI. It can check your architecture against these policies and fail the build if violations are found.

```bash
sruja validate architecture.sruja
```

This ensures that your architecture isn't just a diagram—it's a contract that is continuously verified.
