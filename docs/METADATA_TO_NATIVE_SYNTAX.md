# Metadata to Native Syntax Migration

## Executive Summary

This document proposes converting FAANG-level features from freeform metadata to **optional** native DSL syntax blocks. **All proposed syntax is fully compatible with existing Sruja DSL patterns** and follows the same grammar rules as existing optional blocks like `metadata`, `style`, `scale`, and `adr`.

**Key Principles**:
- ✅ **Optional by default** - Simple architectures don't need them
- ✅ **Contextual enforcement** - Rule engines can make mandatory when appropriate (like linter rules)
- ✅ **Progressive disclosure** - Beginners ignore them, advanced users use them
- ✅ **Empower, don't restrict** - Developers learn important concepts (SLOs, capacity, DR) when ready

**Critical Insight**: These features are **optional** (like `metadata`, `style`, `scale`), but rule engines can **contextually enforce** them when appropriate. This aligns with "empower, don't restrict" - developers learn important concepts while keeping simple architectures simple.

## Syntax Compatibility

✅ **All proposed syntax is compatible with existing patterns:**

- **Block syntax**: `keyword { ... }` - matches `metadata { ... }`, `style { ... }`
- **Nested blocks**: Same pattern as existing nested structures
- **Numeric values**: Matches `scale { min 3 max 10 }` pattern
- **String values**: All strings in quotes (consistent throughout DSL)
- **Arrays**: `[ "item1" "item2" ]` - matches `tags ["frontend", "backend"]`
- **Named blocks**: `failureMode ID "Title" { ... }` - matches `adr ADR001 "Title" { ... }`
- **No colons**: Uses `key "value"` pattern (like metadata/scale), not `key: "value"` (like style)

The parser already supports all these patterns, so implementation is straightforward.

## Problem Statement

Currently, FAANG-level features (SLOs, capacity planning, failure modes, etc.) are implemented using freeform metadata blocks:

```sruja
system ECommerce "E-Commerce Platform" {
  metadata {
    capacity "10M+ users, 1M+ requests/day"
    scaling "Horizontal Pod Autoscaling (HPA) on K8s"
    projectedGrowth "2x traffic expected in next 6 months"
  }
}
```

### Critical Issues with Metadata Approach

1. **No Syntax Error Detection**
   - Typo in key: `capcity` instead of `capacity` → No error, silently ignored
   - Wrong format: `capacity: 20` instead of `capacity "20 instances"` → Parses but wrong type
   - Missing quotes: `capacity 20 instances` → Parse error, but unclear message

2. **No Native Language Support**
   - No autocomplete for metadata keys
   - No type checking (strings only, no structured data)
   - No IDE features (go-to-definition, hover docs)
   - No validation at parse time

3. **Rule Engines Break Silently**
   - `sys.MetaString("capacity")` returns `false` if key is misspelled
   - Rule engines show "Not specified" without explaining why
   - No way to know if metadata was intentionally omitted or accidentally misspelled
   - Rule engines can't validate metadata structure/format

4. **No Discoverability**
   - Users don't know what metadata keys are available
   - No documentation in language itself
   - No way to enforce required vs optional metadata

## Solution: Native DSL Syntax

Convert metadata-based features to native DSL syntax blocks that are part of the grammar.

### Benefits

1. **Syntax Errors Caught at Parse Time**
   - Typo in keyword: `capcity { ... }` → Parse error with clear message
   - Missing braces: `capacity {` → Parse error
   - Wrong structure: Type checking at parse time

2. **Native Language Support**
   - Autocomplete for keywords (`slo`, `capacity`, `failureMode`, etc.)
   - Type checking (numbers, strings, structured blocks)
   - IDE features (hover docs, go-to-definition)
   - Validation rules can reference AST nodes directly

3. **Clear Error Messages**
   - Parse errors: "Expected 'target' after 'availability' at line 5"
   - Validation errors: "SLO target '99.9%' must be between 0% and 100%"
   - Missing required fields: "SLO block requires 'availability' or 'latency'"

4. **Discoverability**
   - Language documentation shows all available blocks
   - IDE autocomplete shows available options
   - Type system enforces structure

5. **Optional with Contextual Enforcement**
   - Features are optional by default (simple architectures stay simple)
   - Rule engines can suggest when appropriate (info-level)
   - Rule engines can enforce when critical (warning/error-level)
   - Configurable per team/organization (like linter rules)

