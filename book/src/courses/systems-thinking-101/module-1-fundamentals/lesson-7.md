---
title: "Lesson 7: Feedback Loops"
weight: 7
summary: "Why do some systems improve over time while others spiral down? Feedback loops are the key."
time: "2 minutes"
---

# Lesson 7: Feedback Loops

## Learning Goals

By the end of this lesson, you'll be able to:
- Understand what feedback loops are and why they matter
- Distinguish between positive (reinforcing) and negative (balancing) feedback
- Identify feedback loops in everyday life and software systems
- Model feedback loops in Sruja

## Understanding Feedback Loops

Think about your home's thermostat. When the temperature drops, the heater turns on. When it rises above the set point, the heater turns off. The system maintains a comfortable temperature automatically.

That's a **feedback loop**—a cycle where an output becomes input for the next cycle.

Now think about social media. You watch a video, the algorithm shows you more similar videos, you watch more, the algorithm learns your preferences and shows even more relevant content. You watch more.

That's also a feedback loop—but a different type.

### What Are Feedback Loops?

**Feedback loops** are cycles where actions create reactions that affect future actions.

**Key insight:** Systems with feedback loops are "alive"—they respond, adapt, and change based on what's happening.

Without feedback loops, systems are static—they just exist. With feedback loops, systems are dynamic—they learn and evolve.

### Why Feedback Loops Matter

Feedback loops enable systems to:
- **Self-regulate:** Automatically maintain stability (like a thermostat)
- **Adapt:** Change behavior based on conditions (like an algorithm)
- **Improve:** Learn from mistakes and get better over time (like you learning from debugging)
- **Monitor health:** Detect and respond to problems automatically

This is powerful. But here's the thing: not all feedback loops are helpful.

## Types of Feedback Loops

### Positive (Reinforcing) Loops

These amplify changes—they make effects stronger over time.

**Real-world example:** Viral content
```
User watches video → Likes/shares it → Platform shows to more people → More people watch → More likes/shares
```

Each cycle amplifies the next one. That's why viral content grows exponentially.

**Software example:** User growth
```
More users → More engagement → More referrals → Even more users → Even more engagement
```

**When to use:** When you want growth, adoption, or network effects.

### Negative (Balancing) Loops

These counteract changes—they maintain stability.

**Real-world example:** Thermostat (as I mentioned earlier)
```
Temperature drops → Heater turns on → Temperature rises → Heater turns off → Temperature drops (repeat)
```

The system maintains a target temperature by counteracting changes.

**Software example:** Auto-scaling
```
Load increases → System adds servers → Load decreases → System removes servers → Load increases (repeat)
```

**When to use:** When you want stability, reliability, or control.

### Delayed Feedback Loops

Sometimes feedback doesn't happen immediately—it's delayed.

**Real-world example:** Email
```
You send email → Recipient sees it later → They reply → You respond later
```

The delay can cause oscillations—over-correcting because you don't see the effect immediately.

**Software example:** Performance monitoring
```
App slows down → Logs show high CPU → Analytics sends alert → You investigate → You optimize → App speeds up (delay: minutes to hours)
```

**When to watch for:** Delays can create unintended behavior—systems over-correcting or oscillating.

## Feedback Loops in Software Architecture

### System Loops: Auto-Scaling

Your application automatically scales based on load.

**Sruja model:**
```sruja
// partial
import { * } from 'sruja.ai/stdlib'

// System components
Monitoring = container "Monitoring System"
AutoScaler = container "Auto-Scaling Service"
App = system "Application"

// Balancing feedback loop
scenario AutoScale "Auto-Scaling Loop" {
  App -> Monitoring "Reports metrics"
  Monitoring -> AutoScaler "Detects high load"
  AutoScaler -> App "Adds instances"
  App -> Monitoring "Reports lower load"
  AutoScaler -> App "Removes instances"
}
```

**What this shows:**
- System self-regulates based on load
- Maintains stability automatically
- Responds to changes dynamically

### User Loops: Satisfaction

User behavior creates feedback that affects system usage.

**Sruja model:**
```sruja
// partial
import { * } from 'sruja.ai/stdlib'

// People
Customer = person "Customer"

// System
ShopSystem = system "Shop System" {
  WebApp = container "Web Application"
  API = container "API Service"
}

// User satisfaction loop
scenario UserLoop "Customer Satisfaction" {
  Customer -> ShopSystem.WebApp "Makes purchase"
  ShopSystem.API -> ShopSystem.Analytics "Updates metrics"
  ShopSystem.API -> Customer "Sends confirmation"
  Customer -> ShopSystem.WebApp "Returns to buy again"
}
```

**What this shows:**
- Happy customers return → creates positive loop
- Dissatisfied customers don't return → creates negative loop (if too many)
- User behavior is part of the system's feedback

### Business Loops: Investment

Business metrics drive decisions that affect system behavior.

**Sruja model:**
```sruja
// partial
import { * } from 'sruja.ai/stdlib'

// People
Customer = person "Customer"
ProductManager = person "Product Manager"

// System
ShopSystem = system "Shop System" {
  WebApp = container "Web Application"
  Analytics = container "Analytics Engine"
}

// Investment feedback loop
scenario BusinessLoop "Usage-Investment Loop" {
  Customer -> ShopSystem.WebApp "Uses system"
  ShopSystem.Analytics -> ProductManager "Reports growth"
  ProductManager -> ShopSystem.WebApp "Adds features"
  Customer -> ShopSystem.WebApp "Uses more"
}
```

