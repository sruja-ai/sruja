---
title: "Lesson 1: Identifying Parts"
weight: 1
summary: "How to find and classify system components"
time: "5 min"
---

# Finding the Building Blocks: Identifying Parts

Ever walked into a room full of people and tried to understand what's going on? You naturally start by identifying the key players: who's talking, who's listening, who's organizing. That's exactly what we do in systems architecture—we identify the parts first, then figure out how they connect.

In this lesson, you'll learn to spot the essential components of any software system. Whether you're building an e-commerce platform, a social network, or an internal tool, the process is the same. Let's dive in.

## Learning Goals

By the end of this lesson, you'll be able to:

- Confidently identify the key parts of any software system
- Understand the C4 model's four-level hierarchy
- Apply a systematic approach to breaking down requirements into components
- Recognize when you've found the right level of detail

## The C4 Model: Your Guide to Structure

Before we start identifying parts, let me introduce you to the C4 model—a framework that gives us a clear hierarchy for organizing software components. Think of it like a building: you start with the people who use it (person level), then the building itself (system), then the rooms within it (containers), and finally the furniture and fixtures inside those rooms (components).

Here's how it breaks down in practice:

```
Level 1: Person (Users, stakeholders)
  ↓ Who interacts with the system?
  
Level 2: System (Software systems)
  ↓ What software systems are involved?
  
Level 3: Container (Applications, databases, services)
  ↓ What are the deployable units?
  
Level 4: Component (Modules, classes, libraries)
  ↓ What are the internal building blocks?
```

I've found this hierarchy incredibly useful because it prevents us from getting lost in the weeds too early. Start at the top, work your way down, and stop when you have enough detail for your audience.

## A Step-by-Step Approach to Finding Parts

Let's walk through a practical example together. Imagine you're reading these requirements:

> "Customers can browse products, add to cart, and checkout. Administrators can manage inventory and view reports. The system sends email notifications for order confirmations."

How would you identify the parts? Here's my approach:

### Step 1: Start with People

Who are the humans interacting with this system? This is always the best starting point because every system exists to serve people.

Looking at the requirements, I can immediately spot two key players:
- **Customers**—they're browsing, adding to cart, and checking out
- **Administrators**—they're managing inventory and viewing reports

```sruja
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"
Administrator = person "Administrator"
```

Notice how I'm already thinking in terms of Sruja syntax? That's because once you understand the concept, the code becomes a natural expression of your thinking.

### Step 2: Identify the Systems

