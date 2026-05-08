---
title: "Lesson 2: Types of Feedback Loops"
weight: 2
summary: "Positive, negative, and balancing feedback loops"
time: "5 min"
---

# Amplifying or Dampening: Types of Feedback Loops

Ever heard a squeaky microphone? You speak, the speaker hears you, and the volume gets louder and louder. Then the speaker notices and turns it down. That's a feedback loop—the output (your voice) affects the input (the microphone level).

Now imagine if the speaker couldn't turn the volume down. Every time you spoke, it got louder and louder. That's a runaway loop—and it would eventually cause damage.

Feedback loops in software systems work the same way. They can amplify growth (good) or cause collapse (bad) or maintain stability (necessary). In this lesson, you'll learn to recognize and classify different types of feedback loops.

Let's start by understanding how to categorize feedback loops.

## Learning Goals

By the end of this lesson, you'll be able to:

- Understand positive (reinforcing) and negative (balancing) feedback loops
- Recognize when each type is appropriate
- Identify vicious (runaway) vs. virtuous (growth) cycles
- Understand delayed feedback and oscillation
- Design feedback loops with appropriate controls

## Feedback Loop Classification

Feedback loops fall into three main categories based on what they do to your system:

```
Feedback Loops
├── Positive (Reinforcing)
│   ├── Virtuous Cycle (Growth)
│   └── Vicious Cycle (Collapse)
├── Negative (Balancing)
│   ├── Self-Regulating (Homeostasis)
│   ├── Error-Correcting (Resilience)
│   └── Stabilizing (Control)
├── Delayed (Time-Based)
    └── Oscillation (Variability)
```

Understanding which type you're dealing with is crucial because the design approach is completely different for each.

## Positive Feedback Loops (Reinforcing)

These loops amplify change, leading to exponential growth or collapse. They're powerful when you want growth, dangerous when you don't have controls.

### Pattern 1: Virtuous Cycle (Growth)

Output amplifies input, creating runaway growth:

```sruja
// partial
// Example: Viral content recommendation
ViralGrowth = scenario "Viral Growth Loop" {
  User -> Platform "Shares content"
  Platform -> UserB "Shows content"
  UserB -> Platform "Shares content"
  
  // More users see content, more shares happen
  Platform -> UserC "Shows content"
  UserC -> Platform "Shares content"
  // Exponential growth
}
```

**Characteristics:**
- Output reinforces input
- Creates exponential growth
- Can lead to runaway success
- Examples: Viral sharing, network effects, learning algorithms

**When to use:** 
- When you want rapid growth and virality
- When amplifying desirable behaviors
- In recommendation systems, social platforms

**Risks:**
- Unchecked growth can overwhelm system
- Can create echo chambers
- May amplify undesirable behaviors (fake news, spam)

**Real-world example:**
I once worked on a social media algorithm that showed viral content more aggressively. Engagement spiked for a few weeks, then crashed because users got sick of seeing the same type of content. The positive feedback loop had run its course. We needed to dampen it—show diverse content even if it meant lower short-term engagement.

### Pattern 2: Vicious Cycle (Collapse)

Output counteracts input, leading to runaway collapse:

```sruja
// partial
// Example: Performance degradation
PerformanceCollapse = scenario "Performance Vicious Cycle" {
  User -> WebApp "Submits request"
  WebApp -> API "Processes request"
  API -> Database "Slow query (1000ms)"
  Database -> Cache "Cache miss"
  
  // System tries to compensate
  Cache -> Database "Query database (slower)"
  Database -> API "Slow response"
  API -> WebApp "Slow response"
  WebApp -> User "Refreshes page"
  
  // User refreshes, creates more load
  User -> WebApp "Submit again"
  // Loop spirals downward
}
```

**Characteristics:**
- Output worsens the problem
- Creates death spiral (collapse)
- Can be irreversible if not caught early
- Examples: System overload, cache stampedes, resource exhaustion

**Risks:**
- System collapse
- User abandonment
- Cascading failures
- Complete breakdown

**Real-world example:**
I've seen systems collapse from vicious feedback loops. One example: An e-commerce site had slow checkout times during peak hours. The system would show "processing" for 30 seconds, but customers would refresh the page thinking it failed. They'd submit again, creating more load, making things even slower. More customers refreshed, more load, slower times... The feedback loop (customers refreshing on slowness) created a death spiral that eventually took down the entire site.

