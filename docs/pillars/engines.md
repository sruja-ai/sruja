# Engines by Well-Architected Pillars

This document organizes all Sruja engines by the six Well-Architected Framework pillars (general software architecture principles), distinguishing between **core** (basic) and **advanced** (extension) engines.

**Note**: Sruja is a **general-purpose software architecture tool** - the Well-Architected Framework is used purely as an organizational structure. Engines work for any software system, not just cloud or AWS-specific architectures.

[← Back to Pillars Index](./README.md)

## ⚠️ Important: You Don't Need All Engines!

**171 engines sounds overwhelming, but you only need a few to get started.**

### 🚀 Quick Start
- **Minimum:** 4 core engines (DSL Parser, Model Composer, Validation, Diagram Generator)
- **Typical:** 10-15 engines (core + basic pillar engines you need)
- **Enterprise:** 30-50 engines (add advanced features as needed)

**Most users never need more than 20 engines.**

👉 **See [Getting Started Guide](../GETTING-STARTED.md) for a clear path.**

---

## Overview

Engines are categorized as:
- **Core** - Basic engines included in the core DSL (minimal but solid)
- **Advanced** - Extension engines for enterprise/advanced use cases

---

## Core Engines (Basic - All Pillars)

These engines provide foundational support across all pillars:

### Architecture Modeling Engines
1. **DSL Parser Engine** - Parses DSL → AST
2. **Global Model Composer** - AST → IR → Unified Model
3. **Reference Resolution Engine** - Cross-file, cross-repo linking
4. **Two-Way Sync Engine** - Diagram ↔ DSL synchronization
5. **Diagram Serializer** - Diff-based DSL regeneration
6. **Undo/Redo Engine** - Unified across DSL & diagram
7. **Auto-layout Engine (ELK)** - Constraint + flow + grouping
8. **Visual Diff Engine** - Structure diff + layout diff
9. **Template Engine** - Microservices, event-driven templates
10. **Architecture Wizard** - Guided creation flows

### Basic Validation
11. **Validation Engine** - 12+ built-in rules (basic checks)
12. **Visual Validation Overlays** - Errors inline on diagram

### Basic Lifecycle
13. **Evolution Timeline Tracker** - Architecture evolution timeline
14. **Drift Detector** - Detect architecture → code → infra drift

---

## Operational Excellence Pillar

### Advanced Engines
- [Architecture-Time Observability Engine (ATOE)](./engines/operational-excellence/architecture-observability-engine.md) - Runtime-to-architecture feedback loop
- [Architecture Runtime Conformance Engine (ARCE)](./engines/operational-excellence/runtime-conformance-engine.md) - Real-time runtime conformance validation
- [Architecture Contract Testing Engine (ACTE)](./engines/operational-excellence/contract-testing-engine.md) - Automated contract testing

### Core Engines
- [Basic Health Check Engine](./engines/operational-excellence/basic-health-check-engine.md) - Health check validation
- [Basic Metrics Engine](./engines/operational-excellence/basic-metrics-engine.md) - Latency, error rate tracking
- [Basic Logging Engine](./engines/operational-excellence/basic-logging-engine.md) - Log format validation

### Advanced Engines

#### Observability & Monitoring
15. **Observability Engine (ATOE)** - Full observability DSL support
16. **Metrics Collection Engine** - Custom metrics definition
17. **Logging Aggregation Engine** - Centralized logging
18. **Distributed Tracing Engine** - Request tracing across services
19. **Alerting Engine** - Alert definition and routing
20. **Dashboard Generation Engine** - Auto-generate monitoring dashboards

#### Operations as Code
21. **Runbook Engine** - Operational procedure automation
22. **Deployment Strategy Engine** - Blue-green, canary, rolling
23. **Incident Response Engine** - Automated incident handling
24. **Operations Automation Engine** - Ops-as-code execution

#### CI/CD Integration
25. **CI/CD Integration Engine** - Validate in pipelines
26. **Pipeline Validation Engine** - Architecture checks in CI/CD
27. **Deployment Validation Engine** - Pre-deployment checks

#### Change Management
28. **Change Impact Analyzer** - Assess change impact
29. **Rollback Controller** - Safe rollback plans
30. **Change Approval Engine** - Change request workflow

---

## Security Pillar

