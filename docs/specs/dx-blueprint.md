# Developer Experience (DX) Blueprint

## The 10 Pillars of Best Possible Developer Experience

This document outlines the complete DX strategy for Sruja DSL, modeled after successful developer tools like Terraform, GraphQL, CDK, Prisma, and Structurizr.

---

## 1. Intelligent LSP (Instant Feedback)

**Status:** ✅ Foundation complete, enhancements in progress

### Core Features

- ✅ Autocomplete (keywords, element IDs, metadata keys)
- ✅ Semantic index for cross-file references
- ✅ Metadata registry for plugin-contributed keys
- 🔄 Go-to-definition (basic implementation, needs enhancement)
- 🔄 Hover docs (basic, needs rich formatting)
- ✅ Context-aware completion
- 🔄 Quick actions/code actions (planned)

### Enhancement Roadmap

1. **Rich Hover Documentation**
   - Element descriptions
   - Metadata tooltips
   - ADR summaries
   - Journey context
   - Relation details

2. **Quick Actions**
   - "Create system with this name"
   - "Create container inside system"
   - "Add metadata stub"
   - "Generate ADR for this decision"
   - "Add relation to element"

3. **Enhanced Diagnostics**
   - Suggest fixes for errors
   - Show related elements
   - Link to documentation

---

## 2. Clean, Human-Friendly DSL Syntax

**Status:** ✅ Complete

The DSL follows best practices:
- Minimal, readable syntax
- Predictable structure
- Similar to Terraform, GraphQL SDL, Structurizr

**Example:**
```sruja
system API {
    container App "Web App" {
        component Checkout "Checkout Component"
    }
}
```

---

## 3. Rich, Beautiful Visualizations

**Status:** 🔄 In progress

### Current Support
- ✅ D2 diagrams
- ✅ Mermaid diagrams
- 🔄 C4 automatic diagrams
- 🔄 Flow diagrams for journeys
- 🔄 Dependency graphs

### Planned Features

1. **Interactive Diagrams**
   - Zoomable views
   - Click to navigate to code
   - Metadata-based coloring
   - Cloud icons (AWS, Azure, GCP plugins)

2. **Diagram Types**
   - System context diagrams
   - Container diagrams
   - Component diagrams
   - Journey flow diagrams
   - Variant diff diagrams

3. **Visual Features**
   - Theme support (dark/light)
   - Metadata visualization
   - Team ownership coloring
   - Risk/criticality indicators

---

## 4. The CLI (Make It Feel Like Terraform)

**Status:** 🔄 Enhancement in progress

### Current Commands
- ✅ `sruja compile` - Generate diagrams
- ✅ `sruja lint` - Validate DSL
- ✅ `sruja fmt` - Format DSL
- ✅ `sruja notebook` - Run notebook
- ✅ `sruja install` - Install dependencies

### Planned Commands

1. **Explainer Commands**
   - `sruja explain <element>` - ChatGPT-style natural explanation
   - `sruja describe <element>` - Detailed element information

2. **Navigation Commands**
   - `sruja list systems` - List all systems
   - `sruja list containers` - List all containers
   - `sruja tree <system>` - Show element hierarchy
   - `sruja flow <journey>` - Show journey flow

3. **Analysis Commands**
   - `sruja diff <file1> <file2>` - Compare architectures
   - `sruja validate --strict` - Strict validation
   - `sruja stats` - Architecture statistics

4. **Plugin Commands**
   - `sruja generate --plugin cloud` - Generate cloud configs
   - `sruja plugins list` - List installed plugins
   - `sruja plugins install <plugin>` - Install plugin

### CLI Output Requirements

- ✅ Colorful output
- 🔄 Structured JSON output flag
- 🔄 Actionable error messages
- 🔄 Progress indicators
- 🔄 Helpful suggestions

---

## 5. Documentation That Reads Like a Story

**Status:** 🔄 In progress

### Documentation Structure

1. **Getting Started**
   - Installation guide
   - Quick start tutorial
   - First architecture example

2. **Core Concepts**
   - Architecture modeling
   - Elements (systems, containers, components)
   - Relations
   - Journeys
   - ADRs

3. **Advanced Features**
   - Imports and multi-file architectures
   - Metadata and plugins
   - Architecture variants
   - Best practices

