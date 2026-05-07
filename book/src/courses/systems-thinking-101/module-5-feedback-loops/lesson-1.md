---
title: "Lesson 1: Understanding Feedback Loops"
weight: 1
summary: "What feedback loops are and why they matter in systems thinking"
time: "5 min"
---

# The Loop That Changes Everything: Understanding Feedback Loops

Ever played a video game where the more you use a weapon, the more damage it does? That's a feedback loop—your actions create consequences that affect your future choices. Use the weapon more, hit harder, progress faster. Use it poorly, miss shots, struggle more.

Feedback loops are everywhere. In nature, in organizations, in software systems, and in everyday life. They're the mechanism through which systems learn, adapt, and regulate themselves.

In this lesson, you'll discover what feedback loops are, why they're crucial for systems thinking, and how to identify them in the architectures you build. You'll never look at a system the same way again.

Let's start by understanding what feedback loops actually are.

## Learning Goals

By the end of this lesson, you'll be able to:

- Understand what feedback loops are and how they differ from linear cause-and-effect
- Recognize feedback loops in everyday systems (and why they matter)
- Identify positive, negative, and balancing feedback loops in software architecture
- Explain why feedback loops are natural, not errors
- Model feedback loops that show self-regulation and adaptation

## What Are Feedback Loops, Really?

At its simplest level, a feedback loop is a cycle where an action creates a reaction that influences future actions. Unlike a linear process where A causes B which causes C, feedback loops circle back—output becomes input for the next cycle.

Think of it like a conversation where you're talking and adjusting based on what the other person says:

```
You speak → They respond → You react to their response → They respond to your reaction → [loop continues]
```

The key insight: **output from one cycle becomes input for the next.** This is what makes feedback loops powerful—and potentially dangerous.

In linear processes, you have a straight line: start → finish.

In feedback loops, you have a circle: act → respond → adjust → act again.

This circular relationship is what enables self-regulation, learning, and adaptation. It's also what can cause runaway growth or system collapse if not properly understood.

## Why Feedback Loops Matter (The Real Reasons)

I used to think of feedback loops as "circular dependencies" and avoid them at all costs. That was a huge mistake. In software systems, avoiding feedback loops means missing opportunities for self-improvement.

Let me share some real-world examples that changed my perspective.

### 1. Self-Regulation Without Explicit Logic

I once worked on a load balancer that needed to scale servers up and down based on traffic. The team implemented a simple rule: "if CPU > 80%, add a server." This created a feedback loop:

```
High CPU detected → Add server → CPU decreases → System monitors → [loop repeats]
```

We didn't design this as a feedback loop—it emerged from the rule. And it worked beautifully. The system self-regulated without explicit programming.

The lesson: Feedback loops don't always need to be designed. They can emerge from simple rules interacting with each other.

### 2. Learning Systems That Actually Learn

I consulted on a recommendation engine that started with terrible suggestions. Users rated things, and the system improved over time. What fascinated me was how the learning emerged from the feedback loop:

```
User watches video → System recommends similar video → User watches it → User rates it → System learns preferences → Next recommendation is better → [loop continues]
```

After a month, the recommendations were actually good. The system didn't have "good recommendations" baked in—it learned them through a feedback loop.

The lesson: Feedback loops enable systems to get smarter over time. Without them, you'd be stuck with static, hardcoded logic.

### 3. Error Recovery Through Retry Loops

I've built systems that failed 30% of the time due to network flakiness. Adding a retry loop transformed reliability:

```
Request fails → Wait 1s → Retry → Fails again → Wait 2s → Retry → Succeeds → [loop ends]
```

This simple feedback loop (failure → wait → retry) took a flaky system and made it reliable. We didn't fix the underlying network issues—we just built a system that could handle them.

The lesson: Feedback loops are one of the most powerful tools for building resilient systems.

## Everyday Examples of Feedback Loops

Feedback loops aren't just a software concept—they're everywhere in your daily life. Let me show you some examples that might feel familiar.

### Example 1: The Thermostat in Your Home

Your home heating system has a feedback loop running 24/7:

```
Room temperature drops
    ↓
Thermostat detects: "It's 65°F, should be 70°F"
    ↓
Thermostat turns on heater
    ↓
Temperature starts rising
    ↓
Thermostat detects: "It's 70°F, turn off heater"
    ↓
Thermostat turns off heater
    ↓
Temperature starts falling again
    ↓
[Loop repeats every few minutes]
```

