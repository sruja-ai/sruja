# Engines Documentation

This directory contains detailed specifications for all Sruja engines, organized by Well-Architected Framework pillars.

[← Back to Engines Overview](../engines.md)

## ⚠️ Don't Be Overwhelmed!

**171 engines sounds like a lot, but you only need a few to get started.**

👉 **Start here:** [Quick Start Guide](./QUICK-START.md) - What engines do you actually need?

👉 **Full guide:** [Getting Started Guide](../../GETTING-STARTED.md) - Complete approachability guide

## Structure

```
engines/
├── core/              # Core engines (foundational)
│   ├── validation-engine.md
│   ├── two-way-sync-engine.md
│   ├── model-composition-engine.md
│   └── import-resolver.md
├── operational/       # Operational Excellence engines
├── security/          # Security engines
├── reliability/       # Reliability engines
│   └── change-simulation-engine.md
├── performance/       # Performance Efficiency engines
├── cost/              # Cost Optimization engines
├── sustainability/    # Sustainability engines
└── cross-pillar/     # Cross-pillar engines (AI, simulation, etc.)
    ├── ai-architecture-review-engine.md
    ├── ai-causal-reasoning-engine.md
    ├── continuous-architecture-delivery-engine.md
    ├── architecture-timeline-engine.md
    ├── policy-as-code-engine.md
    ├── architecture-roadmap-generation-engine.md
    └── architecture-linting-engine.md
```

## Documented Engines

### Core Engines (10)
1. ✅ [Validation Engine](./core/validation-engine.md) - Core validation with plugin system
2. ✅ [Two-Way Sync Engine](./core/two-way-sync-engine.md) - Bidirectional DSL ↔ Diagram sync
3. ✅ [Model Composition Engine](./core/model-composition-engine.md) - Multi-module architecture composition
4. ✅ [Import Resolver](./core/import-resolver.md) - Local, GitHub, and registry imports
37. ✅ [Model-Aware Code Generation Engine](./core/code-generation-engine.md) - Architecture-driven code generation
38. ✅ [Architecture Auto-Documentation Engine](./core/documentation-engine.md) - Auto-generated architecture docs
50. ✅ [Template Engine](./core/template-engine.md) - Template search, filtering, and instantiation
51. ✅ [Auto-Layout Engine](./core/auto-layout-engine.md) - Automatic diagram layout
52. ✅ [Architecture Wizard Engine](./core/architecture-wizard-engine.md) - Guided creation flows
53. ✅ [Visual Diff Engine](./core/visual-diff-engine.md) - Structure and diagram diffing

### Cross-Pillar Engines (7)
5. ✅ [AI-Guided Architecture Review Engine](./cross-pillar/ai-architecture-review-engine.md) - Automated architecture reviews
6. ✅ [AI Causal Reasoning Engine (ACRE)](./cross-pillar/ai-causal-reasoning-engine.md) - Systems thinking and causal analysis
7. ✅ [Continuous Architecture Delivery Engine (CADE)](./cross-pillar/continuous-architecture-delivery-engine.md) - Architecture DevOps pipeline
8. ✅ [Architecture Timeline Engine](./cross-pillar/architecture-timeline-engine.md) - Time-travel through architecture evolution
9. ✅ [Policy-as-Code Engine](./cross-pillar/policy-as-code-engine.md) - Automated governance enforcement
10. ✅ [Architecture Roadmap Generation Engine (ARAGE)](./cross-pillar/architecture-roadmap-generation-engine.md) - Auto-generate architecture roadmaps
11. ✅ [Architecture Linting & Style Engine](./cross-pillar/architecture-linting-engine.md) - DSL style and consistency enforcement

### Reliability Engines (1)
12. ✅ [Architecture Change Simulation Engine](./reliability/change-simulation-engine.md) - Predictive failure and change analysis

