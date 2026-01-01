---
title: "Lesson 2: Setting up the Workspace"
weight: 2
summary: "Initializing the project structure and creating the first architecture file with product requirements."
---

# Lesson 2: Setting up the Workspace

Let's get our hands dirty. We will set up a professional project structure that separates our architectural definitions from our implementation code, and aligns with product requirements.

## Real-World Scenario: Starting a New Product

**Context**: You're building Shopify-lite, a multi-tenant e-commerce platform. Before writing code, you need to:

- Align engineering, product, and DevOps on the architecture
- Document requirements alongside the design
- Set up a structure that scales as the team grows

**Product team needs**: Clear documentation of what we're building and why.

**Engineering team needs**: Technical architecture that supports product goals.

**DevOps team needs**: Deployment and operational considerations from day one.

## 1. Directory Structure

Create a new directory for your project:

```bash
mkdir shopify-lite
cd shopify-lite
```

We will use the following structure (based on real-world best practices):

```text
shopify-lite/
├── architecture/          # Sruja files live here
│   ├── main.sruja        # Main architecture
│   ├── requirements.sruja # Product requirements
│   └── deployment.sruja   # Deployment architecture
├── src/                   # Source code (Go, Node, etc.)
├── docs/                  # Generated documentation
│   └── architecture.md    # Auto-generated from Sruja
├── .github/
│   └── workflows/
│       └── validate-architecture.yml  # CI/CD validation
└── README.md
```

**Why this structure?**

- **Separation of concerns**: Architecture separate from code
- **Version control**: Track architecture changes over time
- **CI/CD ready**: Easy to integrate validation
- **Team collaboration**: Product, engineering, and DevOps can all contribute

## 2. Installing Sruja

If you haven't already, install the Sruja CLI:

```bash
# Quick install
curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash

# Or via Go
go install github.com/sruja-ai/sruja/cmd/sruja@latest

# Verify installation
sruja --version
```