## Negative Feedback Loops (Balancing)

These loops counteract change, maintaining stability and preventing extremes. They're crucial for building resilient systems.

### Pattern 1: Self-Regulating (Homeostasis)

System maintains a target state by adjusting based on output:

```sruja
// partial
// Example: Auto-scaling
AutoScaling = scenario "Self-Regulating System" {
  Monitoring -> App "Reports CPU: 90%"
  App -> AutoScaler "Scale up (target: 70%)"
  AutoScaler -> App "Add instances"
  
  // CPU decreases
  Monitoring -> App "Reports CPU: 50%"
  App -> AutoScaler "Scale down (target: 70%)"
  AutoScaler -> App "Remove instances"
}
```

**Characteristics:**
- Output opposes deviation from target
- Maintains homeostasis
- Responds to disturbances
- Examples: Thermostats, auto-scaling, load balancing

**When to use:**
- When you want to maintain a target state (CPU usage, inventory levels)
- When you need to absorb shocks and disturbances
- When stability is more important than optimal performance

**Real-world example:**
The thermostat example from Lesson 1 is a perfect self-regulating loop. The heater turns on when temperature drops, turns off when temperature rises. It doesn't care about the exact temperature—it just wants to maintain a comfortable range. This creates a stable, predictable environment for the people in the room.

In software, self-regulating loops are everywhere. Auto-scaling systems maintain a target CPU usage. Inventory systems maintain target stock levels. Rate limiters maintain target request rates. They're the unsung heroes of system design.

### Pattern 2: Error-Correcting (Resilience)

System detects errors and automatically corrects:

```sruja
// partial
// Example: Retry with backoff
RetryWithBackoff = scenario "Error-Correcting Loop" {
  User -> App "Submits order"
  App -> Service "Process payment"
  
  // First attempt fails
  Service -> App "Payment failed: timeout"
  App -> Service "Retry after 1s (backoff)"
  Service -> App "Payment failed: timeout"
  App -> Service "Retry after 2s (backoff)"
  Service -> App "Payment succeeded"
  
  // System recovers
  App -> User "Order confirmed"
}
```

**Characteristics:**
- Detects errors and corrects automatically
- Uses backoff to avoid overwhelming failing system
- Increases resilience without manual intervention
- Examples: Retries with backoff, circuit breakers, failover systems

**When to use:**
- When external services are unreliable
- When transient failures are common
- When you need to increase resilience
- When you want to reduce operational overhead (fewer support tickets)

**Real-world example:**
I've worked on systems that didn't have proper error handling. A payment gateway would timeout occasionally, and the app would retry immediately three times with no delay between attempts. This would overwhelm the gateway, making failures worse. We added exponential backoff (wait 1s, then 2s, then 4s) and the system became much more reliable. The error-correcting loop turned a flaky dependency into a resilient one.

### Pattern 3: Stabilizing (Control)

System actively manages resources to prevent oscillation:

```sruja
// partial
// Example: Rate limiting with hysteresis
RateLimiting = scenario "Stabilizing Control" {
  User -> API "Send request"
  API -> RateLimiter "Check limit"
  
  // Under limit: allow
  if under_limit {
    RateLimiter -> API "Allow"
    API -> Service "Process"
  }
  
  // Over limit: throttle
  if over_limit {
    RateLimiter -> API "Throttle (429: Too Many Requests)"
    API -> User "Try again later"
  }
  
  // Dynamic adjustment based on system load
  RateLimiter -> LoadMonitor "Report current rate: 1000 req/s"
  LoadMonitor -> RateLimiter "Adjust limit: 800 req/s"
}
```

**Characteristics:**
- Actively prevents extreme behavior in either direction
- Maintains stability through dynamic adjustments
- Can add hysteresis (limits change based on history)
- Examples: Rate limiting, resource pools, admission control

