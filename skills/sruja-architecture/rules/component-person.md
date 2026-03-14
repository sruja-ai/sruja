# component-person

## Why It Matters

Person components represent **human** external actors that interact with your system. Use them only for users and stakeholders. External software (APIs, SaaS, third-party services) must be modeled as **system**, not person—so that C4 context diagrams correctly distinguish humans from software boundaries.

## When to Apply

Use **person** only for human actors:

- End users of the system (customers, guests)
- Administrators, operators, support staff
- Stakeholders who interact with or view the system (report viewers, managers)
- Developers or API consumers when they are human users

**Do not use person for:** external software systems, third-party APIs, SaaS backends, warehouses, or transform services. Use **system** for those, with optional `tags ["external"]` or `tags ["vendor"]` to mark them as outside your boundary.

## Correct Approach

### Example 1: User Types (humans only)

```sruja
EndUser = person "End User" {
  description "Team member using the application"
}

Admin = person "Administrator" {
  description "System administrator managing users and settings"
}

Viewer = person "Report Viewer" {
  description "External stakeholder viewing reports"
}
```

### Example 2: Humans vs external software

```sruja
// Humans: use person
Customer = person "Customer" {
  description "User making purchases"
}

// External software: use system (optional tags for clarity)
Stripe = system "Stripe" {
  description "External payment processing service"
  tags ["external", "vendor"]
}

AnalyticsService = system "Analytics Service" {
  description "External analytics and tracking"
  tags ["external"]
}
```

### Example 3: External systems as system, not person

```sruja
// ✅ Correct: external software = system
InternalInventory = system "Internal Inventory System" {
  description "Legacy inventory management system"
  tags ["external"]
}

LogisticsProvider = system "FedEx" {
  description "Shipping and logistics partner"
  tags ["external", "vendor"]
}

TaxService = system "Tax Calculation API" {
  description "External tax calculation service"
  tags ["external"]
}
```

## Incorrect Approach

```sruja
# ❌ Treating external services as containers (inside your system)
stripe = container "Stripe" {
  technology "Node.js"
  description "External service"
}

# ❌ Wrong: external software as person (person = human only)
stripe = person "Stripe" {
  description "External payment processing service"
}

# ✅ Correct: external software as system
Stripe = system "Stripe" {
  description "External payment processing service"
  tags ["external"]
}
```

## Common Mistakes

1. **Not Defining All Human Users**: Missing stakeholders, viewers, or admins
   - ❌ Only defining "User"
   - ✅ Define all human actor types: User, Admin, Viewer, Manager

2. **Using person for external software**
   - ❌ Stripe, Control Plane, Destinations, Transformer as person
   - ✅ Those as **system** with description and optional `tags ["external"]`

3. **Treating External Services as Containers**
   - ❌ Stripe as a container inside your system
   - ✅ Stripe as an external **system** (not person, not container)

4. **Vague Descriptions**: Not explaining the actor's role
   - ❌ "External service"
   - ✅ "External payment processing service handling transactions"

5. **Missing Integration Points**: Not showing relationships
   - ❌ Person or system with no relationships
   - ✅ Every external actor (person or system) has clear relationships to your systems

## Best Practices

1. **Use Clear, Descriptive Names**
   - ✅ "End User", "Administrator", "API Consumer" (for humans)
   - ❌ "User", "Admin", "Consumer"

2. **Explain Purpose in Description**
   - Include what they do with the system
   - Mention their goals or responsibilities

3. **Define Before Using**
   - All persons (and external systems) should be defined before relationships
   - Follow top-down: persons and external systems first, then relationships

4. **Reserve person for humans**
   - End users, administrators, stakeholders, developers (as users)
   - For payment gateways, control planes, destinations, transformers, warehouses: use **system**

## Additional Context

Person components (human actors) are entry points to your architecture. External **systems** (APIs, SaaS, backends) are also entry points but must be modeled as **system**, not person. Together they help identify:

- System boundaries (your system vs humans vs external software)
- Authentication and user roles
- API design needs and integration points
- Security considerations

Related rules:

- `component-system` - For major domain boundaries and **external software** (use system, not person)
- `relationship-synchronous` - For API interactions with persons or external systems
- `relationship-asynchronous` - For event-based interactions
- `tradeoff-sync-vs-async` - Choosing communication patterns

## References

- Use Case Modeling
- Actor-Based System Design
- Domain-Driven Design: Context Mapping