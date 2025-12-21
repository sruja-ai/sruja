# Sruja Architecture Strategy

> **Sruja is the compiler that validates. LikeC4 is the runtime that renders.**

This document defines Sruja's strategic positioning as an **AI-Native Architecture Platform** and the technical decisions that enable it.

---

## Executive Summary

Sruja connects **Intent** (DSL) with **Implementation** (Code) via **AI**. Unlike tools that only draw diagrams or list services, Sruja enforces architecture as the source of truth for AI coding assistants, CI/CD pipelines, and secure infrastructure.

| Feature | Structurizr | Backstage | Sourcegraph | **Sruja** |
|---------|-------------|-----------|-------------|-----------|
| C4 Diagrams | ✅ | ❌ | ❌ | ✅ |
| Code Linting | ❌ | ❌ | ❌ | ✅ |
| AI Context Export | ❌ | ❌ | ✅ (Internal) | ✅ (Exportable) |
| Infrastructure Policy | ❌ | ❌ | ❌ | ✅ |

---

## Competitive Landscape

### Diagram-as-Code (Direct Rivals)

| Tool | Strength | Gap |
|------|----------|-----|
| **Structurizr** | Industry standard for C4. Massive ecosystem | Static documentation only. No AI, no infra generation |
| **LikeC4** | Beautiful React diagrams, LSP, typed DSL | Visualization focus. No governance enforcement |

### Visual & SaaS

| Tool | Strength | Gap |
|------|----------|-----|
| **IcePanel** | Best-in-class zooming UI | Proprietary. Can't run in CI to lint code |
| **Eraser.io** | DiagramGPT generates from text | Drawing tool, not modeling. No governance |

### Service Catalogs

| Tool | Strength | Gap |
|------|----------|-----|
| **Backstage** | Open-source Developer Portal standard | Lists things, not how they connect. Sruja complements it |
| **Cortex/OpsLevel** | Production readiness scorecards | Operations focus, not architecture design |

### Code Intelligence (Emerging Threat)

| Tool | Strength | Risk |
|------|----------|------|
| **Sourcegraph** | Indexes all code with SCIP protocol | Could add C4 views on their graph |
| **Cursor** | Internal codebase mapping | Closed source. Sruja gives control back to architects |

---

## Strategic Decision: LikeC4 for Rendering

**Adopt LikeC4 for layout and rendering. Keep Sruja DSL and Go parser.**

### Why LikeC4

| Capability | Benefit |
|------------|---------|
| Layout Engine | `layoutedModel()` returns X/Y coordinates—Sugiyama solved |
| MIT License | Embed in commercial tools without legal issues |
| React Component | `@likec4/diagram`—no manual SVG |
| MCP Server | Built-in Claude/Cursor integration |

### Why Keep Sruja Go Parser

| Feature | LikeC4 DSL | Sruja DSL | Winner |
|---------|------------|-----------|--------|
| Policies | `metadata { policy: "TLS" }` | `policy "TLS" { ... }` | **Sruja** (first-class) |
| SLOs | `metadata { slo: "99.9%" }` | `slo { availability "99.9%" }` | **Sruja** (validated) |
| Contracts | Generic JSON | `contract "OrderAPI" { ... }` | **Sruja** (typed) |
| CI/CD Speed | Node.js startup | **Go native (instant)** | **Sruja** |

### Enterprise Validation

| Scenario | LikeC4 | Sruja |
|----------|--------|-------|
| Missing `owner` field | ✅ Valid | ❌ Policy violation |
| No SLO on critical service | ✅ Silent | ❌ Warning |
| Circular dependency | ✅ Renders | ❌ Error: Cycle detected |

---

## Architecture

```mermaid
flowchart LR
    A[architecture.sruja] --> B[Go Parser]
    B --> C{Validate}
    C -->|Pass| D[Sruja AST]
    C -->|Fail| E[Policy Error]
    D --> F[LikeC4 JSON Export]
    F --> G[@likec4/diagram]
```

### Sruja Core (Keep)

| Component | Location | Purpose |
|-----------|----------|---------|
| Go Parser | `pkg/language/` | DSL parsing, AST generation |
| Validation Engine | `pkg/engine/` | Policy enforcement, cycle detection |
| LSP Server | `pkg/lsp/` | VS Code integration |
| CLI | `cmd/sruja/` | Compile, lint, verify, export |
| Exporters | `pkg/export/` | JSON, Mermaid, Markdown, **LikeC4** |

### Deprecated (Replace with LikeC4)

| Package | Status | Replacement |
|---------|--------|-------------|
| `@sruja/layout` | Archive | LikeC4 layout engine |
| `@sruja/diagram` | Archive | `@likec4/diagram` |

---

## Sruja's Differentiators

### 1. Drift Detection
Compare architecture model against actual code:
```bash
sruja verify ./src
# ❌ Code calls LegacyService, but architecture only allows ModernService
```

### 2. AI Context Generation
Export architecture for AI assistants:
```bash
sruja gen cursorrules  # → .cursorrules
sruja gen agents       # → AGENTS.md
```

### 3. Infrastructure Policy
Generate secure Terraform from architecture graph:
```bash
sruja gen terraform    # → IAM, Security Groups from relationships
```

### 4. Policy Enforcement
Block violations in CI/CD:
```bash
sruja lint architecture.sruja
# ❌ Error: Service "Payment" missing required SLO block
```

---

## Implementation

### Go Transpiler to LikeC4

```go
func (s *System) ToLikeC4() LikeC4Element {
    meta := map[string]interface{}{}
    
    if s.SLO.Availability != "" {
        meta["slo_availability"] = s.SLO.Availability
    }
    
    for _, p := range s.Policies {
        meta["policies"] = append(meta["policies"].([]string), p.Name)
    }

    return LikeC4Element{
        ID:       s.ID,
        Kind:     "system",
        Metadata: meta,
    }
}
```

### Frontend Integration

```jsx
import { LikeC4Diagram } from '@likec4/diagram';

export const ArchitectureView = ({ viewId }) => (
  <LikeC4Diagram 
    viewId={viewId} 
    interactive={true} 
    onNodeClick={(node) => showDetails(node)} 
  />
);
```

### CLI Commands

```bash
# Parse and validate
sruja compile architecture.sruja

# Export to LikeC4 JSON
sruja export likec4 architecture.sruja -o model.json

# Verify against codebase
sruja verify --architecture architecture.sruja --src ./src

# Generate AI context
sruja gen cursorrules
```

---

## Migration Timeline

- [ ] Implement `sruja export likec4` command
- [ ] Update website to use `@likec4/diagram`
- [ ] Update designer to use `@likec4/diagram`
- [ ] Archive `@sruja/layout` package
- [ ] Archive `@sruja/diagram` package
- [ ] Update documentation

---

## Positioning

> "Structurizr draws diagrams. Backstage lists services. **Sruja enforces architecture.**"

**LikeC4** = Markdown of architecture (defines the drawing)
**Sruja** = Linter & Compiler (enforces rules, builds infrastructure)

---

## Related Documentation

- [Architecture Guide](./ARCHITECTURE.md)
- [Language Specification](./LANGUAGE_SPECIFICATION.md)
- [Design Philosophy](./DESIGN_PHILOSOPHY.md)