**When to use:**
- When you need to protect against abuse or overload
- When you have limited resources (database connections, API quotas)
- When you want to maintain service quality for all users
- When fairness matters (don't let heavy users dominate)

**Real-world example:**
I once saw a rate limiting system cause more harm than good. It was configured with a hard limit, and when users hit it, they'd get throttled and immediately retry, creating a burst of traffic that was worse than just letting them through. We added hysteresis—the limit would decrease temporarily after being hit, then slowly recover. This smoothed out traffic and actually improved throughput for everyone while protecting the system.

## Delayed Feedback (Time-Based)

Feedback that occurs after a delay can cause oscillation or overcorrection:

### Pattern: Oscillation

System alternates between states due to delayed feedback:

```sruja
// partial
// Example: Temperature control with delay
DelayedFeedback = scenario "Oscillating Control" {
  Thermostat -> Heater "Turn on (too cold)"
  Heater -> Room "Heats room"
  Room -> Thermostat "Temperature reading (5 min delay)"
  
  // By the time feedback arrives, room is already too warm
  Thermostat -> Heater "Turn off (too hot)"
  Heater -> Room "Cools room"
  Room -> Thermostat "Temperature reading (5 min delay)"
  
  // By the time feedback arrives, room is too cold again
  Thermostat -> Heater "Turn on (too cold)"
  // Oscillates between too hot and too cold
}
```

**Characteristics:**
- Delay between action and feedback causes system to overcorrect
- Can create oscillation (too hot, too cold, too hot, too cold)
- System never settles into stable state
- Examples: Temperature control, stock trading, caching with invalidation

**When to use:**
- When you need to identify and eliminate delayed feedback loops
- When you have slow sensors or reporting systems
- When you want to stabilize oscillating systems
- When you need to add damping or predictive adjustments

**Real-world example:**
I once worked on a caching system that would cache data for 30 minutes. But the data source would update every 10 minutes. The cache would show stale data, then refresh, then show stale data again. This created oscillation—applications would see new data for 20 minutes, then old data for 20 minutes, then new again. Users complained: "Why is the data jumping around? Is it broken?" We fixed it by making the cache time consistent with the source—stale data at the same time as source updates. Eliminating the time delay eliminated the oscillation.

## Comparing Feedback Loop Types

Different feedback loops serve different purposes:

| Type | Effect | Stability | Example | Use When |
|-------|--------|-----------|--------|----------|
| Positive (Virtuous) | Amplifies | Decreases (growth) | Viral sharing, learning algorithms | You want rapid growth |
| Positive (Vicious) | Amplifies | Decreases (collapse) | System overload, resource exhaustion | NEVER (avoid at all costs) |
| Negative (Self-Regulating) | Opposes | Maintains | Auto-scaling, thermostats | You need stability |
| Negative (Error-Correcting) | Corrects | Increases (resilience) | Retries, circuit breakers | You need reliability |
| Negative (Stabilizing) | Constrains | Maintains | Rate limiting, hysteresis | You need control |
| Delayed (Oscillating) | Overcorrects | Decreases (oscillation) | Slow sensors, stale data | You need synchronization |

**Key insight:** Choose the right type of feedback loop for your goal. Want growth? Use positive loops (with controls). Need stability? Use negative loops. Want resilience? Use error-correcting loops.

## Designing Feedback Loops

### Step 1: Identify the Feedback

What creates the feedback? Who acts? Who adjusts?

```sruja
// partial
// Example: Auto-scaling feedback
AutoScalingFeedback = flow "Auto-Scaling Feedback Path" {
  // Who creates feedback?
  App -> Monitoring "Reports CPU usage"
  Monitoring -> App "Shows metrics to operators"
  
  // Who acts on it?
  App -> AutoScaler "Adjusts instances"
  AutoScaler -> App "Adds/removes instances"
}
```

**Ask yourself:**
- What's the signal? (CPU usage, latency, error rate)
- Who measures it? (monitoring system)
- Who makes decisions? (auto-scaler, operations team)
- What's the adjustment? (add/remove instances)

### Step 2: Determine the Type

Based on the effect, classify the loop:

```sruja
// partial
// Classifying feedback loops
ViralGrowth = scenario "Viral Growth" {
  type "positive"
  subtype "virtuous"
  effect "amplifying"
  stability "decreases"
}

SystemOverload = scenario "System Overload" {
  type "positive"
  subtype "vicious"
  effect "amplifying"
  stability "decreases"
  risk "critical"
}

AutoScaling = scenario "Auto-Scaling" {
  type "negative"
  subtype "self-regulating"
  effect "opposes"
  stability "maintains"
}
```

### Step 3: Add Controls

Prevent unwanted behavior and add safety limits:

```sruja
// partial
// Adding controls to positive feedback loop
ControlledGrowth = scenario "Controlled Viral Growth" {
  User -> Platform "Shares content"
  
  // Content moderation
  Platform -> ModerationService "Check for policy violations"
  if violates_policy {
    Platform -> ModerationService "Reject content"
    Platform -> User "Content rejected: violates policy"
  } else {
    Platform -> UserB "Show content"
    UserB -> Platform "Shares content"
  }
  
  // Rate limiting
  Platform -> RateLimiter "Check rate limit"
  if over_rate_limit {
    Platform -> RateLimiter "Throttle shares"
  }
}
```

### Step 4: Monitor and Observe

Track the behavior of your feedback loop over time:

```sruja
// partial
// Monitoring feedback loop behavior
FeedbackMonitoring = scenario "Feedback Loop Monitoring" {
  // Metrics to track
  GrowthRate = metric "User growth rate (new users/day)"
  StabilityScore = metric "System stability (99.9% uptime)"
  FeedbackRatio = metric "Positive:negative feedback ratio"
  
  // Alert on issues
  if GrowthRate > threshold {
    AlertSystem -> OpsTeam "High growth rate detected"
  }
  
  if StabilityScore < threshold {
    AlertSystem -> OpsTeam "Stability degraded"
  }
  
  // Track over time
  GrowthRate -> Dashboard "Plot growth over time"
}
```

**Key indicators to monitor:**
- Growth rate (is it sustainable?)
- Stability metrics (uptime, error rate)
- Feedback distribution (positive vs. negative)
- Resource utilization (are you approaching limits?)

## Pitfalls to Avoid

### Mistake 1: Treating All Cycles as Bad

I've seen teams adopt a "no cycles" mentality and avoid feedback loops entirely. This is throwing away a powerful tool.

```sruja
// Bad: Avoiding feedback loops entirely
NoFeedbackSystem = system "Static System" {
  // No feedback loops
  // No adaptation
  // No learning
}
```

**Why this is wrong:**
- You lose the ability to self-regulate
- You lose opportunities for learning and improvement
- Your system can't adapt to changing conditions
- You're missing a key tool in systems thinking

**The right approach:** Use feedback loops intentionally. Design them with clear purposes and appropriate controls. Embrace them where they add value. Avoid them where they cause harm.

### Mistake 2: Confusing Types

I've seen teams misuse positive feedback loops for situations that need negative (balancing) loops:

```sruja
// partial
// Bad: Using positive feedback for stability
PositiveForStability = scenario "Misclassified Feedback" {
  // Trying to grow load when you should stabilize
  User -> App "Submit request"
  App -> Server "Process"
  
  // High load detected
  Server -> LoadBalancer "Add more instances"
  LoadBalancer -> Server "Add more instances"
  
  // This amplifies load instead of balancing it
}
```

**Why this is wrong:**
- When you need stability (high load), adding more instances is the worst thing to do
- You should use a negative (balancing) loop instead: throttle requests, queue them, or reject some

**The right approach:** Match the feedback loop type to the situation. Need stability? Use negative loops. Need growth? Use positive loops (with controls). Need resilience? Use error-correcting loops.

### Mistake 3: Ignoring Delayed Feedback

I've seen teams ignore delayed feedback because it's "not real-time," only to have it cause major problems:

```sruja
// partial
// Bad: Ignoring delayed feedback causes oscillation
IgnoreDelayed = scenario "Ignoring Delayed Feedback" {
  User -> App "Submits data"
  App -> Service "Process"
  Service -> Database "Persist"
  
  // Another system updates same data later
  ExternalSystem -> Database "Update (5 min delay)"
  Database -> App "Show updated data"
  App -> User "Refreshes page" [sees old data]
  
  // User refreshes again, thinking data is stale
  User -> App "Refreshes page" [sees old data again]
  // User gets frustrated: "Why is this data jumping around?"
}
```

**Why this is wrong:**
- The delay in feedback causes the system to overcorrect
- Users see stale data, refresh, see stale data again
- System never stabilizes
- Users get frustrated and think the system is broken

**The right approach:** Synchronize the timing of feedback loops. Make sure when one system updates data, consuming systems see the update before they'd otherwise request it again. Or make it clear when data was last updated so users don't expect immediate freshness.

## What to Remember

Feedback loops are one of the most powerful concepts in systems thinking. When you design them:

- **Identify the type** — Is it positive (amplifying) or negative (balancing)?
- **Choose the right tool** — Virtuous for growth, stabilizing for control, error-correcting for resilience
- **Add controls** — Prevent runaway behavior in positive loops
- **Monitor behavior** — Track how the loop performs over time
- **Use appropriately** — Different situations require different types

If you take away one thing, let it be this: **feedback loops are the engine of adaptation and learning in systems**. Positive loops accelerate growth. Negative loops maintain stability. Error-correcting loops build resilience. Understanding which type you're dealing with—and how to control it—is the difference between systems that survive and systems that collapse.

---

## Check Your Understanding

Let's see if you've got this. Here are a couple of questions to test your understanding.

### Question 1

You're designing a social media platform's recommendation algorithm. Which feedback loop type is most appropriate for encouraging viral growth of high-quality content?

> "Users can recommend content to their followers. When a follower likes, shares, or comments on recommended content, the algorithm learns these signals and recommends similar content to that follower's connections. The goal is to spread high-quality content that aligns with the platform's values, not clickbait or misleading information."

**A)** Positive (Virtuous) - Virtuous growth loop
**B)** Positive (Vicious) - Vicious growth loop
**C)** Negative (Self-Regulating) - Self-regulating loop
**D)** Negative (Stabilizing) - Stabilizing control loop

