# Human Writing Style Guide
**Systems Thinking 101 Course**

---

## Philosophy

Great technical writing should feel like a conversation with a knowledgeable mentor—clear, engaging, and human. This guide helps you avoid AI-generated patterns and create content that resonates with learners.

---

## Core Principles

### 1. **Variety Over Repetition**
AI uses identical patterns. Humans vary their approach.
- Vary headings, structure, and tone
- Mix prose, examples, and bullets naturally
- Adapt depth to the topic's importance

### 2. **Conversation Over Lecture**
Talk *with* learners, not *at* them.
- Use rhetorical questions
- Add transitional phrases
- Write in second person ("you")
- Include personal touches when appropriate

### 3. **Examples First, Theory Second**
Show before tell.
- Start with relatable examples
- Then explain the concept
- Use everyday analogies

### 4. **Natural Flow Over Rigid Structure**
Good writing breathes. It's not perfectly symmetrical.
- Some sections are longer, some shorter
- Paragraphs mixed with lists
- Conversational transitions between topics

---

## DO vs. DON'T

### Headings

❌ **DON'T: Use identical "What is/What Are" patterns**
```markdown
## What Are Parts?
## What Are Boundaries?
## What Are Flows?
## What Are Feedback Loops?
```

✅ **DO: Vary heading styles**
```markdown
## Understanding Parts
## Boundaries in Practice
## How Flows Work
## Feedback Loops in Action
## Getting Started with Context
```

---

### Learning Goals

❌ **DON'T: Use singular "Learning Goal"**
```markdown
## Learning Goal
Understand the basic concept of systems thinking.
```

✅ **DO: Use plural "Learning Goals"**
```markdown
## Learning Goals

By the end of this lesson, you'll be able to:

- Define what systems thinking is
- Recognize systems thinking in everyday life
- Apply systems thinking to software architecture
```

---

### Opening Sections

❌ **DON'T: Start with rigid definitions**
```markdown
## What is Systems Thinking?

Systems thinking is a holistic approach to understanding how components interact as part of a whole. Instead of looking at parts in isolation, it focuses on relationships, patterns, and emergent behaviors.

Key characteristics:
- Holistic view
- Focus on relationships
- Emergent behaviors
```

✅ **DO: Start with a hook or question**
```markdown
## Understanding Systems Thinking

Have you ever fixed one bug only to have three more pop up? Or optimized a database query only to see no performance improvement?

These problems aren't just bad luck—they're symptoms of thinking about parts in isolation. Systems thinking offers a different approach: look at the whole, not just the pieces.

### The Core Idea

Systems thinking is about understanding how things connect. It's less about "what are the parts?" and more about "how do parts work together?"
```

---

### Section Transitions

❌ **DON'T: Jump abruptly between sections**
```markdown
## Why Boundaries Matter

### 1. Clear Ownership

Who's responsible for what?

## Types of Boundaries

### 1. System Boundary
```

✅ **DO: Add conversational transitions**
```markdown
## Why Boundaries Matter

Boundaries aren't just lines on a diagram—they're about real-world responsibilities. Let's look at why they matter in practice.

### 1. Clear Ownership

Who's responsible for what? This question might seem simple, but the answer shapes your architecture decisions.

---

Now that we understand why boundaries matter, let's explore the different types you'll encounter in real systems.

## Types of Boundaries
```

---

### Using Bullet Points

❌ **DON'T: Over-rely on lists for everything**
```markdown
## Systems thinking focuses on:

- Relationships over parts
- Patterns over events
- Context over isolation
- Flows over structure

These principles help you:

- Understand emergent behavior
- Design resilient systems
- Avoid local optimization
```

✅ **DO: Use paragraphs mixed with lists**
```markdown
## The Core Principles

Systems thinking shifts your focus from individual parts to the connections between them. Think of it like this: a single cog in a clock isn't very interesting, but when it meshes with other cogs, something useful emerges.

The key principles are:

- **Relationships over parts** – How components interact matters more than what they are
- **Patterns over events** – Look for trends, not just incidents
- **Context over isolation** – Systems exist in an environment

This perspective changes how you design. Instead of optimizing components in isolation, you consider how changes ripple through the whole system.
```

---

### Code Blocks

❌ **DON'T: Use code blocks for simple lists**
```markdown
## Types of Systems

```sruja
// People
Customer = person "Customer"
Admin = person "Admin"

// Systems
App = system "Application"
DB = system "Database"
```
```

