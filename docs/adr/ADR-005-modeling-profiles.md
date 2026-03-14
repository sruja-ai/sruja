# ADR-005: Modeling Profiles - C4 as One Lens Among Many

## Status

Accepted

## Context

Sruja's DSL already supports more than C4 modeling:
- Custom element kinds (language-spec.md:7)
- Scenarios and flows (language-spec.md:224)
- Policies, conventions, ADRs, SLOs (language-spec.md:343)
- Deployment concepts (language-spec.md:28)
- View definitions for curated diagrams (language-spec.md:378)

However, the current implementation treats C4 as the universal model:
- Markdown export always generates auto-C4 diagrams (exporter.rs:108, 280)
- CLI markdown command ignores named views (dsl.rs:228)
- Element kinds default to C4 hierarchy (system/container/component/database/queue)

This creates friction when modeling:
- **Libraries and frameworks**: Not deployable boundaries; better described by API surface, modules, extension points, lifecycle
- **Data pipelines**: Flow-centric, not container-centric
- **Organizations**: Policy and governance focused, not system focused
- **Platforms**: Multi-tenant, plugin-based, not hierarchical

## Decision

**Demote C4 from universal model to one profile among several.**

### 1. Modeling Profiles

Define explicit profiles that configure default element kinds, diagram styles, and export behavior:

```sruja
profile library {
  default_kinds [library, module, package, adapter, plugin, extensionPoint, runtime, middleware]
  diagram_style "module-graph"
  export_sections [api-surface, lifecycle, conventions, integration]
}

profile c4 {
  default_kinds [person, system, container, component, database, queue]
  diagram_style "c4-hierarchy"
  export_sections [context, containers, components]
}

profile data-pipeline {
  default_kinds [source, transform, sink, stream, batch, stream-processor]
  diagram_style "dataflow"
  export_sections [lineage, quality, sla]
}

profile governance {
  default_kinds [policy, rule, control, exception, audit]
  diagram_style "policy-graph"
  export_sections [policies, exceptions, audit-trail]
}
```

### 2. Library/Framework Profile (First-Class)

For libraries and frameworks, use:

**Structure** (custom kinds):
```sruja
kind library
kind module
kind package
kind adapter
kind plugin
kind extensionPoint
kind runtime
kind middleware
```

**Behavior** (scenarios and flows):
```sruja
scenario Startup {
  App -> Library.initialize "loads config"
  Library -> PluginRegistry.register "discovers plugins"
  Library -> EventBus.publish "ready"
}

flow RequestLifecycle {
  Client -> Library.Router "incoming request"
  Library.Router -> Library.Handler "dispatch"
  Library.Handler -> Library.Middleware.* "process"
  Library.Handler -> UserCallback "invoke"
}
```

**Guarantees** (policies, conventions, SLOs):
```sruja
policy Compatibility {
  category "stability"
  enforcement "semver"
  description "Major version breaks API; minor adds; patch fixes"
}

convention Threading {
  "All callbacks are async"
  "No blocking I/O in handlers"
}

slo {
  availability "99.9%"
  latency "p50 < 10ms"
}
```

**C4 Usage** (ecosystem context only):
```sruja
// C4 only for who consumes and what externals exist
system "Consumer App 1" {
  description "Primary consumer, web application"
}

system "Consumer App 2" {
  description "Secondary consumer, CLI tool"
}

// Library structure uses custom kinds
Library = library "MyLib" {
  description "Core library"
  
  Core = module "Core" {
    description "Core utilities"
  }
  
  Router = module "Router" {
    description "Request routing"
  }
  
  PluginRegistry = module "Plugin Registry" {
    description "Plugin discovery and lifecycle"
  }
  
  extensionPoint Middleware {
    description "Request/response interception"
  }
}

// Relationships
ConsumerApp1 -> Library "depends on"
ConsumerApp2 -> Library "depends on"
Library.Core -> Library.Router
Library.Router -> Library.PluginRegistry
```

### 3. View-Driven Markdown Export

Named views control what appears in markdown, not auto-C4:

```sruja
view api_surface {
  title "API Surface"
  include Library.*.Public
  exclude Library.*.Internal
}

view extension_points {
  title "Extension Points"
  include Library.extensionPoint.*
}

view request_flow {
  title "Request Lifecycle"
  scenario RequestLifecycle
}
```

**Export behavior**:
- If views are defined: export only named views as sections
- If no views: fall back to current auto-C4 behavior (backwards compatible)
- CLI: `sruja export markdown --view api_surface file.sruja`

### 4. Multi-Page Export

For large models, support multi-page output:

```bash
sruja export markdown --multi-page file.sruja
```

Generates:
- `index.md` - Overview + TOC
- `systems/*.md` - One page per system
- `modules/*.md` - One page per major module (for library profile)
- `scenarios/*.md` - One page per scenario
- `appendix/dependency-graph.md` - Full graph (diagnostic only)

## Consequences

### Positive
- Libraries/frameworks get natural modeling without forcing C4 hierarchy
- Large models become readable through curated views
- DSL becomes profile-aware, not C4-only
- Backwards compatible (no views = current behavior)

### Trade-offs
- More concepts to learn (profiles, view-driven export)
- Need to document library/framework profile patterns
- Existing models may want to add views for better markdown output

### Migration Path
1. Phase 1: Add view-driven markdown export (no profile system yet)
2. Phase 2: Add profile definitions and library profile
3. Phase 3: Add multi-page export for large models
4. Phase 4: Update skills/prompts to recommend profiles based on project type

## Implementation Notes

- View resolution already exists in `crates/sruja-export/src/mermaid/views.rs`
- Markdown exporter needs to check for named views before auto-generating
- CLI needs `--view` and `--multi-page` flags
- Profile system is a new concept; start with view-driven export first
