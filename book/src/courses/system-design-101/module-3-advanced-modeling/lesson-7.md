title: "Lesson 7: Views Best Practices"
weight: 7
summary: "Master the art of creating effective views for different audiences and purposes."
---

# Lesson 7: Views Best Practices

I inherited a project with 47 views. Yes, forty-seven.

Each had a name like "view1", "temp", "test-final", "new-view-v2", and my personal favorite: "DO-NOT-DELETE". No descriptions. No documentation. Half of them showed the same thing with slight variations.

When I asked the team which view to show executives, they said: "Probably view12? Or maybe view18? Definitely not view23 - that one's outdated."

I spent two days consolidating those 47 views into 7 coherent perspectives. Two days of archaeology, trying to understand why each view existed, what audience it served, and whether it was still relevant.

Here's what I learned: **views are powerful, but without governance, they become technical debt**. This lesson is about managing views at scale - not just creating them, but maintaining them, organizing them, and knowing when to kill them.

## The VIEW Governance Framework

After that disaster, I created a framework for view management. I call it VIEW:

**V - Verify the Audience**
- Who needs this view?
- What decisions will they make?
- What detail level do they need?

**I - Intentionally Design**
- What's the specific purpose?
- What should be included/excluded?
- How will it be maintained?

**E - Explicitly Document**
- Clear, descriptive name
- Purpose statement
- When to use (and not use)

**W - Watch Lifecycle**
- Review regularly
- Update when architecture changes
- Delete when no longer needed

Let me break down each piece.

---

## When to Create Views (Decision Framework)

The mistake I made early in my career was creating views for everything. Every meeting, every question, every discussion resulted in a new view. That's how you get to 47 views.

Here's the framework I use now:

### Always Create:

**1. Executive View**
- **Purpose:** Show business value and scope
- **Audience:** C-suite, board members, investors
- **Includes:** Systems, external dependencies, users
- **Excludes:** Technical implementation details
- **Update frequency:** When business scope changes

**2. Architect View**
- **Purpose:** Show service boundaries and tech stack
- **Audience:** Architects, tech leads, senior engineers
- **Includes:** Containers, databases, external systems
- **Excludes:** Business context, implementation details
- **Update frequency:** When architecture changes

**3. Developer View**
- **Purpose:** Show implementation details
- **Audience:** Developers building features
- **Includes:** Components, internal structure, key dependencies
- **Excludes:** External systems, high-level architecture
- **Update frequency:** When implementation patterns change

### Create Conditionally:

**4. Security View**
- **Create when:** Handling sensitive data, compliance requirements
- **Skip when:** Internal tools with no sensitive data
- **Includes:** Data stores, external integrations, trust boundaries
- **Excludes:** UI components, non-sensitive services

**5. Performance View**
- **Create when:** Performance is critical, have SLAs/SLOs
- **Skip when:** Simple CRUD with no performance requirements
- **Includes:** Databases, caches, high-traffic components
- **Excludes:** Admin tools, low-traffic features

**6. Data Flow View**
- **Create when:** Complex data pipelines, multiple data stores
- **Skip when:** Single database, simple data access patterns
- **Includes:** Data stores, data transformation services
- **Excludes:** UI components, business logic services

**7. Deployment View**
- **Create when:** Complex deployment, multiple environments
- **Skip when:** Simple deployment (single service + database)
- **Includes:** Containers, infrastructure, deployment pipelines
- **Excludes:** Business context, implementation details

**8. Feature-Specific Views**
- **Create when:** Complex feature spanning multiple services
- **Skip when:** Feature contained in single service
- **Includes:** Services/components involved in feature
- **Excludes:** Unrelated parts of system

### Rarely Create:

**9. User Journey Views**
- **Usually better as:** Scenarios (see Lesson 6)
- **Create as view when:** Static diagram for presentation
- **Create as scenario when:** Documenting behavior flow

**10. Integration Views**
- **Usually better as:** Part of architect view
- **Create separately when:** Many external integrations

## View Lifecycle Management

Views aren't static. They're living documentation that needs maintenance. Here's the lifecycle:

### Stage 1: Creation