✅ **DO: Use code blocks only when showing actual syntax**
```markdown
## Types of Systems

When modeling with Sruja, you'll typically work with:

- **People**: Users, stakeholders, administrators
- **Systems**: Applications, external services, platforms
- **Containers**: Databases, APIs, web servers

Here's how you define these in Sruja:

```sruja
Customer = person "Customer"
App = system "Application"
DB = database "PostgreSQL"
```
```

---

### Examples

❌ **DON'T: Use generic, abstract examples**
```markdown
## Example: System Interaction

System A interacts with System B, which then sends data to System C.

System A -> System B "Sends request"
System B -> System C "Processes data"
System C -> System A "Returns response"
```

✅ **DO: Use concrete, relatable examples**
```markdown
## Real-World Example: Coffee Shop

Let's start with something you've probably experienced—buying coffee. A coffee shop is a system with multiple parts:

```
Customer orders → Barista makes coffee → Customer receives coffee → Customer might return
```

Sounds simple, right? But look what happens when we consider the connections:

- The coffee machine needs beans. What if they run out? → **Supply chain dependency**
- The barista needs training. What if it's their first day? → **Human system variable**
- The shop needs to be busy enough to stay open. Too slow? → **Economic feedback loop**

**Emergent behavior**: Wait times fluctuate based on peak hours, staffing, and how many customers return. You can't predict this by looking at the parts alone.
```

---

### "Key Takeaways" Sections

❌ **DON'T: Use identical, formulaic takeaways**
```markdown
## Key Takeaway
Parts define structure. Relationships define behavior. Label relationships with protocols and actions to make diagrams actionable.
```

✅ **DO: Make takeaways varied and optional**
```markdown
## What to Remember

The distinction between parts and relationships might seem subtle, but it's crucial. Parts tell you *what* exists; relationships tell you *how* it works together.

A simple rule of thumb: if you can't describe how two components interact, you don't really understand the system yet.
```

OR simply don't include this section at all if the main content covers it well.

---

### Quizzes

❌ **DON'T: Use identical quiz structure for every question**
```markdown
**Question 1:** What is systems thinking?

- [ ] a) Option 1
- [ ] b) Option 2
- [ ] c) Option 3
- [ ] d) Option 4

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Explanation:</strong> [Explanation]
  </div>
</div>

**Question 2:** [Same structure again]
```

✅ **DO: Vary question types and formats**
```markdown
## Check Your Understanding

### Quick Questions

**1. Which perspective focuses on relationships over parts?**

[ ] A. Reductionist thinking
[ ] B. Systems thinking
[ ] C. Object-oriented design
[ ] D. Agile methodology

**2. In the coffee shop example, what represents a "relationship"?**

[ ] A. The coffee machine
[ ] B. The barista
[ ] C. "Customer orders → Barista makes coffee"
[ ] D. The beans

### Think About It

**3. Consider your current project. Can you identify one part that's been optimized in isolation? How might that optimization be affecting the whole system?**

(There's no single right answer—this is about building intuition!)

---

### Answers

**1. B. Systems thinking** – This perspective focuses on how components interact, not just what they are.

**2. C. The arrow showing the flow** – The machine and barista are parts. The "orders → makes" connection is the relationship.

**3. (Your answer)** – No right or wrong here. The point is to start seeing connections.
```

---

### Best Practices Sections

❌ **DON'T: Use rigid ✅/❌ format**
```markdown
## Best Practices

✅ Good: Label relationships with protocols
❌ Bad: Use vague labels like "uses"

✅ Good: Show external dependencies
❌ Bad: Assume everything is internal
```

✅ **DO: Write natural guidance**
```markdown
## Practical Tips

Here are some patterns that work well in practice:

**Label relationships specifically**
Instead of `API → DB "uses"`, try `API → DB "PostgreSQL/Reads"`. This small change makes your diagrams more actionable—developers can immediately understand what's happening.

**Mark external systems clearly**
When you depend on something you don't control, make that visible. Use metadata tags or visual distinctions. This helps everyone understand risks and dependencies.

**Keep it simple**
Not every diagram needs every detail. Match the level of detail to your audience. A high-level architecture diagram for executives looks different from one for developers.
```

---

## Templates

### Lesson Structure Template

```markdown
---
title: "Lesson X: [Topic]"
weight: X
summary: "[Brief, engaging summary]"
time: "2 minutes"
---

# Lesson X: [Engaging Title]

## Learning Goals

By the end of this lesson, you'll be able to:

- [Goal 1 - action-oriented]
- [Goal 2]
- [Goal 3]

## [Varied Heading - e.g., Understanding [Topic]]

[Conversational opening - maybe a question or relatable scenario]

[Definition/example in paragraph form, not just bullets]

[Optional: Visual diagram or code example]

## [Another Section - varied heading based on topic]

[Conversational transition from previous section]

[Content with natural mix of paragraphs and bullets]

## Practical Example

[Concrete, real-world example with Sruja code]

## [Optional: Tips/Patterns/Anti-patterns]

[Guidance in natural language, not rigid lists]

## [Optional: Exercise or Reflection]

[Something to help learners apply the concept]

## [Optional: Check Your Understanding]

[Varied quiz format - not identical to every lesson]
```

