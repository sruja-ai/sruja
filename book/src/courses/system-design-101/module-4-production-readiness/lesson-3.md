---
title: "Lesson 3: Governance as Code"
weight: 3
summary: "Automating architectural compliance with Policies and Rules."
---

# Lesson 3: Governance as Code

The SOC 2 auditor asked a simple question: "Can you show me all databases that store customer PII and confirm they're encrypted?"

I froze.

We had 47 databases across 12 services. I had no idea which ones stored PII. I had no idea which ones were encrypted. I had no idea which ones were even in scope for the audit.

It took us three weeks to manually audit every database. We found three unencrypted databases with customer data. We failed the audit. The company lost a $2M contract that required SOC 2 compliance.

**The worst part?** We'd passed the audit six months earlier. But in those six months, developers had added new databases. No one checked if they were encrypted. No one even knew they existed.

That's when I learned: **manual governance doesn't scale**. If your governance depends on people remembering rules, you've already failed.

This lesson is about Governance as Code: treating architectural policies as executable code that validates your architecture automatically.

## What is Governance as Code?

**Governance as Code** means expressing architectural policies as machine-readable rules that can be validated automatically:

- "All databases must be encrypted" → Validator checks encryption tags
- "No circular dependencies" → Validator checks dependency graph
- "All services must have SLOs" → Validator checks for SLO definitions
- "No public APIs without authentication" → Validator checks auth requirements

**Without Governance as Code:**
- Policies exist in wikis and documents
- Compliance depends on code reviews
- Violations found late (or never)
- Audits require manual inspection
- Inconsistent enforcement

**With Governance as Code:**
- Policies are executable code
- Validation runs in CI/CD
- Violations caught immediately
- Audits are automated
- Consistent, reliable enforcement

## The Three Types of Governance

### Type 1: Guardrails (Prevent Bad Things)

**Purpose:** Stop dangerous or non-compliant architecture choices.

**Examples:**
- Databases storing PII must be encrypted
- No public endpoints without authentication
- No single points of failure
- No databases in unauthorized regions

**Real-world example:** Netflix

Netflix has a guardrail: "No service can depend on a single availability zone." Their validation tool checks every service's deployment configuration. If it's single-AZ, the build fails. This guardrail has prevented dozens of potential outages.

**Sruja example:**

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

// Define the policy
policy EncryptionPolicy "All databases must be encrypted" {
  category "security"
  enforcement "required"
  
  rule {
    element_type "database"
    required_tags ["encrypted"]
    error "Database {element} missing 'encrypted' tag. All databases must be encrypted at rest."
  }
}

// Apply to your architecture
ECommerce = system "E-Commerce" {
  // This will PASS validation
  SecureDB = database "Customer Database" {
    technology "PostgreSQL"
    tags ["encrypted", "pci-compliant"]
  }
  
  // This will FAIL validation
  InsecureDB = database "Analytics Database" {
    technology "MySQL"
    // Missing "encrypted" tag - violation!
  }
}

view index {
  include *
}
```

### Type 2: Standards (Enforce Consistency)

**Purpose:** Ensure architectural consistency across teams.

**Examples:**
- All services must use the same logging format
- All APIs must follow REST naming conventions
- All services must have health check endpoints
- All databases must have backup policies

**Real-world example:** Google

Google has thousands of services, but they all follow the same API design guidelines. Why? Because they have automated validators that check every API against their standards. Inconsistent APIs fail the build.

**Sruja example:**

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

policy LoggingStandard "Services must have structured logging" {
  category "operations"
  enforcement "required"
  
  rule {
    element_type "container"
    required_tags ["structured-logging"]
    error "Service {element} must implement structured logging per company standard."
  }
}

policy SLOStandard "Services must have SLOs defined" {
  category "reliability"
  enforcement "required"
  
  rule {
    element_type "container"
    requires_slo true
    error "Service {element} missing SLO definitions. All production services must have SLOs."
  }
}

ECommerce = system "E-Commerce" {
  API = container "API Service" {
    technology "Rust"
    tags ["structured-logging"]
    
    slo {
      availability {
        target "99.9%"
        window "30 days"
      }
      latency {
        p95 "200ms"
        p99 "500ms"
      }
    }
  }
}

view index {
  include *
}
```

### Type 3: Best Practices (Codified Wisdom)

**Purpose:** Encode architectural lessons learned.

**Examples:**
- Services with > 10 dependencies should be split
- Databases accessed by > 5 services need a cache layer
- Services handling payments need circuit breakers
- Critical services need multi-region deployment

**Real-world example:** Amazon

