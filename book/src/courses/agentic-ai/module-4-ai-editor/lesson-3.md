---
title: "Lesson 3: AI Pair Programming Patterns"
weight: 3
summary: "Effective patterns for AI-assisted architecture work."
---

# Lesson 3: AI Pair Programming Patterns

## The Problem with Unstructured AI Help

When you ask AI to "add a new feature" without context:
- AI doesn't know your architecture constraints
- AI might suggest changes that break dependencies
- AI doesn't understand your team's conventions

## Architecture-Aware AI Interactions

### Pattern 1: Before Changing X, Ask About Dependencies

```
You: "What would break if I change UserService?"

AI (with Sruja context): "Changing UserService would affect:
- LoginComponent (depends on UserService)
- ProfileUI (calls UserService.getProfile())
- NotificationService (receives user events)

You should also update:
- user-auth contract (API boundary)
- test/UserService.test.ts (affected tests)
"
```

### Pattern 2: Use AI to Explore, Then Validate

```bash
# AI helps explore
"What does the checkout flow look like?"

# AI shows understanding
"Based on your architecture, checkout involves:
- CartService -> PaymentService -> OrderService

Shall I trace through each step?"
```

Then validate with Sruja:
```bash
# After AI proposes changes
sruja lint
sruja drift -r .
```

### Pattern 3: AI-Assisted Impact Analysis

```bash
# Before making a big change
"Use sruja impact to show me blast radius of removing the legacy import service"

# AI calls MCP tool
# Blast radius analysis shown
```

### Pattern 4: Ask for Sruja DSL, Then Validate

```
You: "Generate Sruja DSL for a new notification service"

AI: [generates Sruja code]

You: "Validate this with sruja lint and check it fits our patterns"
```

### Pattern 5: AI as Architecture Reviewer

```
You: "Review this code change for architectural compliance"

AI with context: [checks against policies, finds issues]
"1. This change violates your 'no-direct-db-access' policy
2. New dependency creates circular reference with BillingService
3. Missing required 'tech' field in container definition"
```

## Effective Prompts for Architecture Context

| Instead of... | Try... |
|--------------|--------|
| "Add auth to this endpoint" | "Add auth following the OAuth2 pattern in PaymentContract" |
| "Refactor this service" | "Refactor this service keeping the ADR-0012 constraints" |
| "Add caching" | "Add caching to match the fitness function latency < 100ms" |
| "Split this component" | "Split along the boundary suggested by community detection" |

## Workflow: AI-Assisted Feature Development

1. **Define the task**
   ```
   "I need to add a new payment method to checkout"
   ```

2. **AI explores context**
   ```
   AI: "Let me check your PaymentContract and checkout flow..."
   ```

3. **AI proposes with constraints**
   ```
   AI: "Based on your architecture, this requires:
   - Adding endpoint to PaymentContract
   - Updating FraudDetection rules
   - New scenario test for the payment flow"
   ```

4. **Human approves**
   ```
   "Sounds right, implement it"
   ```

5. **AI implements**

6. **Human validates with Sruja**
   ```bash
   sruja lint
   sruja validate --scenarios
   ```

## Anti-Patterns to Avoid

| Anti-Pattern | Why It's Bad | Better Approach |
|--------------|--------------|-----------------|
| "Just make it work" | Ignores architecture | "Make it work AND pass sruja lint" |
| "AI doesn't know our patterns" | Unstructured | "Follow the patterns in our repo.sruja" |
| Trust AI blindly | AI can hallucinate | Always validate with Sruja |
| Ask without context | Generic answers | Build context first with `sruja focus` |

## Hands-On: Practice AI Pair Programming

1. **Set up context for your task:**
   ```bash
   # Focus on the area you're working on
   sruja focus --file src/payment.rs -r .
   ```

2. **Ask architecture-aware questions:**
   ```
   "What does this component depend on?"
   "What would break if I change this?"
   "Show me the blast radius of this change"
   ```

3. **Validate AI suggestions:**
   ```bash
   # After AI proposes changes
   sruja lint repo.sruja
   sruja drift -r . -a repo.sruja
   ```

4. **Use impact analysis:**
   ```bash
   # Check what would be affected
   sruja impact ComponentName -r . --depth 3
   ```

## Learning Outcomes

- ✅ Apply architecture-aware AI interaction patterns
- ✅ Use AI to explore and understand architecture
- ✅ Validate AI suggestions with Sruja commands
- ✅ Avoid common anti-patterns in AI-assisted development

## Quiz: Test Your Understanding

### Q1: What should you do before making a significant change?

A) Just make the change quickly
B) Ask about dependencies and use impact analysis
C) Delete the old code first
D) Ignore architecture concerns

### Q2: Why should you validate AI suggestions with Sruja?

A) Because AI is always wrong
B) Because AI can hallucinate and make mistakes
C) To make the code faster
D) It's not necessary

### Q3: What command shows the blast radius of changing a component?

A) `sruja lint`
B) `sruja context`
C) `sruja impact`
D) `sruja drift`

## Module Complete!

You've completed the AI Editor Integration module. You now understand:
- ✅ Setting up Sruja MCP server
- ✅ Building architecture context for AI
- ✅ Using context-driven AI assistance
- ✅ Effective AI pair programming patterns

This module completes the Agentic AI course with practical AI editor integration skills.