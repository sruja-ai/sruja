# LikeC4 Syntax with Sruja Extensions

This document describes how to use LikeC4 syntax with Sruja-specific features (requirements, policies, ADRs, etc.).

## Overview

Sruja uses LikeC4 syntax as the primary DSL format, with Sruja-specific extensions added directly in the `model` block. This provides:
- LikeC4 compatibility for diagram rendering
- Sruja governance features (requirements, policies, ADRs, scenarios)
- Single unified syntax (no need to support both formats)

## Syntax Structure

```sruja
specification {
  element person
  element system
  element container
  element component
  element database
  element queue
}

model {
  // LikeC4 elements
  user = person "User"
  Backend = system "Backend" {
    API = container "REST API" {
      technology "Go"
    }
  }
  
  user -> Backend.API "uses"
  
  // Sruja extensions (allowed in model block)
  requirement R1 functional "Persist user data"
  policy P1 security "Use HTTPS"
  adr ADR001 "Adopt Stripe" {
    status "Accepted"
  }
  scenario S1 "User flow" {
    step "Step 1"
    step "Step 2"
  }
}

views {
  view index {
    title "Overview"
    include *
  }
}
```

## Sruja Extensions

All Sruja-specific features can be used directly in the `model` block:

### Requirements
```sruja
model {
  requirement R1 functional "Persist user data"
  requirement R2 constraint "Expose public API"
  requirement R3 performance "Fast checkout"
}
```

### Policies
```sruja
model {
  policy P1 security "All API endpoints must use HTTPS"
  policy P2 compliance "GDPR compliance required"
}
```

### ADRs (Architecture Decision Records)
```sruja
model {
  adr ADR001 "Adopt Stripe for payments" {
    status "Accepted"
    context "Need payment processing"
    decision "Use Stripe API"
    consequences "Easy integration, good documentation"
  }
}
```

### Scenarios
```sruja
model {
  scenario S1 "User purchases product" {
    step "User adds item to cart"
    step "User proceeds to checkout"
    step "Payment is processed"
  }
}
```

### Flows
```sruja
model {
  flow F1 "Order processing" {
    step "Receive order"
    step "Process payment"
    step "Fulfill order"
  }
}
```

### Constraints & Conventions
```sruja
model {
  constraints {
    constraint1 "Value 1"
    constraint2 "Value 2"
  }
  
  conventions {
    convention1 "Value 1"
    convention2 "Value 2"
  }
}
```

### Contracts
```sruja
model {
  contracts {
    contract C1 {
      type "REST"
      schema "OpenAPI 3.0"
    }
  }
}
```

### Metadata on Elements
```sruja
model {
  Backend = system "Backend" {
    metadata {
      owner "platform-team"
      slo_availability "99.9%"
    }
  }
}
```

## Migration from Legacy Syntax

### Before (Legacy)
```sruja
architecture "My System" {
  system Backend "Backend" {
    container API "API"
  }
  requirement R1 functional "Persist data"
}
```

### After (LikeC4 + Sruja Extensions)
```sruja
specification {
  element system
  element container
}

model {
  Backend = system "Backend" {
    API = container "API"
  }
  requirement R1 functional "Persist data"
}

views {
  view index {
    include *
  }
}
```

## Benefits

1. **LikeC4 Compatibility**: Works with LikeC4 tooling and ecosystem
2. **Sruja Governance**: All Sruja-specific features are supported
3. **Single Syntax**: No need to support multiple syntaxes
4. **Extensible**: Easy to add more Sruja features in the future

## Example

See `examples/likec4/ecommerce_with_sruja_extensions.sruja` for a complete example.