Amazon learned the hard way that services with too many dependencies become bottlenecks. They codified this lesson: "If a service has more than 20 dependencies, architecture review required." This rule is enforced automatically.

**Sruja example:**

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

policy DependencyLimit "Services should not have too many dependencies" {
  category "architecture"
  enforcement "warning"
  
  rule {
    element_type "container"
    max_incoming_relations 10
    warning "Service {element} has {count} incoming dependencies. Consider splitting if > 10."
  }
  
  rule {
    element_type "container"
    max_outgoing_relations 15
    error "Service {element} has {count} outgoing dependencies. Split required if > 15."
  }
}

ECommerce = system "E-Commerce" {
  API = container "API Service" {
    technology "Rust"
  }
  
  // Imagine 20 services all calling API
  Service1 = container "Service 1" { API -> Service1 }
  Service2 = container "Service 2" { API -> Service2 }
  // ... and 18 more
  
  // This would trigger the dependency limit warning
}

view index {
  include *
}
```

## Real-World Governance Stories

### Netflix: Resilience Governance

**The problem:** Netflix had services that weren't resilient. They'd fail when dependencies failed.

**The solution:** Governance rules requiring:
- Every service must have fallback behavior
- Every external call must have a timeout
- Critical services must have circuit breakers

**The enforcement:** Their Chaos Monkey tool tests these rules in production. If a service can't handle failure, Chaos Monkey finds out. Publicly.

**The result:** 99.99% availability, even with thousands of service failures per day.

### Amazon: Team Size Governance

**The problem:** Large teams move slowly and create coordination overhead.

**The solution:** "Two-pizza team" rule - teams should be small enough to be fed by two pizzas (6-10 people).

**The enforcement:** Each service has a defined owner. If the team grows too large, governance tools flag it. Architecture review required.

**The result:** Faster decisions, clearer ownership, decentralized architecture.

### Google: API Standards Governance

**The problem:** Inconsistent APIs made integration difficult. Every team invented their own patterns.

**The solution:** Google API Design Guide - full standards for all APIs.

**The enforcement:** Automated linters check every API definition. Non-compliant APIs fail CI builds. No exceptions.

**The result:** Consistent developer experience across thousands of APIs.

### Stripe: Security Governance

**The problem:** Handling payments requires strict security. Manual security reviews don't scale.

**The solution:** Codified security policies:
- All PII must be encrypted at rest
- All APIs must use TLS 1.3+
- All databases must have audit logs
- All services must have vulnerability scanning

**The enforcement:** Automated security scanners check every deployment. Violations block production.

**The result:** PCI-DSS compliance maintained across thousands of changes per day.

## Common Governance Rules

Here are the governance rules I see most often in production systems:

### Security Rules

```sruja
// partial
// Rule 1: All databases encrypted
policy EncryptionPolicy "All databases must be encrypted" {
  rule {
    element_type "database"
    required_tags ["encrypted"]
  }
}

// Rule 2: No sensitive data in caches
policy CacheDataPolicy "No PII in cache layers" {
  rule {
    element_type "database"
    tag "cache"
    forbidden_tags ["pii", "sensitive"]
  }
}

// Rule 3: All external APIs authenticated
policy APIAuthPolicy "External APIs must require authentication" {
  rule {
    element_type "container"
    tag "public-api"
    required_tags ["authentication"]
  }
}

// Rule 4: No databases in unauthorized regions
policy DataResidencyPolicy "Data must stay in approved regions" {
  rule {
    element_type "database"
    tag "pii"
    allowed_regions ["us-east-1", "eu-west-1"]
  }
}
```

### Architecture Rules

```sruja
// partial
// Rule 5: No circular dependencies
policy NoCircularDeps "Services cannot have circular dependencies" {
  rule {
    check_circular_dependencies true
    error "Circular dependency detected between {source} and {target}"
  }
}

// Rule 6: Services must have owners
policy OwnershipPolicy "All services must have defined owners" {
  rule {
    element_type "container"
    required_metadata ["owner", "team"]
  }
}

// Rule 7: No single points of failure
policy RedundancyPolicy "Critical services must be redundant" {
  rule {
    element_type "container"
    tag "critical"
    requires_scale true
    min_replicas 3
  }
}

// Rule 8: Layer violations prohibited
policy LayerPolicy "Respect architectural layers" {
  rule {
    element_type "container"
    tag "presentation"
    cannot_depend_on "datastore"
  }
}
```

### Operations Rules

```sruja
// partial
// Rule 9: All services must have SLOs
policy SLOPolicy "Services must have SLOs defined" {
  rule {
    element_type "container"
    requires_slo true
  }
}

