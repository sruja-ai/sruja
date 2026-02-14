---
title: "Lesson 1: Documenting Decisions (ADRs)"
weight: 1
summary: "Master architecture decision records: when to write them, how to structure them, and how to maintain them."
---

# Lesson 1: Documenting Decisions (ADRs)

We made the same architectural decision three times in one year.

**First time:** "We should use Elasticsearch for search. It'll be fast and scalable." We implemented it. Six months later, the team that built it left.

**Second time:** New team, same problem. "We need search. Let's use Elasticsearch." They implemented it their way. Three months later, they rotated to another project.

**Third time:** Yet another team. "Why don't we have search? We should use Elasticsearch."

I stopped them. "Wait. We've done this twice already. Why are we doing it again?"

The answer: **No one remembered why we made those decisions.** The first team's implementation had issues. The second team didn't know about the first. The third team didn't know about either.

We had code. We had documentation. But we had **no memory of our decisions**.

That's when I learned about Architecture Decision Records (ADRs). Not just writing them, but **living them** - making them part of how the team thinks and works.

This lesson is about mastering ADRs: not just what they are, but how to write them well, maintain them over time, and use them to build institutional memory.

## What is an ADR?

An **Architecture Decision Record (ADR)** is a document that captures an important architectural decision, including:

- **Context:** Why did we need to make this decision?
- **Decision:** What did we decide to do?
- **Consequences:** What are the trade-offs and impacts?

We covered ADRs briefly in Module 3 (Lesson 6) as part of advanced DSL features. This lesson is a **deep dive** into ADRs specifically - how to write them effectively, maintain them over time, and make them a natural part of your workflow.

### The Three Whys

Before writing any ADR, ask yourself:

1. **Why does this decision matter?** (If it doesn't matter, don't write an ADR)
2. **Why now?** (Is this the right time to make this decision?)
3. **Why will someone care in six months?** (If no one will care, reconsider)

If you can't answer these clearly, you might not need an ADR.

## The ADR Lifecycle

ADRs aren't static documents. They have a lifecycle that matches your decision-making process:

### Stage 1: Proposed

When someone has an architectural idea that needs discussion:

```sruja
ADR003 = adr "Use GraphQL for API layer" {
  status "Proposed"
  
  context "REST API is becoming complex. Clients need different data shapes. Over-fetching and under-fetching are common complaints."
  
  decision "Migrate to GraphQL for flexibility"
  
  consequences "Learning curve for team. Need to implement query complexity analysis. Caching becomes more complex."
  
  // Document alternatives considered
  option "REST with expansions" {
    pros "Team familiar with REST"
    cons "Still over/under-fetching, more endpoints"
  }
  
  option "GraphQL" {
    pros "Flexible queries, single endpoint, great tooling"
    cons "Learning curve, complexity analysis needed"
  }
}
```

**What happens at this stage:**
- ADR is created with status "Proposed"
- Team reviews and discusses
- Alternatives are documented
- Questions and concerns are raised

### Stage 2: Accepted

After team alignment and approval:

```sruja
ADR003 = adr "Use GraphQL for API layer" {
  status "Accepted"
  
  // ... context and decision remain the same ...
  
  accepted_date "2024-02-15"
  accepted_by "Architecture Review Board"
  
  decision "GraphQL with query complexity analysis"
  
  consequences "Learning curve for team (training budget allocated). Query complexity analysis required (use @cost directives). Caching via persisted queries."
  
  implementation_notes "Start with new endpoints only. Migrate existing REST endpoints gradually over 6 months."
}
```

**What changes:**
- Status changes to "Accepted"
- Specific decision details added
- Implementation notes added
- Acceptance metadata recorded

### Stage 3: Active (Implementation)

During implementation, the ADR guides work:

```sruja
GraphQL API = container "GraphQL API Service" {
  technology "Apollo Server"
  description "Implements ADR003 - GraphQL API layer"
  
  // Link ADR to implementation
  adr ADR003
  
  metadata {
    implementation_status "In Progress"
    target_completion "2024-06-01"
    team "Platform Team"
  }
}
```

**What happens:**
- ADR is referenced in code comments and PRs
- Implementation follows ADR decisions
- Deviations trigger ADR updates