This is a classic **balancing feedback loop**—it keeps temperature within a desired range. The output (current temperature) becomes input for the next decision (whether to heat).

### Example 2: Code Reviews at Work

Most software teams have a feedback loop around code quality:

```
Developer writes code → Submits for review → Reviewer provides feedback → Developer makes changes → Resubmits for review → Code quality improves → Future reviews have higher standards → [loop continues]
```

This is a **learning feedback loop**—the developer's skills improve over time through the feedback they receive. Each iteration raises the bar for what's acceptable.

I've been in code reviews where the feedback was unhelpful ("this is bad") rather than constructive ("consider breaking this into smaller functions"). The loop still existed, but it didn't create learning—just frustration.

The lesson: Feedback loops exist whether you design them intentionally or not. Make sure your feedback actually creates improvement.

### Example 3: Social Media Algorithms

When you scroll through Instagram or TikTok, you're participating in a feedback loop:

```
You watch a video → You watch it completely → You like it → Algorithm shows you more similar content → You watch more → You like those → Algorithm learns your preferences → Next recommendations are even better → [loop reinforces]
```

This is a **reinforcing feedback loop**—it amplifies your behavior. The more you engage with certain types of content, the more you see it. This can create viral growth (positive) or filter bubbles (negative).

I've seen people get stuck in these loops—endlessly watching the same type of content because the algorithm keeps serving it. The loop works, but it's not always healthy.

The lesson: Feedback loops can amplify behaviors, both good and bad. Design them carefully.

## Feedback Loops in Software Architecture

Now let's translate these concepts into software architecture. Feedback loops are everywhere in the systems we build.

### Example 1: Auto-Scaling Based on Load

Modern cloud applications automatically scale up and down based on traffic. This is a feedback loop:

```sruja
// partial
// Self-regulating feedback loop
AutoScaling = scenario "Auto-Scaling Based on CPU" {
  // System monitors itself
  MonitoringService -> App.Api "Reports CPU usage: 85%"
  
  // CPU is high → scale up
  if cpu_high {
    App.Api -> AutoScaler "Request scale up to handle load"
    AutoScaler -> App.Api "Adds new instance"
  }
  
  // After scaling, CPU decreases
  App.Api -> MonitoringService "Reports CPU usage: 45%"
  
  // CPU is now acceptable
  MonitoringService -> AutoScaler "CPU normal, can reduce instances"
  AutoScaler -> App.Api "Removes an instance"
  
  // System self-regulates to target CPU level
  App.Api -> MonitoringService "Reports CPU usage: 65%"
}
```

