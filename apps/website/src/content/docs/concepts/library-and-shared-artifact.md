---
title: "Library and Shared Artifact"
weight: 61
summary: "Model reusable catalogs and artifacts shared across architectures."
---

# Library and Shared Artifact

Use `library` to define reusable entries; reference via `sharedArtifact` where needed.

## Syntax

```sruja
architecture "Shop" {
  library Platform {
    sharedArtifact "Logging" {
      description "Structured logging format v2"
      version "2.0.0"
      url "https://internal.docs/logging-v2"
    }
  }

  system App {
    container API {
      properties {
        logging "Platform.Logging@2.0.0"
      }
    }
  }
}
```

## Guidance
- Keep library items small and versioned; link to external docs.
- Reference artifacts by FQN and version for clarity.
- Prefer libraries for cross‑team standards (logging, tracing, security).

