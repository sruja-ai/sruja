# Sruja Skills

This directory contains skills formatted for the [skills.sh](https://skills.sh) platform - an open agent skills ecosystem.

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
2. Add `SKILL.md` with YAML frontmatter (`name`, `description`) and body.
3. Add `AGENTS.md` (compiled guide) and optional `rules/*.md`.
4. Commit; users install with `npx skills add sruja-ai/sruja --skill <skill-name>`.

## Related documentation

- [AI Editor Integration](../docs/AI_EDITOR_INTEGRATION.md)
- [.architecture-skill.md](../.architecture-skill.md) (pointer file)
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

## Resources

- [Skills.sh Documentation](https://skills.sh/docs)
- [Skills CLI](https://skills.sh/docs/cli)
- [Sruja Documentation](https://sruja.ai)