**For DevOps**: Add to your CI/CD pipeline (we'll cover this in Module 5).

## 3. Hello World: The Context View

Create your first file at `architecture/main.sruja`. We'll start with a high-level **Context View** to define the boundaries of our system.

### Product Requirements First

Before modeling architecture, let's capture product requirements:

```sruja
import { * } from 'sruja.ai/stdlib'


// Product Requirements (from product team)
requirement R1 functional "Merchants can create and manage online stores"
requirement R2 functional "Shoppers can browse products and make purchases"
requirement R3 functional "Platform processes payments securely"
requirement R4 nonfunctional "Platform must support 10,000+ stores"
requirement R5 nonfunctional "Checkout must complete in < 3 seconds"
requirement R6 nonfunctional "99.9% uptime SLA"

// Business Goals (for product/executive alignment)
metadata {
    businessGoal "Enable small businesses to sell online"
    targetMarket "Small to medium businesses (SMBs)"
    successMetrics "Number of active stores, GMV (Gross Merchandise Value)"
}

view index {
include *
}
```

### The Architecture Context

Now let's model the system context:

```sruja
import { * } from 'sruja.ai/stdlib'


// Product Requirements
requirement R1 functional "Merchants can create and manage online stores"
requirement R2 functional "Shoppers can browse products and make purchases"
requirement R3 functional "Platform processes payments securely"
requirement R4 nonfunctional "Platform must support 10,000+ stores"
requirement R5 nonfunctional "Checkout must complete in < 3 seconds"
requirement R6 nonfunctional "99.9% uptime SLA"

// 1. The System
Platform = system "E-Commerce Platform" {
    description "The core multi-tenant e-commerce engine that enables merchants to create stores and shoppers to make purchases."

    // Link to requirements
    requirement R1
    requirement R2
    requirement R3
    requirement R4
    requirement R5
    requirement R6
}

// 2. The Users (from product personas)
Merchant = person "Store Owner" {
    description "Small business owner who creates and manages their online store"
}
Shopper = person "Customer" {
    description "End customer who browses products and makes purchases"
}

// 3. External Systems (from product integrations)
Stripe = system "Payment Gateway" {
    external
    description "Third-party payment processor (PCI-compliant)"
}

EmailService = system "Email Service" {
    tags ["external"]
    description "Sends transactional emails (order confirmations, etc.)"
}

// 4. High-Level Interactions (user journeys)
Merchant -> Platform "Manages Store" {
    description "Creates products, manages inventory, views analytics"
}
Shopper -> Platform "Browses & Buys" {
    description "Browses products, adds to cart, completes checkout"
}
Platform -> Stripe "Processes Payments" {
    description "Secure payment processing for customer orders"
}
Platform -> EmailService "Sends Notifications" {
    description "Order confirmations, shipping updates"
}

// 5. Model user journeys as scenarios
ShopperCheckout = scenario "Shopper Checkout Journey" {
    Shopper -> Platform "Browses products"
    Shopper -> Platform "Adds items to cart"
    Shopper -> Platform "Initiates checkout"
    Platform -> Stripe "Processes payment"
    Stripe -> Platform "Confirms payment"
    Platform -> EmailService "Sends order confirmation"
    EmailService -> Shopper "Delivers confirmation email"
}

MerchantManagement = scenario "Merchant Store Management" {
    Merchant -> Platform "Logs into admin dashboard"
    Merchant -> Platform "Creates new product"
    Merchant -> Platform "Updates inventory"
    Merchant -> Platform "Views sales analytics"
}

// Executive view: Business context
view executive {
title "Executive Overview"
include Merchant
include Shopper
include Platform
include Stripe
include EmailService
}

// Product view: User journeys
view product {
title "Product View - User Experience"
include Merchant
include Shopper
include Platform
exclude Stripe
exclude EmailService
}

// Technical view: System integrations
view technical {
title "Technical View - System Integration"
include Platform Stripe EmailService
exclude Merchant Shopper
}

// Default view: Complete system
view index {
title "Complete System View"
include *
}
```

### Why This Approach?

**For Product Teams:**

- Requirements are visible and linked to architecture
- Business goals are documented
- Success metrics are clear

**For Engineering:**

- Architecture shows what to build
- Requirements guide implementation priorities
- External dependencies are identified early

**For DevOps:**

- Uptime SLA (R6) informs infrastructure planning
- Performance requirements (R5) guide monitoring setup
- Scale requirements (R4) inform capacity planning

## 4. Visualize It

Run the Sruja CLI to visualize your architecture:

```bash
# View the architecture diagram
sruja view architecture/main.sruja

# Or export to different formats
sruja export markdown architecture/main.sruja > docs/architecture.md
sruja export json architecture/main.sruja > docs/architecture.json
```

You should see a clean diagram showing:

- Your platform in the center
- Users (Merchant, Shopper) on the left
- External systems (Stripe, EmailService) on the right
- Interactions between them

## 5. Validate Your Architecture

Before moving forward, validate your architecture:

```bash
# Lint for errors
sruja lint architecture/main.sruja

# Check for orphan elements
sruja tree architecture/main.sruja
```

**Common issues to watch for:**

- Missing relations (orphan elements)
- Invalid references
- Unclear descriptions

## 6. Set Up CI/CD (DevOps Best Practice)

Create `.github/workflows/validate-architecture.yml`:

```yaml
name: Validate Architecture

on:
  push:
    paths:
      - "architecture/**"
  pull_request:
    paths:
      - "architecture/**"

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Sruja
        run: |
          curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
          echo "$HOME/go/bin" >> $GITHUB_PATH
      - name: Validate Architecture
        run: sruja lint architecture/main.sruja
      - name: Generate Docs
        run: |
          sruja export markdown architecture/main.sruja > docs/architecture.md
```

**Why this matters**: Catches architecture errors before they reach production.

## Key Takeaways

1. **Start with requirements**: Document what you're building and why
2. **Model context first**: Understand system boundaries before diving into details
3. **Link requirements to architecture**: Show how architecture supports product goals
4. **Set up CI/CD early**: Automate validation from day one
5. **Think about all stakeholders**: Product, engineering, and DevOps all need different views

## Exercise: Create Your Context View

**Tasks:**

1. Create a new project directory
2. Install Sruja CLI
3. Create `architecture/main.sruja` with:
   - At least 3 product requirements
   - System context (your system, users, external systems)
   - High-level interactions
4. Validate and visualize your architecture
5. (Optional) Set up CI/CD validation

**Time**: 15 minutes

## Further Reading

- Tutorial: [CLI Basics](/tutorials/basic/cli-basics)
- Tutorial: [Validation & Linting](/tutorials/basic/validation-linting)
- Docs: [Requirements Concepts](/docs/concepts/requirements)
- Course: [System Design 101 - Module 1: Fundamentals](/courses/system-design-101/module-1-fundamentals)