## Proposed DSL Syntax

### 1. SLO Block

**Current (Metadata):**
```sruja
metadata {
  slo_availability "99.9%"
  slo_latency_p95 "200ms"
  slo_error_rate "0.1%"
}
```

**Proposed (Native):**
```sruja
slo {
  availability {
    target "99.9%"
    window "30 days"
    current "99.95%"
  }
  latency {
    p95 "200ms"
    p99 "500ms"
    window "7 days"
    current {
      p95 "180ms"
      p99 "420ms"
    }
  }
  errorRate {
    target "0.1%"
    window "7 days"
    current "0.08%"
  }
  throughput {
    target "10000 req/s"
    window "peak hour"
    current "8500 req/s"
  }
}
```

**Parse-Time Validation:**
- `target` must be valid percentage or duration
- `window` must be valid time period
- `current` optional but validated if present

**Syntax Compatibility:**
- Follows existing block pattern: `keyword { ... }`
- Nested blocks use same pattern: `availability { target "99.9%" }`
- No colons needed (consistent with `scale { min 3 }` pattern)
- Arrays use bracket syntax: `[ "item1" "item2" ]` (consistent with `tags ["frontend", "backend"]`)

### 2. Capacity Block

**Current (Metadata):**
```sruja
metadata {
  capacity "20 instances, 4 vCPU, 8GB RAM each"
  scaling "Auto-scale 10-50 instances"
  projectedGrowth "2x traffic in 6 months"
  bottlenecks "Database write IOPS at 80%"
}
```

**Proposed (Native):**
```sruja
capacity {
  current {
    apiServers 20
    instanceType "4 vCPU, 8GB RAM"
    database "db.r5.2xlarge"
    cache "redis-16gb-3nodes"
  }
  scaling {
    horizontal "10-50 instances"
    vertical "db.r5.2xlarge to db.r5.4xlarge"
    readReplicas 3
  }
  projectedGrowth {
    "6 months" "2x traffic → 40 API instances, db.r5.4xlarge"
    "12 months" "5x traffic → 100 API instances, db.r5.8xlarge"
  }
  bottlenecks [
    "Database write capacity at 80% (monitor closely)"
    "Cache memory usage at 70% (healthy)"
  ]
}
```

**Parse-Time Validation:**
- Numbers must be positive integers
- Instance types validated against known patterns
- Time periods validated

**Syntax Compatibility:**
- Follows `scale { min 3 max 10 }` pattern for numeric values
- Nested blocks consistent with existing patterns
- String values in quotes (consistent with all existing syntax)

### 3. Failure Mode Block

**Current (Metadata):**
```sruja
metadata {
  failureMode_platform_impact "Complete service outage"
  failureMode_platform_detection "Health check failures >3 consecutive"
  failureMode_platform_mitigation "Auto-scale, circuit breakers"
  failureMode_platform_rto "15 minutes"
  failureMode_platform_rpo "5 minutes"
}
```

**Proposed (Native):**
```sruja
failureMode PlatformFailure "Platform Failure" {
  impact "Complete service outage, 100% of users affected"
  detection [
    "Health check failures (>3 consecutive)"
    "Error rate spike (>5% for 1 minute)"
    "Alert: PagerDuty escalation"
  ]
  mitigation [
    "Auto-scaling triggers (scale up 2x)"
    "Circuit breakers activate"
    "Read-only mode enabled"
  ]
  recovery {
    rto "15 minutes"
    rpo "5 minutes"
    steps [
      "Identify root cause"
      "Rollback if needed"
      "Scale up"
      "Verify"
    ]
  }
  fallback [
    "Static product catalog (CDN cached)"
    "Queue orders for later processing"
    "Display maintenance message"
  ]
}
```

**Note**: Follows the same pattern as `adr ADR001 "Title" { ... }` - ID, optional quoted label, then body.

**Parse-Time Validation:**
- `rto` and `rpo` must be valid durations
- `steps` must be non-empty array
- `impact` required

**Syntax Compatibility:**
- Follows `adr ADR001 "Title" { ... }` pattern (ID, optional quoted label, body)
- Nested `recovery { rto "15 minutes" }` follows existing block patterns
- Arrays use bracket syntax consistent with existing DSL

### 4. Disaster Recovery Block