<details>
<summary>Click to see the answer</summary>

**Answer: A) Positive (Virtuous) - Virtuous growth loop**

**Explanation:**

Let's analyze each option:

**A) Correct!** A virtuous cycle (positive, reinforcing) is the right choice because:
- The goal is **amplifying** desirable behavior (high-quality content)
- The feedback loop **reinforces** the right things (when users like/share/comment on good content, recommend more of it)
- Output amplifies input in a **controlled way** that leads to exponential growth
- It's designed to create a **virtuous cycle** where good content gets recommended to more people, who like/share/comment, creating more signals for the algorithm
- This type of loop is intentionally designed for **growth**, but with safeguards (content quality checks, relevance filters) to prevent amplifying undesirable content

**Why other options are wrong:**

**B)** Incorrect. A vicious cycle (positive, destructive) would amplify content indiscriminately without quality checks. This could lead to:
- Clickbait or misleading content going viral (engagement without quality)
- Misinformation spreading faster than fact-checking can keep up
- Users getting tired of seeing the same type of content repeatedly
- Platform losing trust when low-quality content becomes common

The vicious cycle amplifies output, but it's destructive amplification. The virtuous cycle amplifies output too, but in a **controlled, quality-focused way**. The key difference is that the virtuous cycle has safeguards and quality filters, while the vicious cycle doesn't.