This feedback loop allows the system to maintain a target CPU usage (let's say 70%). It adds instances when load increases, removes them when load decreases. It self-regulates without human intervention.

I've worked with teams that manually scaled servers based on alerts. They'd get an email: "CPU high, add a server." Sometimes they'd remember, sometimes not. After implementing a feedback loop like this, the system just handled it itself.

The lesson: Feedback loops enable systems to self-regulate—adjusting automatically to maintain desired states.

### Example 2: User Experience Improvement

Many systems improve user experience through feedback loops:

```sruja
// partial
// Learning feedback loop
UserExperience = scenario "Feature Usage Feedback" {
  // User tries new feature
  User -> App.WebApp "Uses new feature"
  App.WebApp -> App.Api "Logs feature usage"
  App.Api -> Analytics "Analyzes user behavior"
  
  // System identifies issues
  Analytics -> ProductTeam "Feature has high abandonment rate"
  
  // Team makes improvements
  ProductTeam -> App.Api "Updates feature: clearer UI"
  
  // Next user has better experience
  User -> App.WebApp "Uses improved feature"
  App.WebApp -> App.Api "Logs feature usage"
  App.Api -> Analytics "Analyzes user behavior"
  Analytics -> ProductTeam "Feature abandonment decreased by 40%"
}
```

This is a **learning and adaptation feedback loop**. The system learns from how users interact, the team makes changes, and users have a better experience next time.

I once launched a feature without this feedback loop. We assumed users would love it. Six months later, we checked analytics and found 60% abandonment. We fixed issues, but the damage was done. We'd lost months of user trust.

The lesson: Always build feedback loops into your systems. Don't assume—measure, learn, adapt.

### Example 3: Cache Hit Rate Optimization

Systems often optimize themselves through feedback loops:

```sruja
// partial
// Resource optimization feedback loop
CacheOptimization = scenario "Cache Learning" {
  // System checks cache performance
  App.Api -> Cache "Request data"
  
  // Cache miss → query database and cache result
  if cache_miss {
    Cache -> Database "Query data"
    Database -> Cache "Store result"
  }
  
  // Cache hit → serve from cache (faster)
  if cache_hit {
    Cache -> App.Api "Return data from cache"
  }
  
  // System monitors cache hit rate
  App.Api -> Monitoring "Logs cache hits and misses"
  Monitoring -> CacheOptimzer "Calculates hit rate: 75%"
  
  // Low hit rate → optimize
  if hit_rate_low {
    Monitoring -> CacheOptimzer "Analyze access patterns"
    CacheOptimzer -> Cache "Adjust eviction policy"
  }
  
  // Next time, higher hit rate
  App.Api -> Cache "Request data"
  Cache -> App.Api "Return from cache (optimized)"
  App.Api -> Monitoring "Logs cache hits and misses"
  Monitoring -> CacheOptimzer "Hit rate now: 92%"
}
```

This feedback loop improves cache performance over time. The system monitors its own behavior (hit rate) and adjusts accordingly.

I've seen teams set cache policies once and never touch them again. But access patterns change over time. A policy that was good six months ago might be terrible today. Without a feedback loop, you'd never know.

The lesson: Systems need feedback loops to adapt to changing conditions. Static configurations become stale.

## Types of Feedback Loops

After studying feedback loops in both nature and software, I've found there are really three main types you'll encounter. Understanding which type you're dealing with helps you model it correctly.

### Type 1: Positive Feedback Loops (Reinforcing)

Output amplifies the input, leading to growth or collapse.

**Characteristics:**
- Output reinforces input
- Can create virtuous cycles (growth)
- Can create vicious cycles (collapse)
- Examples: viral growth, network effects, echo chambers

**Virtuous Cycle Example:**

```sruja
// partial
// Viral growth loop
ViralGrowth = scenario "Social Network Effects" {
  UserA -> App.Share "Shares content"
  App.Share -> UserB "Sees content"
  UserB -> App.Share "Shares with friends"
  App.Share -> UserC "Sees content"
  
  // More users → More shares → Exponential growth
  Analytics -> ProductTeam "User growth: +500% this week"
}
```

This is a virtuous cycle—each share brings more users, which leads to more shares. The feedback loop amplifies growth.

**Vicious Cycle Example:**

```sruja
// partial
// Performance collapse loop
PerformanceCollapse = scenario "Performance Degradation" {
  // System gets slow
  App.Api -> Database "Slow query (5s latency)"
  
  // Users retry due to slowness
  User -> App.WebApp "Refreshes page"
  App.WebApp -> App.Api "Make request (retry)"
  
  // More load makes it even slower
  App.Api -> Database "Even slower query (10s latency)"
  
  // More users retry
  User -> App.WebApp "Refreshes again"
  App.WebApp -> App.Api "More requests (retries)"
  
  // System collapses under load
  App.Api -> Database "Timeout (database overwhelmed)"
}
```

This is a vicious cycle—slowness causes retries, retries create more load, more load makes it slower, eventually everything fails. The feedback loop amplifies the problem.

### Type 2: Negative Feedback Loops (Balancing)

Output counteracts the input, maintaining stability and equilibrium.

**Characteristics:**
- Output opposes change
- Creates homeostasis (maintains equilibrium)
- Prevents runaway in either direction
- Examples: thermostats, load balancing, rate limiting

**Thermostat Example (Balancing):**

```sruja
// partial
// Thermostat maintains equilibrium
Thermostat = scenario "Temperature Regulation" {
  // Temperature drops
  Room -> Thermostat "Temperature: 65°F"
  
  // Too cold → turn on heat
  Thermostat -> Heater "Turn on heater"
  Heater -> Room "Heats room"
  Room -> Thermostat "Temperature: 70°F"
  
  // Target reached → turn off heat
  Thermostat -> Heater "Turn off heater"
  Heater -> Room "Cooling down"
  Room -> Thermostat "Temperature: 72°F"
  
  // Too warm → turn off (or cool more)
  Thermostat -> Heater "Keep off"
  Heater -> Room "Continues cooling"
  Room -> Thermostat "Temperature: 69°F"
  
  // Too cold → turn on again
  Thermostat -> Heater "Turn on heater"
  Heater -> Room "Heats room"
  Room -> Thermostat "Temperature: 70°F"
  
  // System oscillates around target (70°F)
}
```

The thermostat doesn't try to make the room hotter or colder—it tries to maintain a target temperature. If it gets too warm, it stops heating. If it gets too cold, it starts heating again. This negative feedback loop creates stability.

**Load Balancing Example (Balancing):**

```sruja
// partial
// Load balancer distributes work
LoadBalancing = scenario "Request Distribution" {
  // Server A is overloaded
  ServerA -> LoadBalancer "High load: 95% CPU"
  
  // Load balancer sends new requests to other servers
  User -> LoadBalancer "Make request"
  LoadBalancer -> ServerB "Route to Server B (lighter load)"
  LoadBalancer -> ServerC "Route to Server C (lighter load)"
  
  // Servers A, B, C balance out
  ServerA -> LoadBalancer "Load: 70% CPU"
  ServerB -> LoadBalancer "Load: 65% CPU"
  ServerC -> LoadBalancer "Load: 68% CPU"
  
  // Next time, requests distributed evenly
  User -> LoadBalancer "Make request"
  LoadBalancer -> ServerA "Route to Server A"
  LoadBalancer -> ServerB "Route to Server B"
  LoadBalancer -> ServerC "Route to Server C"
}
```

The load balancer sends fewer requests to the overloaded server, allowing it to recover. Other servers take up the slack. This negative feedback loop creates balance across all servers.

### Type 3: Delayed Feedback

Output affects input after a delay, which can cause oscillation.

**Characteristics:**
- There's a time delay between action and response
- Can cause system to overcorrect
- Often seen in monitoring and alerting systems
- Examples: inventory systems, cache invalidation

**Inventory Example (Delayed):**

```sruja
// partial
// Inventory management with delayed feedback
InventoryManagement = scenario "Inventory Replenishment" {
  // Item sells, stock decreases
  OrderService -> Inventory "Decrement stock: 5 units"
  Inventory -> OrderService "Stock: 15 units"
  
  // Low stock alert
  Inventory -> Alerting "Stock low: 15 units, threshold: 20"
  Alerting -> WarehouseManager "Send restock alert"
  
  // Manager sees alert, orders more stock
  WarehouseManager -> Suppliers "Order 50 more units"
  
  // System waits... (this is the delay)
  
  // New stock arrives
  Suppliers -> Warehouse "Ship 50 units"
  Warehouse -> Inventory "Add stock: +50 units"
  Inventory -> OrderService "Stock: 65 units"
  
  // Meanwhile, more items sold
  OrderService -> Inventory "Decrement stock: 10 units"
  Inventory -> OrderService "Stock: 55 units"
  
  // New stock finally arrives and is added to existing low stock
  Suppliers -> Warehouse "Ship 50 units"
  Warehouse -> Inventory "Add stock: +50 units"
  Inventory -> OrderService "Stock: 105 units"
  
  // System overcorrected—now have too much stock
  Inventory -> Alerting "Stock now: 105 units (oversupply!)"
}
```

The system delayed between detecting low stock and receiving new stock. During that delay, more items sold, so when new stock arrived, it was added to already-low stock. The result is an oversupply.

I've seen this exact pattern in retail systems. The solution is to either:
- Reduce the delay (automated restocking)
- Increase the trigger threshold (order sooner when stock is low)
- Account for expected sales during the delay period

The lesson: Delayed feedback can cause oscillation and overcorrection. Account for delays in your feedback loops.

## Feedback Loops vs. Circular Dependencies

This is one of the most important distinctions in systems thinking. Let me explain clearly.

### Circular Dependency (The "Bad" Kind)

```
Module A depends on Module B
Module B depends on Module A
```

This is a **circular dependency**—a compile-time, structural issue. It's bad because:

- You can't initialize either module (chicken and egg problem)
- It creates tight coupling
- There's no clear purpose—just mutual dependence
- Traditional software engineering says "avoid this"

### Feedback Loop (The "Good" Kind)

```
System performs an action
System receives feedback about the action
System adjusts future actions based on feedback
```

This is a **feedback loop**—a runtime, behavioral pattern. It's good because:

- It enables learning and adaptation
- It has a clear purpose (self-regulation, improvement)
- Coupling is loose (through feedback, not direct dependency)
- Systems thinking embraces this

**The Key Difference:**

| Aspect | Circular Dependency | Feedback Loop |
|--------|-------------------|--------------|
| When it happens | Compile-time (static) | Runtime (dynamic) |
| Purpose | None (accidental) | Intentional (regulation, learning) |
| Coupling | Tight (direct reference) | Loose (through feedback) |
| In systems thinking | Avoid | Embrace |
| Example | Module A → Module B → Module A | User → System → User → System |

I spent years avoiding any circular structures in my architectures. Then I learned about feedback loops in systems thinking and realized I'd been throwing away a powerful tool. Now I embrace feedback loops while avoiding circular dependencies. The distinction is crucial.

## Why Feedback Loops Are Natural, Not Errors

Here's something I've learned: In traditional software engineering, we're trained to avoid circular dependencies. But in systems thinking, feedback loops aren't errors—they're natural, desirable patterns.

Think about nature:

- Your body temperature regulation is a feedback loop
- Predator-prey populations are regulated by feedback loops
- Ecosystems self-regulate through feedback loops

These aren't "errors" in nature—they're essential for life.

In software:
- Auto-scaling is a feedback loop (not a bug)
- Machine learning is built on feedback loops (not anti-pattern)
- Load balancing uses feedback loops (not architectural flaw)

**The mindset shift:**
- Traditional engineering: "Cycles are bad, avoid them"
- Systems thinking: "Feedback loops are natural, embrace them"

I've seen teams reject feedback loop architectures because "cycles are bad." They're confusing two different concepts. Don't make that mistake.

Feedback loops are about **behavior over time**, not **circular dependencies in code**. They enable systems to:
- Self-regulate (maintain homeostasis)
- Learn (improve over time)
- Adapt (respond to changes)
- Recover (handle failures gracefully)

Embrace feedback loops as a powerful tool in your systems thinking toolkit.

## What to Remember

Feedback loops are cycles where output becomes input for the next cycle. When you're modeling or analyzing systems:

- **Look for the loop**—identify where output feeds back into input
- **Understand the purpose**—is it self-regulation, learning, amplification?
- **Check the type**—positive (reinforcing), negative (balancing), or delayed?
- **Design intentionally**—what controls does the loop have? What prevents runaway?
- **Embrace them**—feedback loops are natural and powerful, not errors to avoid

If you take away one thing, let it be this: **feedback loops are everywhere, and understanding them is key to understanding how systems behave over time**. Static diagrams show structure. Feedback loops show behavior.

---

## Check Your Understanding

Let's see if you've got this. Here are a couple of questions to test your understanding.

### Question 1

You're reviewing an e-commerce system and notice this behavior:

> "When a product goes out of stock, users can still see it but can't add it to cart. When stock is replenished, users who previously couldn't purchase are notified. After restocking, the product sells faster than usual for a few days, then sales normalize."

Which type of feedback loop best describes this behavior?

**A)** Positive feedback loop (reinforcing)
**B)** Negative feedback loop (balancing)
**C)** Delayed feedback loop (oscillation)
**D)** Not a feedback loop

