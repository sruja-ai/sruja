---
name: sruja-harness
description: >
  Verification harness for any AI agent skill. Use with any other skills; before marking
  a task done, run verify-task. On failure, record a correction learning. Ensures lint,
  tests, drift, and intent checks pass regardless of which skill generated the code.
license: Apache-2.0
---

# Sruja Harness Skill

Universal verification adapter — works with any coding, bugfix, or review skill.

## Quick Start

1. **Install alongside other skills:**
   ```bash
   npx skills add https://github.com/sruja-ai/sruja --skill sruja-harness
   npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
   ```

2. **Use with any skill:** This skill does not generate code. It verifies output from other skills.

3. **Before marking a task done:**
   ```bash
   sruja verify-task --profile coding -r .
   ```

4. **On verify failure:**
   ```bash
   sruja agent record -r . \
     -c "<task context>" \
     -H "<what was tried>" \
     -o failed \
     -g "<correction for next time>"
   ```

## Rules

- **Always verify before done** — Run `sruja verify-task` with the appropriate profile before declaring a task complete.
- **Profile selection:**
  - `coding` — New features, refactors (default)
  - `bugfix` — Bug fixes (use `--file` to target the bug location)
  - `review` — Pre-merge review of changes
  - `arch` — Architecture/DSL changes
- **On failure:** Record a correction learning before retrying.
- **On success:** Optionally record an affirmation learning ("what worked").

## Integration with Other Skills

```
Coding Skill (e.g. /build) → sruja-harness verify → record_learning → done
```

This skill does not replace coding skills — it ensures their output passes Sruja's architecture gates.

## Profiles

| Profile | Verifies |
|---------|----------|
| `coding` | `lint repo.sruja` + `make check` + `drift` |
| `bugfix` | `focus --file` + `make check` + `intent check` |
| `review` | `review -f json` + `intent check` + `drift` |
| `arch` | `lint repo.sruja` + `drift` + `intent check` + `review` |

## References

- [docs/HOST_AGENT_INTEGRATION.md](../../docs/HOST_AGENT_INTEGRATION.md) — Full integration contract
- [docs/plans/AGENT_DELIVERY_PLAN.md](../../docs/plans/AGENT_DELIVERY_PLAN.md) — Delivery roadmap
