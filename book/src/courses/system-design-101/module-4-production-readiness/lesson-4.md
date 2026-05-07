title: "Lesson 4: SLOs & Scale Integration"
weight: 4
summary: "Define meaningful SLOs and align capacity to meet reliability targets."
---

# Lesson 4: SLOs & Scale Integration

"We guarantee 99.99% availability."

That's what our sales team promised the enterprise customer. It was in the contract. A $5M contract that would make our quarter.

Six months later, the customer demanded their SLA credits. They'd experienced 14 hours of downtime. We'd promised 99.99% availability (52 minutes of downtime per year), but we'd delivered 99.5%.

**The problem?** We had no idea we were failing. We weren't measuring availability. We had no SLOs, no monitoring, no alerts. We just had a promise we couldn't keep.

The customer got $500K in credits. The sales team was furious. The engineering team was embarrassed. And I learned a hard lesson: **a promise without measurement is just a lie.**

That's when I discovered SLOs (Service Level Objectives). Not as a theoretical concept, but as a survival mechanism. This lesson is about how to define meaningful SLOs, measure them rigorously, and align your architecture to actually meet them.

## What Are SLOs, Really?

**Service Level Objectives (SLOs)** are specific, measurable targets for your service's reliability. They answer the question: "What does 'good enough' look like?"

### The Three Components

Every SLO has three parts:

1. **The Metric:** What are you measuring? (latency, availability, error rate)
2. **The Target:** What's the threshold? (99.9%, 200ms, < 0.1%)
3. **The Window:** Over what time period? (30 days, 7 days, 24 hours)

**Example:**
```
Metric: Availability
Target: 99.9%
Window: 30 days
```

This means: "Over any 30-day period, our service must be available 99.9% of the time."

**Why this matters:**
- **Customers know what to expect:** Clear reliability commitment
- **Engineering knows what to build:** Target to design for
- **Product knows when to freeze features:** If SLO is breached, stop shipping
- **Finance knows what it costs:** Reliability has a price

## The SLO Hierarchy

### SLA > SLO > SLI

**SLA (Service Level Agreement):** The promise you make to customers. Usually has financial consequences.

**Example:** "99.9% availability or we'll give you 10% credit on your bill."

**SLO (Service Level Objective):** The internal target you set for your team. Should be stricter than your SLA.

**Example:** "We target 99.95% availability internally so we never breach the 99.9% SLA."

**SLI (Service Level Indicator):** The actual measurement.

**Example:** "Last month we achieved 99.93% availability."