### Core Engines
- [Basic Authentication Engine](./engines/security/basic-authentication-engine.md) - Authentication type validation
- [Basic Encryption Engine](./engines/security/basic-encryption-engine.md) - Encryption requirement checks
- [Basic Security Tags Engine](./engines/security/basic-security-tags-engine.md) - Security tag validation

### Advanced Engines

#### Security & Compliance
31. **Security Validation Engine** - Security policy enforcement
32. **Compliance Engine** - Security + org policies
33. **IAM Engine** - Identity and access management
34. **Network Security Engine** - VPC, security groups, firewalls
35. **Data Classification Engine** - PII, PCI-DSS data handling
36. **Security Audit Engine** - Security compliance auditing
37. **Vulnerability Detection Engine** - Security vulnerability scanning
38. **Threat Modeling Engine** - Security threat analysis

#### Governance
39. **Governance Policy Engine** - Technology + patterns + boundaries
40. **Policy-as-Code Engine** - Governance rules + auto-fixes
41. **Architecture Audit Engine** - Full compliance + decision history

---

## Reliability Pillar

### Advanced Engines
- [Architecture Change Simulation Engine](./engines/reliability/change-simulation-engine.md) - Predictive failure and change analysis
- [Failure Propagation Engine (FPE)](./engines/reliability/failure-propagation-engine.md) - Cascading failure simulation
- [Recovery & Failover Strategy Engine (RFSE)](./engines/reliability/recovery-failover-engine.md) - Recovery strategy simulation
- [Architecture Resilience Testing Engine (ARTE)](./engines/reliability/resilience-testing-engine.md) - Chaos engineering and resilience testing

### Core Engines
- [Basic Retry Engine](./engines/reliability/basic-retry-engine.md) - Retry policy validation
- [Basic Timeout Engine](./engines/reliability/basic-timeout-engine.md) - Timeout requirement checks
- [Basic Circuit Breaker Engine](./engines/reliability/basic-circuit-breaker-engine.md) - Circuit breaker configuration

### Advanced Engines

#### Resilience & Fault Tolerance
42. **Resilience Engine** - Full resilience DSL support
43. **Health Check Engine** - Liveness, readiness, startup probes
44. **Retry Policy Engine** - Exponential, linear, custom retries
45. **Circuit Breaker Engine** - Failure threshold management
46. **Bulkhead Engine** - Resource isolation
47. **Rate Limiting Engine** - Request rate control
48. **Backpressure Engine** - Flow control
49. **Degradation Engine** - Graceful degradation strategies
50. **Auto-healing Engine** - Automatic recovery
51. **Autoscaling Engine** - CPU, latency, event-driven scaling
52. **Failover Engine** - Active-passive, active-active strategies

#### Failure Analysis
53. **Failure Scenario Engine** - Failure mode modeling
54. **Chaos Engineering Engine** - Chaos testing automation
55. **Incident Propagation Simulator** - MTTR, cascade failures
56. **Stress Simulation Engine** - Load, failure, scaling tests
57. **Change Impact Simulator** - Simulates change propagation

#### Redundancy & High Availability
58. **Redundancy Engine** - Multi-region, multi-AZ planning
59. **High Availability Engine** - HA configuration validation

---

## Cost Optimization Pillar

### Advanced Engines
- [Cost Modeling & Optimization Engine (CMOE)](./engines/cost-optimization/cost-modeling-engine.md) - Full cost modeling and optimization

## Sustainability Pillar

### Advanced Engines
- [Architecture Sustainability / Carbon Impact Engine (ASCIE)](./engines/sustainability/carbon-impact-engine.md) - Carbon footprint modeling and optimization
- [Architecture Sustainability Engine (ASE)](./engines/sustainability/architecture-sustainability-engine.md) - Comprehensive sustainability analysis

## Security Pillar

### Advanced Engines
- [Architecture Threat Modeling Engine (ATME)](./engines/security/threat-modeling-engine.md) - STRIDE/LINDDUN threat modeling

## Performance Efficiency Pillar

### Core Engines
- [Basic Latency Engine](./engines/performance-efficiency/basic-latency-engine.md) - Latency target validation
- [Basic Throughput Engine](./engines/performance-efficiency/basic-throughput-engine.md) - Throughput target checks
- [Basic Scaling Engine](./engines/performance-efficiency/basic-scaling-engine.md) - Min/max scaling validation

### Advanced Engines

