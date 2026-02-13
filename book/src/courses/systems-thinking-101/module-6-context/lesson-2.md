title: "Lesson 2: Stakeholders"
weight: 2
summary: "The people you forgot to ask are the ones who'll block your launch"
time: "7 min"
---

# The Hidden Stakeholder Problem: Why Everyone Matters (Even People You've Never Met)

We launched the new admin dashboard on a Tuesday. By Wednesday, the project was in crisis.

The dashboard was beautiful—sleek, modern, exactly what the product team wanted. The developers had done great work. But within 24 hours, we had three critical problems:

The support team couldn't find customer information quickly enough. The old dashboard had a search box right at the top; the new one buried it three levels deep. Support ticket resolution time doubled.

The compliance team realized the new audit logs didn't capture user IP addresses, which was required for their quarterly reports. They couldn't sign off on the release.

The finance team discovered that the revenue reports they'd been getting automatically every Monday morning were now manual—and nobody had told them.

We'd spent three months building the perfect product for the product team. We'd forgotten that three other teams also depended on the system.

The launch was delayed six weeks while we added features for stakeholders we'd never even talked to.

This lesson is about avoiding that mistake. You'll learn how to identify every stakeholder who matters (including the hidden ones), understand their competing needs, and model them in a way that prevents surprises.

## Learning Goals

By the end of this lesson, you'll be able to:

- Identify all stakeholder types, not just the obvious ones
- Recognize why different stakeholders often have conflicting needs
- Model stakeholder relationships and interactions in Sruja
- Document stakeholder needs, pain points, and priorities
- Avoid the "hidden stakeholder" problem that derails projects

## Stakeholders: The Real System Owners

Here's a truth that took me years to learn: **Users don't own systems. Stakeholders do.**

A user is someone who interacts with your system. A stakeholder is anyone affected by it or who can affect it. That's a much bigger group.

Think about an e-commerce platform:

- **Customers** use it to buy things (users AND stakeholders)
- **Administrators** use it to manage products (users AND stakeholders)
- **Business owners** never touch it but depend on its revenue (stakeholders, not users)
- **Support agents** need data from it but might not log in directly (stakeholders, not necessarily users)
- **Compliance officers** audit it but don't use it (stakeholders, not users)
- **Developers** maintain it (stakeholders, users of a different kind)

Miss any of these, and you're building an incomplete picture. Each has different needs, different priorities, different success criteria.

## The Five Stakeholder Types (And Why They Conflict)

After years of stakeholder surprises, I've learned to look for five specific groups. Each sees the system differently.

### 1. Primary Users: The People You Think About

These are the direct users—the ones product teams interview, the ones in user stories, the ones you're probably already thinking about.

**Who they are:** Customers, administrators, anyone who logs in and clicks buttons.

**What they want:** Speed, ease of use, features that help them do their job.

**Example in Sruja:**

```sruja
Customer = person "Customer" {
  description "Shoppers who purchase products"
  metadata {
    needs ["Fast checkout", "Easy search", "Order tracking"]
    pain_points ["Complex forms", "Slow page loads"]
    usage "Daily, mostly mobile"
  }
}
```

**The trap:** It's easy to focus only on primary users and forget everyone else.

### 2. Secondary Users: The People Who Need Your Data

These users don't interact with your system directly, but they depend on its outputs—reports, data exports, APIs.

**Who they are:** Support teams, analysts, people who receive automated reports.

**What they want:** Data access, clear reports, reliable exports.

**Example:**

```sruja
SupportAgent = person "Support Agent" {
  description "Helps customers with order issues"
  metadata {
    needs ["Quick customer lookup", "Order history", "Ability to modify orders"]
    pain_points ["Can't find customer data", "Too many clicks to resolve issues"]
    usage "Uses admin tools to look up information"
  }
}
```

**The trap:** These users are invisible until something breaks. I've seen launches delayed because the weekly report that "nobody uses" suddenly turns out to be critical for the CEO.

