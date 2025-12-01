---
title: "Export Diagrams"
weight: 40
summary: "Export architecture to D2 or Sruja Format (interactive SVG)."
tags: [export, d2, svg]
aliases: ["/tutorials/export-d2-diagrams/"]
---

# Export Diagrams

Sruja supports two export formats: **D2** (for further processing) and **Sruja Format** (interactive SVG).

## Export Formats

### 1. D2 Format (Default)

Export to D2 for further processing with D2 tools:

```bash
sruja export d2 architecture.sruja > architecture.d2
```

**Render with D2 CLI:**
```bash
d2 architecture.d2
```

**Use cases:**
- Further customization with D2
- Integration with D2 tooling
- Custom styling and layouts

### 2. Sruja Format (Interactive SVG)

Export to interactive, self-contained SVG:

```bash
sruja export svg architecture.sruja > architecture.svg
```

**Features:**
- ✅ Self-contained (single file, no dependencies)
- ✅ Interactive (click elements to view documentation)
- ✅ C4 model levels (System Context, Containers, Components)
- ✅ Embedded documentation (requirements, ADRs, technology)
- ✅ Shareable (open in any browser)

**Use cases:**
- Sharing with stakeholders
- Architecture reviews
- Documentation websites
- Presentations

## Tooltips and Shapes

Descriptions become tooltips; certain elements map to shapes (e.g., datastore → cylinder).

## Choosing the Right Format

- **D2 Format**: When you need further customization or integration with D2 tooling
- **Sruja Format**: When you want a complete, shareable, interactive diagram

**Note**: The Playground uses Sruja Format by default for the best out-of-the-box experience.

