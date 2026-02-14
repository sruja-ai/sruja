title: "Lesson 5: Views & Styling"
weight: 5
summary: "Focus diagrams with views; improve legibility with styling."
---

# Lesson 5: Views & Styling

I once worked with an architect who loved colors. His diagrams were... enthusiastic.

Every service had a different color. Databases were purple. APIs were orange. Message queues were pink. External systems were yellow. Internal systems were green. And the relationships? Rainbow gradients.

It looked like a unicorn had exploded on his screen.

When I asked him to walk me through the architecture, he spent 10 minutes explaining his color coding system. By minute 3, I'd forgotten what we were looking at. By minute 10, I had a headache.

Here's the thing: **styling should clarify, not decorate**. When done right, styling makes diagrams instantly understandable. When done wrong, it creates visual noise that obscures the very thing you're trying to show.

## The Traffic Light Principle

Think about traffic lights. They use three colors with specific meanings:

- **Red** = Stop, danger, critical
- **Yellow** = Caution, warning
- **Green** = Go, safe, normal

You don't need a legend. You don't need to think. You instantly understand.

Good architecture styling works the same way. It uses visual elements to communicate meaning, not to look pretty.

## Why Styling Matters

Your brain processes visual information in two ways:

1. **Fast thinking (System 1)** - Instant pattern recognition, gut reactions
2. **Slow thinking (System 2)** - Deliberate analysis, reading labels

Good styling triggers System 1. Before you even read the labels, you should understand:
- What's important (visual weight)
- How things connect (line styles)
- What things are (shapes and colors)
- Where to look first (hierarchy)

Let me show you the difference.

### Before: The Uniform Diagram

```sruja
import { * } from 'sruja.ai/stdlib'

// Everything looks the same. No visual hierarchy.
Customer = person "Customer"

Shop = system "E-Commerce Shop" {
  WebApp = container "Web Application"
  API = container "API Service"
  DB = database "Database"
}

Customer -> Shop.WebApp "Uses"
Shop.WebApp -> Shop.API "Calls"
Shop.API -> Shop.DB "Reads/Writes"

view index {
  title "Everything Looks the Same"
  include *
}
```

Everything has equal visual weight. Your eye doesn't know where to go. You have to read every label to understand what's happening.

### After: Styled with Purpose

```sruja
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"

Shop = system "E-Commerce Shop" {
  WebApp = container "Web Application"
  API = container "API Service"
  DB = database "Database"
}

Customer -> Shop.WebApp "Uses"
Shop.WebApp -> Shop.API "Calls"
Shop.API -> Shop.DB "Reads/Writes"

// Global styles apply to all views
style {
  // Databases are always cylinders in green
  element "Database" {
    shape cylinder
    color "#10b981"  // Green - stable, persistent
  }
  
  // Read operations in blue (cool, safe)
  relation "Reads/Writes" {
    color "#3b82f6"
  }
}

view index {
  title "Styled with Purpose"
  include *
}
```

Now, before you read anything, you already know:
- That cylinder thing is a database (shape)
- The database is stable/persistent (green color)
- Data flows to/from it (blue line)

## Sruja Styling: The Basics

Sruja gives you two levels of styling:

### 1. Global Styles

Apply to all views. Use for consistent element types:

```sruja
// Define once, apply everywhere
style {
  element "Database" {
    shape cylinder
    color "#22c55e"
  }
  
  element "Cache" {
    shape cylinder
    color "#f59e0b"  // Orange = fast, temporary
  }
  
  relation "Processes payments" {
    color "#ef4444"  // Red = critical path
    thickness 3
  }
}
```

### 2. View-Specific Styles

Apply to one view. Use to highlight what matters in that context:

```sruja
view security {
  title "Security View"
  include PaymentGateway Shop.API Shop.DB
  
  // Only in this view: highlight security concerns
  style {
    element "API" {
      color "#ef4444"  // Red = security focus
    }
    
    relation "Processes payments" {
      style dashed  // Dashed = needs review
    }
  }
}
```

