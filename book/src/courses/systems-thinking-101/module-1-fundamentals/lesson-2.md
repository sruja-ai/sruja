---
title: "Lesson 2: The Iceberg Model"
weight: 2
summary: "Why do same bugs keep appearing? Learn to look deeper than surface events."
time: "2 minutes"
---

# Lesson 2: Seeing Deeper with the Iceberg Model

## Learning Goals

By the end of this lesson, you'll be able to:
- Identify the four levels of the iceberg model
- Recognize when you're stuck at the surface events level
- Analyze problems by looking for patterns and root causes
- Understand how mental models shape system behavior

## Understanding the Iceberg Model

Have you ever felt like you're constantly putting out fires? You fix one bug, test it thoroughly, celebrate—and then two more similar bugs appear the next day. Or maybe you've watched your team debate the same architectural decision repeatedly, never quite resolving it?

These aren't just frustrating experiences. They're symptoms of looking at problems at the surface level—the "events" level—without understanding the deeper patterns, structures, and beliefs that create them.

The iceberg model gives us a way to look deeper.

## The Four Levels

The iceberg model has four levels, from surface to depth. Let's walk through each one with examples that might feel familiar.

### Events: What You See

**Events** are the things that happen right now—the incidents, bugs, crashes, and alerts you deal with daily.

Think of a typical Monday: "Server crashed at 2:37 PM" or "User reported a critical bug." These are events. They're immediate, observable, and often prompt a reactive response: "Fix it now."

The problem? Events are just symptoms. Fixing an event doesn't prevent it from happening again tomorrow.

### Patterns: What's Happening Over Time

When you look at events collectively, **patterns** start to emerge. These are trends and recurring sequences that tell you what's really going on.

Maybe you notice: "Similar bugs occur repeatedly after releases" or "Server crashes every Sunday night during backups." These aren't isolated incidents anymore—they're patterns.

Patterns tell you something important: this isn't random. There's something systemic happening here.

### Structures: What's Causing the Patterns

**Structures** are the underlying arrangements and mechanisms that create the patterns you're seeing. They're often invisible until you look for them.

Why do bugs keep appearing after releases? Maybe it's because components are tightly coupled, so a change in one creates cascading failures elsewhere. Why do servers crash during backups? Maybe it's because a single database becomes a bottleneck when backup processes run.

Structures are often about architecture, processes, or technology choices. They're the root causes.

### Mental Models: What's Shaping the Structures

This is the deepest level. **Mental models** are the beliefs, assumptions, and worldviews that influence the structures you create.

Why is your system tightly coupled? Maybe because your team believes "speed to market is everything, let's worry about architecture later." Why is there a single database bottleneck? Maybe because everyone assumes "we can scale a single database if we need to—let's not over-engineer."

Mental models are the hardest to change because they're often unspoken assumptions. But they're also the most powerful because they shape everything above them.

Here's how they connect:

```
"Mental models" → influence → "Structures" → create → "Patterns" → manifest as → "Events"
```

When you fix an event without changing the mental model, the pattern eventually returns.

## Seeing It in Practice

Let's make this concrete with a real example.

### A Real Architecture Scenario

Imagine you're working on an e-commerce platform, and you're dealing with slow page loads. Here's how the iceberg model helps you diagnose the problem:

```sruja
import { * } from 'sruja.ai/stdlib'

// Event: Slow page loads
// Pattern: Performance degrades after releases
// Structure: Monolithic architecture, no caching
// Mental Model: "Optimization is premature"

App = system "Web Application" {
  Monolith = container "Monolithic App"
  DB = database "Single Database"
}

Monolith -> DB "Heavy queries"
```

**Let's analyze this at each level:**

- **Event:** Users report slow page loads. You investigate, find slow queries, optimize them. Performance improves briefly.
- **Pattern:** Performance degrades after every release. Optimized queries don't solve it long-term. Something else is happening.
- **Structure:** The system is monolithic with no caching. Every page load hits the database. As the codebase grows, queries become heavier.
- **Mental Model:** The team believes "Optimization is premature—ship features first, worry about performance later." This shapes architectural decisions.

If you only fix the event (optimize queries), the pattern returns. If you redesign the structure (add caching, modularize), but don't address the mental model, you'll build similar bottlenecks in the new architecture. The real solution requires changing the belief about performance.

## When to Use Each Level

So when do you actually use each level? Here's a practical guide.

### Start with Events Level

You're at this level when you're reacting to immediate problems: debugging specific bugs, handling production alerts, or investigating user complaints. This is day-to-day firefighting. It's necessary, but if you never go deeper, you'll keep putting out the same fires.

### Move to Patterns Level

When you notice recurring issues, shift to the patterns level. This is for analyzing historical data, identifying trends, and planning for capacity and scaling.

Think about it: "Similar bugs appear after every release" is a pattern. It tells you something systemic is happening. At this level, you're asking: "What keeps happening?"