### Stage 4: Deprecated

When circumstances change or better alternatives emerge:

```sruja
ADR001 = adr "Use MongoDB for all data storage" {
  status "Deprecated"
  
  // ... original context and decision ...
  
  deprecated_date "2024-05-20"
  deprecated_reason "Transaction requirements require relational database. See ADR005 for replacement."
  
  replacement ADR005
  
  migration_notes "Migrate to PostgreSQL (ADR005). MongoDB remains for specific use cases (see ADR006)."
}
```

**What triggers deprecation:**
- Better technology emerges
- Requirements change significantly
- Decision proven wrong by experience
- Team learns better approach

### Stage 5: Superseded

When a new ADR replaces an old one:

```sruja
ADR005 = adr "Use PostgreSQL for transactional data" {
  status "Accepted"
  supersedes ADR001
  
  context "MongoDB (ADR001) lacks strong transaction support. Financial data requires ACID guarantees."
  
  decision "Use PostgreSQL for all transactional data. MongoDB for document storage only."
  
  consequences "Two database technologies to maintain. Clear data ownership boundaries needed."
}
```

**The relationship:**
- Old ADR points to new ADR (replacement)
- New ADR points to old ADR (supersedes)
- Clear migration path documented
- Both remain in history

## ADR Structure: The Complete Template

Here's a comprehensive ADR template you can adapt:

```sruja
ADR### = adr "[Short Title]" {
  // REQUIRED: Current status
  status "Proposed" | "Accepted" | "Deprecated" | "Superseded"
  
  // REQUIRED: Decision context
  context "
    What is the issue/problem we're addressing?
    What constraints exist?
    What requirements must be met?
  "
  
  // REQUIRED: The decision
  decision "
    What is the change or action being proposed/taken?
    Be specific and actionable.
  "
  
  // REQUIRED: Impact analysis
  consequences {
    positive "
      What benefits do we expect?
      What becomes easier?
    "
    
    negative "
      What are the trade-offs?
      What becomes harder?
      What risks exist?
    "
    
    neutral "
      What are the side effects?
      What else changes?
    "
  }
  
  // OPTIONAL: Alternatives considered
  option "[Alternative 1]" {
    pros "Why was this attractive?"
    cons "Why didn't we choose this?"
    rejected_because "Specific reason for rejection"
  }
  
  option "[Alternative 2]" {
    pros "Benefits of this approach"
    cons "Drawbacks of this approach"
    rejected_because "Why not chosen"
  }
  
  // OPTIONAL: Implementation guidance
  implementation {
    who_responsible "Team or person"
    timeline "Expected completion"
    first_steps "What to do first"
    success_criteria "How to measure success"
  }
  
  // OPTIONAL: Metadata
  metadata {
    author "Name"
    created "YYYY-MM-DD"
    accepted_date "YYYY-MM-DD"
    review_date "YYYY-MM-DD"  // When to revisit
    related_adrs [ADR001, ADR002]  // Related decisions
    tags ["database", "performance", "security"]
  }
}
```

## Real-World ADR Examples

Let me show you some famous ADRs from real companies:

### Example 1: Netflix - Chaos Engineering

**Context:** Netflix needed to ensure resilience in their distributed system. Traditional testing wasn't enough.

**Decision:** Intentionally inject failures in production (Chaos Monkey).

**Consequences:**
- Positive: Higher confidence in resilience, proactive failure detection
- Negative: Potential customer impact during experiments, requires cultural shift
- Neutral: Need dedicated team, monitoring requirements

**Why this ADR is famous:** It challenged conventional wisdom. Most companies tried to prevent failures. Netflix decided to cause them.

### Example 2: Amazon - Two-Pizza Teams

**Context:** Communication overhead was slowing down development as teams grew.

**Decision:** Limit teams to 6-10 people (two pizzas can feed them). Each team owns their service end-to-end.

**Consequences:**
- Positive: Faster decisions, clear ownership, reduced coordination
- Negative: Potential duplication, need for clear APIs between teams
- Neutral: Requires decentralized architecture

**Why this ADR is famous:** It's simple but profound. Team size directly impacts architecture.