```sruja
// partial
view security {
  title "Security Architecture"
  description "Shows trust boundaries, data encryption, and external integrations. Use for security reviews and compliance audits."
  
  include ECommerce.API
  include PaymentGateway
  include ECommerce.OrderDB
  exclude Customer Admin ECommerce.WebApp
  
  // Document when created
  metadata {
    created "2024-01-15"
    owner "Security Team"
    review_frequency "quarterly"
  }
}
```

**Best practices at creation:**
- ✅ Clear, descriptive name
- ✅ Descriptive title
- ✅ Purpose description
- ✅ Owner (who maintains it)
- ✅ Review frequency

### Stage 2: Active Maintenance

Views need updates when:
- Architecture changes (new services, removed services)
- Audience needs change (new questions, different decisions)
- Scope changes (system grows, features added)

**Update checklist:**
- [ ] Do included elements still exist?
- [ ] Should new elements be included?
- [ ] Is the purpose still valid?
- [ ] Is the audience still the same?

### Stage 3: Deprecation

**Signs a view should be deprecated:**
- ❌ No one's used it in 6 months
- ❌ It duplicates another view
- ❌ The audience no longer exists (team disbanded, project canceled)
- ❌ The questions it answered are no longer relevant

**Deprecation process:**
1. Mark as deprecated
2. Add deprecation date and reason
3. Point to replacement view (if any)
4. Keep for 30 days, then delete

```sruja
// partial
view old-performance {
  title "Performance View (DEPRECATED)"
  description "DEPRECATED: Use 'performance-v2' instead. This view doesn't include the new caching layer."
  
  metadata {
    deprecated "2024-03-15"
    replacement "performance-v2"
    reason "Doesn't reflect current caching architecture"
  }
  
  // ... rest of view
}
```

### Stage 4: Deletion

**Delete when:**
- Deprecated period has passed (30+ days)
- No one has complained
- Replacement view exists and is used

**Before deleting:**
- Search for references in documentation
- Check if anyone's bookmarked it
- Announce deletion to team

## Naming Conventions That Scale

Bad names kill view discoverability. Here's what works:

### Pattern 1: Audience-Based Names

```sruja
// partial
view executive { /* for executives */ }
view product { /* for product managers */ }
view architect { /* for architects */ }
view developer { /* for developers */ }
view operations { /* for ops team */ }
```

**When to use:** Most common pattern, works for 80% of views

### Pattern 2: Concern-Based Names

```sruja
// partial
view security { /* security focus */ }
view performance { /* performance focus */ }
view dataflow { /* data dependencies */ }
view deployment { /* deployment architecture */ }
```

**When to use:** When concern is more important than audience

### Pattern 3: Feature-Based Names

```sruja
// partial
view checkout { /* checkout feature */ }
view search { /* search functionality */ }
view analytics { /* analytics pipeline */ }
view notifications { /* notification system */ }
```

**When to use:** Large features spanning multiple services

### Pattern 4: Layer-Based Names

```sruja
// partial
view context { /* C4 context layer */ }
view containers { /* C4 container layer */ }
view components { /* C4 component layer */ }
```

**When to use:** When following strict C4 model

### Naming Anti-Patterns

❌ **Temporal names:** `view1`, `view2`, `new-view`, `old-view`
- Problem: Doesn't say what it is, only when it was created

❌ **Person names:** `johns-view`, `sarahs-diagram`
- Problem: What if John leaves? What if Sarah changes roles?

❌ **Temporal qualifiers:** `temp`, `test`, `draft`, `wip`
- Problem: These never get renamed. They become permanent.

❌ **Cryptic abbreviations:** `mv`, `arch-v2`, `dev-fin`
- Problem: Requires mental translation every time

✅ **Clear, descriptive names:** `executive`, `security-audit`, `checkout-flow`
- Solution: Self-documenting, searchable, scalable

## Organization Patterns for Large Systems

When your architecture grows beyond 10 services, you need organization strategies.

### Pattern 1: The Core 5

Every system should have these 5 views minimum:

```sruja
// partial
view index { /* Complete system */ }
view executive { /* Business context */ }
view architect { /* Technical architecture */ }
view developer { /* Implementation details */ }
view security { /* Security & compliance */ }
```

These cover 90% of use cases.

### Pattern 2: Concern Extensions

Add concern-specific views as needed:

```sruja
// partial
view performance { /* When performance matters */ }
view dataflow { /* When data pipelines exist */ }
view deployment { /* When deployment is complex */ }
view integration { /* When many external APIs */ }
```