View-specific styles override global styles. This lets you maintain consistency while highlighting what matters for each audience.

## Complete Example: E-Commerce with Purposeful Styling

Let me show you a real-world example with intentional styling:

```sruja
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"

ECommerce = system "E-Commerce System" {
  WebApp = container "Web Application"
  API = container "API Service"
  OrderDB = database "Order Database"
  ProductDB = database "Product Database"
}

Customer -> ECommerce.WebApp "Browses"
ECommerce.WebApp -> ECommerce.API "Fetches data"
ECommerce.API -> ECommerce.OrderDB "Stores orders"
ECommerce.API -> ECommerce.ProductDB "Queries products"

// Global styles: Consistent visual language
style {
  // All databases look the same
  element "Database" {
    shape cylinder
    color "#22c55e"  // Green = stable, persistent
  }
  
  // Data retrieval = blue (cool, safe)
  relation "Fetches data" {
    color "#3b82f6"
  }
  
  // Data storage = orange (warm, action)
  relation "Stores orders" {
    color "#f59e0b"
    thickness 2
  }
  
  // Read operations = thinner
  relation "Queries products" {
    color "#3b82f6"
    thickness 1
  }
}

// Complete view with all styling
view index {
  title "Complete System View"
  include *
}

// Data flow view with custom emphasis
view dataflow {
  title "Data Flow View"
  include ECommerce.API
  include ECommerce.OrderDB
  include ECommerce.ProductDB
  exclude Customer
  exclude ECommerce.WebApp
  
  // Override global styles for this view
  style {
    element "API" {
      color "#0ea5e9"  // Blue = data orchestrator
      thickness 3
    }
    
    // Emphasize write operations
    relation "Stores orders" {
      color "#ef4444"  // Red = critical writes
      thickness 3
    }
  }
}
```

### What This Styling Communicates

Without reading a single label, the styling tells you:

- **Green cylinders** = Databases (stable, persistent storage)
- **Blue lines** = Read operations (safe, idempotent)
- **Orange/red lines** = Write operations (changes state, be careful)
- **Thick lines** = Critical paths (important, frequent)
- **Thin lines** = Secondary paths (less critical)

## Real-World Case Studies

### AWS: Color Coding by Service Type

AWS architecture diagrams use consistent colors:

- **Orange** = Compute (EC2, Lambda)
- **Blue** = Storage (S3, EBS)
- **Green** = Database (RDS, DynamoDB)
- **Purple** = Analytics (Redshift, Athena)

This consistency means AWS architects can look at any AWS diagram and instantly understand the service mix. No legend needed.

### GitHub: Minimal but Meaningful

GitHub's internal architecture docs use minimal styling:

- **Gray** = Existing services
- **Blue** = New services (in a proposal)
- **Red borders** = Deprecated services

Three colors. Clear meaning. Zero confusion.

### Spotify: Styling for Scale

Spotify's architecture diagrams use visual weight to show scale:

- **Large boxes** = Major services (100+ instances)
- **Medium boxes** = Standard services (10-100 instances)
- **Small boxes** = Small services (<10 instances)
- **Dashed borders** = External dependencies

Size communicates scale. Style communicates meaning.

## When to Style (Framework)

Here's my framework for deciding when to apply styling:

### Always Style:

**1. Element Types**
- Databases should look different from services
- External systems should look different from internal ones
- Use consistent shapes and colors for each type

**2. Critical Relationships**
- Payment flows
- Security boundaries
- Data pipelines
- Anything that, if broken, causes major issues

**3. State Changes**
- Read vs. write operations
- Sync vs. async communication
- Permanent vs. temporary data

### Sometimes Style:

**4. Performance Indicators**
- Hot paths (frequently used)
- Bottlenecks
- Cached vs. uncached data

**5. Ownership Boundaries**
- Team A's services
- Team B's services
- Shared infrastructure

