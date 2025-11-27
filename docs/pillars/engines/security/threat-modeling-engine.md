# Architecture Threat Modeling Engine (ATME)

**Status**: Advanced Engine  
**Pillars**: Security

[← Back to Engines](../README.md)

## Overview

The Architecture Threat Modeling Engine (ATME) provides security-by-design: full STRIDE/LINDDUN threat modeling, risk scoring, attack graph generation, compensating controls, simulation & governance integration.

**ATME turns your product into a security-first architectural workspace.**

## Purpose

The Architecture Threat Modeling Engine (ATME):

- ✅ Identifies threats and vulnerabilities
- ✅ Generates STRIDE & LINDDUN analyses
- ✅ Builds attack graphs
- ✅ Computes risk score per component
- ✅ Simulates attacker movement
- ✅ Suggests countermeasures
- ✅ Enforces security policies
- ✅ Integrates with governance engine
- ✅ Generates compliance-ready reports
- ✅ Visualizes threats directly on diagrams

**This makes architecture secure by construction.**

## Threat Modeling Frameworks Supported

### STRIDE (Microsoft)
- Spoofing
- Tampering
- Repudiation
- Information Disclosure
- Denial of Service
- Elevation of Privilege

### LINDDUN Privacy Analysis
- Linkability
- Identifiability
- Non-repudiation
- Detectability
- Disclosure of Information
- Unawareness
- Non-compliance

### MITRE ATT&CK (Mappings)
- TTP patterns
- Known exploit paths

### OWASP ASVS / Cloud Security Rules
- API security
- auth/z
- secrets
- best practices

## Input Sources

ATME uses *everything* in the platform:

### From Architecture Model
- services
- edges
- trust boundaries
- domains
- data classifications

### From Knowledge Graph
- semantic similarity
- component types
- data flows

### From Governance Engine
- violations
- encryption rules
- domain rules

### From Simulation Engine
- failure propagation
- load scenarios
- DoS exposure

### From System Thinking Models
- unintended reinforcing loops in security

### From IAA
- auto-suggested fixes
- architecture refactor proposals

## Outputs

### Threat Lists
Per component, per domain, per data flow.

### Attack Graphs
Full graph of attacker movement possibilities.

### Threat Scores
CVSS-like rating:
```
low | medium | high | critical
```

### Remediation Suggestions
Auto-generated based on threat category + context.

### Security Coverage Metrics
- encryption coverage
- data classification coverage
- auth coverage
- domain boundary enforcement

### Compliance Reports
- SOC2
- GDPR
- HIPAA
- PCI-DSS

### Architecture Diagram Overlays
Visual red/yellow indicators.

### Security Test Cases
Generated automatically for later integration with DevSecOps.

## Architecture

```
ThreatModelingEngine
 ├── DataFlowAnalyzer
 ├── TrustBoundaryDetector
 ├── STRIDEAssessor
 ├── LINDDUNAssessor
 ├── AttackGraphBuilder
 ├── ExposureCalculator
 ├── RiskScorer
 ├── PatternMatcher (MITRE mappings)
 ├── RemediationGenerator
 ├── SecurityPolicyEnforcer
 ├── ThreatOverlayRenderer
 ├── ComplianceReportGenerator
 └── MCP Interface
```

## Threat Modeling Process

### Step 1 — Identify Assets
- user identities
- secrets
- data stores
- sensitive flows
- critical flows

### Step 2 — Identify Trust Boundaries
Automatically derived from:

- domain boundaries
- network boundaries
- privilege tiers
- authentication boundaries

### Step 3 — Apply STRIDE
For each component and data flow:

- Spoofing: Can identity be forged?
- Tampering: Can data be modified?
- Repudiation: Can actions be denied?
- Information Disclosure: Can data leak?
- Denial of Service: Can service be disrupted?
- Elevation of Privilege: Can privileges be escalated?

### Step 4 — Build Attack Graph
Shows all possible attack paths through the system.

### Step 5 — Calculate Risk Scores
Combines:

- likelihood
- impact
- exploitability
- business value at risk

### Step 6 — Generate Remediations
For each threat:

- security controls
- architecture changes
- policy updates
- monitoring recommendations

## MCP API

```
threat.analyze(model)
threat.stride(component)
threat.attackGraph()
threat.riskScore(component)
threat.remediations(threatId)
threat.complianceReport(standard)
threat.visualize()
```

## UI Features

### Threat Overlay
Visual indicators on architecture diagram:
- 🔴 Critical threats
- 🟠 High threats
- 🟡 Medium threats
- 🟢 Low threats

### Attack Graph Visualization
Interactive graph showing attack paths.

### Threat Dashboard
List of all threats with scores and status.

### Remediation Panel
Suggested fixes for each threat.

### Compliance Reports
Export-ready reports for audits.

## Implementation Status

✅ Architecture designed  
✅ Threat frameworks specified  
✅ Attack graph algorithm defined  
📋 Implementation in progress

---

*ATME makes architecture secure by construction, integrating threat modeling directly into the design process.*