### Pattern 3: Feature Slices

For large systems, create feature-specific views:

```sruja
// partial
view checkout { /* Checkout feature */ }
view search { /* Search feature */ }
view recommendations { /* Recommendations feature */ }
```

### Pattern 4: Team Domains

For orgs with domain-driven design:

```sruja
// partial
view catalog-domain { /* Catalog team */ }
view order-domain { /* Order team */ }
view payment-domain { /* Payment team */ }
view user-domain { /* User team */ }
```

## Real-World Case Studies

### Netflix: View Governance at Scale

Netflix has 700+ microservices. They can't afford view chaos. Their approach:

**1. Mandatory Views (enforced by CI/CD):**
- Executive view (business context)
- Domain view (service groupings)
- Dependencies view (critical paths)

**2. Optional Views (team discretion):**
- Feature-specific views
- Performance views
- Deployment views

**3. View Review Process:**
- Quarterly review of all views
- Delete unused views
- Update stale views
- Add missing views

**Result:** Despite 700+ services, they maintain ~15 core views. Focused, not fragmented.

### Amazon: Two-Pizza Team Views

Amazon's two-pizza teams (6-10 people) each own their services. Their view strategy:

**1. Team Views:**
- Each team maintains 3-5 views for their domain
- Team is responsible for keeping views updated

**2. Cross-Team Views:**
- Central architecture team maintains integration views
- Shows how teams connect

**3. Executive Views:**
- Aggregated from team views
- Shows business capabilities, not services

**Result:** Distributed ownership prevents central bottleneck. Clear ownership keeps views accurate.

### Stripe: Developer-Focused Views

Stripe's famous for developer experience. Their view strategy:

**1. Public Documentation Views:**
- Customer-facing integration views
- Minimal, clear, focused

**2. Internal Developer Views:**
- Detailed implementation views
- Component-level for feature teams

**3. Compliance Views:**
- Security and audit views
- Separate from technical views

**Result:** Different views for different audiences. No confusion about which view to use.

## Common View Management Mistakes

### Mistake #1: View Proliferation

**What happens:** Every meeting spawns a new view. 50+ views exist.

**Why it fails:**
- Can't find the right view
- Maintenance nightmare
- Outdated information everywhere

**The fix:**
- Start with Core 5
- Require justification for new views
- Quarterly view audit and cleanup

### Mistake #2: Orphaned Views

**What happens:** View created for a specific meeting, never maintained.

**Why it fails:**
- Shows outdated architecture
- Misleads people who find it
- Erodes trust in documentation

**The fix:**
- Every view needs an owner
- Document creation date and purpose
- Regular review schedule

### Mistake #3: Generic Views

**What happens:** One view tries to serve everyone.

**Why it fails:**
- Too much detail for executives
- Too little detail for developers
- Satisfies no one

**The fix:**
- Create audience-specific views
- Be explicit about who each view serves
- Don't try to make one view do everything

### Mistake #4: No Default View

**What happens:** Someone opens your model and doesn't know which view to use.

**Why it fails:**
- Confusion from the start
- Wrong view used for wrong purpose
- Frustration with documentation

**The fix:**
- Always have `view index` with `include *`
- Make default view match most common use case
- Document other views in default view's description

### Mistake #5: Stale Views After Refactoring

**What happens:** Architecture gets refactored, views don't get updated.

**Why it fails:**
- Views show services that no longer exist
- New services missing from views
- Documentation lies

**The fix:**
- View updates part of refactoring PRs
- CI/CD validation of view elements
- Architecture review includes view review

### Mistake #6: Inconsistent Styling Across Views

**What happens:** Same element looks different in different views.

**Why it fails:**
- Confusing for viewers
- Looks unprofessional
- Harder to understand

**The fix:**
- Use global styles for consistency
- View-specific styles only for emphasis
- Style guide for team

## The VIEW-SCALE Framework

For managing views at scale, use this framework:

**V - Verify Need**
- Does this view serve a specific audience?
- Does it answer questions not answered by existing views?
- Will it be maintained?

**I - Identify Audience**
- Who will use this view?
- What decisions will they make?
- What's their technical level?

**E - Establish Purpose**
- What questions does this view answer?
- What's out of scope?
- When should this view be used?

