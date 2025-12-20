---
title: "Design Mode Workflow"
weight: 6
summary: "A guided, layered workflow to design architectures step‑by‑step and share focused views."
tags: ["workflow", "design", "studio"]
---

# Design Mode Workflow

Design Mode helps you build architecture assets step by step, starting with high‑level context and progressively adding detail. It also lets you focus on a specific system or container and share audience‑specific views.

## Workflow Steps

### Step 1: Context — define `person` and `system`

Start with the high-level context:

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  person User
  system Shop
}

views {
  view index {
    include *
  }
}
```

### Step 2: Containers — add `container`, `datastore`, `queue` to a chosen system

Add containers and datastores:

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  person User
  system App {
    WebApp = container "Web Application"
    API = container "API Service"
    DB = datastore "Database"
  }

  User -> App.WebApp "Uses"
  App.WebApp -> App.API "Calls"
  App.API -> App.DB "Reads/Writes"
}

views {
  view index {
    include *
  }
}
```

### Step 3: Components — add `component` inside a chosen container

Drill down into components:

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  system App {
    container WebApp {
      component UI
    }
    container API {
      component Auth
    }
  }

  // Component‑level interaction
  App.WebApp.UI -> App.API.Auth "Calls"
}

views {
  view index {
    include *
  }
}
```

### Step 4: Stitch — add relations and optional scenarios; share focused views

Add relations and scenarios to complete the model.

## Layers and Focus

- **Levels**: L1 Context, L2 Containers, L3 Components, All
- **Focus**:
  - L2 focus by `systemId`
  - L3 focus by `systemId.containerId`

When focused, non‑relevant nodes/edges are dimmed so you can work deeper without distractions.

## Share Deep Links

Viewer opens focused views via URL params:
- `?level=1` → Context
- `?level=2&focus=Shop` → Containers of system `Shop`
- `?level=3&focus=Shop.API` → Components in container `API` of system `Shop`
- DSL payload is passed with `#code=<lz-base64>` or `?code=<urlencoded>`.

## Studio Experience

- **Diagram‑first**: Studio opens with the diagram; a Design Mode overlay guides steps
- **Contextual palette**: add containers at L2 (focused system), components at L3 (focused container)
- **Autosave on close**: resume drafts; share per‑layer links from the toolbar

## Viewer Experience

- Use level buttons and focus to tailor the view
- Dimming clarifies what's relevant at each depth
- Share via copied URL (includes `level`, `focus`, and DSL)

## See Also

- [Layering](/docs/concepts/layering)
- [Validation](/docs/concepts/validation)
- [Scenario](/docs/concepts/scenario)