### Cross-Pillar Engines (11)
5. ✅ [AI-Guided Architecture Review Engine](./cross-pillar/ai-architecture-review-engine.md) - Automated architecture reviews
6. ✅ [AI Causal Reasoning Engine (ACRE)](./cross-pillar/ai-causal-reasoning-engine.md) - Systems thinking and causal analysis
7. ✅ [Continuous Architecture Delivery Engine (CADE)](./cross-pillar/continuous-architecture-delivery-engine.md) - Architecture DevOps pipeline
8. ✅ [Architecture Timeline Engine](./cross-pillar/architecture-timeline-engine.md) - Time-travel through architecture evolution
9. ✅ [Policy-as-Code Engine](./cross-pillar/policy-as-code-engine.md) - Automated governance enforcement
10. ✅ [Architecture Roadmap Generation Engine (ARAGE)](./cross-pillar/architecture-roadmap-generation-engine.md) - Auto-generate architecture roadmaps
11. ✅ [Architecture Linting & Style Engine](./cross-pillar/architecture-linting-engine.md) - DSL style and consistency enforcement
12. ✅ [Architecture Drift Auto-Remediation Engine (ADARE)](./cross-pillar/architecture-drift-remediation-engine.md) - Auto-fix architecture drift
13. ✅ [Architecture Evolution Knowledge Graph (AEKG)](./cross-pillar/architecture-knowledge-graph-engine.md) - Central knowledge graph
14. ✅ [Architecture Query Language (AQL)](./cross-pillar/architecture-query-language.md) - Query interface for architecture
15. ✅ [Architecture Drift Prevention Engine (ADPE)](./cross-pillar/architecture-drift-prevention-engine.md) - Proactive drift prevention

### Operational Excellence Engines (1)
16. ✅ [Architecture-Time Observability Engine (ATOE)](./operational-excellence/architecture-observability-engine.md) - Runtime-to-architecture feedback loop

### Cost Optimization Engines (1)
17. ✅ [Cost Modeling & Optimization Engine (CMOE)](./cost-optimization/cost-modeling-engine.md) - Full cost modeling and optimization

### Cross-Pillar Engines (12)
5. ✅ [AI-Guided Architecture Review Engine](./cross-pillar/ai-architecture-review-engine.md) - Automated architecture reviews
6. ✅ [AI Causal Reasoning Engine (ACRE)](./cross-pillar/ai-causal-reasoning-engine.md) - Systems thinking and causal analysis
7. ✅ [Continuous Architecture Delivery Engine (CADE)](./cross-pillar/continuous-architecture-delivery-engine.md) - Architecture DevOps pipeline
8. ✅ [Architecture Timeline Engine](./cross-pillar/architecture-timeline-engine.md) - Time-travel through architecture evolution
9. ✅ [Policy-as-Code Engine](./cross-pillar/policy-as-code-engine.md) - Automated governance enforcement
10. ✅ [Architecture Roadmap Generation Engine (ARAGE)](./cross-pillar/architecture-roadmap-generation-engine.md) - Auto-generate architecture roadmaps
11. ✅ [Architecture Linting & Style Engine](./cross-pillar/architecture-linting-engine.md) - DSL style and consistency enforcement
12. ✅ [Architecture Drift Auto-Remediation Engine (ADARE)](./cross-pillar/architecture-drift-remediation-engine.md) - Auto-fix architecture drift
13. ✅ [Architecture Evolution Knowledge Graph (AEKG)](./cross-pillar/architecture-knowledge-graph-engine.md) - Central knowledge graph
14. ✅ [Architecture Query Language (AQL)](./cross-pillar/architecture-query-language.md) - Query interface for architecture
15. ✅ [Architecture Drift Prevention Engine (ADPE)](./cross-pillar/architecture-drift-prevention-engine.md) - Proactive drift prevention
18. ✅ [Architecture Optimization Engine (AOE)](./cross-pillar/architecture-optimization-engine.md) - Multi-objective architecture optimization

### Operational Excellence Engines (2)
16. ✅ [Architecture-Time Observability Engine (ATOE)](./operational-excellence/architecture-observability-engine.md) - Runtime-to-architecture feedback loop
19. ✅ [Architecture Runtime Conformance Engine (ARCE)](./operational-excellence/runtime-conformance-engine.md) - Real-time runtime conformance validation

### Security Engines (1)
20. ✅ [Architecture Threat Modeling Engine (ATME)](./security/threat-modeling-engine.md) - STRIDE/LINDDUN threat modeling

### Cost Optimization Engines (1)
17. ✅ [Cost Modeling & Optimization Engine (CMOE)](./cost-optimization/cost-modeling-engine.md) - Full cost modeling and optimization

### Sustainability Engines (2)
21. ✅ [Architecture Sustainability / Carbon Impact Engine (ASCIE)](./sustainability/carbon-impact-engine.md) - Carbon footprint modeling and optimization
34. ✅ [Architecture Sustainability Engine (ASE)](./sustainability/architecture-sustainability-engine.md) - Comprehensive sustainability analysis