// Rule 10: All services must be monitored
policy MonitoringPolicy "Services must be monitored" {
  rule {
    element_type "container"
    required_tags ["monitored"]
  }
}

// Rule 11: All databases must have backups
policy BackupPolicy "Databases must have backup policies" {
  rule {
    element_type "database"
    required_tags ["backed-up"]
    required_metadata ["backup_frequency", "backup_retention"]
  }
}

// Rule 12: All services must have health checks
policy HealthCheckPolicy "Services must implement health checks" {
  rule {
    element_type "container"
    required_tags ["health-check"]
  }
}
```

### Compliance Rules

```sruja
// partial
// Rule 13: PII handling requirements
policy PIIHandlingPolicy "PII must be handled correctly" {
  rule {
    element_type "database"
    tag "pii"
    required_tags ["encrypted", "audit-logged", "access-controlled"]
  }
}

// Rule 14: Payment data requirements
policy PCICompliancePolicy "Payment data must be PCI compliant" {
  rule {
    element_type "container"
    tag "payment-processing"
    required_tags ["pci-compliant", "pci-audited"]
  }
}

// Rule 15: Data retention requirements
policy DataRetentionPolicy "Data must have retention policies" {
  rule {
    element_type "database"
    required_metadata ["retention_period", "deletion_policy"]
  }
}
```

## CI/CD Integration

Governance only works if it's enforced. Here's how to integrate with your pipeline:

### Stage 1: Pre-Commit Hooks

**What:** Validate architecture changes before they're committed.

```bash
# .git/hooks/pre-commit
#!/bin/bash
sruja validate architecture.sruja
if [ $? -ne 0 ]; then
  echo "Architecture validation failed. Fix violations before committing."
  exit 1
fi
```

**Catches:** Basic violations early, before code review.

### Stage 2: Pull Request Validation

**What:** Validate architecture in CI when PRs are created.

```yaml
# .github/workflows/architecture-validation.yml
name: Architecture Validation