**W - Write Documentation**
- Clear name and title
- Purpose description
- Owner and review schedule

**S - Scale Check**
- Does this duplicate existing views?
- Can existing views be extended instead?
- Will this add value or noise?

**C - Create Minimal**
- Start with essential elements only
- Add more if needed
- Less is more

**A - Assign Owner**
- Who's responsible for maintenance?
- What triggers updates?
- When should it be deprecated?

**L - Lifecycle Plan**
- Review frequency
- Update triggers
- Deprecation criteria

**E - Evaluate Regularly**
- Is it still used?
- Is it still accurate?
- Should it be updated or deleted?

## Practical Example: E-Commerce at Scale

Let me show you a well-organized view structure for a growing e-commerce platform:

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"
Admin = person "Administrator"

ECommerce = system "E-Commerce Platform" {
  WebApp = container "Web Application" {
    CartComponent = component "Shopping Cart"
    ProductComponent = component "Product Catalog"
  }
  API = container "API Service" {
    OrderController = component "Order Controller"
    PaymentController = component "Payment Controller"
  }
  OrderDB = database "Order Database"
  ProductDB = database "Product Database"
  Cache = database "Redis Cache"
  EventQueue = queue "Event Queue"
}

PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}

Customer -> ECommerce.WebApp "Browses"
ECommerce.WebApp -> ECommerce.API "Fetches data"
ECommerce.API -> ECommerce.OrderDB "Stores orders"
ECommerce.API -> ECommerce.Cache "Caches queries"
ECommerce.API -> PaymentGateway "Processes payments"

// CORE 5 VIEWS (always present)

view index {
  title "Complete System"
  description "Full architecture with all elements. Use for reference, not presentations."
  include *
}

view executive {
  title "Business Overview"
  description "High-level business context for executives and stakeholders. Shows business capabilities and external dependencies."
  
  include Customer Admin
  include ECommerce PaymentGateway
  
  // Hide all technical details
  exclude ECommerce.WebApp ECommerce.API 
  exclude ECommerce.OrderDB ECommerce.ProductDB 
  exclude ECommerce.Cache ECommerce.EventQueue
  
  metadata {
    audience "C-suite, Board, Investors"
    owner "Product Team"
    review "quarterly"
  }
}

view architect {
  title "Technical Architecture"
  description "Container-level architecture showing service boundaries, data stores, and external integrations."
  
  include ECommerce
  include ECommerce.WebApp ECommerce.API
  include ECommerce.OrderDB ECommerce.ProductDB 
  include ECommerce.Cache ECommerce.EventQueue
  include PaymentGateway
  
  exclude Customer Admin
  
  metadata {
    audience "Architects, Tech Leads"
    owner "Architecture Team"
    review "monthly"
  }
}

view developer {
  title "Developer Guide"
  description "Implementation details for developers. Shows components, internal structure, and key dependencies."
  
  include ECommerce.WebApp
  include ECommerce.API
  include ECommerce.OrderDB ECommerce.ProductDB ECommerce.Cache
  
  exclude Customer Admin PaymentGateway
  
  metadata {
    audience "Developers"
    owner "Engineering Teams"
    review "sprint"
  }
}

view security {
  title "Security Architecture"
  description "Security focus: trust boundaries, data encryption, external integrations. Use for security reviews and compliance audits."
  
  include ECommerce.API
  include PaymentGateway
  include ECommerce.OrderDB
  
  exclude Customer Admin ECommerce.WebApp
  exclude ECommerce.ProductDB ECommerce.Cache ECommerce.EventQueue
  
  metadata {
    audience "Security Team, Compliance"
    owner "Security Team"
    review "quarterly"
  }
}

// EXTENSION VIEWS (added as needed)

view performance {
  title "Performance Architecture"
  description "Performance-critical components: databases, caches, high-traffic services. Use for capacity planning and optimization."
  
  include ECommerce.API
  include ECommerce.Cache
  include ECommerce.OrderDB ECommerce.ProductDB
  
  exclude Customer Admin PaymentGateway ECommerce.WebApp ECommerce.EventQueue
  
  metadata {
    audience "Performance Team, SRE"
    owner "Platform Team"
    review "monthly"
  }
}

