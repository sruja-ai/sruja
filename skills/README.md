# Sruja Skills

This directory contains skills for **AI code editors** (Cursor, Codex, and others) and the [skills.sh](https://skills.sh) ecosystem. Install a skill so your AI assistant generates valid Sruja DSL and applies architectural patterns.

Editor integration is **skills + CLI**. Sruja also ships an optional MCP stdio server (`sruja mcp`) for tool-based clients (e.g. Cursor), but most users do not need MCP to use the skills.

**Versioning:** Skills do not have a separate version field. They follow the **Sruja repo release version** (Git tag / GitHub Release). When updating a skill CHANGELOG, note that it aligns with repo version X.Y.Z. See [.github/workflows/README.md](../.github/workflows/README.md#version-consistency-release-please) for how crates, extension, and skills stay consistent.

## Install in your editor

**Recommended stack (Cursor, Codex, skills.sh):**

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-harness
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

Harness only (no `repo.sruja` required):

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-harness
```

**Full guide** – [docs/GETTING_STARTED_SKILL.md](../docs/GETTING_STARTED_SKILL.md) – editor-specific steps and recommended prompts.

**Repo pointer** – [.architecture-skill.md](../.architecture-skill.md) in the repo root has the same one-command install and links.

## Available Skills

### sruja-harness

**Install first** for any AI coding workflow. Verification adapter — does not generate code; runs `verify-task` before tasks are marked done.

**Install:**

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-harness
```

**Contents:**

- Profiles: `coding`, `bugfix`, `review`, `arch`
- Integration with any community coding skill
- Correction learnings via `sruja agent record` on verify failure

See [skills/sruja-harness/SKILL.md](sruja-harness/SKILL.md) and [docs/HOST_AGENT_INTEGRATION.md](../docs/HOST_AGENT_INTEGRATION.md).

### sruja-architecture

Optional Tier 1b: reviewed `repo.sruja` in version control. Covers architecture design, repo discovery, and `.sruja` generation.

**Install (appears on [skills.sh](https://skills.sh) leaderboard via install telemetry):**

```bash
npx skills add sruja-ai/sruja --skill sruja-architecture
```

**Contents:**

- 50+ rules across 6 categories
- Evidence-based discovery from code and specs
- `.sruja` generation and update workflow
- Architectural principles and patterns
- Component types and guidelines
- Relationship best practices
- Anti-patterns to avoid
- Trade-offs and decision frameworks

**Categories:**

1. Architectural Principles (CRITICAL)
2. Component Types (CRITICAL)
3. Architectural Patterns (HIGH)
4. Relationship Guidelines (HIGH)
5. Anti-Patterns (MEDIUM)
6. Trade-offs & Decisions (MEDIUM)

## Best practices (SKILL.md)

Sruja skills follow the [SKILL.md specification](https://www.mdskills.ai/specs/skill-md) and [skills.sh](https://skills.sh) conventions so AI agents discover and use them correctly:

| Practice | How we apply it |
|----------|------------------|
| **YAML frontmatter** | Every `SKILL.md` has `name`, `description`, and optional `license` / `metadata`. Required for discovery. |
| **Description** | Third-person; states **what** the skill does and **when** to use it (trigger terms: e.g. ".sruja", "architecture", "discover"). |
| **Concise SKILL.md** | Main file under ~500 lines so activation stays within token limits. |
| **Progressive disclosure** | Detail lives in referenced files: `AGENTS.md`, `rules/*.md`, or `REFERENCE.md`. Agent loads full SKILL.md on match, then references as needed. |
| **One-level-deep references** | SKILL.md links to `rules/`, `AGENTS.md`, or `REFERENCE.md` directly; no deep nesting. |

These align with Cursor’s [create-skill](https://cursor.com) guidance and the [agentskills.io](https://agentskills.io) specification.

## Skill Structure

Each skill follows the skills.sh format:

```
skills/
└── sruja-architecture/
    ├── SKILL.md          # Main skill description
    ├── AGENTS.md         # Compiled guide for AI agents
    └── rules/           # Individual rule files
        ├── principle-*.md
        ├── component-*.md
        ├── pattern-*.md
        ├── relationship-*.md
        ├── anti-*.md
        └── tradeoff-*.md
```

## Using Skills

### For AI Agents

The `AGENTS.md` file is a comprehensive guide containing all rules expanded. AI agents can reference this for complete architectural guidance.

### For Developers

Individual rule files in `rules/` provide focused guidance on specific topics. Each rule includes:

- Explanation of why it matters
- When to apply it
- Correct and incorrect examples
- Common mistakes
- Related rules

## Listing on skills.sh

Skills are **listed automatically** on [skills.sh](https://skills.sh): when users run `npx skills add sruja-ai/sruja --skill sruja-architecture`, anonymous telemetry records the install and the skill is ranked on the leaderboard. No separate submission is required. Ensure `SKILL.md` has YAML frontmatter (`name`, `description`) so the directory can display the skill correctly.

## Publishing a new skill

1. Create a directory under `skills/<skill-name>/`.
2. Add `SKILL.md` with YAML frontmatter (`name`, `description`) and a concise body (under ~500 lines). Use `REFERENCE.md` or `AGENTS.md` for long content.
3. Add `AGENTS.md` (compiled guide) and optional `rules/*.md` or `REFERENCE.md`.
4. Commit; users install with `npx skills add sruja-ai/sruja --skill <skill-name>`.

## Related documentation

- [Getting started with skills](../docs/GETTING_STARTED_SKILL.md) – one-page install guide for end users
- [AI Editor Integration](../docs/AI_EDITOR_INTEGRATION.md)
- [.architecture-skill.md](../.architecture-skill.md) (pointer file in repo root)
- [Language specification](../docs/LANGUAGE_SPECIFICATION.md)

## Contributing

To contribute new skills or improve existing ones:

1. Create new rule file following the format
2. Add to appropriate category
3. Update `SKILL.md` with new rule
4. Recompile into `AGENTS.md`
5. Update documentation

## Future Skills

Planned skills for Sruja:

- `sruja-governed-delivery` - Maps community SDLC skills to verify profiles (see [AGENT_DELIVERY_PLAN](../docs/plans/AGENT_DELIVERY_PLAN.md))
- `sruja-validation` - Architecture validation rules and checks
- `sruja-migration` - Patterns for architecture migrations
- `sruja-security` - Security architecture patterns

## Improving the skills

See **[docs/SRUJA_SKILL_IMPROVEMENTS.md](../docs/SRUJA_SKILL_IMPROVEMENTS.md)** for a concrete list of improvements to make the Sruja skill "super awesome": canonical prompts, DSL consistency, lint→fix guidance, scope ladder, and UX.

## Resources

- [Skills.sh Documentation](https://skills.sh/docs)
- [Skills CLI](https://skills.sh/docs/cli)
- [Sruja Documentation](https://sruja.ai)
