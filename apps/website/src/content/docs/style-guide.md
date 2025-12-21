---
title: "Documentation Style Guide"
weight: 100
summary: "Standards for tutorials, how‑tos, reference, and explanation."
tags: ["docs", "style", "quality"]
---

# Documentation Style Guide

## Goals
- Align with the Diátaxis framework: Tutorials, How‑to Guides, Reference, Explanation
- Improve clarity, consistency, and task‑orientation
- Raise quality to industry standards (Stripe, React, Kubernetes, MDN)

## Front Matter
- Required: `title`, `summary`
- Recommended: `prerequisites`, `learning_objectives`, `estimated_time`, `difficulty`, `tags`, `last_updated`

## Headings
- Use Title Case for H1/H2/H3
- Keep headings unique; avoid duplicates within a page

## Code Blocks
- Always specify language fences: `bash`, `sh`, `json`, `yaml`, `go`, `ts`, `tsx`, `md`, `sruja`
- Prefer copy‑ready commands; avoid interactive prompts where possible

## Admonitions
- Use standard callouts: Note, Tip, Warning
- Keep callouts short and action‑oriented

## Links
- Prefer descriptive link text (not raw URLs)
- Cross‑link to Reference and Examples when teaching a concept or task

## Images & Diagrams
- Include small screenshots or diagram previews for expected outcomes
- Use alt text that describes the intent and context

## Tutorials
- Structure: Overview → Prerequisites → Steps → Outcome → Troubleshooting → Next Steps
- Include at least one end‑to‑end task with an expected output

## How‑to Guides
- Task‑oriented and concise
- Structure: Purpose → Steps → Validation → References

## Reference
- Precise, complete, and skimmable tables/lists
- Avoid narrative; link outward to tutorials for workflows

## Explanation
- Conceptual background, rationale, trade‑offs
- Link to reference for details and to tutorials for practice

## Quality Gates
- Markdown lint for headings, lists, links
- Link checking for external and internal links
- Optional accessibility lint (alt text, heading levels)

## Review Checklist
- Front matter present and complete
- Headings consistent and unique
- Code fences have language tags
- Cross‑links added to relevant Reference/Examples
- Outcome preview or screenshot included where appropriate
