# Drift Detector

**Status**: Core Engine  
**Pillars**: Core (Drift Detection)

[← Back to Engines](../README.md)

## Overview

The Drift Detector detects architecture → code → infrastructure drift, comparing desired state (architecture model) with actual state (code/infrastructure).

**This uses "desired state vs. actual state" for architecture drift detection.**

## Purpose

The Drift Detector:

- ✅ Detects architecture → code drift
- ✅ Detects architecture → infrastructure drift
- ✅ Compares desired vs. actual state
- ✅ Identifies missing components
- ✅ Identifies changed components
- ✅ Identifies deleted components
- ✅ Reports drift violations

## Drift Types

### Architecture → Code Drift

Detects:

- Missing methods in code
- Changed method signatures
- Deleted DTO fields
- New required dependencies
- Missing components
- Changed component structure

### Architecture → Infrastructure Drift

Detects:

- Missing infrastructure resources
- Changed resource configurations
- Deleted resources
- New resources not in architecture
- Configuration mismatches

## Code Drift Detection

Compare:

```
generated code 
vs
existing code
```

Detect:

- Missing methods
- Changed method signature
- Deleted DTO fields
- New required dependencies

Example output:

```json
{
  "drift": [
    {
      "component": "PaymentService",
      "issue": "Missing method: refund()"
    }
  ]
}
```

## Architecture Drift Detection

Compare:

```
architecture model
vs
actual system state
```

Detect:

- Cross-domain creep
- Increasing coupling
- Unstable components (high churn)
- Domain boundary erosion
- Architecture smells growing

Drift score:

```
DriftScore = normalize(component_changes + domain_violations + coupling_delta)
```

## Detection Methods

### Static Analysis
- Code structure analysis
- Dependency analysis
- Component detection

### Runtime Analysis
- Telemetry comparison
- Dependency mapping
- Component discovery

### Infrastructure Analysis
- Resource discovery
- Configuration comparison
- State comparison

## Integration Points

### Architecture-Time Observability Engine (ATOE)
- Runtime telemetry
- Dependency mapping
- Component discovery

### Code Generation Engine
- Generated code comparison
- Signature comparison
- Structure comparison

### Validation Engine
- Boundary validation
- Rule violations
- Constraint violations

## MCP API

```
drift.detect(model, code)
drift.code(model, codebase)
drift.infrastructure(model, state)
drift.report()
```

## Strategic Value

The Drift Detector provides:

- ✅ Architecture-code sync validation
- ✅ Infrastructure compliance
- ✅ Drift early warning
- ✅ Compliance monitoring
- ✅ Change tracking

**This is critical for maintaining architecture-code consistency.**

## Implementation Status

✅ Architecture designed  
✅ Drift types specified  
✅ Detection methods defined  
📋 Implementation in progress

---

*The Drift Detector detects architecture → code → infrastructure drift.*

