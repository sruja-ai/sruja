# Template Engine

**Status**: Core Engine  
**Pillars**: Core (Templates)

[← Back to Engines](../README.md)

## Overview

The Template Engine provides template search, filtering, instantiation, and auto-layout for architecture templates. It enables users to quickly start with pre-built architecture patterns.

**This engine powers the Architecture Wizard experience.**

## Purpose

The Template Engine:

- ✅ Provides template registry and search
- ✅ Enables fuzzy search and filtering
- ✅ Supports template instantiation
- ✅ Applies auto-layout to templates
- ✅ Renders templates in the visual editor
- ✅ Integrates with Architecture Wizard

## Template Data Model

```ts
export interface ArchitectureTemplate {
  id: string;
  name: string;
  description: string;
  tags: string[];             // ["microservices", "event-driven"]
  complexity: "small" | "medium" | "large";
  layout: "vertical-hierarchy" | "horizontal-flow" | "layered" | "radial";
  category: "microservices" | "event" | "data" | "monolith" | "serverless" | "ml" | "saas" | "minimal";
  dsl: string;                // auto-loaded DSL
}
```

## Template Search & Filtering

### Search Query Model

```ts
export interface TemplateSearchQuery {
  text?: string;                      // fuzzy search string
  tags?: string[];                    // filter by tags
  category?: string | null;           // filter by category
  complexity?: "small" | "medium" | "large" | null;
  layout?: string | null;             // "vertical-hierarchy" etc.
}
```

### Search Engine

The search engine supports:

- **Fuzzy text search** (using Fuse.js)
- **Tag filtering**
- **Category filtering**
- **Complexity filtering**
- **Layout preference filtering**
- **Scoring and ranking**

### Scoring System

Templates are ranked by:

1. Text match (via Fuse)
2. Tag match count
3. Category match priority
4. Complexity & layout match bonus

## Template Instantiation

The instantiation process:

1. Load template DSL
2. Parse DSL to IR
3. Apply auto-layout
4. Render in visual editor
5. Enable editing

## Auto-Layout Integration

Templates use the Auto-Layout Engine with 4 layout modes:

- **vertical-hierarchy** - UI → APIs → Services → DBs
- **horizontal-flow** - Producer → Queue → Consumers
- **layered** - Multiple abstraction layers
- **radial** - Central hub with spokes

## Template Categories

Supported categories:

- Microservices
- Event-Driven
- Data Pipelines
- Monolith
- Serverless
- Machine Learning
- SaaS
- Minimal

## Predefined Tags

```ts
export const TEMPLATE_TAGS = [
  "microservices",
  "event-driven",
  "reactive",
  "serverless",
  "monolith",
  "etl",
  "pipeline",
  "ml",
  "saas",
  "ecommerce",
  "queue",
  "kafka",
  "payments",
  "auth",
];
```

## MCP API

```
template.search(query)
template.get(id)
template.instantiate(id)
template.list()
template.categories()
template.tags()
```

## Strategic Value

The Template Engine provides:

- ✅ Quick start with architecture patterns
- ✅ Best-practice templates
- ✅ Consistent architecture patterns
- ✅ Reduced setup time
- ✅ Learning resource

**This is critical for onboarding and rapid prototyping.**

## Implementation Status

✅ Architecture designed  
✅ Search engine specified  
✅ Template model defined  
📋 Implementation in progress

---

*The Template Engine provides template search, filtering, and instantiation for architecture patterns.*

