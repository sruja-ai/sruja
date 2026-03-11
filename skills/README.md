# Sruja Skills

This directory contains skills for **AI code editors** (Cursor, Codex, and others) and the [skills.sh](https://skills.sh) ecosystem. Install a skill so your AI assistant generates valid Sruja DSL and applies architectural patterns.

**No MCP server** — editor integration is **skills + CLI** only. There is no `sruja mcp` in this repo.

## Install in your editor

**One command (Cursor, Codex, skills.sh):**

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

**Full guide** – [docs/INSTALL_AS_SKILL.md](../docs/INSTALL_AS_SKILL.md) – editor-specific steps, Cursor options, and which skill to choose.

**Repo pointer** – [.architecture-skill.md](../.architecture-skill.md) in the repo root has the same one-command install and links.

## Available Skills

### sruja-architecture

Comprehensive guide for software architecture design using Sruja DSL.

**Install (appears on [skills.sh](https://skills.sh) leaderboard via install telemetry):**

```bash
npx skills add sruja-ai/sruja --skill sruja-architecture
```

**Contents:**

- 50+ rules across 6 categories
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

### sruja-architecture-agent

AI-powered architecture discovery skill for analyzing codebases and generating Sruja DSL.

**Install:**

```bash
npx skills add sruja-ai/sruja --skill sruja-architecture-agent
```

**Contents:**

- Codebase analysis patterns
- Technology detection
- Dependency discovery
- Sruja DSL generation
- Multi-repo support

### sruja-architecture-collaboration

Collaborative architecture intelligence with multi-agent teams, knowledge graphs, and review workflows.

**Install:**

```bash
npx skills add sruja-ai/sruja --skill sruja-architecture-collaboration
```

**Contents:**

- Multi-agent team roles (Analyst, Architect, Reviewer, Validator, Facilitator)
- Collaboration workflows (review cycles, live sessions)
- Knowledge management (pattern library, ADRs, traceability)
- CI/CD integration for architecture reviews

**Categories:**

1. Agent Roles (5 specialized roles)
2. Collaboration Workflows (review, session)
3. Knowledge Graph (patterns, decisions, traceability)

## Best practices (SKILL.md)

Sruja skills follow the [SKILL.md / agent skills](https://www.mdskills.ai/specs/skill-md) and [skills.sh](https://skills.sh) conventions so agents discover and use them correctly:

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

- [Install as skill](../docs/INSTALL_AS_SKILL.md) – one-page install guide for end users
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

- `sruja-validation` - Architecture validation rules and checks
- `sruja-migration` - Patterns for architecture migrations
- `sruja-security` - Security architecture patterns

## Skill Dependencies

Some skills depend on others for full functionality:

```
sruja-architecture-collaboration
  ├── sruja-architecture (design principles)
  └── sruja-architecture-agent (discovery)
```

Install dependencies first for best results.

## Improving the skills

See **[docs/SRUJA_SKILL_IMPROVEMENTS.md](../docs/SRUJA_SKILL_IMPROVEMENTS.md)** for a concrete list of improvements to make the Sruja skill "super awesome": canonical prompts, DSL consistency, lint→fix guidance, scope ladder, and UX.

## Resources

- [Skills.sh Documentation](https://skills.sh/docs)
- [Skills CLI](https://skills.sh/docs/cli)
- [Sruja Documentation](https://sruja.ai)