#### Performance Analysis
60. **Performance Engine** - Full performance DSL support
61. **Latency Analysis Engine** - P50, P90, P99 analysis
62. **Throughput Analysis Engine** - RPS, QPS, TPS analysis
63. **SLO/SLI Engine** - Service level objectives/indicators
64. **Performance Budget Engine** - Performance budget tracking
65. **Bottleneck Detection Engine** - Identify performance bottlenecks
66. **Capacity Planning Engine** - Resource capacity forecasting
67. **Workload Analysis Engine** - Traffic pattern analysis

#### Optimization
68. **Performance Optimization Engine** - Query, algorithm optimization
69. **Caching Strategy Engine** - Cache placement and strategy
70. **Resource Right-Sizing Engine** - Optimal resource selection
71. **Scaling Strategy Engine** - Horizontal/vertical scaling
72. **Load Balancing Engine** - Load distribution strategies

#### Resource Selection
73. **Resource Selection Engine** - Match tech to requirements
74. **Technology Fit Engine** - Technology recommendation
75. **Architecture Ranking Engine** - Score scenarios for optimal choice

---

## Cost Optimization Pillar

### Core Engines
- [Basic Cost Tracking Engine](./engines/cost-optimization/basic-cost-tracking-engine.md) - Cost estimate validation
- [Basic Cost Tags Engine](./engines/cost-optimization/basic-cost-tags-engine.md) - Cost center tagging

### Advanced Engines

#### Cost Analysis & FinOps
76. **Cost Analysis Engine** - Cloud cost predictions
77. **FinOps Engine** - Full cost & FinOps DSL support
78. **Cost Modeling Engine** - Per-service, per-request costing
79. **Budget Engine** - Budget definition and tracking
80. **Cost Allocation Engine** - Team, project cost allocation
81. **Cost Optimization Engine** - Cost reduction strategies
82. **Reserved Instance Engine** - RI planning and optimization
83. **Spot Instance Engine** - Spot instance strategy
84. **Cost Forecasting Engine** - Future cost prediction

---

## Sustainability Pillar

### Core Engines
- [Basic Resource Efficiency Engine](./engines/sustainability/basic-resource-efficiency-engine.md) - Resource efficiency validation
- [Basic Carbon Tracking Engine](./engines/sustainability/basic-carbon-tracking-engine.md) - Carbon footprint tracking

### Advanced Engines

#### Sustainability & Green Computing
85. **Sustainability Engine** - Full sustainability DSL support
86. **Carbon Accounting Engine** - Carbon footprint calculation
87. **Resource Efficiency Engine** - CPU, memory, storage efficiency
88. **Green Computing Engine** - Renewable energy, carbon-neutral
89. **Sustainability Goals Engine** - Carbon reduction targets
90. **Idle Resource Cleanup Engine** - Automatic resource cleanup
91. **Data Lifecycle Engine** - Hot/warm/cold data management

---

## Cross-Pillar Engines

These engines span multiple pillars:

### AI-Powered Architecture
92. **Architecture Review AI** - AI-driven anti-pattern detection
93. **Architecture Advisor** - Recommendations, fixes, refactors
94. **Pattern Recommendation Engine** - Suggest best patterns
95. **Technology Fit Engine** - Match tech to requirements
96. **Architecture Search Engine** - Similarity search across archives
97. **Domain Modeling AI** - Generate bounded contexts
98. **Model Explanation Engine** - Explain complex architectures
99. **MCP Code Generation Engine** - Generate code aligned with architecture

### Systems Thinking & Simulation
100. **Systems Thinking Compiler** - Transpiles DSL v3 → simulation model
101. **Causal Graph Generator** - Loops, causal edges, weights
102. **Behavior Simulator** - Stock-flow dynamics simulation
103. **AI Causal Reasoning Engine (ACRE)** - Explain loops, consequences
104. **Simulation Dashboard** - Charts, scenarios, time loops
105. **Feedback Loop Detector** - Discover reinforcing/balancing loops
106. **Reinforcement Loop Analyzer** - Quantify loop strength
107. **Emergent Outcome Predictor** - Detect tipping points
108. **Scenario Comparison Engine** - Compare two simulation runs
109. **Architecture Stability Score** - Measures systemic fragility
110. **Architecture Hotspot Detector** - Identifies bottlenecks & stress points
111. **Drift Forecasting Engine** - Predict future architecture drift

