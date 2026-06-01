# Sruja product messaging

**Public one-liner:** Sruja scans your repo and reports structural problems AI coding tends to introduce; optional `.sruja` and MCP help your editor stay inside boundaries.

**Do not lead with:** architecture DSL, diagram designer, context engineering platform, or “architecture intelligence.”

---

## Tier 1a — OSS hero (no `.sruja` required)

Deterministic structural scan. Catches what AI changes in repo topology.

```bash
curl -fsSL https://sruja.ai/install.sh | bash
sruja start -r .
sruja drift -r . --structural-only --advisory
```

Always show: **what we found**, **what we could not infer**, **what changed** (on repeat runs). Use `drift -f json` for machines.

**Messaging principles**

- Lead with scan + anti-drift + MCP gates — not skill install or DSL syntax.
- Avoid hero copy: “instant”, “60 seconds”, “zero-config wow”, “intelligence”.
- `.sruja` is a **structured snapshot for viz + optional CI intent**, not day-one requirement.

## Tier 1b — Skill (optional promotion)

Install `sruja-architecture` when you want reviewed `repo.sruja` in version control:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

The skill runs Tier-1 commands (sync, drift, lint) — it is an accelerator, not the product definition.

**Draft vs reviewed:** `repo.sruja.draft` = evidence sketch; `repo.sruja` = reviewed truth. `sruja sync` writes `.sruja/author_evidence.json` for synthesis.

---

## Retrieval ladder (CLI + MCP)

| Step | When | CLI | MCP |
|------|------|-----|-----|
| 1 | Before editing | `sruja focus -r . --file <path>` | `sruja_get_focus_briefing` |
| 2 | Paste-ready brief | `sruja ai -r . --task "…"` | `sruja_get_task_context` |
| 3 | Deep graph | — | Ladder + `sruja_hybrid_query` |
| 4 | Investigation | `sruja why` / `query` | `sruja_query_graph`, etc. |

## MCP tool profiles

- **`coding`** (default, ≤18 tools): ladder, focus, drift, verify, memory search, hybrid query, critique, prune.
- **`minimal`**, **`arch`**, **`full`** (compat only — not in public templates).

Set `SRUJA_MCP_TOOL_PROFILE=coding`. **Not** in coding profile: `sruja_agent_run` — hosts run the LLM loop; Sruja supplies `verify-task` and evidence.

---

## vs Structurizr / LikeC4

See [STRUCTURIZR_VS_SRUJA.md](./STRUCTURIZR_VS_SRUJA.md).

---

## Host loop (not `agent run`)

```text
1. MCP: focus / drift state / boundary context
2. Host LLM edits code
3. sruja confidence -r . -f md          (or verify-task for CI gates)
4. sruja agent record -c "…" on failure (optional)
```

The **confidence report** composes verify-task, intent check, drift, and review into a single human-readable artifact: what changed, what evidence was checked, what risks remain, and what to inspect at 3AM.

`sruja agent run` is internal/deprecated — use `agent plan` + host apply + `confidence`.

---

## Three pillars (technical docs)

- **Grounding** — scan/graph evidence in prompts
- **Validation** — lint, drift, verify-task
- **Persistence** — optional reviewed `repo.sruja`

**Pitch line:** Sruja gives the model your real topology and validates what it produces.

See [FEATURE_TIERS.md](./FEATURE_TIERS.md) and [OSS_METRICS.md](./OSS_METRICS.md).