### Example 3: Google - Code Search Index

**Context:** Google's codebase was too large for traditional search tools.

**Decision:** Build custom code search with trigram indexes.

**Consequences:**
- Positive: Fast searches across billions of lines of code
- Negative: Significant infrastructure investment
- Neutral: Became internal tool, later open-sourced (Kythe)

**Why this ADR is famous:** Shows how scale forces custom solutions.

### Example 4: Stripe - API Versioning

**Context:** API changes were breaking existing integrations.

**Decision:** Every API change creates a new version. Clients pin to specific versions.

**Consequences:**
- Positive: No breaking changes for existing users
- Negative: Multiple API versions to maintain
- Neutral: Requires comprehensive testing across versions

**Why this ADR is famous:** Solved API evolution elegantly.

## When to Write an ADR

Not every decision needs an ADR. Here's the framework I use:

### Always Write an ADR:

**1. Technology Choices**
- Choosing a database (PostgreSQL vs. MongoDB vs. CockroachDB)
- Choosing a language/framework (Rust vs. Go, React vs. Vue)
- Choosing infrastructure (AWS vs. GCP, Kubernetes vs. ECS)

**Why:** These are expensive to change later. Document the reasoning.

**2. Architectural Patterns**
- Microservices vs. monolith
- Event-driven vs. request-response
- Sync vs. async processing

**Why:** These shape the entire system. Future architects need to understand why.

**3. Security Decisions**
- Authentication approach (OAuth, SAML, custom)
- Encryption standards (at rest, in transit)
- Compliance requirements (SOC2, HIPAA, GDPR)

**Why:** Security auditors will ask. Be prepared.

**4. Performance Standards**
- Latency SLAs (p95, p99)
- Throughput requirements
- Caching strategies

**Why:** These constrain implementation choices.

**5. Team Structure Decisions**
- How teams are organized
- Ownership boundaries
- Communication protocols

**Why:** This affects how architecture evolves.

### Sometimes Write an ADR:

**6. API Design Decisions**
- REST vs. GraphQL vs. gRPC
- Versioning strategy
- Error handling approach

**Write when:** API is public or long-lived
**Skip when:** Internal, short-lived API

**7. Data Model Decisions**
- Normalization strategy
- Schema vs. schemaless
- Data retention policies

**Write when:** Data is complex or critical
**Skip when:** Simple CRUD operations

**8. Deployment Decisions**
- CI/CD approach
- Deployment strategy (blue-green, canary)
- Environment management

**Write when:** Complex deployment requirements
**Skip when:** Simple deployment

### Rarely Write an ADR:

**9. Implementation Details**
- Variable naming conventions
- Code formatting rules
- Internal refactoring

**Why:** Too granular. Changes frequently. Use code comments instead.

**10. Temporary Decisions**
- Short-term workarounds
- Prototypes and experiments
- Quick fixes

**Why:** Won't matter in six months. Write a comment instead.

### The Decision Test

Ask: **"Will someone ask 'why did we do this?' in six months?"**

- Yes → Write an ADR
- Maybe → Consider writing an ADR
- No → Don't write an ADR

## ADR Anti-Patterns

### Anti-Pattern #1: The Hindsight ADR

**What happens:** You write ADRs for decisions made months or years ago.

**Why it fails:**
- Memory is unreliable
- Context is lost
- Alternatives forgotten
- Feels like homework

**The fix:** Write ADRs when decisions are made. If you must write retrospective ADRs, mark them clearly:

```sruja
ADR001 = adr "Use PostgreSQL" {
  status "Accepted"
  retrospective true  // This is a historical record
  context "This decision was made in 2022. Context reconstructed from git history and Slack archives."
}
```

### Anti-Pattern #2: The Obvious ADR

**What happens:** You write an ADR for a decision that doesn't need documentation.

```sruja
// DON'T DO THIS
ADR042 = adr "Use Git for version control" {
  context "We need version control"
  decision "Use Git because it's standard"
  consequences "Everyone knows Git"
}
```

**Why it fails:**
- Wastes time
- Dilutes importance of real ADRs
- Becomes noise

**The fix:** Use the decision test. If no one will ask "why?" in six months, don't write it.