<details>
<summary>Click to see the answer</summary>

**Answer: A) Positive feedback loop (reinforcing)**

Let's analyze this scenario:

1. **Product goes out of stock** → System hides add-to-cart button
2. **Users who wanted the product wait** → System remembers their interest
3. **Stock is replenished** → System notifies waiting users
4. **Waiting users purchase immediately** → Sales spike
5. **Sales spike suggests the product is popular** → System might order more next time

This is a **reinforcing feedback loop** because the output (sales data) amplifies the input (stocking decisions):

- High sales after restocking → System learns product is popular → Orders more stock next time → More sales
- The loop reinforces the behavior: stock the product, it sells well, stock more.

**Why other options are wrong:**

**B)** Incorrect. A negative (balancing) feedback loop would counteract changes to maintain stability. For example, if a product sells too much, the system would reduce ordering. But here, the system is increasing ordering after high sales—amplifying, not counteracting.

**C)** Incorrect. A delayed feedback loop involves a time delay that causes oscillation or overcorrection. There's no oscillation here—the behavior is straightforward: stock runs out, restock, sell out again, restock again. There's no overcorrection (ordering too much) or oscillation (alternating between too much and too little stock).

**D)** Incorrect. This absolutely is a feedback loop! The system's stocking decisions are influenced by past sales data, and sales influence future stocking decisions. Output becomes input for the next cycle.