### Rarely Style:

**6. Technology Stack**
- Java vs. Python services
- PostgreSQL vs. MySQL databases
- This is usually noise, not signal

**7. Version Information**
- v1 vs. v2 services
- Deprecated systems (use red border, but don't overdo it)

## Common Styling Mistakes

### Mistake #1: The Rainbow Diagram

**What happens:** You use 10+ colors because "they all have meaning."

**Why it fails:**
- No one can remember 10 color meanings
- Visual overload
- Looks unprofessional

**The fix:** Limit to 3-5 colors with clear, distinct meanings. If you need more, use shapes or line styles instead.

### Mistake #2: No Visual Hierarchy

**What happens:** Everything is equally bold, equally colorful, equally large.

**Why it fails:**
- Viewer doesn't know where to look first
- Critical paths get lost in the noise
- Feels overwhelming

**The fix:** Make important things visually prominent. Use size, color intensity, and thickness to create hierarchy.

### Mistake #3: Inconsistent Meaning

**What happens:** Blue means "database" in one diagram and "external service" in another.

**Why it fails:**
- Confuses people who see multiple diagrams
- Undermines trust in documentation
- Requires constant legend-checking

**The fix:** Create a style guide and stick to it across all diagrams.

### Mistake #4: Style Over Substance

**What happens:** Beautiful diagrams that don't communicate clearly.

**Why it fails:**
- Form over function
- People admire the diagram but don't understand the architecture
- Time wasted on decoration

**The fix:** Before adding any style, ask: "Does this help the viewer understand faster?" If no, remove it.

### Mistake #5: Ignoring Accessibility

**What happens:** Red/green color coding that's invisible to colorblind viewers.

**Why it fails:**
- 8% of men have some color blindness
- Excludes part of your audience
- May fail accessibility requirements

**The fix:**
- Don't rely on color alone (use shapes, patterns, labels too)
- Test with color blindness simulators
- Use colorblind-friendly palettes (blue/orange instead of red/green)

## The STYLE Framework

When styling your diagrams, use this framework:

**S - Semantic Meaning**
- Every style choice should communicate something
- If you can't explain why, remove it

**T - Type Differentiation**
- Different element types should look different
- Databases ≠ Services ≠ External Systems

**Y - Yield Hierarchy**
- Important things should be visually prominent
- Critical paths should stand out

**L - Limit Colors**
- Maximum 5 colors with distinct meanings
- Use shapes and line styles for additional differentiation

**E - Ensure Consistency**
- Same element type = same style across all diagrams
- Create and follow a style guide

## Practical Exercise

Take the e-commerce example above and create a new styled view:

**Challenge:** Create a "Performance View" that shows:
- Database query hot paths (thick lines)
- Cacheable vs. non-cacheable data (dashed vs. solid)
- Critical user journey (highlighted)

**Hint:** Use view-specific styles to override global styles for performance concerns.

## What to Remember

1. **Styling is communication, not decoration** - Every style choice should convey meaning
2. **Limit your palette** - 3-5 colors max, each with distinct meaning
3. **Create visual hierarchy** - Important things should look important
4. **Be consistent** - Same element type = same style across all diagrams
5. **Test accessibility** - Don't rely on color alone; consider color blindness
6. **Global styles for consistency, view styles for emphasis** - Define element types globally, highlight specifics per view
7. **When in doubt, simplify** - Remove styling that doesn't clarify

## When to Skip Styling

Sometimes, plain is better:

- **Early exploration** - When you're still figuring out the architecture, don't waste time on styling
- **Internal discussions** - If only architects see it and they know the system, minimal styling is fine
- **Simple systems** - 3 boxes don't need color coding
- **Rapid prototyping** - Get the structure right first, style later

Style when you need to communicate. Skip it when you're just thinking through problems.

---

**Next up:** We'll explore how to model complex real-world systems with all the techniques we've learned.