**The relationship:**
- SLI is reality (what you actually delivered)
- SLO is the goal (what you're aiming for)
- SLA is the contract (what you promised)

**Best practice:** Set your SLO higher than your SLA. Give yourself headroom.

## What Makes a Good SLO?

### The SMART Framework for SLOs

**S - Specific:** Clear metric with no ambiguity

❌ "The system should be fast"
✅ "API latency p95 < 200ms"

**M - Measurable:** Can be objectively measured automatically

❌ "Users should be happy"
✅ "Error rate < 0.1%"

**A - Achievable:** Realistic given your current architecture

❌ "100% availability" (impossible)
✅ "99.9% availability" (challenging but achievable)

**R - Relevant:** Measures something users actually care about

❌ "Server CPU utilization"
✅ "Request latency" (users care about speed)

**T - Time-bound:** Defined over a specific window

❌ "System is usually available"
✅ "99.9% availability over 30 days"

## Types of SLOs

### 1. Availability SLOs

**What it measures:** Is the service working?

**How to calculate:** `(Total time - Downtime) / Total time`

**Example:**
```sruja
// partial
slo {
  availability {
    target "99.9%"  // 8.76 hours downtime/year allowed
    window "30 days"
    current "99.95%"
  }
}
```

**Real-world example:** Netflix

Netflix targets 99.99% availability for their streaming service. That's 52 minutes of downtime per year. How do they achieve it? Chaos engineering. They break things on purpose to ensure resilience.

### 2. Latency SLOs

**What it measures:** How fast does the service respond?

**How to calculate:** Percentiles (p50, p95, p99)

**Example:**
```sruja
// partial
slo {
  latency {
    p95 "200ms"  // 95% of requests faster than 200ms
    p99 "500ms"  // 99% of requests faster than 500ms
    window "7 days"
    current {
      p95 "180ms"
      p99 "450ms"
    }
  }
}
```

**Real-world example:** Amazon

Amazon found that every 100ms of latency cost 1% in sales. They have strict latency SLOs: p99 < 100ms for most services. Their architecture is optimized for speed because speed directly impacts revenue.

**Why percentiles matter:**

❌ **Average latency:** "Average latency is 50ms"
- Problem: Hides outliers. If 5% of requests take 10 seconds, average might still look fine.

✅ **Percentile latency:** "p95 latency is 200ms"
- Benefit: 95% of users get < 200ms. You know the worst case most users experience.

### 3. Error Rate SLOs

**What it measures:** What percentage of requests fail?

**How to calculate:** `Failed requests / Total requests`

**Example:**
```sruja
// partial
slo {
  error_rate {
    target "< 0.1%"  // Fewer than 1 in 1000 requests fail
    window "30 days"
    current "0.05%"
  }
}
```

**Real-world example:** Stripe

Stripe processes billions in payments. Their error rate SLO is < 0.01% (1 in 10,000 requests). Every failed payment is lost revenue and frustrated customers. They achieve this through retry logic, circuit breakers, and graceful degradation.

### 4. Throughput SLOs

**What it measures:** How many requests can you handle?

**How to calculate:** `Requests per second (req/s)`

**Example:**
```sruja
// partial
slo {
  throughput {
    target "1000 req/s"  // Handle 1000 requests per second
    window "1 hour"
    current "950 req/s"
  }
}
```

**Real-world example:** Uber

During New Year's Eve, Uber's throughput spikes 10x. Their throughput SLO ensures they can handle the surge: 100,000 ride requests per second globally. They achieve this through massive auto-scaling and capacity planning.

## Error Budgets: The Most Important Concept

**What is an error budget?**

An error budget is the amount of unreliability you can afford before breaching your SLO. It's the difference between 100% and your SLO target.

**Example:**
```
SLO: 99.9% availability over 30 days
Total time in 30 days: 43,200 minutes
Allowed downtime (error budget): 43.2 minutes

If you've had 20 minutes of downtime this month:
Remaining error budget: 23.2 minutes
```

### How to Use Error Budgets

**Google's approach (popularized in their SRE book):**

**1. When error budget is HEALTHY (plenty remaining):**
- Take more risks
- Launch new features faster
- Reduce operational toil
- Experiment with architecture changes

**2. When error budget is DEPLETED (barely any remaining):**
- Freeze new features
- Focus on reliability
- Pay down technical debt
- Add more tests
- Improve monitoring

**3. When error budget is EXCEEDED (SLO breached):**
- Incident review required
- Post-mortem mandatory
- No new features until SLO recovers

**Real-world example:** Google Search

Google Search has a 99.99% availability SLO. When they're within budget, they push changes aggressively. When budget is tight, they slow down. This balance lets them innovate while staying reliable.

### Error Budget Calculator

```
Monthly Error Budget for 99.9% availability:
- 30 days × 24 hours × 60 minutes = 43,200 minutes
- Allowed downtime: 0.1% × 43,200 = 43.2 minutes/month

Monthly Error Budget for 99.99% availability:
- Allowed downtime: 0.01% × 43,200 = 4.32 minutes/month

Monthly Error Budget for 99.999% availability:
- Allowed downtime: 0.001% × 43,200 = 0.432 minutes/month (26 seconds!)
```

**The lesson:** Higher SLOs are exponentially harder and more expensive.

## Aligning Scale with SLOs

This is where architecture meets reliability. Your SLOs determine your capacity requirements.

### The Capacity-SLO Relationship

**Key insight:** You need headroom to meet SLOs during traffic spikes.

**Example:**
```
Current traffic: 500 req/s
Traffic spikes: Up to 2x (1000 req/s during peak)
SLO target: 1000 req/s throughput
Required capacity: 1000 req/s minimum

But wait - if you're at 100% capacity, you can't handle:
- Unexpected spikes (> 2x)
- Server failures
- Deployment rollouts
- Performance degradation

Best practice: Run at 50-70% capacity. Have 30-50% headroom.
```

### Sruja: Modeling Scale with SLOs

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

ECommerce = system "E-Commerce Platform" {
  API = container "API Service" {
    technology "Rust"
    
    // Define your SLO first
    slo {
      throughput {
        target "1000 req/s"
        window "1 hour"
      }
      
      latency {
        p95 "200ms"
        p99 "500ms"
        window "7 days"
      }
      
      availability {
        target "99.9%"
        window "30 days"
      }
    }
    
    // Then define scale to support the SLO
    scale {
      metric "cpu"
      
      // Baseline capacity (support 1000 req/s at 50% CPU)
      min 5
      
      // Burst capacity (handle 2x spikes)
      max 20
      
      // Auto-scale trigger
      scale_up "cpu > 70%"
      scale_down "cpu < 30%"
    }
  }
}

view index {
  title "Production System with SLO-Aligned Scale"
  include *
}
```

**Key principle:** Start with SLO, then design scale. Not the other way around.

### The Headroom Calculation

**Formula:**
```
Required Capacity = Peak Traffic × Headroom Factor

Where Headroom Factor:
- 1.3x for non-critical services (30% headroom)
- 1.5x for standard services (50% headroom)
- 2.0x for critical services (100% headroom)
```

**Example:**
```
Service: Payment API (critical)
Peak traffic: 500 req/s
Headroom factor: 2.0x
Required capacity: 1000 req/s

If each instance handles 100 req/s:
Min instances: 10
```

### Real-World Capacity Planning

**Netflix's approach:**

Netflix tracks their "efficiency" metric: actual usage vs. provisioned capacity.

- **Too low (< 30%):** Wasting money
- **Too high (> 80%):** Risk of SLO breach
- **Target (50-70%):** Optimal balance

They auto-scale based on this, ensuring they meet SLOs without overspending.

## Setting SLOs: The Framework

### Step 1: Identify User Journeys

What are the critical paths users take through your system?

**Example for e-commerce:**
1. User searches for product
2. User views product details
3. User adds to cart
4. User checks out
5. User pays

**Prioritize:** Which journeys matter most? (Checkout and payment are more critical than search)

### Step 2: Choose Metrics

For each journey, what metrics matter?

- **Search:** Latency (users want fast results)
- **Product view:** Availability (must work)
- **Cart:** Availability + durability (don't lose items)
- **Checkout:** Availability + latency (fast and reliable)
- **Payment:** Availability + error rate (must succeed)

### Step 3: Measure Current Performance

Before setting targets, measure reality:

```
Current performance (last 30 days):
- Availability: 99.5%
- Latency p95: 350ms
- Error rate: 0.3%
```

### Step 4: Set Achievable Targets

Based on current performance, set realistic targets:

```
Current: 99.5% availability
Target: 99.7% availability (improvement, not perfection)
Future: 99.9% availability (long-term goal)
```

**The mistake to avoid:** Setting 99.99% SLO when you're at 99%. You'll fail constantly.

### Step 5: Create Alerts

Alert when you're burning error budget too fast:

```
Alert 1: Burn rate > 10x (will exhaust budget in 3 days)
Alert 2: Burn rate > 2x (will exhaust budget in 15 days)
Alert 3: Budget < 20% remaining
```

## SLO Anti-Patterns

### Anti-Pattern #1: The 100% SLO

**What happens:** You set 100% availability as your SLO.

**Why it fails:**
- Impossible to achieve
- Paralyzes the team (afraid to make changes)
- Expensive (massive over-provisioning)
- Still fails eventually

**The fix:** 100% is not an SLO, it's a fantasy. Aim for 99.9% or 99.99%.

### Anti-Pattern #2: Measuring the Wrong Thing

**What happens:** You measure CPU utilization instead of user experience.

**Why it fails:**
- CPU can be high while users are happy
- CPU can be low while users are frustrated
- Doesn't capture actual reliability

**The fix:** Measure what users experience: latency, availability, errors.

### Anti-Pattern #3: Too Many SLOs

**What happens:** You define 20 different SLOs for one service.

**Why it fails:**
- Information overload
- No clear priorities
- Alert fatigue
- Impossible to track

**The fix:** 3-5 SLOs per service maximum. Focus on what matters most.

### Anti-Pattern #4: SLOs in a Drawer

**What happens:** You define SLOs, document them, then never look at them again.

**Why it fails:**
- No feedback loop
- No behavior change
- Wasted effort

**The fix:** SLOs must be:
- Visible on dashboards
- Part of deployment decisions
- Reviewed regularly
- Updated when needed

### Anti-Pattern #5: SLOs Set by Management

**What happens:** Management dictates 99.99% availability without consulting engineering.

**Why it fails:**
- Unrealistic given architecture
- No buy-in from team
- Sets up for failure

**The fix:** Collaborative SLO setting:
- Management sets business requirements
- Engineering measures current performance
- Together, define achievable targets

### Anti-Pattern #6: No Error Budget

**What happens:** You have SLOs but no concept of error budget.

**Why it fails:**
- Binary view (pass/fail)
- No nuance in decision-making
- Can't balance reliability vs. velocity

**The fix:** Calculate and track error budgets. Use them to guide decisions.

## The SLO Framework: RELIABLE

When implementing SLOs, use this framework:

**R - Review User Journeys**
- What do users actually do?
- What matters most to them?

**E - Establish Metrics**
- Availability, latency, errors, throughput
- What captures user experience?

**L - Look at Current Performance**
- Measure before targeting
- Don't guess, measure

**I - Incremental Targets**
- Improve gradually
- Don't aim for perfection immediately

**A - Automate Measurement**
- Continuous monitoring
- Real-time dashboards

**B - Burn Rate Alerts**
- Alert before SLO breach
- Give time to react

**L - Link to Decisions**
- Error budget guides feature velocity
- SLO status affects deployments

**E - Evolve Over Time**
- SLOs aren't static
- Adjust as you improve

## Real-World SLO Examples

### Google: The Gold Standard

Google popularized SLOs in their SRE books. Here's how they approach it:

**Service:** Google Search
**Availability SLO:** 99.99%
**Latency SLO:** p99 < 200ms
**Error budget policy:** When budget exhausted, freeze launches until recovered

**How they achieve it:**
- Massive redundancy (multiple data centers)
- Automatic failover
- Chaos engineering
- Rigorous capacity planning

**Result:** Google Search is down so rarely it makes global news when it happens.

### Netflix: Chaos Engineering

**Service:** Streaming Video
**Availability SLO:** 99.99%
**Throughput SLO:** Handle all subscriber streams concurrently

**How they achieve it:**
- Chaos Monkey (randomly kills instances)
- Multi-region active-active
- Graceful degradation (lower quality vs. failure)

**Result:** Even when AWS has outages, Netflix stays up.

### Amazon: Revenue-Driven SLOs

**Service:** Product Page
**Latency SLO:** p99 < 100ms
**Why:** Every 100ms of latency = 1% sales drop

**How they achieve it:**
- Edge caching (CloudFront)
- Optimized databases (DynamoDB)
- Microservices (isolated failures)

**Result:** Fast page loads drive revenue.

### Stripe: Payment Reliability

**Service:** Payment Processing
**Availability SLO:** 99.99%
**Error Rate SLO:** < 0.01%
**Latency SLO:** p95 < 300ms

**How they achieve it:**
- Retry logic (failed payments retry automatically)
- Circuit breakers (fail fast when downstream issues)
- Redundant payment processors
- Extensive monitoring

**Result:** Billions in payments processed with minimal failures.

## Common Questions About SLOs

**Q: What if I don't know what targets to set?**

A: Start by measuring current performance. Your first SLO can be "maintain current performance." Then improve from there.

**Q: How often should I review SLOs?**

A: Monthly for critical services, quarterly for others. Adjust targets based on actual performance and business needs.

**Q: What if my SLO is too hard to meet?**

A: Lower it. An unachievable SLO is worse than a loose one. Gradually tighten as you improve.

**Q: What if stakeholders demand 100% uptime?**

A: Explain the cost. 99.9% might cost $10K/month. 99.99% might cost $100K/month. 99.999% might cost $1M/month. Let them choose.

**Q: Should every service have SLOs?**

A: Production services: Yes. Internal tools: Maybe. Experiments: No. Focus on what matters.

## Practical Exercise: Define Your SLOs

Take a service you work on (real or hypothetical):

**Step 1: Identify Critical User Journey**
- What's the most important thing users do?
- Example: "Customer completes purchase"

**Step 2: Choose 3 Metrics**
- Availability (must work)
- Latency (must be fast)
- Error rate (must succeed)

**Step 3: Measure Current Performance**
- Look at last 30 days of data
- What's your current availability?
- What's your current latency?
- What's your current error rate?

**Step 4: Set Targets**
- Aim for 10-20% improvement
- If current availability is 99.5%, target 99.7%
- If current latency p95 is 500ms, target 400ms

**Step 5: Calculate Error Budget**
- How much downtime/latency/errors allowed?
- Example: 99.7% over 30 days = 130 minutes downtime allowed

**Step 6: Create Dashboard**
- Show SLO status
- Show error budget remaining
- Make it visible to team

**Time:** 45-60 minutes

## Complete Example: E-Commerce Platform

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

// ============ SERVICE DEFINITION ============

ECommerce = system "E-Commerce Platform" {
  API = container "API Service" {
    technology "Rust"
    description "Core API handling all business logic"
    
    // Metadata for governance
    metadata {
      owner "platform-team"
      criticality "high"
    }
    
    // SLOs define reliability targets
    slo {
      // Availability: Service must be up
      availability {
        target "99.9%"  // 8.76 hours downtime/year
        window "30 days"
        current "99.92%"
        
        error_budget {
          total "43.2 minutes"
          remaining "35.1 minutes"
          status "healthy"
        }
      }
      
      // Latency: Service must be fast
      latency {
        p95 "200ms"  // 95% of requests < 200ms
        p99 "500ms"  // 99% of requests < 500ms
        window "7 days"
        current {
          p95 "180ms"
          p99 "420ms"
        }
      }
      
      // Error Rate: Service must succeed
      error_rate {
        target "< 0.1%"  // Fewer than 1 in 1000 requests fail
        window "30 days"
        current "0.08%"
      }
      
      // Throughput: Service must handle load
      throughput {
        target "1000 req/s"  // Handle peak traffic
        window "1 hour"
        current "950 req/s"
      }
    }
    
    // Scale configuration supports SLOs
    scale {
      metric "cpu"
      
      // Minimum: Always have capacity for normal load + headroom
      min 5  // 5 instances × 200 req/s = 1000 req/s capacity
      
      // Maximum: Can scale for 2x spikes
      max 15  // 15 instances × 200 req/s = 3000 req/s capacity
      
      // Auto-scale triggers
      scale_up "cpu > 70%"
      scale_down "cpu < 30% cooldown 10m"
    }
  }
  
  Database = database "PostgreSQL" {
    technology "PostgreSQL"
    description "Primary data store"
    
    metadata {
      owner "platform-team"
      criticality "high"
    }
    
    slo {
      availability {
        target "99.95%"  // Database more critical than API
        window "30 days"
        current "99.96%"
      }
      
      latency {
        p95 "50ms"  // Database should be fast
        p99 "100ms"
        window "7 days"
        current {
          p95 "45ms"
          p99 "92ms"
        }
      }
    }
  }
  
  Cache = database "Redis" {
    technology "Redis"
    description "Cache layer for performance"
    
    slo {
      availability {
        target "99.9%"
        window "30 days"
      }
      
      hit_rate {
        target "> 80%"  // 80% of requests served from cache
        current "85%"
      }
    }
  }
}

// Relationships
ECommerce.API -> ECommerce.Database "Reads/Writes"
ECommerce.API -> ECommerce.Cache "Caches queries"

// ============ VIEWS ============

view index {
  title "E-Commerce Platform Architecture"
  include *
}

view slos {
  title "SLO Dashboard"
  include ECommerce.API ECommerce.Database ECommerce.Cache
  description "Monitor all SLOs in one view"
}

view capacity {
  title "Capacity & Scale"
  include ECommerce.API
  description "Focus on throughput and scaling"
}

// ============ GOVERNANCE ============

policy SLOPolicy "Critical services must have SLOs" {
  category "reliability"
  enforcement "required"
  
  rule {
    element_type "container"
    tag "criticality:high"
    requires_slo true
    error "Critical service {element} must have SLOs defined"
  }
}

// Tag critical services
ECommerce.API.tags ["criticality:high"]
ECommerce.Database.tags ["criticality:high"]
```

**Run validation:**
```bash
sruja validate architecture.sruja
```

This checks:
- SLOs are defined for critical services
- Scale configuration supports throughput targets
- All required metadata present

## What to Remember

1. **SLOs are promises backed by measurement** - Without measurement, it's just wishful thinking

2. **Start with user journeys** - Define what matters to users, then measure it

3. **Measure before you target** - Know your current performance before setting goals

4. **Error budgets enable velocity** - When budget is healthy, move fast. When tight, slow down.

5. **Headroom is essential** - Run at 50-70% capacity. Have 30-50% buffer.

6. **Percentiles over averages** - p95/p99 reveal what users actually experience

7. **Three SLOs maximum per service** - Availability, latency, error rate. Maybe throughput.

8. **SLOs should evolve** - As you improve, tighten targets. As business changes, adjust metrics.

9. **Make SLOs visible** - Dashboards, alerts, deployment gates. Everyone should know status.

10. **Collaborative target-setting** - Management defines needs, engineering defines feasibility, together define SLOs

## When to Start with SLOs

**Phase 1: Prototype (Skip SLOs)**
- Learning what to build
- No real users yet
- Focus on functionality

**Phase 2: Launch (Basic SLOs)**
- Real users exist
- Define availability SLO
- Basic monitoring

**Phase 3: Growth (Comprehensive SLOs)**
- Traffic increasing
- Add latency, error rate SLOs
- Error budget tracking
- SLO-based decisions

**Phase 4: Scale (Advanced SLOs)**
- High traffic
- Multi-region SLOs
- Per-feature SLOs
- Sophisticated alerting

---

**Next up:** Lesson 5 brings everything together with a comprehensive production readiness review - how to know when your architecture is truly ready for production.
