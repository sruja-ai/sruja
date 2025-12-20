---
title: "Policy"
weight: 50
summary: "Define architectural rules and constraints that must be followed."
---

# Policy

Policies define architectural rules, standards, and constraints that your system must follow. They help enforce best practices, compliance requirements, and organizational standards directly in your architecture model.

## Syntax

```sruja
policy PolicyID "Description" {
  category "category-name"
  enforcement "required" // "required" | "recommended" | "optional"
  description "Detailed description"
  metadata {
    // Additional metadata
  }
}
```

## Simple Policy

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
  policy SecurityPolicy "Enforce TLS 1.3 for all external communications"
}

views {
  view index {
    include *
  }
}
```

## Policy with Category and Enforcement

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
  policy DataRetentionPolicy "Retain order data for 7 years for tax compliance" {
    category "compliance"
    enforcement "required"
  }
}

views {
  view index {
    include *
  }
}
```

## Policy Fields

- **`ID`**: Unique identifier for the policy (e.g., `SecurityPolicy`, `GDPR_Compliance`)
- **`Description`**: Human-readable description of the policy
- **`category`** (optional): Policy category (e.g., "security", "compliance", "performance")
- **`enforcement`** (optional): Enforcement level ("required", "recommended", "optional")
- **`description`** (optional): Detailed description within the policy body
- **`metadata`** (optional): Additional metadata key-value pairs

## Example: Security Policies

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
  policy TLSEnforcement "All external communications must use TLS 1.3" {
    category "security"
    enforcement "required"
  }
  
  policy EncryptionAtRest "Sensitive data must be encrypted at rest" {
    category "security"
    enforcement "required"
  }
  
  system BankingApp {
    API = container "API Service"
    CustomerDB = datastore "Customer Database"
  }
}

views {
  view index {
    include *
  }
}
```

## Example: Compliance Policies

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
  policy HIPAACompliance "Must comply with HIPAA regulations" {
    category "compliance"
    enforcement "required"
    description "All patient data must be encrypted and access logged"
  }
  
  policy DataRetention "Medical records retained for 10 years" {
    category "compliance"
    enforcement "required"
  }
}

views {
  view index {
    include *
  }
}
```

## Example: Observability Policies

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
  policy Observability "All services must expose health check endpoints" {
    category "observability"
    enforcement "required"
    metadata {
      metricEndpoint "/health"
      logLevel "info"
    }
  }
}

views {
  view index {
    include *
  }
}
```

## Policy Categories

Common policy categories include:

- **`security`**: Security standards and practices
- **`compliance`**: Regulatory and legal requirements
- **`performance`**: Performance standards and SLAs
- **`observability`**: Monitoring, logging, and metrics requirements
- **`architecture`**: Architectural patterns and principles
- **`data`**: Data handling and privacy requirements

## Enforcement Levels

- **`required`**: Policy must be followed (non-negotiable)
- **`recommended`**: Policy should be followed (best practice)
- **`optional`**: Policy is a guideline (suggested)

## Benefits

- **Documentation**: Policies are part of your architecture, not separate documents
- **Validation**: Can be validated against actual implementations
- **Communication**: Clear standards for development teams
- **Compliance**: Track regulatory and organizational requirements
- **Governance**: Enforce architectural decisions and patterns

## Note on Rules

The `rule` keyword inside policies is not yet implemented. For now, policies serve as documentation and can be validated manually or through external tooling.

## See Also

- [Architecture](/docs/concepts/architecture)
- [System](/docs/concepts/system)
- [Requirements](/docs/concepts/requirement)
- [ADR](/docs/concepts/adr)