**Key insight:** This is a common pattern in inventory systems. It's why you often see products go out of stock, then when they're restocked, they sell out again even faster—the system has learned the product is popular and amplifies that knowledge. This isn't bad design—it's a natural reinforcing feedback loop.

</details>

---

### Question 2

You're designing an auto-scaling system and want to prevent a vicious cycle where the system keeps adding instances unnecessarily. Which control mechanism would you implement?

> "The system monitors CPU usage. If CPU goes above 80%, it adds a server instance. When CPU drops below 60%, it removes an instance. Without controls, a sudden spike in traffic could cause: high CPU → add instances → load redistributes → some instances now idle, but overall CPU still high → add more instances → load redistributes again → more idle instances → [system keeps adding instances]"

**A)** Minimum instance limit (can't go below 2 instances)
**B)** Maximum instance limit (can't go above 10 instances)
**C)** Cooldown period (must wait 5 minutes after scaling down before scaling up again)
**D)** Scale incrementally (add 1 instance at a time, not 5 at once)

<details>
<summary>Click to see the answer</summary>

**Answer: C) Cooldown period (must wait 5 minutes after scaling down before scaling up again)**

Let's analyze each option:

**A)** Incorrect. A minimum instance limit (can't go below 2) prevents the system from removing too many instances, but it doesn't prevent it from adding too many. The vicious cycle described is about the system **adding** instances unnecessarily, not removing them. This control addresses the wrong side of the problem.

**B)** Incorrect. A maximum instance limit (can't go above 10) prevents unbounded growth, but it doesn't address the core issue: the system keeps adding instances because the new instances aren't helping (they stay idle while CPU stays high). The limit would stop the cycle at 10 instances, but doesn't solve the underlying problem of inefficient scaling decisions.

**C)** Correct! A cooldown period is exactly what's needed here. Here's why:

The vicious cycle happens because:
1. System scales up when CPU is high
2. New instances are added
3. Load redistributes, but CPU stays high (why? Maybe the new instances aren't helping, or there's a warmup period)
4. System sees CPU still high and scales up again
5. This happens repeatedly, adding more and more instances

A cooldown period prevents this by saying: "After you scale down, you must wait 5 minutes before you can scale up again." This gives the system time to:
- Allow new instances to warm up and start contributing effectively
- Allow the load redistribution to stabilize
- Prevent the system from reacting to transient CPU spikes with rapid up-and-down scaling

The cooldown breaks the cycle by adding a delay between "scale down" and "scale up" actions.

**D)** Incorrect. Scaling incrementally (adding 1 instance at a time instead of 5 at once) reduces the jump in instances, but it doesn't prevent the vicious cycle. The system could still add 1 instance, wait a bit, see CPU is still high, add another, wait a bit, add another—slowly but still unnecessarily adding instances. It doesn't have a mechanism to recognize "we just scaled down, let's pause and see if that helps" like a cooldown does.

**Key insight:** Feedback loops can create vicious cycles (runaway growth) if not properly controlled. The solution isn't always to limit the bounds (min/max instances)—sometimes you need to add a delay or a hysteresis to give the system time to stabilize. Cooldown periods, rate limiting, and circuit breakers are common controls for managing feedback loops.

</details>

---

## What's Next?

Now you understand what feedback loops are and why they matter. You know they're natural, not errors, and you can identify them in systems. You've seen examples from thermostats to social media algorithms to auto-scaling systems.

But we've only talked about **what** feedback loops are and **why** they matter. We haven't talked about **how** to classify different types of feedback loops or how to model them explicitly in your architectures.

In the next lesson, you'll learn about **types of feedback loops**—positive (reinforcing), negative (balancing), and delayed (oscillating). You'll discover when to use each type, how to identify virtuous vs. vicious cycles, and what controls prevent runaway behavior.

See you there!