**Current (Metadata):**
```sruja
metadata {
  dr_primaryRegion "us-east-1"
  dr_drRegion "us-west-2"
  dr_rto "30 minutes"
  dr_rpo "5 minutes"
  dr_backupStrategy "Database: every 6 hours, 30 days retention"
}
```

**Proposed (Native):**
```sruja
disasterRecovery {
  primaryRegion "us-east-1"
  drRegion "us-west-2"
  rto "30 minutes"
  rpo "5 minutes"
  backupStrategy {
    database "every 6 hours, 30 days retention"
    applicationState "every 1 hour"
    configuration "GitOps, versioned in Git"
  }
  failoverProcedure [
    "Detection: Automated health checks detect primary region failure"
    "Decision: On-call engineer confirms (manual approval)"
    "Failover: DNS cutover to DR region (Route53)"
    "Verification: Smoke tests confirm service health"
    "Communication: Status page update, customer notification"
  ]
  recoveryTesting {
    frequency "Quarterly"
    lastTest "2024-01-15"
    results "RTO achieved (28 minutes), RPO within target (4 minutes)"
  }
}
```

### 5. Monitoring & Observability Block

**Current (Metadata):**
```sruja
metadata {
  metrics "Prometheus (custom metrics)"
  dashboards "System Health, Service Health, Business Metrics"
  alerting "PagerDuty for P0/P1"
  logging "CloudWatch Logs (7 days retention)"
  tracing "AWS X-Ray, 1% sample rate"
}
```

**Proposed (Native):**
```sruja
observability {
  metrics {
    application "Prometheus (custom metrics)"
    infrastructure "CloudWatch (AWS native)"
    business "Custom dashboards (Grafana)"
  }
  dashboards [
    "System Health: Overall uptime, error rates, latency"
    "Service Health: Per-service metrics (API, DB, Cache)"
    "Business Metrics: Orders/day, revenue, conversion rate"
    "Cost Dashboard: AWS costs by service, cost per transaction"
  ]
  alerting {
    critical "PagerDuty (on-call rotation)"
    warning "Slack (#alerts channel)"
    info "Email digest (daily)"
  }
  logging {
    application "CloudWatch Logs (7 days retention)"
    access "S3 (90 days retention)"
    audit "S3 (7 years retention for compliance)"
  }
  tracing {
    tool "AWS X-Ray"
    sampleRate "1% (production), 100% (staging)"
    retention "7 days"
  }
}
```

### 6. Compliance Block

**Current (Metadata):**
```sruja
metadata {
  pciDss "Level 1"
  soc2 "Type II"
  gdpr "compliant"
  iso27001 "in progress"
}
```

**Proposed (Native):**
```sruja
compliance {
  pciDss {
    level "Level 1"
    status "certified"
    via "Stripe"
  }
  soc2 {
    type "Type II"
    status "certified"
    via "AWS infrastructure"
  }
  gdpr {
    status "compliant"
    note "Data processing agreements in place"
  }
  iso27001 {
    status "in progress"
    expected "Q2 2024"
  }
  hipaa {
    status "not applicable"
    reason "No PHI"
  }
}
```

### 7. Cost Analysis Block

**Current (Metadata):**
```sruja
metadata {
  monthlyCost "$14,300/month"
  cost_compute "$8,000"
  cost_database "$2,500"
  costPerTransaction "$0.0143"
}
```

**Proposed (Native):**
```sruja
cost {
  monthly {
    compute "$8,000 (80 instances × $100/month)"
    database "$2,500 (db.r5.2xlarge)"
    cache "$1,200 (Redis cluster)"
    messageQueue "$800 (Kafka cluster)"
    storage "$500 (100TB)"
    cdn "$1,000 (500GB transfer)"
    monitoring "$300"
    total "$14,300"
  }
  perTransaction {
    average "$0.0143"
    breakdown {
      compute "57%"
      database "17%"
      cache "8%"
      other "18%"
    }
  }
  optimization {
    reservedInstances "40% savings (1-year term)"
    spotInstances "70% savings for non-critical workloads"
    s3Lifecycle "80% savings (move to Glacier)"
    projectedSavings "$3,000/month (21% reduction)"
  }
}
```

## Syntax Compatibility Analysis

### ✅ Compatible Patterns

All proposed syntax follows existing Sruja DSL patterns:

1. **Block Syntax**: `keyword { ... }` - matches `metadata { ... }`, `style { ... }`, `scale { ... }`
2. **Nested Blocks**: `slo { availability { target "99.9%" } }` - same pattern as nested structures
3. **Numeric Values**: `capacity { current { apiServers 20 } }` - matches `scale { min 3 max 10 }`
4. **String Values**: All strings in quotes - consistent with existing syntax
5. **Arrays**: `[ "item1" "item2" ]` - matches `tags ["frontend", "backend"]`
6. **Named Blocks**: `failureMode PlatformFailure "Title" { ... }` - matches `adr ADR001 "Title" { ... }`
7. **Optional Fields**: All fields optional except where noted - matches existing flexibility

### ⚠️ Considerations

1. **Colons**: Existing syntax is inconsistent:
   - `metadata { key "value" }` - no colon
   - `style { key: "value" }` - with colon
   - `properties { "key": "value" }` - quoted key with colon
   
   **Decision**: Use no-colon pattern (like metadata/scale) for consistency and simplicity.

2. **Nested Structures**: Some blocks have nested structures (like `recovery { rto "15 minutes" }`). This is compatible as the parser already handles nested blocks.

3. **Arrays**: Using bracket syntax `[ "item1" "item2" ]` which matches existing `tags` syntax.

## Optional Features Model

### Progressive Disclosure

**Beginner** (10 minutes to productivity):
```sruja
architecture "My App" {
  system App {
    container Web "Web Server"
    datastore DB "Database"
  }
}
```
✅ No FAANG features needed - simple and productive

**Intermediate** (When ready to learn):
```sruja
architecture "My App" {
  system App {
    container Web "Web Server"
    datastore DB "Database"
    // Optional: Add SLOs when you have SLA requirements
    slo {
      availability { target "99.9%" }
    }
  }
}
```
✅ FAANG features available but optional - learn when needed

**Advanced** (Enterprise/FAANG):
```sruja
architecture "Production System" {
  system App {
    container Web "Web Server"
    datastore DB "Database"
    slo { ... }
    capacity { ... }
    failureMode { ... }
    disasterRecovery { ... }
    observability { ... }
    compliance { ... }
    cost { ... }
  }
}
```
✅ All features available - comprehensive documentation

### Contextual Enforcement via Rule Engines

Rule engines can enforce these features contextually, similar to linter rules:

**Example Rule: SLO Recommendation**
```go
func (r *SLORule) Validate(program *language.Program) []diagnostics.Diagnostic {
  for _, sys := range arch.Systems {
    hasSLAReq := false
    for _, req := range sys.Requirements {
      if req.Type != nil && *req.Type == "performance" {
        if strings.Contains(strings.ToLower(*req.Description), "sla") ||
           strings.Contains(strings.ToLower(*req.Description), "availability") {
          hasSLAReq = true
          break
        }
      }
    }
    
    if hasSLAReq && sys.SLO == nil {
      return []diagnostics.Diagnostic{
        {
          Severity: diagnostics.SeverityInfo,  // Info-level suggestion
          Message: "System has SLA requirements but no SLO block defined. Consider adding 'slo { ... }' block.",
          Suggestions: []string{"Add SLO block to document service level objectives"},
        },
      }
    }
  }
}
```

**Example Rule: Failure Mode Enforcement**
```go
func (r *FailureModeRule) Validate(program *language.Program) []diagnostics.Diagnostic {
  for _, sys := range arch.Systems {
    isCritical := hasMetadata(sys, "tier", "critical") || 
                  hasMetadata(sys, "tags", "production")
    if isCritical && len(sys.FailureModes) == 0 {
      return []diagnostics.Diagnostic{
        {
          Severity: diagnostics.SeverityWarning,  // Warning-level for best practices
          Message: "Critical system has no failure modes defined. Consider adding 'failureMode { ... }' blocks.",
        },
      }
    }
  }
}
```

**Rule Engine Configuration** (like ESLint, golangci-lint):
```yaml
# .sruja-lint.yml
rules:
  # Make SLO mandatory if SLA requirement exists
  require-slo-for-sla:
    enabled: true
    severity: info  # Info for learning, warning for best practices, error for strict enforcement
    
  # Make capacity planning mandatory if scaling is configured
  require-capacity-for-scaling:
    enabled: true
    severity: info
    
  # Make failure modes mandatory for critical systems
  require-failure-modes-for-critical:
    enabled: true
    severity: warning  # Best practice enforcement
    
  # Make compliance mandatory for sensitive data
  require-compliance-for-sensitive:
    enabled: true
    severity: error  # Stricter for compliance/regulatory requirements
```