### Anti-Pattern #3: The Novel ADR

**What happens:** ADRs are long, beautifully written essays that no one reads.

**Why it fails:**
- Too long to read quickly
- Key information buried
- Becomes documentation debt

**The fix:** Be concise. Use structure. Highlight key points. The best ADRs can be scanned in 2 minutes.

### Anti-Pattern #4: The Orphan ADR

**What happens:** ADR exists but isn't connected to the architecture or code.

**Why it fails:**
- ADRs and code drift apart
- No one knows ADR exists
- Decisions get re-made

**The fix:** Link ADRs to architecture:

```sruja
PaymentService = system "Payment Service" {
  adr ADR003  // "Use Stripe for payments"
  description "Implements payment processing per ADR003"
}
```

And reference in code:

```rust
// This implementation follows ADR003: Use Stripe for payments
// See: docs/adrs/ADR003-use-stripe-for-payments.md
pub fn process_payment(payment: Payment) -> Result<PaymentResult> {
    // ...
}
```

### Anti-Pattern #5: The Immutable ADR

**What happens:** ADRs are treated as unchangeable holy texts.

**Why it fails:**
- Circumstances change
- Decisions become outdated
- ADRs lose credibility

**The fix:** ADRs have a lifecycle. Update status, mark deprecated, supersede with new ADRs. Keep them current.

### Anti-Pattern #6: The Secret ADR

**What happens:** ADRs are written but not shared with the team.

**Why it fails:**
- Decisions aren't discussed
- Team doesn't buy in
- ADRs become one person's opinion

**The fix:** ADRs should be:
- Reviewed by team
- Discussed in architecture meetings
- Accessible to everyone
- Part of onboarding

## ADR Best Practices

### Practice #1: Number Your ADRs

Use sequential numbering: ADR001, ADR002, ADR003...

**Why:**
- Easy to reference
- Shows history
- Enables linking

**Don't use:**
- Dates (ADR-2024-03-15)
- Categories (ADR-DB-001)
- Both create confusion

### Practice #2: Use Consistent Titles

**Good titles:**
- "Use PostgreSQL for transactional data"
- "Implement rate limiting at API gateway"
- "Adopt event-driven architecture for order processing"

**Bad titles:**
- "Database decision" (too vague)
- "New architecture" (not specific)
- "Meeting notes from March" (not a decision)

### Practice #3: Include Decision Date

Always record when decisions were made:

```sruja
ADR003 = adr "Use GraphQL" {
  metadata {
    created "2024-01-15"
    accepted "2024-02-20"
    review_date "2024-08-20"  // 6 months later
  }
}
```

**Why:** Context changes over time. Knowing when helps evaluate if the decision is still valid.

### Practice #4: Document Alternatives

Always show what you considered and rejected:

```sruja
option "Alternative 1" {
  pros "Why it was attractive"
  cons "Why it had issues"
  rejected_because "Specific deal-breaker"
}
```

**Why:** Shows you did due diligence. Prevents re-litigating the decision later.

### Practice #5: Be Specific About Consequences

Don't just say "pros and cons". Be concrete:

```sruja
consequences {
  positive "
    - Reduces API calls by 60% (measured in prototype)
    - Enables client-side caching
    - Single endpoint simplifies monitoring
  "
  
  negative "
    - 2-week learning curve for team (training scheduled)
    - Query complexity analysis required (use Apollo Studio)
    - Caching less straightforward (implement persisted queries)
  "
  
  neutral "
    - New dependency: Apollo Server
    - Different testing approach needed
    - Monitoring tools need updates
  "
}
```

### Practice #6: Set Review Dates

Decisions aren't forever. Set review dates:

```sruja
metadata {
  review_date "2024-08-15"  // 6 months
  review_questions [
    "Is GraphQL meeting our needs?",
    "Are there new alternatives?",
    "Should we expand or restrict usage?"
  ]
}
```

### Practice #7: Make ADRs Searchable

Organize ADRs for discovery:

```
docs/
  adr/
    ADR001-use-postgresql.md
    ADR002-microservices-architecture.md
    ADR003-graphql-api-layer.md
    README.md  // Index of all ADRs
```

Create an index:

