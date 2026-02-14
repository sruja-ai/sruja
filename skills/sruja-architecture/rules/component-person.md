# component-person

## Why It Matters

Person components represent external actors that interact with your system. Identifying all external actors first ensures you capture all system boundaries and interactions, leading to more complete and accurate architectures.

## When to Apply

Use person components when:

- Identifying users of the system
- Modeling external systems you integrate with
- Defining third-party services or APIs
- Showing stakeholders or administrators

## Correct Approach

### Example 1: User Types

```sruja
architecture "Project Management App" {
  end_user = person "End User" {
    description "Team member using the application"
  }

  admin = person "Administrator" {
    description "System administrator managing users and settings"
  }

  viewer = person "Report Viewer" {
    description "External stakeholder viewing reports"
  }
}
```

### Example 2: External Systems

```sruja
architecture "Payment Processing" {
  customer = person "Customer" {
    description "User making purchases"
  }

  stripe = person "Stripe" {
    description "External payment processing service"
  }

  analytics = person "Analytics Service" {
    description "External analytics and tracking"
  }

  notification_service = person "Twilio" {
    description "External SMS and notification service"
  }
}
```

### Example 3: System Boundaries

```sruja
architecture "E-Commerce Platform" {
  internal_system = person "Internal Inventory System" {
    description "Legacy inventory management system"
  }

  logistics_provider = person "FedEx" {
    description "Shipping and logistics partner"
  }

  tax_service = person "Tax Calculation API" {
    description "External tax calculation service"
  }
}
```

## Incorrect Approach

```sruja
# ❌ Treating external services as containers
stripe = container "Stripe" {
  technology "Node.js"
  description "External service"
}

# ✅ Correct: Use person for external actors
stripe = person "Stripe" {
  description "External payment processing service"
}
```

## Common Mistakes

1. **Not Defining All Users**: Missing stakeholders, viewers, or admins
   - ❌ Only defining "User"
   - ✅ Define all actor types: User, Admin, Viewer, Manager

2. **Treating External Services as Containers**
   - ❌ Stripe as a container in your system
   - ✅ Stripe as an external person/component

3. **Vague Descriptions**: Not explaining the actor's role
   - ❌ "External service"
   - ✅ "External payment processing service handling transactions"

4. **Missing Integration Points**: Not showing relationships
   - ❌ Person component with no relationships
   - ✅ Person has clear relationships to your systems

## Best Practices

1. **Use Clear, Descriptive Names**
   - ✅ "End User", "Administrator", "API Consumer"
   - ❌ "User", "Admin", "Consumer"

2. **Explain Purpose in Description**
   - Include what they do with the system
   - Mention their goals or responsibilities

3. **Define Before Using**
   - All persons should be defined before relationships
   - Follow top-down: persons first, then relationships

4. **Consider All Stakeholders**
   - End users
   - Administrators
   - Third-party integrations
   - External systems
   - Business stakeholders (report viewers)

## Additional Context

Person components are the entry points to your architecture and help identify:

- System boundaries
- Authentication requirements
- API design needs
- Integration points
- Security considerations

Related rules:

- `component-system` - For major domain boundaries
- `relationship-synchronous` - For API interactions with persons
- `relationship-asynchronous` - For event-based interactions
- `tradeoff-sync-vs-async` - Choosing communication patterns

## References

- Use Case Modeling
- Actor-Based System Design
- Domain-Driven Design: Context Mapping