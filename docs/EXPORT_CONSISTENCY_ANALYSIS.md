# Export Consistency Analysis & Improvements

## Current State

### ✅ What's Working
- **Markdown Export**: Fully implemented with Go templates, all Phase 1-3 sections
- **JSON Export**: Basic structure exists, exports architecture data
- **HTML Export**: Uses JSON export internally
- **SVG Export**: Separate implementation
- **LSP**: Provides language server features (completion, hover, diagnostics)
- **WASM**: Exposes parser and validator to web

### ❌ Gaps Identified

1. **JSON Export**: Missing new FAANG sections (Executive Summary, Capacity Planning, etc.)
2. **LSP**: Doesn't expose new metadata fields for IDE features
3. **WASM**: May not expose new architecture fields
4. **Markdown Order**: Not optimized for architecture review workflow
5. **Template Usage**: Some sections still use string building instead of templates

## Recommendations

### 1. Markdown Section Order for Architecture Review

**Current Order Issues:**
- Executive Summary appears early (good) but operational sections are scattered
- Security appears before operational concerns
- Cost analysis appears very late

**Recommended Order for Architecture Review:**

```markdown
1. Executive Summary          # High-level overview first
2. Architecture Overview      # Visual diagrams
3. Systems                    # Core architecture
4. Requirements               # What we need to build
5. ADRs                       # Why we made these decisions
6. Quality Attributes         # Non-functional requirements
7. Security                   # Security concerns
8. Capacity Planning          # Operational: capacity
9. Monitoring & Observability # Operational: observability
10. Failure Modes             # Operational: reliability
11. Dependency Risk Assessment # Risk analysis
12. Compliance & Certifications # Compliance
13. Cost Analysis             # Financial
14. API Versioning            # API management
15. Multi-Region Architecture # Deployment
16. Data Lifecycle Management # Data management
17. Policies                  # Governance
18. Flows                     # Data flows
19. Contracts                 # Integration contracts
20. Scenarios                 # Use cases
21. Relations                 # Reference
22. Data Consistency          # Reference
23. Constraints               # Reference
24. Conventions               # Reference
25. Domain Model              # Reference
26. Metadata                  # Reference
27. Glossary                  # Reference
```

**Rationale:**
- **Executive → Architecture → Systems**: Top-down understanding
- **Requirements → ADRs**: What and why
- **Quality → Security → Operations**: Non-functional concerns grouped
- **Risk → Compliance → Cost**: Business concerns grouped
- **API → Deployment → Data**: Technical concerns grouped
- **Reference sections last**: Details for deep dive

### 2. JSON Export Updates Needed

**Current JSON Structure:**
```json
{
  "metadata": {...},
  "architecture": {
    "systems": [...],
    "persons": [...],
    "relations": [...]
  },
  "navigation": {...},
  "views": {...}  // if extended
}
```

**Missing Fields:**
- Executive summary data
- Capacity planning data
- Monitoring/observability config
- Security threat model
- Compliance matrix
- Dependency risk assessment
- Cost analysis
- API versioning info
- Multi-region config
- Data lifecycle policies

**Recommended JSON Structure:**
```json
{
  "metadata": {...},
  "architecture": {
    "systems": [...],
    "persons": [...],
    "relations": [...],
    "executiveSummary": {
      "overview": "...",
      "keyMetrics": [...],
      "highlights": [...],
      "risks": [...]
    },
    "capacityPlanning": {
      "current": {...},
      "scaling": {...},
      "projected": {...}
    },
    "monitoring": {...},
    "security": {
      "threatModel": {...},
      "controls": [...]
    },
    "compliance": {...},
    "dependencies": {
      "external": [...],
      "internal": [...]
    },
    "cost": {...},
    "apiVersioning": {...},
    "multiRegion": {...},
    "dataLifecycle": {...}
  },
  "navigation": {...},
  "views": {...}
}
```

### 3. LSP Updates Needed

**Current LSP Features:**
- Completion (autocomplete)
- Hover (documentation)
- Diagnostics (errors)
- Symbols (go to definition)
- References (find usages)

**Missing Features:**
- Hover on metadata keys shows FAANG section info
- Completion for new metadata keys (capacity, monitoring, etc.)
- Quick info for compliance requirements
- Cost analysis hints

**Recommended Updates:**
- Add hover documentation for new metadata keys
- Add completion for FAANG-related metadata
- Add diagnostics for missing required sections (optional)

### 4. WASM Updates Needed

**Current WASM Exports:**
- `parse()` - Parse DSL to AST
- `validate()` - Validate architecture
- `exportJSON()` - Export to JSON

**Missing:**
- New architecture fields may not be exposed
- No direct access to FAANG sections

**Recommended:**
- Ensure all new AST fields are exported
- Add helper functions for FAANG sections if needed

### 5. Template Usage

**Current State:**
- ✅ Main document uses template
- ✅ Individual sections use templates
- ❌ Some helper functions still use string building

**Recommendation:**
- Keep current approach (templates for structure, Go code for logic)
- This is the right balance - templates for structure, Go for complex logic

## Implementation Plan

### Phase 1: Fix Markdown Order
1. Reorder sections in `main.tmpl`
2. Update TOC to match new order
3. Test with examples

### Phase 2: Update JSON Export
1. Add new fields to `ArchitectureJSON` type
2. Extract FAANG sections in `convertArchitectureToJSON`
3. Update JSON schema documentation

### Phase 3: Update LSP (Optional)
1. Add hover docs for new metadata keys
2. Add completion for FAANG metadata
3. Test in VS Code extension

### Phase 4: Verify WASM (Optional)
1. Test that new fields are accessible
2. Add helper functions if needed

## Priority

1. **High**: Fix markdown order (affects all users)
2. **Medium**: Update JSON export (affects HTML export and integrations)
3. **Low**: LSP/WASM updates (nice to have, doesn't break anything)