**What this shows:**
- More usage → reports of growth → more investment in features → even more usage
- Business metrics drive system evolution
- This loop can accelerate improvement

## Identifying Feedback Loops

How do you find feedback loops in your systems?

### Look for Cycles

Ask yourself: "Where does output become input again?"

Look for cycles in your system:
- User takes action → System responds → User reacts → System adjusts
- System creates output → That output affects future input
- Metrics trigger actions → Those actions change metrics

### Check for Time Delays

Ask yourself: "Is there a delay between cause and effect?"

Delayed feedback can cause:
- Over-correction (responding too strongly to old data)
- Oscillations (swinging between extremes)
- Missed opportunities (reacting to what was, not what is)

### Determine the Type

Ask yourself: "Is this loop amplifying or counteracting?"

- **Amplifying (positive):** Effects get stronger over time → Growth, adoption, learning
- **Counteracting (negative/balancing):** Effects oppose changes → Stability, control, efficiency

Understanding the type helps you predict behavior.

## Modeling Feedback Loops in Sruja

### The Sruja Syntax

In Sruja, feedback loops are modeled using scenarios that show cycles:

```sruja
// partial
scenario LoopName "Description" {
  Step1 -> Step2 "Action"
  Step2 -> Step3 "Reaction"
  Step3 -> Step1 "Feedback"
}
```

### Naming Conventions

Use descriptive names that explain what the loop does:

**Good names:**
```sruja
scenario AutoScale "System auto-scales based on load"
scenario Satisfaction "User satisfaction drives repeat purchases"
scenario Stability "System maintains optimal performance"
```

**Bad names:**
```sruja
scenario Loop1 "First loop"
scenario Feedback "Some feedback"
```

### Best Practices

**1. Make cycles explicit**
Show the complete cycle, not just one or two steps.

**2. Label relationships clearly**
Indicate what's happening at each step.

**3. Document the purpose**
Add comments explaining why this loop matters.

**4. Consider delays**
If there are time delays, note them in comments.

## Common Mistakes

### Ignoring Feedback Loops

**Mistake:** Not recognizing that systems have feedback.

**Example:** Treating user behavior as external to your system, when user behavior actually shapes your system's design.

**Solution:** Model users and stakeholders as part of the system with feedback loops.

### Confusing Reinforcing vs. Balancing

**Mistake:** Thinking all feedback loops are "good" or "bad."

**Reality:** Both are useful—they just do different things.

- Reinforcing loops amplify—use for growth, learning, adoption
- Balancing loops stabilize—use for reliability, control, efficiency

**Solution:** Understand which type you're designing for and choose intentionally.

### Forgetting Delays

**Mistake:** Assuming feedback is always immediate.

**Reality:** Many systems have delayed feedback (caches, batch processing, asynchronous communication).

**Solution:** Model delays explicitly when they matter to system behavior.

## What to Remember

Feedback loops are what make systems dynamic and alive. Without them, systems are static and fragile. With them, systems can adapt, improve, and stabilize.

The key insight: **Design your feedback loops intentionally.**

Ask yourself:
- What behavior do I want this system to have?
- Should it amplify changes (growth) or counteract them (stability)?
- Are there delays I need to account for?

Most systems you interact with have feedback loops—your email client, your team's processes, your application's performance. Learning to identify and design them intentionally will make you a better architect.

The difference between a fragile system and a resilient one often comes down to: does it have intentional feedback loops?

## Check Your Understanding

Let's see if feedback loops make sense to you.

### Quick Check

**1. Your application's performance has degraded over time. More users report slowness, which causes more support tickets, which takes engineering time away from improvements. What type of feedback loop is this?**

[ ] A. Positive (reinforcing) loop—amplifying the problem
[ ] B. Negative (balancing) loop—counteracting the problem
[ ] C. This isn't a feedback loop, just bad luck
[ ] D. It depends on whether you fix it

**2. You're designing a new feature that you want users to adopt quickly and tell others about. What type of feedback loop should you design for?**

[ ] A. A reinforcing (positive) loop—encourage usage and sharing
[ ] B. A balancing (negative) loop—limit usage to prevent overload
[ ] C. No feedback loop needed
[ ] D. Both types equally

---

### Answers & Discussion

**1. A. Positive (reinforcing) loop—amplifying the problem** – This is a "vicious cycle." More users report slowness → More support tickets → Less engineering time → Performance degrades further → Even more users report slowness. The loop reinforces and amplifies the problem. Breaking this loop requires intervention (not designing it in the first place).

**2. A. A reinforcing (positive) loop—encourage usage and sharing** – When you want rapid adoption and word-of-mouth growth, design a reinforcing feedback loop: Users use feature → They like it → They tell others → More users use feature → The algorithm recommends it more. This loop amplifies adoption. You're intentionally designing a viral growth mechanism.

## What's Next

Now that you understand feedback loops—how systems respond, adapt, and self-regulate—let's explore the final concept in this module: **Context**. Context captures the environment your system operates in—stakeholders, dependencies, and constraints that shape your system's design and behavior.

This completes our foundational understanding of systems thinking. With all eight concepts, you have a complete toolkit for thinking about systems holistically.