Now, what software systems are involved? Sometimes this is obvious (one main system you're building), and sometimes it's more complex (multiple systems talking to each other).

From our requirements, I can see:
- **E-commerce platform**—this is the main system we're building
- **Email service**—this is an external system we'll integrate with

```sruja
ECommerce = system "E-Commerce Platform"
EmailService = system "Email Service"
```

Don't worry if you miss some systems initially. You can always come back and add more as you discover dependencies.

### Step 3: Break Down Systems into Containers

Now we get to the interesting part. What are the deployable units that make up our e-commerce platform? Think about what you'd actually deploy: web apps, APIs, databases, caches—these are all containers.

For our e-commerce platform, a sensible breakdown might be:

```sruja
ECommerce = system "E-Commerce Platform" {
  WebApp = container "Web Application" {
    technology "React"
  }
  API = container "API Service" {
    technology "Node.js"
  }
  Database = database "PostgreSQL" {
    technology "PostgreSQL 14"
  }
  Cache = queue "Redis" {
    technology "Redis 7"
  }
}
```

I've included a cache because I know from experience that almost every e-commerce platform benefits from one for performance. This is where real-world experience comes in—you'll naturally add components that make sense based on patterns you've seen before.

### Step 4: Consider Components (But Only If Needed)

This is where many people go wrong. They immediately start modeling every single class and module. Stop and ask yourself: *Does my audience need this level of detail?*

If you're talking to developers who need to understand the internal architecture, yes—break containers into components. If you're talking to stakeholders who just want to understand the overall system, skip this level.

For example, if we were documenting the API's internal structure for our engineering team:

```sruja
API = container "API Service" {
  ProductService = component "Product Service"
  CartService = component "Cart Service"
  OrderService = component "Order Service"
  PaymentService = component "Payment Service"
}
```

The key insight here: **match the level of detail to your audience**. Not every diagram needs components. Not every discussion needs to dive this deep.

## Common Patterns You'll See Everywhere

After building systems for years, you start to recognize patterns that repeat across different domains. Here are three I see constantly:

### Pattern 1: The Classic Three-Tier Architecture

This is probably the most common pattern you'll encounter. It's simple, it works, and it's a great starting point for most web applications.

```sruja
App = system "Application" {
  Frontend = container "Web App" {
    technology "React"
  }
  Backend = container "API Service" {
    technology "Node.js"
  }
  Database = database "Database" {
    technology "PostgreSQL"
  }
}
```

I've used this pattern more times than I can count. It's a solid foundation that scales well and most developers immediately understand.

### Pattern 2: Microservices

When your system grows, you might need to split it into smaller, independently deployable services. That's where this pattern comes in.

```sruja
App = system "Microservice Application" {
  APIGateway = container "API Gateway"
  UserService = container "User Service"
  OrderService = container "Order Service"
  NotificationService = container "Notification Service"
}
```

The trick here is knowing when to use this. Don't start with microservices just because it's trendy. Start simple, then split when you have a real reason (scaling, team size, complexity).

### Pattern 3: Event-Driven Architecture

When you need systems to react to events in real-time, this pattern shines. It's more complex but incredibly powerful for the right use cases.

```sruja
App = system "Event-Driven System" {
  Producer = container "Event Producer"
  Consumer = container "Event Consumer"
  MessageQueue = queue "Kafka Cluster"
}
```

I've seen teams jump into this pattern too early and regret it. Make sure you actually need event-driven architecture before adopting it—it adds significant complexity.

## Pitfalls to Avoid (I've Made All of These)

Let me save you some trouble by sharing mistakes I've made and seen others make:

### Mistake 1: Oversimplifying

When you're rushing, it's tempting to just model everything as one big system:

```sruja
// Don't do this—it's too simple
App = system "The App"
```

This tells you nothing about how the system is structured. Anyone looking at this diagram will be left with more questions than answers.

```sruja
// This is better—it shows structure
App = system "The App" {
  Frontend = container "Frontend"
  Backend = container "Backend"
  Database = database "Database"
}
```

### Mistake 2: Over-Engineering

On the flip side, I've seen people model every single class and component, creating diagrams that are essentially unreadable:

```sruja
// Don't do this—it's too detailed
App = system "The App" {
  Frontend = container "Frontend" {
    Header = component "Header"
    Body = component "Body"
    Footer = component "Footer"
  }
}
```

Who cares about the header, body, and footer at the architecture level? No one. That's implementation detail, not architecture.

```sruja
// This is the right level of detail
App = system "The App" {
  Frontend = container "Frontend"
  Backend = container "Backend"
}
```

### Mistake 3: Mixing Levels Inconsistently

This one trips up even experienced architects. You might have some parts modeled at the container level and others at the component level, creating confusion:

```sruja
// Don't do this—inconsistent detail
App = system "The App" {
  Frontend = container "Frontend"
  UserService = component "User Service"  // Skips container level
  Database = database "Database"
}
```

What's the UserService doing here without a parent container? Is it a standalone service? Part of another system? It's confusing.

```sruja
// This is consistent and clear
App = system "The App" {
  Frontend = container "Frontend"
  Backend = container "Backend" {
    UserService = component "User Service"
  }
  Database = database "Database"
}
```

## What to Remember

Identifying parts is both an art and a science. The science is following the C4 hierarchy and being systematic. The art is knowing how much detail to include and what to leave out.

If you take away just one thing from this lesson, let it be this: **start with people, then systems, then containers, and only add components when your audience truly needs that level of detail**.

Everything else flows from that principle.

---

## Check Your Understanding

Let's see if you've got this. Here are a couple of questions to test your understanding:

### Question 1

You're building a project management tool with these requirements:

> "Team members can create tasks, assign them to others, and track progress. Managers can view reports and approve tasks. The system sends email notifications for task assignments and due dates. Task data is stored in a database. The system integrates with Slack for notifications."

At the **person** level, which parts should you identify?

**A)** Task, Report, Email, Database, Slack
**B)** Team Member, Manager
**C)** Project Management Tool, Slack, Email Service
**D)** Task Assignment, Task Approval, Report Generation

<details>
<summary>Click to see the answer</summary>

**Answer: B) Team Member, Manager**

The person level is about **humans** who interact with the system. Team members and managers are the people using the system. Tasks, reports, and notifications are things the system handles, not people. The project management tool, Slack, and email service are systems, not people.

Remember: people first, then systems, then everything else.
</details>

---

### Question 2

You're modeling a simple blog platform. When should you break your containers down into components?

**A)** Always—components provide the most detail
**B)** Never—containers are sufficient for all audiences
**C)** When your audience needs to understand internal architecture or when a container is complex
**D)** Only when you're using microservices

<details>
<summary>Click to see the answer</summary>

**Answer: C) When your audience needs to understand internal architecture or when a container is complex**

Components are an optional level in the C4 model. You should add them when:

- You're documenting for developers who need to understand implementation
- A container has grown complex and needs to be broken down
- Different teams own different parts of the system

Don't add components just because you can. Match the level of detail to your audience's needs. A stakeholder doesn't need to see your component architecture—a developer probably does.
</details>

---

## What's Next?

Now that you know how to identify parts, you're probably wondering: "How do I actually define these parts in Sruja? What's the syntax?"

In the next lesson, you'll learn exactly that. We'll cover the four core element types (person, system, container, component), how to use them, and what details to include. You'll have a complete toolkit for modeling any software system.

See you there!