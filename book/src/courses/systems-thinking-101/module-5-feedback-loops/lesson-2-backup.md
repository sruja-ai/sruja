# Lesson 2: Types of Feedback Loops

## Learning Goals

- Understand positive, negative, and balancing feedback loops
- Learn when to use each type
- Recognize feedback loop behavior in systems

## Feedback Loop Classification

```
Feedback Loops
├── Positive (Reinforcing)
│   ├── Virtuous cycle (good)
│   └── Vicious cycle (bad)
└── Negative (Balancing)
    ├── Self-regulating
    ├── Error-correcting
    └── Stabilizing
```

## Positive Feedback Loops (Reinforcing)

Amplify change, leading to exponential growth or collapse.

### Pattern 1: Virtuous Cycle (Growth)

```sruja
VirtuousCycle = scenario "Network Effects" {
  UserA -> App "Joins platform"
  App -> UserB "Invites friend"
  UserB -> App "Joins platform"
  App -> UserC "Invites friend"

  // More users → More value → More users
  App -> Users "Platform value increases"
  Users -> App "More users join"
}
```

**Use when**: Viral growth, network effects, learning systems

**Examples**:

- Social networks (more users = more connections)
- Marketplaces (more sellers = more buyers)
- Machine learning (more data = better predictions)

### Pattern 2: Vicious Cycle (Collapse)

```sruja
ViciousCycle = scenario "Performance Degradation" {
  User -> App "Makes request"
  App -> Database "Query"

  // Slow query → Queue builds → Slower queries
  Database -> App "Slow response"
  App -> Queue "Requests back up"
  Queue -> Database "More load"
  Database -> App "Even slower response"

  // If not interrupted, system collapses
}
```

**Use when**: Identifying failure modes, designing circuit breakers

**Examples**:

- System overload
- Cache stampede
- Database deadlock cascade

### Pattern 3: Learning Loop (Improvement)

```sruja
LearningLoop = scenario "ML Model Improvement" {
  User -> Model "Makes prediction"
  Model -> User "Shows result"
  User -> Model "Rates accuracy"

  // Better data → Better model → Better predictions
  Model -> Training "Updates with rating"
  Training -> Model "Improved model"
  Model -> User "Better predictions"
}
```

**Use when**: Machine learning, recommendation systems, A/B testing

**Examples**:

- Search engine relevance
- Ad targeting
- Personalized recommendations

## Negative Feedback Loops (Balancing)

Counteract change, maintaining stability and equilibrium.

### Pattern 1: Self-Regulating (Homeostasis)

```sruja
Homeostasis = scenario "Auto-Scaling" {
  Monitoring -> App "Detects high load"
  App -> ScalingService "Request scale up"
  ScalingService -> App "Adds instances"
  App -> Monitoring "Reports lower load"

  // System self-regulates to target load
  Monitoring -> App "Detects low load"
  App -> ScalingService "Request scale down"
  ScalingService -> App "Removes instances"
}
```

**Use when**: Auto-scaling, resource management, capacity planning

**Examples**:

- Server auto-scaling
- Database connection pooling
- Rate limiting

### Pattern 2: Error-Correcting (Resilience)

```sruja
ErrorCorrection = scenario "Retry with Backoff" {
  App -> Service "Make request"
  Service -> App "Failure"

  // Exponential backoff
  App -> Service "Retry after 1s"
  Service -> App "Failure"
  App -> Service "Retry after 2s"
  Service -> App "Failure"
  App -> Service "Retry after 4s"
  Service -> App "Success"

  // System recovers from transient errors
}
```

**Use when**: Error handling, resilience, fault tolerance

**Examples**:

- Retry logic
- Circuit breakers
- Fallback mechanisms

### Pattern 3: Stabilizing (Control)

```sruja
Stabilizing = scenario "Rate Limiting" {
  User -> API "Send request"
  API -> RateLimiter "Check rate"

  // If under limit, allow
  RateLimiter -> API "Under limit"
  API -> User "Process request"

  // If over limit, throttle
  API -> RateLimiter "Check rate"
  RateLimiter -> API "Over limit"
  API -> User "Rate limit exceeded"

  // System stabilizes request rate
}
```

