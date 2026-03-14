---
name: sruja-architecture
description: Architecture discovery and DSL authoring for Sruja. Use this skill to generate, validate, and maintain repo.sruja files from codebases. This is the core skill for architecture-as-code with Sruja.
license: Apache-2.0
---

# Sruja Architecture Skill

Core skill for architecture discovery and modeling with Sruja. This skill provides a deterministic, evidence-first workflow for generating and maintaining repo.sruja files (architecture.sruja is also supported for backward compatibility). **No DSL learning required—the skill guides your AI to write valid architecture for you.**

## Core Principles

- **Evidence-first**: Gather evidence before modeling
- **No guessing**: Surface open questions instead of fabricating answers
- **Minimal DSL**: Generate only what evidence supports
- **Validation**: Always lint and fix errors before considering complete

## Evidence source: static graph (Tree-sitter)

Discovery and sync are backed by a **static analysis graph** built from **Tree-sitter** parsing of source code. The CLI parses supported languages (e.g. TypeScript, Python, Go, Rust, Java, C#, Ruby, and others) to extract modules, imports, and dependencies, producing a deterministic **nodes-and-edges graph**. This graph is used to:

- **Verify** — Evidence is code-based and reproducible; drift compares the declared architecture to this graph.
- **Assist the AI** — The skill uses this graph (via `sruja sync` / `sruja discover` or `.sruja/context.json`) so the AI can stay evidence-first and avoid inventing components or relationships not present in the code.

When using the skill, prefer evidence from this pipeline (context file or discover output) over guessing; the static graph is the single source of truth for what the codebase contains.

## Workflow

### 1. Collect Evidence

**Editor integration:** If the user has run **Sruja: Refresh repo context** in the editor (or `.sruja/context.json` exists), use that file for evidence first. The file includes `updated_at`, `truth_status`, `baseline_path`, and `git_commit` when produced by `sruja sync`. If the file has an `updated_at` timestamp older than about 1 hour, treat it as stale and suggest refreshing ( **Sruja: Refresh repo context** or `sruja sync -r .`).

Otherwise, run discovery to gather deterministic evidence:

```bash
sruja sync -r .
```
or, for JSON only without writing the file: `sruja discover --context -r . --format json`

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

// External actors: person = humans only; external software = system (optional tags ["external"])
Person = person "Person"
ExternalAPI = system "External API" { description "Third-party or backend service" }

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
System.Container -> ExternalAPI "HTTPS"
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
- E206: Invalid references (target element missing or typo)

### 5. Refine (Optional)

Once a baseline exists, run drift detection to find new violations:

```bash
sruja drift -r .
```

(If no `repo.sruja` or `architecture.sruja` exists, drift runs structural-only analysis; use `-a path` to specify a different file.)

Use drift results to identify areas needing refinement.

## Operating Modes

This skill supports four modes:

1. **Local authoring** — Create or update `repo.sruja` (or `architecture.sruja`) in the repo. Use evidence + targeted questions, then generate minimal DSL and run `sruja lint`.
2. **System context** — Use discover output or `.sruja/context.json` to get the right slice of the system for the current task. Prefer canonical element IDs and boundaries from the evidence.
3. **Drift refinement** — After code or intent changes, use `sruja drift -r .` (and optionally `sruja intent propose`) to turn evidence deltas into DSL updates or open questions. Do not invent changes without evidence.
4. **Multi-repo (federation)** — When a `system.index.json` is available (from `sruja compose`), load only the **impacted slice** (nodes/edges for the current repo or task). Use **canonical IDs** (`repo_id::local_id`) when referring to elements from other repos. Do not assume same-named elements across repos are the same; check `conflicts` and ownership.

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

## Multi-repo (publish and compose)

For systems spanning multiple repos:

- **Publish** — In each repo, run `sruja publish -r . -o repo.bundle.json` to produce a bundle (repo metadata, DSL snapshot, context, truth state). Collect bundles in a shared directory or artifact store.
- **Compose** — Run `sruja compose -i <dir-or-bundle> -o system.index.json` to build a single system graph with canonical IDs (`repo_id::local_id`). Use this index for cross-repo views and editor retrieval (see **Retrieval order** above).
- **Conflicts** — Same kind+label in multiple repos are reported in `system.index.json` → `conflicts`; resolve ownership or rename; never silently merge.

See **docs/FEDERATION.md** for artifact schemas and Phase 4 retrieval behavior.

## Related References

- **Discovery workflow**: See REFERENCE.md
- **Modeling rules**: See rules/ directory
- **Prompt patterns**: See PROMPTS.md and AGENTS.md
- **Refinement workflow**: See REFERENCE.md
- **Multi-repo federation**: See docs/FEDERATION.md

## Prerequisites

The skill’s prompts run `sruja sync`, `sruja discover`, `sruja lint`, and `sruja drift`. The CLI **must** include the `sync` and `discover` subcommands for evidence gathering. Install a recent Sruja CLI:

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

Or build from this repo (ensures `sync` and `discover` are present):

```bash
git clone https://github.com/sruja-ai/sruja && cd sruja && cargo install --path crates/sruja-cli
```

Verify: run `sruja --help` and confirm you see `sync` and `discover` in the command list. If not, you have an older or different binary; use one of the methods above.

### Extension (optional, recommended in VS Code / Cursor)

Install the [Sruja extension](https://marketplace.visualstudio.com/items?itemName=SrujaAI.sruja) for a better in-editor experience:

- **Syntax highlighting, diagnostics, snippets** for `.sruja` files
- **Sruja: Run validation** — lint after each AI edit (same as `sruja lint`)
- **Sruja: Open Diagram Preview** — Mermaid diagram from the current file
- **Sruja: Export architecture to Markdown** — full architecture doc (TOC, systems, requirements, ADRs, diagrams)
- **Sruja: Refresh repo context** — runs discovery and writes `.sruja/context.json`; the skill uses this file as evidence when present and recent, so you (or the AI) don’t need to run `sruja discover` in the terminal

When the extension is installed, run **Sruja: Refresh repo context** once (or after big repo changes); the skill will prefer `.sruja/context.json` over re-running discover.

### Troubleshooting: "CLI lacks sync/discover"

If the AI says the installed Sruja CLI lacks `sync` or `discover`, the `sruja` on your PATH is likely an older release or a different build (e.g. an old `cargo install sruja-cli` from crates.io). The skill then falls back to gathering evidence from repository structure and codebase only.

**Fix:** Install a CLI that includes these commands:

1. **Install script (recommended):** `curl -fsSL https://sruja.ai/install.sh | bash`
2. **Build from this repo:** `git clone https://github.com/sruja-ai/sruja && cd sruja && cargo install --path crates/sruja-cli`

Then run `sruja --help` and confirm `sync` and `discover` appear. Ensure this binary is first on your PATH when using the skill.

## Installation

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

## Retrieval order (Phase 4)

When doing architecture-aware codegen or review, load context in this order:

1. **Local repo truth** — `repo.sruja` (or `architecture.sruja`) in the current repo.
2. **Fresh evidence** — `.sruja/context.json` (from `sruja sync`); if missing or stale, run `sruja sync -r .` or suggest the user run **Sruja: Refresh repo context**.
3. **Relevant slice from system index** — If `system.index.json` exists (e.g. from `sruja compose`), load only the impacted slice (elements for the current repo or task). Use canonical IDs (`repo_id::local_id`) for cross-repo references.
4. **Intent and contract refs** — From repo or bundle (ADRs, intent files).
5. **Truth/drift status** — From `.sruja/context.json` or `sruja status -r . --format json`.

Prefer canonical element IDs over path guesses; include ownership and recent drift when present. If context is missing, ask a targeted question or mark `unknown`; do not invent.

## Quick Start Prompt

Copy-paste this into your AI assistant:

```
Use sruja-architecture skill. If .sruja/context.json exists and is recent (e.g. updated_at within the last hour), use it for evidence; otherwise run `sruja sync -r .` or `sruja discover --context -r . --format json`. Gather evidence, ask targeted questions only if scope or externals are unclear, generate a minimal repo.sruja (or architecture.sruja) with evidence-based components and relationships, then run `sruja lint` and fix all errors until it passes. Do not guess about missing information; list open questions instead.
```

**Cursor/.cursorrules:** For architecture tasks, you can add: "Use `.sruja/context.json` when present and recent; else run `sruja sync -r .` or `sruja discover --context -r . --format json`. For multi-repo tasks, use the impacted slice from `system.index.json` when available; see docs/FEDERATION.md."