### 3. Business Stakeholders: The People Who Pay For It

These are the decision-makers, budget-owners, and revenue-responsible people. They might never use your system, but they decide if it succeeds.

**Who they are:** Product managers, business owners, executives, finance teams.

**What they want:** Revenue, metrics, ROI, competitive advantage.

**Example:**

```sruja
BusinessOwner = person "Business Owner" {
  description "Accountable for revenue and profit"
  metadata {
    needs ["Revenue reports", "Conversion metrics", "Cost tracking"]
    concerns ["Is the system making money?", "Are customers happy?", "What's the ROI?"]
    success_criteria "10% increase in conversion rate"
  }
}

ProductManager = person "Product Manager" {
  description "Owns product strategy and roadmap"
  metadata {
    needs ["User analytics", "Feature usage data", "A/B test results"]
    concerns ["Are users adopting features?", "What should we build next?"]
  }
}
```

**The trap:** Business stakeholders often have goals that conflict with user experience. Fast checkout might reduce revenue (fewer impulse buys). Easy returns might increase costs. You need to model these tensions explicitly.

### 4. Technical Stakeholders: The People Who Build and Run It

These are your teammates—the developers, DevOps engineers, DBAs, security teams. The system affects their daily work.

**Who they are:** Developers, operations teams, security engineers, database administrators.

**What they want:** Clear architecture, good documentation, easy deployment, monitoring.

**Example:**

```sruja
Developer = person "Developer" {
  description "Builds and maintains the system"
  metadata {
    needs ["Clear architecture docs", "API documentation", "Debugging tools"]
    pain_points ["Unclear requirements", "Technical debt", "Poor test coverage"]
  }
}

DevOpsEngineer = person "DevOps Engineer" {
  description "Deploys and operates the system"
  metadata {
    needs ["Monitoring dashboards", "Easy deployment", "Clear logs"]
    pain_points ["Manual deployments", "Poor observability"]
  }
}
```

**The trap:** Technical stakeholders often get ignored in architecture diagrams, but their needs are real. A system that's perfect for users but impossible to operate is a failed system.

### 5. Compliance and Governance: The People Who Can Say "No"

These stakeholders can block your launch. They don't use the system, but they regulate it.

**Who they are:** Compliance officers, security auditors, legal teams, data privacy officers.

**What they want:** Audit trails, data protection, regulatory compliance.

**Example:**

```sruja
ComplianceOfficer = person "Compliance Officer" {
  description "Ensures regulatory compliance"
  metadata {
    needs ["Audit logs", "Data retention policies", "Access controls"]
    requirements ["PCI-DSS", "GDPR", "SOX"]
    can_block_launch true
  }
}

SecurityAuditor = person "Security Auditor" {
  description "Reviews security posture"
  metadata {
    needs ["Vulnerability reports", "Penetration test results", "Access logs"]
    concerns ["Data breaches", "Unauthorized access", "Injection attacks"]
  }
}
```

**The trap:** These stakeholders are invisible until they're not. I've seen projects delayed months because compliance requirements were discovered too late.

## A Real Stakeholder Conflict (And How We Solved It)

Let me share a specific example that taught me why stakeholder modeling matters.

**The situation:** We were building a customer support dashboard. The product team wanted a clean, minimal interface—fewer buttons, more white space, "Apple-like" design.

**The conflict:** Support agents needed dense information displays. They handled 50+ tickets per day and couldn't afford extra clicks. What product called "cluttered," support called "efficient."

**The mistake:** We designed for product's vision first. Support hated it.

**The solution:** We modeled both stakeholders explicitly:

```sruja
ProductManager = person "Product Manager" {
  metadata {
    vision "Clean, minimal, modern interface"
    priority "User experience, simplicity"
  }
}

SupportAgent = person "Support Agent" {
  metadata {
    needs ["Dense information display", "Minimal clicks", "Keyboard shortcuts"]
    metric "50+ tickets per day"
    priority "Speed over aesthetics"
  }
}

// Make the conflict explicit
ProductManager -> SupportDashboard "Wants clean interface"
SupportAgent -> SupportDashboard "Needs dense information"
```

