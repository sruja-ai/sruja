---
name: sruja-architecture
description: Architecture discovery and DSL authoring for Sruja. Use this skill to generate, validate, and maintain repo.sruja files from codebases. This is the core skill for architecture-as-code with Sruja.
license: Apache-2.0
---

# Sruja Architecture Skill

Core skill for architecture discovery and DSL authoring with Sruja. This skill provides a deterministic, evidence-first workflow for generating and maintaining repo.sruja files (architecture.sruja is also supported for backward compatibility).

## Core Principles

- **Evidence-first**: Gather evidence before modeling
- **No guessing**: Surface open questions instead of fabricating answers
- **Minimal DSL**: Generate only what evidence supports
- **Validation**: Always lint and fix errors before considering complete

## Workflow

### 1. Collect Evidence

**Editor integration:** If the user has run **Sruja: Refresh repo context** in the editor (or `.sruja/context.json` exists), use that file for evidence first. If the file has an `updated_at` timestamp older than about 1 hour, treat it as stale and run discovery again or suggest refreshing.

Otherwise, run discovery to gather deterministic evidence:

```bash
sruja discover --context -r . --format json
```

Evidence (from file or command) returns:
- Repository structure
- Detected technologies
- Module boundaries
- Entry points
- External dependencies
- Scan scope (what was included/excluded)

### 2. Ask Targeted Questions

Ask 2-5 questions only when evidence is ambiguous:

- What are the main system boundaries?
- What external services do you integrate with?
- What datastores are used?
- How are components deployed?
- What are the main data flows?

Never ask about information that's clear from evidence.

### 3. Generate Minimal DSL

Generate a minimal `repo.sruja` covering only what evidence supports. Start with C4 context and container levels. Add component level only when evidence justifies it.

Use this structure:

```sruja
// Import standard kinds (optional)
import { * } from 'sruja.ai/stdlib'

// Define external actors
Person = person "Person"

// Define major systems
System = system "System" {
  // Containers (deployable units)
  Container = container "Container" {
    technology "Technology"
    description "Description"
  }
}

// Relationships
Person -> System.Container "Protocol"
```

### 4. Validate and Repair

Always lint the generated DSL:

```bash
sruja lint repo.sruja
```

Fix all errors before proceeding. Common issues:
- E201: Invalid kind or type
- E204: Circular dependencies
- E205: Orphan elements (no relationships)

### 5. Refine (Optional)

Once a baseline exists, run drift detection to find new violations:

```bash
sruja drift -r .
```

(If no `repo.sruja` or `architecture.sruja` exists, drift runs structural-only analysis; use `-a path` to specify a different file.)

Use drift results to identify areas needing refinement.

## Operating Modes

This skill supports three modes:

1. **Local authoring** — Create or update `repo.sruja` (or `architecture.sruja`) in the repo. Use evidence + targeted questions, then generate minimal DSL and run `sruja lint`.
2. **System context** — Use discover output or `.sruja/context.json` to get the right slice of the system for the current task. Prefer canonical element IDs and boundaries from the evidence.
3. **Drift refinement** — After code or intent changes, use `sruja drift -r .` (and optionally `sruja intent propose`) to turn evidence deltas into DSL updates or open questions. Do not invent changes without evidence.

## When to Apply

Use this skill when:
- Discovering architecture from a new codebase
- Generating initial repo.sruja from requirements
- Refactoring existing architecture
- Validating architecture against code
- Maintaining architecture documentation

## What NOT to Do

- Don't guess about missing information - list as open questions
- Don't generate framework/domain/security narratives without evidence
- Don't create components just for completeness
- Don't add relationships that aren't supported by evidence
- Don't skip linting

## Open Questions

When information is missing or unclear, surface it explicitly:

```sruja
// Instead of guessing deployment strategy
// Add a comment or separate section:
//
// OPEN QUESTIONS:
// - How is authentication implemented?
// - What is the message queue for async operations?
// - Are there external API integrations not detected?
```

## Related References

- **Discovery workflow**: See REFERENCE.md
- **Modeling rules**: See rules/ directory
- **Prompt patterns**: See AGENTS.md
- **Refinement workflow**: See REFERENCE.md

## Prerequisites

The skill’s prompts run `sruja discover`, `sruja lint`, and `sruja drift`. Install the Sruja CLI first:

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

Alternatively: build from source: `cargo install sruja-cli`.

### Extension (optional, recommended in VS Code / Cursor)

Install the [Sruja extension](https://marketplace.visualstudio.com/items?itemName=SrujaAI.sruja) for a better in-editor experience:

- **Syntax highlighting, diagnostics, snippets** for `.sruja` files
- **Sruja: Run validation** — lint after each AI edit (same as `sruja lint`)
- **Sruja: Open Diagram Preview** — Mermaid diagram from the current file
- **Sruja: Export architecture to Markdown** — full architecture doc (TOC, systems, requirements, ADRs, diagrams)
- **Sruja: Refresh repo context** — runs discovery and writes `.sruja/context.json`; the skill uses this file as evidence when present and recent, so you (or the AI) don’t need to run `sruja discover` in the terminal

When the extension is installed, run **Sruja: Refresh repo context** once (or after big repo changes); the skill will prefer `.sruja/context.json` over re-running discover.

## Installation

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

## Quick Start Prompt

Copy-paste this into your AI assistant:

```
Use sruja-architecture skill. If .sruja/context.json exists and is recent (e.g. updated_at within the last hour), use it for evidence; otherwise run `sruja discover --context -r . --format json`. Gather evidence, ask targeted questions only if scope or externals are unclear, generate a minimal repo.sruja (or architecture.sruja) with evidence-based components and relationships, then run `sruja lint` and fix all errors until it passes. Do not guess about missing information; list open questions instead.
```

**Cursor/.cursorrules:** For architecture tasks, you can add: "Use `.sruja/context.json` when present and recent; else run `sruja discover --context -r . --format json`."