---

### Heading Variations

Use these instead of always "What is/What Are [Topic]?"

**For introducing concepts:**
- "Understanding [Topic]"
- "[Topic] in Practice"
- "Getting Started with [Topic]"
- "How [Topic] Works"
- "[Topic]: The Basics"

**For diving deeper:**
- "[Topic] in Action"
- "Putting [Topic] to Work"
- "When to Use [Topic]"
- "Common [Topic] Patterns"

**For examples:**
- "Real-World Example"
- "[Topic] in Everyday Life"
- "A Concrete Example"
- "[Topic] at Work"

---

### Transition Phrases

Use these to make sections flow naturally:

**Between sections:**
- "Now that we've covered [X], let's look at [Y]"
- "This raises an important question: [Y]"
- "Here's where [X] gets interesting:"
- "Let's see this in practice with an example"
- "So far, we've focused on [X]. Now, consider [Y]"

**Before examples:**
- "To make this concrete, let's look at..."
- "Here's a real-world example:"
- "Consider this scenario:"
- "Let's apply this to something you've probably seen:"

**After examples:**
- "This example shows..."
- "What's happening here?"
- "Notice how..."
- "The key insight is..."

**Before activities:**
- "Let's try this:"
- "Here's something to think about:"
- "Time to apply what you've learned:"
- "Your turn:"

---

## Before & After Examples

### Example 1: Opening a Lesson

❌ **Before (AI-style):**
```markdown
## What is Systems Thinking?

Systems thinking is a holistic approach to understanding how components interact as part of a whole. Instead of looking at parts in isolation, it focuses on relationships, patterns, and emergent behaviors that arise when components work together.

Traditional architecture often takes a reductionist approach: break systems into parts, understand each part, then put them together. But this misses the magic—the interactions that emerge only when parts work together.
```

✅ **After (Human-style):**
```markdown
## Understanding Systems Thinking

Have you ever fixed a bug, tested it thoroughly, and celebrated—only to have three new bugs appear the next day? Or optimized a database query to perfection, only to see no improvement in response times?

These aren't just frustrating coincidences. They're symptoms of thinking about systems the wrong way—focusing on parts in isolation rather than how everything connects.

**Systems thinking** offers a different approach. Instead of asking "what are the components?", it asks "how do components work together?" It's about understanding the whole system, not just its parts.

This might sound abstract, so let's make it concrete with an example.
```

---

### Example 2: Explaining a Concept

❌ **Before (AI-style):**
```markdown
## What Are Flows?

Flows represent how information, data, and actions move through a system from one component to another.

### Why Flows Matter

Flows show data lineage, process sequences, bottlenecks, and error paths.

### Types of Flows

**Data Flows:** Information movement (API → Database)
**Control Flows:** Operation sequences and decisions
**Event Flows:** Messages and notifications
**User Flows:** Complete user journeys
```

✅ **After (Human-style):**
```markdown
## How Flows Work

So far, we've looked at systems as snapshots—showing what parts exist and how they're connected. But real systems aren't static. Data moves, actions trigger other actions, and users follow paths through your application.

**Flows** capture this movement. They show you not just what's connected, but how things flow through the system over time.

### Why This Matters

Understanding flows helps you answer questions like:

- Where does this data actually come from?
- What happens when a user clicks this button?
- Where are the bottlenecks in this process?
- What if this step fails—what's the backup plan?

These are the questions that matter when you're debugging, designing, or trying to improve a system.

### Common Flow Patterns

You'll encounter several types of flows in practice:

- **Data flows**: How information moves from one place to another (e.g., "User submits form → API validates → Database stores")
- **Control flows**: The sequence of operations and decisions (e.g., "If payment succeeds, send confirmation. If it fails, show error")
- **Event flows**: How messages and notifications propagate (e.g., "Order created → Payment processed → Email sent")
- **User flows**: The complete journey a person takes through your system

Each type of flow reveals different aspects of how your system behaves.
```

---

### Example 3: Presenting Best Practices

❌ **Before (AI-style):**
```markdown
## Best Practices

✅ Good: Label relationships with protocols and actions
❌ Bad: Use vague labels like "uses" or "connects to"

✅ Good: Show external dependencies explicitly
❌ Bad: Assume everything is internal

✅ Good: Match detail level to audience
❌ Bad: Show every implementation detail

✅ Good: Use consistent naming
❌ Bad: Mix naming conventions
```