Making the conflict visible in the architecture forced a conversation. The solution was **modes**: a "standard" view for occasional users and a "power user" view for support agents. Both stakeholders got what they needed, but only because we'd modeled the conflict explicitly.

## Documenting Stakeholders in Sruja

Sruja gives you multiple ways to capture stakeholder information. Here's what works best for each situation.

### Basic Stakeholder Declaration

```sruja
Customer = person "Customer"
```

Simple and clear. Use this when you just need to show that someone exists.

### Detailed Stakeholder Profile

```sruja
Customer = person "Customer" {
  description "End users who purchase products"
  metadata {
    tags ["primary-user", "external"]
    priority "critical"
    needs [
      "Fast and easy checkout",
      "Product search and filtering",
      "Order tracking"
    ]
    pain_points [
      "Complex forms",
      "Slow page loads",
      "Lack of mobile support"
    ]
    context "Busy professionals, often shopping on mobile during commute"
  }
}
```

Use this for primary stakeholders where you need to capture their full context.

### Stakeholder With Relationships

```sruja
Customer = person "Customer"
Shop = system "Shop"

Customer -> Shop "Purchases products"
Shop -> Customer "Sends order updates"
```

Relationships show how stakeholders interact with the system. Notice the bidirectional flow—customers buy, but the system also reaches out to customers.

### Stakeholder Personas (Advanced)

For critical user types, create detailed personas:

```sruja
Sarah = person "Sarah (Customer Persona)" {
  description "Busy professional, 35, shops on mobile during commute"
  metadata {
    demographics {
      age "35"
      occupation "Marketing manager"
      device "iPhone 13"
    }
    goals [
      "Find products quickly",
      "Complete checkout in under 2 minutes",
      "Track orders without logging into email"
    ]
    frustrations [
      "Sites that aren't mobile-friendly",
      "Long forms that don't autofill",
      "Slow loading pages"
    ]
    scenario "Shopping on the train to work, 15 minutes before her stop"
  }
}
```

Personas bring stakeholders to life. They're especially useful when you need to make design trade-offs and want to ask "What would Sarah prefer?"

## A Complete Example: E-Commerce Platform

Let me show you how all this comes together in a real architecture:

```sruja
import { * } from 'sruja.ai/stdlib'

// =========== STAKEHOLDERS ===========

// Primary users
Customer = person "Customer" {
  description "Shoppers who purchase products"
  metadata {
    needs ["Fast checkout", "Easy search", "Mobile-friendly"]
    priority "critical"
  }
}

Administrator = person "Administrator" {
  description "Manages products, orders, and inventory"
  metadata {
    needs ["Bulk operations", "Reporting", "Quick updates"]
    priority "high"
  }
}

// Secondary users
SupportAgent = person "Support Agent" {
  description "Helps customers with order issues"
  metadata {
    needs ["Customer lookup", "Order history", "Refund processing"]
    priority "high"
  }
}

// Business stakeholders
ProductManager = person "Product Manager" {
  description "Owns product strategy"
  metadata {
    needs ["Analytics", "Feature usage", "User feedback"]
    priority "medium"
  }
}

BusinessOwner = person "Business Owner" {
  description "Accountable for revenue"
  metadata {
    needs ["Revenue reports", "Conversion metrics", "Cost tracking"]
    priority "high"
  }
}

// Compliance
ComplianceOfficer = person "Compliance Officer" {
  description "Ensures PCI-DSS compliance"
  metadata {
    needs ["Audit logs", "Access controls", "Data encryption"]
    can_block_launch true
  }
}

// =========== SYSTEM ===========

Shop = system "Shop" {
  WebApp = container "Web Application"
  API = container "API Service"
  Database = database "Database"
  
  metadata {
    slo {
      availability { target "99.9%" }
      latency { p95 "200ms" }
    }
  }
}

// =========== STAKEHOLDER INTERACTIONS ===========

// Primary user interactions
Customer -> Shop.WebApp "Browses and purchases"
Administrator -> Shop.WebApp "Manages products and orders"

// Secondary user interactions
SupportAgent -> Shop.WebApp "Looks up customer info"

// Business stakeholder needs (indirect)
ProductManager -> Shop "Reviews analytics"
BusinessOwner -> Shop "Monitors revenue"

// Compliance oversight
ComplianceOfficer -> Shop "Audits compliance"

view index {
  include *
}
```

