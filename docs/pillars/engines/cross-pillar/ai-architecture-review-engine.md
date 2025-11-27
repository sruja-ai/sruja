# AI-Guided Architecture Review Engine

**Status**: Advanced Engine  
**Pillars**: Cross-Pillar (Operational Excellence, Security, Reliability, Performance, Cost)

[← Back to Engines](../README.md)

## Overview

The AI-Guided Architecture Review Engine performs automated, AI-powered architecture reviews, detecting anti-patterns, risks, and providing improvement recommendations.

## Purpose

This engine transforms Sruja into a **real Architecture Governance Platform**, similar to:
- AWS Well-Architected
- Google Architecture Review
- Microsoft CAF
- ThoughtWorks Tech Radar

But **fully automated, model-aware, code-aware, and AI-native**.

## Capabilities

### Automated Analysis
- ✅ Rule-based analysis (deterministic)
- ✅ AI-semantic reasoning (patterns, smells, risks)
- ✅ Cross-layer analysis (VHLD → HLD → LLD)
- ✅ Cross-file and cross-boundary checks
- ✅ Code/architecture consistency checks
- ✅ Compliance scoring
- ✅ Narrative review reports
- ✅ Change impact review
- ✅ Improvement suggestions
- ✅ Architectural anti-pattern detection

## Architecture

```
Review Engine
 ├── Static Rules Engine (deterministic)
 ├── Pattern Detector (graph analysis)
 ├── Risk Analyzer (AI-based)
 ├── Alignment Checker (requirements / ADR / code)
 ├── Review Report Generator
 └── MCP Tools (for AI assistants)
```

## Pipeline

```
GlobalModel
  ↓
Static Rules Validator (mandatory rules)
  ↓
Pattern Detection (graph + heuristics)
  ↓
AI Reasoning Layer (semantic analysis)
  ↓
Trace Consistency Checker
  ↓
Code ↔ Architecture Drift Checker
  ↓
AI-Guided Recommendations
  ↓
Final Score + Report
```

## Static Rules Engine

Deterministic checks for architectural constraints:

- ❌ UI cannot call DB directly
- ❌ External systems must pass through API Gateway
- ❌ No cross-boundary data access without contract
- ❌ No cycles allowed in services
- ❌ Microservice must have bounded context
- ❌ Event producers must have consumers
- ❌ Require retries for network calls
- ❌ No domain leaks (anti-corruption layer missing)

Rules are defined using AQL (Architecture Query Language):

```aql
FIND relations
WHERE source.kind = "ui" AND target.kind = "database"
```

If result > 0 → violation.

## Pattern Detection

### Design Pattern Detection
- ✅ Saga pattern
- ✅ CQRS pattern
- ✅ Event-driven microservices
- ✅ Gateway + Backend-for-Frontend
- ✅ Hexagonal Architecture
- ✅ Layered Monolith
- ✅ Shared Kernel

### Anti-Pattern Detection
- ❌ God Service (too many responsibilities)
- ❌ God Module (too many incoming dependencies)
- ❌ Big Ball of Mud (no boundaries; dense graph)
- ❌ Distributed Monolith (coupled across services)
- ❌ Chatty Services (many synchronous calls)
- ❌ Broken Domain Boundaries
- ❌ Data Overexposure
- ❌ Missing Idempotency
- ❌ Missing Circuit Breaker
- ❌ Spaghetti Event Graph
- ❌ Wormhole dependencies (skip layers)

### Algorithm

```typescript
graphMetrics = {
  fanIn: count inbound edges,
  fanOut: count outbound edges,
  density: |E| / |V|^2,
  boundaryCrossings: …
}
```

Threshold-based detection.

## AI Reasoning Layer

AI analyzes:
- The architecture
- Requirements
- ADRs
- Code snippets
- Boundaries
- Diagram layout
- Validation issues
- Anti-patterns
- System constraints
- NFRs
- Semantics
- Naming conventions
- Complexity

