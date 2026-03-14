# Install Sruja as a Skill

The core Sruja product is the `sruja-architecture` skill – use it for architecture discovery, DSL generation, and modeling with AI assistance.

## Install

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

Supported editors: Cursor, GitHub Copilot, Claude, Continue.dev, and any AI editor with skills.sh support.

## Quick Start

### 1. Install CLI

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

Verify: `sruja --version`

### 2. Install the skill

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

### 3. Generate architecture

In your AI editor, run:

```
Use sruja-architecture. Run `sruja discover --context -r . --format json`, gather evidence, ask targeted questions if scope or externals are unclear, generate architecture.sruja, then run `sruja lint` and fix until it passes.
```

### 4. Validate

```bash
sruja lint architecture.sruja
```

## Recommended Prompt: Architecture Discovery

This is the primary workflow for discovering architecture from code:

```
Use sruja-architecture. Run `sruja discover --context -r . --format json`, gather evidence from the repo, ask targeted questions only if scope or externals are unclear, generate a minimal architecture.sruja with evidence-based components and relationships using C4 context and container levels, then run `sruja lint architecture.sruja` and fix all errors until it passes. Do not guess about missing information; list open questions instead.
```

## What the Skill Does

### Evidence-First Discovery

1. Runs `sruja discover --context -r . --format json` to collect:
   - Repository structure
   - Detected technologies
   - Module boundaries
   - Entry points
   - Dependencies
   - Scan scope

2. Asks targeted questions only when evidence is ambiguous:
   - System boundaries
   - External integrations
   - Datastores
   - Deployment model
   - Data flows

3. Generates minimal `architecture.sruja` based on evidence:
   - Uses C4 modeling (person, system, container, database)
   - Includes only what evidence supports
   - Adds open questions for missing information

4. Validates with `sruja lint`:
   - Fixes all errors before completion
   - Ensures DSL is valid

### No Guessing

The skill follows these principles:

- **Never guess** about missing information
- **Surface open questions** instead of fabricating answers
- **Generate minimal DSL** covering only what evidence supports
- **Validate thoroughly** before considering complete

## CLI Commands Used by the Skill

The skill depends on these stable CLI commands:

```bash
# Collect evidence (primary contract)
sruja discover --context -r . --format json

# Validate DSL
sruja lint --format json architecture.sruja

# Detect drift (when baseline exists)
sruja drift --format json -r . -a architecture.sruja
```

## Documentation

- [Getting Started with Skill](GETTING_STARTED_SKILL.md) – Full workflow with examples
- [Skill Reference](../skills/sruja-architecture/SKILL.md) – Core orchestration guide
- [Skill Workflow Reference](../skills/sruja-architecture/REFERENCE.md) – Detailed discovery and modeling
- [Prompt Patterns](../skills/sruja-architecture/PROMPTS.md) – Reusable AI prompts
- [Compiled Guide](../skills/sruja-architecture/AGENTS.md) – Complete guide with all rules

## Skill Catalog

Currently, only one skill is supported:

| Skill | Purpose |
|-------|---------|
| `sruja-architecture` | **Primary** – Design, discover, and generate Sruja architecture |

Additional skills will be added after the core workflow proves out.