**Team/Organization Rules**:
- **Startup**: All rules disabled (keep it simple)
- **Mid-size**: Info-level suggestions (learn as you go)
- **Enterprise**: Warning-level enforcement (best practices)
- **FAANG**: Error-level enforcement (compliance required)

## Migration Strategy

### Phase 1: Add Native Syntax (Backward Compatible)

1. **Extend Parser Grammar**
   - Add new AST node types for each block type
   - Parse both old metadata and new native syntax
   - Keep metadata parsing for backward compatibility

2. **Update AST Structures**
   - Add `SLOBlock`, `CapacityBlock`, `FailureModeBlock`, etc. to AST
   - Keep `Metadata` field for backward compatibility
   - Add helper methods to extract from either source

3. **Update Exporters**
   - Markdown exporter checks for native blocks first, falls back to metadata
   - JSON exporter includes both formats during transition

### Phase 2: Validation Rules

1. **Parse-Time Validation**
   - Validate SLO targets are valid percentages/durations
   - Validate capacity numbers are positive
   - Validate RTO/RPO are valid durations
   - Validate compliance status values

2. **Semantic Validation**
   - Warn if native blocks exist but metadata also has same info (duplicate)
   - Suggest migration from metadata to native syntax
   - Validate cross-references (e.g., failure mode references to systems)

3. **Contextual Enforcement Rules** (New)
   - **SLO Rule**: Suggest SLO block if system has SLA requirements
   - **Capacity Rule**: Suggest capacity block if scaling is configured
   - **Failure Mode Rule**: Suggest failure modes for critical systems
   - **Compliance Rule**: Suggest compliance block for sensitive data
   - **Disaster Recovery Rule**: Suggest DR block for production systems
   - **Observability Rule**: Suggest observability block for production systems
   - **Cost Rule**: Suggest cost block for cost-conscious organizations
   
   **Severity Levels**:
   - `info`: Learning/suggestion (default for most rules)
   - `warning`: Best practice enforcement
   - `error`: Mandatory for compliance/regulatory requirements

### Phase 3: Deprecation

1. **Deprecation Warnings**
   - Warn when metadata keys are used that have native syntax equivalents
   - Provide migration suggestions
   - Document migration path

2. **Remove Metadata Support (Future)**
   - After sufficient migration period, remove metadata parsing for these keys
   - Keep metadata for truly freeform/extensible use cases

## Implementation Plan

### Step 1: Parser Extensions

**Files to Modify:**
- `pkg/language/ast_core.go` - Add new block types
- `pkg/language/parser.go` - Add grammar rules
- `pkg/language/ast_elements.go` - Add blocks to System/Container/etc.

**Example AST Addition:**
```go
type SLOBlock struct {
    Pos          lexer.Position
    Availability *SLOAvailability `parser:"( 'availability' '{' @@ '}' )?"`
    Latency      *SLOLatency       `parser:"( 'latency' '{' @@ '}' )?"`
    ErrorRate    *SLOErrorRate      `parser:"( 'errorRate' '{' @@ '}' )?"`
    Throughput   *SLOThroughput     `parser:"( 'throughput' '{' @@ '}' )?"`
}

type SLOAvailability struct {
    Target  string  `parser:"'target' @String"`
    Window  string  `parser:"'window' @String"`
    Current *string `parser:"( 'current' @String )?"`
}
```

### Step 2: Validation Rules

**New Validation Rules:**
- `SLOValidationRule` - Validates SLO targets, windows, formats
- `CapacityValidationRule` - Validates capacity numbers, scaling ranges
- `FailureModeValidationRule` - Validates RTO/RPO formats, required fields

**New Contextual Enforcement Rules:**
- `SLOEnforcementRule` - Suggests SLO block when SLA requirements exist
- `CapacityEnforcementRule` - Suggests capacity block when scaling configured
- `FailureModeEnforcementRule` - Suggests failure modes for critical systems
- `ComplianceEnforcementRule` - Suggests compliance block for sensitive data
- `DREnforcementRule` - Suggests DR block for production systems
- `ObservabilityEnforcementRule` - Suggests observability block for production
- `CostEnforcementRule` - Suggests cost block when cost matters