on: [pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      
      - name: Validate Architecture
        run: |
          sruja validate architecture.sruja --strict
          
      - name: Check Compliance
        run: |
          sruja compliance-check --policies ./policies/
```

**Catches:** All violations before merge.

### Stage 3: Deployment Gates

**What:** Validate architecture before production deployment.

```yaml
# deployment-pipeline.yml
stages:
  - name: validate-architecture
    steps:
      - sruja validate architecture.sruja
      - sruja compliance-check --policies ./policies/production/
      
  - name: deploy-production
    needs: validate-architecture
    if: success()
    steps:
      - deploy-to-production
```

**Catches:** Production-specific violations (security, compliance).

### Stage 4: Continuous Monitoring

**What:** Validate running infrastructure matches architecture.

```bash
# Run continuously (e.g., every hour)
sruja drift-detect --architecture architecture.sruja --live-infrastructure
```

**Catches:** Configuration drift, manual changes, unapproved modifications.

## Governance Maturity Model

Where is your organization on the governance journey?

### Level 0: No Governance

**What it looks like:**
- No documented policies
- Decisions made ad-hoc
- Compliance discovered during audits
- Inconsistent architecture

**Real-world example:** Early-stage startups

**The problem:** You'll fail audits eventually. But you're probably too small to care yet.

**When it's okay:** < 10 engineers, pre-revenue, learning phase.

### Level 1: Manual Governance

**What it looks like:**
- Policies documented in wikis
- Architecture reviews are manual
- Compliance checks during audits
- Some consistency through code review

**Real-world example:** Growing companies (50-200 engineers)

**The problem:** Doesn't scale. Policies become outdated. Reviews are inconsistent. Violations slip through.

**How to improve:** Start automating the most critical checks.

### Level 2: Automated Checks

**What it looks like:**
- Key policies automated
- CI/CD validation runs automatically
- Violations caught early
- Consistent enforcement

**Real-world example:** Mature companies (200-1000 engineers)

**The benefit:** Scales with team size. Consistent enforcement. Early violation detection.

**How to improve:** Expand coverage, add more policies.

### Level 3: Continuous Enforcement

**What it looks like:**
- Most policies automated
- Real-time validation
- Drift detection
- Self-documenting compliance

**Real-world example:** Tech giants (Google, Netflix, Amazon)

**The benefit:** Compliance is continuous, not periodic. Audits are easy. Architecture stays healthy.

**How to improve:** Fine-tune policies, reduce false positives.

### Level 4: Self-Service with Guardrails

**What it looks like:**
- Developers can deploy freely
- Guardrails prevent bad choices
- Compliance is transparent
- Architecture evolves safely

**Real-world example:** Very few companies (Spotify, Netflix)

**The benefit:** Fast development, safe architecture. Best of both worlds.

**The goal:** This is where you want to be.

## Common Governance Mistakes

### Mistake #1: Governance Theater

**What happens:** You have lots of policies, but they're not enforced.

**Example:**
- Wiki says "All databases must be encrypted"
- But no automated checks
- Some databases encrypted, some not
- Audit fails

**The fix:** If a policy isn't enforced, delete it or enforce it.

### Mistake #2: Too Many Rules

**What happens:** You create policies for everything.

**Example:**
- 200 governance rules
- Developers need exceptions for 50% of changes
- Governance becomes a bottleneck
- People work around it

**The fix:** Start with 5-10 critical policies. Add more only when needed.

### Mistake #3: Rules Without Context

**What happens:** Policies exist but no one knows why.

**Example:**
```sruja
// partial
// BAD: No explanation
policy Rule42 "Services must have tag X" {
  // Why? What's the purpose?
}
```

**The fix:** Every policy should explain:
- Why it exists
- What problem it solves
- When it applies
- How to comply

```sruja
// partial
// GOOD: Clear context
policy EncryptionPolicy "All databases must be encrypted" {
  category "security"
  
  description "
    Unencrypted databases expose customer data if compromised.
    Required for SOC 2, PCI-DSS, GDPR compliance.
    Applies to all databases storing production data.
    Encrypt using AWS KMS or equivalent.
  "
  
  rule {
    element_type "database"
    required_tags ["encrypted"]
  }
}
```

### Mistake #4: No Exceptions Process

**What happens:** Rules are rigid with no way to handle edge cases.

**Example:**
- Legacy system can't comply with new encryption rule
- No way to get exception
- System remains non-compliant forever
- Governance loses credibility

**The fix:** Create an exceptions process:
1. Document why exception is needed
2. Define mitigation plan
3. Set expiration date
4. Require approval
5. Review periodically

### Mistake #5: One-Size-Fits-All Rules

**What happens:** Same rules applied to all systems regardless of context.

**Example:**
- "All services must have 99.99% availability"
- Even internal admin tools
- Over-engineering everywhere
- Wasted resources

**The fix:** Tiered policies based on criticality:
- Critical services: Strict rules
- Standard services: Moderate rules
- Internal tools: Basic rules

### Mistake #6: Governance as Afterthought

**What happens:** Architecture is designed first, governance added later.

**Example:**
- Build the system
- Try to add governance
- Discover fundamental violations
- Expensive refactoring

**The fix:** Governance from the start. Define policies first, design architecture to comply.

## The GOVERN Framework

When implementing Governance as Code, use this framework:

**G - Identify Goals**
- What are you trying to achieve?
- What problems are you solving?
- What's the risk of no governance?

**O - Define Outcomes**
- What does compliance look like?
- How will you measure success?
- What's acceptable vs. unacceptable?

**V - Validate Automatically**
- Which rules can be automated?
- What checks run in CI/CD?
- What needs continuous monitoring?

**E - Educate Teams**
- Do developers understand the rules?
- Is documentation clear?
- How do people learn about violations?

**R - Review Regularly**
- Are policies still relevant?
- Are there too many false positives?
- What new policies are needed?

**N - Nurture Culture**
- Is governance seen as help or hindrance?
- Do teams buy in?
- How do you handle exceptions?

## What to Remember

1. **Manual governance doesn't scale** - If you're relying on people remembering rules, you've already failed

2. **Start with critical policies** - Security, compliance, reliability. Add more later.

3. **Automate enforcement** - Policies without enforcement are just suggestions

4. **Integrate with CI/CD** - Validate early, validate often, validate automatically

5. **Explain the why** - Every policy should have clear context and rationale

6. **Allow exceptions** - Rigid rules without exceptions create workarounds

7. **Tier your rules** - Not all services need the same governance level

8. **Make compliance transparent** - Developers should know status without asking

9. **Review and evolve** - Governance should improve over time, not stagnate

10. **Governance enables speed** - Good governance lets teams move fast safely

## When to Start Governance

**Phase 1: Prototype (Skip governance)**
- Focus on learning
- Minimal policies
- Manual reviews fine

**Phase 2: Production (Start governing)**
- Critical security policies
- Basic compliance checks
- CI/CD integration

**Phase 3: Scale (Govern seriously)**
- full policies
- Continuous enforcement
- Self-service with guardrails

**Phase 4: Enterprise (Govern everything)**
- Full audit automation
- Real-time compliance
- Multi-team coordination

## Practical Exercise

Implement governance for a real or hypothetical system:

**Step 1: Identify Critical Policies**
- What are your top 5 security risks?
- What compliance requirements exist?
- What architectural standards matter?

**Step 2: Write Policies as Code**
- Express each policy in Sruja
- Include context and rationale
- Define clear validation rules

**Step 3: Integrate with CI/CD**
- Add validation to pull requests
- Block non-compliant changes
- Provide clear error messages

**Step 4: Create Compliance Dashboard**
- Show current compliance status
- Track violations over time
- Make status visible to all

**Step 5: Document Exception Process**
- How to request exceptions
- Who approves
- How to track

**Time:** 60-90 minutes

## Complete Example: Production Governance

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

// ============ GOVERNANCE POLICIES ============

// Security policies
policy EncryptionPolicy "All databases must be encrypted" {
  category "security"
  enforcement "required"
  
  description "
    Unencrypted databases expose data if compromised.
    Required for SOC 2, PCI-DSS compliance.
  "
  
  rule {
    element_type "database"
    required_tags ["encrypted"]
    error "Database {element} must be encrypted. Add 'encrypted' tag."
  }
}

policy PIIPolicy "PII data requires special handling" {
  category "security"
  enforcement "required"
  
  rule {
    element_type "database"
    tag "pii"
    required_tags ["encrypted", "audit-logged", "access-controlled"]
    error "PII database {element} missing required controls."
  }
}

// Reliability policies
policy SLOPolicy "Services must have SLOs" {
  category "reliability"
  enforcement "required"
  
  rule {
    element_type "container"
    tag "production"
    requires_slo true
    error "Production service {element} must have SLOs defined."
  }
}

policy RedundancyPolicy "Critical services must be redundant" {
  category "reliability"
  enforcement "required"
  
  rule {
    element_type "container"
    tag "critical"
    requires_scale true
    min_replicas 3
    error "Critical service {element} must have min 3 replicas."
  }
}

// Architecture policies
policy OwnershipPolicy "Services must have owners" {
  category "operations"
  enforcement "required"
  
  rule {
    element_type "container"
    required_metadata ["owner", "team"]
    error "Service {element} missing owner/team metadata."
  }
}

policy NoCircularDeps "No circular dependencies" {
  category "architecture"
  enforcement "required"
  
  rule {
    check_circular_dependencies true
    error "Circular dependency detected. Services cannot depend on each other."
  }
}

// ============ ARCHITECTURE ============

PaymentService = system "Payment Service" {
  API = container "Payment API" {
    technology "Rust"
    tags ["production", "critical"]
    
    metadata {
      owner "payments-team"
      team "payments@company.com"
    }
    
    slo {
      availability {
        target "99.99%"
        window "30 days"
      }
      latency {
        p95 "100ms"
        p99 "200ms"
      }
    }
  }
  
  DB = database "Payment Database" {
    technology "PostgreSQL"
    tags ["encrypted", "pii", "pci-compliant", "audit-logged", "access-controlled"]
    
    slo {
      availability {
        target "99.99%"
        window "30 days"
      }
    }
  }
}

Auditor = person "Security Auditor"
Auditor -> PaymentService.API "Reviews"
PaymentService.API -> PaymentService.DB "Reads/Writes"

// ============ VIEWS ============

view index {
  title "Payment Service Architecture"
  include *
}

view compliance {
  title "Compliance Status"
  include PaymentService.API PaymentService.DB
  description "Shows compliance with governance policies"
}

view slos {
  title "SLO Dashboard"
  include PaymentService.API PaymentService.DB
  exclude Auditor
}

// ============ VALIDATION ============
// Run: sruja validate architecture.sruja
// This will check all policies and report violations
```

---

**Congratulations!** You've completed Module 4: Production Readiness. You now have all the tools to create production-ready architecture that's documented, deployable, and governed.

**Next steps:** Apply these lessons to your real systems. Start with one module, get good at it, then expand. Architecture is a practice, not a destination.

---

**Course Complete!** 🎉

You've finished the System Design 101 course. You now understand:
- How to think about systems (Module 1)
- How to model them effectively (Module 2)
- Advanced techniques for complex scenarios (Module 3)
- How to make architecture production-ready (Module 4)

The best architects aren't the ones who know the most patterns. They're the ones who can communicate their decisions clearly, maintain their documentation, and govern their architecture effectively. You now have those skills.

Go build great systems! 🚀