### Cross-Pillar Engines (14)
5. ✅ [AI-Guided Architecture Review Engine](./cross-pillar/ai-architecture-review-engine.md) - Automated architecture reviews
6. ✅ [AI Causal Reasoning Engine (ACRE)](./cross-pillar/ai-causal-reasoning-engine.md) - Systems thinking and causal analysis
7. ✅ [Continuous Architecture Delivery Engine (CADE)](./cross-pillar/continuous-architecture-delivery-engine.md) - Architecture DevOps pipeline
8. ✅ [Architecture Timeline Engine](./cross-pillar/architecture-timeline-engine.md) - Time-travel through architecture evolution
9. ✅ [Policy-as-Code Engine](./cross-pillar/policy-as-code-engine.md) - Automated governance enforcement
10. ✅ [Architecture Roadmap Generation Engine (ARAGE)](./cross-pillar/architecture-roadmap-generation-engine.md) - Auto-generate architecture roadmaps
11. ✅ [Architecture Linting & Style Engine](./cross-pillar/architecture-linting-engine.md) - DSL style and consistency enforcement
12. ✅ [Architecture Drift Auto-Remediation Engine (ADARE)](./cross-pillar/architecture-drift-remediation-engine.md) - Auto-fix architecture drift
13. ✅ [Architecture Evolution Knowledge Graph (AEKG)](./cross-pillar/architecture-knowledge-graph-engine.md) - Central knowledge graph
14. ✅ [Architecture Query Language (AQL)](./cross-pillar/architecture-query-language.md) - Query interface for architecture
15. ✅ [Architecture Drift Prevention Engine (ADPE)](./cross-pillar/architecture-drift-prevention-engine.md) - Proactive drift prevention
18. ✅ [Architecture Optimization Engine (AOE)](./cross-pillar/architecture-optimization-engine.md) - Multi-objective architecture optimization
22. ✅ [Architecture Compliance Score Engine (ACSE)](./cross-pillar/architecture-compliance-score-engine.md) - Unified architecture scoring
23. ✅ [Architecture Communication Hub (ACH)](./cross-pillar/architecture-communication-hub.md) - Cross-team architecture communication

### Reliability Engines (3)
16. ✅ [Architecture Change Simulation Engine](./reliability/change-simulation-engine.md) - Predictive failure and change analysis
24. ✅ [Failure Propagation Engine (FPE)](./reliability/failure-propagation-engine.md) - Cascading failure simulation
25. ✅ [Recovery & Failover Strategy Engine (RFSE)](./reliability/recovery-failover-engine.md) - Recovery strategy simulation

### Operational Excellence Engines (3)
17. ✅ [Architecture-Time Observability Engine (ATOE)](./operational-excellence/architecture-observability-engine.md) - Runtime-to-architecture feedback loop
19. ✅ [Architecture Runtime Conformance Engine (ARCE)](./operational-excellence/runtime-conformance-engine.md) - Real-time runtime conformance validation
26. ✅ [Architecture Contract Testing Engine (ACTE)](./operational-excellence/contract-testing-engine.md) - Automated contract testing

### Security Engines (1)
20. ✅ [Architecture Threat Modeling Engine (ATME)](./security/threat-modeling-engine.md) - STRIDE/LINDDUN threat modeling

### Cost Optimization Engines (1)
21. ✅ [Cost Modeling & Optimization Engine (CMOE)](./cost-optimization/cost-modeling-engine.md) - Full cost modeling and optimization

### Sustainability Engines (2)
21. ✅ [Architecture Sustainability / Carbon Impact Engine (ASCIE)](./sustainability/carbon-impact-engine.md) - Carbon footprint modeling and optimization
34. ✅ [Architecture Sustainability Engine (ASE)](./sustainability/architecture-sustainability-engine.md) - Comprehensive sustainability analysis

### Cross-Pillar Engines (16)
5. ✅ [AI-Guided Architecture Review Engine](./cross-pillar/ai-architecture-review-engine.md) - Automated architecture reviews
6. ✅ [AI Causal Reasoning Engine (ACRE)](./cross-pillar/ai-causal-reasoning-engine.md) - Systems thinking and causal analysis
7. ✅ [Continuous Architecture Delivery Engine (CADE)](./cross-pillar/continuous-architecture-delivery-engine.md) - Architecture DevOps pipeline
8. ✅ [Architecture Timeline Engine](./cross-pillar/architecture-timeline-engine.md) - Time-travel through architecture evolution
9. ✅ [Policy-as-Code Engine](./cross-pillar/policy-as-code-engine.md) - Automated governance enforcement
10. ✅ [Architecture Roadmap Generation Engine (ARAGE)](./cross-pillar/architecture-roadmap-generation-engine.md) - Auto-generate architecture roadmaps
11. ✅ [Architecture Linting & Style Engine](./cross-pillar/architecture-linting-engine.md) - DSL style and consistency enforcement
12. ✅ [Architecture Drift Auto-Remediation Engine (ADARE)](./cross-pillar/architecture-drift-remediation-engine.md) - Auto-fix architecture drift
13. ✅ [Architecture Evolution Knowledge Graph (AEKG)](./cross-pillar/architecture-knowledge-graph-engine.md) - Central knowledge graph
14. ✅ [Architecture Query Language (AQL)](./cross-pillar/architecture-query-language.md) - Query interface for architecture
15. ✅ [Architecture Drift Prevention Engine (ADPE)](./cross-pillar/architecture-drift-prevention-engine.md) - Proactive drift prevention
18. ✅ [Architecture Optimization Engine (AOE)](./cross-pillar/architecture-optimization-engine.md) - Multi-objective architecture optimization
22. ✅ [Architecture Compliance Score Engine (ACSE)](./cross-pillar/architecture-compliance-score-engine.md) - Unified architecture scoring
23. ✅ [Architecture Communication Hub (ACH)](./cross-pillar/architecture-communication-hub.md) - Cross-team architecture communication
27. ✅ [Dynamic Architecture Hotspot Detection Engine (DAHDE)](./cross-pillar/hotspot-detection-engine.md) - Automated hotspot detection
28. ✅ [Architecture Ranking Engine (ARE)](./cross-pillar/architecture-ranking-engine.md) - Multi-dimensional architecture scoring

