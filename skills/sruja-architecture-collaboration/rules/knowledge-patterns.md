# Knowledge: Pattern Library

## Description

Reusable architecture patterns with provenance and usage guidance.

## Pattern Structure

```sruja
pattern "[Pattern Name]" {
  description "[What this pattern does]"
  
  applies_when [
    "[Condition 1]",
    "[Condition 2]"
  ]
  
  benefits [
    "[Benefit 1]",
    "[Benefit 2]"
  ]
  
  drawbacks [
    "[Drawback 1]",
    "[Drawback 2]"
  ]
  
  implementation {
    // Example component structure
  }
  
  related_patterns ["[pattern-1]", "[pattern-2]"]
  
  metadata {
    category "[structural|behavioral|deployment]"
    maturity "[stable|evolving|experimental]"
    provenance "[source or origin]"
  }
}
```

## Pattern Categories

### Structural Patterns
- Microservices
- Monolith
- Layered Architecture
- Hexagonal Architecture

### Communication Patterns
- API Gateway
- Backend for Frontend
- Service Mesh
- Event-Driven

### Data Patterns
- Database per Service
- Shared Database
- CQRS
- Event Sourcing

### Deployment Patterns
- Blue-Green
- Canary
- Sidecar
- Ambassador

## Usage in Proposals

```sruja
// Reference patterns in architecture
system "My Platform" {
  metadata {
    patterns [
      { name "api-gateway" reason "Multiple services" },
      { name "database-per-service" reason "Independent scaling" }
    ]
  }
}
```

## Pattern Discovery

```
When to add to library:

1. Used successfully 3+ times
2. Documented with examples
3. Reviewed by team
4. Added to pattern registry
```

## Best Practices

- Document thoroughly
- Include real examples
- Note when NOT to use
- Link to related patterns
- Track usage and outcomes

## Anti-Patterns

- ❌ Undocumented patterns
- ❌ Copy-paste without context
- ❌ Not tracking provenance
- ❌ Not updating based on learnings