**C)** Incorrect. A self-regulating loop (negative, balancing) would maintain the current state of recommendations rather than promoting growth. This doesn't align with the goal of "encouraging viral growth of high-quality content." A self-regulating loop is for **stability**, not for **amplification**. It would keep the recommendation system in a steady state rather than spreading the best content to more people.

**D)** Incorrect. A stabilizing control loop (negative, constraining) would limit or throttle recommendations, which is the opposite of the goal. This type of loop is for **preventing** extremes (preventing spam, preventing abuse), not for **promoting** growth. It constrains rather than amplifies.

**Key insight:** Positive feedback loops aren't inherently good or bad—they're tools. The virtuous cycle uses positive feedback to achieve a desirable goal (growth) while having safeguards to prevent negative outcomes. The vicious cycle lacks those safeguards and causes collapse. The key is designing your feedback loop intentionally—knowing what you want to amplify and what controls you need to prevent unwanted side effects.

</details>

---

### Question 2

You're analyzing an e-commerce site's auto-scaling system. The system currently has these behaviors:
- When CPU > 80%, it adds instances immediately
- When CPU < 60%, it removes instances immediately
- You're seeing CPU oscillate between 55% and 85% every few minutes

What's the problem and which type of feedback loop would you implement to fix it?

**A)** Add a delay between scaling actions
**B)** Switch to negative (stabilizing) feedback loop
**C)** Add hysteresis to the scaling algorithm
**D)** Implement a deadband zone (min CPU and max CPU)