**Files to Create:**
- `pkg/engine/slo_rule.go`
- `pkg/engine/capacity_rule.go`
- `pkg/engine/failure_mode_rule.go`
- `pkg/engine/slo_enforcement_rule.go`
- `pkg/engine/capacity_enforcement_rule.go`
- `pkg/engine/failure_mode_enforcement_rule.go`
- `pkg/engine/compliance_enforcement_rule.go`

### Step 3: Export Updates

**Files to Modify:**
- `pkg/export/markdown/faang_helpers.go` - Check native blocks first
- `pkg/export/json/json.go` - Export native blocks

**Migration Helper:**
```go
func (sys *System) GetSLO() *SLOBlock {
    if sys.SLO != nil {
        return sys.SLO  // Native syntax
    }
    // Fallback to metadata extraction
    return extractSLOFromMetadata(sys.Metadata)
}
```

## Benefits Summary

1. **Syntax Errors Caught Early**
   - Parse errors with clear messages
   - Type validation at parse time
   - No silent failures

2. **Better Developer Experience**
   - Autocomplete for all keywords
   - IDE hover documentation
   - Type checking and validation

3. **Clear Error Messages**
   - Rule engines can reference AST nodes
   - Validation errors point to exact location
   - Missing required fields clearly identified

4. **Optional by Default**
   - Simple architectures stay simple (no FAANG features needed)
   - Beginners can ignore them completely
   - Advanced users can use them when ready

5. **Contextual Enforcement**
   - Rule engines suggest when appropriate (info-level)
   - Rule engines enforce when critical (warning/error-level)
   - Configurable per team/organization
   - Like linter rules - guide, don't force

6. **Progressive Learning**
   - Developers learn important concepts (SLOs, capacity, DR) when ready
   - Rule engines guide learning ("consider adding SLO block")
   - Aligns with "empower, don't restrict" philosophy

7. **Maintainability**
   - Language evolves with structured syntax
   - Easier to add new features
   - Better tooling support

8. **Backward Compatibility**
   - Metadata still works during transition
   - Gradual migration path
   - No breaking changes

## Design Philosophy Alignment

### ✅ Aligns with Core Principles

1. **"Start simple, stay simple"**
   - ✅ Features are optional - beginners don't need them
   - ✅ Simple architectures stay simple
   - ✅ 10-minute productivity goal maintained

2. **"Empower, don't restrict"**
   - ✅ Developers learn important concepts (SLOs, capacity, DR)
   - ✅ Rule engines guide, don't force
   - ✅ Progressive disclosure model

3. **"Approachability first"**
   - ✅ Simple by default, advanced when needed
   - ✅ Rule engines suggest when appropriate
   - ✅ No complexity for simple use cases

4. **"Prevent over-engineering"**
   - ✅ Optional by default prevents forcing complexity
   - ✅ Rule engines enforce contextually (only when needed)
   - ✅ Simple designs easier than complex ones

5. **"Less is more"**
   - ✅ Optional features don't add complexity for simple use cases
   - ✅ Rule engines make mandatory only when appropriate
   - ✅ Language grows with user needs

### Comparison with Existing Optional Features

Sruja already has many **optional** features that follow the same pattern:

| Feature | Required? | When Used | Pattern |
|---------|-----------|-----------|---------|
| `metadata` | No | When custom data needed | Optional |
| `style` | No | When custom styling needed | Optional |
| `scale` | No | When scaling configuration needed | Optional |
| `requirement` | No | When documenting requirements | Optional |
| `adr` | No | When documenting decisions | Optional |
| **`slo`** | **No** | **When SLA requirements exist** | **Optional** |
| **`capacity`** | **No** | **When capacity planning needed** | **Optional** |
| **`failureMode`** | **No** | **When failure handling needed** | **Optional** |

**Pattern**: All are **optional** - simple architectures don't need them, but they're available when needed.

## Next Steps

1. **Design Review**: Review proposed syntax with team
2. **Parser Implementation**: Start with SLO block as proof of concept
3. **Validation Rules**: Implement validation for native blocks
4. **Enforcement Rules**: Implement contextual enforcement rules (info/warning/error levels)
5. **Rule Engine Configuration**: Add `.sruja-lint.yml` support for rule configuration
6. **Export Updates**: Update exporters to use native blocks
7. **Documentation**: Update language spec with new syntax
8. **Migration Tools**: Create tool to migrate metadata to native syntax
