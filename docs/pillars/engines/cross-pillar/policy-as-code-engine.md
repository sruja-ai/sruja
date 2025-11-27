# Policy-as-Code Engine

**Status**: Advanced Engine  
**Pillars**: Cross-Pillar (Security, Governance, Operational Excellence)

[← Back to Engines](../README.md)

## Overview

The Policy-as-Code Engine enforces architectural constraints **automatically**, turning your platform into a **governance enforcement system** similar to OPA, HashiCorp Sentinel, AWS Config Rules, but built specifically for software architecture.

## Purpose

This engine enables:
- ✅ Enforce architectural best practices
- ✅ Block violating designs
- ✅ Approve system design automatically
- ✅ Validate microservice boundaries
- ✅ Enforce layered architecture
- ✅ Ensure domain isolation
- ✅ Detect illegal dependencies
- ✅ Ensure ADR alignment
- ✅ AI-based semantic rule checks
- ✅ Prevent distributed monoliths
- ✅ Detect shared DBs
- ✅ Validate event contracts

## Architecture

```
Policy Engine
 ├── Rule Loader (local + remote plugins + registry)
 ├── Rule Parser (Zod + DSL-based)
 ├── AQL Executor
 ├── Graph Rule Executor
 ├── Code Rule Executor
 ├── Domain Rule Executor
 ├── AI-Semantic Rule Layer
 ├── Auto-Fix Engine
 ├── Git/PR Integration
 └── Report Generator + Severity Scorer
```

## Rule Types Supported

### 1. AQL-based Rules (exact search)

Example rule file:

```yaml
id: no-ui-db-direct
severity: error
aql: |
  FIND relations WHERE source.kind="ui" AND target.kind="database"
message: "UI must not call DB directly—use API Gateway."
```

### 2. Graph Structural Rules

Example:

```yaml
id: no-cycles
severity: error
graph:
  forbidCycles: true
message: "Cycles detected in service graph."
```

### 3. Domain Boundary Rules (DDD)

Example:

```yaml
id: bounded-context-isolation
severity: error
boundaries:
  forbid:
    - from: Payments
      to: Accounts
      unless: ["DomainEvents"]
```

### 4. Dependency Rules

```yaml
id: layer-violation
severity: error
layers:
  - name: UI
    canDependOn: [API]
  - name: API
    canDependOn: [Service]
  - name: Service
    canDependOn: [Data]
```

### 5. Code Consistency Rules

```yaml
id: architecture-match-ports
severity: warning
code:
  checkInterfaces: true
  requireAdaptersForPorts: true
```

### 6. AI-Semantic Rules (Natural Language)

```yaml
id: secure-boundary
severity: warning
ai:
  check: "Ensure no sensitive data flows across public boundaries without encryption."
```

This uses:
- extracted data flows
- context + LLM reasoning

### 7. ADR Alignment

```yaml
id: enforce-adr
severity: error
adr:
  mustAlignWith:
    - "Use event-driven architecture for async operations"
```

Engine checks:
- ADR statement
- DSL
- Graph

### 8. Custom Plugin Rules

Plugins can define:
- AQL rules
- Graph conditions
- New DSL validations
- Custom severity scoring
- Auto-fix functions

## Policy Evaluation Pipeline

```
Load policies → 
Normalize → 
Run exact static rules → 
Run graph rules → 
Run domain/boundary rules → 
Run code consistency → 
Run AI semantic rules → 
Aggregate → 
Score → 
Fix suggestions → 
Report → 
Block/Allow
```

Runs on:
- UI Save
- Commit
- Push
- PR
- Release

## Automated Fixes (Auto-Fix Engine)

For rules with:

```yaml
fix:
  type: dsl-patch
```

Example auto-fix:

### Input
Service has direct DB dependency.

### Auto-Fix
Insert API Gateway:

```
- Add API Gateway component
- Route UI → Gateway → Service
- Remove direct UI → DB edge
```

Engine outputs a DSL patch:

```
REMOVE RELATION ui -> db
ADD COMPONENT api_gateway
ADD RELATION ui -> api_gateway
ADD RELATION api_gateway -> service
```

## Integration with Change Simulation Engine

Before applying a fix:

- simulate performance impact
- simulate domain effects
- simulate failure propagation

If results negative → mark fix as "unsafe".

## MCP Tools for Agents

### `policy.evaluate`
Runs entire policy set.

### `policy.listViolations`
List all rule violations.

### `policy.fix`
Return auto-fix patches.

### `policy.explain`
LLM explains why it's a violation.

### `policy.suggest`
LLM proposes architecture improvements.

### `policy.generateRule`
AI creates rule based on natural-language request.

Example:
> "Create a rule ensuring no service calls more than 3 downstream services."

Generates:

```yaml
graph:
  maxFanOut: 3
```

## UI Integration

### Rule Violation Badges
Diagram nodes show icons:
- 🔴 red (errors)
- 🟡 yellow (warnings)
- 🔵 blue (improvements)

### Policy Panel
Three tabs:
1. Violations
2. Rules
3. Fixes

### Auto-Fix Preview (Diff Viewer)
Shows proposed DSL changes visually + as code diff.

## Plugin System for Enterprise Rulesets

Provide built-in rule libraries:

### Security Ruleset
- encryption boundaries
- secret isolation
- public/internet exposure

### FinTech Ruleset
- PCI compliance
- bank boundary isolation
- money movement boundaries

### Health Ruleset
- HIPAA
- PHI flow rules

### DDD Ruleset
- aggregate referencing
- bounded context isolation
- domain event flow

### Cloud Architecture Ruleset
- AWS / GCP / Azure
- resiliency patterns
- multi-region design

Each is a plugin.

## Example Rules

### Rule: "No distributed monolith"

```yaml
id: distributed-monolith
severity: error
graph:
  maxSyncChain: 3
  maxFanIn: 10
  maxFanOut: 15
message: "Service graph shows distributed monolith pattern."
```

### Rule: "Event-driven async operations"

```yaml
id: async-for-long-ops
severity: warning
aql: |
  FIND relations 
  WHERE type="sync" AND latency > 100ms
ai:
  suggest: "Consider async/event-driven pattern for long operations"
```

## Implementation Status

✅ Architecture designed  
✅ Rule types specified  
✅ Auto-fix engine designed  
📋 Plugin system in progress  
📋 Git/PR integration planned

---

*The Policy-as-Code Engine enables automated architectural governance at scale.*