<details>
<summary>Click to see the answer</summary>

**Answer: C) Add hysteresis to the scaling algorithm**

**Explanation:**

Let's analyze the situation and each option:

**The problem:** The system is oscillating—CPU bounces between 55% and 85% every few minutes. This is a classic sign of **delayed feedback causing oscillation**. The system's feedback loop has a time delay, and it's overcorrecting in response.

**Why it's happening:**
1. CPU hits 85% → System adds instances
2. A few minutes pass (delay)
3. CPU drops to 55% because new instances take time to warm up
4. System sees low CPU → Removes instances
5. CPU bounces back up → System adds instances again
6. [Loop repeats]

The delay (time for instances to warm up/cool down) combined with the system's aggressive reaction (add/remove immediately) creates a continuous oscillation around the target (70%) that the system can never settle at.

**Why option A is wrong:**
Adding a delay would actually make oscillation worse. The system would respond even more slowly to changes, and the phase lag between the action and feedback would increase, potentially creating more severe oscillation or making the system more unstable. You don't want more delay in a feedback loop that's already oscillating.

**Why option B is wrong:**
Switching to a negative (stabilizing) feedback loop is appropriate when you have a target and want to maintain it. But in this case, the system doesn't have a clear target—it's reacting to load with add/remove decisions. The problem isn't that it's scaling wrong (too much or too little), it's that it's **oscillating**. A stabilizing loop would help if the system was consistently scaling too high or too low, but here the issue is the **instability** caused by the oscillation itself, not the scaling decisions.

**Why option D is wrong:**
A deadband zone (min CPU and max CPU) would prevent oscillation by restricting the system's range. This would stop the oscillation, but at a cost: the system couldn't scale above max CPU even if needed, and would throttle during legitimate spikes. This is a brute-force solution that sacrifices flexibility for stability. It might be appropriate in some cases (preventing DDoS attacks), but it's not the right solution for a general oscillation problem.

**Why option C is correct:**
Adding hysteresis (memory of past states) to the scaling algorithm would solve the oscillation:

```sruja
// partial
// Hysteresis-based auto-scaling
HysteresisScaling = scenario "Auto-Scaling with Hysteresis" {
  // Track past CPU values
  CPUHistory -> History "Store last 5 CPU readings"
  
  // Smoothed response with memory
  History -> HysteresisController "Calculate smoothed CPU"
  
  // Use smoothed CPU for decisions
  HysteresisController -> AutoScaler "Use smoothed CPU (65%)"
  
  // Don't scale up just because of one spike
  if cpu_spike {
    HysteresisController -> AutoScaler "Wait, verify spike is real"
  }
}
```

**How hysteresis works:**
- Instead of reacting instantly to every CPU reading, the system remembers recent values
- It averages or smooths the readings to filter out transient spikes
- It considers the trend, not just the current value
- It adds inertia—resists changing direction based on a single anomaly

**Why this fixes oscillation:**
- When CPU spikes temporarily (one reading at 85%), the system remembers "we were just at 70%, this is probably a spike"
- It doesn't scale up aggressively in response to the spike
- Instead, it scales up gradually if the smoothed CPU remains high
- When CPU drops back down, it doesn't scale down immediately— remembers "we were just high"

This smooths out the oscillation. The system still scales to meet load, but it does so more calmly and predictably, eliminating the bouncy behavior.

**Key insight:** Delayed feedback often causes oscillation because systems overcorrect to changes that were already addressed. Hysteresis (adding memory) smooths out the response by considering the system's history, preventing the overcorrection that drives oscillation. It's like a thermostat that remembers the room has been warming up for the last 10 minutes and doesn't turn off the heater just because the temperature momentarily hits the target.

</details>

---

## What's Next?

Now you understand the different types of feedback loops—positive (reinforcing) and negative (balancing)—and when to use each. You've seen how positive loops can create virtuous growth or vicious collapse. You've seen how negative loops maintain stability through self-regulation, error-correction, and control.

In the next lesson, you'll learn to **model cycles explicitly** in Sruja. You'll discover how to create valid feedback loops, distinguish them from circular dependencies (which are bad), and use cycles to represent self-regulating systems, learning mechanisms, and adaptive behaviors.

See you there!