### Evolution & Migration
112. **Migration Planning Engine** - Generate migration roadmap
113. **Architecture Transformation Execution Engine (ATEX)** - Step-by-step migration
114. **Architecture Orchestration Engine (AAOE)** - Autonomous execution
115. **Architecture Roadmap Auto-Generation Engine (ARAGE)** - Auto-generate roadmaps
116. **Continuous Architecture Delivery Engine (CADE)** - Continuous delivery
117. **Architecture Impact Forecasting Engine (AIFE)** - Forecast future impact
118. **Architecture Evolution Simulator (MAES)** - Evolution simulation
119. **Multi-Scenario Modeling Engine (MSME)** - Scenario comparison
120. **Architecture Drift Auto-Remediation Engine (ADARE)** - Auto-fix drift

### Governance & Knowledge
121. **Architecture Knowledge Graph Engine (AKGE)** - Full enterprise KG
122. **Architecture Query Language (AQL)** - Graph queries
123. **ADR Manager** - Lifecycle: propose → approve → supersede
124. **Tech Catalog** - Approved/forbidden stacks
125. **Pattern Catalog** - CQRS, Saga, Event Sourcing, etc.
126. **Risk Analysis Engine** - Determine risk blast radius
127. **Architecture Goals Engine** - Align architecture to business KPIs

### Documentation & Communication
128. **Architecture Auto-Documentation Engine** - AI-generated docs
129. **Architecture Communication Hub (ACH)** - Team notifications
130. **Architecture Timeline Engine** - Evolution visualization
131. **Architecture Linting & Style Engine** - Code style for architecture

---

## Engine Status

| Category | Core | Advanced | Total |
|----------|------|----------|-------|
| Core Engines | 14 | 0 | 14 |
| Operational Excellence | 3 | 16 | 19 |
| Security | 3 | 11 | 14 |
| Reliability | 3 | 18 | 21 |
| Performance Efficiency | 3 | 16 | 19 |
| Cost Optimization | 2 | 9 | 11 |
| Sustainability | 2 | 7 | 9 |
| Cross-Pillar | 0 | 40 | 40 |
| **Total** | **30** | **117** | **147** |

---

## Implementation Priority

### Phase 1 (MVP) - Core Engines
- All 14 core engines
- Basic pillar engines (3 per pillar = 18)
- **Total: 32 engines**

### Phase 2 - Advanced Pillar Engines
- Operational Excellence advanced (16)
- Security advanced (11)
- Reliability advanced (18)
- Performance Efficiency advanced (16)
- Cost Optimization advanced (9)
- Sustainability advanced (7)
- **Total: 77 engines**

### Phase 3 - Cross-Pillar Engines
- AI-Powered Architecture (8)
- Systems Thinking & Simulation (11)
- Evolution & Migration (9)
- Governance & Knowledge (7)
- Documentation & Communication (4)
- **Total: 39 engines**

---

## Engine Integration

Engines integrate through:

1. **Architecture Knowledge Graph (AKGE)** - Central data model
2. **MCP API** - Standardized interface
3. **Plugin System** - Extensible architecture
4. **Event Bus** - Inter-engine communication

---

## Detailed Engine Documentation

Detailed specifications for major engines are available in the [engines directory](./engines/):

### Core Engines
- [Validation Engine](./engines/core/validation-engine.md) - Core validation engine with plugin system
- [Two-Way Sync Engine](./engines/core/two-way-sync-engine.md) - Bidirectional DSL ↔ Diagram synchronization
- [Model Composition Engine](./engines/core/model-composition-engine.md) - Multi-module architecture composition
- [Import Resolver](./engines/core/import-resolver.md) - Local, GitHub, and registry import resolution
- [Model-Aware Code Generation Engine](./engines/core/code-generation-engine.md) - Architecture-driven code generation
- [Architecture Auto-Documentation Engine](./engines/core/documentation-engine.md) - Auto-generated architecture docs
- [Template Engine](./engines/core/template-engine.md) - Template search, filtering, and instantiation
- [Auto-Layout Engine](./engines/core/auto-layout-engine.md) - Automatic diagram layout
- [Architecture Wizard Engine](./engines/core/architecture-wizard-engine.md) - Guided creation flows
- [Visual Diff Engine](./engines/core/visual-diff-engine.md) - Structure and diagram diffing
- [Undo/Redo Engine](./engines/core/undo-redo-engine.md) - Unified history management
- [Diagram Serializer Engine](./engines/core/diagram-serializer-engine.md) - Diff-based DSL serialization
- [Graph Engine](./engines/core/graph-engine.md) - Graph operations and cycle detection
- [Visual Validation Overlays](./engines/core/visual-validation-overlays.md) - Live validation feedback on diagram
- [Evolution Timeline Tracker](./engines/core/evolution-timeline-tracker.md) - Git-powered architecture time-travel
- [Reference Resolution Engine](./engines/core/reference-resolution-engine.md) - Cross-module and cross-context reference resolution
- [Drift Detector](./engines/core/drift-detector.md) - Architecture → code → infrastructure drift detection
- [DSL Parser Engine](./engines/core/dsl-parser-engine.md) - Parses DSL text into AST using participle (handles all features: core, extensions, systems thinking)
- [AST Transformer Engine](./engines/core/ast-transformer-engine.md) - Transforms AST into typed IR models