```markdown
# Architecture Decision Records

## Active ADRs
- [ADR003: Use GraphQL](ADR003-graphql-api-layer.md) - 2024-02-20
- [ADR002: Microservices](ADR002-microservices-architecture.md) - 2023-11-10

## Deprecated ADRs
- [ADR001: Use MongoDB](ADR001-use-mongodb.md) - Deprecated 2024-05-20

## By Category
### Database
- ADR001: Use MongoDB (Deprecated)
- ADR005: Use PostgreSQL for transactions

### API
- ADR003: Use GraphQL
```

### Practice #8: Review ADRs Regularly

Schedule quarterly ADR reviews:

**Review checklist:**
- [ ] Are all ADRs still relevant?
- [ ] Any need to be deprecated?
- [ ] Any need to be updated?
- [ ] Any new ADRs needed?
- [ ] Are ADRs being followed?

## ADR Review Process

ADRs should be reviewed before acceptance:

### The Lightweight Process (Small Teams)

1. **Author creates ADR** with status "Proposed"
2. **Share with team** in Slack/channel
3. **Discussion period** (2-3 days)
4. **Team consensus** → Update status to "Accepted"
5. **Implementation begins**

### The Formal Process (Larger Teams)

1. **Author creates ADR** with status "Proposed"
2. **Architecture review** in weekly meeting
3. **Feedback incorporated**
4. **Second review** if needed
5. **Approval** from architecture team → Status "Accepted"
6. **Implementation begins**

### The ADR Review Checklist

When reviewing an ADR, ask:

- [ ] Is the context clear? (Do I understand the problem?)
- [ ] Is the decision specific? (Do I know what to do?)
- [ ] Are consequences realistic? (Not just optimistic?)
- [ ] Were alternatives considered? (Did we do our homework?)
- [ ] Is it actionable? (Can someone implement this?)
- [ ] Is it necessary? (Does this need an ADR?)
- [ ] Is it reversible? (Can we change our minds later?)

## Complete Example: E-Commerce Payment ADR

Let me show you a complete, production-ready ADR:

```sruja
import { * } from 'sruja.ai/stdlib'

ADR003 = adr "Use Stripe for payment processing" {
  status "Accepted"
  
  context "
    Our e-commerce platform needs to process credit card payments. 
    Requirements:
    - Support major credit cards (Visa, MC, Amex)
    - Handle international currencies (USD, EUR, GBP, JPY)
    - PCI-DSS compliance
    - Recurring payments for subscriptions
    - Low latency (< 3s for payment authorization)
    
    Constraints:
    - Small team (3 engineers)
    - Need to launch in 6 weeks
    - Budget: $500/month + transaction fees
  "
  
  decision "Integrate Stripe for all payment processing"
  
  consequences {
    positive "
      - Faster time to market (2-3 weeks vs. 2-3 months for custom)
      - PCI-DSS compliance handled by Stripe (we don't store card numbers)
      - Excellent documentation and SDKs
      - Supports all required currencies
      - Built-in recurring payment support
      - Dashboard for finance team
    "
    
    negative "
      - Vendor lock-in (would take 2+ months to switch)
      - Transaction fees: 2.9% + $0.30 per transaction
      - Less control over payment flow
      - Stripe outages affect us (99.99% SLA, but still)
    "
    
    neutral "
      - New dependency in our stack
      - Finance team needs training on Stripe dashboard
      - Legal review of Stripe terms required
    "
  }
  
  option "PayPal" {
    pros "Recognized brand, easy setup"
    cons "Higher fees (3.49% + $0.49), less developer-friendly, limited customization"
    rejected_because "Higher fees at scale, worse API experience"
  }
  
  option "Adyen" {
    pros "Lower fees at scale, more payment methods"
    cons "Longer integration time, more complex setup, overkill for our size"
    rejected_because "Overkill for our current scale, 6-week timeline too tight"
  }
  
  option "Build custom" {
    pros "No vendor lock-in, complete control"
    cons "PCI-DSS compliance burden, 3+ months to build, ongoing maintenance"
    rejected_because "Timeline constraints, PCI compliance too risky for small team"
  }
  
  implementation {
    who_responsible "Payments Team"
    timeline "3 weeks"
    first_steps "
      1. Create Stripe account
      2. Complete legal review of terms
      3. Implement payment intent flow
      4. Set up webhook handlers
      5. Test with Stripe test cards
      6. Security review
      7. Production deployment
    "
    success_criteria "
      - Process test payments successfully
      - < 3s authorization time
      - All webhooks handled correctly
      - Finance team can generate reports
    "
  }
  
  metadata {
    author "Jane Smith"
    created "2024-02-10"
    accepted "2024-02-15"
    review_date "2024-08-15"
    tags ["payments", "vendor", "compliance"]
    related_adrs []
  }
}

// Link ADR to architecture
PaymentService = system "Payment Service" {
  PaymentAPI = container "Payment API" {
    technology "Node.js"
    description "Implements ADR003: Stripe integration"
  }
}

// Mark the relationship
PaymentService.PaymentAPI -> ADR003 "Implements"

view index {
  include *
}
```

