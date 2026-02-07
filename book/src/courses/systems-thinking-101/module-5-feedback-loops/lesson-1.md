# Lesson 1: Understanding Feedback Loops

## Learning Goals

- Understand what feedback loops are
- Recognize feedback loops in everyday systems
- Learn why cycles are not errors

## What Are Feedback Loops?

A feedback loop occurs when an action creates a reaction that affects future actions. It's a cycle where the output becomes input for the next iteration.

```
Action → Response → Adjustment → Action (repeated)
```

## Everyday Examples

### Example 1: Thermostat

```
Temperature drops
    ↓
Thermostat detects low temp
    ↓
Turns on heater
    ↓
Temperature rises
    ↓
Thermostat turns off heater
    ↓
Temperature drops
    ↓
[Loop repeats]
```

### Example 2: Feedback at Work

```
Submit code for review
    ↓
Manager provides feedback
    ↓
Developer fixes issues
    ↓
Submit revised code
    ↓
[Loop repeats until approved]
```

### Example 3: Social Media Algorithm

```
User watches video
    ↓
Algorithm recommends similar videos
    ↓
User watches more
    ↓
Algorithm learns preferences
    ↓
[Loop reinforces the behavior]
```

## Feedback Loops in Software Architecture

### Example 1: Auto-Scaling

```sruja
// Positive feedback loop: Scale up when load increases
MonitoringSystem -> App.API "Detects high load"
App.API -> AutoScaler "Request scale up"
AutoScaler -> App.API "Adds more instances"
App.API -> MonitoringSystem "Reports lower load"
```

### Example 2: User Experience

```sruja
// User feedback loop
User -> App.WebApp "Submits form"
App.WebApp -> App.API "Validates"
App.API -> App.WebApp "Returns errors"
App.WebApp -> User "Shows errors"
// User corrects and resubmits (loop)
```

### Example 3: Inventory Management

```sruja
// Inventory feedback loop
Shop.API -> Inventory "Updates stock"
Inventory -> Shop.API "Notifies low stock"
Shop.API -> Admin "Sends alert"
Admin -> Shop.API "Restocks inventory"
Shop.API -> Inventory "Updates stock"
// Loop continues as inventory changes
```

## Why Feedback Loops Matter

### 1. Self-Regulation

Systems can adjust automatically:

```sruja
AutoScaling = scenario "Auto-Scaling Feedback" {
  Monitor -> App "Detects high CPU"
  App -> ScalingService "Request scale up"
  ScalingService -> App "Add instances"
  App -> Monitor "CPU decreases"
  // If CPU still high, loop repeats
}
```

### 2. Learning and Adaptation

Systems improve over time:

```sruja
MLFeedback = scenario "Machine Learning Feedback" {
  User -> App "Rates recommendation"
  App -> MLModel "Update preferences"
  MLModel -> App "Improved recommendations"
  App -> User "Better suggestions"
  // User rates again, model improves
}
```

### 3. Error Recovery

Systems recover from failures:

```sruja
RetryLoop = scenario "Retry with Backoff" {
  App -> ExternalAPI "Make request"
  ExternalAPI -> App "Request failed (timeout)"

  App -> ExternalAPI "Retry in 1s"
  ExternalAPI -> App "Request failed"

  App -> ExternalAPI "Retry in 2s"
  ExternalAPI -> App "Success"
}
```

### 4. Resource Management

Optimize resource usage:

```sruja
ResourceFeedback = scenario "Cache Feedback" {
  App -> Cache "Check cache"
  Cache -> App "Cache miss"
  App -> Database "Query data"
  Database -> App "Return data"
  App -> Cache "Store in cache"

  // Next time: Cache hit
  App -> Cache "Check cache"
  Cache -> App "Cache hit (faster)"
}
```

## Feedback Loop Types

### 1. Positive Feedback (Reinforcing)

Increases the effect, leading to exponential growth or collapse:

```sruja
// Example: Viral sharing
UserA -> App "Shares content"
App -> UserB "Shows content"
UserB -> App "Shares content"
App -> UserC "Shows content"
UserC -> App "Shares content"
// More users see and share
```

**Use when**: viral growth, network effects, learning systems

### 2. Negative Feedback (Balancing)

Reduces the effect, maintaining stability:

```sruja
// Example: Temperature control
Thermostat -> Heater "Turn on if too cold"
Heater -> Room "Heats room"
Room -> Thermostat "Temperature reading"
Thermostat -> Heater "Turn off if too warm"
// Maintains stable temperature
```

**Use when**: Stability, control, regulation

### 3. Delayed Feedback

Feedback occurs after a delay:

```sruja
// Example: Performance monitoring
App -> Database "Slow query"
Database -> App "Returns data"
App -> Analytics "Logs slow query"
Analytics -> App "Sends alert (after threshold)"

// Alert arrives later, system adjusts
```

## Are Cycles Bad?

In traditional software architecture, **circular dependencies** are considered bad. But in systems thinking, **cycles are natural and valid**.

### Circular Dependency (Bad)

```sruja
// Bad: Module A depends on B, B depends on A
ModuleA -> ModuleB "Calls"
ModuleB -> ModuleA "Calls"

// Problem: Impossible to initialize, tight coupling
```

### Feedback Loop (Good)

```sruja
// Good: System learns from output
User -> System "Submits data"
System -> User "Shows result"
User -> System "Adjusts based on feedback"

// Problem: Natural, enables adaptation
```

## Key Differences

| Circular Dependency   | Feedback Loop                     |
| --------------------- | --------------------------------- |
| Static (compile-time) | Dynamic (runtime)                 |
| Tight coupling        | Loose coupling with clear purpose |
| Avoid                 | Embrace                           |
| Makes system brittle  | Makes system adaptive             |
| No clear purpose      | Serves a specific function        |

## Exercise

Identify feedback loops in these scenarios:

1. **Chat application**: User sends message, app shows typing indicator, receiver sees it, receiver starts typing, sender sees typing indicator...

2. **Recommendation system**: User watches video, algorithm updates preferences, recommends more videos, user watches more, algorithm learns more...

3. **CI/CD pipeline**: Developer commits code, tests run, if fails developer fixes, commits again, tests run...

## Key Takeaways

1. **Feedback loops are cycles** where output affects future input
2. **They're everywhere**: In nature, software, everyday life
3. **Not errors**: Unlike circular dependencies, feedback loops are valid patterns
4. **Enable adaptation**: Systems can self-regulate and improve
5. **Two main types**: Positive (reinforcing) and negative (balancing)

## Next Lesson

In [Lesson 2](lesson-2.md), you'll learn about different types of feedback loops and when to use each.