And produces:
- ✅ Findings
- ✅ Risks
- ✅ Smells
- ✅ Suggested improvements
- ✅ Architectural alternatives
- ✅ Scalability concerns
- ✅ Failure modes
- ✅ Security gaps
- ✅ Domain modeling problems
- ✅ Data flow issues

### Example AI Output

> "CheckoutService integrates directly with 3 external APIs synchronously.  
> This introduces a latency coupling and a single point of failure.  
> Suggest introducing an orchestrator or asynchronous compensation."

## Trace Consistency Checker

Uses the traceability engine:

```
Requirement → Component → Code → Tests
```

Checks:
- ❌ requirement missing implementation
- ❌ requirement implemented by multiple conflicting components
- ❌ ADR says "use event-based", architecture uses sync calls
- ❌ No test for a high-priority requirement
- ❌ Requirement violated in code (AI checks this)

## Architecture ↔ Code Drift Checker

Codegen + code scanner + LLM evaluate:
- ✅ class/method naming
- ✅ missing required modules
- ✅ architecture says "async", code is synchronous
- ✅ architecture says "DB per service", code has shared db client
- ✅ architecture says "API Gateway", code bypasses it
- ✅ microservices merging into monolith (imports/leaks)
- ✅ hexagonal ports/adapters violations

## Scoring System

Score architecture across dimensions:

| Category | Weight | Examples |
|---------|--------|----------|
| **Reliability** | 25% | retries, queues, idempotency |
| **Security** | 25% | secure boundaries, zero trust |
| **Scalability** | 20% | async events, load patterns |
| **Maintainability** | 15% | modularity, boundaries |
| **Complexity** | 10% | graph density, cycles |
| **Cost Efficiency** | 5% | overprovisioning, redundancy |

Final score: `score = sum(categoryScore * weight)`

## Review Report Structure

### Section 1 — Executive Summary
- Score
- Top 5 issues
- Top 5 recommendations

### Section 2 — Architecture Model Summary
- Contexts, domains, services, relations

### Section 3 — Automated Rule Violations
- broken constraints
- risky dependencies
- missing boundaries

### Section 4 — Anti-Patterns Detected
- distributed monolith
- chatty microservices
- domain leaks
- unbounded fan-in/out

### Section 5 — Requirements Compliance
- missing implementation
- consistency issues
- coverage report

### Section 6 — Code/Architecture Alignment
- drift
- code smells
- missing modules

### Section 7 — AI Recommendations
- domain changes
- event patterns
- scaling strategies
- resilience improvements

### Section 8 — Next Steps
- step-by-step fix plan

## MCP Tools

AI agents can use these tools:

### `review.run`
Runs complete review, returns structured results + overview

### `review.summary`
Returns high-level AI summary of current architecture

### `review.impact`
Impact of change on architecture

### `review.recommendations`
Improvement suggestions

### `review.validateRequirement`
Ensure requirement satisfied across architecture & code

### `review.checkPattern`
Detect design patterns or anti-patterns

## UI Integration

### Badge System in Diagram
- 🔴 red = critical
- 🟡 yellow = warning
- 🔵 blue = improvement
- 🟣 purple = pattern recognized

### Hover Cards
Over components showing:
- risk summary
- recommendations

### Review Panel
- full report
- grouping by domain
- ability to dismiss findings
- re-run review on demand

### AI "Explain this problem" Popup
Shows why something is a risk.

## Plugins

Allow org-specific rules:
- PCI-DSS
- HIPAA
- GDPR
- SOC2
- Internal company standards
- Banking architecture rules
- Healthcare event chains
- AWS Well-Architected
- GCP Best Practices

## Implementation Status

✅ Architecture designed  
✅ Static rules engine specified  
✅ Pattern detection algorithms defined  
📋 AI reasoning layer in progress  
📋 UI integration planned

---

*The AI-Guided Architecture Review Engine provides enterprise-grade architecture governance with AI-powered insights.*