### Reliability Engines (4)
16. ✅ [Architecture Change Simulation Engine](./reliability/change-simulation-engine.md) - Predictive failure and change analysis
24. ✅ [Failure Propagation Engine (FPE)](./reliability/failure-propagation-engine.md) - Cascading failure simulation
25. ✅ [Recovery & Failover Strategy Engine (RFSE)](./reliability/recovery-failover-engine.md) - Recovery strategy simulation
29. ✅ [Architecture Resilience Testing Engine (ARTE)](./reliability/resilience-testing-engine.md) - Chaos engineering and resilience testing

### Cross-Pillar Engines (20)
5. ✅ [AI-Guided Architecture Review Engine](./cross-pillar/ai-architecture-review-engine.md) - Automated architecture reviews
6. ✅ [AI Causal Reasoning Engine (ACRE)](./cross-pillar/ai-causal-reasoning-engine.md) - Systems thinking and causal analysis
7. ✅ [Continuous Architecture Delivery Engine (CADE)](./cross-pillar/continuous-architecture-delivery-engine.md) - Architecture DevOps pipeline
8. ✅ [Architecture Timeline Engine](./cross-pillar/architecture-timeline-engine.md) - Time-travel through architecture evolution
9. ✅ [Policy-as-Code Engine](./cross-pillar/policy-as-code-engine.md) - Automated governance enforcement
10. ✅ [Architecture Roadmap Generation Engine (ARAGE)](./cross-pillar/architecture-roadmap-generation-engine.md) - Auto-generate architecture roadmaps
11. ✅ [Architecture Linting & Style Engine](./cross-pillar/architecture-linting-engine.md) - DSL style and consistency enforcement
12. ✅ [Architecture Drift Auto-Remediation Engine (ADARE)](./cross-pillar/architecture-drift-remediation-engine.md) - Auto-fix architecture drift
13. ✅ [Architecture Evolution Knowledge Graph (AEKG)](./cross-pillar/architecture-knowledge-graph-engine.md) - Central knowledge graph
14. ✅ [Architecture Query Language (AQL)](./cross-pillar/architecture-query-language.md) - Query interface for architecture
15. ✅ [Architecture Drift Prevention Engine (ADPE)](./cross-pillar/architecture-drift-prevention-engine.md) - Proactive drift prevention
18. ✅ [Architecture Optimization Engine (AOE)](./cross-pillar/architecture-optimization-engine.md) - Multi-objective architecture optimization
22. ✅ [Architecture Compliance Score Engine (ACSE)](./cross-pillar/architecture-compliance-score-engine.md) - Unified architecture scoring
23. ✅ [Architecture Communication Hub (ACH)](./cross-pillar/architecture-communication-hub.md) - Cross-team architecture communication
27. ✅ [Dynamic Architecture Hotspot Detection Engine (DAHDE)](./cross-pillar/hotspot-detection-engine.md) - Automated hotspot detection
28. ✅ [Architecture Ranking Engine (ARE)](./cross-pillar/architecture-ranking-engine.md) - Multi-dimensional architecture scoring
30. ✅ [Multi-Simulation Orchestration Engine (MSOE)](./cross-pillar/multi-simulation-orchestration-engine.md) - Batch and parallel simulation orchestration
31. ✅ [Behavior Sensitivity Analysis Engine (BSAE)](./cross-pillar/behavior-sensitivity-analysis-engine.md) - Parameter sensitivity analysis
32. ✅ [Reinforcement Loop Analyzer Engine (RLAE)](./cross-pillar/reinforcement-loop-analyzer-engine.md) - Feedback loop detection and analysis
33. ✅ [Scenario Comparison Engine (SCE)](./cross-pillar/scenario-comparison-engine.md) - Architecture version and scenario comparison
34. ✅ [Architecture Value Realization Engine (AVRE)](./cross-pillar/architecture-value-realization-engine.md) - Business value quantification
35. ✅ [Architecture Impact Forecasting Engine (AIFE)](./cross-pillar/architecture-impact-forecasting-engine.md) - Predictive impact analysis
36. ✅ [Architecture Scenario Forecasting Engine (ASFE)](./cross-pillar/architecture-scenario-forecasting-engine.md) - Long-term scenario forecasting
39. ✅ [Autonomous Architecture Orchestration Engine (AAOE)](./cross-pillar/autonomous-architecture-orchestration-engine.md) - Autonomous execution automation
40. ✅ [Architecture Transformation Execution Engine (ATEX)](./cross-pillar/architecture-transformation-execution-engine.md) - Transformation planning and execution
41. ✅ [Architecture Modernization Recommendation Engine (AMRE)](./cross-pillar/architecture-modernization-recommendation-engine.md) - AI-driven modernization recommendations
42. ✅ [Risk Modeling & Impact Analysis Engine (RMIAE)](./cross-pillar/risk-modeling-engine.md) - Comprehensive risk analysis
43. ✅ [Architecture Debt Analyzer Engine (ADAE)](./cross-pillar/architecture-debt-analyzer-engine.md) - Debt tracking and remediation
44. ✅ [Architecture Fitness Function Engine (AFFE)](./cross-pillar/architecture-fitness-function-engine.md) - Continuous quality enforcement
45. ✅ [Architecture Governance Engine (AGE)](./cross-pillar/architecture-governance-engine.md) - Unified governance framework
46. ✅ [Architecture Governance & Policy Engine (AGPE)](./cross-pillar/architecture-governance-policy-engine.md) - Policy enforcement
47. ✅ [Architecture Benchmarking Engine (ABE)](./cross-pillar/architecture-benchmarking-engine.md) - External benchmarking
48. ✅ [Architecture Evolution Timeline Engine (AETE)](./cross-pillar/architecture-evolution-timeline-engine.md) - Git-integrated evolution tracking
49. ✅ [Architecture Refactoring Simulation Engine (ARSE)](./cross-pillar/architecture-refactoring-simulation-engine.md) - Refactoring impact simulation
50. ✅ [Template Engine](./core/template-engine.md) - Template search, filtering, and instantiation
51. ✅ [Auto-Layout Engine](./core/auto-layout-engine.md) - Automatic diagram layout
52. ✅ [Architecture Wizard Engine](./core/architecture-wizard-engine.md) - Guided creation flows
53. ✅ [Visual Diff Engine](./core/visual-diff-engine.md) - Structure and diagram diffing
54. ✅ [Undo/Redo Engine](./core/undo-redo-engine.md) - Unified history management
55. ✅ [Diagram Serializer Engine](./core/diagram-serializer-engine.md) - Diff-based DSL serialization
56. ✅ [Graph Engine](./core/graph-engine.md) - Graph operations and cycle detection
57. ✅ [Visual Validation Overlays](./core/visual-validation-overlays.md) - Live validation feedback on diagram
58. ✅ [Evolution Timeline Tracker](./core/evolution-timeline-tracker.md) - Git-powered architecture time-travel
59. ✅ [Reference Resolution Engine](./core/reference-resolution-engine.md) - Cross-module and cross-context reference resolution
60. ✅ [Drift Detector](./core/drift-detector.md) - Architecture → code → infrastructure drift detection
61. ✅ [DSL Parser Engine](./core/dsl-parser-engine.md) - Parses DSL text into AST using participle (handles all features)
62. ✅ [AST Transformer Engine](./core/ast-transformer-engine.md) - Transforms AST into typed IR models
63. ✅ [Visual Simulation Dashboard](./cross-pillar/visual-simulation-dashboard.md) - Interactive simulation visualization and architecture cockpit
64. ✅ [Behavior Simulator](./cross-pillar/behavior-simulator.md) - Stock-flow dynamics simulation and system behavior prediction
65. ✅ [Systems Thinking Compiler](./cross-pillar/systems-thinking-compiler.md) - Transpiles DSL v3 into simulation models
66. ✅ [Causal Graph Generator](./cross-pillar/causal-graph-generator.md) - Builds causal graphs with loops, edges, and weights
67. ✅ [Feedback Loop Detector](./cross-pillar/feedback-loop-detector.md) - Discovers reinforcing and balancing loops
68. ✅ [Architecture Evolution Simulator (MAES)](./cross-pillar/architecture-evolution-simulator.md) - Multi-system evolution simulation
69. ✅ [Multi-System Scenario Modeling Engine (MSME)](./cross-pillar/multi-scenario-modeling-engine.md) - Alternative architecture futures modeling
70. ✅ [Migration Planning Engine](./cross-pillar/migration-planning-engine.md) - Generate migration roadmap
71. ✅ [Emergent Outcome Predictor](./cross-pillar/emergent-outcome-predictor.md) - Detect tipping points
72. ✅ [Architecture Stability Score](./cross-pillar/architecture-stability-score.md) - Measures systemic fragility
73. ✅ [Drift Forecasting Engine](./cross-pillar/drift-forecasting-engine.md) - Predict future architecture drift
74. ✅ [Stress Simulation Engine](./reliability/stress-simulation-engine.md) - Load, failure, scaling tests
75. ✅ [Incident Propagation Simulator](./reliability/incident-propagation-simulator.md) - MTTR, cascade failures
76. ✅ [CI/CD Integration Engine](./operational-excellence/cicd-integration-engine.md) - Validate in pipelines
77. ✅ [Rollback Controller](./cross-pillar/rollback-controller.md) - Safe rollback plans
78. ✅ [Architecture Audit Engine](./cross-pillar/architecture-audit-engine.md) - Full compliance + decision history
79. ✅ [ADR Manager](./cross-pillar/adr-manager.md) - Lifecycle: propose → approve → supersede
80. ✅ [Tech Catalog](./cross-pillar/tech-catalog.md) - Approved/forbidden stacks
81. ✅ [Pattern Catalog](./cross-pillar/pattern-catalog.md) - CQRS, Saga, Event Sourcing, etc.
82. ✅ [Architecture Goals Engine](./cross-pillar/architecture-goals-engine.md) - Align architecture to business KPIs
83. ✅ [Architecture Advisor](./cross-pillar/architecture-advisor.md) - Recommendations, fixes, refactors
84. ✅ [Pattern Recommendation Engine](./cross-pillar/pattern-recommendation-engine.md) - Suggest best patterns
85. ✅ [Technology Fit Engine](./cross-pillar/technology-fit-engine.md) - Match tech to requirements
86. ✅ [Architecture Search Engine](./cross-pillar/architecture-search-engine.md) - Similarity search across archives
87. ✅ [Domain Modeling AI](./cross-pillar/domain-modeling-ai.md) - Generate bounded contexts
88. ✅ [Model Explanation Engine](./cross-pillar/model-explanation-engine.md) - Explain complex architectures
89. ✅ [Metrics Collection Engine](./operational-excellence/metrics-collection-engine.md) - Custom metrics definition
90. ✅ [Logging Aggregation Engine](./operational-excellence/logging-aggregation-engine.md) - Centralized logging
91. ✅ [Distributed Tracing Engine](./operational-excellence/distributed-tracing-engine.md) - Request tracing across services
92. ✅ [Alerting Engine](./operational-excellence/alerting-engine.md) - Alert definition and routing
93. ✅ [Dashboard Generation Engine](./operational-excellence/dashboard-generation-engine.md) - Auto-generate monitoring dashboards
94. ✅ [Runbook Engine](./operational-excellence/runbook-engine.md) - Operational procedure automation
95. ✅ [Deployment Strategy Engine](./operational-excellence/deployment-strategy-engine.md) - Blue-green, canary, rolling
96. ✅ [Incident Response Engine](./operational-excellence/incident-response-engine.md) - Automated incident handling
97. ✅ [Operations Automation Engine](./operational-excellence/operations-automation-engine.md) - Ops-as-code execution
98. ✅ [Pipeline Validation Engine](./operational-excellence/pipeline-validation-engine.md) - Architecture checks in CI/CD
99. ✅ [Deployment Validation Engine](./operational-excellence/deployment-validation-engine.md) - Pre-deployment checks
100. ✅ [Change Impact Analyzer](./cross-pillar/change-impact-analyzer.md) - Assess change impact
101. ✅ [Change Approval Engine](./cross-pillar/change-approval-engine.md) - Change request workflow
102. ✅ [Resilience Engine](./reliability/resilience-engine.md) - Full resilience DSL support
103. ✅ [Health Check Engine](./reliability/health-check-engine.md) - Liveness, readiness, startup probes
104. ✅ [Retry Policy Engine](./reliability/retry-policy-engine.md) - Exponential, linear, custom retries
105. ✅ [Circuit Breaker Engine](./reliability/circuit-breaker-engine.md) - Failure threshold management
106. ✅ [Bulkhead Engine](./reliability/bulkhead-engine.md) - Resource isolation
107. ✅ [Rate Limiting Engine](./reliability/rate-limiting-engine.md) - Request rate control
108. ✅ [Backpressure Engine](./reliability/backpressure-engine.md) - Flow control
109. ✅ [Degradation Engine](./reliability/degradation-engine.md) - Graceful degradation strategies
110. ✅ [Auto-healing Engine](./reliability/auto-healing-engine.md) - Automatic recovery
111. ✅ [Autoscaling Engine](./reliability/autoscaling-engine.md) - CPU, latency, event-driven scaling
112. ✅ [Failover Engine](./reliability/failover-engine.md) - Active-passive, active-active strategies
113. ✅ [Failure Scenario Engine](./reliability/failure-scenario-engine.md) - Failure mode modeling
114. ✅ [Chaos Engineering Engine](./reliability/chaos-engineering-engine.md) - Chaos testing automation
115. ✅ [Redundancy Engine](./reliability/redundancy-engine.md) - Multi-region, multi-AZ planning
116. ✅ [High Availability Engine](./reliability/high-availability-engine.md) - HA configuration validation
117. ✅ [Performance Engine](./performance-efficiency/performance-engine.md) - Full performance DSL support
118. ✅ [Latency Analysis Engine](./performance-efficiency/latency-analysis-engine.md) - P50, P90, P99 analysis
119. ✅ [Throughput Analysis Engine](./performance-efficiency/throughput-analysis-engine.md) - RPS, QPS, TPS analysis
120. ✅ [SLO/SLI Engine](./performance-efficiency/slo-sli-engine.md) - Service level objectives/indicators
121. ✅ [Performance Budget Engine](./performance-efficiency/performance-budget-engine.md) - Performance budget tracking
122. ✅ [Bottleneck Detection Engine](./performance-efficiency/bottleneck-detection-engine.md) - Identify performance bottlenecks
123. ✅ [Capacity Planning Engine](./performance-efficiency/capacity-planning-engine.md) - Resource capacity forecasting
124. ✅ [Workload Analysis Engine](./performance-efficiency/workload-analysis-engine.md) - Traffic pattern analysis
125. ✅ [Performance Optimization Engine](./performance-efficiency/performance-optimization-engine.md) - Query, algorithm optimization
126. ✅ [Caching Strategy Engine](./performance-efficiency/caching-strategy-engine.md) - Cache placement and strategy
127. ✅ [Resource Right-Sizing Engine](./performance-efficiency/resource-right-sizing-engine.md) - Optimal resource selection
128. ✅ [Scaling Strategy Engine](./performance-efficiency/scaling-strategy-engine.md) - Horizontal/vertical scaling
129. ✅ [Load Balancing Engine](./performance-efficiency/load-balancing-engine.md) - Load distribution strategies
130. ✅ [Resource Selection Engine](./performance-efficiency/resource-selection-engine.md) - Match tech to requirements
131. ✅ [Cost Analysis Engine](./cost-optimization/cost-analysis-engine.md) - Cloud cost predictions
132. ✅ [FinOps Engine](./cost-optimization/finops-engine.md) - Full cost & FinOps DSL support
133. ✅ [Budget Engine](./cost-optimization/budget-engine.md) - Budget definition and tracking
134. ✅ [Cost Allocation Engine](./cost-optimization/cost-allocation-engine.md) - Team, project cost allocation
135. ✅ [Cost Optimization Engine](./cost-optimization/cost-optimization-engine.md) - Cost reduction strategies
136. ✅ [Reserved Instance Engine](./cost-optimization/reserved-instance-engine.md) - RI planning and optimization
137. ✅ [Spot Instance Engine](./cost-optimization/spot-instance-engine.md) - Spot instance strategy
138. ✅ [Cost Forecasting Engine](./cost-optimization/cost-forecasting-engine.md) - Future cost prediction
139. ✅ [Sustainability Engine](./sustainability/sustainability-engine.md) - Full sustainability DSL support
140. ✅ [Carbon Accounting Engine](./sustainability/carbon-accounting-engine.md) - Carbon footprint calculation
141. ✅ [Resource Efficiency Engine](./sustainability/resource-efficiency-engine.md) - CPU, memory, storage efficiency
142. ✅ [Green Computing Engine](./sustainability/green-computing-engine.md) - Renewable energy, carbon-neutral
143. ✅ [Sustainability Goals Engine](./sustainability/sustainability-goals-engine.md) - Carbon reduction targets
144. ✅ [Idle Resource Cleanup Engine](./sustainability/idle-resource-cleanup-engine.md) - Automatic resource cleanup
145. ✅ [Data Lifecycle Engine](./sustainability/data-lifecycle-engine.md) - Hot/warm/cold data management
146. ✅ [Security Validation Engine](./security/security-validation-engine.md) - Security policy enforcement
147. ✅ [Compliance Engine](./security/compliance-engine.md) - Security + org policies
148. ✅ [IAM Engine](./security/iam-engine.md) - Identity and access management
149. ✅ [Network Security Engine](./security/network-security-engine.md) - VPC, security groups, firewalls
150. ✅ [Data Classification Engine](./security/data-classification-engine.md) - PII, PCI-DSS data handling
151. ✅ [Security Audit Engine](./security/security-audit-engine.md) - Security compliance auditing
152. ✅ [Vulnerability Detection Engine](./security/vulnerability-detection-engine.md) - Security vulnerability scanning
153. ✅ [Architecture Knowledge Graph Engine (AKGE)](./cross-pillar/architecture-knowledge-graph-engine-akge.md) - Full enterprise KG
154. ✅ [MCP Code Generation Engine](./cross-pillar/mcp-code-generation-engine.md) - MCP integration for code generation
155. ✅ [Risk Analysis Engine](./cross-pillar/risk-analysis-engine.md) - Risk blast radius analysis
156. ✅ [Basic Health Check Engine](./operational-excellence/basic-health-check-engine.md) - Health check validation
157. ✅ [Basic Metrics Engine](./operational-excellence/basic-metrics-engine.md) - Latency, error rate tracking
158. ✅ [Basic Logging Engine](./operational-excellence/basic-logging-engine.md) - Log format validation
159. ✅ [Basic Authentication Engine](./security/basic-authentication-engine.md) - Authentication type validation
160. ✅ [Basic Encryption Engine](./security/basic-encryption-engine.md) - Encryption requirement checks
161. ✅ [Basic Security Tags Engine](./security/basic-security-tags-engine.md) - Security tag validation
162. ✅ [Basic Retry Engine](./reliability/basic-retry-engine.md) - Retry policy validation
163. ✅ [Basic Timeout Engine](./reliability/basic-timeout-engine.md) - Timeout requirement checks
164. ✅ [Basic Circuit Breaker Engine](./reliability/basic-circuit-breaker-engine.md) - Circuit breaker configuration
165. ✅ [Basic Latency Engine](./performance-efficiency/basic-latency-engine.md) - Latency target validation
166. ✅ [Basic Throughput Engine](./performance-efficiency/basic-throughput-engine.md) - Throughput target checks
167. ✅ [Basic Scaling Engine](./performance-efficiency/basic-scaling-engine.md) - Min/max scaling validation
168. ✅ [Basic Cost Tracking Engine](./cost-optimization/basic-cost-tracking-engine.md) - Cost estimate validation
169. ✅ [Basic Cost Tags Engine](./cost-optimization/basic-cost-tags-engine.md) - Cost center tagging
170. ✅ [Basic Resource Efficiency Engine](./sustainability/basic-resource-efficiency-engine.md) - Resource efficiency validation
171. ✅ [Basic Carbon Tracking Engine](./sustainability/basic-carbon-tracking-engine.md) - Carbon footprint tracking

