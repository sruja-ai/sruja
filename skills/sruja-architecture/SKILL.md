---
name: sruja-architecture
description: Architecture discovery and DSL authoring for Sruja. Use this skill to generate, validate, and maintain architecture.sruja files from codebases. This is the core skill for architecture-as-code with Sruja.
license: Apache-2.0
---

# Sruja Architecture Skill

Core skill for architecture discovery and DSL authoring with Sruja. This skill provides a deterministic, evidence-first workflow for generating and maintaining architecture.sruja files.

## Core Principles

- **Evidence-first**: Gather evidence before modeling
- **No guessing**: Surface open questions instead of fabricating answers
- **Minimal DSL**: Generate only what evidence supports
- **Validation**: Always lint and fix errors before considering complete

## Workflow

### 1. Collect Evidence

Run discovery to gather deterministic evidence from the codebase:

```bash
sruja discover --context -r . --format json
```

This returns:
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

Generate a minimal `architecture.sruja` covering only what evidence supports. Start with C4 context and container levels. Add component level only when evidence justifies it.

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
sruja lint architecture.sruja
```

Fix all errors before proceeding. Common issues:
- E201: Invalid kind or type
- E204: Circular dependencies
- E205: Orphan elements (no relationships)

### 5. Refine (Optional)

Once a baseline exists, run drift detection to find new violations:

```bash
sruja drift -r . -a architecture.sruja --format json
```

Use drift results to identify areas needing refinement.

## When to Apply

Use this skill when:
- Discovering architecture from a new codebase
- Generating initial architecture.sruja from requirements
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

## Installation

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

## Quick Start Prompt

Copy-paste this into your AI assistant:

```
Use sruja-architecture skill. Run `sruja discover --context -r . --format json`, gather evidence from the repo, ask targeted questions only if scope or externals are unclear, generate a minimal architecture.sruja with evidence-based components and relationships, then run `sruja lint architecture.sruja` and fix all errors until it passes. Do not guess about missing information; list open questions instead.
```
