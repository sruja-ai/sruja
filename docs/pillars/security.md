# Security Pillar

This document describes how Sruja supports the Security pillar of the Well-Architected Framework.

[← Back to Pillars Index](./README.md)

## Overview

**Security** focuses on protecting data, systems, and assets while delivering business value.

**Key principles:**
- Strong identity foundation
- Layered security
- Traceability
- Automation of security controls
- Protecting data in transit and at rest

---

## Core Support (Basic)

The core DSL includes basic security features:

### Authentication

```sruja
system MyService {
  authentication {
    type: "OAuth2"
    provider: "Auth0"
  }
}
```

### Encryption

```sruja
container Database {
  encryption {
    at_rest: "AES-256"
    in_transit: "TLS 1.3"
  }
}
```

### Basic Security Tags

```sruja
system PaymentService {
  tags: ["pci-dss", "encrypted", "audited"]
}
```

---

## Advanced Extensions

### Security DSL

Complete security modeling:

```sruja
security {
  authentication OAuth2Auth {
    type: "OAuth2"
    provider: "Auth0"
    scopes: ["read", "write"]
  }

  authorization RBAC {
    model: "role-based"
    roles: ["admin", "user", "guest"]
  }

  encryption DataEncryption {
    at_rest: "AES-256"
    in_transit: "TLS 1.3"
  }

  security_policy PaymentPolicy {
    requires: [encryption, authentication, audit_logging]
    compliance: ["PCI-DSS", "GDPR"]
  }
}
```

### Identity & Access Management

```sruja
iam {
  role APIServiceRole {
    permissions: ["s3:read", "dynamodb:write"]
    principal: "APIService"
  }

  policy DataAccessPolicy {
    effect: "allow"
    actions: ["s3:GetObject"]
    resources: ["arn:aws:s3:::data/*"]
  }
}
```

### Network Security

```sruja
network_security {
  vpc VPC1 {
    subnets: ["private", "public"]
    nat_gateway: true
  }

  security_group APISecurityGroup {
    ingress: [
      { port: 443, protocol: "tcp", source: "0.0.0.0/0" }
    ]
    egress: [
      { port: 5432, protocol: "tcp", destination: "database" }
    ]
  }
}
```

### Data Classification

```sruja
data_classification {
  pii_data {
    classification: "PII"
    encryption: "required"
    retention: "7 years"
    compliance: ["GDPR", "CCPA"]
  }

  payment_data {
    classification: "PCI-DSS"
    encryption: "required"
    access: "restricted"
  }
}
```

### Compliance

```sruja
compliance {
  pci_dss {
    standard: "PCI-DSS v3.2.1"
    requirements: [
      "encryption_at_rest",
      "encryption_in_transit",
      "access_controls",
      "audit_logging"
    ]
  }

  gdpr {
    standard: "GDPR"
    requirements: [
      "data_protection",
      "right_to_erasure",
      "data_portability"
    ]
  }
}
```

---

## Integration with Other Pillars

### Operational Excellence
- Security monitoring and alerting
- Security incident response

### Reliability
- Security as a reliability factor
- Secure failure modes

### Cost Optimization
- Security cost considerations
- Compliance cost modeling

---

## Validation Rules

Security validation includes:

- ✅ All public-facing services must have authentication
- ✅ All data stores must have encryption
- ✅ PII data must be classified
- ✅ Compliance requirements must be met
- ✅ Network security must be defined
- ✅ Access controls must be specified

---

## Engines

### Core Engines
- **Basic Authentication Engine** - Authentication type validation
- **Basic Encryption Engine** - Encryption requirement checks
- **Basic Security Tags Engine** - Security tag validation

### Advanced Engines
- **Security Validation Engine** - Security policy enforcement
- **Compliance Engine** - Security + org policies
- **IAM Engine** - Identity and access management
- **Network Security Engine** - VPC, security groups, firewalls
- **Data Classification Engine** - PII, PCI-DSS data handling
- **Security Audit Engine** - Security compliance auditing
- **Vulnerability Detection Engine** - Security vulnerability scanning
- **Threat Modeling Engine** - Security threat analysis
- **Governance Policy Engine** - Technology + patterns + boundaries
- **Policy-as-Code Engine** - Governance rules + auto-fixes
- **Architecture Audit Engine** - Full compliance + decision history

See: [Engines by Pillars](./engines.md#security-pillar) for complete list

---

## DSL Reference

See: [DSL Extensions - Security](../specs/dsl-extensions.md#4-security-dsl)

---

## Examples

- [Security Example](../examples/security/)
- [Compliance Example](../examples/compliance/)

---

*Security is foundational - protect your data, systems, and assets while delivering business value.*