This diagram tells a complete story. You can see who matters, what they need, and how they interact with the system. That's the power of explicit stakeholder modeling.

## Prioritizing Stakeholders (When You Can't Please Everyone)

Here's the uncomfortable truth: **stakeholders have conflicting needs, and you can't satisfy everyone.**

The customer wants the cheapest price. The business owner wants the highest margin. Those are fundamentally in tension.

The developer wants clean code. The product manager wants features fast. Also in tension.

I've learned to prioritize stakeholders using a simple framework:

**Critical:** Primary users and anyone who can block launch (compliance, security)
**High:** Business owners and secondary users
**Medium:** Technical stakeholders and internal teams
**Low:** Nice-to-have but not essential

In Sruja:

```sruja
Customer = person "Customer" {
  metadata {
    priority "critical"
    rationale "Primary user, revenue source"
  }
}

ComplianceOfficer = person "Compliance Officer" {
  metadata {
    priority "critical"
    rationale "Can block launch"
  }
}

MarketingAnalyst = person "Marketing Analyst" {
  metadata {
    priority "low"
    rationale "Nice to have, not essential for launch"
  }
}
```

Making priorities explicit helps when you need to make trade-offs.

## The Stakeholder Discovery Process

How do you find stakeholders you don't know about? I've learned to ask three questions:

1. **Who uses the system directly?** (Primary users)
2. **Who receives data or reports from the system?** (Secondary users)
3. **Who can say "no" to this launch?** (Compliance, business, security)

I also look for "zombie stakeholders"—people who used to matter but haven't been involved recently. They often resurface at the worst possible moment.

## Common Stakeholder Mistakes

After years of stakeholder surprises, I've seen these patterns repeat:

**Mistake 1: Only modeling users.** You remember the customers and admins, but forget the compliance officer who can block your launch.

**Mistake 2: Not documenting conflicts.** Business wants speed, compliance wants audit trails. These tensions exist whether you document them or not. Documenting them makes them solvable.

**Mistake 3: Assuming stakeholders agree.** Different stakeholders want different things. Don't assume alignment—verify it.

**Mistake 4: Invisible stakeholders.** The finance team getting automated reports, the support team needing data exports—these stakeholders are easy to miss until something breaks.

## What to Remember

**Stakeholders are more than users.** Anyone affected by or affecting your system is a stakeholder, whether they log in or not.

**Five types to look for:** Primary users, secondary users, business stakeholders, technical stakeholders, and compliance/governance.

**Conflicts are normal.** Different stakeholders want different things. Model the conflicts explicitly so you can solve them.

**Hidden stakeholders cause surprises.** Ask "Who can block this launch?" to find stakeholders you might have missed.

**Document priorities.** When you can't satisfy everyone, know who matters most.

**Use personas for critical stakeholders.** Detailed personas help you make design decisions when stakeholders aren't available to ask.

Stakeholder modeling isn't about pleasing everyone—it's about understanding the full picture so you can make informed trade-offs. The time you spend identifying stakeholders pays back in avoided crises and smoother launches.

## What's Next

Now that you understand who your stakeholders are, [Lesson 3](lesson-3.md) covers the other half of context: external dependencies, constraints, and success criteria. You'll learn how to document what your system depends on and what "success" actually means.