✅ **After (Human-style):**
```markdown
## Practical Tips for Better Diagrams

After working with many teams, I've noticed a few patterns that consistently help create clearer, more useful diagrams.

**Be specific with relationships**
Instead of writing `API → DB "uses"`, try something like `API → DB "PostgreSQL/Reads"`. The difference might seem small, but it's huge in practice. Specific labels mean developers can look at your diagram and immediately understand what's actually happening.

**Make external systems visible**
When you depend on something outside your control, make that clear. Use tags, colors, or annotations. This isn't just for documentation—it helps everyone understand risk and plan accordingly. If your system depends on a payment gateway and that gateway goes down, everyone needs to know that's a single point of failure.

**Keep your audience in mind**
The right level of detail depends on who you're talking to. Executives need to understand the big picture—systems, major dependencies, and data flows. Developers need to know containers, APIs, and databases. Don't try to put everything in one diagram.

**Use consistent naming**
It sounds obvious, but I see this all the time: "API Service" in one place, "API" in another, "backend API" somewhere else. Pick a convention and stick to it. Small inconsistencies add up and create confusion.

These aren't hard-and-fast rules, but following them will make your diagrams more useful to everyone who reads them.
```

---

### Example 4: Quiz Questions

❌ **Before (AI-style):**
```markdown
**Question 1:** What is the primary purpose of modeling flows?

- [ ] a) Optimize database performance
- [ ] b) Visualize data and action movement
- [ ] c) Reduce components
- [ ] d) Generate code

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Explanation:</strong> Visualizes data lineage, bottlenecks, dependencies, and error paths
  </div>
</div>

**Question 2:** After customer submits an order, what happens next?
```

✅ **After (Human-style):**
```markdown
## Check Your Understanding

Let's see if these concepts are clicking.

### Quick Check

**1. You're debugging a slow checkout process. Should you start by looking at individual components or tracing the flow?**

[ ] A. Individual components—that's where the problem is
[ ] B. Tracing the flow—you need to understand the whole path
[ ] C. Both equally
[ ] D. It depends

**2. Which of these is a *flow* (not just a relationship)?**

[ ] A. "WebApp → API: uses"
[ ] B. "User clicks checkout → WebApp sends data → API processes → Database saves"
[ ] C. "System depends on Payment Gateway"
[ ] D. All of the above

### Think About It

**3. Think about your current project. Can you identify one flow that's critical to understanding the system? Maybe a user registration flow, or a payment processing flow? What steps are involved?**

Take a moment to sketch it out mentally (or on paper). Don't worry about perfect syntax—just capture the sequence.

---

### Answers & Discussion

**1. B. Tracing the flow** – Individual components might look fine, but the slowness could be in how they interact—network latency, database contention, or some other issue that only appears when you look at the whole path.

**2. B. The sequence of steps** – The other options are static relationships showing connections. Only B shows the actual movement and sequence of actions.

**3. (Your flow)** – There's no wrong answer here! The important part is practicing the skill of seeing flows. If you're not sure where to start, pick something a user does (sign up, buy something, search) and trace it through your system.
```

---

## Quick Reference Checklist

Before publishing or reviewing a lesson, ask yourself:

### Structure
- [ ] Headings vary (not all "What is/What Are")
- [ ] Section lengths vary naturally (not perfectly symmetric)
- [ ] Has conversational transitions between major sections
- [ ] Mixes paragraphs with bullet points appropriately

### Content
- [ ] Starts with a hook (question, scenario, or relatable example)
- [ ] Uses concrete, real-world examples
- [ ] Explains *why* something matters, not just *what* it is
- [ ] Avoids over-structuring simple explanations in code blocks

### Voice
- [ ] Uses second person ("you") consistently
- [ ] Asks rhetorical questions
- [ ] Has conversational tone (not academic or overly formal)
- [ ] Avoids AI-typical phrases like "In conclusion," "Furthermore," "It's important to note"

### Quizzes (if included)
- [ ] Question formats vary (not identical for every question)
- [ ] Includes some open-ended or reflection questions
- [ ] Explanations feel natural, not robotic
- [ ] Answers provide context, not just the letter

### Overall
- [ ] Would a human mentor speak this way?
- [ ] Does it feel like a conversation, not a lecture?
- [ ] Is there variety in structure and tone?
- [ ] Would I want to learn from this?

---

## Remember

The goal isn't to be perfect—it's to be human. Some variation is good. Your voice matters. Write like you're teaching a colleague over coffee.

Happy writing! 📝