### Dive to Structures Level

This is where architecture and design decisions happen. You're here when designing systems, performing root cause analysis, planning refactoring, or evaluating technology choices.

At the patterns level, you know *what's* happening. At the structures level, you figure out *why* it's happening. You map out component relationships and identify root causes.

### Reach Mental Models Level

This is the deepest—and most powerful. You're here when making strategic decisions, changing team culture, setting long-term priorities, or aligning stakeholders on vision.

This is the hardest level to reach because mental models are often unspoken. But changing them creates lasting impact. You might ask: "What do we believe that makes this structure seem right?" or "What assumptions are driving our decisions?"

## How to Shift Between Levels

The real skill isn't knowing the four levels—it's knowing how to move between them when you're analyzing problems.

### From Events to Patterns

You're stuck at the events level when you keep seeing isolated incidents. To shift:

1. **Collect data over time** - Don't just look at today's incident. Look at the last month's incidents.
2. **Look for correlations and trends** - Are these incidents connected? Do they follow a pattern?
3. **Ask the right question:** "What keeps happening?"

### From Patterns to Structures

You're at the patterns level when you see recurring issues but don't know the cause. To shift:

1. **Analyze root causes** - Don't just acknowledge the pattern. Ask why it exists.
2. **Map out component relationships** - How do parts connect? Where are dependencies?
3. **Ask the right question:** "What creates these patterns?"

### From Structures to Mental Models

This is the hardest shift. You're at the structures level when you understand the architecture but don't understand why decisions were made. To shift:

1. **Identify assumptions and beliefs** - What does your team believe about this problem?
2. **Challenge deeply held views** - Are these assumptions still valid? What evidence supports them?
3. **Ask the right question:** "What do we believe that makes this structure seem right?"

### A Practical Example

Let's say your team keeps introducing tightly coupled components. Here's how you might analyze this:

**Start:** You notice a bug in Component A. Fix it. Same bug appears in Component B. This is the events level.

**Shift to patterns:** You realize "Coupling issues appear after every feature release." This is the patterns level.

**Shift to structures:** You investigate and find "Components share the same database table and call each other directly." This is the structures level.

**Shift to mental models:** You ask "Why did we design it this way?" and realize "We believed tight coupling would speed development." This is the mental models level.

Now you can change the mental model—and create different structures in the future.

## What to Remember

The iceberg model gives you a way to look deeper than surface problems. But here's the important part: the deeper you go, the more powerful your solutions become.

Think of it this way: fixing an event is like taking an aspirin for a headache—it might work temporarily, but the headache might come back tomorrow if you don't address the cause. Addressing patterns is like understanding what triggers your headaches. Addressing structures is like changing lifestyle habits that cause headaches. Addressing mental models is like understanding why you adopted those habits in the first place.

The shift in thinking is subtle but profound. Instead of asking "How do I fix this bug?", you ask "Why do similar bugs keep appearing? What structures create these patterns? What beliefs led to these structures?"

This doesn't mean you should always start at the mental models level. Sometimes you genuinely do need to fix a bug quickly. But when you see recurring problems, that's your signal to go deeper.

The iceberg model helps you ask better questions—and asking better questions is the first step to finding better answers.

## Check Your Understanding

Let's see if this is clicking.

### Quick Check

**1. You're investigating slow page loads on your site. You optimized a specific query and it's faster now—but similar performance issues keep appearing. Which level of the iceberg model should you move to next?**

[ ] A. Stay at Events level - keep optimizing individual queries
[ ] B. Patterns level - look for why these issues keep recurring
[ ] C. Structures level - redesign the architecture now
[ ] D. Mental Models level - change team beliefs about performance

**2. Your team believes "Ship fast, fix bugs later." You've noticed this leads to tightly coupled components and cascading failures. What level of the iceberg model is this belief?**

[ ] A. Events - it's a statement in meetings
[ ] B. Patterns - this belief appears in every project
[ ] C. Structures - it creates tight coupling in code
[ ] D. Mental Models - it's an assumption shaping your architecture

---

### Answers & Discussion

**1. B. Patterns level** – Optimizing individual queries is working at the Events level (fixing incidents). But since "similar performance issues keep appearing," that's a pattern signal. You need to identify what creates this recurring pattern before you can solve it effectively.

**2. D. Mental Models** – The belief "Ship fast, fix bugs later" is a mental model—a worldview that influences decisions. It leads to Structures (tightly coupled components) that create Patterns (cascading failures) that manifest as Events (bugs). Changing the architecture without changing this belief will likely create similar problems in new forms.

## What's Next

Now that you understand the iceberg model and can see systems at different levels, let's apply this to real software systems. In the next lesson, we'll explore **Systems in Software Architecture**—how every application is actually a system of systems, with multiple layers and dependencies.

This will help you see the bigger picture when designing your own applications.