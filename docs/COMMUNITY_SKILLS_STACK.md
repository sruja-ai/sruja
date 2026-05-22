# Community Skills Stack

How to combine Sruja with community skills from agentskills.io, skills.sh, and custom skills.

---

## Recommended Stack

| Skill | Purpose | Install |
|-------|---------|---------|
| `sruja-harness` | Verification adapter (run before done) | `npx skills add https://github.com/sruja-ai/sruja --skill sruja-harness` |
| `sruja-architecture` | Architecture discovery + DSL authoring (optional) | `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture` |
| Addy/skills.sh skills | Coding, bugfix, review, debug | `npx skills add <name>` |

**No false framing:** Community skills do NOT replace Sruja. They generate code; Sruja validates it against architecture truth.

---

## agentskills.io Format

Skills use YAML frontmatter:

```yaml
---
name: my-skill
description: Short description of what this skill does
license: MIT
---
```

Both `sruja-architecture` and `sruja-harness` follow this format for compatibility with `npx skills add`.

---

## Install for Cursor

```bash
# Install skills to your project (harness first)
npx skills add https://github.com/sruja-ai/sruja --skill sruja-harness
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
npx skills add <community-skill>
```

Skills install to `.agents/skills/` (or your configured skills directory).

---

## skills.sh Search Tips

```bash
# Search for skills
npx skills search "typescript"
npx skills search "react"
npx skills search "debug"

# Install a skill
npx skills install <skill-name>
```

---

## Conflict Avoidance

| Skill | Use When | Avoid With |
|-------|----------|------------|
| `interview-me` | Exploratory requirements gathering | `grill-me` (both ask questions; pick one) |
| `grill-me` | Deep technical interrogation | `interview-me` |
| `sruja-harness` | Always (verification adapter) | Nothing — it composes with everything |

**Rule:** Only one "question-asking" skill active at a time. `sruja-harness` is always safe to combine.

---

## Workflow: Skill + Harness

```
Community Skill (generates code)
    ↓
sruja-harness (verifies)
    ↓
verify-task passes?
    ├── Yes → record_learning (affirmation) → done
    └── No → fix → re-verify → record_learning (correction)
```

---

## Custom Skills

To create a custom skill that integrates with Sruja:

1. Create `skills/my-skill/SKILL.md` with agentskills.io frontmatter
2. Reference Sruja commands in your workflow steps
3. Always end with `sruja verify-task` before marking done

Example:

```markdown
---
name: my-feature-builder
description: Builds new features with Sruja verification
---

# My Feature Builder

1. Implement the feature
2. Run `sruja verify-task --profile coding -r .`
3. If failed, fix and re-verify
4. If passed, mark done
```

---

## References

- [docs/HOST_AGENT_INTEGRATION.md](HOST_AGENT_INTEGRATION.md) — Integration contract
- [docs/plans/AGENT_DELIVERY_PLAN.md](plans/AGENT_DELIVERY_PLAN.md) — Delivery roadmap
- [skills/sruja-architecture/SKILL.md](../skills/sruja-architecture/SKILL.md) — Architecture skill
- [skills/sruja-harness/SKILL.md](../skills/sruja-harness/SKILL.md) — Harness skill