**Use when**: Traffic control, resource allocation, load balancing

**Examples**:

- API rate limiting
- Load balancers
- Queue management

## Comparing Feedback Loops

| Type                 | Effect      | Stability | Growth    | Example         |
| -------------------- | ----------- | --------- | --------- | --------------- |
| Positive (Virtuous)  | Amplifies   | Decreases | Increases | Viral growth    |
| Positive (Vicious)   | Amplifies   | Decreases | Decreases | System collapse |
| Negative (Balancing) | Counteracts | Increases | Maintains | Auto-scaling    |

## Delayed Feedback

Feedback occurs after a delay, can cause oscillation.

```sruja
DelayedFeedback = scenario "Delayed Monitoring" {
  App -> System "Request processed"

  // Delay before feedback
  System -> Analytics "Log event"
  Analytics -> Monitoring "Aggregate metrics"
  Monitoring -> App "Send alert (5 min delay)"

  // By the time alert arrives, system may have changed
}
```

**Mitigation**:

- Use real-time monitoring
- Reduce delays
- Add hysteresis (threshold buffer)

## Feedback Loop Behaviors

### Convergent

System reaches a stable state:

```sruja
Convergent = scenario "Converging to Equilibrium" {
  // System oscillates but stabilizes
  App -> Cache "Check cache"
  Cache -> App "Miss"
  App -> Database "Query"
  Database -> App "Data"
  App -> Cache "Store"

  // Next time: Cache hit (faster, stabilizes)
  App -> Cache "Check cache"
  Cache -> App "Hit (stable)"
}
```

### Divergent

System grows without bound or collapses:

```sruja
Divergent = scenario "Exponential Growth" {
  User -> App "Invite friend"
  App -> Friend "Invitation"
  Friend -> App "Accept and invite"
  App -> Friend2 "Invitation"
  Friend2 -> App "Accept and invite"

  // Without limits, grows exponentially
}
```

### Oscillating

System cycles between states:

```sruja
Oscillating = scenario "Hysteresis Loop" {
  Monitor -> App "Load high"
  App -> Scaling "Scale up"
  App -> Monitor "Load normal"

  Monitor -> App "Load low"
  App -> Scaling "Scale down"
  App -> Monitor "Load high"

  // Oscillates between scale up/down
}
```

## Designing Feedback Loops

### Step 1: Identify the Feedback

What creates the feedback?

```sruja
// What output becomes input?
User -> App "Action"
App -> User "Response"
// User's response becomes next action's input
```

### Step 2: Determine the Type

Is it reinforcing or balancing?

```sruja
// Reinforcing: Increases effect
App -> ML "More data"
ML -> App "Better model"

// Balancing: Maintains equilibrium
App -> Monitor "Check load"
Monitor -> App "Adjust capacity"
```

### Step 3: Add Controls

Prevent runaway behavior:

```sruja
AutoScaling = container "Auto-Scaling Service" {
  scale {
    min 2
    max 10
    metric "cpu > 80%"
  }
}
```

### Step 4: Monitor

Observe behavior:

```sruja
Monitor = system "Monitoring System" {
  metrics {
    cpu_usage
    request_rate
    error_rate
    feedback_loop_iterations
  }
}
```

## Exercise

Classify these feedback loops:

1. "More users join platform → More content → More users join..."
2. "System detects high load → Scales up → Load decreases → Scales down..."
3. "User likes video → Algorithm shows similar videos → User likes more → Algorithm learns..."
4. "Request fails → Retry with backoff → Eventually succeeds..."

## Key Takeaways

1. **Positive loops**: Reinforce change (growth or collapse)
2. **Negative loops**: Maintain stability (balance and regulate)
3. **Delayed feedback**: Can cause oscillation, needs monitoring
4. **Design intentionally**: Add controls to prevent runaway behavior
5. **Monitor behavior**: Ensure loops behave as expected

## Next Lesson

In [Lesson 3](lesson-3.md), you'll learn how to model feedback loops as valid cycles in Sruja.