## What to Remember

1. **ADRs are memory, not paperwork** - They preserve decision context for future you and future teams

2. **ADRs have a lifecycle** - Proposed → Accepted → Active → Deprecated → Superseded

3. **Write ADRs when decisions are made** - Not months later when memory fades

4. **Be specific about consequences** - Concrete pros/cons/neutral impacts, not vague statements

5. **Document alternatives** - Show you did due diligence, prevent re-litigation

6. **Set review dates** - Circumstances change, revisit decisions periodically

7. **Number sequentially** - ADR001, ADR002... not dates or categories

8. **Link to architecture** - ADRs should be referenced in code and architecture models

9. **Keep them concise** - 2-minute scan, not 20-minute read

10. **Share with the team** - ADRs are team artifacts, not personal journals

## When Not to Write an ADR

Don't write an ADR when:

- **Decision is obvious** - "We'll use Git for version control"
- **Decision is temporary** - "Quick fix for production bug"
- **Decision is reversible with low cost** - "Let's try this library"
- **Decision is purely implementation** - "We'll use camelCase for variables"

Write an ADR when:

- **Decision has lasting impact** - Database choice, architecture pattern
- **Decision is expensive to reverse** - Major refactoring, vendor lock-in
- **Decision affects multiple teams** - API contracts, shared infrastructure
- **Decision will be questioned** - Non-obvious choice, trade-offs involved

## Practical Exercise

Create an ADR for a decision in your current or recent project:

**Step 1:** Identify a significant architectural decision
- Technology choice
- Pattern adoption
- Structural change
- Security approach

**Step 2:** Use the complete template:
- Context (problem, constraints, requirements)
- Decision (specific, actionable)
- Consequences (positive, negative, neutral)
- Alternatives considered (at least 2)
- Implementation notes
- Metadata (dates, author, tags)

**Step 3:** Review with your team
- Does everyone understand the context?
- Are consequences realistic?
- Were alternatives adequately considered?

**Step 4:** Link to your architecture
- Reference the ADR in your Sruja model
- Add code comments pointing to the ADR
- Include in onboarding documentation

## ADR Maturity Model

Where is your team on the ADR journey?

**Level 0: No ADRs**
- Decisions live in Slack, email, or people's heads
- Re-litigating decisions is common
- New hires struggle to understand "why"

**Level 1: Reactive ADRs**
- ADRs written when someone asks "why?"
- Usually retrospective (memory reconstruction)
- Inconsistent format
- Not linked to code/architecture

**Level 2: Proactive ADRs**
- ADRs written when decisions are made
- Consistent format
- Reviewed by team
- Some links to code/architecture

**Level 3: Living ADRs**
- ADRs have clear lifecycle (proposed → accepted → deprecated)
- Regularly reviewed and updated
- Fully integrated with code and architecture
- Part of team culture and onboarding

**Level 4: ADR-Driven Development**
- ADRs written before implementation
- ADRs drive architecture reviews
- ADRs inform technical strategy
- ADRs are strategic assets

**Goal:** Reach Level 3. Level 4 is aspirational for most teams.

---

**Next up:** Lesson 2 explores production readiness patterns - how to make your architecture resilient, observable, and maintainable in production environments.