view dataflow {
  title "Data Flow"
  description "Data dependencies and movement. Shows how data flows through the system. Use for data architecture discussions."
  
  include ECommerce.API
  include ECommerce.OrderDB ECommerce.ProductDB 
  include ECommerce.Cache ECommerce.EventQueue
  
  exclude Customer Admin ECommerce.WebApp PaymentGateway
  
  metadata {
    audience "Data Team, Architects"
    owner "Data Team"
    review "monthly"
  }
}

view deployment {
  title "Deployment Architecture"
  description "Deployment and infrastructure. Shows containers and their deployment targets. Use for deployment planning."
  
  include ECommerce.WebApp ECommerce.API
  include ECommerce.OrderDB ECommerce.ProductDB 
  include ECommerce.Cache ECommerce.EventQueue
  
  exclude Customer Admin PaymentGateway
  
  metadata {
    audience "DevOps, SRE"
    owner "Platform Team"
    review "monthly"
  }
}

view checkout {
  title "Checkout Feature"
  description "Checkout flow components. Shows services and components involved in checkout process. Use for checkout feature development."
  
  include Customer
  include ECommerce.WebApp ECommerce.WebApp.CartComponent
  include ECommerce.API ECommerce.API.OrderController ECommerce.API.PaymentController
  include ECommerce.OrderDB
  include PaymentGateway
  
  exclude Admin ECommerce.ProductDB ECommerce.Cache ECommerce.EventQueue
  
  metadata {
    audience "Checkout Team"
    owner "Checkout Team"
    review "sprint"
  }
}
```

**Notice the organization:**
- Core 5 views first (always present)
- Extension views grouped by concern
- Feature views at the end
- Each view has metadata: audience, owner, review schedule
- Clear descriptions of when to use

## View Maintenance Checklist

Use this checklist monthly:

**Review:**
- [ ] Are all view elements still valid (no deleted services)?
- [ ] Are new elements missing from relevant views?
- [ ] Are view descriptions still accurate?
- [ ] Are owners still responsible?

**Clean:**
- [ ] Any views unused for 3+ months? → Mark deprecated
- [ ] Any deprecated views older than 30 days? → Delete
- [ ] Any duplicate views? → Consolidate
- [ ] Any missing critical views? → Create

**Validate:**
- [ ] Do views render correctly?
- [ ] Are styles consistent?
- [ ] Are names clear and descriptive?
- [ ] Is documentation current?

## What to Remember

1. **Start with Core 5** - Executive, Architect, Developer, Security, Index
2. **Use the VIEW framework** - Verify audience, Intentionally design, Explicitly document, Watch lifecycle
3. **Name for clarity** - Audience or concern, never temporal or personal
4. **Assign ownership** - Every view needs an owner and review schedule
5. **Lifecycle matters** - Create → Maintain → Deprecate → Delete
6. **Less is more** - 7 focused views > 47 scattered views
7. **Review quarterly** - Audit, clean, and validate views regularly
8. **Document purpose** - When to use, what it shows, who it's for
9. **Avoid duplication** - Consolidate similar views, extend existing views
10. **Update with architecture** - Refactoring includes view updates

## When You're Tempted to Create Another View

Ask yourself:

1. **Does this audience already have a view?** → Update existing view
2. **Is this a temporary need?** → Don't create, use ad-hoc screenshot
3. **Can this be a scenario or flow instead?** → Use right tool for job
4. **Will this be maintained?** → If no, don't create
5. **Is this worth the maintenance cost?** → Every view has a cost

## The 7-View Rule

From experience: **most systems need 5-10 views, maximum**.

- **5 views:** Simple system, one team
- **7 views:** Medium system, 2-3 teams
- **10 views:** Large system, multiple teams
- **15+ views:** Very large system OR view proliferation

If you have more than 10 views, question whether they're all necessary. Consolidate, deprecate, delete.

## Summary

Views are powerful, but power requires responsibility. A few well-maintained views are worth more than dozens of orphaned, outdated perspectives.

**The formula:**
- Start with Core 5
- Add only when justified
- Maintain with discipline
- Clean up regularly
- Document clearly

Your future self (and your team) will thank you.

---

**Congratulations!** You've completed Module 3: Advanced Modeling. You now have all the tools to create production-ready architecture models.

👉 **[Module 4: Production Readiness](../module-4-production-readiness/lesson-1)** - Learn how to make your architecture production-ready with real-world examples and best practices.
