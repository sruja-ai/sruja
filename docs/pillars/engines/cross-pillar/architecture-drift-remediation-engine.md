# Architecture Drift Auto-Remediation Engine (ADARE)

**Status**: Advanced Engine  
**Pillars**: Cross-Pillar (Operational Excellence, Reliability, Governance)

[← Back to Engines](../README.md)

## Overview

The Architecture Drift Auto-Remediation Engine (ADARE) automatically detects, diagnoses, and **fixes** architecture drift — generating patches, DSL updates, refactor plans, and enforcement actions across the entire ecosystem.

## Purpose

ADARE continuously:

- ✅ Detects drift
- ✅ Identifies root cause
- ✅ Proposes fixes
- ✅ Auto-generates architecture patches
- ✅ Auto-generates code scaffolding (MCP → IDE)
- ✅ Enforces rules using SSAGE
- ✅ Fixes broken boundaries
- ✅ Corrects dependency violations
- ✅ Patches the DSL
- ✅ Repairs diagrams
- ✅ Helps teams refactor safely

**It is the healing system of the architecture platform.**

## Types of Drift ADARE Fixes

### 1. Runtime Dependency Drift Fix

When real runtime traffic shows dependencies not in architecture DSL:

| Drift | Fix |
|------|------|
| Unknown dependency appears | Add to DSL or flag governance violation |
| Service calling wrong domain | Replace with correct gateway/event |
| DB queries bypassing API | Insert missing service / DTO boundary |

### 2. Domain Boundary Drift Fix

E.g., component from Payments interacts with Inventory.

**Fixes:**
- Suggest domain reshaping
- Move components between domains
- Introduce domain events
- Split or merge bounded contexts

### 3. Governance/Policy Drift Fix

If SSAGE violation detected:

- Auto-generate "governance fix patch"
- Update API to approved protocol
- Modify data flow to secure zone
- Add circuit breaker or retry policy

### 4. Performance Drift Auto-Tuning

If runtime latency increases:

- Recommend caching
- Suggest async messaging
- Identify bottleneck in chain
- Propose relocation (microservice → function)

### 5. Resilience Drift Fix

When failure propagation increases:

- Auto-generate resilient pattern:
  - retry
  - backoff
  - idempotency
  - fallback
  - circuit breaker
- Suggest service mesh config

### 6. Data Flow Drift Fix

If PII flows outside secure zone:

- Mask PII at source
- Insert anonymization layer
- Update DSL + data lineage

### 7. Architecture Model Drift Fix

If DSL diverges from diagrams:

- Update DSL text
- Update diagram layout
- Update IR for consistency

### 8. Team/Ownership Drift

E.g., system ownership changed but DSL not updated:

**Fix:**
- Update team mapping
- Suggest transfer review
- Update bounded context docs

## Architecture

```
ADARE (Auto-Remediation Engine)
 ├── DriftCollector
 │    ├── From ATOE (runtime)
 │    ├── From SSAGE (violations)
 │    ├── From IR (model drift)
 │    ├── From Git (changes)
 ├── DriftClassifier
 ├── RootCauseAnalyzer
 ├── FixSuggestionGenerator
 │    ├── Architecture Patches
 │    ├── DSL Fixes
 │    ├── Diagram Fixes
 │    ├── Domain Restructuring
 │    ├── Code Scaffolding (via MCP)
 ├── PatchBuilder
 │    ├── DSL Patch
 │    ├── Graph Patch
 │    ├── IR Diff Patch
 │    ├── Code Template Patch
 ├── RemediationStrategySelector
 ├── AutoApplyEngine
 │    ├── Safe Apply
 │    ├── Dry Run
 │    ├── Manual Review Required
 ├── DriftReportGenerator
 ├── ACH Notifications
 ├── MCP Interface
```

## Types of Remediation Output

### 1. DSL Patch

Example:

```sruja
patch {
  add Connection from OrderService to InventoryService type "async-event"
  remove DirectCall from OrderService to InventoryDB
}
```

### 2. Graph Patch

For ReactFlow/diagram updates:

```sruja
move node PaymentGateway -> Domain:Payments
delete edge Catalog -> AdService
```

### 3. IR Patch

Internal representation changes.

### 4. Governance Fix Suggestion

Auto-repairs violations:

- enforce allowed protocols
- add monitoring
- fix domain purity issues

### 5. Code Generation Patch (MCP → IDE)

E.g.:

- create a new adapter layer
- generate event producer code
- generate gateway wrappers
- generate DTOs
- generate service boundaries

### 6. Auto-Review Message (ACH)

Example:

```
Drift fixed: OrderService → PaymentService dependency now documented.
```

### 7. Architecture Review Result

Shows:

- what drift
- why drift
- root cause
- how fixed
- impact before/after

## Remediation Modes

### 🟢 Mode 1 — Auto-Apply
ADARE auto-fixes drift continuously.  
(only for low-risk fixes)

### 🟡 Mode 2 — Semi-Automatic
ADARE suggests patches → Architect reviews → Applies.

### 🟠 Mode 3 — Manual Guided Fix
ADARE provides list of recommendations & code patches.

### 🔴 Mode 4 — Simulation-Guided
Before applying, MAES simulates consequences.

## Drift Classification Engine

ADARE classifies drift into:

- structural
- domain
- governance
- resilience
- data
- performance
- runtime-only dependencies
- team ownership
- missing documentation
- violating abstractions
- violating layering

This uses the **global knowledge graph** and runtime signals.

## Root Cause Analysis (RCA)

ADARE identifies WHY drift occurred:

Examples:

- "Team introduced new dependency during a refactor."
- "Fallback handler created new traffic path."
- "Database schema change created new flows."
- "Circuit breaker misconfiguration created short-circuit dependency."
- "Developer bypassed API gateway."

RCA gives clarity and accountability.

## MCP API

```
adare.detect()
adare.classify()
adare.suggestions()
adare.apply(patch)
adare.patchDSL()
adare.patchCode()
adare.patchGraph()
adare.rootCause()
adare.simulateBeforeAfter()
adare.reviewDraft()
```

## UI Features

### Drift Heatmap
Color-coded drift zones.

### Suggested Fix Panel
Shows patches with apply button.

### RCA Explorer
Tree of root causes.

### Code Patch Preview (MCP → IDE)
Developers can accept/reject fixes.

### Drift Timeline
How drift evolves over time.

### Governance Fix Tasklist
Automatically creates tasks.

## Strategic Impact

ADARE:

- ✅ Keeps architecture healthy automatically
- ✅ Removes manual architecture review bottlenecks
- ✅ Enables safe continuous delivery
- ✅ Empowers teams to remain autonomous
- ✅ Prevents architectural decay
- ✅ Fixes tech debt before it becomes debt
- ✅ Is a **unique differentiator** in the market
- ✅ Closes the Architecture Feedback Loop (design → runtime → repair → design)

**This module is how you build self-healing architecture.**

## Implementation Status

✅ Architecture designed  
✅ Drift types defined  
✅ Remediation modes specified  
📋 Implementation in progress

---

*ADARE transforms architecture from static diagrams into self-healing systems.*