### Cross-Pillar Engines
- [AI-Guided Architecture Review Engine](./engines/cross-pillar/ai-architecture-review-engine.md) - Automated architecture reviews
- [AI Causal Reasoning Engine (ACRE)](./engines/cross-pillar/ai-causal-reasoning-engine.md) - Systems thinking and causal analysis
- [Continuous Architecture Delivery Engine (CADE)](./engines/cross-pillar/continuous-architecture-delivery-engine.md) - Architecture DevOps pipeline
- [Architecture Timeline Engine](./engines/cross-pillar/architecture-timeline-engine.md) - Time-travel through architecture evolution
- [Policy-as-Code Engine](./engines/cross-pillar/policy-as-code-engine.md) - Automated governance enforcement
- [Architecture Roadmap Generation Engine (ARAGE)](./engines/cross-pillar/architecture-roadmap-generation-engine.md) - Auto-generate architecture roadmaps
- [Architecture Linting & Style Engine](./engines/cross-pillar/architecture-linting-engine.md) - DSL style and consistency enforcement
- [Architecture Drift Auto-Remediation Engine (ADARE)](./engines/cross-pillar/architecture-drift-remediation-engine.md) - Auto-fix architecture drift
- [Architecture Evolution Knowledge Graph (AEKG)](./engines/cross-pillar/architecture-knowledge-graph-engine.md) - Central knowledge graph
- [Architecture Query Language (AQL)](./engines/cross-pillar/architecture-query-language.md) - Query interface for architecture
- [Architecture Drift Prevention Engine (ADPE)](./engines/cross-pillar/architecture-drift-prevention-engine.md) - Proactive drift prevention
- [Architecture Optimization Engine (AOE)](./engines/cross-pillar/architecture-optimization-engine.md) - Multi-objective architecture optimization
- [Architecture Compliance Score Engine (ACSE)](./engines/cross-pillar/architecture-compliance-score-engine.md) - Unified architecture scoring
- [Architecture Communication Hub (ACH)](./engines/cross-pillar/architecture-communication-hub.md) - Cross-team architecture communication
- [Dynamic Architecture Hotspot Detection Engine (DAHDE)](./engines/cross-pillar/hotspot-detection-engine.md) - Automated hotspot detection
- [Architecture Ranking Engine (ARE)](./engines/cross-pillar/architecture-ranking-engine.md) - Multi-dimensional architecture scoring
- [Multi-Simulation Orchestration Engine (MSOE)](./engines/cross-pillar/multi-simulation-orchestration-engine.md) - Batch and parallel simulation orchestration
- [Behavior Sensitivity Analysis Engine (BSAE)](./engines/cross-pillar/behavior-sensitivity-analysis-engine.md) - Parameter sensitivity analysis
- [Reinforcement Loop Analyzer Engine (RLAE)](./engines/cross-pillar/reinforcement-loop-analyzer-engine.md) - Feedback loop detection and analysis
- [Scenario Comparison Engine (SCE)](./engines/cross-pillar/scenario-comparison-engine.md) - Architecture version and scenario comparison
- [Architecture Value Realization Engine (AVRE)](./engines/cross-pillar/architecture-value-realization-engine.md) - Business value quantification
- [Architecture Impact Forecasting Engine (AIFE)](./engines/cross-pillar/architecture-impact-forecasting-engine.md) - Predictive impact analysis
- [Architecture Scenario Forecasting Engine (ASFE)](./engines/cross-pillar/architecture-scenario-forecasting-engine.md) - Long-term scenario forecasting
- [Autonomous Architecture Orchestration Engine (AAOE)](./engines/cross-pillar/autonomous-architecture-orchestration-engine.md) - Autonomous execution automation
- [Architecture Transformation Execution Engine (ATEX)](./engines/cross-pillar/architecture-transformation-execution-engine.md) - Transformation planning and execution
- [Architecture Modernization Recommendation Engine (AMRE)](./engines/cross-pillar/architecture-modernization-recommendation-engine.md) - AI-driven modernization recommendations
- [Risk Modeling & Impact Analysis Engine (RMIAE)](./engines/cross-pillar/risk-modeling-engine.md) - Comprehensive risk analysis
- [Architecture Debt Analyzer Engine (ADAE)](./engines/cross-pillar/architecture-debt-analyzer-engine.md) - Debt tracking and remediation
- [Architecture Fitness Function Engine (AFFE)](./engines/cross-pillar/architecture-fitness-function-engine.md) - Continuous quality enforcement
- [Architecture Governance Engine (AGE)](./engines/cross-pillar/architecture-governance-engine.md) - Unified governance framework
- [Architecture Governance & Policy Engine (AGPE)](./engines/cross-pillar/architecture-governance-policy-engine.md) - Policy enforcement
- [Architecture Benchmarking Engine (ABE)](./engines/cross-pillar/architecture-benchmarking-engine.md) - External benchmarking
- [Architecture Evolution Timeline Engine (AETE)](./engines/cross-pillar/architecture-evolution-timeline-engine.md) - Git-integrated evolution tracking
- [Architecture Refactoring Simulation Engine (ARSE)](./engines/cross-pillar/architecture-refactoring-simulation-engine.md) - Refactoring impact simulation
- [Visual Simulation Dashboard](./engines/cross-pillar/visual-simulation-dashboard.md) - Interactive simulation visualization and architecture cockpit
- [Behavior Simulator](./engines/cross-pillar/behavior-simulator.md) - Stock-flow dynamics simulation and system behavior prediction
- [Systems Thinking Compiler](./engines/cross-pillar/systems-thinking-compiler.md) - Transpiles DSL v3 into simulation models
- [Causal Graph Generator](./engines/cross-pillar/causal-graph-generator.md) - Builds causal graphs with loops, edges, and weights
- [Feedback Loop Detector](./engines/cross-pillar/feedback-loop-detector.md) - Discovers reinforcing and balancing loops
- [Architecture Evolution Simulator (MAES)](./engines/cross-pillar/architecture-evolution-simulator.md) - Multi-system evolution simulation
- [Multi-System Scenario Modeling Engine (MSME)](./engines/cross-pillar/multi-scenario-modeling-engine.md) - Alternative architecture futures modeling
- [Migration Planning Engine](./engines/cross-pillar/migration-planning-engine.md) - Generate migration roadmap
- [Emergent Outcome Predictor](./engines/cross-pillar/emergent-outcome-predictor.md) - Detect tipping points
- [Architecture Stability Score](./engines/cross-pillar/architecture-stability-score.md) - Measures systemic fragility
- [Drift Forecasting Engine](./engines/cross-pillar/drift-forecasting-engine.md) - Predict future architecture drift
- [Stress Simulation Engine](./engines/reliability/stress-simulation-engine.md) - Load, failure, scaling tests
- [Incident Propagation Simulator](./engines/reliability/incident-propagation-simulator.md) - MTTR, cascade failures
- [CI/CD Integration Engine](./engines/operational-excellence/cicd-integration-engine.md) - Validate in pipelines
- [Metrics Collection Engine](./engines/operational-excellence/metrics-collection-engine.md) - Custom metrics definition
- [Logging Aggregation Engine](./engines/operational-excellence/logging-aggregation-engine.md) - Centralized logging
- [Distributed Tracing Engine](./engines/operational-excellence/distributed-tracing-engine.md) - Request tracing across services
- [Alerting Engine](./engines/operational-excellence/alerting-engine.md) - Alert definition and routing
- [Dashboard Generation Engine](./engines/operational-excellence/dashboard-generation-engine.md) - Auto-generate monitoring dashboards
- [Runbook Engine](./engines/operational-excellence/runbook-engine.md) - Operational procedure automation
- [Deployment Strategy Engine](./engines/operational-excellence/deployment-strategy-engine.md) - Blue-green, canary, rolling
- [Incident Response Engine](./engines/operational-excellence/incident-response-engine.md) - Automated incident handling
- [Operations Automation Engine](./engines/operational-excellence/operations-automation-engine.md) - Ops-as-code execution
- [Pipeline Validation Engine](./engines/operational-excellence/pipeline-validation-engine.md) - Architecture checks in CI/CD
- [Deployment Validation Engine](./engines/operational-excellence/deployment-validation-engine.md) - Pre-deployment checks
- [Change Impact Analyzer](./engines/cross-pillar/change-impact-analyzer.md) - Assess change impact
- [Change Approval Engine](./engines/cross-pillar/change-approval-engine.md) - Change request workflow
- [Resilience Engine](./engines/reliability/resilience-engine.md) - Full resilience DSL support
- [Health Check Engine](./engines/reliability/health-check-engine.md) - Liveness, readiness, startup probes
- [Retry Policy Engine](./engines/reliability/retry-policy-engine.md) - Exponential, linear, custom retries
- [Circuit Breaker Engine](./engines/reliability/circuit-breaker-engine.md) - Failure threshold management
- [Bulkhead Engine](./engines/reliability/bulkhead-engine.md) - Resource isolation
- [Rate Limiting Engine](./engines/reliability/rate-limiting-engine.md) - Request rate control
- [Backpressure Engine](./engines/reliability/backpressure-engine.md) - Flow control
- [Degradation Engine](./engines/reliability/degradation-engine.md) - Graceful degradation strategies
- [Auto-healing Engine](./engines/reliability/auto-healing-engine.md) - Automatic recovery
- [Autoscaling Engine](./engines/reliability/autoscaling-engine.md) - CPU, latency, event-driven scaling
- [Failover Engine](./engines/reliability/failover-engine.md) - Active-passive, active-active strategies
- [Failure Scenario Engine](./engines/reliability/failure-scenario-engine.md) - Failure mode modeling
- [Chaos Engineering Engine](./engines/reliability/chaos-engineering-engine.md) - Chaos testing automation
- [Redundancy Engine](./engines/reliability/redundancy-engine.md) - Multi-region, multi-AZ planning
- [High Availability Engine](./engines/reliability/high-availability-engine.md) - HA configuration validation
- [Performance Engine](./engines/performance-efficiency/performance-engine.md) - Full performance DSL support
- [Latency Analysis Engine](./engines/performance-efficiency/latency-analysis-engine.md) - P50, P90, P99 analysis
- [Throughput Analysis Engine](./engines/performance-efficiency/throughput-analysis-engine.md) - RPS, QPS, TPS analysis
- [SLO/SLI Engine](./engines/performance-efficiency/slo-sli-engine.md) - Service level objectives/indicators
- [Performance Budget Engine](./engines/performance-efficiency/performance-budget-engine.md) - Performance budget tracking
- [Bottleneck Detection Engine](./engines/performance-efficiency/bottleneck-detection-engine.md) - Identify performance bottlenecks
- [Capacity Planning Engine](./engines/performance-efficiency/capacity-planning-engine.md) - Resource capacity forecasting
- [Workload Analysis Engine](./engines/performance-efficiency/workload-analysis-engine.md) - Traffic pattern analysis
- [Performance Optimization Engine](./engines/performance-efficiency/performance-optimization-engine.md) - Query, algorithm optimization
- [Caching Strategy Engine](./engines/performance-efficiency/caching-strategy-engine.md) - Cache placement and strategy
- [Resource Right-Sizing Engine](./engines/performance-efficiency/resource-right-sizing-engine.md) - Optimal resource selection
- [Scaling Strategy Engine](./engines/performance-efficiency/scaling-strategy-engine.md) - Horizontal/vertical scaling
- [Load Balancing Engine](./engines/performance-efficiency/load-balancing-engine.md) - Load distribution strategies
- [Resource Selection Engine](./engines/performance-efficiency/resource-selection-engine.md) - Match tech to requirements
- [Cost Analysis Engine](./engines/cost-optimization/cost-analysis-engine.md) - Cloud cost predictions
- [FinOps Engine](./engines/cost-optimization/finops-engine.md) - Full cost & FinOps DSL support
- [Budget Engine](./engines/cost-optimization/budget-engine.md) - Budget definition and tracking
- [Cost Allocation Engine](./engines/cost-optimization/cost-allocation-engine.md) - Team, project cost allocation
- [Cost Optimization Engine](./engines/cost-optimization/cost-optimization-engine.md) - Cost reduction strategies
- [Reserved Instance Engine](./engines/cost-optimization/reserved-instance-engine.md) - RI planning and optimization
- [Spot Instance Engine](./engines/cost-optimization/spot-instance-engine.md) - Spot instance strategy
- [Cost Forecasting Engine](./engines/cost-optimization/cost-forecasting-engine.md) - Future cost prediction
- [Sustainability Engine](./engines/sustainability/sustainability-engine.md) - Full sustainability DSL support
- [Carbon Accounting Engine](./engines/sustainability/carbon-accounting-engine.md) - Carbon footprint calculation
- [Resource Efficiency Engine](./engines/sustainability/resource-efficiency-engine.md) - CPU, memory, storage efficiency
- [Green Computing Engine](./engines/sustainability/green-computing-engine.md) - Renewable energy, carbon-neutral
- [Sustainability Goals Engine](./engines/sustainability/sustainability-goals-engine.md) - Carbon reduction targets
- [Idle Resource Cleanup Engine](./engines/sustainability/idle-resource-cleanup-engine.md) - Automatic resource cleanup
- [Data Lifecycle Engine](./engines/sustainability/data-lifecycle-engine.md) - Hot/warm/cold data management
- [Security Validation Engine](./engines/security/security-validation-engine.md) - Security policy enforcement
- [Compliance Engine](./engines/security/compliance-engine.md) - Security + org policies
- [IAM Engine](./engines/security/iam-engine.md) - Identity and access management
- [Network Security Engine](./engines/security/network-security-engine.md) - VPC, security groups, firewalls
- [Data Classification Engine](./engines/security/data-classification-engine.md) - PII, PCI-DSS data handling
- [Security Audit Engine](./engines/security/security-audit-engine.md) - Security compliance auditing
- [Vulnerability Detection Engine](./engines/security/vulnerability-detection-engine.md) - Security vulnerability scanning
- [Architecture Knowledge Graph Engine (AKGE)](./engines/cross-pillar/architecture-knowledge-graph-engine-akge.md) - Full enterprise KG
- [Risk Analysis Engine](./engines/cross-pillar/risk-analysis-engine.md) - Determine risk blast radius
- [MCP Code Generation Engine](./engines/cross-pillar/mcp-code-generation-engine.md) - MCP integration for code generation
- [Risk Analysis Engine](./engines/cross-pillar/risk-analysis-engine.md) - Risk blast radius analysis
- [Rollback Controller](./engines/cross-pillar/rollback-controller.md) - Safe rollback plans
- [Architecture Audit Engine](./engines/cross-pillar/architecture-audit-engine.md) - Full compliance + decision history
- [ADR Manager](./engines/cross-pillar/adr-manager.md) - Lifecycle: propose → approve → supersede
- [Tech Catalog](./engines/cross-pillar/tech-catalog.md) - Approved/forbidden stacks
- [Pattern Catalog](./engines/cross-pillar/pattern-catalog.md) - CQRS, Saga, Event Sourcing, etc.
- [Architecture Goals Engine](./engines/cross-pillar/architecture-goals-engine.md) - Align architecture to business KPIs
- [Architecture Advisor](./engines/cross-pillar/architecture-advisor.md) - Recommendations, fixes, refactors
- [Pattern Recommendation Engine](./engines/cross-pillar/pattern-recommendation-engine.md) - Suggest best patterns
- [Technology Fit Engine](./engines/cross-pillar/technology-fit-engine.md) - Match tech to requirements
- [Architecture Search Engine](./engines/cross-pillar/architecture-search-engine.md) - Similarity search across archives
- [Domain Modeling AI](./engines/cross-pillar/domain-modeling-ai.md) - Generate bounded contexts
- [Model Explanation Engine](./engines/cross-pillar/model-explanation-engine.md) - Explain complex architectures

### Reliability Engines
- [Architecture Change Simulation Engine](./engines/reliability/change-simulation-engine.md) - Predictive failure and change analysis

---

## Next Steps

1. **Start with Core** - Implement 14 core engines + 18 basic pillar engines
2. **Add Pillar Extensions** - Enable advanced engines per pillar as needed
3. **Enable Cross-Pillar** - Add AI, simulation, evolution engines
4. **Read Detailed Specs** - See [engines directory](./engines/) for complete specifications

---

*This comprehensive engine catalog ensures Sruja supports all aspects of well-architected systems, from basic modeling to advanced enterprise features.*