**Total: 171 engines fully documented**

## Remaining Engines

The conversation file contains detailed specifications for 100+ more engines. The structure is ready to extract additional engines as needed. See [Engines Overview](../engines.md) for the complete catalog of 147 engines.

## Engine Categories

### Core Engines
Foundational engines used across all pillars:
- DSL Parser Engine
- Model Composer
- Two-Way Sync Engine
- Validation Engine
- Auto-layout Engine
- And more...

### Pillar-Specific Engines
Engines specific to each Well-Architected pillar:
- **Operational Excellence**: Observability, CI/CD, incident response
- **Security**: Compliance, IAM, threat modeling
- **Reliability**: Resilience, chaos engineering, failure analysis
- **Performance Efficiency**: Performance analysis, optimization, capacity planning
- **Cost Optimization**: FinOps, cost modeling, optimization
- **Sustainability**: Carbon accounting, resource efficiency

### Cross-Pillar Engines
Engines that span multiple pillars:
- AI-Powered Architecture engines
- Systems Thinking & Simulation engines
- Evolution & Migration engines
- Governance & Knowledge engines

---

## How to Use

1. **Start with Core** - Understand foundational engines first
2. **Choose Your Pillars** - Focus on engines for relevant pillars
3. **Explore Cross-Pillar** - Advanced engines for enterprise use

---

*Each engine document includes: architecture, API, implementation plan, and integration details.*