4. **Plugin Development**
   - Plugin SDK guide
   - Metadata schema definition
   - LSP extension API
   - Testing utilities

5. **Examples**
   - Real-world architectures
   - Common patterns
   - Snippets library

---

## 6. Plugin Ecosystem (Your Real Superpower)

**Status:** ✅ Foundation complete

### Plugin Capabilities

- ✅ Metadata keys registration
- ✅ LSP completion contributions
- 🔄 Validation rules
- 🔄 Diagram generators
- 🔄 Exporters (Terraform, CloudFormation, etc.)
- 🔄 CLI command extensions

### Plugin Development Experience

**Current:**
- Plugin interface defined
- Metadata registry support
- LSP integration

**Planned:**
- Plugin SDK package
- Development templates
- Hot-reloading during development
- Testing utilities
- Example plugins

---

## 7. Opinionated Defaults, Configurable Options

**Status:** 🔄 Planned

### Configuration File: `sruja.config.json`

```json
{
  "diagrams": {
    "theme": "dark",
    "showMetadata": ["team", "criticality"],
    "defaultFormat": "d2"
  },
  "plugins": ["cloud", "security", "rate-limits"],
  "validation": {
    "strict": false,
    "rules": ["unique-ids", "valid-references"]
  },
  "lsp": {
    "metadataSuggestions": true,
    "quickActions": true
  }
}
```

### Default Features
- Sensible diagram defaults
- Core metadata keys
- Standard validation rules
- Default plugins (optional)

---

## 8. First-Class Git Integration

**Status:** 🔄 Planned

### Features

1. **Diff Support**
   - `sruja diff HEAD~1 HEAD` - Compare architectures
   - Visual diff diagrams
   - Breaking change detection

2. **PR Integration**
   - Automatic architecture checks
   - ADR detection
   - Missing metadata warnings
   - Diagram generation

3. **Git Hooks**
   - Pre-commit validation
   - Format checking
   - Architecture consistency checks

---

## 9. Explainers, Describe Functions, AI Integration

**Status:** 🔄 In progress

### Explain Command

```bash
sruja explain BillingAPI
```

**Output:**
- Purpose and description
- Incoming relations
- Outgoing relations
- Metadata (SLO, owner, team)
- Related ADRs
- Journey participation
- Visual diagram
- Risk assessment
- Dependencies

### Implementation Plan

1. Element explainer service
2. Rich formatting (markdown, diagrams)
3. Relationship traversal
4. Metadata aggregation
5. ADR linking

---

## 10. Consistent, Intuitive Error Messages

**Status:** 🔄 Enhancement in progress

### Error Message Requirements

**Bad:**
```
Error: invalid relation
```

**Good:**
```
Error: Unknown target "Billing" in relation.

  → Did you mean "BillingAPI"?
  → Or did you forget to import billing.sruja as Billing?

  At: example.sruja:12:5
  Context:
    10 |   system Frontend {
    11 |     container App {
    12 |       App -> Billing  ← Error here
    13 |     }
    14 |   }
```

### Enhancement Plan

1. Contextual error messages
2. Suggestions for fixes
3. Location highlighting
4. Related element hints
5. Quick fix actions

---

## Implementation Priority

### Phase 1: Core DX (Current)
- ✅ LSP foundation
- ✅ Metadata registry
- ✅ Semantic index
- 🔄 Enhanced error messages
- 🔄 CLI explain command

### Phase 2: Rich Features
- 🔄 Rich hover documentation
- 🔄 Quick actions
- 🔄 Configuration system
- 🔄 Tree/diff/list commands

### Phase 3: Advanced Features
- 🔄 Git integration
- 🔄 Plugin SDK
- 🔄 Interactive diagrams
- 🔄 AI explainer integration

---

## Success Metrics

1. **Developer Satisfaction**
   - Time to first diagram < 5 minutes
   - Error resolution time < 30 seconds
   - Plugin development time < 2 hours

2. **Feature Adoption**
   - >80% use autocomplete
   - >60% use hover docs
   - >40% use explain command

3. **Ecosystem Growth**
   - 10+ plugins within 6 months
   - Active community contributions
   - Real-world architecture examples

---

## References

- [LSP Completion Architecture](./lsp-completion-architecture.md)
- [Metadata Model](./metadata-model.md)
- [Plugin Framework](./plugin-framework.md) (planned)

