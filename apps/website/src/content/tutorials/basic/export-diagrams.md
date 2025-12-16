---
title: "Export Diagrams: Mermaid & Studio"
weight: 40
summary: "Export architecture to Mermaid (Markdown) or interactive Studio."
tags: ["export", "diagrams", "studio", "mermaid"]
---

# Export Diagrams

Sruja currently supports export to **Mermaid** (for Markdown) and interactive visualization in **Studio**.

## Export Formats

### 1. Mermaid (Markdown)

Export to Mermaid code fences for use in Markdown pages:

```bash
sruja export mermaid architecture.sruja > architecture.md
```

The output includes ```mermaid blocks that render in most Markdown engines with Mermaid enabled.

**Use cases:**

- Documentation sites using Markdown
- Lightweight diagrams without external tooling

### 2. Studio (Interactive)

Open and preview diagrams interactively in Studio:

```text
Open in Studio from the Learn examples or visit /studio/
```

**Features:**

- Interactive preview and navigation
- C4 model views (context, containers, components)
- Embedded documentation and metadata

**Use cases:**

- Architecture reviews
- Presentations
- Iterative modeling and validation

## Mermaid Styling

You can customize Mermaid via frontmatter or exporter configuration.
See `pkg/export/markdown/MERMAID_CONFIG.md` for available options.

## Choosing the Right Path

- **Mermaid**: For Markdown-first workflows and lightweight sharing
- **Studio**: For interactive exploration and richer documentation

**Note**: Sruja Designer provides interactive diagrams and editing capabilities.
