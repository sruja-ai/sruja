# Architecture Debt Analyzer Engine (ADAE)

**Status**: Advanced Engine  
**Pillars**: Cross-Pillar (Debt Management)

[← Back to Engines](../README.md)

## Overview

The Architecture Debt Analyzer Engine (ADAE) quantifies, tracks, predicts, and eliminates architectural debt — structural, domain, operational, and organizational — with a unified scoring and remediation engine.

**It acts as the Credit Bureau of your system architecture.**

## Purpose

ADAE answers:

- ✅ How much architecture debt do we have?
- ✅ Where is it located?
- ✅ What caused it?
- ✅ What's the interest on this debt?
- ✅ What will this debt do to us in 6 months?
- ✅ What is the most cost-effective paydown plan?
- ✅ Which teams own which debt?
- ✅ How does debt affect business outcomes?

## Types of Architecture Debt

ADAE tracks **7 categories** of architectural debt:

### 1. Structural Debt
- cyclic dependencies
- deep call chains
- architecture "spaghetti" formation
- tightly coupled clusters
- central bottlenecks
- N+1 dependency patterns

### 2. Domain / DDD Debt
- bounded context violations
- inconsistent domain language
- domain leakage
- over-expanded contexts
- integration blobs
- incorrect team alignment

### 3. Reliability Debt
- single points of failure
- insufficient redundancy
- inconsistent retries
- non-idempotent APIs
- cascading fail paths

### 4. Performance Debt
- synchronous-by-default patterns
- slow paths buried inside modules
- unbounded fan-outs
- over-reliance on blocking calls

### 5. Security Debt
- missing auth/authorization boundaries
- direct DB access
- too many public endpoints
- unencrypted flows

### 6. Operational Debt
- high cognitive load
- high on-call overhead
- manual interventions
- inconsistent deployments

### 7. Organizational Debt
- wrong team topology
- unclear ownership
- high coordination cost
- fractured responsibility domains

## Debt Score Model

ADAE computes a unified score:

```
ArchitectureDebtScore = Σ (DebtTypeScore × Weight × InterestFactor)
```

Where:

- **Weight** = organizational importance
- **InterestFactor** = how fast this debt compounds
- **DebtTypeScore** = 0–100 for each category

Produces:

- Global Debt Score (0–1000)
- Debt Trend Indicator
- High-Interest Hotspots
- Team Debt Ownership

## Architecture

```
ArchitectureDebtAnalyzerEngine
 ├── StructuralDebtScanner
 ├── DomainDebtScanner
 ├── ReliabilityDebtScanner
 ├── PerformanceDebtScanner
 ├── SecurityDebtScanner
 ├── OperationalDebtScanner
 ├── OrgDebtScanner
 ├── DebtHistoryTracker
 ├── DebtForecastModel
 │     ├── time-series projection
 │     ├── compounding model
 │     ├── causal model (AEKG)
 ├── DebtScoringEngine
 ├── DebtHotspotMapper
 ├── AutoPaydownPlanner
 ├── LLM Explanation Engine
 ├── AEKG Sync
 └── MCP API
```

## Debt Detection Techniques

### Structural Debt Detection
- cycle detection
- graph centrality scoring
- path length scoring
- coupling density
- articulation points
- bottleneck node identification

### Domain Debt Detection
Uses Systems Thinking + DDD DSL:

- domain boundary violations
- cross-boundary chatter
- inconsistent context mappings
- domain concept mutation

### Reliability Debt Detection
Driven by:

- ATOE telemetry
- synthetic fault injection patterns
- dependency failure graphs

### Operational & Organizational Debt
- cognitive load estimation
- team ownership ambiguities
- RACI mismatch
- operational toil metrics

## Output

### Debt Report Example

```
ARCHITECTURE DEBT REPORT — March Release

GLOBAL DEBT SCORE: 614/1000 (High)
TREND: +12% MoM (increasing)
PRIORITY HOTSPOTS: Billing, Identity, DataHub

STRUCTURAL DEBT
 - 3 cycles detected (score 44)
 - 2 deep chains > 7 hops (score 31)
 - Billing service too central (betweenness 0.82)

DOMAIN DEBT
 - Bounded context violation: Subscription <-> Billing
 - Ubiquitous language mismatch: "charge" used inconsistently

RELIABILITY DEBT
 - EventStore no retries
 - BillingAggregator single point of failure

ORGANIZATIONAL DEBT
 - Team ownership unclear for LedgerService
 - Cognitive load for BillingTeam too high

FORECAST
 - Debt projected to reach 710 in 6 months
 - Impact: +19% MTTR, -14% feature velocity

RECOMMENDED PAYDOWN PLAN
 1. Split BillingAggregator (gain 36 points)
 2. Introduce retry policies in EventProcessor (gain 22)
 3. Move LedgerService ownership to Finance Team (gain 15)
```

## Paydown Plan Generator

ADAE automatically produces:

- impact-weighted strategy
- effort estimate
- quick wins
- long-term structural fixes
- sequencing plan
- team responsibility map
- cost savings estimate
- business impact mapping

## UI Modules

- ✔ Debt Heatmap (per domain / per team)
- ✔ Trend Analytics Dashboard
- ✔ Hotspot Explorer
- ✔ Compounding Forecast Graph
- ✔ Paydown Strategy Builder
- ✔ "Debt Interest" Calculator
- ✔ Architecture Score Tracker

## MCP API

```
adae.scan(model)
adae.score(model)
adae.trend()
adae.hotspots()
adae.forecast(months)
adae.paydownPlan(strategy)
adae.teamDebt(team)
adae.domainDebt(domain)
adae.explainDebt(id)
```

## Strategic Value

ADAE provides:

- ✅ Early warning system for debt accumulation
- ✅ Quantified debt measurement
- ✅ Debt forecasting
- ✅ Automated paydown planning
- ✅ Team accountability
- ✅ Business impact visibility

**This is critical for maintaining architecture health.**

## Implementation Status

✅ Architecture designed  
✅ Debt categories defined  
✅ Scoring model specified  
📋 Implementation in progress

---

*ADAE quantifies and tracks architectural debt across all dimensions.*

