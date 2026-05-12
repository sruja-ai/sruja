---
title: "Lesson 2: Critique Engine & Adversarial Review"
weight: 2
summary: "Use the AI critique engine to challenge architecture decisions."
---

# Lesson 2: Critique Engine & Adversarial Review

## The Problem: We Miss Our Own Blind Spots

When we design architecture, we:
- Make assumptions without realizing it
- Miss risks we don't know to look for
- Accept trade-offs without questioning them
- Repeat patterns that worked before but may not fit now

## Solution: The "Angry Agent" Critique Engine

From v0.34.0, Sruja includes an adversarial critique agent—sometimes called the "Angry Agent"—that:
- Challenges your assumptions
- Identifies risks you didn't consider
- Questions trade-offs you accepted
- Finds gaps in your reasoning

## Running Critique

```bash
# Critique current architecture
sruja critique -r .

# Critique specific intent
sruja critique -r . --intent "Low Latency"

# Critique proposed changes
sruja critique -r . --diff

# Verbose output
sruja critique -r . --verbose
```

## Understanding Critique Output

The critique engine produces a report with:

| Finding | Severity | Description |
|---------|----------|-------------|
| **Assumption Challenged** | High | "You assume X but..." |
| **Risk Identified** | High | "If Y happens, Z will break" |
| **Trade-off Questioned** | Medium | "You chose A over B, but..." |
| **Gap Found** | Medium | "Missing consideration: W" |

## Example Critique

```json
{
  "critique": {
    "target": "Payment Service Architecture",
    "findings": [
      {
        "type": "assumption_challenged",
        "severity": "high",
        "title": "Assumption: Payment retries are safe",
        "body": "You assume idempotent retries won't cause issues. But what if the downstream payment processor is not idempotent? This could result in double-charges.",
        "evidence": "PaymentProcessor.endpoint does not have idempotency key"
      },
      {
        "type": "risk_identified",
        "severity": "high",
        "title": "Risk: Database single point of failure",
        "body": "Your architecture shows a single database for the payment service. If the DB goes down, no payments can be processed.",
        "mitigation": "Consider read replicas or multi-region setup"
      },
      {
        "type": "tradeoff_questioned",
        "severity": "medium",
        "title": "Trade-off: Synchronous vs Async",
        "body": "You chose synchronous processing for payments. This gives immediate feedback but limits throughput. Have you considered async for non-critical paths?",
        "options": ["Hybrid approach", "Event-driven fallback"]
      }
    ]
  }
}
```

## Responding to Critique

For each finding, you can:
- **Accept**: Update architecture to address
- **Reject**: Provide rationale for keeping current approach
- **Defer**: Add to backlog for later consideration
- **Clarify**: Update documentation

```bash
# Accept a finding
sruja critique -r . --accept "database single point of failure"

# Add rationale for rejection
sruja critique -r . --reject "sync vs async" --rationale "Async not suitable for payment confirmation UX"

# Defer
sruja critique -r . --defer "tech stack risk" --ticket "ARCH-123"
```

## Integrating into Workflow

```bash
# Part of code review
git pull-request-create --critique

# Pre-commit check
sruja critique -r . --fail-on-high
```

## Next Steps

Lesson 3 covers mapping evidence from code to intent to prove compliance.